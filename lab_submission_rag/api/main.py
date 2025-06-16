from fastapi import FastAPI, UploadFile, File, HTTPException
from fastapi.middleware.cors import CORSMiddleware
from pydantic import BaseModel
from typing import List, Optional, Dict, Any
import os
import shutil
from pathlib import Path
import json

# Import the actual RAG system
from rag_orchestrator import rag_system
from models.submission import LabSubmission, ExtractionResult

app = FastAPI(
    title="Laboratory Submission RAG API",
    description="API for processing laboratory submissions using RAG",
    version="1.0.0"
)

# Configure CORS
app.add_middleware(
    CORSMiddleware,
    allow_origins=["*"],
    allow_credentials=True,
    allow_methods=["*"],
    allow_headers=["*"],
)

@app.on_event("startup")
async def startup_event():
    """Initialize database on startup"""
    try:
        await rag_system.initialize_database()
    except Exception as e:
        print(f"Warning: Failed to initialize database: {e}")
        # Continue startup even if database initialization fails

# Use the actual RAG system
# rag_system is already initialized in rag_orchestrator.py

class QueryRequest(BaseModel):
    query: str
    submission_id: Optional[str] = None
    session_id: Optional[str] = "default"
    k: Optional[int] = 5

# Updated response models to match Rust expectations
class RagExtractionResult(BaseModel):
    success: bool
    submission: Optional[Dict[str, Any]] = None
    confidence_score: float
    missing_fields: List[str] = []
    warnings: List[str] = []
    processing_time: float
    source_document: str

class QueryResponse(BaseModel):
    answer: str

# Updated endpoints to match Rust expectations
@app.post("/process-document", response_model=RagExtractionResult)
async def process_document_and_create_samples(file: UploadFile = File(...)):
    """Process a laboratory submission document - matches Rust endpoint expectation"""
    try:
        # Create uploads directory if it doesn't exist
        upload_dir = Path("uploads")
        upload_dir.mkdir(exist_ok=True)
        
        # Save uploaded file
        file_path = upload_dir / file.filename
        with file_path.open("wb") as buffer:
            shutil.copyfileobj(file.file, buffer)
        
        # Process the submission using RAG system
        result = await rag_system.process_document(str(file_path))
        
        # Convert ExtractionResult to dictionary format expected by frontend
        response_dict = {
            "success": result.success,
            "confidence_score": result.confidence_score or 0.0,
            "missing_fields": result.missing_fields or [],
            "warnings": result.warnings or [],
            "processing_time": result.processing_time,
            "source_document": result.source_document
        }
        
        # Add submission data if extraction was successful
        if result.success and result.submission:
            response_dict["submission"] = result.submission.dict()
        
        return RagExtractionResult(**response_dict)
    except Exception as e:
        raise HTTPException(status_code=500, detail=str(e))

@app.post("/query", response_model=QueryResponse)
async def query_submission_information(request: QueryRequest):
    """Query the RAG system with a specific question - matches Rust endpoint expectation"""
    try:
        answer = await rag_system.query_submissions(
            query=request.query,
            filter_metadata={"submission_id": request.submission_id} if request.submission_id else None,
            session_id=request.session_id or "default"
        )
        return QueryResponse(answer=answer)
    except Exception as e:
        raise HTTPException(status_code=500, detail=str(e))

@app.get("/health")
async def health_check():
    """Health check endpoint."""
    return {"status": "healthy"}

@app.get("/system-info")
async def get_system_info():
    """Get information about the RAG system."""
    try:
        info = await rag_system.get_system_status()
        return info
    except Exception as e:
        raise HTTPException(status_code=500, detail=str(e))

# Legacy endpoint for backward compatibility
@app.post("/process")
async def process_submission_legacy(file: UploadFile = File(...)):
    """Legacy endpoint - redirects to new format"""
    return await process_document_and_create_samples(file)

@app.get("/samples/count")
async def get_sample_count(
    sample_type: Optional[str] = None,
    storage_condition: Optional[str] = None
):
    """Get count of samples with optional filtering"""
    try:
        from database import db_manager
        from repositories.submission_repository import SubmissionRepository
        
        async with db_manager.get_session() as session:
            repo = SubmissionRepository(session)
            count = await repo.get_sample_count(
                sample_type=sample_type,
                storage_condition=storage_condition
            )
            return {"count": count, "filters": {"sample_type": sample_type, "storage_condition": storage_condition}}
    except Exception as e:
        raise HTTPException(status_code=500, detail=str(e))

@app.get("/samples/statistics")
async def get_sample_statistics():
    """Get comprehensive sample statistics"""
    try:
        from database import db_manager
        from repositories.submission_repository import SubmissionRepository
        
        async with db_manager.get_session() as session:
            repo = SubmissionRepository(session)
            stats = await repo.get_sample_statistics()
            return stats
    except Exception as e:
        raise HTTPException(status_code=500, detail=str(e))

@app.get("/samples/search")
async def search_samples(q: str, limit: int = 50):
    """Search samples by various fields"""
    try:
        from database import db_manager
        from repositories.submission_repository import SubmissionRepository
        
        async with db_manager.get_session() as session:
            repo = SubmissionRepository(session)
            samples = await repo.search_samples(q, limit)
            
            # Convert to dict format for JSON response
            results = []
            for sample in samples:
                results.append({
                    "sample_id": sample.sample_id,
                    "sample_name": sample.sample_name,
                    "patient_id": sample.patient_id,
                    "source_type": sample.source_type.value if sample.source_type else None,
                    "storage_conditions": sample.storage_conditions,
                    "created_at": sample.created_at.isoformat() if sample.created_at else None
                })
            
            return {"results": results, "count": len(results), "query": q}
    except Exception as e:
        raise HTTPException(status_code=500, detail=str(e))

@app.get("/submissions")
async def get_submissions(
    limit: int = 100,
    offset: int = 0,
    client_id: Optional[str] = None,
    status: Optional[str] = None,
    sample_type: Optional[str] = None
):
    """Get submissions with optional filtering"""
    try:
        from database import db_manager
        from repositories.submission_repository import SubmissionRepository
        
        async with db_manager.get_session() as session:
            repo = SubmissionRepository(session)
            submissions = await repo.get_submissions(
                limit=limit,
                offset=offset,
                client_id=client_id,
                status=status,
                sample_type=sample_type
            )
            
            # Convert to dict format for JSON response
            results = []
            for submission in submissions:
                results.append({
                    "submission_id": submission.submission_id,
                    "client_name": submission.client_name,
                    "client_email": submission.client_email,
                    "sample_type": submission.sample_type.value if submission.sample_type else None,
                    "sample_count": submission.sample_count,
                    "status": submission.status.value if submission.status else None,
                    "submission_date": submission.submission_date.isoformat() if submission.submission_date else None,
                    "created_at": submission.created_at.isoformat() if submission.created_at else None
                })
            
            return {"results": results, "count": len(results), "limit": limit, "offset": offset}
    except Exception as e:
        raise HTTPException(status_code=500, detail=str(e))

@app.get("/database/status")
async def get_database_status():
    """Get database connection status and basic statistics"""
    try:
        from database import db_manager
        from repositories.submission_repository import SubmissionRepository
        
        async with db_manager.get_session() as session:
            repo = SubmissionRepository(session)
            
            # Get basic counts
            submissions = await repo.get_submissions(limit=1000)
            submission_count = len(submissions)
            
            stats = await repo.get_sample_statistics()
            sample_count = stats.get("total_samples", 0)
            
            return {
                "status": "connected",
                "database_initialized": rag_system._database_initialized,
                "total_submissions": submission_count,
                "total_samples": sample_count,
                "sample_breakdown": stats
            }
    except Exception as e:
        return {
            "status": "error",
            "error": str(e),
            "database_initialized": getattr(rag_system, '_database_initialized', False)
        } 
