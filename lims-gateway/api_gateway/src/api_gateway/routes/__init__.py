"""
Route organization for the TracSeq 2.0 API Gateway.

This module provides centralized route registration and organization
for all API endpoints including the comprehensive finder functionality
and service proxy routes.
"""

from typing import List
from fastapi import APIRouter, FastAPI

from .finder import router as finder_router
from .auth import router as auth_router
from .health import router as health_router
from .services import router as services_router


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
    api_v1_router.include_router(auth_router, prefix="/auth", tags=["auth"])
    api_v1_router.include_router(health_router, prefix="/health", tags=["health"])
    
    # Register service proxy routes (no prefix since they handle their own /api paths)
    api_v1_router.include_router(services_router, tags=["services"])
    
    # Register the direct /users/me route for frontend compatibility
    from .auth import proxy_users_me
    api_v1_router.add_api_route("/users/me", proxy_users_me, methods=["GET"], tags=["users"])
    
    # Register the API router
    app.include_router(api_v1_router)
    
    # Register health check at root level
    app.include_router(health_router, tags=["health"])


def get_registered_routes() -> List[str]:
    """
    Get a list of all registered route paths.
    
    Returns:
        List of route paths
    """
    return [
        "/health",
        "/api/finder/*",
        "/api/auth/*",
        "/api/users/me",
        "/api/templates/*",
        "/api/samples/*",
        "/api/storage/*",
        "/api/reports/*",
        "/api/projects/*",
        "/api/rag/*",
        "/api/sequencing/*",
        "/api/notifications/*",
        "/api/events/*",
        "/api/transactions/*",
        "/api/qaqc/*",
        "/api/services",
    ]


# Export commonly used functions and routers
__all__ = [
    "register_routes",
    "get_registered_routes",
    "finder_router",
    "auth_router", 
    "health_router"
]