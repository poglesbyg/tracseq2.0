"""
Health check routes for the TracSeq 2.0 API Gateway.
"""

from fastapi import APIRouter

router = APIRouter()

@router.get("/health")
async def gateway_health():
    """Gateway health check."""
    return {"status": "healthy", "service": "api-gateway"}

@router.get("/api/health")
async def api_health():
    """API health check endpoint for frontend compatibility."""
    return {"status": "healthy", "service": "api-gateway", "version": "2.0.0"} 