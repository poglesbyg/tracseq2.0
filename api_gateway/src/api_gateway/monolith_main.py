"""
TracSeq API Gateway - Monolith Router

Gradual migration router that initially routes all requests to the monolith,
with feature flags to gradually extract services.
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

from api_gateway.core.monolith_config import MonolithRouterConfig, get_monolith_config

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
    config = get_monolith_config()
    logger.info("🚀 Starting TracSeq API Gateway (Monolith Router)",
                version=config.version,
                environment=config.environment,
                monolith_url=config.monolith.base_url)

    try:
        # Initialize HTTP client for proxying
        app.state.http_client = httpx.AsyncClient(
            timeout=httpx.Timeout(config.request_timeout),
            limits=httpx.Limits(max_connections=config.max_concurrent_requests)
        )

        # Log current routing configuration
        service_status = config.get_service_status()
        logger.info("✅ Service routing configuration loaded",
                   monolith_active=service_status["monolith"]["active"],
                   microservices_enabled=sum(1 for s in service_status["microservices"].values() if s["enabled"]))

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
    config = get_monolith_config()

    app = FastAPI(
        title="TracSeq API Gateway (Monolith Router)",
        description="Gradual migration router between monolith and microservices",
        version=config.version,
        docs_url="/docs",
        redoc_url="/redoc",
        lifespan=lifespan,
    )

    # Add middleware
    if config.cors.enabled:
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
        service_status = config.get_service_status()
        return {
            "service": "TracSeq API Gateway (Monolith Router)",
            "version": config.version,
            "status": "operational",
            "environment": config.environment,
            "routing": {
                "monolith": service_status["monolith"],
                "microservices_enabled": [name for name, svc in service_status["microservices"].items() if svc["enabled"]],
                "microservices_disabled": [name for name, svc in service_status["microservices"].items() if not svc["enabled"]]
            },
            "docs": "/docs",
            "health": "/health",
            "routing_status": "/routing-status"
        }

    # Health check endpoint
    @app.get("/health")
    async def health_check():
        """Gateway health check."""
        return {
            "status": "healthy",
            "service": "TracSeq API Gateway (Monolith Router)",
            "version": config.version,
            "timestamp": time.time(),
            "monolith_url": config.monolith.base_url
        }

    # Routing status endpoint
    @app.get("/routing-status")
    async def routing_status():
        """Show current routing configuration."""
        return {
            "routing_configuration": config.get_service_status(),
            "feature_flags": {
                "auth": config.feature_flags.use_auth_service,
                "samples": config.feature_flags.use_sample_service,
                "templates": config.feature_flags.use_template_service,
                "storage": config.feature_flags.use_storage_service,
                "sequencing": config.feature_flags.use_sequencing_service,
                "notifications": config.feature_flags.use_notification_service,
                "rag": config.feature_flags.use_rag_service,
                "barcode": config.feature_flags.use_barcode_service,
                "qaqc": config.feature_flags.use_qaqc_service,
                "library": config.feature_flags.use_library_service,
                "event": config.feature_flags.use_event_service,
                "transaction": config.feature_flags.use_transaction_service,
                "spreadsheet": config.feature_flags.use_spreadsheet_service
            }
        }

    # Main proxy handler
    @app.api_route(
        "/{path:path}",
        methods=["GET", "POST", "PUT", "DELETE", "PATCH", "OPTIONS", "HEAD"]
    )
    async def proxy_request(request: Request, path: str):
        """Proxy requests to monolith or microservices based on feature flags."""
        config = get_monolith_config()
        full_path = f"/{path}"

        # Determine routing target
        service_type, base_url = config.route_request(full_path)

        # Build upstream URL
        if service_type == "monolith":
            # Route to monolith - keep full path
            upstream_url = f"{base_url}{full_path}"
            target_service = "monolith"
        else:
            # Route to microservice - strip /api prefix
            # Convert /api/templates -> /templates for microservice
            if full_path.startswith('/api/'):
                microservice_path = full_path[4:]  # Remove '/api' prefix
            else:
                microservice_path = full_path
            upstream_url = f"{base_url}{microservice_path}"
            target_service = "microservice"

        # Get request data
        body = await request.body()

        # Log request routing (for debugging)
        if config.monitoring.log_requests:
            logger.info("🔀 Routing request",
                       path=full_path,
                       method=request.method,
                       target=target_service,
                       upstream_url=upstream_url)

        try:
            # Proxy the request
            http_client: httpx.AsyncClient = request.app.state.http_client

            # Forward headers but filter out hop-by-hop headers
            headers = dict(request.headers)
            hop_by_hop_headers = {
                "connection", "keep-alive", "proxy-authenticate",
                "proxy-authorization", "te", "trailers", "transfer-encoding",
                "upgrade", "host"
            }
            filtered_headers = {k: v for k, v in headers.items() if k.lower() not in hop_by_hop_headers}

            response = await http_client.request(
                method=request.method,
                url=upstream_url,
                headers=filtered_headers,
                content=body,
                params=dict(request.query_params)
            )

            # Log response (for debugging)
            if config.monitoring.log_responses:
                logger.info("✅ Request completed",
                           path=full_path,
                           method=request.method,
                           status_code=response.status_code,
                           target=target_service)

            # Return response
            return Response(
                content=response.content,
                status_code=response.status_code,
                headers=dict(response.headers)
            )

        except httpx.TimeoutException:
            logger.error("⏱️ Request timeout",
                        path=full_path,
                        target=target_service,
                        upstream_url=upstream_url)
            raise HTTPException(status_code=504, detail="Gateway timeout")

        except httpx.ConnectError as e:
            logger.error("🔌 Connection error",
                        path=full_path,
                        target=target_service,
                        upstream_url=upstream_url,
                        error=str(e))
            raise HTTPException(status_code=503, detail=f"Service unavailable: {target_service}")

        except Exception as e:
            logger.error("❌ Proxy error",
                        path=full_path,
                        target=target_service,
                        error=str(e))
            raise HTTPException(status_code=502, detail="Bad gateway")

    return app


# Create the application instance
app = create_app()


if __name__ == "__main__":
    config = get_monolith_config()

    logger.info("🚀 Starting TracSeq API Gateway (Monolith Router)",
                host=config.host,
                port=config.port,
                environment=config.environment,
                monolith_url=config.monolith.base_url)

    uvicorn.run(
        "api_gateway.monolith_main:app",
        host=config.host,
        port=config.port,
        reload=config.is_development,
        log_config=None,
    )
