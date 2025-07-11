"""
Enhanced RAG Service - Main Application

A comprehensive microservice for AI-powered laboratory document processing.
"""

import asyncio
import logging
import time
import uuid
from contextlib import asynccontextmanager
from datetime import datetime
from pathlib import Path
from typing import Any, Dict, List, Optional

import structlog
import uvicorn
from fastapi import FastAPI, File, HTTPException, Request, UploadFile
from fastapi.middleware.cors import CORSMiddleware
from fastapi.middleware.gzip import GZipMiddleware
from fastapi.responses import JSONResponse
from pydantic import BaseModel

# Configure structured logging
structlog.configure(
    processors=[
        structlog.stdlib.filter_by_level,
        structlog.stdlib.add_logger_name,
        structlog.stdlib.add_log_level,
        structlog.stdlib.PositionalArgumentsFormatter(),
        structlog.processors.TimeStamper(fmt="iso"),
        structlog.processors.StackInfoRenderer(),
        structlog.processors.format_exc_info,
        structlog.processors.UnicodeDecoder(),
        structlog.processors.JSONRenderer()
    ],
    context_class=dict,
    logger_factory=structlog.stdlib.LoggerFactory(),
    wrapper_class=structlog.stdlib.BoundLogger,
    cache_logger_on_first_use=True,
)

logger = structlog.get_logger(__name__)

# Pydantic models for request/response
class DocumentProcessResponse(BaseModel):
    success: bool
    confidence_score: float
    samples_found: int
    processing_time: float
    extracted_data: Dict[str, Any]
    message: str
    submission_id: Optional[str] = None

class QueryRequest(BaseModel):
    query: str
    session_id: Optional[str] = "default"

class QueryResponse(BaseModel):
    answer: str
    session_id: Optional[str]
    confidence_score: float
    processing_time: float
    sources: List[Dict[str, Any]]

# In-memory storage for demo purposes
submissions_storage: List[Dict[str, Any]] = []

@asynccontextmanager
async def lifespan(app: FastAPI):
    """Application lifespan manager."""
    logger.info("🚀 Starting Enhanced RAG Service")

    # Initialize services here
    try:
        # Create upload directory
        upload_dir = Path("/app/uploads")
        upload_dir.mkdir(exist_ok=True)
        
        logger.info("✅ Enhanced RAG Service startup complete")
        yield
    except Exception as e:
        logger.error("❌ Failed to initialize service", error=str(e))
        raise
    finally:
        logger.info("🛑 Shutting down Enhanced RAG Service")


def create_app() -> FastAPI:
    """Create and configure the FastAPI application."""

    app = FastAPI(
        title="Enhanced RAG Service",
        description="AI-Powered Laboratory Document Processing Microservice",
        version="0.1.0",
        docs_url="/docs",
        redoc_url="/redoc",
        lifespan=lifespan,
    )

    # Add middleware
    app.add_middleware(
        CORSMiddleware,
        allow_origins=["*"],  # Configure appropriately for production
        allow_credentials=True,
        allow_methods=["*"],
        allow_headers=["*"],
    )

    app.add_middleware(GZipMiddleware, minimum_size=1000)

    # Root endpoint
    @app.get("/")
    async def root():
        """Root endpoint with service information."""
        return {
            "service": "Enhanced RAG Service",
            "version": "0.1.0",
            "status": "operational",
            "docs": "/docs",
            "health": "/api/v1/health",
            "endpoints": {
                "document_processing": "/api/rag/process",
                "file_upload": "/upload",
                "query": "/api/samples/rag/query"
            }
        }

    # Health check endpoint
    @app.get("/api/v1/health")
    async def health_check():
        """Basic health check endpoint."""
        return {
            "status": "healthy",
            "service": "Enhanced RAG Service",
            "version": "0.1.0",
            "features": ["document_processing", "file_upload", "intelligent_queries"]
        }

    # Document processing endpoint
    @app.post("/api/rag/process", response_model=DocumentProcessResponse)
    async def process_document_upload(file: UploadFile = File(...)):
        """Process uploaded document and extract laboratory information"""
        start_time = time.time()
        
        logger.info(f"Processing document upload: {file.filename}")

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
                        message="Unable to process file. Please upload a text file."
                    )

            # Basic information extraction using regex patterns
            extraction_result = extract_sample_info_basic(text)
            
            # Calculate confidence score
            confidence_score = calculate_confidence_score_basic(extraction_result)
            
            # Count samples found
            samples_found = 1 if extraction_result.get('sample', {}).get('sample_id') else 0
            
            processing_time = time.time() - start_time
            submission_id = str(uuid.uuid4())[:8]

            # Store in memory for demo
            if samples_found > 0:
                submission_data = {
                    "id": submission_id,
                    "filename": file.filename,
                    "extracted_data": extraction_result,
                    "confidence_score": confidence_score,
                    "created_at": datetime.utcnow().isoformat(),
                    "status": "completed"
                }
                submissions_storage.append(submission_data)

            logger.info(f"Document processing completed: {samples_found} samples found, confidence: {confidence_score}")

            return DocumentProcessResponse(
                success=True,
                confidence_score=confidence_score,
                samples_found=samples_found,
                processing_time=processing_time,
                extracted_data=extraction_result,
                message="Document processed successfully" if samples_found > 0 else "Document processed but no complete sample information found",
                submission_id=submission_id
            )

        except Exception as e:
            processing_time = time.time() - start_time
            logger.error(f"Document processing failed: {e}")

            return DocumentProcessResponse(
                success=False,
                confidence_score=0.0,
                samples_found=0,
                processing_time=processing_time,
                extracted_data={"error": str(e)},
                message=f"Processing failed: {str(e)}"
            )

    # File upload endpoint (alternative endpoint)
    @app.post("/upload")
    async def upload_document(file: UploadFile = File(...)):
        """Upload and process a laboratory document"""
        logger.info(f"Document upload via /upload endpoint: {file.filename}")
        return await process_document_upload(file)

    # Document processing endpoint (alternative)
    @app.post("/process-document")
    async def process_document_alt(file: UploadFile = File(...)):
        """Process a laboratory submission document - alternative endpoint"""
        logger.info(f"Document processing via /process-document endpoint: {file.filename}")
        return await process_document_upload(file)

    # Query endpoint for intelligent queries
    @app.post("/api/samples/rag/query", response_model=QueryResponse)
    async def query_rag_system(request: QueryRequest):
        """Handle intelligent queries about laboratory management"""
        start_time = time.time()
        
        logger.info(f"Processing RAG query: {request.query}")
        
        try:
            # Generate intelligent response (mock implementation)
            answer = get_intelligent_rag_response(request.query, request.session_id or "default")
            processing_time = time.time() - start_time
            
            return QueryResponse(
                answer=answer,
                session_id=request.session_id,
                confidence_score=0.85,
                processing_time=processing_time,
                sources=[
                    {"title": "Laboratory Management Knowledge Base", "relevance": 0.9},
                    {"title": "Sample Processing Guidelines", "relevance": 0.85},
                    {"title": "Quality Control Standards", "relevance": 0.8}
                ]
            )
            
        except Exception as e:
            logger.error(f"Query processing failed: {e}")
            raise HTTPException(status_code=500, detail=f"Query processing failed: {str(e)}")

    # Get submissions endpoint
    @app.get("/api/submissions")
    async def get_submissions():
        """Get all processed submissions"""
        return {"submissions": submissions_storage, "total": len(submissions_storage)}

    # RAG submissions endpoint (frontend expects this path)
    @app.get("/api/rag/submissions")
    async def get_rag_submissions():
        """Get all RAG processed submissions"""
        logger.info("Fetching RAG submissions")
        
        # Mock data to match frontend expectations
        mock_submissions = [
            {
                "id": "RAG-001",
                "submission_id": "RAG-001",
                "source_document": "lab_report_2024_01.pdf",
                "submitter_name": "Dr. Smith",
                "submitter_email": "dr.smith@lab.com",
                "sample_type": "DNA",
                "confidence_score": 0.92,
                "status": "completed",
                "created_at": datetime.utcnow().isoformat(),
                "processing_time": 2.3,
                "extracted_data": {
                    "administrative": {
                        "submitter_name": "Dr. Smith",
                        "submitter_email": "dr.smith@lab.com",
                        "institution": "Research Lab"
                    },
                    "sample": {
                        "sample_type": "DNA",
                        "volume": "50μL",
                        "concentration": "100ng/μL"
                    }
                }
            },
            {
                "id": "RAG-002",
                "submission_id": "RAG-002",
                "source_document": "sequencing_request_2024_02.pdf",
                "submitter_name": "Dr. Johnson",
                "submitter_email": "dr.johnson@lab.com",
                "sample_type": "RNA",
                "confidence_score": 0.88,
                "status": "processing",
                "created_at": datetime.utcnow().isoformat(),
                "processing_time": 1.8,
                "extracted_data": {
                    "administrative": {
                        "submitter_name": "Dr. Johnson",
                        "submitter_email": "dr.johnson@lab.com",
                        "institution": "University Lab"
                    },
                    "sample": {
                        "sample_type": "RNA",
                        "volume": "25μL",
                        "concentration": "200ng/μL"
                    }
                }
            },
            {
                "id": "RAG-003",
                "submission_id": "RAG-003",
                "source_document": "protein_analysis_2024_03.pdf",
                "submitter_name": "Dr. Brown",
                "submitter_email": "dr.brown@lab.com",
                "sample_type": "Protein",
                "confidence_score": 0.95,
                "status": "completed",
                "created_at": datetime.utcnow().isoformat(),
                "processing_time": 3.1,
                "extracted_data": {
                    "administrative": {
                        "submitter_name": "Dr. Brown",
                        "submitter_email": "dr.brown@lab.com",
                        "institution": "Biotech Lab"
                    },
                    "sample": {
                        "sample_type": "Protein",
                        "volume": "100μL",
                        "concentration": "500ng/μL"
                    }
                }
            }
        ]
        
        # Combine with actual submissions from storage
        all_submissions = mock_submissions + submissions_storage
        
        # Calculate statistics
        total_count = len(all_submissions)
        processing_count = len([s for s in all_submissions if s.get("status") == "processing"])
        completed_count = len([s for s in all_submissions if s.get("status") == "completed"])
        failed_count = len([s for s in all_submissions if s.get("status") == "failed"])
        
        return {
            "submissions": all_submissions,
            "data": all_submissions,  # Frontend expects both keys
            "totalCount": total_count,
            "processing": processing_count,
            "completed": completed_count,
            "failed": failed_count
        }

    # Individual RAG submission endpoint (frontend expects this)
    @app.get("/api/rag/submissions/{submission_id}")
    async def get_rag_submission(submission_id: str):
        """Get individual RAG submission by ID"""
        logger.info(f"Fetching RAG submission: {submission_id}")
        
        # Mock data for known submissions
        mock_submissions = {
            "RAG-001": {
                "id": "RAG-001",
                "submission_id": "RAG-001",
                "source_document": "lab_report_2024_01.pdf",
                "submitter_name": "Dr. Smith",
                "submitter_email": "dr.smith@lab.com",
                "sample_type": "DNA",
                "confidence_score": 0.92,
                "status": "completed",
                "created_at": datetime.utcnow().isoformat(),
                "processing_time": 2.3,
                "extracted_data": {
                    "administrative": {
                        "submitter_name": "Dr. Smith",
                        "submitter_email": "dr.smith@lab.com",
                        "institution": "Research Lab"
                    },
                    "sample": {
                        "sample_type": "DNA",
                        "volume": "50μL",
                        "concentration": "100ng/μL"
                    }
                }
            },
            "RAG-002": {
                "id": "RAG-002",
                "submission_id": "RAG-002",
                "source_document": "sequencing_request_2024_02.pdf",
                "submitter_name": "Dr. Johnson",
                "submitter_email": "dr.johnson@lab.com",
                "sample_type": "RNA",
                "confidence_score": 0.88,
                "status": "processing",
                "created_at": datetime.utcnow().isoformat(),
                "processing_time": 1.8,
                "extracted_data": {
                    "administrative": {
                        "submitter_name": "Dr. Johnson",
                        "submitter_email": "dr.johnson@lab.com",
                        "institution": "University Lab"
                    },
                    "sample": {
                        "sample_type": "RNA",
                        "volume": "25μL",
                        "concentration": "200ng/μL"
                    }
                }
            },
            "RAG-003": {
                "id": "RAG-003",
                "submission_id": "RAG-003",
                "source_document": "protein_analysis_2024_03.pdf",
                "submitter_name": "Dr. Brown",
                "submitter_email": "dr.brown@lab.com",
                "sample_type": "Protein",
                "confidence_score": 0.95,
                "status": "completed",
                "created_at": datetime.utcnow().isoformat(),
                "processing_time": 3.1,
                "extracted_data": {
                    "administrative": {
                        "submitter_name": "Dr. Brown",
                        "submitter_email": "dr.brown@lab.com",
                        "institution": "Biotech Lab"
                    },
                    "sample": {
                        "sample_type": "Protein",
                        "volume": "100μL",
                        "concentration": "500ng/μL"
                    }
                }
            }
        }
        
        # Check mock data first
        if submission_id in mock_submissions:
            return mock_submissions[submission_id]
        
        # Check actual storage
        for submission in submissions_storage:
            if submission.get("id") == submission_id or submission.get("submission_id") == submission_id:
                return submission
        
        # Not found
        raise HTTPException(status_code=404, detail=f"Submission {submission_id} not found")

    return app


def extract_sample_info_basic(text: str) -> Dict[str, Any]:
    """Basic extraction of sample information using regex patterns"""
    import re
    
    extraction_result = {
        "administrative": {},
        "sample": {},
        "sequencing": {},
        "storage": {}
    }
    
    # Extract email addresses
    email_pattern = r'\b[A-Za-z0-9._%+-]+@[A-Za-z0-9.-]+\.[A-Z|a-z]{2,}\b'
    emails = re.findall(email_pattern, text)
    if emails:
        extraction_result["administrative"]["submitter_email"] = emails[0]
    
    # Extract sample IDs (pattern: letters followed by numbers)
    sample_id_pattern = r'\b[A-Z]{2,4}[-_]?\d{3,8}\b'
    sample_ids = re.findall(sample_id_pattern, text)
    if sample_ids:
        extraction_result["sample"]["sample_id"] = sample_ids[0]
    
    # Extract sample types
    sample_types = ["DNA", "RNA", "Protein", "Plasma", "Serum", "Tissue", "Blood"]
    for sample_type in sample_types:
        if sample_type.lower() in text.lower():
            extraction_result["sample"]["sample_type"] = sample_type
            break
    
    # Extract concentration (μg/μL, ng/μL, etc.)
    conc_pattern = r'(\d+\.?\d*)\s*(μg/μL|ng/μL|μg/ml|ng/ml|mg/ml)'
    conc_matches = re.findall(conc_pattern, text, re.IGNORECASE)
    if conc_matches:
        extraction_result["sample"]["concentration"] = f"{conc_matches[0][0]} {conc_matches[0][1]}"
    
    # Extract volume
    vol_pattern = r'(\d+\.?\d*)\s*(μL|ml|L|ul)'
    vol_matches = re.findall(vol_pattern, text, re.IGNORECASE)
    if vol_matches:
        extraction_result["sample"]["volume"] = f"{vol_matches[0][0]} {vol_matches[0][1]}"
    
    # Extract submitter name (basic pattern)
    name_pattern = r'(?:submitter|contact|name):\s*([A-Za-z\s]+)'
    name_matches = re.findall(name_pattern, text, re.IGNORECASE)
    if name_matches:
        extraction_result["administrative"]["submitter_name"] = name_matches[0].strip()
    
    return extraction_result


def calculate_confidence_score_basic(extraction_result: Dict[str, Any]) -> float:
    """Calculate confidence score based on extracted information"""
    score = 0.0
    total_fields = 0
    
    # Check administrative fields
    admin_fields = ["submitter_email", "submitter_name"]
    for field in admin_fields:
        total_fields += 1
        if extraction_result.get("administrative", {}).get(field):
            score += 1
    
    # Check sample fields
    sample_fields = ["sample_id", "sample_type", "concentration", "volume"]
    for field in sample_fields:
        total_fields += 1
        if extraction_result.get("sample", {}).get(field):
            score += 1
    
    return (score / total_fields) if total_fields > 0 else 0.0


def get_intelligent_rag_response(query: str, session_id: str) -> str:
    """Generate intelligent response to laboratory management queries"""
    query_lower = query.lower()
    
    # Sample handling queries
    if any(word in query_lower for word in ["sample", "storage", "temperature"]):
        return f"For sample storage, TracSeq 2.0 supports multiple temperature zones: -80°C for long-term storage, -20°C for intermediate storage, 4°C for short-term storage, and room temperature for immediate processing. Each zone has IoT monitoring for temperature excursions and automated alerts."
    
    # Submission queries
    elif any(word in query_lower for word in ["submission", "submit", "upload"]):
        return f"To submit samples to TracSeq 2.0, upload your laboratory documents through the web interface. Our AI system will automatically extract sample information, validate the data, and assign storage locations. You'll receive a submission ID for tracking."
    
    # Quality control queries
    elif any(word in query_lower for word in ["quality", "qc", "validation"]):
        return f"TracSeq 2.0 implements comprehensive quality control including automated data validation, sample integrity checks, and chain of custody tracking. All QC metrics are recorded and available in the dashboard."
    
    # Sequencing queries
    elif any(word in query_lower for word in ["sequencing", "sequence", "analysis"]):
        return f"TracSeq 2.0 supports multiple sequencing platforms and analysis pipelines. The system automatically assigns samples to appropriate workflows based on sample type and analysis requirements specified in your submission."
    
    # Default response
    else:
        return f"I'm here to help with TracSeq 2.0 laboratory management. I can assist with sample submissions, storage management, quality control, and sequencing workflows. What specific aspect would you like to know more about?"


# Create the application instance
app = create_app()


if __name__ == "__main__":
    logger.info("🚀 Starting Enhanced RAG Service", host="0.0.0.0", port=8000)

    uvicorn.run(
        app,
        host="0.0.0.0",
        port=8000,
        log_config=None,
    )
