"""
TracSeq API Gateway - Main Application

Intelligent routing and management for TracSeq microservices ecosystem.
"""

import asyncio
import time
from contextlib import asynccontextmanager
from typing import Any, Dict

import httpx
import structlog
import uvicorn
from fastapi import FastAPI, HTTPException, Request, Response
from fastapi.middleware.cors import CORSMiddleware
from fastapi.middleware.gzip import GZipMiddleware
from fastapi.responses import JSONResponse

from api_gateway.core.config import AppConfig, get_config

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
    config = get_config()
    logger.info("🚀 Starting TracSeq API Gateway",
                version=config.app_version,
                environment=config.environment)

    try:
        # Initialize HTTP client for proxying
        app.state.http_client = httpx.AsyncClient(
            timeout=httpx.Timeout(config.gateway.request_timeout),
            limits=httpx.Limits(max_connections=1000)
        )

        logger.info("✅ TracSeq API Gateway startup complete")
        yield

    except Exception as e:
        logger.error("❌ Failed to initialize gateway", error=str(e))
        raise

    finally:
        logger.info("🛑 Shutting down TracSeq API Gateway")
        if hasattr(app.state, "http_client"):
            await app.state.http_client.aclose()


def create_app() -> FastAPI:
    """Create and configure the FastAPI application."""
    config = get_config()

    app = FastAPI(
        title="TracSeq API Gateway",
        description="Intelligent routing and management for TracSeq microservices",
        version=config.app_version,
        docs_url="/docs",
        redoc_url="/redoc",
        lifespan=lifespan,
    )

    # Add middleware
    app.add_middleware(
        CORSMiddleware,
        allow_origins=config.cors.allow_origins,
        allow_credentials=config.cors.allow_credentials,
        allow_methods=config.cors.allow_methods,
        allow_headers=config.cors.allow_headers,
    )

    app.add_middleware(GZipMiddleware, minimum_size=1000)

    # Root endpoint
    @app.get("/")
    async def root():
        """Root endpoint with gateway information."""
        return {
            "service": "TracSeq API Gateway",
            "version": config.app_version,
            "status": "operational",
            "environment": config.environment,
            "docs": "/docs",
            "health": "/health"
        }

    # Health check endpoint
    @app.get("/health")
    async def health_check():
        """Gateway health check."""
        return {
            "status": "healthy",
            "service": "TracSeq API Gateway",
            "version": config.app_version,
            "timestamp": time.time()
        }

    # Service discovery endpoint
    @app.get("/services")
    async def list_services():
        """List all available services."""
        services = [
            {"name": "auth", "url": config.services.auth_service_url},
            {"name": "samples", "url": config.services.sample_service_url},
            {"name": "storage", "url": config.services.storage_service_url},
            {"name": "templates", "url": config.services.template_service_url},
            {"name": "sequencing", "url": config.services.sequencing_service_url},
            {"name": "rag", "url": config.services.rag_service_url},
            {"name": "notifications", "url": config.services.notification_service_url}
        ]

        return {
            "services": services,
            "total": len(services),
            "gateway_version": config.app_version
        }

    # Main proxy handler
    @app.api_route(
        "/{service_path:path}",
        methods=["GET", "POST", "PUT", "DELETE", "PATCH", "OPTIONS", "HEAD"]
    )
    async def proxy_request(request: Request, service_path: str):
        """Proxy requests to appropriate microservices."""
        config = get_config()
        full_path = f"/{service_path}"

        # Find target service
        service_endpoint = config.get_service_by_path(full_path)
        if not service_endpoint:
            raise HTTPException(
                status_code=404,
                detail=f"No service found for path: {full_path}"
            )

        # Build upstream URL
        upstream_path = full_path[len(service_endpoint.path_prefix):]
        # For health checks, use the configured health check path directly
        if upstream_path == "/health" or upstream_path.startswith("/health"):
            upstream_url = service_endpoint.health_url
        else:
            # Special handling for templates service which uses /templates/* directly
            if service_endpoint.name == "templates":
                upstream_url = f"{service_endpoint.base_url}/templates{upstream_path}"
            else:
                upstream_url = f"{service_endpoint.base_url}/api/v1{upstream_path}"

        # Get request data
        body = await request.body()

        try:
            # Proxy the request
            http_client: httpx.AsyncClient = request.app.state.http_client

            response = await http_client.request(
                method=request.method,
                url=upstream_url,
                headers=dict(request.headers),
                content=body,
                params=dict(request.query_params)
            )

            # Return response
            return Response(
                content=response.content,
                status_code=response.status_code,
                headers=dict(response.headers)
            )

        except httpx.TimeoutException:
            logger.error("Request timeout", service=service_endpoint.name, path=full_path)
            raise HTTPException(status_code=504, detail="Gateway timeout")

        except httpx.ConnectError:
            logger.error("Service unavailable", service=service_endpoint.name, path=full_path)
            raise HTTPException(status_code=503, detail="Service unavailable")

        except Exception as e:
            logger.error("Proxy error", service=service_endpoint.name, error=str(e))
            raise HTTPException(status_code=502, detail="Bad gateway")

    return app


# Create the application instance
app = create_app()


if __name__ == "__main__":
    config = get_config()

    logger.info("🚀 Starting TracSeq API Gateway",
                host=config.gateway.host,
                port=config.gateway.port,
                environment=config.environment)

    uvicorn.run(
        "api_gateway.main:app",
        host=config.gateway.host,
        port=config.gateway.port,
        reload=config.is_development(),
        log_config=None,
    )
