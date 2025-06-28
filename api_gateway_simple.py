#!/usr/bin/env python3
"""
Simple API Gateway for TracSeq 2.0 Development
Provides basic endpoints for frontend development without full microservices.
"""

from fastapi import FastAPI, HTTPException
from fastapi.middleware.cors import CORSMiddleware
from datetime import datetime
import uvicorn
import json

app = FastAPI(title="TracSeq 2.0 Simple API Gateway", version="1.0.0")

# Enable CORS for frontend development
app.add_middleware(
    CORSMiddleware,
    allow_origins=["http://localhost:5173", "http://localhost:3000"],
    allow_credentials=True,
    allow_methods=["*"],
    allow_headers=["*"],
)

# Mock data for development
MOCK_SAMPLES = [
    {
        "id": "smp-001",
        "name": "Sample Alpha-001",
        "barcode": "LAB-20240101-001",
        "location": "Zone A-1",
        "status": "Pending",
        "created_at": datetime.now().isoformat(),
        "metadata": {
            "confidence_score": 0.95,
            "processing_time": 2.1,
            "source_document": "lab_submission_form_001.pdf",
            "submitter_name": "Dr. Sarah Wilson",
            "submitter_email": "sarah.wilson@lab.edu",
            "extraction_method": "manual",
            "sample_type": "DNA",
            "volume": 150,
            "concentration": 25.7
        }
    },
    {
        "id": "smp-002",
        "name": "AI-Generated Sample Beta",
        "barcode": "LAB-20240101-002",
        "location": "Zone B-2",
        "status": "InStorage",
        "created_at": datetime.now().isoformat(),
        "metadata": {
            "confidence_score": 0.92,
            "processing_time": 3.5,
            "source_document": "lab_submission_form_001.pdf",
            "submitter_name": "Dr. Jane Smith",
            "submitter_email": "jane.smith@lab.edu",
            "rag_submission_id": "rag-001",
            "extraction_method": "ai_rag",
            "sample_type": "RNA",
            "volume": 100,
            "concentration": 22.3,
            "validation_warnings": [],
            "extraction_warnings": ["Low confidence in volume measurement"]
        }
    }
]

MOCK_RAG_SUBMISSIONS = [
    {
        "id": "rag-001",
        "filename": "lab_submission_form_001.pdf",
        "status": "completed",
        "uploadedAt": datetime.now().isoformat(),
        "processedAt": datetime.now().isoformat(),
        "confidence": 0.92,
        "extractedSamples": 1,
        "errors": [],
        "metadata": {
            "submission_id": "SUB-20240101-001",
            "source_document": "lab_submission_form_001.pdf",
            "submitter_name": "Dr. Jane Smith",
            "submitter_email": "jane.smith@lab.edu",
            "confidence_score": 0.92,
            "processing_time": 3.5,
            "samples_created": 1
        }
    },
    {
        "id": "rag-002",
        "filename": "research_protocol_v2.pdf",
        "status": "processing",
        "uploadedAt": datetime.now().isoformat(),
        "confidence": 0.78,
        "extractedSamples": 0,
        "errors": [],
        "metadata": {
            "submission_id": "SUB-20240101-002",
            "source_document": "research_protocol_v2.pdf",
            "submitter_name": "Dr. Bob Johnson",
            "submitter_email": "bob.johnson@research.org",
            "confidence_score": 0.78,
            "processing_time": 5.2,
            "samples_created": 0
        }
    }
]

MOCK_TEMPLATES = [
    {
        "id": "tpl-001",
        "name": "Standard Sample Submission Form",
        "description": "Standard laboratory sample submission template",
        "file_type": "PDF",
        "version": "1.0",
        "created_at": datetime.now().isoformat(),
        "updated_at": datetime.now().isoformat()
    },
    {
        "id": "tpl-002",
        "name": "Research Protocol Template",
        "description": "Template for research protocol documentation",
        "file_type": "DOCX",
        "version": "2.1",
        "created_at": datetime.now().isoformat(),
        "updated_at": datetime.now().isoformat()
    },
    {
        "id": "tpl-003",
        "name": "Quality Control Checklist",
        "description": "Quality control and validation checklist",
        "file_type": "XLSX",
        "version": "1.5",
        "created_at": datetime.now().isoformat(),
        "updated_at": datetime.now().isoformat()
    }
]

MOCK_SEQUENCING_JOBS = [
    {
        "id": "job-001",
        "name": "Genomic Analysis Batch #42",
        "status": "in_progress",
        "platform": "Illumina NovaSeq",
        "samples_count": 24,
        "priority": "high",
        "estimated_completion": "2024-01-15T14:30:00Z",
        "created_at": datetime.now().isoformat()
    },
    {
        "id": "job-002",
        "name": "RNA-Seq Project Delta",
        "status": "completed",
        "platform": "Oxford Nanopore",
        "samples_count": 12,
        "priority": "normal",
        "completed_at": "2024-01-10T09:15:00Z",
        "created_at": datetime.now().isoformat()
    }
]

@app.get("/health")
async def health_check():
    """Health check endpoint"""
    return {"status": "healthy", "timestamp": datetime.now().isoformat(), "service": "api-gateway"}

@app.get("/api/samples")
async def get_samples(extraction_method: str = ""):
    """Get all samples, optionally filtered by extraction method"""
    if extraction_method == "ai_rag":
        return [s for s in MOCK_SAMPLES if s["metadata"].get("extraction_method") == "ai_rag"]
    return MOCK_SAMPLES

@app.get("/api/rag/samples")
async def get_rag_samples():
    """Get AI-generated samples"""
    return [s for s in MOCK_SAMPLES if s["metadata"].get("extraction_method") == "ai_rag"]

@app.get("/api/rag/submissions")
async def get_rag_submissions():
    """Get RAG submissions"""
    return MOCK_RAG_SUBMISSIONS

@app.get("/api/rag/submissions/{submission_id}")
async def get_rag_submission(submission_id: str):
    """Get a specific RAG submission"""
    submission = next((s for s in MOCK_RAG_SUBMISSIONS if s["id"] == submission_id), None)
    if not submission:
        raise HTTPException(status_code=404, detail="Submission not found")
    return submission

@app.post("/api/rag/process")
async def process_document():
    """Process uploaded document"""
    return {
        "success": True,
        "samples": [
            {
                "name": f"AI-Extracted Sample {datetime.now().strftime('%H%M%S')}",
                "barcode": f"AUTO-{datetime.now().strftime('%H%M%S')}",
                "location": "Zone-AI-1",
                "metadata": {
                    "extraction_method": "ai_rag",
                    "confidence_score": 0.88,
                    "source_document": "uploaded_document.pdf",
                    "sample_type": "RNA",
                    "volume": 120,
                    "concentration": 18.5
                }
            }
        ],
        "confidence_score": 0.88,
        "validation_warnings": ["Sample type inference has medium confidence"],
        "processing_time": 2.3,
        "source_document": "uploaded_document.pdf"
    }

@app.post("/api/samples/rag/query")
async def rag_query(request: dict):
    """Handle RAG queries"""
    query = request.get("query", "")
    
    # Simple response based on query content
    if "sample" in query.lower():
        response = f"Found {len(MOCK_SAMPLES)} samples in the system. 1 sample was created using AI extraction."
    elif "template" in query.lower():
        response = f"There are {len(MOCK_TEMPLATES)} templates available in the system."
    elif "sequencing" in query.lower():
        response = f"Currently tracking {len(MOCK_SEQUENCING_JOBS)} sequencing jobs."
    else:
        response = "TracSeq 2.0 laboratory management system is operational. You can ask about samples, templates, or sequencing jobs."
    
    return {
        "success": True,
        "response": response,
        "confidence": 0.85,
        "sources": ["api-gateway-mock"]
    }

@app.get("/api/templates")
async def get_templates():
    """Get all templates"""
    return MOCK_TEMPLATES

@app.get("/api/sequencing")
async def get_sequencing_jobs():
    """Get sequencing jobs"""
    return MOCK_SEQUENCING_JOBS

@app.post("/api/auth/login")
async def login(request: dict):
    """Mock login endpoint"""
    email = request.get("email", "")
    password = request.get("password", "")
    
    # Accept any email with password "password"
    if password == "password":
        return {
            "success": True,
            "token": "mock-jwt-token-12345",
            "user": {
                "id": "user-001",
                "email": email,
                "name": "Test User",
                "role": "admin"
            }
        }
    
    raise HTTPException(status_code=401, detail="Invalid credentials")

@app.get("/api/dashboard/metrics")
async def get_dashboard_metrics():
    """Get dashboard metrics"""
    return {
        "total_samples": len(MOCK_SAMPLES),
        "samples_in_storage": len([s for s in MOCK_SAMPLES if s["status"] == "InStorage"]),
        "samples_pending": len([s for s in MOCK_SAMPLES if s["status"] == "Pending"]),
        "active_sequencing_jobs": len([j for j in MOCK_SEQUENCING_JOBS if j["status"] == "in_progress"]),
        "processing_time": {
            "average": 2.8,
            "median": 2.9,
            "bottlenecks": ["Document parsing", "Quality validation"]
        },
        "throughput": {
            "samples_per_day": 12,
            "documents_processed": 8,
            "efficiency_score": 0.87
        }
    }

if __name__ == "__main__":
    print("🚀 Starting TracSeq 2.0 Simple API Gateway...")
    print("📍 API Gateway: http://localhost:8089")
    print("📊 Health Check: http://localhost:8089/health")
    print("🔗 API Docs: http://localhost:8089/docs")
    
    uvicorn.run(app, host="0.0.0.0", port=8089) 