"""
Health check routes for the TracSeq 2.0 API Gateway.
"""

from fastapi import APIRouter

router = APIRouter()

@router.get("/health")
async def gateway_health():
    """Gateway health check."""
    return {"status": "healthy", "service": "api-gateway"} 