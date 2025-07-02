#!/usr/bin/env python3
"""
Simple API Gateway for TracSeq 2.0
Minimal working implementation for demonstration
"""

import json
import os
import sys
import uuid
from datetime import datetime, timedelta
from typing import Any, Dict, Optional

import httpx
import uvicorn
from fastapi import FastAPI, HTTPException, Request, Response
from fastapi.middleware.cors import CORSMiddleware
from pydantic import BaseModel
from fastapi.responses import JSONResponse

# Initialize FastAPI app
app = FastAPI(
    title="TracSeq 2.0 API Gateway",
    description="Central routing hub for TracSeq microservices",
    version="2.0.0"
)

# CORS middleware
app.add_middleware(
    CORSMiddleware,
    allow_origins=["*"],  # In production, specify exact origins
    allow_credentials=True,
    allow_methods=["*"],
    allow_headers=["*"],
)

# Service URLs from environment
RAG_SERVICE_URL = os.getenv("RAG_SERVICE_URL", "http://rag-service:8000")
AUTH_SERVICE_URL = os.getenv("AUTH_SERVICE_URL", "http://auth-service:8080")
SEQUENCING_SERVICE_URL = os.getenv("SEQUENCING_SERVICE_URL", "http://lims-sequencing:8084")
NOTIFICATION_SERVICE_URL = os.getenv("NOTIFICATION_SERVICE_URL", "http://lims-notification:8000")
# Use lab_manager for new features (temporary fix)
LAB_MANAGER_URL = os.getenv("LAB_MANAGER_URL", "http://host.docker.internal:3001")
PROJECT_SERVICE_URL = os.getenv("PROJECT_SERVICE_URL", LAB_MANAGER_URL)
LIBRARY_PREP_SERVICE_URL = os.getenv("LIBRARY_PREP_SERVICE_URL", LAB_MANAGER_URL)
QAQC_SERVICE_URL = os.getenv("QAQC_SERVICE_URL", "http://lims-qaqc:8089")
FLOW_CELL_SERVICE_URL = os.getenv("FLOW_CELL_SERVICE_URL", LAB_MANAGER_URL)
# Enhanced services
EVENT_SERVICE_URL = os.getenv("EVENT_SERVICE_URL", "http://lims-events:8087")
TRANSACTION_SERVICE_URL = os.getenv("TRANSACTION_SERVICE_URL", "http://lims-transactions:8088")
TEMPLATE_SERVICE_URL = os.getenv("TEMPLATE_SERVICE_URL", "http://lims-templates:8000")

# Pydantic models for API requests
class LoginRequest(BaseModel):
    email: str
    password: str

class LoginResponse(BaseModel):
    token: str
    user: Dict[str, Any]

# Mock data for demonstration
MOCK_USERS = {
    "admin@tracseq.com": {
        "id": "1",
        "email": "admin@tracseq.com",
        "name": "Admin User",
        "role": "admin",
        "password": "admin123"  # In production, this would be hashed
    },
    "user@tracseq.com": {
        "id": "2",
        "email": "user@tracseq.com",
        "name": "Lab User",
        "role": "user",
        "password": "user123"
    }
}

@app.get("/")
async def root():
    """Root endpoint"""
    return {
        "service": "TracSeq 2.0 API Gateway",
        "status": "running",
        "version": "2.0.0",
        "description": "Central routing hub for TracSeq microservices"
    }

@app.get("/health")
async def health():
    """Health check endpoint"""
    return {
        "status": "healthy",
        "service": "api-gateway",
        "timestamp": datetime.now().isoformat()
    }

# Authentication endpoints
@app.post("/api/auth/login")
async def login(request: Request):
    """User login endpoint - flexible payload handling"""
    try:
        # Try to get JSON body
        body = await request.json()

        # Handle different payload formats
        email = body.get("email") or body.get("username")
        password = body.get("password")

        if not email or not password:
            raise HTTPException(status_code=400, detail="Email and password are required")

        user = MOCK_USERS.get(email)

        if not user or user["password"] != password:
            raise HTTPException(status_code=401, detail="Invalid credentials")

        # In production, generate a proper JWT token
        mock_token = f"mock_jwt_token_for_{user['id']}"

        return {
            "token": mock_token,
            "user": {
                "id": user["id"],
                "email": user["email"],
                "name": user["name"],
                "role": user["role"]
            }
        }

    except ValueError:
        raise HTTPException(status_code=400, detail="Invalid JSON payload")
    except Exception as e:
        raise HTTPException(status_code=500, detail=f"Login error: {e!s}")

@app.get("/api/auth/me")
async def get_current_user(request: Request):
    """Get current user info"""
    # Mock user info - in production, decode JWT from Authorization header
    return {
        "id": "1",
        "email": "admin@tracseq.com",
        "name": "Admin User",
        "role": "admin"
    }

# Additional auth endpoint that frontend might be calling
@app.get("/api/users/me")
async def get_current_user_alt(request: Request):
    """Get current user info (alternative endpoint)"""
    # Mock user info - in production, decode JWT from Authorization header
    return {
        "id": "1",
        "email": "admin@tracseq.com",
        "name": "Admin User",
        "role": "admin"
    }

# Dashboard endpoints
@app.get("/api/dashboard/stats")
async def get_dashboard_stats():
    """Get dashboard statistics"""
    return {
        "totalSamples": 1247,
        "activeSamples": 89,
        "completedSamples": 1158,
        "pendingSamples": 23,
        "activeJobs": 12,
        "completedJobs": 456,
        "storageUtilization": 78.5,
        "systemHealth": "healthy",
        "lastUpdated": datetime.now().isoformat()
    }

# Samples endpoints
@app.get("/api/samples")
async def get_samples(request: Request):
    """Get all samples - with support for filtering by extraction method"""
    # Check for extraction_method query parameter
    extraction_method = request.query_params.get("extraction_method")

    if extraction_method == "ai_rag":
        # Return RAG-processed samples when specifically requested
        rag_samples_data = [
            {
                "id": "SMPL-RAG-001",
                "originalId": "SMPL-001",
                "name": "Sample 001 (RAG Processed)",
                "type": "DNA",
                "status": "RAG_Analyzed",
                "submittedBy": "Dr. Smith",
                "submittedDate": (datetime.now() - timedelta(days=1)).isoformat(),
                "location": "Freezer A1-B2",
                "ragProcessingDate": (datetime.now() - timedelta(hours=2)).isoformat(),
                "extractedMetadata": {
                    "concentration": "50 ng/μL",
                    "volume": "100 μL",
                    "quality": "High",
                    "extractionMethod": "Qiagen DNeasy"
                },
                "confidenceScore": 0.94,
                "ragStatus": "Completed"
            },
            {
                "id": "SMPL-RAG-002",
                "originalId": "SMPL-002",
                "name": "Sample 002 (RAG Processed)",
                "type": "RNA",
                "status": "RAG_Processing",
                "submittedBy": "Dr. Johnson",
                "submittedDate": (datetime.now() - timedelta(days=3)).isoformat(),
                "location": "Freezer A2-C1",
                "ragProcessingDate": (datetime.now() - timedelta(minutes=30)).isoformat(),
                "extractedMetadata": {
                    "concentration": "75 ng/μL",
                    "volume": "50 μL",
                    "quality": "Medium",
                    "extractionMethod": "TRIzol"
                },
                "confidenceScore": 0.87,
                "ragStatus": "Processing"
            },
            {
                "id": "SMPL-RAG-003",
                "originalId": "SMPL-003",
                "name": "Sample 003 (RAG Pending)",
                "type": "Protein",
                "status": "RAG_Pending",
                "submittedBy": "Dr. Williams",
                "submittedDate": datetime.now().isoformat(),
                "location": "Intake Bay",
                "extractedMetadata": {},
                "confidenceScore": 0.0,
                "ragStatus": "Pending"
            }
        ]

        # Return RAG samples with enhanced metadata
        return {
            "data": rag_samples_data,  # For frontend expecting .data.filter()
            "samples": rag_samples_data,  # For other consumers
            "totalCount": len(rag_samples_data),
            "page": 1,
            "pageSize": 10,
            "extractionMethod": "ai_rag",
            "processingStats": {
                "completed": 1,
                "processing": 1,
                "pending": 1,
                "failed": 0
            }
        }

    else:
        # Return regular samples for normal requests
        samples_data = [
            {
                "id": "SMPL-001",
                "name": "Sample 001",
                "type": "DNA",
                "status": "Processing",
                "submittedBy": "Dr. Smith",
                "submittedDate": (datetime.now() - timedelta(days=1)).isoformat(),
                "location": "Freezer A1-B2"
            },
            {
                "id": "SMPL-002",
                "name": "Sample 002",
                "type": "RNA",
                "status": "Completed",
                "submittedBy": "Dr. Johnson",
                "submittedDate": (datetime.now() - timedelta(days=3)).isoformat(),
                "location": "Freezer A2-C1"
            },
            {
                "id": "SMPL-003",
                "name": "Sample 003",
                "type": "Protein",
                "status": "Pending",
                "submittedBy": "Dr. Williams",
                "submittedDate": datetime.now().isoformat(),
                "location": "Intake Bay"
            }
        ]

        # Return both formats for compatibility
        return {
            "data": samples_data,  # For frontend expecting .data.filter()
            "samples": samples_data,  # For other consumers
            "totalCount": 1247,
            "page": 1,
            "pageSize": 10
        }

@app.post("/api/samples")
async def create_sample(request: Request):
    """Create a new sample"""
    # Mock sample creation
    return {
        "id": f"SMPL-{datetime.now().strftime('%Y%m%d%H%M%S')}",
        "status": "created",
        "message": "Sample created successfully"
    }

# Templates endpoints
@app.get("/api/templates")
async def get_templates():
    """Get all templates"""
    templates_data = [
        {
            "id": "TPL-001",
            "name": "DNA Extraction Template",
            "description": "Standard DNA extraction workflow",
            "category": "Extraction",
            "version": "1.2",
            "isActive": True,
            "createdDate": (datetime.now() - timedelta(days=30)).isoformat()
        },
        {
            "id": "TPL-002",
            "name": "RNA Sequencing Template",
            "description": "RNA-seq analysis pipeline",
            "category": "Sequencing",
            "version": "2.1",
            "isActive": True,
            "createdDate": (datetime.now() - timedelta(days=15)).isoformat()
        },
        {
            "id": "TPL-003",
            "name": "Protein Analysis Template",
            "description": "Protein characterization workflow",
            "category": "Analysis",
            "version": "1.0",
            "isActive": False,
            "createdDate": (datetime.now() - timedelta(days=60)).isoformat()
        }
    ]

    # Return array directly for the templates tab in ProjectManagement
    return templates_data

# Sequencing endpoints
@app.get("/api/sequencing/jobs")
async def get_sequencing_jobs():
    """Get sequencing jobs"""
    jobs_data = [
        {
            "id": "SEQ-001",
            "name": "Whole Genome Sequencing - Batch 1",
            "status": "Running",
            "progress": 67,
            "sampleCount": 24,
            "startTime": (datetime.now() - timedelta(hours=4)).isoformat(),
            "estimatedCompletion": (datetime.now() + timedelta(hours=2)).isoformat(),
            "instrument": "Illumina NovaSeq"
        },
        {
            "id": "SEQ-002",
            "name": "RNA-seq Analysis - Project Alpha",
            "status": "Completed",
            "progress": 100,
            "sampleCount": 48,
            "startTime": (datetime.now() - timedelta(days=2)).isoformat(),
            "completionTime": (datetime.now() - timedelta(hours=6)).isoformat(),
            "instrument": "Illumina MiSeq"
        },
        {
            "id": "SEQ-003",
            "name": "Targeted Sequencing - Panel B",
            "status": "Queued",
            "progress": 0,
            "sampleCount": 12,
            "queuePosition": 2,
            "estimatedStart": (datetime.now() + timedelta(hours=6)).isoformat(),
            "instrument": "Ion Torrent"
        }
    ]

    # Return both formats for compatibility
    return {
        "data": jobs_data,  # For frontend expecting .data.filter()
        "jobs": jobs_data,  # For other consumers
        "totalCount": 45,
        "activeJobs": 12,
        "queuedJobs": 8
    }

@app.post("/api/sequencing/jobs")
async def create_sequencing_job(request: Request):
    """Create a new sequencing job"""
    return {
        "id": f"SEQ-{datetime.now().strftime('%Y%m%d%H%M%S')}",
        "status": "created",
        "message": "Sequencing job created successfully"
    }

# Storage endpoints
@app.get("/api/storage/locations")
async def get_storage_locations():
    """Get storage locations"""
    locations_data = [
        {
            "id": "STOR-001",
            "name": "Freezer A1",
            "temperature": -80,
            "capacity": 1000,
            "occupied": 750,
            "status": "Normal"
        },
        {
            "id": "STOR-002",
            "name": "Refrigerator B2",
            "temperature": 4,
            "capacity": 500,
            "occupied": 320,
            "status": "Normal"
        }
    ]

    # Return both formats for compatibility
    return {
        "data": locations_data,  # For frontend expecting .data.filter()
        "locations": locations_data  # For other consumers
    }

# Storage Service Proxy (forward to actual storage service)
storage_service_url = os.getenv("STORAGE_SERVICE_URL", "http://storage-service:8000")

@app.get("/api/storage/health")
async def storage_health():
    """Check storage service health"""
    try:
        async with httpx.AsyncClient() as client:
            response = await client.get(f"{storage_service_url}/api/storage/health", timeout=5.0)
            return response.json()
    except Exception as e:
        return {"status": "unhealthy", "error": str(e)}, 503

@app.get("/api/storage/status")
async def storage_status():
    """Get storage service status"""
    try:
        async with httpx.AsyncClient() as client:
            response = await client.get(f"{storage_service_url}/api/storage/status", timeout=5.0)
            return response.json()
    except Exception as e:
        return {"operational": False, "error": str(e)}, 503

# Reports endpoints
@app.get("/api/reports/templates")
async def get_report_templates():
    """Get available report templates."""
    return {
        "data": [
            {
                "id": "RPT-001",
                "name": "Sample Summary Report",
                "description": "Summary of all samples in the system",
                "category": "samples",
                "sql": "SELECT status, COUNT(*) as count FROM samples GROUP BY status ORDER BY count DESC;",
                "created_at": datetime.now().isoformat(),
                "updated_at": datetime.now().isoformat()
            },
            {
                "id": "RPT-002",
                "name": "Storage Utilization Report",
                "description": "Current storage usage by temperature zone",
                "category": "storage",
                "sql": "SELECT temperature_zone, SUM(capacity) as total_capacity, SUM(current_usage) as total_usage FROM storage_locations GROUP BY temperature_zone;",
                "created_at": datetime.now().isoformat(),
                "updated_at": datetime.now().isoformat()
            },
            {
                "id": "RPT-003",
                "name": "Monthly Activity Report",
                "description": "Summary of all activities in the past month",
                "category": "activity",
                "sql": "SELECT DATE(created_at) as date, COUNT(*) as activity_count FROM samples WHERE created_at >= CURRENT_DATE - INTERVAL '30 days' GROUP BY DATE(created_at) ORDER BY date;",
                "created_at": datetime.now().isoformat(),
                "updated_at": datetime.now().isoformat()
            }
        ],
        "templates": [
            {
                "id": "RPT-001",
                "name": "Sample Summary Report",
                "description": "Summary of all samples in the system",
                "category": "samples",
                "sql": "SELECT status, COUNT(*) as count FROM samples GROUP BY status ORDER BY count DESC;"
            },
            {
                "id": "RPT-002",
                "name": "Storage Utilization Report",
                "description": "Current storage usage by temperature zone",
                "category": "storage",
                "sql": "SELECT temperature_zone, SUM(capacity) as total_capacity, SUM(current_usage) as total_usage FROM storage_locations GROUP BY temperature_zone;"
            },
            {
                "id": "RPT-003",
                "name": "Monthly Activity Report",
                "description": "Summary of all activities in the past month",
                "category": "activity",
                "sql": "SELECT DATE(created_at) as date, COUNT(*) as activity_count FROM samples WHERE created_at >= CURRENT_DATE - INTERVAL '30 days' GROUP BY DATE(created_at) ORDER BY date;"
            }
        ],
        "totalCount": 3
    }

@app.get("/api/reports/schema")
async def get_database_schema():
    """Get database schema information."""
    return {
        "tables": [
            {
                "name": "samples",
                "columns": [
                    {"name": "id", "type": "uuid", "nullable": False},
                    {"name": "name", "type": "varchar", "nullable": False},
                    {"name": "type", "type": "varchar", "nullable": False},
                    {"name": "status", "type": "varchar", "nullable": False},
                    {"name": "created_at", "type": "timestamp", "nullable": False},
                    {"name": "updated_at", "type": "timestamp", "nullable": False}
                ]
            },
            {
                "name": "storage_locations",
                "columns": [
                    {"name": "id", "type": "uuid", "nullable": False},
                    {"name": "name", "type": "varchar", "nullable": False},
                    {"name": "temperature_zone", "type": "varchar", "nullable": False},
                    {"name": "capacity", "type": "integer", "nullable": False},
                    {"name": "current_usage", "type": "integer", "nullable": False}
                ]
            },
            {
                "name": "users",
                "columns": [
                    {"name": "id", "type": "uuid", "nullable": False},
                    {"name": "email", "type": "varchar", "nullable": False},
                    {"name": "name", "type": "varchar", "nullable": False},
                    {"name": "role", "type": "varchar", "nullable": False},
                    {"name": "created_at", "type": "timestamp", "nullable": False}
                ]
            }
        ]
    }

@app.post("/api/reports/execute")
async def execute_report(request: Request):
    """Execute a custom SQL report."""
    # Mock response - in production this would execute the SQL safely
    return {
        "data": [
            {"sample_count": 150, "status": "active"},
            {"sample_count": 25, "status": "pending"},
            {"sample_count": 10, "status": "completed"}
        ],
        "columns": ["sample_count", "status"],
        "rowCount": 3,
        "executionTime": 0.125
    }

# RAG Service endpoints
@app.get("/api/rag/submissions")
async def get_rag_submissions():
    """Get RAG submissions"""
    submissions_data = [
        {
            "id": "RAG-001",
            "filename": "lab_report_2024_01.pdf",
            "status": "Processed",
            "submittedDate": (datetime.now() - timedelta(days=2)).isoformat(),
            "processedDate": (datetime.now() - timedelta(days=2, hours=2)).isoformat(),
            "extractedFields": 15,
            "confidenceScore": 0.92,
            "submittedBy": "Dr. Smith"
        },
        {
            "id": "RAG-002",
            "filename": "sample_manifest_batch_47.xlsx",
            "status": "Processing",
            "submittedDate": (datetime.now() - timedelta(hours=4)).isoformat(),
            "extractedFields": 8,
            "confidenceScore": 0.87,
            "submittedBy": "Lab Tech Johnson"
        },
        {
            "id": "RAG-003",
            "filename": "quality_control_summary.docx",
            "status": "Pending",
            "submittedDate": (datetime.now() - timedelta(minutes=30)).isoformat(),
            "submittedBy": "Dr. Williams"
        }
    ]

    # Return both formats for compatibility
    return {
        "data": submissions_data,  # For frontend expecting .data.filter()
        "submissions": submissions_data,  # For other consumers
        "totalCount": 127,
        "processing": 3,
        "completed": 118,
        "failed": 6
    }

@app.get("/api/rag/submissions/{submission_id}")
async def get_rag_submission_detail(submission_id: str):
    """Get detailed RAG submission information"""
    # Mock detailed submission data
    submission_details = {
        "RAG-001": {
            "id": "RAG-001",
            "submission_id": "RAG-001",
            "source_document": "lab_report_2024_01.pdf",
            "submitter_name": "Dr. Smith",
            "submitter_email": "dr.smith@lab.com",
            "confidence_score": 0.92,
            "processing_time": 2.3,
            "created_at": (datetime.now() - timedelta(days=2)).isoformat(),
            "status": "Processed",
            "samples_created": 15,
            "extracted_data": {
                "administrative_info": {
                    "submitter_name": "Dr. Smith",
                    "submitter_email": "dr.smith@lab.com",
                    "project_name": "Cancer Research Study 2024",
                    "institution": "Advanced Medical Research Lab"
                },
                "source_material": {
                    "sample_type": "Blood",
                    "source_organism": "Human",
                    "collection_date": "2024-01-15",
                    "collection_method": "Venipuncture"
                },
                "sample_details": {
                    "sample_count": 15,
                    "volume_per_sample": "5ml",
                    "container_type": "EDTA tubes",
                    "storage_temperature": "-80°C"
                }
            }
        },
        "RAG-002": {
            "id": "RAG-002",
            "submission_id": "RAG-002", 
            "source_document": "sample_manifest_batch_47.xlsx",
            "submitter_name": "Lab Tech Johnson",
            "submitter_email": "johnson@lab.com",
            "confidence_score": 0.87,
            "processing_time": 1.8,
            "created_at": (datetime.now() - timedelta(hours=4)).isoformat(),
            "status": "Processing",
            "samples_created": 8,
            "extracted_data": {
                "administrative_info": {
                    "submitter_name": "Lab Tech Johnson",
                    "submitter_email": "johnson@lab.com",
                    "project_name": "Batch 47 Processing",
                    "institution": "Clinical Testing Laboratory"
                },
                "source_material": {
                    "sample_type": "Tissue",
                    "source_organism": "Human",
                    "collection_date": "2024-06-28"
                }
            }
        },
        "RAG-003": {
            "id": "RAG-003",
            "submission_id": "RAG-003",
            "source_document": "quality_control_summary.docx",
            "submitter_name": "Dr. Williams",
            "submitter_email": "williams@lab.com",
            "confidence_score": 0.75,
            "processing_time": 0.0,
            "created_at": (datetime.now() - timedelta(minutes=30)).isoformat(),
            "status": "Pending",
            "samples_created": 0,
            "extracted_data": {}
        }
    }
    
    if submission_id not in submission_details:
        raise HTTPException(status_code=404, detail="Submission not found")
    
    return submission_details[submission_id]

@app.post("/api/rag/process")
async def process_rag_document(request: Request):
    """Process a document with RAG system"""
    try:
        # Handle multipart form data (file upload)
        form = await request.form()
        
        # Check if file is present
        file = form.get("file")
        if not file:
            raise HTTPException(status_code=400, detail="No file uploaded")
        
        # Get processing parameters
        auto_create_str = form.get("auto_create", "false")
        auto_create = str(auto_create_str).lower() == "true" if auto_create_str else False
        
        confidence_threshold_str = form.get("confidence_threshold", "0.8")
        confidence_threshold = float(str(confidence_threshold_str)) if confidence_threshold_str else 0.8
        
        # Generate document ID
        document_id = f"DOC-{datetime.now().strftime('%Y%m%d%H%M%S')}"
        
        # Mock extraction result (in production, this would process the actual file)
        extraction_result = {
            "success": True,
            "id": document_id,
            "status": "completed",
            "message": "Document processed successfully",
            "confidence_score": 0.92,
            "processing_time": 2.5,
            "samples": [
                {
                    "name": "Sample 001 from document",
                    "barcode": f"BC-{datetime.now().strftime('%Y%m%d')}-001",
                    "location": "Freezer A1",
                    "metadata": {
                        "type": "DNA",
                        "volume": "100 μL",
                        "concentration": "50 ng/μL"
                    }
                },
                {
                    "name": "Sample 002 from document", 
                    "barcode": f"BC-{datetime.now().strftime('%Y%m%d')}-002",
                    "location": "Freezer A2",
                    "metadata": {
                        "type": "RNA",
                        "volume": "50 μL",
                        "concentration": "75 ng/μL"
                    }
                }
            ],
            "validation_warnings": [] if confidence_threshold <= 0.9 else ["Some fields extracted with lower confidence"],
            "extraction_result": {
                "success": True,
                "confidence_score": 0.92,
                "warnings": [],
                "source_document": getattr(file, 'filename', 'uploaded_document')
            }
        }
        
        return extraction_result

    except HTTPException:
        raise
    except Exception as e:
        raise HTTPException(status_code=400, detail=f"Error processing document: {e!s}")

@app.post("/api/samples/rag/query")
async def query_rag_samples(request: Request):
    """Query RAG system for sample information"""
    try:
        # Get the request body
        body = await request.json()
        query = body.get("query", "")
        
        # Add logging
        print(f"[RAG QUERY] Received query: '{query}'", file=sys.stderr)
        
        # Mock RAG response based on query
        query_lower = query.lower()
        
        # Initialize related_samples
        related_samples = []
        
        # Check for AI document processing related queries
        ai_keywords = ["ai document", "document processing", "ai processing", "rag submission", "upload document"]
        is_ai_query = any(keyword in query_lower for keyword in ai_keywords)
        is_submit_query = "submit" in query_lower and ("sample" in query_lower or "document" in query_lower)
        
        print(f"[RAG QUERY] is_ai_query={is_ai_query}, is_submit_query={is_submit_query}", file=sys.stderr)
        
        if is_ai_query or is_submit_query:
            response_text = """To submit a new sample using AI document processing:

1. **Navigate to RAG Submissions**: Click on 'AI Document Submissions' from the main dashboard or go to the RAG Submissions page.

2. **Upload Your Document**: 
   - Drag and drop your document (PDF, DOCX, or TXT) into the upload area
   - Or click "Upload a file" to browse and select your document
   - Files up to 50MB are supported

3. **Configure Processing Options**:
   - Set the confidence threshold (default: 0.8) - lower values accept more extracted data
   - Check "Automatically create samples" if you want samples created immediately after extraction

4. **Process the Document**:
   - Click "Preview" to see what will be extracted without creating samples
   - Click "Process & Extract" to extract and create samples

5. **Review Results**:
   - The AI will extract sample information including names, barcodes, locations, and metadata
   - Check the confidence scores and any validation warnings
   - If in preview mode, you can "Confirm & Create Samples" after review

The AI system uses advanced language models to understand laboratory documents and extract structured data automatically. This saves time compared to manual data entry and reduces errors."""
        elif "samples" in query.lower():
            response_text = f"Based on your query '{query}', I found information about laboratory samples. Currently, there are 1,247 samples in the system with 89 active samples and 1,158 completed samples. The most recent samples were submitted by Dr. Smith and include DNA, RNA, and protein samples."
            related_samples = [
                {
                    "id": "SMPL-001",
                    "name": "Sample 001",
                    "type": "DNA",
                    "status": "Processing",
                    "submittedBy": "Dr. Smith",
                    "relevance": 0.95
                },
                {
                    "id": "SMPL-002",
                    "name": "Sample 002",
                    "type": "RNA",
                    "status": "Completed",
                    "submittedBy": "Dr. Johnson",
                    "relevance": 0.88
                }
            ]
        elif "storage" in query.lower():
            response_text = f"Regarding storage for '{query}', we have multiple storage locations including Freezer A1 (-80°C) with 750/1000 capacity and Refrigerator B2 (4°C) with 320/500 capacity. All storage units are operating normally."
            related_samples = [
                {
                    "id": "STOR-001",
                    "name": "Freezer A1",
                    "status": "Normal",
                    "relevance": 0.90
                }
            ]
        elif "sequencing" in query.lower():
            response_text = f"For sequencing information related to '{query}', there are currently 12 active sequencing jobs with 45 total jobs in the system. The most recent job is 'Whole Genome Sequencing - Batch 1' running at 67% progress on Illumina NovaSeq."
            related_samples = [
                {
                    "id": "SEQ-001",
                    "name": "Whole Genome Sequencing - Batch 1",
                    "status": "Running",
                    "progress": 67,
                    "relevance": 0.93
                }
            ]
        else:
            response_text = f"I understand you're asking about '{query}'. While I don't have specific information about this topic, I can help you with questions about samples, storage, sequencing, or other laboratory management topics."
            related_samples = []

        query_result = {
            "id": f"QRES-{datetime.now().strftime('%Y%m%d%H%M%S')}",
            "query": query,
            "response": response_text,
            "confidence": 0.85,
            "sources": [
                {
                    "document": "laboratory_database",
                    "section": "current_status",
                    "relevance": 0.92
                }
            ],
            "relatedItems": related_samples,
            "timestamp": datetime.now().isoformat()
        }

        # Return both single result and data array format for compatibility
        return {
            "data": [query_result],  # For frontend expecting .data.filter()
            "result": query_result,  # Single result for other consumers
            "relatedSamples": related_samples  # Direct access to related items
        }

    except Exception as e:
        raise HTTPException(status_code=400, detail=f"Invalid query: {e!s}")

@app.post("/api/rag/submissions")
async def create_rag_submission(request: Request):
    """Create a new RAG submission"""
    return {
        "id": f"RAG-{datetime.now().strftime('%Y%m%d%H%M%S')}",
        "status": "submitted",
        "message": "Document submitted for processing"
    }

# Debug endpoint to test query logic
@app.post("/api/debug/test-query")
async def test_query(request: Request):
    """Test query logic"""
    try:
        body = await request.json()
        query = body.get("query", "")
        query_lower = query.lower()
        
        ai_keywords = ["ai document", "document processing", "ai processing", "rag submission", "upload document"]
        is_ai_query = any(keyword in query_lower for keyword in ai_keywords)
        is_submit_query = "submit" in query_lower and ("sample" in query_lower or "document" in query_lower)
        
        return {
            "query": query,
            "query_lower": query_lower,
            "is_ai_query": is_ai_query,
            "is_submit_query": is_submit_query,
            "should_match": is_ai_query or is_submit_query,
            "ai_keyword_matches": [kw for kw in ai_keywords if kw in query_lower]
        }
    except Exception as e:
        return {"error": str(e)}

# Debug endpoint to see all registered routes
@app.get("/api/debug/routes")
async def debug_routes():
    """Debug endpoint to see all registered routes"""
    routes = []
    for route in app.routes:
        if hasattr(route, "path") and hasattr(route, "methods"):
            routes.append({
                "path": route.path,
                "methods": list(route.methods) if route.methods else ["GET"]
            })
    return {"routes": sorted(routes, key=lambda x: x["path"])}

# Note: Removed catch-all endpoint to prevent route conflicts

@app.get("/api/v1/status")
async def api_status():
    """API status endpoint"""
    services = {
        "api-gateway": "healthy",
        "rag-service": "unknown"
    }

    # Check RAG service health
    try:
        async with httpx.AsyncClient() as client:
            response = await client.get(f"{RAG_SERVICE_URL}/api/v1/health", timeout=5.0)
            if response.status_code == 200:
                services["rag-service"] = "healthy"
            else:
                services["rag-service"] = "unhealthy"
    except Exception:
        services["rag-service"] = "unreachable"

    return {
        "services": services,
        "overall": "healthy" if all(s in ["healthy", "unknown"] for s in services.values()) else "degraded"
    }

@app.api_route("/api/v1/rag/{path:path}", methods=["GET", "POST", "PUT", "DELETE", "PATCH"])
async def proxy_rag(path: str, request: Request):
    """Proxy requests to RAG service"""
    try:
        async with httpx.AsyncClient() as client:
            # Forward the request to RAG service
            url = f"{RAG_SERVICE_URL}/{path}"

            # Get request body if present
            body = None
            if request.method in ["POST", "PUT", "PATCH"]:
                body = await request.body()

            response = await client.request(
                method=request.method,
                url=url,
                headers=dict(request.headers),
                params=request.query_params,
                content=body,
                timeout=30.0
            )

            return response.json() if response.headers.get("content-type", "").startswith("application/json") else response.text

    except Exception as e:
        raise HTTPException(status_code=502, detail=f"Service unavailable: {e!s}")

# Proxy routes for other services
@app.api_route("/api/auth/{path:path}", methods=["GET", "POST", "PUT", "DELETE", "PATCH"])
async def proxy_auth(path: str, request: Request):
    """Proxy requests to Auth service"""
    try:
        async with httpx.AsyncClient() as client:
            url = f"{AUTH_SERVICE_URL}/api/v1/{path}"
            body = None
            if request.method in ["POST", "PUT", "PATCH"]:
                body = await request.body()

            response = await client.request(
                method=request.method,
                url=url,
                headers=dict(request.headers),
                params=request.query_params,
                content=body,
                timeout=30.0
            )
            return response.json()
    except Exception as e:
        raise HTTPException(status_code=502, detail=f"Auth service unavailable: {e!s}")

@app.api_route("/api/sequencing/{path:path}", methods=["GET", "POST", "PUT", "DELETE", "PATCH"])
async def proxy_sequencing(path: str, request: Request):
    """Proxy requests to Sequencing service"""
    try:
        async with httpx.AsyncClient() as client:
            # For health checks, use the direct health endpoint
            if path == "health":
                url = f"{SEQUENCING_SERVICE_URL}/health"
            else:
                url = f"{SEQUENCING_SERVICE_URL}/api/v1/{path}"
            body = None
            if request.method in ["POST", "PUT", "PATCH"]:
                body = await request.body()

            response = await client.request(
                method=request.method,
                url=url,
                headers=dict(request.headers),
                params=request.query_params,
                content=body,
                timeout=30.0
            )
            return response.json()
    except Exception as e:
        raise HTTPException(status_code=502, detail=f"Sequencing service unavailable: {e!s}")

@app.api_route("/api/notification/{path:path}", methods=["GET", "POST", "PUT", "DELETE", "PATCH"])
async def proxy_notification(path: str, request: Request):
    """Proxy requests to Notification service"""
    try:
        async with httpx.AsyncClient() as client:
            url = f"{NOTIFICATION_SERVICE_URL}/api/v1/{path}"
            body = None
            if request.method in ["POST", "PUT", "PATCH"]:
                body = await request.body()

            response = await client.request(
                method=request.method,
                url=url,
                headers=dict(request.headers),
                params=request.query_params,
                content=body,
                timeout=30.0
            )
            return response.json()
    except Exception as e:
        raise HTTPException(status_code=502, detail=f"Notification service unavailable: {e!s}")

# Enhanced services proxy routes
@app.api_route("/api/notifications/{path:path}", methods=["GET", "POST", "PUT", "DELETE", "PATCH"])
async def proxy_notifications(path: str, request: Request):
    """Proxy requests to Notifications service (plural)"""
    notification_url = "http://lims-notification:8000"
    try:
        async with httpx.AsyncClient() as client:
            # For health checks, use the direct health endpoint
            if path == "health":
                url = f"{notification_url}/health"
            else:
                url = f"{notification_url}/api/v1/{path}"
            
            body = None
            if request.method in ["POST", "PUT", "PATCH"]:
                body = await request.body()

            response = await client.request(
                method=request.method,
                url=url,
                headers=dict(request.headers),
                params=request.query_params,
                content=body,
                timeout=30.0
            )
            
            # Return the response content and status
            return Response(
                content=response.content,
                status_code=response.status_code,
                headers=dict(response.headers)
            )
    except httpx.ConnectError:
        raise HTTPException(status_code=503, detail="Notifications service unavailable")
    except Exception as e:
        raise HTTPException(status_code=502, detail=f"Notifications service error: {e!s}")

@app.api_route("/api/events/{path:path}", methods=["GET", "POST", "PUT", "DELETE", "PATCH"])
async def proxy_events(path: str, request: Request):
    """Proxy requests to Events service"""
    events_url = "http://lims-events:8087"
    try:
        async with httpx.AsyncClient() as client:
            # For health checks, use the direct health endpoint
            if path == "health":
                url = f"{events_url}/health"
            else:
                url = f"{events_url}/api/v1/{path}"
            
            body = None
            if request.method in ["POST", "PUT", "PATCH"]:
                body = await request.body()

            response = await client.request(
                method=request.method,
                url=url,
                headers=dict(request.headers),
                params=request.query_params,
                content=body,
                timeout=30.0
            )
            
            # Return the response content and status
            return Response(
                content=response.content,
                status_code=response.status_code,
                headers=dict(response.headers)
            )
    except httpx.ConnectError:
        raise HTTPException(status_code=503, detail="Events service unavailable")
    except Exception as e:
        raise HTTPException(status_code=502, detail=f"Events service error: {e!s}")

@app.api_route("/api/transactions/{path:path}", methods=["GET", "POST", "PUT", "DELETE", "PATCH"])
async def proxy_transactions(path: str, request: Request):
    """Proxy requests to Transactions service"""
    try:
        async with httpx.AsyncClient() as client:
            # For health checks, use the direct health endpoint
            if path == "health":
                url = f"{TRANSACTION_SERVICE_URL}/health"
            else:
                url = f"{TRANSACTION_SERVICE_URL}/api/v1/{path}"
            
            body = None
            if request.method in ["POST", "PUT", "PATCH"]:
                body = await request.body()

            response = await client.request(
                method=request.method,
                url=url,
                headers=dict(request.headers),
                params=request.query_params,
                content=body,
                timeout=30.0
            )
            
            # Return the response content and status
            return Response(
                content=response.content,
                status_code=response.status_code,
                headers=dict(response.headers)
            )
    except httpx.ConnectError:
        raise HTTPException(status_code=503, detail="Transactions service unavailable")
    except Exception as e:
        raise HTTPException(status_code=502, detail=f"Transactions service error: {e!s}")

@app.api_route("/api/qaqc/{path:path}", methods=["GET", "POST", "PUT", "DELETE", "PATCH"])
async def proxy_qaqc(path: str, request: Request):
    """Proxy requests to QA/QC service"""
    # Mock health check for QA/QC while the service is being fixed
    if path == "health" and request.method == "GET":
        return JSONResponse({
            "service": "qaqc-service",
            "status": "unavailable",
            "message": "Service binary issue - being fixed",
            "timestamp": datetime.now().isoformat()
        }, status_code=503)
    
    try:
        async with httpx.AsyncClient() as client:
            # For health checks, use the direct health endpoint
            if path == "health":
                url = f"{QAQC_SERVICE_URL}/health"
            else:
                url = f"{QAQC_SERVICE_URL}/api/v1/{path}"
            body = None
            if request.method in ["POST", "PUT", "PATCH"]:
                body = await request.body()

            response = await client.request(
                method=request.method,
                url=url,
                headers=dict(request.headers),
                params=request.query_params,
                content=body,
                timeout=30.0
            )
            return Response(
                content=response.content,
                status_code=response.status_code,
                headers=dict(response.headers),
                media_type=response.headers.get("content-type", "application/json")
            )
    except Exception as e:
        print(f"Error proxying to QA/QC service: {e}")
        return JSONResponse({"error": "Service unavailable"}, status_code=503)

@app.api_route("/api/templates/{path:path}", methods=["GET", "POST", "PUT", "DELETE", "PATCH"])
async def proxy_templates(path: str, request: Request):
    """Proxy requests to Templates service"""
    try:
        async with httpx.AsyncClient() as client:
            # For health checks, use the direct health endpoint
            if path == "health":
                url = f"{TEMPLATE_SERVICE_URL}/health"
            else:
                url = f"{TEMPLATE_SERVICE_URL}/api/v1/{path}"
            body = None
            if request.method in ["POST", "PUT", "PATCH"]:
                body = await request.body()

            response = await client.request(
                method=request.method,
                url=url,
                headers=dict(request.headers),
                params=request.query_params,
                content=body,
                timeout=30.0
            )
            return Response(
                content=response.content,
                status_code=response.status_code,
                headers=dict(response.headers),
                media_type=response.headers.get("content-type", "application/json")
            )
    except Exception as e:
        print(f"Error proxying to Templates service: {e}")
        return JSONResponse({"error": "Service unavailable"}, status_code=503)

# NOTE: Removed proxy endpoints - using direct endpoints below
# Project endpoints
@app.get("/api/projects")
async def get_projects():
    """Get all projects"""
    # Return empty list for now - in production, this would query the database
    return []

@app.get("/api/projects/batches")
async def get_batches():
    """Get all batches"""
    # Return empty list for now
    return []

@app.post("/api/projects")
async def create_project(project: dict):
    """Create a new project"""
    # In production, this would save to database
    return {"id": str(uuid.uuid4()), **project}

@app.get("/api/projects/{project_id}")
async def get_project(project_id: str):
    """Get a specific project"""
    # Return mock project
    return {
        "id": project_id,
        "project_code": "PROJ-2024-001",
        "name": "Sample Project",
        "status": "active",
        "priority": "high",
        "created_at": datetime.now().isoformat()
    }

@app.get("/api/projects/{project_id}/files")
async def get_project_files(project_id: str):
    """Get project files"""
    return []

@app.get("/api/projects/{project_id}/signoffs")
async def get_project_signoffs(project_id: str):
    """Get project signoffs"""
    return []

# NOTE: Removed proxy endpoints for library prep - using direct endpoints below
# Library Prep endpoints
@app.get("/api/library-prep/preparations")
async def get_library_preparations():
    """Get library preparations"""
    return []

@app.get("/api/library-prep/protocols/active")
async def get_active_protocols():
    """Get active library prep protocols"""
    return []

@app.post("/api/library-prep/preparations")
async def create_library_prep(prep: dict):
    """Create a new library preparation"""
    return {"id": str(uuid.uuid4()), **prep}

@app.get("/api/library-prep/protocols")
async def get_protocols():
    """Get all library prep protocols"""
    return []

# Proxy endpoints for QC
# QC endpoints
@app.get("/api/qc/reviews")
async def get_qc_reviews():
    """Get QC reviews"""
    return []

@app.get("/api/qc/metrics")
async def get_qc_metrics():
    """Get QC metrics"""
    return []

@app.get("/api/qc/control-samples")
async def get_control_samples():
    """Get control samples"""
    return []

@app.post("/api/qc/reviews")
async def create_qc_review(review: dict):
    """Create a new QC review"""
    return {"id": str(uuid.uuid4()), **review}

# Flow Cell endpoints
@app.get("/api/flow-cells/types")
async def get_flow_cell_types():
    """Get flow cell types"""
    return [
        {"id": str(uuid.uuid4()), "name": "NovaSeq S4", "lane_count": 4, "reads_per_lane": 3200000000},
        {"id": str(uuid.uuid4()), "name": "NovaSeq S2", "lane_count": 2, "reads_per_lane": 1650000000},
        {"id": str(uuid.uuid4()), "name": "MiSeq v3", "lane_count": 1, "reads_per_lane": 25000000}
    ]

@app.post("/api/flow-cells/designs")
async def create_flow_cell_design(design: dict):
    """Create a new flow cell design"""
    return {"id": str(uuid.uuid4()), **design}

@app.post("/api/flow-cells/optimize")
async def optimize_flow_cell(optimization_request: dict):
    """Optimize flow cell design"""
    # Mock optimization result
    return {
        "optimized": True,
        "lane_assignments": [],
        "balance_score": 0.95,
        "estimated_reads": 3200000000
    }

# Add dedicated endpoint for RAG samples search
@app.get("/api/rag/samples")
async def get_rag_samples():
    """Get RAG-processed samples"""
    rag_samples_data = [
        {
            "id": "SMPL-RAG-001",
            "originalId": "SMPL-001",
            "name": "Sample 001 (RAG Processed)",
            "type": "DNA",
            "status": "RAG_Analyzed",
            "submittedBy": "Dr. Smith",
            "submittedDate": (datetime.now() - timedelta(days=1)).isoformat(),
            "ragProcessingDate": (datetime.now() - timedelta(hours=2)).isoformat(),
            "extractedMetadata": {
                "concentration": "50 ng/μL",
                "volume": "100 μL",
                "quality": "High",
                "extractionMethod": "Qiagen DNeasy"
            },
            "confidenceScore": 0.94,
            "ragStatus": "Completed"
        },
        {
            "id": "SMPL-RAG-002",
            "originalId": "SMPL-002",
            "name": "Sample 002 (RAG Processed)",
            "type": "RNA",
            "status": "RAG_Processing",
            "submittedBy": "Dr. Johnson",
            "submittedDate": (datetime.now() - timedelta(days=3)).isoformat(),
            "ragProcessingDate": (datetime.now() - timedelta(minutes=30)).isoformat(),
            "extractedMetadata": {
                "concentration": "75 ng/μL",
                "volume": "50 μL",
                "quality": "Medium",
                "extractionMethod": "TRIzol"
            },
            "confidenceScore": 0.87,
            "ragStatus": "Processing"
        },
        {
            "id": "SMPL-RAG-003",
            "originalId": "SMPL-003",
            "name": "Sample 003 (RAG Pending)",
            "type": "Protein",
            "status": "RAG_Pending",
            "submittedBy": "Dr. Williams",
            "submittedDate": datetime.now().isoformat(),
            "extractedMetadata": {},
            "confidenceScore": 0.0,
            "ragStatus": "Pending"
        }
    ]

    # Return both formats for compatibility
    return {
        "data": rag_samples_data,  # For frontend expecting .data.filter()
        "ragSamples": rag_samples_data,  # For other consumers
        "totalCount": len(rag_samples_data),
        "processing": 1,
        "completed": 1,
        "pending": 1
    }

@app.post("/api/rag/samples/search")
async def search_rag_samples(request: Request):
    """Search RAG-processed samples"""
    try:
        body = await request.json()
        search_term = body.get("searchTerm", "")

        # Mock search results
        search_results = [
            {
                "id": "SMPL-RAG-001",
                "name": "Sample 001 (RAG Processed)",
                "type": "DNA",
                "status": "RAG_Analyzed",
                "relevance": 0.95,
                "matchedFields": ["name", "type"],
                "extractedMetadata": {
                    "concentration": "50 ng/μL",
                    "volume": "100 μL",
                    "quality": "High"
                }
            }
        ] if search_term else []

        return {
            "data": search_results,  # For frontend expecting .data.filter()
            "searchResults": search_results,  # For other consumers
            "query": search_term,
            "totalResults": len(search_results),
            "timestamp": datetime.now().isoformat()
        }

    except Exception as e:
        raise HTTPException(status_code=400, detail=f"Invalid search request: {e!s}")

# Spreadsheet endpoints for dataset management
@app.get("/api/spreadsheets/datasets")
async def get_spreadsheet_datasets():
    """Get spreadsheet datasets"""
    datasets_data = [
        {
            "id": "DS-001",
            "name": "Sample Tracking Dataset",
            "description": "Main sample tracking spreadsheet with QC data",
            "fileName": "sample_tracking_2024.xlsx",
            "version": "1.3",
            "lastModified": (datetime.now() - timedelta(hours=2)).isoformat(),
            "createdBy": "Dr. Smith",
            "status": "Active",
            "recordCount": 1247,
            "columns": [
                {"name": "Sample_ID", "type": "string", "required": True},
                {"name": "Type", "type": "enum", "values": ["DNA", "RNA", "Protein"]},
                {"name": "Concentration", "type": "number", "unit": "ng/μL"},
                {"name": "Volume", "type": "number", "unit": "μL"},
                {"name": "Quality_Score", "type": "number", "range": [0, 100]},
                {"name": "Storage_Location", "type": "string"},
                {"name": "Submitted_Date", "type": "date"},
                {"name": "Status", "type": "enum", "values": ["Pending", "Processing", "Completed", "Failed"]}
            ]
        },
        {
            "id": "DS-002",
            "name": "Sequencing Results Dataset",
            "description": "Sequencing job results and quality metrics",
            "fileName": "sequencing_results_2024.xlsx",
            "version": "2.1",
            "lastModified": (datetime.now() - timedelta(days=1)).isoformat(),
            "createdBy": "Lab Tech Johnson",
            "status": "Active",
            "recordCount": 456,
            "columns": [
                {"name": "Job_ID", "type": "string", "required": True},
                {"name": "Sample_Count", "type": "number"},
                {"name": "Platform", "type": "enum", "values": ["Illumina NovaSeq", "Illumina MiSeq", "Ion Torrent"]},
                {"name": "Coverage", "type": "number", "unit": "X"},
                {"name": "Quality_Score", "type": "number", "range": [0, 40]},
                {"name": "Completion_Date", "type": "date"},
                {"name": "Output_Files", "type": "array"}
            ]
        },
        {
            "id": "DS-003",
            "name": "Storage Inventory Dataset",
            "description": "Current storage locations and capacity tracking",
            "fileName": "storage_inventory_2024.xlsx",
            "version": "1.0",
            "lastModified": (datetime.now() - timedelta(hours=6)).isoformat(),
            "createdBy": "Dr. Williams",
            "status": "Active",
            "recordCount": 2500,
            "columns": [
                {"name": "Location_ID", "type": "string", "required": True},
                {"name": "Temperature", "type": "number", "unit": "°C"},
                {"name": "Capacity", "type": "number"},
                {"name": "Occupied", "type": "number"},
                {"name": "Utilization", "type": "number", "unit": "%"},
                {"name": "Last_Check", "type": "datetime"}
            ]
        }
    ]

    # Return both formats for compatibility
    return {
        "data": datasets_data,  # For frontend expecting .data.filter()
        "datasets": datasets_data,  # For other consumers
        "totalCount": len(datasets_data),
        "activeDatasets": 3,
        "totalRecords": sum(ds["recordCount"] for ds in datasets_data)
    }

@app.post("/api/spreadsheets/datasets")
async def create_spreadsheet_dataset(request: Request):
    """Create a new spreadsheet dataset"""
    try:
        body = await request.json()
        dataset_name = body.get("name", f"Dataset-{datetime.now().strftime('%Y%m%d%H%M%S')}")

        return {
            "id": f"DS-{datetime.now().strftime('%Y%m%d%H%M%S')}",
            "name": dataset_name,
            "status": "created",
            "message": "Spreadsheet dataset created successfully",
            "version": "1.0",
            "createdDate": datetime.now().isoformat()
        }

    except Exception as e:
        raise HTTPException(status_code=400, detail=f"Invalid dataset creation request: {e!s}")

@app.get("/api/spreadsheets/datasets/{dataset_id}")
async def get_spreadsheet_dataset(dataset_id: str):
    """Get specific spreadsheet dataset"""
    # Mock dataset details
    return {
        "data": {
            "id": dataset_id,
            "name": f"Dataset {dataset_id}",
            "description": "Detailed dataset information",
            "version": "1.0",
            "records": [
                {
                    "Sample_ID": "SMPL-001",
                    "Type": "DNA",
                    "Concentration": 50.0,
                    "Volume": 100.0,
                    "Quality_Score": 95,
                    "Storage_Location": "Freezer A1-B2",
                    "Status": "Completed"
                },
                {
                    "Sample_ID": "SMPL-002",
                    "Type": "RNA",
                    "Concentration": 75.0,
                    "Volume": 50.0,
                    "Quality_Score": 88,
                    "Storage_Location": "Freezer A2-C1",
                    "Status": "Processing"
                }
            ]
        },
        "totalRecords": 2,
        "lastModified": datetime.now().isoformat()
    }

# Redirect handlers for double /api URLs (frontend routing issue)
@app.get("/api/api/storage/locations")
async def redirect_storage_locations():
    """Redirect handler for double /api prefix - frontend routing issue"""
    # Just redirect to the correct endpoint internally
    return await get_storage_locations()

@app.get("/api/api/samples")
async def redirect_samples(request: Request):
    """Redirect handler for double /api prefix - frontend routing issue"""
    return await get_samples(request)

@app.get("/api/api/templates")
async def redirect_templates():
    """Redirect handler for double /api prefix - frontend routing issue"""
    return await get_templates()

@app.get("/api/api/sequencing/jobs")
async def redirect_sequencing_jobs():
    """Redirect handler for double /api prefix - frontend routing issue"""
    return await get_sequencing_jobs()

# Additional debug endpoint to help identify which endpoint is causing filter issues
@app.get("/api/debug/problematic-endpoints")
async def debug_problematic_endpoints():
    """Debug endpoint to test all data formats"""
    test_results = {}

    # Test each endpoint's data format
    endpoints_to_test = [
        ("/api/samples", "samples"),
        ("/api/templates", "templates"),
        ("/api/sequencing/jobs", "sequencing jobs"),
        ("/api/storage/locations", "storage locations"),
        ("/api/rag/submissions", "rag submissions"),
        ("/api/rag/samples", "rag samples"),
        ("/api/spreadsheets/datasets", "spreadsheet datasets")
    ]

    for endpoint, name in endpoints_to_test:
        try:
            # Simulate a request to test data format
            if endpoint == "/api/samples":
                # Create a mock request for testing
                from fastapi import Request
                from starlette.datastructures import QueryParams
                mock_request = type("MockRequest", (), {"query_params": QueryParams("")})()
                response = await get_samples(mock_request)
            elif endpoint == "/api/templates":
                response = await get_templates()
            elif endpoint == "/api/sequencing/jobs":
                response = await get_sequencing_jobs()
            elif endpoint == "/api/storage/locations":
                response = await get_storage_locations()
            elif endpoint == "/api/rag/submissions":
                response = await get_rag_submissions()
            elif endpoint == "/api/rag/samples":
                response = await get_rag_samples()
            elif endpoint == "/api/spreadsheets/datasets":
                response = await get_spreadsheet_datasets()

            test_results[name] = {
                "endpoint": endpoint,
                "has_data_field": "data" in response,
                "data_is_array": isinstance(response.get("data"), list) if "data" in response else False,
                "data_length": len(response.get("data", [])) if isinstance(response.get("data"), list) else 0,
                "data_type": str(type(response.get("data", None))),
                "status": "✅ OK" if (isinstance(response.get("data"), list) and len(response.get("data", [])) > 0) else "❌ ISSUE"
            }
        except Exception as e:
            test_results[name] = {
                "endpoint": endpoint,
                "error": str(e),
                "status": "❌ ERROR"
            }

    return {
        "debug_info": "Testing all endpoints for proper array data format",
        "test_results": test_results,
        "timestamp": datetime.now().isoformat()
    }

if __name__ == "__main__":
    # Get configuration from environment
    host = os.getenv("HOST", "0.0.0.0")
    port = int(os.getenv("PORT", "8000"))
    log_level = os.getenv("LOG_LEVEL", "info").lower()

    print(f"🚀 Starting TracSeq 2.0 API Gateway on {host}:{port}")
    print(f"📊 RAG Service URL: {RAG_SERVICE_URL}")

    # Run the application
    uvicorn.run(
        app,
        host=host,
        port=port,
        log_level=log_level,
        access_log=True
    )
