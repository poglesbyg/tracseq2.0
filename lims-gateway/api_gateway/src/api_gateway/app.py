#!/usr/bin/env python3
"""
Application Factory for TracSeq 2.0 API Gateway
Centralized application initialization with dependency injection
"""

import asyncio
from contextlib import asynccontextmanager
from typing import Dict, Any

from fastapi import FastAPI, Request, Response
from fastapi.middleware.trustedhost import TrustedHostMiddleware

from .core.config import get_config
from .core.logging import setup_logging, main_logger
from .core.database import init_database, close_database
from .core.exceptions import setup_exception_handlers
from .middleware import (
    setup_cors, LoggingMiddleware, RateLimitMiddleware, 
    RequestIDMiddleware, SecurityMiddleware
)
from .routes import setup_routes
from .services.proxy import service_proxy


@asynccontextmanager
async def lifespan(app: FastAPI):
    """Application lifespan manager"""
    
    # Startup
    main_logger.info("Starting TracSeq 2.0 API Gateway...")
    
    try:
        # Initialize database
        await init_database()
        main_logger.info("Database initialized successfully")
        
        # Initialize other services
        await _initialize_services()
        main_logger.info("Services initialized successfully")
        
        main_logger.info("TracSeq 2.0 API Gateway started successfully")
        
    except Exception as e:
        main_logger.error(f"Failed to start application: {e}", exc_info=True)
        raise
    
    yield
    
    # Shutdown
    main_logger.info("Shutting down TracSeq 2.0 API Gateway...")
    
    try:
        # Close database
        await close_database()
        main_logger.info("Database closed successfully")
        
        # Cleanup other services
        await _cleanup_services()
        main_logger.info("Services cleaned up successfully")
        
        main_logger.info("TracSeq 2.0 API Gateway shut down successfully")
        
    except Exception as e:
        main_logger.error(f"Error during shutdown: {e}", exc_info=True)


async def _initialize_services():
    """Initialize additional services"""
    # Add any additional service initialization here
    pass


async def _cleanup_services():
    """Cleanup services during shutdown"""
    # Add any additional service cleanup here
    pass


def create_app() -> FastAPI:
    """Create and configure the FastAPI application"""
    
    # Get configuration
    config = get_config()
    
    # Setup logging first
    setup_logging()
    
    # Create FastAPI app with lifespan
    app = FastAPI(
        title=config.gateway.title,
        description=config.gateway.description,
        version=config.gateway.version,
        debug=config.gateway.debug,
        lifespan=lifespan
    )
    
    # Setup exception handlers
    setup_exception_handlers(app)
    
    # Setup middleware (order matters!)
    _setup_middleware(app)
    
    # Setup routes
    setup_routes(app)
    
    # Add health check endpoints
    _add_health_endpoints(app)
    
    main_logger.info(
        f"FastAPI application created",
        extra={
            "title": config.gateway.title,
            "version": config.gateway.version,
            "debug": config.gateway.debug,
            "environment": "development" if config.is_development else "production"
        }
    )
    
    return app


def _setup_middleware(app: FastAPI):
    """Setup middleware stack"""
    
    config = get_config()
    
    # Security middleware (first)
    app.add_middleware(SecurityMiddleware)
    
    # Request ID middleware
    app.add_middleware(RequestIDMiddleware)
    
    # Rate limiting middleware
    if config.monitoring.enable_metrics:
        app.add_middleware(
            RateLimitMiddleware,
            requests_per_minute=config.security.rate_limit_requests
        )
    
    # Logging middleware
    if config.logging.enable_access_log:
        app.add_middleware(LoggingMiddleware)
    
    # CORS middleware
    setup_cors(app)
    
    # Trusted host middleware (for production)
    if config.is_production:
        app.add_middleware(
            TrustedHostMiddleware,
            allowed_hosts=["*"]  # Configure this properly in production
        )
    
    main_logger.info("Middleware stack configured")


def _add_health_endpoints(app: FastAPI):
    """Add health check endpoints"""
    
    @app.get("/health")
    async def health_check():
        """Basic health check"""
        return {
            "status": "healthy",
            "service": "api-gateway",
            "version": get_config().gateway.version
        }
    
    @app.get("/health/detailed")
    async def detailed_health_check():
        """Detailed health check with service status"""
        try:
            # Get service health
            service_health = await service_proxy.get_all_service_health()
            
            # Get circuit breaker status
            circuit_breaker_status = service_proxy.get_circuit_breaker_status()
            
            return {
                "status": "healthy",
                "service": "api-gateway",
                "version": get_config().gateway.version,
                "services": service_health,
                "circuit_breakers": circuit_breaker_status,
                "database": "connected"  # Add database health check
            }
        except Exception as e:
            main_logger.error(f"Health check failed: {e}")
            return {
                "status": "unhealthy",
                "service": "api-gateway",
                "error": str(e)
            }
    
    @app.get("/metrics")
    async def metrics():
        """Metrics endpoint for monitoring"""
        config = get_config()
        
        if not config.monitoring.enable_metrics:
            return {"error": "Metrics disabled"}
        
        try:
            # Get service health
            service_health = await service_proxy.get_all_service_health()
            
            # Get circuit breaker status
            circuit_breaker_status = service_proxy.get_circuit_breaker_status()
            
            return {
                "gateway": {
                    "version": config.gateway.version,
                    "uptime": "unknown",  # Add uptime tracking
                    "environment": "development" if config.is_development else "production"
                },
                "services": service_health,
                "circuit_breakers": circuit_breaker_status
            }
        except Exception as e:
            main_logger.error(f"Metrics collection failed: {e}")
            return {"error": str(e)}
    
    main_logger.info("Health check endpoints added")


def get_app() -> FastAPI:
    """Get the configured FastAPI application"""
    return create_app()


# Create the application instance
app = create_app()