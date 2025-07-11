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
    return await proxy_request(STORAGE_SERVICE_URL, "/api/storage", request)

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
    return await proxy_request(REPORTS_SERVICE_URL, "/reports", request)

@router.post("/reports", tags=["reports"])
async def create_report(request: Request):
    """Create a new report"""
    return await proxy_request(REPORTS_SERVICE_URL, "/reports", request)

@router.get("/reports/{report_id}", tags=["reports"])
async def get_report(request: Request, report_id: str):
    """Get a specific report"""
    return await proxy_request(REPORTS_SERVICE_URL, f"/reports/{report_id}", request)

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