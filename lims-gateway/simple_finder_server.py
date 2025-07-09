#!/usr/bin/env python3

"""
Simple FastAPI server to provide missing endpoints for the Finder
"""

from fastapi import FastAPI, HTTPException
from fastapi.middleware.cors import CORSMiddleware
from datetime import datetime
import uvicorn
import uuid

app = FastAPI(title="Simple Finder API", version="1.0.0")

# Add CORS middleware
app.add_middleware(
    CORSMiddleware,
    allow_origins=["*"],
    allow_credentials=True,
    allow_methods=["*"],
    allow_headers=["*"],
)

@app.get("/health")
async def health_check():
    return {
        "status": "healthy", 
        "service": "api-gateway", 
        "timestamp": datetime.now().isoformat(),
        "database": {
            "healthy": True,
            "type": "mock",
            "info": {
                "database_name": "mock_db",
                "current_user": "mock_user",
                "version": "Mock Database v1.0",
                "server_time": datetime.now().isoformat(),
                "database_size": "Mock",
                "table_count": 10,
                "active_connections": 1
            }
        },
        "features": {
            "standardized_db": True,
            "enhanced_monitoring": True
        }
    }

@app.get("/api/projects")
async def get_projects():
    """Get all projects"""
    return {
        "data": [
            {
                "id": str(uuid.uuid4()),
                "name": "Sample Analysis Project",
                "project_code": "PROJ-2024-001",
                "project_type": "Research",
                "status": "active",
                "priority": "high",
                "department": "Genomics",
                "budget_approved": 50000,
                "budget_used": 25000,
                "description": "Analysis of genomic samples for research purposes",
                "created_at": datetime.now().isoformat(),
                "updated_at": datetime.now().isoformat()
            },
            {
                "id": str(uuid.uuid4()),
                "name": "Clinical Validation Study",
                "project_code": "PROJ-2024-002",
                "project_type": "Clinical",
                "status": "active",
                "priority": "medium",
                "department": "Clinical",
                "budget_approved": 75000,
                "budget_used": 30000,
                "description": "Validation study for clinical diagnostic assays",
                "created_at": datetime.now().isoformat(),
                "updated_at": datetime.now().isoformat()
            },
            {
                "id": str(uuid.uuid4()),
                "name": "Quality Control Testing",
                "project_code": "PROJ-2024-003",
                "project_type": "QC",
                "status": "completed",
                "priority": "low",
                "department": "Quality",
                "budget_approved": 25000,
                "budget_used": 24500,
                "description": "Routine quality control testing procedures",
                "created_at": datetime.now().isoformat(),
                "updated_at": datetime.now().isoformat()
            }
        ]
    }

@app.get("/api/reports")
async def get_reports():
    """Get all generated reports"""
    return {
        "data": [
            {
                "id": str(uuid.uuid4()),
                "name": "Monthly Sample Report",
                "format": "PDF",
                "status": "completed",
                "file_path": "/reports/monthly_sample_report.pdf",
                "file_size": 2048576,
                "description": "Monthly summary of all sample activities",
                "created_at": datetime.now().isoformat(),
                "completed_at": datetime.now().isoformat()
            },
            {
                "id": str(uuid.uuid4()),
                "name": "Storage Utilization Report",
                "format": "Excel",
                "status": "completed",
                "file_path": "/reports/storage_utilization.xlsx",
                "file_size": 1024000,
                "description": "Current storage usage across all temperature zones",
                "created_at": datetime.now().isoformat(),
                "completed_at": datetime.now().isoformat()
            },
            {
                "id": str(uuid.uuid4()),
                "name": "Quality Metrics Dashboard",
                "format": "HTML",
                "status": "generating",
                "file_path": "/reports/quality_metrics.html",
                "file_size": 512000,
                "description": "Interactive dashboard showing quality metrics",
                "created_at": datetime.now().isoformat(),
                "completed_at": None
            }
        ]
    }

@app.get("/api/samples")
async def get_samples():
    """Get all samples"""
    return {
        "data": [
            {
                "id": str(uuid.uuid4()),
                "name": "Sample-001",
                "barcode": "SMPL-001-2024",
                "status": "InStorage",
                "location": "Freezer-A-01",
                "created_at": datetime.now().isoformat(),
                "updated_at": datetime.now().isoformat(),
                "description": "DNA sample from patient cohort A",
                "metadata": {
                    "sample_type": "DNA",
                    "concentration_ng_ul": 125.5,
                    "project": "PROJ-2024-001",
                    "volume_ul": 50.0
                }
            },
            {
                "id": str(uuid.uuid4()),
                "name": "Sample-002",
                "barcode": "SMPL-002-2024",
                "status": "Validated",
                "location": "Freezer-B-02",
                "created_at": datetime.now().isoformat(),
                "updated_at": datetime.now().isoformat(),
                "description": "RNA sample from patient cohort B",
                "metadata": {
                    "sample_type": "RNA",
                    "concentration_ng_ul": 89.2,
                    "project": "PROJ-2024-002",
                    "volume_ul": 75.0
                }
            },
            {
                "id": str(uuid.uuid4()),
                "name": "Sample-003",
                "barcode": "SMPL-003-2024",
                "status": "InSequencing",
                "location": "Sequencer-01",
                "created_at": datetime.now().isoformat(),
                "updated_at": datetime.now().isoformat(),
                "description": "Protein sample for mass spectrometry",
                "metadata": {
                    "sample_type": "Protein",
                    "concentration_ng_ul": 200.0,
                    "project": "PROJ-2024-003",
                    "volume_ul": 25.0
                }
            }
        ]
    }

@app.get("/api/templates")
async def get_templates():
    """Get all templates"""
    return {
        "data": [
            {
                "id": str(uuid.uuid4()),
                "name": "Sample Submission Template",
                "version": "v2.1",
                "is_active": True,
                "description": "Standard template for sample submission forms",
                "created_at": datetime.now().isoformat(),
                "updated_at": datetime.now().isoformat()
            },
            {
                "id": str(uuid.uuid4()),
                "name": "Quality Control Template",
                "version": "v1.5",
                "is_active": True,
                "description": "Template for quality control documentation",
                "created_at": datetime.now().isoformat(),
                "updated_at": datetime.now().isoformat()
            },
            {
                "id": str(uuid.uuid4()),
                "name": "Project Proposal Template",
                "version": "v3.0",
                "is_active": False,
                "description": "Template for new project proposals",
                "created_at": datetime.now().isoformat(),
                "updated_at": datetime.now().isoformat()
            }
        ]
    }

# RAG Submission endpoints
@app.get("/api/rag/submissions")
async def get_rag_submissions():
    """Get RAG submissions with uploaded documents"""
    return {
        "data": [
            {
                "id": str(uuid.uuid4()),
                "filename": "sample_submission_001.pdf",
                "status": "Processed",
                "submittedDate": datetime.now().isoformat(),
                "submittedBy": "Dr. Jane Smith",
                "submitterEmail": "jane.smith@lab.com",
                "confidenceScore": 0.92,
                "extracted_data": {
                    "sample_name": "DNA-001",
                    "sample_type": "DNA",
                    "concentration": "125.5 ng/ul",
                    "volume": "50 ul",
                    "submitter": "Dr. Jane Smith"
                },
                "file_path": "/uploads/sample_submission_001.pdf",
                "file_size": 1024000
            },
            {
                "id": str(uuid.uuid4()),
                "filename": "clinical_samples_batch_02.xlsx",
                "status": "Processing",
                "submittedDate": datetime.now().isoformat(),
                "submittedBy": "Lab Technician",
                "submitterEmail": "tech@lab.com",
                "confidenceScore": 0.85,
                "extracted_data": {
                    "batch_id": "BATCH-002",
                    "sample_count": 24,
                    "sample_type": "RNA",
                    "project": "Clinical Study 2024"
                },
                "file_path": "/uploads/clinical_samples_batch_02.xlsx",
                "file_size": 2048000
            },
            {
                "id": str(uuid.uuid4()),
                "filename": "quality_control_report.pdf",
                "status": "Processed",
                "submittedDate": datetime.now().isoformat(),
                "submittedBy": "QC Manager",
                "submitterEmail": "qc@lab.com",
                "confidenceScore": 0.96,
                "extracted_data": {
                    "qc_batch": "QC-2024-001",
                    "test_results": "PASSED",
                    "instruments_checked": 5,
                    "issues_found": 0
                },
                "file_path": "/uploads/quality_control_report.pdf",
                "file_size": 512000
            }
        ]
    }

# Additional endpoints for file operations
@app.get("/api/files/{file_id}")
async def get_file(file_id: str):
    """Get file metadata"""
    return {
        "id": file_id,
        "name": f"file_{file_id}.pdf",
        "size": 1024000,
        "type": "application/pdf",
        "created_at": datetime.now().isoformat(),
        "download_url": f"/api/files/{file_id}/download"
    }

@app.get("/api/files/{file_id}/download")
async def download_file(file_id: str):
    """Download file content"""
    # In a real implementation, this would serve the actual file
    return {"message": f"File {file_id} download would start here"}

@app.post("/api/files/{file_id}/open")
async def open_file(file_id: str):
    """Open file for viewing"""
    return {
        "success": True,
        "file_id": file_id,
        "viewer_url": f"/viewer/{file_id}",
        "message": "File opened successfully"
    }

if __name__ == "__main__":
    print("🚀 Starting Simple Finder API server on port 8089...")
    uvicorn.run(app, host="0.0.0.0", port=8089) 