"""
Route organization for the TracSeq 2.0 API Gateway.

This module provides centralized route registration and organization
for all API endpoints including the comprehensive finder functionality.
"""

from typing import List
from fastapi import APIRouter, FastAPI

from .finder import router as finder_router
from .auth import router as auth_router
from .health import router as health_router


def register_routes(app: FastAPI) -> None:
    """
    Register all route modules with the FastAPI application.
    
    Args:
        app: FastAPI application instance
    """
    # API v1 router
    api_v1_router = APIRouter(prefix="/api")
    
    # Register individual route modules
    api_v1_router.include_router(finder_router, prefix="/finder", tags=["finder"])
    api_v1_router.include_router(auth_router, prefix="/auth", tags=["authentication"])
    api_v1_router.include_router(health_router, tags=["health"])
    
    # Register the main API router
    app.include_router(api_v1_router)


def get_all_routes() -> List[APIRouter]:
    """
    Get all route modules for external access.
    
    Returns:
        List of APIRouter instances
    """
    return [
        finder_router,
        auth_router,
        health_router
    ]


# Export commonly used functions and routers
__all__ = [
    "register_routes",
    "get_all_routes",
    "finder_router",
    "auth_router", 
    "health_router"
]