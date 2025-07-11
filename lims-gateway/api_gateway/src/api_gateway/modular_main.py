"""
TracSeq API Gateway - Modular Main Application

This is a modular version of the API Gateway that uses the organized route structure
while maintaining compatibility with the existing configuration system.
"""

import os
import time
from contextlib import asynccontextmanager
from typing import Any, Dict

import httpx
import uvicorn
from fastapi import FastAPI, HTTPException, Request, Response
from fastapi.middleware.cors import CORSMiddleware
from fastapi.middleware.gzip import GZipMiddleware
from fastapi.responses import JSONResponse

# Import the modular routes
from api_gateway.routes import register_routes


@asynccontextmanager
async def lifespan(app: FastAPI):
    """Application lifespan manager."""
    print("🚀 Starting TracSeq API Gateway (Modular)")
    
    try:
        # Initialize HTTP client for proxying
        app.state.http_client = httpx.AsyncClient(
            timeout=httpx.Timeout(30.0),
            limits=httpx.Limits(max_connections=1000)
        )

        print("✅ TracSeq API Gateway startup complete")
        yield

    except Exception as e:
        print(f"❌ Failed to initialize gateway: {e}")
        raise

    finally:
        print("🛑 Shutting down TracSeq API Gateway")
        if hasattr(app.state, "http_client"):
            await app.state.http_client.aclose()


def create_app() -> FastAPI:
    """Create and configure the FastAPI application."""
    
    app = FastAPI(
        title="TracSeq API Gateway (Modular)",
        description="Modular API Gateway for TracSeq microservices",
        version="2.0.0",
        docs_url="/docs",
        redoc_url="/redoc",
        lifespan=lifespan,
    )

    # Add CORS middleware
    app.add_middleware(
        CORSMiddleware,
        allow_origins=["*"],  # Configure as needed
        allow_credentials=True,
        allow_methods=["*"],
        allow_headers=["*"],
    )

    app.add_middleware(GZipMiddleware, minimum_size=1000)

    # Register modular routes
    register_routes(app)

    # Root endpoint
    @app.get("/")
    async def root():
        """Root endpoint with gateway information."""
        return {
            "service": "TracSeq API Gateway (Modular)",
            "version": "2.0.0",
            "status": "operational",
            "environment": os.getenv("ENVIRONMENT", "development"),
            "docs": "/docs",
            "health": "/health"
        }

    # Health check endpoint
    @app.get("/health")
    async def health_check():
        """Gateway health check."""
        return {
            "status": "healthy",
            "service": "TracSeq API Gateway (Modular)",
            "version": "2.0.0",
            "timestamp": time.time()
        }

    # Service discovery endpoint
    @app.get("/services")
    async def list_services():
        """List all available services."""
        services = [
            {
                "name": "auth",
                "display_name": "Authentication Service",
                "base_url": os.getenv("AUTH_SERVICE_URL", "http://lims-auth:8000"),
                "health_url": f"{os.getenv('AUTH_SERVICE_URL', 'http://lims-auth:8000')}/health"
            },
            {
                "name": "samples",
                "display_name": "Sample Service",
                "base_url": os.getenv("SAMPLE_SERVICE_URL", "http://lims-samples:8000"),
                "health_url": f"{os.getenv('SAMPLE_SERVICE_URL', 'http://lims-samples:8000')}/health"
            },
            {
                "name": "storage",
                "display_name": "Storage Service",
                "base_url": os.getenv("STORAGE_SERVICE_URL", "http://lims-storage:8082"),
                "health_url": f"{os.getenv('STORAGE_SERVICE_URL', 'http://lims-storage:8082')}/health"
            }
        ]

        return {
            "services": services,
            "total": len(services),
            "gateway_version": "2.0.0"
        }

    return app


# Create the application instance
app = create_app()


if __name__ == "__main__":
    host = os.getenv("GATEWAY_HOST", "0.0.0.0")
    port = int(os.getenv("GATEWAY_PORT", "8000"))
    environment = os.getenv("ENVIRONMENT", "development")

    print(f"🚀 Starting TracSeq API Gateway (Modular) on {host}:{port}")
    print(f"Environment: {environment}")

    uvicorn.run(
        "api_gateway.modular_main:app",
        host=host,
        port=port,
        reload=environment == "development",
        log_config=None,
    ) 