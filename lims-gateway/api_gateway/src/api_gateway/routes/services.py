"""
Service proxy routes for TracSeq 2.0 API Gateway.

This module provides proxy routing to all microservices including:
- Templates Service
- Samples Service  
- Storage Service
- Reports Service
- Sequencing Service
- Notification Service
- Events Service
- Transactions Service
- QA/QC Service
"""

import os
import httpx
from typing import Optional, Dict, Any
from fastapi import APIRouter, HTTPException, Request, Response, Form, File, UploadFile
from fastapi.responses import JSONResponse, StreamingResponse

router = APIRouter()

# Service URLs from environment variables (matching Docker container names)
AUTH_SERVICE_URL = os.getenv("AUTH_SERVICE_URL", "http://lims-auth:8000")
SAMPLE_SERVICE_URL = os.getenv("SAMPLE_SERVICE_URL", "http://lims-samples:8000")
STORAGE_SERVICE_URL = os.getenv("STORAGE_SERVICE_URL", "http://lims-storage:8082")
TEMPLATE_SERVICE_URL = os.getenv("TEMPLATE_SERVICE_URL", "http://lims-templates:8000")
REPORTS_SERVICE_URL = os.getenv("REPORTS_SERVICE_URL", "http://lims-reports:8000")
RAG_SERVICE_URL = os.getenv("RAG_SERVICE_URL", "http://lims-rag:8000")

# Additional services that may not be running but should be supported
SEQUENCING_SERVICE_URL = os.getenv("SEQUENCING_SERVICE_URL", "http://lims-sequencing:8000")
NOTIFICATION_SERVICE_URL = os.getenv("NOTIFICATION_SERVICE_URL", "http://lims-notification:8000")
EVENT_SERVICE_URL = os.getenv("EVENT_SERVICE_URL", "http://lims-events:8000")
TRANSACTION_SERVICE_URL = os.getenv("TRANSACTION_SERVICE_URL", "http://lims-transactions:8000")
QAQC_SERVICE_URL = os.getenv("QAQC_SERVICE_URL", "http://lims-qaqc:8000")
PROJECT_SERVICE_URL = os.getenv("PROJECT_SERVICE_URL", "http://lims-projects:8000")
LIBRARY_PREP_SERVICE_URL = os.getenv("LIBRARY_PREP_SERVICE_URL", "http://lims-library-prep:8000")
FLOW_CELL_SERVICE_URL = os.getenv("FLOW_CELL_SERVICE_URL", "http://lims-flow-cells:8000")
SPREADSHEET_SERVICE_URL = os.getenv("SPREADSHEET_SERVICE_URL", "http://lims-spreadsheets:8000")

async def proxy_request(
    service_url: str,
    path: str,
    request: Request,
    timeout: float = 30.0
) -> Response:
    """
    Generic proxy function to forward requests to microservices.
    
    Args:
        service_url: Base URL of the target service
        path: API path to forward to
        request: Original FastAPI request
        timeout: Request timeout in seconds
        
    Returns:
        Response from the target service
    """
    try:
        async with httpx.AsyncClient() as client:
            # Build target URL
            if path.startswith('/'):
                target_url = f"{service_url}{path}"
            else:
                target_url = f"{service_url}/{path}"
            
            # Get request body if present
            body = None
            if request.method in ["POST", "PUT", "PATCH"]:
                body = await request.body()
            
            # Forward headers (excluding host)
            headers = dict(request.headers)
            headers.pop("host", None)
            headers.pop("content-length", None)
            
            # Make the request
            response = await client.request(
                method=request.method,
                url=target_url,
                headers=headers,
                params=request.query_params,
                content=body,
                timeout=timeout
            )
            
            # Return response with same status code and headers
            return Response(
                content=response.content,
                status_code=response.status_code,
                headers=dict(response.headers),
                media_type=response.headers.get("content-type")
            )
            
    except httpx.ConnectError:
        raise HTTPException(
            status_code=503, 
            detail=f"Service unavailable: {service_url}"
        )
    except httpx.TimeoutException:
        raise HTTPException(
            status_code=504, 
            detail=f"Service timeout: {service_url}"
        )
    except Exception as e:
        raise HTTPException(
            status_code=502, 
            detail=f"Service error: {str(e)}"
        )

# Templates Service Proxy Routes
@router.get("/templates", tags=["templates"])
async def get_templates(request: Request):
    """Get all templates"""
    return await proxy_request(TEMPLATE_SERVICE_URL, "/templates", request)

@router.post("/templates", tags=["templates"])
async def create_template(request: Request):
    """Create a new template"""
    return await proxy_request(TEMPLATE_SERVICE_URL, "/templates", request)

@router.get("/templates/{template_id}", tags=["templates"])
async def get_template(request: Request, template_id: str):
    """Get a specific template"""
    return await proxy_request(TEMPLATE_SERVICE_URL, f"/templates/{template_id}", request)

@router.put("/templates/{template_id}", tags=["templates"])
async def update_template(request: Request, template_id: str):
    """Update a template"""
    return await proxy_request(TEMPLATE_SERVICE_URL, f"/templates/{template_id}", request)

@router.delete("/templates/{template_id}", tags=["templates"])
async def delete_template(request: Request, template_id: str):
    """Delete a template"""
    return await proxy_request(TEMPLATE_SERVICE_URL, f"/templates/{template_id}", request)

@router.get("/templates/{template_id}/data", tags=["templates"])
async def get_template_data(request: Request, template_id: str):
    """Get template data (spreadsheet content)"""
    return await proxy_request(TEMPLATE_SERVICE_URL, f"/templates/{template_id}/data", request)

@router.post("/templates/upload", tags=["templates"])
async def upload_template(request: Request):
    """Upload a new template file"""
    return await proxy_request(TEMPLATE_SERVICE_URL, "/templates/upload", request)

@router.get("/templates/health", tags=["templates"])
async def templates_health():
    """Templates service health check"""
    try:
        async with httpx.AsyncClient() as client:
            response = await client.get(f"{TEMPLATE_SERVICE_URL}/health", timeout=5.0)
            return response.json()
    except Exception as e:
        raise HTTPException(status_code=503, detail=f"Templates service unavailable: {str(e)}")

# Samples Service Proxy Routes  
@router.get("/samples", tags=["samples"])
async def get_samples(request: Request):
    """Get all samples"""
    return await proxy_request(SAMPLE_SERVICE_URL, "/samples", request)

@router.post("/samples", tags=["samples"])
async def create_sample(request: Request):
    """Create a new sample"""
    return await proxy_request(SAMPLE_SERVICE_URL, "/samples", request)

@router.get("/samples/{sample_id}", tags=["samples"])
async def get_sample(request: Request, sample_id: str):
    """Get a specific sample"""
    return await proxy_request(SAMPLE_SERVICE_URL, f"/samples/{sample_id}", request)

@router.put("/samples/{sample_id}", tags=["samples"])
async def update_sample(request: Request, sample_id: str):
    """Update a sample"""
    return await proxy_request(SAMPLE_SERVICE_URL, f"/samples/{sample_id}", request)

@router.delete("/samples/{sample_id}", tags=["samples"])
async def delete_sample(request: Request, sample_id: str):
    """Delete a sample"""
    return await proxy_request(SAMPLE_SERVICE_URL, f"/samples/{sample_id}", request)

# Storage Service Proxy Routes
@router.get("/storage", tags=["storage"])
async def get_storage(request: Request):
    """Get storage information"""
    return await proxy_request(STORAGE_SERVICE_URL, "/api/storage/locations", request)

@router.get("/storage/locations", tags=["storage"])
async def get_storage_locations(request: Request):
    """Get storage locations"""
    return await proxy_request(STORAGE_SERVICE_URL, "/api/storage/locations", request)

@router.post("/storage/locations", tags=["storage"])
async def create_storage_location(request: Request):
    """Create storage location"""
    return await proxy_request(STORAGE_SERVICE_URL, "/api/storage/locations", request)

@router.get("/storage/locations/{location_id}", tags=["storage"])
async def get_storage_location(request: Request, location_id: str):
    """Get specific storage location"""
    return await proxy_request(STORAGE_SERVICE_URL, f"/api/storage/locations/{location_id}", request)

@router.get("/storage/containers", tags=["storage"])
async def get_storage_containers(request: Request):
    """Get storage containers"""
    return await proxy_request(STORAGE_SERVICE_URL, "/api/storage/containers", request)

@router.get("/storage/containers/{container_id}", tags=["storage"])
async def get_storage_container(request: Request, container_id: str):
    """Get specific storage container"""
    return await proxy_request(STORAGE_SERVICE_URL, f"/api/storage/containers/{container_id}", request)

@router.get("/storage/health", tags=["storage"])
async def storage_health():
    """Storage service health check"""
    try:
        async with httpx.AsyncClient() as client:
            response = await client.get(f"{STORAGE_SERVICE_URL}/health", timeout=5.0)
            return response.json()
    except Exception as e:
        raise HTTPException(status_code=503, detail=f"Storage service unavailable: {str(e)}")

@router.get("/storage/{path:path}", tags=["storage"])
async def get_storage_path(request: Request, path: str):
    """Get storage path information"""
    return await proxy_request(STORAGE_SERVICE_URL, f"/api/storage/{path}", request)

@router.post("/storage/{path:path}", tags=["storage"])
async def post_storage_path(request: Request, path: str):
    """Post to storage path"""
    return await proxy_request(STORAGE_SERVICE_URL, f"/api/storage/{path}", request)

# Reports Service Proxy Routes
@router.get("/reports", tags=["reports"])
async def get_reports(request: Request):
    """Get all reports"""
    return await proxy_request(REPORTS_SERVICE_URL, "/api/reports", request)

@router.post("/reports", tags=["reports"])
async def create_report(request: Request):
    """Create a new report"""
    return await proxy_request(REPORTS_SERVICE_URL, "/api/reports", request)

# Analytics Reports - MUST be before /reports/{report_id} to avoid conflicts
@router.get("/reports/analytics/samples", tags=["reports"])
async def get_sample_analytics(request: Request):
    """Get sample analytics"""
    return await proxy_request(REPORTS_SERVICE_URL, "/api/reports/analytics/samples", request)

@router.get("/reports/analytics/sequencing", tags=["reports"])
async def get_sequencing_analytics(request: Request):
    """Get sequencing analytics"""
    return await proxy_request(REPORTS_SERVICE_URL, "/api/reports/analytics/sequencing", request)

@router.get("/reports/analytics/storage", tags=["reports"])
async def get_storage_analytics(request: Request):
    """Get storage analytics"""
    return await proxy_request(REPORTS_SERVICE_URL, "/api/reports/analytics/storage", request)

@router.get("/reports/analytics/financial", tags=["reports"])
async def get_financial_analytics(request: Request):
    """Get financial analytics"""
    return await proxy_request(REPORTS_SERVICE_URL, "/api/reports/analytics/financial", request)

@router.get("/reports/analytics/performance", tags=["reports"])
async def get_performance_analytics(request: Request):
    """Get performance analytics"""
    return await proxy_request(REPORTS_SERVICE_URL, "/api/reports/analytics/performance", request)

# Report Templates - also must be before /reports/{report_id}
@router.get("/reports/templates", tags=["reports"])
async def get_report_templates(request: Request):
    """Get all report templates"""
    return await proxy_request(REPORTS_SERVICE_URL, "/api/reports/templates", request)

@router.post("/reports/templates", tags=["reports"])
async def create_report_template(request: Request):
    """Create a new report template"""
    return await proxy_request(REPORTS_SERVICE_URL, "/api/reports/templates", request)

@router.get("/reports/templates/{template_id}", tags=["reports"])
async def get_report_template(request: Request, template_id: str):
    """Get a specific report template"""
    return await proxy_request(REPORTS_SERVICE_URL, f"/api/reports/templates/{template_id}", request)

# Report Schedules - also must be before /reports/{report_id}
@router.get("/reports/schedules", tags=["reports"])
async def get_report_schedules(request: Request):
    """Get all report schedules"""
    return await proxy_request(REPORTS_SERVICE_URL, "/api/reports/schedules", request)

@router.post("/reports/schedules", tags=["reports"])
async def create_report_schedule(request: Request):
    """Create a new report schedule"""
    return await proxy_request(REPORTS_SERVICE_URL, "/api/reports/schedules", request)

@router.get("/reports/schedules/{schedule_id}", tags=["reports"])
async def get_report_schedule(request: Request, schedule_id: str):
    """Get a specific report schedule"""
    return await proxy_request(REPORTS_SERVICE_URL, f"/api/reports/schedules/{schedule_id}", request)

@router.put("/reports/schedules/{schedule_id}", tags=["reports"])
async def update_report_schedule(request: Request, schedule_id: str):
    """Update a report schedule"""
    return await proxy_request(REPORTS_SERVICE_URL, f"/api/reports/schedules/{schedule_id}", request)

@router.delete("/reports/schedules/{schedule_id}", tags=["reports"])
async def delete_report_schedule(request: Request, schedule_id: str):
    """Delete a report schedule"""
    return await proxy_request(REPORTS_SERVICE_URL, f"/api/reports/schedules/{schedule_id}", request)

# Export endpoints - also must be before /reports/{report_id}
@router.post("/reports/export/pdf", tags=["reports"])
async def export_pdf(request: Request):
    """Export report as PDF"""
    return await proxy_request(REPORTS_SERVICE_URL, "/api/reports/export/pdf", request)

@router.post("/reports/export/excel", tags=["reports"])
async def export_excel(request: Request):
    """Export report as Excel"""
    return await proxy_request(REPORTS_SERVICE_URL, "/api/reports/export/excel", request)

@router.post("/reports/export/csv", tags=["reports"])
async def export_csv(request: Request):
    """Export report as CSV"""
    return await proxy_request(REPORTS_SERVICE_URL, "/api/reports/export/csv", request)

# Custom queries - also must be before /reports/{report_id}
@router.post("/reports/query", tags=["reports"])
async def execute_query(request: Request):
    """Execute custom query"""
    return await proxy_request(REPORTS_SERVICE_URL, "/api/reports/query", request)

@router.get("/reports/query/saved", tags=["reports"])
async def get_saved_queries(request: Request):
    """Get saved queries"""
    return await proxy_request(REPORTS_SERVICE_URL, "/api/reports/query/saved", request)

@router.post("/reports/query/saved", tags=["reports"])
async def save_query(request: Request):
    """Save query"""
    return await proxy_request(REPORTS_SERVICE_URL, "/api/reports/query/saved", request)

@router.get("/reports/schema", tags=["reports"])
async def get_reports_schema(request: Request):
    """Get reports schema"""
    return await proxy_request(REPORTS_SERVICE_URL, "/api/reports/schema", request)

# Health check
@router.get("/reports/health", tags=["reports"])
async def reports_health():
    """Reports service health check"""
    try:
        async with httpx.AsyncClient() as client:
            response = await client.get(f"{REPORTS_SERVICE_URL}/health", timeout=5.0)
            return response.json()
    except Exception as e:
        raise HTTPException(status_code=503, detail=f"Reports service unavailable: {str(e)}")

# Generate report - also must be before /reports/{report_id}
@router.post("/reports/generate", tags=["reports"])
async def generate_report(request: Request):
    """Generate a new report"""
    return await proxy_request(REPORTS_SERVICE_URL, "/api/reports/generate", request)

# General report routes - MUST be LAST to avoid conflicts
@router.get("/reports/{report_id}", tags=["reports"])
async def get_report(request: Request, report_id: str):
    """Get a specific report"""
    return await proxy_request(REPORTS_SERVICE_URL, f"/api/reports/{report_id}", request)

@router.get("/reports/{report_id}/download", tags=["reports"])
async def download_report(request: Request, report_id: str):
    """Download a specific report"""
    return await proxy_request(REPORTS_SERVICE_URL, f"/api/reports/{report_id}/download", request)

# Projects Service Proxy Routes (handled by samples service for now)
@router.get("/projects", tags=["projects"])
async def get_projects(request: Request):
    """Get all projects"""
    return await proxy_request(SAMPLE_SERVICE_URL, "/api/projects", request)

@router.post("/projects", tags=["projects"])
async def create_project(request: Request):
    """Create a new project"""
    return await proxy_request(SAMPLE_SERVICE_URL, "/api/projects", request)

@router.get("/projects/{project_id}", tags=["projects"])
async def get_project(request: Request, project_id: str):
    """Get a specific project"""
    return await proxy_request(SAMPLE_SERVICE_URL, f"/api/projects/{project_id}", request)

# RAG/AI Service Proxy Routes
@router.get("/rag/health", tags=["rag"])
async def rag_health():
    """RAG service health check"""
    try:
        async with httpx.AsyncClient() as client:
            response = await client.get(f"{RAG_SERVICE_URL}/health", timeout=5.0)
            return response.json()
    except Exception as e:
        raise HTTPException(status_code=503, detail=f"RAG service unavailable: {str(e)}")

# File-based persistence for RAG submissions
import json
import os
from datetime import datetime
from pathlib import Path

RAG_SUBMISSIONS_FILE = Path("/tmp/rag_submissions.json")

def load_rag_submissions():
    """Load RAG submissions from file"""
    if RAG_SUBMISSIONS_FILE.exists():
        try:
            with open(RAG_SUBMISSIONS_FILE, 'r') as f:
                return json.load(f)
        except Exception:
            pass
    return []

def save_rag_submissions(submissions):
    """Save RAG submissions to file"""
    try:
        with open(RAG_SUBMISSIONS_FILE, 'w') as f:
            json.dump(submissions, f, indent=2, default=str)
    except Exception as e:
        print(f"Error saving submissions: {e}")

@router.get("/rag/submissions", tags=["rag"])
async def get_rag_submissions():
    """Get all RAG submissions with persistence"""
    try:
        # Load existing submissions
        submissions = load_rag_submissions()
        
        # If no submissions exist, create some sample data
        if not submissions:
            submissions = [
                {
                    "id": "RAG-001",
                    "submission_id": "RAG-001",
                    "source_document": "lab_report_2024_01.pdf",
                    "submitter_name": "Dr. Smith",
                    "submitter_email": "dr.smith@lab.com",
                    "sample_type": "DNA",
                    "confidence_score": 0.92,
                    "status": "completed",
                    "created_at": datetime.now().isoformat(),
                    "processing_time": 2.3,
                    "extracted_data": {
                        "administrative": {
                            "submitter_name": "Dr. Smith",
                            "submitter_email": "dr.smith@lab.com",
                            "institution": "Research Lab"
                        },
                        "sample": {
                            "sample_type": "DNA",
                            "volume": "50µL",
                            "concentration": "100ng/µL"
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
                    "created_at": datetime.now().isoformat(),
                    "processing_time": 1.8,
                    "extracted_data": {
                        "administrative": {
                            "submitter_name": "Dr. Johnson",
                            "submitter_email": "dr.johnson@lab.com",
                            "institution": "University Lab"
                        },
                        "sample": {
                            "sample_type": "RNA",
                            "volume": "25µL",
                            "concentration": "200ng/µL"
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
                    "created_at": datetime.now().isoformat(),
                    "processing_time": 3.1,
                    "extracted_data": {
                        "administrative": {
                            "submitter_name": "Dr. Brown",
                            "submitter_email": "dr.brown@lab.com",
                            "institution": "Biotech Lab"
                        },
                        "sample": {
                            "sample_type": "Protein",
                            "volume": "100µL",
                            "concentration": "500ng/µL"
                        }
                    }
                }
            ]
            save_rag_submissions(submissions)
        
        return {
            "submissions": submissions,
            "totalCount": len(submissions),
            "processing": len([s for s in submissions if s["status"] == "processing"]),
            "completed": len([s for s in submissions if s["status"] == "completed"]),
            "failed": len([s for s in submissions if s["status"] == "failed"])
        }
    except Exception as e:
        raise HTTPException(status_code=500, detail=f"Failed to fetch RAG submissions: {str(e)}")

@router.post("/rag/process", tags=["rag"])
async def process_rag_document(request: Request):
    """Process document via RAG service and store result"""
    try:
        # Forward to RAG service
        async with httpx.AsyncClient() as client:
            # Get the request body and headers
            body = await request.body()
            headers = dict(request.headers)
            
            # Remove host header to avoid conflicts
            headers.pop('host', None)
            
            response = await client.post(
                f"{RAG_SERVICE_URL}/api/rag/process",
                content=body,
                headers=headers,
                timeout=30.0
            )
            
            if response.status_code == 200:
                result = response.json()
                
                # If processing was successful, store the submission
                if result.get("success") and result.get("submission_id"):
                    submissions = load_rag_submissions()
                    
                    # Create new submission record
                    new_submission = {
                        "id": result["submission_id"],
                        "submission_id": result["submission_id"],
                        "source_document": f"uploaded_document_{result['submission_id']}.pdf",
                        "submitter_name": "User Upload",
                        "submitter_email": "user@example.com",
                        "sample_type": "Unknown",
                        "confidence_score": result.get("confidence_score", 0.0),
                        "status": "completed" if result.get("samples_found", 0) > 0 else "processing",
                        "created_at": datetime.now().isoformat(),
                        "processing_time": result.get("processing_time", 0.0),
                        "extracted_data": result.get("extracted_data", {})
                    }
                    
                    submissions.append(new_submission)
                    save_rag_submissions(submissions)
                
                return response.json()
            else:
                raise HTTPException(status_code=response.status_code, detail=response.text)
                
    except Exception as e:
        raise HTTPException(status_code=500, detail=f"RAG processing failed: {str(e)}")

@router.get("/rag/submissions/{submission_id}", tags=["rag"])
async def get_rag_submission(submission_id: str):
    """Get specific RAG submission by ID"""
    try:
        submissions = load_rag_submissions()
        submission = next((s for s in submissions if s["id"] == submission_id), None)
        
        if not submission:
            raise HTTPException(status_code=404, detail="Submission not found")
        
        return submission
    except HTTPException:
        raise
    except Exception as e:
        raise HTTPException(status_code=500, detail=f"Failed to fetch submission: {str(e)}")

@router.post("/rag/submissions", tags=["rag"])
async def create_rag_submission(request: Request):
    """Create new RAG submission"""
    return await proxy_request(RAG_SERVICE_URL, "/api/rag/submissions", request)

@router.get("/rag/samples", tags=["rag"])
async def get_rag_samples():
    """Get RAG samples data"""
    try:
        # Get submissions data
        submissions = load_rag_submissions()
        
        # Transform submissions into samples format
        samples = []
        for submission in submissions:
            sample_data = submission.get("extracted_data", {}).get("sample", {})
            samples.append({
                "id": submission["id"],
                "submission_id": submission["submission_id"],
                "source_document": submission["source_document"],
                "sample_type": sample_data.get("sample_type", submission.get("sample_type", "Unknown")),
                "volume": sample_data.get("volume", "N/A"),
                "concentration": sample_data.get("concentration", "N/A"),
                "status": submission["status"],
                "confidence_score": submission["confidence_score"],
                "created_at": submission["created_at"]
            })
        
        return {
            "data": samples,
            "submissions": submissions,
            "totalCount": 127,  # Mock total for demo
            "processing": len([s for s in submissions if s["status"] == "processing"]),
            "completed": len([s for s in submissions if s["status"] == "completed"]),
            "failed": len([s for s in submissions if s["status"] == "failed"])
        }
    except Exception as e:
        raise HTTPException(status_code=500, detail=f"Failed to fetch RAG samples: {str(e)}")

@router.post("/rag/query", tags=["rag"])
async def query_rag_system(request: Request):
    """Query RAG system"""
    return await proxy_request(RAG_SERVICE_URL, "/api/rag/query", request)

@router.delete("/rag/submissions/{submission_id}", tags=["rag"])
async def delete_rag_submission(submission_id: str):
    """Delete RAG submission"""
    try:
        submissions = load_rag_submissions()
        submissions = [s for s in submissions if s["id"] != submission_id]
        save_rag_submissions(submissions)
        return {"message": "Submission deleted successfully"}
    except Exception as e:
        raise HTTPException(status_code=500, detail=f"Failed to delete submission: {str(e)}")

@router.put("/rag/submissions/{submission_id}", tags=["rag"])
async def update_rag_submission(submission_id: str, request: Request):
    """Update RAG submission"""
    return await proxy_request(RAG_SERVICE_URL, f"/api/rag/submissions/{submission_id}", request)

# Health check routes for services that may not be running
@router.get("/sequencing/health", tags=["sequencing"])
async def sequencing_health():
    """Sequencing service health check"""
    try:
        async with httpx.AsyncClient() as client:
            response = await client.get(f"{SEQUENCING_SERVICE_URL}/health", timeout=5.0)
            return response.json()
    except Exception as e:
        raise HTTPException(status_code=503, detail=f"Sequencing service unavailable: {str(e)}")

@router.get("/notifications/health", tags=["notifications"])
async def notifications_health():
    """Notifications service health check"""
    try:
        async with httpx.AsyncClient() as client:
            response = await client.get(f"{NOTIFICATION_SERVICE_URL}/health", timeout=5.0)
            return response.json()
    except Exception as e:
        raise HTTPException(status_code=503, detail=f"Notifications service unavailable: {str(e)}")

@router.get("/events/health", tags=["events"])
async def events_health():
    """Events service health check"""
    try:
        async with httpx.AsyncClient() as client:
            response = await client.get(f"{EVENT_SERVICE_URL}/health", timeout=5.0)
            return response.json()
    except Exception as e:
        raise HTTPException(status_code=503, detail=f"Events service unavailable: {str(e)}")

@router.get("/transactions/health", tags=["transactions"])
async def transactions_health():
    """Transactions service health check"""
    try:
        async with httpx.AsyncClient() as client:
            response = await client.get(f"{TRANSACTION_SERVICE_URL}/health", timeout=5.0)
            return response.json()
    except Exception as e:
        raise HTTPException(status_code=503, detail=f"Transactions service unavailable: {str(e)}")

@router.get("/qaqc/health", tags=["qaqc"])
async def qaqc_health():
    """QA/QC service health check"""
    try:
        async with httpx.AsyncClient() as client:
            response = await client.get(f"{QAQC_SERVICE_URL}/health", timeout=5.0)
            return response.json()
    except Exception as e:
        # QA/QC service might not be running, return a mock response
        return JSONResponse({
            "service": "qaqc-service",
            "status": "unavailable",
            "message": "Service binary issue - being fixed",
            "timestamp": "2025-01-01T00:00:00Z"
        }, status_code=503)

# Service Status Overview
@router.get("/services", tags=["services"])
async def get_services_status():
    """Get status of all microservices"""
    services = {
        "auth": AUTH_SERVICE_URL,
        "samples": SAMPLE_SERVICE_URL,
        "storage": STORAGE_SERVICE_URL,
        "templates": TEMPLATE_SERVICE_URL,
        "reports": REPORTS_SERVICE_URL,
        "rag": RAG_SERVICE_URL,
        "sequencing": SEQUENCING_SERVICE_URL,
        "notifications": NOTIFICATION_SERVICE_URL,
        "events": EVENT_SERVICE_URL,
        "transactions": TRANSACTION_SERVICE_URL,
        "qaqc": QAQC_SERVICE_URL,
        "projects": PROJECT_SERVICE_URL,
        "library_prep": LIBRARY_PREP_SERVICE_URL,
        "flow_cells": FLOW_CELL_SERVICE_URL,
        "spreadsheets": SPREADSHEET_SERVICE_URL,
    }
    
    status = {}
    
    async with httpx.AsyncClient() as client:
        for service_name, service_url in services.items():
            try:
                response = await client.get(f"{service_url}/health", timeout=3.0)
                if response.status_code == 200:
                    status[service_name] = "healthy"
                else:
                    status[service_name] = "unhealthy"
            except:
                status[service_name] = "unreachable"
    
    return {
        "services": status,
        "overall": "healthy" if all(s in ["healthy"] for s in status.values()) else "degraded"
    }



# =============================================================================
# LIBRARY PREP SERVICE PROXY ROUTES  
# =============================================================================

@router.get("/library-prep/preparations", tags=["library-prep"])
async def get_library_preparations(request: Request):
    """Get all library preparations"""
    return await proxy_request(LIBRARY_PREP_SERVICE_URL, "/api/v1/preparations", request)

@router.post("/library-prep/preparations", tags=["library-prep"])
async def create_library_preparation(request: Request):
    """Create a new library preparation"""
    return await proxy_request(LIBRARY_PREP_SERVICE_URL, "/api/v1/preparations", request)

@router.get("/library-prep/preparations/{prep_id}", tags=["library-prep"])
async def get_library_preparation(prep_id: str, request: Request):
    """Get library preparation by ID"""
    return await proxy_request(LIBRARY_PREP_SERVICE_URL, f"/api/v1/preparations/{prep_id}", request)

@router.get("/library-prep/protocols", tags=["library-prep"])
async def get_library_protocols(request: Request):
    """Get all library prep protocols"""
    return await proxy_request(LIBRARY_PREP_SERVICE_URL, "/api/v1/protocols", request)

@router.get("/library-prep/protocols/active", tags=["library-prep"])
async def get_active_library_protocols(request: Request):
    """Get active library prep protocols"""
    return await proxy_request(LIBRARY_PREP_SERVICE_URL, "/api/v1/protocols/active", request)

# =============================================================================
# FLOW CELLS SERVICE PROXY ROUTES
# =============================================================================

@router.get("/flow-cells/types", tags=["flow-cells"])
async def get_flow_cell_types(request: Request):
    """Get all flow cell types"""
    return await proxy_request(FLOW_CELL_SERVICE_URL, "/api/v1/types", request)

@router.get("/flow-cells", tags=["flow-cells"])
async def get_flow_cells(request: Request):
    """Get all flow cells"""
    return await proxy_request(FLOW_CELL_SERVICE_URL, "/api/v1/flow-cells", request)

@router.post("/flow-cells", tags=["flow-cells"])
async def create_flow_cell(request: Request):
    """Create a new flow cell"""
    return await proxy_request(FLOW_CELL_SERVICE_URL, "/api/v1/flow-cells", request)

@router.get("/flow-cells/{flow_cell_id}", tags=["flow-cells"])
async def get_flow_cell(flow_cell_id: str, request: Request):
    """Get flow cell by ID"""
    return await proxy_request(FLOW_CELL_SERVICE_URL, f"/api/v1/flow-cells/{flow_cell_id}", request)

# =============================================================================
# QC SERVICE PROXY ROUTES
# =============================================================================

@router.get("/qc/dashboard/stats", tags=["qc"])
async def get_qc_dashboard_stats(request: Request):
    """Get QC dashboard statistics"""
    return await proxy_request(QAQC_SERVICE_URL, "/api/v1/dashboard/stats", request)

@router.get("/qc/reviews", tags=["qc"])
async def get_qc_reviews(request: Request):
    """Get QC reviews"""
    return await proxy_request(QAQC_SERVICE_URL, "/api/v1/reviews", request)

@router.post("/qc/reviews", tags=["qc"])
async def create_qc_review(request: Request):
    """Create a new QC review"""
    return await proxy_request(QAQC_SERVICE_URL, "/api/v1/reviews", request)

@router.get("/qc/metrics", tags=["qc"])
async def get_qc_metrics(request: Request):
    """Get QC metrics"""
    return await proxy_request(QAQC_SERVICE_URL, "/api/v1/metrics", request)

@router.get("/qc/metrics/recent", tags=["qc"])
async def get_recent_qc_metrics(request: Request):
    """Get recent QC metrics"""
    return await proxy_request(QAQC_SERVICE_URL, "/api/v1/metrics/recent", request)

@router.get("/qc/reviews/{review_id}", tags=["qc"])
async def get_qc_review(review_id: str, request: Request):
    """Get QC review by ID"""
    return await proxy_request(QAQC_SERVICE_URL, f"/api/v1/reviews/{review_id}", request)

# =============================================================================
# SEQUENCING SERVICE PROXY ROUTES
# =============================================================================

@router.get("/sequencing/jobs", tags=["sequencing"])
async def get_sequencing_jobs(request: Request):
    """Get all sequencing jobs"""
    return await proxy_request(SEQUENCING_SERVICE_URL, "/api/v1/jobs", request)

@router.post("/sequencing/jobs", tags=["sequencing"])
async def create_sequencing_job(request: Request):
    """Create a new sequencing job"""
    return await proxy_request(SEQUENCING_SERVICE_URL, "/api/v1/jobs", request)

@router.get("/sequencing/jobs/{job_id}", tags=["sequencing"])
async def get_sequencing_job(job_id: str, request: Request):
    """Get sequencing job by ID"""
    return await proxy_request(SEQUENCING_SERVICE_URL, f"/api/v1/jobs/{job_id}", request)

@router.put("/sequencing/jobs/{job_id}", tags=["sequencing"])
async def update_sequencing_job(job_id: str, request: Request):
    """Update sequencing job"""
    return await proxy_request(SEQUENCING_SERVICE_URL, f"/api/v1/jobs/{job_id}", request)

@router.get("/sequencing/runs", tags=["sequencing"])
async def get_sequencing_runs(request: Request):
    """Get all sequencing runs"""
    return await proxy_request(SEQUENCING_SERVICE_URL, "/api/v1/runs", request)

@router.get("/sequencing/platforms", tags=["sequencing"])
async def get_sequencing_platforms(request: Request):
    """Get available sequencing platforms"""
    return await proxy_request(SEQUENCING_SERVICE_URL, "/api/v1/platforms", request)

# =============================================================================
# SPREADSHEETS SERVICE PROXY ROUTES
# =============================================================================

@router.get("/spreadsheets/datasets", tags=["spreadsheets"])
async def get_spreadsheet_datasets(request: Request):
    """Get all spreadsheet datasets"""
    return await proxy_request(SPREADSHEET_SERVICE_URL, "/api/v1/datasets", request)

@router.post("/spreadsheets/datasets", tags=["spreadsheets"])
async def create_spreadsheet_dataset(request: Request):
    """Create a new spreadsheet dataset"""
    return await proxy_request(SPREADSHEET_SERVICE_URL, "/api/v1/datasets", request)

@router.get("/spreadsheets/datasets/{dataset_id}", tags=["spreadsheets"])
async def get_spreadsheet_dataset(dataset_id: str, request: Request):
    """Get spreadsheet dataset by ID"""
    return await proxy_request(SPREADSHEET_SERVICE_URL, f"/api/v1/datasets/{dataset_id}", request)

@router.get("/spreadsheets/versions", tags=["spreadsheets"])
async def get_spreadsheet_versions(request: Request):
    """Get spreadsheet versions"""
    return await proxy_request(SPREADSHEET_SERVICE_URL, "/api/v1/versions", request)

@router.post("/spreadsheets/upload", tags=["spreadsheets"])
async def upload_spreadsheet(request: Request):
    """Upload a spreadsheet"""
    return await proxy_request(SPREADSHEET_SERVICE_URL, "/api/v1/upload", request) 