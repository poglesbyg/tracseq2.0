"""
Enhanced RAG Service - Main Application

A comprehensive microservice for AI-powered laboratory document processing.
"""

import asyncio
import logging
from contextlib import asynccontextmanager
from typing import Any

import structlog
import uvicorn
from fastapi import FastAPI, Request
from fastapi.middleware.cors import CORSMiddleware
from fastapi.middleware.gzip import GZipMiddleware
from fastapi.responses import JSONResponse

# Configure structured logging
structlog.configure(
    processors=[
        structlog.stdlib.filter_by_level,
        structlog.stdlib.add_logger_name,
        structlog.stdlib.add_log_level,
        structlog.stdlib.PositionalArgumentsFormatter(),
        structlog.processors.TimeStamper(fmt="iso"),
        structlog.processors.StackInfoRenderer(),
        structlog.processors.format_exc_info,
        structlog.processors.UnicodeDecoder(),
        structlog.processors.JSONRenderer()
    ],
    context_class=dict,
    logger_factory=structlog.stdlib.LoggerFactory(),
    wrapper_class=structlog.stdlib.BoundLogger,
    cache_logger_on_first_use=True,
)

logger = structlog.get_logger(__name__)


@asynccontextmanager
async def lifespan(app: FastAPI):
    """Application lifespan manager."""
    logger.info("🚀 Starting Enhanced RAG Service")
    
    # Initialize services here
    try:
        logger.info("✅ Enhanced RAG Service startup complete")
        yield
    except Exception as e:
        logger.error("❌ Failed to initialize service", error=str(e))
        raise
    finally:
        logger.info("🛑 Shutting down Enhanced RAG Service")


def create_app() -> FastAPI:
    """Create and configure the FastAPI application."""
    
    app = FastAPI(
        title="Enhanced RAG Service",
        description="AI-Powered Laboratory Document Processing Microservice",
        version="0.1.0",
        docs_url="/docs",
        redoc_url="/redoc",
        lifespan=lifespan,
    )
    
    # Add middleware
    app.add_middleware(
        CORSMiddleware,
        allow_origins=["*"],  # Configure appropriately for production
        allow_credentials=True,
        allow_methods=["*"],
        allow_headers=["*"],
    )
    
    app.add_middleware(GZipMiddleware, minimum_size=1000)
    
    # Root endpoint
    @app.get("/")
    async def root():
        """Root endpoint with service information."""
        return {
            "service": "Enhanced RAG Service",
            "version": "0.1.0",
            "status": "operational",
            "docs": "/docs",
            "health": "/api/v1/health"
        }
    
    # Health check endpoint
    @app.get("/api/v1/health")
    async def health_check():
        """Basic health check endpoint."""
        return {
            "status": "healthy",
            "service": "Enhanced RAG Service",
            "version": "0.1.0"
        }
    
    return app


# Create the application instance
app = create_app()


if __name__ == "__main__":
    logger.info("🚀 Starting Enhanced RAG Service", host="0.0.0.0", port=8000)
    
    uvicorn.run(
        app,
        host="0.0.0.0",
        port=8000,
        log_config=None,
    )
