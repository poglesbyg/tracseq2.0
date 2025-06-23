#!/usr/bin/env python3
"""
Frontend API Bridge for RAG Submissions
Provides API endpoints that the lab_manager frontend can call to display RAG submissions
"""

import json
import uuid
from datetime import datetime
from pathlib import Path
from typing import List, Optional

import asyncpg
from fastapi import FastAPI, File, HTTPException, UploadFile
from fastapi.middleware.cors import CORSMiddleware
from pydantic import BaseModel

# Import our fixed RAG system
from fixed_improved_rag import process_document_fixed

app = FastAPI(
    title="RAG Submissions API Bridge",
    description="API bridge for lab_manager frontend to access RAG submissions",
    version="1.0.0",
)

# Enable CORS for lab_manager frontend
app.add_middleware(
    CORSMiddleware,
    allow_origins=["http://localhost:8080", "http://localhost:3001"],
    allow_credentials=True,
    allow_methods=["*"],
    allow_headers=["*"],
)

# Database connection
DB_CONFIG = {
    "host": "localhost",
    "port": 5433,
    "database": "lab_manager",
    "user": "postgres",
    "password": "postgres",
}


class RagSubmissionResponse(BaseModel):
    """Response model for RAG submissions"""

    id: str
    submission_id: str
    submitter_name: Optional[str]
    submitter_email: Optional[str]
    sample_type: Optional[str]
    sample_name: Optional[str]
    confidence_score: float
    created_at: str
    status: str = "completed"


class ProcessingResult(BaseModel):
    """Result of document processing"""

    success: bool
    submission_id: Optional[str] = None
    message: str
    processing_time: float = 0.0


async def get_db_connection():
    """Get database connection"""
    return await asyncpg.connect(**DB_CONFIG)


@app.get("/")
async def root():
    """Root endpoint"""
    return {"message": "RAG Submissions API Bridge", "status": "operational"}


@app.get("/health")
async def health_check():
    """Health check endpoint"""
    try:
        conn = await get_db_connection()
        await conn.close()
        return {"status": "healthy", "database": "connected"}
    except Exception as e:
        raise HTTPException(status_code=503, detail=f"Database connection failed: {e}")


@app.get("/api/rag/submissions", response_model=List[RagSubmissionResponse])
async def get_rag_submissions(limit: int = 50, offset: int = 0):
    """Get RAG submissions for the frontend"""
    try:
        conn = await get_db_connection()

        # Query RAG submissions from database
        submissions = await conn.fetch(
            """
            SELECT 
                submission_id,
                submitter_name,
                submitter_email,
                sample_type,
                document_name,
                confidence_score,
                created_at,
                extracted_data
            FROM rag_submissions 
            ORDER BY created_at DESC 
            LIMIT $1 OFFSET $2
        """,
            limit,
            offset,
        )

        await conn.close()

        # Convert to response format
        result = []
        for row in submissions:
            result.append(
                RagSubmissionResponse(
                    id=str(uuid.uuid4())[:8],  # Short ID for display
                    submission_id=row["submission_id"],
                    submitter_name=row["submitter_name"],
                    submitter_email=row["submitter_email"],
                    sample_type=row["sample_type"] or "Unknown",
                    sample_name=row["document_name"],
                    confidence_score=row["confidence_score"] or 0.0,
                    created_at=row["created_at"].isoformat() if row["created_at"] else "",
                    status="completed",
                )
            )

        return result

    except Exception as e:
        raise HTTPException(status_code=500, detail=f"Failed to fetch submissions: {e}")


@app.get("/api/rag/submissions/{submission_id}")
async def get_rag_submission_details(submission_id: str):
    """Get detailed RAG submission information"""
    try:
        conn = await get_db_connection()

        submission = await conn.fetchrow(
            """
            SELECT * FROM rag_submissions 
            WHERE submission_id = $1
        """,
            submission_id,
        )

        await conn.close()

        if not submission:
            raise HTTPException(status_code=404, detail="Submission not found")

        # Parse extracted data
        extracted_data = (
            json.loads(submission["extracted_data"]) if submission["extracted_data"] else {}
        )

        return {
            "submission_id": submission["submission_id"],
            "submitter_name": submission["submitter_name"],
            "submitter_email": submission["submitter_email"],
            "sample_type": submission["sample_type"],
            "document_name": submission["document_name"],
            "confidence_score": submission["confidence_score"],
            "created_at": submission["created_at"].isoformat() if submission["created_at"] else "",
            "source_document": submission["source_document"],
            "extracted_data": extracted_data,
        }

    except HTTPException:
        raise
    except Exception as e:
        raise HTTPException(status_code=500, detail=f"Failed to fetch submission details: {e}")


@app.post("/api/rag/process", response_model=ProcessingResult)
async def process_document_via_api(file: UploadFile = File(...)):
    """Process a document using our fixed RAG system"""
    try:
        # Save uploaded file temporarily
        temp_dir = Path("temp_uploads")
        temp_dir.mkdir(exist_ok=True)

        file_path = temp_dir / f"{uuid.uuid4()}_{file.filename}"

        with open(file_path, "wb") as buffer:
            content = await file.read()
            buffer.write(content)

        # Process with our fixed system
        start_time = datetime.now()
        result = await process_document_fixed(str(file_path))
        processing_time = (datetime.now() - start_time).total_seconds()

        # Clean up temp file
        file_path.unlink()

        if result.success:
            return ProcessingResult(
                success=True,
                submission_id=result.submission.source_document,
                message=f"Successfully processed {file.filename}",
                processing_time=processing_time,
            )
        else:
            return ProcessingResult(
                success=False,
                message=f"Processing failed: {result.warnings}",
                processing_time=processing_time,
            )

    except Exception as e:
        return ProcessingResult(
            success=False, message=f"Error processing document: {e}", processing_time=0.0
        )


@app.get("/api/rag/stats")
async def get_rag_statistics():
    """Get RAG system statistics"""
    try:
        conn = await get_db_connection()

        # Get submission counts
        total_submissions = await conn.fetchval("SELECT COUNT(*) FROM rag_submissions")

        # Get recent activity
        recent_count = await conn.fetchval(
            """
            SELECT COUNT(*) FROM rag_submissions 
            WHERE created_at >= NOW() - INTERVAL '7 days'
        """
        )

        # Get average confidence
        avg_confidence = await conn.fetchval(
            """
            SELECT AVG(confidence_score) FROM rag_submissions 
            WHERE confidence_score > 0
        """
        )

        await conn.close()

        return {
            "total_submissions": total_submissions or 0,
            "recent_submissions": recent_count or 0,
            "average_confidence": float(avg_confidence or 0.0),
            "status": "operational",
        }

    except Exception as e:
        raise HTTPException(status_code=500, detail=f"Failed to get statistics: {e}")


@app.delete("/api/rag/submissions/{submission_id}")
async def delete_rag_submission(submission_id: str):
    """Delete a RAG submission"""
    try:
        conn = await get_db_connection()

        result = await conn.execute(
            """
            DELETE FROM rag_submissions 
            WHERE submission_id = $1
        """,
            submission_id,
        )

        await conn.close()

        if result == "DELETE 0":
            raise HTTPException(status_code=404, detail="Submission not found")

        return {"message": "Submission deleted successfully"}

    except HTTPException:
        raise
    except Exception as e:
        raise HTTPException(status_code=500, detail=f"Failed to delete submission: {e}")


# Startup event to test database connection
@app.on_event("startup")
async def startup_event():
    """Test database connection on startup"""
    try:
        conn = await get_db_connection()
        await conn.close()
        print("✅ Database connection successful")
    except Exception as e:
        print(f"❌ Database connection failed: {e}")


if __name__ == "__main__":
    import uvicorn

    print("🚀 Starting RAG Submissions API Bridge")
    print("📡 Frontend will be able to access RAG data at:")
    print("   GET  /api/rag/submissions")
    print("   GET  /api/rag/submissions/{id}")
    print("   POST /api/rag/process")
    print("   GET  /api/rag/stats")
    print("🌐 CORS enabled for lab_manager frontend")

    uvicorn.run(app, host="0.0.0.0", port=3002)
