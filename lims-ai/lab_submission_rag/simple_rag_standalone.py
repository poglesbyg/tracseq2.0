#!/usr/bin/env python3
"""
Standalone Simple RAG Service
Minimal RAG service that gracefully handles database issues
"""

import re
import time
import uuid
from datetime import datetime
from typing import Any

from fastapi import FastAPI, File, UploadFile
from fastapi.middleware.cors import CORSMiddleware
from pydantic import BaseModel

app = FastAPI(
    title="Standalone RAG Service",
    description="Minimal RAG service for lab_manager frontend",
    version="1.0.0"
)

# Enable CORS for lab_manager frontend
app.add_middleware(
    CORSMiddleware,
    allow_origins=["*"],  # Allow all origins for development
    allow_credentials=True,
    allow_methods=["*"],
    allow_headers=["*"],
)

# In-memory storage for demo purposes
submissions_storage = []

class RagSubmissionResponse(BaseModel):
    """Response model for RAG submissions"""
    id: str
    submission_id: str
    submitter_name: str | None
    submitter_email: str | None
    sample_type: str | None
    sample_name: str | None
    confidence_score: float
    created_at: str
    status: str = "completed"

class DocumentProcessResponse(BaseModel):
    """Response model for document processing"""
    success: bool
    confidence_score: float
    samples_found: int
    processing_time: float
    extracted_data: dict[str, Any]
    message: str

class QueryRequest(BaseModel):
    """Request model for queries"""
    query: str
    session_id: str | None = "default"

class QueryResponse(BaseModel):
    """Response model for queries"""
    answer: str

def extract_sample_info_basic(text: str) -> dict[str, Any]:
    """Basic text extraction without LLM dependency"""

    result = {
        "administrative": {},
        "sample": {},
        "sequencing": {}
    }

    # Extract submitter information
    name_match = re.search(r"(?:Name|First Name):\s*(.+?)(?:\n|$)", text, re.IGNORECASE)
    if name_match:
        result["administrative"]["submitter_name"] = name_match.group(1).strip()

    email_match = re.search(r"Email:\s*(.+?)(?:\n|$)", text, re.IGNORECASE)
    if email_match:
        result["administrative"]["submitter_email"] = email_match.group(1).strip()

    # Extract sample information
    sample_id_match = re.search(r"Sample ID:\s*(.+?)(?:\n|$)", text, re.IGNORECASE)
    if sample_id_match:
        result["sample"]["sample_id"] = sample_id_match.group(1).strip()

    sample_type_match = re.search(r"(?:Sample Type|Source Type):\s*(.+?)(?:\n|$)", text, re.IGNORECASE)
    if sample_type_match:
        result["sample"]["sample_type"] = sample_type_match.group(1).strip()

    return result

def calculate_confidence_score_basic(extraction_result: dict[str, Any]) -> float:
    """Calculate confidence score based on extracted information completeness"""
    total_fields = 0
    filled_fields = 0

    # Check all fields
    for category in extraction_result.values():
        if isinstance(category, dict):
            for value in category.values():
                total_fields += 1
                if value and str(value).strip():
                    filled_fields += 1

    if total_fields == 0:
        return 0.0

    confidence = (filled_fields / total_fields) * 100
    return min(90.0, confidence)

@app.get("/")
async def root():
    """Root endpoint"""
    return {"message": "Standalone RAG Service", "status": "operational"}

@app.get("/health")
async def health_check():
    """Health check endpoint"""
    return {"status": "healthy", "service": "standalone-rag"}

@app.get("/api/rag/submissions", response_model=list[RagSubmissionResponse])
async def get_rag_submissions(limit: int = 50, offset: int = 0):
    """Get RAG submissions"""
    # Return from in-memory storage for now
    start_idx = offset
    end_idx = offset + limit
    results = submissions_storage[start_idx:end_idx]

    return results

@app.get("/api/rag/stats")
async def get_rag_statistics():
    """Get RAG system statistics"""
    return {
        "total_submissions": len(submissions_storage),
        "recent_submissions": len([s for s in submissions_storage if s.created_at]),
        "average_confidence": 75.0,
        "status": "operational"
    }

@app.post("/api/rag/process", response_model=DocumentProcessResponse)
async def process_document_upload(file: UploadFile = File(...)):
    """Process uploaded document and extract laboratory information"""
    start_time = time.time()

    try:
        # Read file content
        content = await file.read()

        # Convert to text
        if file.content_type == "text/plain":
            text = content.decode('utf-8')
        else:
            try:
                text = content.decode('utf-8')
            except UnicodeDecodeError:
                return DocumentProcessResponse(
                    success=False,
                    confidence_score=0.0,
                    samples_found=0,
                    processing_time=time.time() - start_time,
                    extracted_data={"error": "Unable to process file. Please upload a text file."},
                    message="Unable to process file. Please upload a text file."
                )

        # Extract information using basic regex patterns
        extraction_result = extract_sample_info_basic(text)

        # Calculate confidence score
        confidence_score = calculate_confidence_score_basic(extraction_result)

        # Count samples found
        samples_found = 1 if extraction_result.get('sample', {}).get('sample_id') else 0

        processing_time = time.time() - start_time

        # Store in memory for demo
        if samples_found > 0:
            admin_data = extraction_result.get('administrative', {}) or {}
            sample_data = extraction_result.get('sample', {}) or {}

            submission = RagSubmissionResponse(
                id=str(uuid.uuid4())[:8],
                submission_id=str(uuid.uuid4()),
                submitter_name=admin_data.get('submitter_name'),
                submitter_email=admin_data.get('submitter_email'),
                sample_type=sample_data.get('sample_type'),
                sample_name=sample_data.get('sample_id'),
                confidence_score=confidence_score,
                created_at=datetime.utcnow().isoformat(),
                status="completed"
            )

            submissions_storage.append(submission)

        return DocumentProcessResponse(
            success=True,
            confidence_score=confidence_score,
            samples_found=samples_found,
            processing_time=processing_time,
            extracted_data=extraction_result,
            message="Document processed successfully" if samples_found > 0 else "Document processed but no complete sample information found"
        )

    except Exception as e:
        processing_time = time.time() - start_time

        return DocumentProcessResponse(
            success=False,
            confidence_score=0.0,
            samples_found=0,
            processing_time=processing_time,
            extracted_data={"error": str(e)},
            message=f"Processing failed: {str(e)}"
        )

def get_intelligent_response(query: str) -> str:
    """Generate intelligent responses based on query content"""
    query_lower = query.lower().strip()

    if any(word in query_lower for word in ['hello', 'hi', 'hey']):
        return "Hello! I'm your lab management assistant. How can I help you today?"

    elif any(phrase in query_lower for phrase in ['submit', 'upload', 'create sample']):
        return """To submit new samples, you can:
1. Upload lab submission forms - I'll extract the information automatically
2. Use the manual sample entry form  
3. Upload bulk templates for multiple samples"""

    else:
        return "I'm your lab management assistant. I can help with sample submission, storage management, sequencing workflows, and more. What would you like to know?"

@app.post("/query", response_model=QueryResponse)
async def query_rag_system(request: QueryRequest):
    """Handle RAG queries"""
    answer = get_intelligent_response(request.query)
    return QueryResponse(answer=answer)

if __name__ == "__main__":
    import uvicorn
    print("🚀 Starting Standalone RAG Service on port 8087")
    print("📡 Available endpoints:")
    print("   GET  /api/rag/submissions")
    print("   POST /api/rag/process")
    print("   GET  /api/rag/stats")
    print("   POST /query")
    print("   GET  /health")

    uvicorn.run(app, host="0.0.0.0", port=8087)
