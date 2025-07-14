from fastapi import FastAPI, Request, Response, HTTPException
from fastapi.middleware.cors import CORSMiddleware
import httpx
import uvicorn
import os
import time
import sys
import logging
from typing import Dict, Any

from logging_config import (
    setup_logging, 
    RequestLoggingMiddleware, 
    log_performance, 
    log_business_event,
    log_health_check,
    get_logger
)

app = FastAPI(title="TracSeq API Gateway (Simple)", version="1.0.0")

# Setup structured logging
logger = setup_logging("api-gateway", os.getenv("LOG_LEVEL", "INFO"))

# Add request logging middleware
app.add_middleware(RequestLoggingMiddleware, service_name="api-gateway")

# Log service startup
logger.info("API Gateway starting up", extra={"version": "1.0.0"})

# Add CORS middleware
app.add_middleware(
    CORSMiddleware,
    allow_origins=["*"],  # In production, specify your frontend domain
    allow_credentials=True,
    allow_methods=["*"],
    allow_headers=["*"],
)

# Service URLs from environment variables
DASHBOARD_SERVICE_URL = os.getenv("DASHBOARD_SERVICE_URL", "http://dashboard-service:8080")
SAMPLES_SERVICE_URL = os.getenv("SAMPLES_SERVICE_URL", "http://samples-service:8080")
SEQUENCING_SERVICE_URL = os.getenv("SEQUENCING_SERVICE_URL", "http://sequencing-service:8080")
SPREADSHEET_SERVICE_URL = os.getenv("SPREADSHEET_SERVICE_URL", "http://spreadsheet-service:8080")
QAQC_SERVICE_URL = os.getenv("QAQC_SERVICE_URL", "http://qaqc-service:8103")

# Service routing configuration
SERVICE_ROUTES = {
    "/api/dashboard": DASHBOARD_SERVICE_URL,
    "/api/samples": "http://host.docker.internal:8104",  # Rust Sample Service
    "/api/sequencing": "http://host.docker.internal:8105",  # Rust Sequencing Service
    "/api/spreadsheet": SPREADSHEET_SERVICE_URL,
    "/api/templates": SPREADSHEET_SERVICE_URL,  # Templates handled by spreadsheet service
    "/api/auth": DASHBOARD_SERVICE_URL,  # Auth handled by dashboard service for now
    "/api/storage": DASHBOARD_SERVICE_URL,  # Storage handled by dashboard service for now
    "/api/qaqc": "http://host.docker.internal:8103",  # Rust QAQC Service
    "/api/qc": "http://host.docker.internal:8103",  # Alternative QC route (Rust QAQC Service)
}

@app.get("/")
async def root():
    """Root endpoint - API Gateway information"""
    return {
        "service": "TracSeq API Gateway (Simple)",
        "version": "1.0.0",
        "status": "operational",
        "available_routes": list(SERVICE_ROUTES.keys()),
        "docs": "/docs",
        "health": "/health"
    }

@app.get("/health")
async def health_check():
    """API Gateway health check"""
    return {
        "status": "healthy",
        "service": "TracSeq API Gateway (Simple)",
        "version": "1.0.0",
        "timestamp": time.time(),
        "services": list(SERVICE_ROUTES.keys())
    }

@app.get("/services")
async def list_services():
    """List all available services and their status"""
    services = {}
    
    async with httpx.AsyncClient(timeout=5.0) as client:
        for route, url in SERVICE_ROUTES.items():
            try:
                response = await client.get(f"{url}/health")
                services[route] = {
                    "url": url,
                    "status": "healthy" if response.status_code == 200 else "unhealthy",
                    "response_time": response.elapsed.total_seconds() if hasattr(response, 'elapsed') else 0
                }
            except Exception as e:
                services[route] = {
                    "url": url,
                    "status": "unavailable",
                    "error": str(e)
                }
    
    return {
        "gateway": "TracSeq API Gateway (Simple)",
        "services": services,
        "total_services": len(services),
        "healthy_services": len([s for s in services.values() if s["status"] == "healthy"])
    }

def get_service_url(path: str) -> str | None:
    """Get the service URL for a given path"""
    for route, url in SERVICE_ROUTES.items():
        if path.startswith(route):
            return url
    return None

@app.api_route(
    "/api/{path:path}",
    methods=["GET", "POST", "PUT", "DELETE", "PATCH", "OPTIONS", "HEAD"]
)
async def proxy_api_request(request: Request, path: str):
    """Proxy all /api/* requests to the appropriate service"""
    
    full_path = f"/api/{path}"
    service_url = get_service_url(full_path)
    
    if not service_url:
        raise HTTPException(
            status_code=404,
            detail=f"No service found for path: {full_path}"
        )
    
    # Build target URL - remove the /api prefix and route to service
    remaining_path = full_path
    for route in SERVICE_ROUTES.keys():
        if full_path.startswith(route):
            remaining_path = full_path[len(route):]
            break
    
    # For health checks, route directly to /health
    if remaining_path == "/health" or remaining_path.startswith("/health"):
        target_url = f"{service_url}/health"
    else:
        target_url = f"{service_url}/api{remaining_path}" if remaining_path else f"{service_url}/api"
    
    # Get request data
    body = await request.body()
    
    try:
        async with httpx.AsyncClient(timeout=30.0) as client:
            # Clean headers to avoid conflicts
            headers = dict(request.headers)
            headers.pop("host", None)
            headers.pop("content-length", None)
            
            response = await client.request(
                method=request.method,
                url=target_url,
                headers=headers,
                content=body,
                params=dict(request.query_params)
            )
            
            # Clean response headers to avoid conflicts
            response_headers = dict(response.headers)
            response_headers.pop("content-length", None)
            response_headers.pop("transfer-encoding", None)
            response_headers.pop("content-encoding", None)
            response_headers.pop("server", None)
            response_headers.pop("date", None)
            
            # Return response with same status code and cleaned headers
            return Response(
                content=response.content,
                status_code=response.status_code,
                headers=response_headers,
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
            detail=f"Gateway error: {str(e)}"
        )

if __name__ == "__main__":
    uvicorn.run(app, host="0.0.0.0", port=8000) 