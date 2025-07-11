#!/usr/bin/env python3
"""
Routes Package for TracSeq 2.0 API Gateway
Centralized route organization and registration
"""

from fastapi import FastAPI

from .auth import auth_router
from .samples import samples_router
from .storage import storage_router
from .rag import rag_router
from .reports import reports_router
from .proxy import proxy_router
from .debug import debug_router


def setup_routes(app: FastAPI) -> None:
    """Setup all route handlers for the application"""
    
    # Authentication routes
    app.include_router(auth_router, prefix="/api/auth", tags=["authentication"])
    
    # Sample management routes
    app.include_router(samples_router, prefix="/api/samples", tags=["samples"])
    
    # Storage management routes
    app.include_router(storage_router, prefix="/api/storage", tags=["storage"])
    
    # RAG and AI routes
    app.include_router(rag_router, prefix="/api/rag", tags=["rag"])
    
    # Reports routes
    app.include_router(reports_router, prefix="/api/reports", tags=["reports"])
    
    # Debug routes (development only)
    from ..core.config import get_config
    if get_config().is_development:
        app.include_router(debug_router, prefix="/api/debug", tags=["debug"])
    
    # Proxy routes (catch-all for microservices)
    app.include_router(proxy_router, tags=["proxy"])


__all__ = [
    "setup_routes",
    "auth_router",
    "samples_router", 
    "storage_router",
    "rag_router",
    "reports_router",
    "proxy_router",
    "debug_router"
]