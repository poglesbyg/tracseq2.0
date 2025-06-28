#!/usr/bin/env python3
"""
Simple Frontend API Bridge for RAG Submissions - Working File Upload Version
Provides basic API endpoints that the lab_manager frontend needs
"""

import json
import re
import time
import uuid
from datetime import datetime
from typing import Any

import asyncpg
from fastapi import FastAPI, File, HTTPException, UploadFile
from fastapi.middleware.cors import CORSMiddleware
from pydantic import BaseModel

app = FastAPI(
    title="Simple RAG Submissions API Bridge",
    description="Basic API bridge for lab_manager frontend to access RAG submissions",
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

# Database connection
DB_CONFIG = {
    'host': 'localhost',
    'port': 5433,
    'database': 'lab_manager',
    'user': 'postgres',
    'password': 'postgres'
}

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

class SampleInfo(BaseModel):
    """Sample information extracted from document"""
    sample_id: str | None = None
    sample_name: str | None = None
    sample_type: str | None = None
    concentration: str | None = None
    volume: str | None = None
    storage_conditions: str | None = None

class SubmitterInfo(BaseModel):
    """Submitter information extracted from document"""
    name: str | None = None
    email: str | None = None
    phone: str | None = None
    institution: str | None = None
    project_name: str | None = None

class SequencingInfo(BaseModel):
    """Sequencing information extracted from document"""
    platform: str | None = None
    analysis_type: str | None = None
    coverage: str | None = None
    read_length: str | None = None

class DocumentProcessResponse(BaseModel):
    """Response model for document processing"""
    success: bool
    confidence_score: float
    samples_found: int
    processing_time: float
    extracted_data: dict[str, Any]
    submitter_info: SubmitterInfo
    sample_info: SampleInfo
    sequencing_info: SequencingInfo
    message: str

async def get_db_connection():
    """Get database connection"""
    return await asyncpg.connect(**DB_CONFIG)

def extract_sample_info_basic(text: str) -> dict[str, Any]:
    """Basic text extraction without LLM dependency"""

    # Initialize result structure
    result = {
        "administrative": {},
        "sample": {},
        "sequencing": {}
    }

    # Extract submitter information
    name_match = re.search(r"Name:\s*(.+?)(?:\n|$)", text, re.IGNORECASE)
    if name_match:
        result["administrative"]["submitter_name"] = name_match.group(1).strip()

    email_match = re.search(r"Email:\s*(.+?)(?:\n|$)", text, re.IGNORECASE)
    if email_match:
        result["administrative"]["submitter_email"] = email_match.group(1).strip()

    phone_match = re.search(r"Phone:\s*(.+?)(?:\n|$)", text, re.IGNORECASE)
    if phone_match:
        result["administrative"]["submitter_phone"] = phone_match.group(1).strip()

    institution_match = re.search(r"Institution:\s*(.+?)(?:\n|$)", text, re.IGNORECASE)
    if institution_match:
        result["administrative"]["institution"] = institution_match.group(1).strip()

    project_match = re.search(r"Project:\s*(.+?)(?:\n|$)", text, re.IGNORECASE)
    if project_match:
        result["administrative"]["project_name"] = project_match.group(1).strip()

    # Extract sample information
    sample_id_match = re.search(r"Sample ID:\s*(.+?)(?:\n|$)", text, re.IGNORECASE)
    if sample_id_match:
        result["sample"]["sample_id"] = sample_id_match.group(1).strip()

    sample_type_match = re.search(r"Sample Type:\s*(.+?)(?:\n|$)", text, re.IGNORECASE)
    if sample_type_match:
        result["sample"]["sample_type"] = sample_type_match.group(1).strip()

    concentration_match = re.search(r"Concentration:\s*(.+?)(?:\n|$)", text, re.IGNORECASE)
    if concentration_match:
        result["sample"]["concentration"] = concentration_match.group(1).strip()

    volume_match = re.search(r"Volume:\s*(.+?)(?:\n|$)", text, re.IGNORECASE)
    if volume_match:
        result["sample"]["volume"] = volume_match.group(1).strip()

    storage_match = re.search(r"Storage Temperature:\s*(.+?)(?:\n|$)", text, re.IGNORECASE)
    if storage_match:
        result["sample"]["storage_conditions"] = storage_match.group(1).strip()

    # Extract sequencing information
    platform_match = re.search(r"Platform:\s*(.+?)(?:\n|$)", text, re.IGNORECASE)
    if platform_match:
        result["sequencing"]["platform"] = platform_match.group(1).strip()

    analysis_match = re.search(r"Analysis Type:\s*(.+?)(?:\n|$)", text, re.IGNORECASE)
    if analysis_match:
        result["sequencing"]["analysis_type"] = analysis_match.group(1).strip()

    coverage_match = re.search(r"Target Coverage:\s*(.+?)(?:\n|$)", text, re.IGNORECASE)
    if coverage_match:
        result["sequencing"]["coverage"] = coverage_match.group(1).strip()

    read_length_match = re.search(r"Read Length:\s*(.+?)(?:\n|$)", text, re.IGNORECASE)
    if read_length_match:
        result["sequencing"]["read_length"] = read_length_match.group(1).strip()

    return result

def calculate_confidence_score_basic(extraction_result: dict[str, Any]) -> float:
    """Calculate confidence score based on extracted information completeness"""
    total_fields = 0
    filled_fields = 0

    # Check administrative fields
    admin = extraction_result.get('administrative', {}) or {}
    admin_fields = ['submitter_name', 'submitter_email', 'project_name', 'institution']
    for field in admin_fields:
        total_fields += 1
        if admin.get(field):
            filled_fields += 1

    # Check sample fields
    sample = extraction_result.get('sample', {}) or {}
    sample_fields = ['sample_id', 'sample_type', 'concentration', 'volume']
    for field in sample_fields:
        total_fields += 1
        if sample.get(field):
            filled_fields += 1

    # Check sequencing fields
    sequencing = extraction_result.get('sequencing', {}) or {}
    seq_fields = ['platform', 'analysis_type', 'coverage']
    for field in seq_fields:
        total_fields += 1
        if sequencing.get(field):
            filled_fields += 1

    if total_fields == 0:
        return 0.0

    # Convert to percentage
    confidence = (filled_fields / total_fields) * 100
    return min(90.0, confidence)  # Cap at 90% for regex-based extraction

@app.get("/")
async def root():
    """Root endpoint"""
    return {"message": "Simple RAG Submissions API Bridge", "status": "operational"}

@app.get("/health")
async def health_check():
    """Health check endpoint"""
    try:
        conn = await get_db_connection()
        await conn.close()
        return {"status": "healthy", "database": "connected"}
    except Exception as e:
        raise HTTPException(status_code=503, detail=f"Database connection failed: {e}")

@app.get("/api/rag/submissions", response_model=list[RagSubmissionResponse])
async def get_rag_submissions(limit: int = 50, offset: int = 0):
    """Get RAG submissions for the frontend"""
    try:
        conn = await get_db_connection()

        # Query RAG submissions from database
        submissions = await conn.fetch("""
            SELECT 
                submission_id,
                submitter_name,
                submitter_email,
                sample_type,
                document_name,
                confidence_score,
                created_at
            FROM rag_submissions 
            ORDER BY created_at DESC 
            LIMIT $1 OFFSET $2
        """, limit, offset)

        await conn.close()

        # Convert to response format
        result = []
        for row in submissions:
            result.append(RagSubmissionResponse(
                id=str(uuid.uuid4())[:8],  # Short ID for display
                submission_id=row['submission_id'],
                submitter_name=row['submitter_name'],
                submitter_email=row['submitter_email'],
                sample_type=row['sample_type'] or "Unknown",
                sample_name=row['document_name'],
                confidence_score=row['confidence_score'] or 0.0,
                created_at=row['created_at'].isoformat() if row['created_at'] else "",
                status="completed"
            ))

        return result

    except Exception as e:
        raise HTTPException(status_code=500, detail=f"Failed to fetch submissions: {e}")

@app.get("/api/rag/stats")
async def get_rag_statistics():
    """Get RAG system statistics"""
    try:
        conn = await get_db_connection()

        # Get submission counts
        total_submissions = await conn.fetchval("SELECT COUNT(*) FROM rag_submissions")

        # Get recent activity
        recent_count = await conn.fetchval("""
            SELECT COUNT(*) FROM rag_submissions 
            WHERE created_at >= NOW() - INTERVAL '7 days'
        """)

        # Get average confidence
        avg_confidence = await conn.fetchval("""
            SELECT AVG(confidence_score) FROM rag_submissions 
            WHERE confidence_score > 0
        """)

        await conn.close()

        return {
            "total_submissions": total_submissions or 0,
            "recent_submissions": recent_count or 0,
            "average_confidence": float(avg_confidence or 0.0),
            "status": "operational"
        }

    except Exception as e:
        raise HTTPException(status_code=500, detail=f"Failed to get statistics: {e}")

# FIXED: Handle file uploads properly (what frontend sends)
@app.post("/api/rag/process", response_model=DocumentProcessResponse)
async def process_document_upload(file: UploadFile = File(...)):
    """Process uploaded document and extract laboratory information using regex-based extraction"""
    start_time = time.time()

    try:
        # Read file content
        content = await file.read()

        # Convert to text (basic text extraction)
        if file.content_type == "text/plain":
            text = content.decode('utf-8')
        else:
            # For non-text files, try to decode as UTF-8
            try:
                text = content.decode('utf-8')
            except UnicodeDecodeError:
                return DocumentProcessResponse(
                    success=False,
                    confidence_score=0.0,
                    samples_found=0,
                    processing_time=time.time() - start_time,
                    extracted_data={"error": "Unable to process file. Please upload a text file."},
                    submitter_info=SubmitterInfo(),
                    sample_info=SampleInfo(),
                    sequencing_info=SequencingInfo(),
                    message="Unable to process file. Please upload a text file."
                )

        # Extract information using basic regex patterns
        extraction_result = extract_sample_info_basic(text)

        # Calculate confidence score based on how much information was extracted
        confidence_score = calculate_confidence_score_basic(extraction_result)

        # Count samples found
        samples_found = 1 if extraction_result.get('sample', {}).get('sample_id') else 0

        # Convert extracted data to structured format
        admin_data = extraction_result.get('administrative', {}) or {}
        sample_data = extraction_result.get('sample', {}) or {}
        sequencing_data = extraction_result.get('sequencing', {}) or {}

        submitter_info = SubmitterInfo(
            name=admin_data.get('submitter_name'),
            email=admin_data.get('submitter_email'),
            phone=admin_data.get('submitter_phone'),
            institution=admin_data.get('institution'),
            project_name=admin_data.get('project_name')
        )

        sample_info = SampleInfo(
            sample_id=sample_data.get('sample_id'),
            sample_name=sample_data.get('sample_id'),  # Use ID as name if no separate name
            sample_type=sample_data.get('sample_type'),
            concentration=sample_data.get('concentration'),
            volume=sample_data.get('volume'),
            storage_conditions=sample_data.get('storage_conditions')
        )

        sequencing_info = SequencingInfo(
            platform=sequencing_data.get('platform'),
            analysis_type=sequencing_data.get('analysis_type'),
            coverage=sequencing_data.get('coverage'),
            read_length=sequencing_data.get('read_length')
        )

        processing_time = time.time() - start_time

        # Store the extraction in database if successful
        if samples_found > 0:
            try:
                await store_extraction_result(
                    submission_id=str(uuid.uuid4()),
                    filename=file.filename or "uploaded_document.txt",
                    extracted_data=extraction_result,
                    confidence_score=confidence_score
                )
            except Exception as db_error:
                print(f"Failed to store extraction result: {db_error}")

        return DocumentProcessResponse(
            success=True,
            confidence_score=confidence_score,
            samples_found=samples_found,
            processing_time=processing_time,
            extracted_data=extraction_result,
            submitter_info=submitter_info,
            sample_info=sample_info,
            sequencing_info=sequencing_info,
            message="Document processed successfully using regex extraction" if samples_found > 0 else "Document processed but no complete sample information found"
        )

    except Exception as e:
        processing_time = time.time() - start_time
        print(f"Document processing failed: {e}")

        return DocumentProcessResponse(
            success=False,
            confidence_score=0.0,
            samples_found=0,
            processing_time=processing_time,
            extracted_data={"error": str(e)},
            submitter_info=SubmitterInfo(),
            sample_info=SampleInfo(),
            sequencing_info=SequencingInfo(),
            message=f"Processing failed: {str(e)}"
        )

async def store_extraction_result(submission_id: str, filename: str, extracted_data: dict[str, Any], confidence_score: float):
    """Store extraction result in database"""
    try:
        conn = await get_db_connection()

        admin_data = extracted_data.get('administrative', {}) or {}
        sample_data = extracted_data.get('sample', {}) or {}

        await conn.execute("""
            INSERT INTO rag_submissions (
                submission_id, document_name, submitter_name, submitter_email,
                sample_type, confidence_score, extracted_data, created_at
            ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
        """,
            submission_id,
            filename,
            admin_data.get('submitter_name'),
            admin_data.get('submitter_email'),
            sample_data.get('sample_type'),
            confidence_score / 100.0,  # Store as decimal
            json.dumps(extracted_data),
            datetime.utcnow()
        )

        await conn.close()
        print(f"Stored extraction result: {submission_id}")

    except Exception as e:
        print(f"Failed to store extraction: {e}")
        # Don't raise - this is not critical for the user experience

class QueryRequest(BaseModel):
    """Request model for queries"""
    query: str
    session_id: str | None = "default"

class QueryResponse(BaseModel):
    """Response model for queries"""
    answer: str

def get_intelligent_response(query: str) -> str:
    """Generate intelligent responses based on query content"""
    query_lower = query.lower().strip()

    # Greeting responses
    if any(word in query_lower for word in ['hello', 'hi', 'hey', 'greetings']):
        return """Hello! I'm your lab management assistant. I can help you with sample processing, storage management, sequencing workflows, and more.

What can I help you with today? You can ask me about:
• Submitting new samples
• Storage requirements 
• Setting up sequencing jobs
• Generating reports
• Using the lab management system"""

    # Sample submission and processing (check this BEFORE general help)
    elif any(phrase in query_lower for phrase in ['submit', 'upload', 'create sample', 'new sample', 'add sample', 'submit a sample', 'submit sample', 'submission']):
        return """To submit new samples, you have several options:

1. AI DOCUMENT PROCESSING (Recommended)
   • Upload lab submission forms (PDF, Word, or text)
   • I'll automatically extract sample information
   • Review and confirm the extracted data
   
2. MANUAL SAMPLE ENTRY
   • Use the "Create Sample" form
   • Fill in all required fields manually
   • Generate barcodes automatically

3. BULK UPLOAD VIA TEMPLATES
   • Download Excel templates
   • Fill in multiple samples at once
   • Upload for batch processing

Which method would you prefer to use?"""

    # Default response for unmatched queries
    else:
        return """I'm your lab management assistant and I can help with many laboratory tasks. Here are some things you might want to know about:

COMMON TASKS:
• "How do I submit a new sample?"
• "What are the storage requirements for DNA?"
• "How do I create a sequencing job?"
• "Can you help me generate a report?"
• "Where is sample XYZ located?"

TRY ASKING ABOUT:
• Sample submission and processing
• Storage locations and temperatures  
• Sequencing workflows and QC
• Barcode tracking and scanning
• Data export and reporting
• System navigation and troubleshooting

Could you rephrase your question or ask about a specific lab management task? I'm here to help make your laboratory work more efficient!"""

@app.post("/query", response_model=QueryResponse)
async def query_submission_information(request: QueryRequest):
    """Query the RAG system for information about submitted samples"""
    try:
        answer = get_intelligent_response(request.query)
        return QueryResponse(answer=answer)

    except Exception:
        # Return a helpful error message
        return QueryResponse(
            answer="I apologize, but I'm having trouble processing your question right now. This could be due to a temporary system issue. Please try again in a moment, or contact your lab administrator if the problem persists."
        )

# Startup event to test database connection
@app.on_event("startup")
async def startup_event():
    """Test database connection on startup"""
    try:
        conn = await get_db_connection()
        await conn.close()
        print("Database connection successful")
        print("Using regex-based document extraction (LLM integration available in future versions)")
        print("API now handles file uploads correctly for frontend integration")
    except Exception as e:
        print(f"Database connection failed: {e}")

if __name__ == "__main__":
    import uvicorn
    print("Starting Simple RAG Submissions API Bridge")
    print("Providing basic RAG data access for frontend")
    print("CORS enabled for all origins")
    print("Using regex-based document extraction")
    print("File upload handling enabled for frontend integration")

    uvicorn.run(app, host="0.0.0.0", port=8000)
