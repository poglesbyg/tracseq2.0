"""
TracSeq API Gateway - Unified Main Application

Production-ready API Gateway that properly routes frontend requests to microservices
and ensures database connectivity across all services.
"""

import asyncio
import time
import os
import logging
from contextlib import asynccontextmanager
from typing import Any, Dict, Optional, List
from urllib.parse import urlparse

import httpx
import uvicorn
from fastapi import FastAPI, HTTPException, Request, Response, status, Depends
from fastapi.middleware.cors import CORSMiddleware
from fastapi.middleware.gzip import GZipMiddleware
from fastapi.responses import JSONResponse, StreamingResponse
from fastapi.websockets import WebSocket, WebSocketDisconnect
from fastapi.security import HTTPBearer, HTTPAuthorizationCredentials
from starlette.middleware.base import BaseHTTPMiddleware
from starlette.requests import Request as StarletteRequest
from starlette.responses import Response as StarletteResponse
import asyncpg
import json

# Try to import structlog, fallback to standard logging if not available
try:
    import structlog
    
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
    
except ImportError:
    # Fallback to standard logging if structlog is not available
    import logging
    logging.basicConfig(
        level=logging.INFO,
        format='%(asctime)s - %(name)s - %(levelname)s - %(message)s'
    )
    logger = logging.getLogger(__name__)
    logger.warning("structlog not available, using standard logging")

from api_gateway.core.config import TracSeqAPIGatewayConfig, get_config, ServiceEndpoint

# Try to import enhanced components, use fallbacks if not available
try:
    from api_gateway.core.monitoring import MonitoringManager
    from api_gateway.core.circuit_breaker import CircuitBreakerManager, CircuitBreakerConfig, CircuitOpenError
    from api_gateway.core.rate_limiter import RateLimitManager, RateLimitConfig, RateLimitAlgorithm
    from api_gateway.core.database import init_database, close_database, get_db_health_status, get_db_connection
    ENHANCED_FEATURES = True
    logger.info("Enhanced features loaded successfully")
except ImportError as e:
    logger.warning(f"Enhanced features not available: {e}, using basic implementations")
    ENHANCED_FEATURES = False
    
    # Fallback implementations
    class MonitoringManager:
        async def start(self): pass
        async def stop(self): pass
        async def get_health_status(self): return {"status": "healthy", "checks": {}}
        def register_health_check(self, *args, **kwargs): pass
    
    class CircuitBreakerManager:
        def get_breaker(self, *args, **kwargs): 
            return type('MockBreaker', (), {'get_status': lambda: 'closed'})()
        def get_all_status(self): return {}
    
    class RateLimitManager:
        def get_status(self): return {}
    
    class CircuitOpenError(Exception): pass
    
    # Fallback database functions
    async def init_database(): 
        await init_db()
        return None
    
    async def close_database(): 
        await close_db()
    
    def get_db_health_status(): 
        return {"healthy": db_pool is not None, "fallback": True}
    
    from contextlib import asynccontextmanager
    
    @asynccontextmanager
    async def get_db_connection():
        global db_pool
        if db_pool:
            async with db_pool.acquire() as conn:
                yield conn
        else:
            raise RuntimeError("Database pool not initialized")

# HTTP Bearer token security
security = HTTPBearer(auto_error=False)

# Database connection pool (fallback implementation)
DATABASE_URL = os.getenv("DATABASE_URL", "postgres://postgres:postgres@lims-postgres:5432/lims_db")
db_pool = None

async def init_db():
    """Initialize database connection pool (fallback)."""
    global db_pool
    try:
        db_pool = await asyncpg.create_pool(DATABASE_URL, min_size=2, max_size=10)
        logger.info("Database connection pool initialized successfully")
    except Exception as e:
        logger.error(f"Failed to initialize database pool: {e}")
        db_pool = None

async def close_db():
    """Close database connection pool (fallback)."""
    global db_pool
    if db_pool:
        await db_pool.close()
        logger.info("Database connection pool closed")


class TracSeqAPIGateway:
    """
    TracSeq API Gateway - Routes frontend requests to microservices with database connectivity.
    """
    
    def __init__(self, config: TracSeqAPIGatewayConfig):
        self.config = config
        self.app: Optional[FastAPI] = None
        self.http_client: Optional[httpx.AsyncClient] = None
        self.monitoring_manager = MonitoringManager()
        self.circuit_manager = CircuitBreakerManager()
        self.rate_limit_manager: Optional[RateLimitManager] = None
        self.start_time = time.time()
        
        # WebSocket connections for real-time features
        self.websocket_connections: Dict[str, List[WebSocket]] = {}
        
    async def _initialize_components(self):
        """Initialize all gateway components."""
        logger.info("Initializing TracSeq API Gateway components...")
        
        # Initialize database connection
        await init_database()
        
        # Initialize HTTP client with proper configuration
        self.http_client = httpx.AsyncClient(
            timeout=httpx.Timeout(
                connect=10.0,
                read=self.config.request_timeout,
                write=10.0,
                pool=5.0
            ),
            limits=httpx.Limits(
                max_connections=self.config.max_concurrent_requests,
                max_keepalive_connections=200,
                keepalive_expiry=30
            ),
            follow_redirects=True,
            verify=False  # For development - enable in production
        )
        
        # Initialize rate limiting
        if ENHANCED_FEATURES:
            self.rate_limit_manager = RateLimitManager()
        
        # Register health checks for all services
        await self._register_health_checks()
        
        # Start monitoring
        await self.monitoring_manager.start()
        
        logger.info("TracSeq API Gateway components initialized successfully")
        
    async def _register_health_checks(self):
        """Register health checks for all microservices."""
        for service_name, endpoint in self.config.services.items():
            if service_name != "gateway":  # Skip self-reference
                check_func = self._create_health_check(service_name, endpoint)
                self.monitoring_manager.register_health_check(
                    service_name,
                    check_func,
                    interval=30,
                    critical=True
                )
    
    def _create_health_check(self, service_name: str, endpoint: ServiceEndpoint):
        """Create health check function for a service."""
        async def check_health():
            try:
                if self.http_client is None:
                    return {"healthy": False, "details": {"error": "HTTP client not initialized"}}
                    
                response = await self.http_client.get(
                    f"{endpoint.base_url}{endpoint.health_check_path}",
                    timeout=5.0
                )
                
                return {
                    "healthy": response.status_code == 200,
                    "details": {
                        "status_code": response.status_code,
                        "response_time": response.elapsed.total_seconds() if response.elapsed else 0,
                        "service_url": endpoint.base_url
                    }
                }
            except Exception as e:
                logger.warning(f"Health check failed for {service_name}: {str(e)}")
                return {
                    "healthy": False,
                    "details": {
                        "error": str(e),
                        "service_url": endpoint.base_url
                    }
                }
        
        return check_health
    
    async def _cleanup_components(self):
        """Cleanup gateway components."""
        logger.info("Cleaning up TracSeq API Gateway components...")
        
        if self.http_client:
            await self.http_client.aclose()
        
        if self.monitoring_manager:
            await self.monitoring_manager.stop()
        
        # Close database connection
        await close_database()
        
        logger.info("TracSeq API Gateway components cleaned up")
    
    async def _validate_token(self, token: str) -> Optional[Dict[str, Any]]:
        """Validate JWT token with auth service."""
        try:
            auth_service = self.config.services.get("auth")
            if not auth_service or self.http_client is None:
                return None
            
            response = await self.http_client.post(
                f"{auth_service.base_url}/validate/token",
                json={"token": token},
                timeout=5.0
            )
            
            if response.status_code == 200:
                return response.json()
            
            return None
            
        except Exception as e:
            logger.error(f"Token validation failed: {str(e)}")
            return None
    
    def _get_service_for_path(self, path: str) -> Optional[ServiceEndpoint]:
        """Get the appropriate service for a given path."""
        # Try exact path matching first
        for service_name, endpoint in self.config.services.items():
            if path.startswith(endpoint.path_prefix):
                return endpoint
        
        # Fallback patterns for common endpoints
        fallback_patterns = {
            "/api/users": "auth",
            "/api/auth": "auth",
            "/api/samples": "samples",
            "/api/storage": "storage",
            "/api/templates": "templates",
            "/api/sequencing": "sequencing",
            "/api/qc": "qc",
            "/api/qaqc": "qaqc",
            "/api/library-prep": "library-prep",
            "/api/flow-cells": "flow-cells",
            "/api/projects": "projects",
            "/api/notifications": "notifications",
            "/api/events": "events",
            "/api/spreadsheets": "spreadsheets",
            "/api/reports": "reports",
            "/api/dashboard": "dashboard",
            "/api/rag": "rag",
            "/api/chat": "chat",
        }
        
        for pattern, service_name in fallback_patterns.items():
            if path.startswith(pattern):
                service = self.config.services.get(service_name)
                if service:
                    return service
        
        # Ultimate fallback to lab manager
        return self.config.services.get("lab-manager")
    
    def _build_upstream_url(self, service: ServiceEndpoint, path: str, request: Request) -> str:
        """Build the upstream URL for the service."""
        if service.strip_path_prefix and path.startswith(service.path_prefix):
            # Remove the path prefix
            upstream_path = path[len(service.path_prefix):]
            if not upstream_path.startswith("/"):
                upstream_path = "/" + upstream_path
        else:
            upstream_path = path
        
        # Add version prefix if configured
        if service.add_version_prefix and not upstream_path.startswith("/api/v1"):
            if upstream_path.startswith("/api/"):
                upstream_path = upstream_path.replace("/api/", "/api/v1/", 1)
            else:
                upstream_path = f"/api/v1{upstream_path}"
        
        # Handle health checks specially
        if upstream_path.endswith("/health") or upstream_path == "/health":
            upstream_url = f"{service.base_url}{service.health_check_path}"
        else:
            # Special handling for auth service - transform /api/auth/* to /auth/*
            if service.name == "Auth Service" and upstream_path.startswith("/api/auth"):
                upstream_path = upstream_path.replace("/api/auth", "/auth", 1)
            upstream_url = f"{service.base_url}{upstream_path}"
        
        return upstream_url
    
    async def _proxy_request(self, request: Request, service: ServiceEndpoint, upstream_url: str) -> Response:
        """Proxy request to upstream service."""
        start_time = time.time()
        
        if self.http_client is None:
            raise HTTPException(status_code=503, detail="Gateway not properly initialized")
        
        try:
            # Get request body
            body = await request.body()
            
            # Prepare headers
            headers = dict(request.headers)
            
            # Add custom headers for service
            headers.update(service.custom_headers)
            
            # Remove problematic headers
            headers.pop("host", None)
            headers.pop("content-length", None)
            
            # Add tracing headers
            headers["X-Request-ID"] = str(time.time())
            headers["X-Forwarded-For"] = request.client.host if request.client else "unknown"
            
            logger.info(
                f"Proxying request: {request.method} {request.url.path} -> {service.name} ({upstream_url})"
            )
            
            # Make the request
            response = await self.http_client.request(
                method=request.method,
                url=upstream_url,
                headers=headers,
                content=body,
                params=dict(request.query_params),
                timeout=service.timeout
            )
            
            # Process response
            duration = time.time() - start_time
            
            # Log response
            logger.info(
                "Request completed",
                method=request.method,
                path=request.url.path,
                service=service.name,
                status_code=response.status_code,
                duration=f"{duration:.3f}s"
            )
            
            # Prepare response headers
            response_headers = dict(response.headers)
            response_headers["X-Gateway-Service"] = service.name
            response_headers["X-Gateway-Duration"] = f"{duration:.3f}s"
            
            # Handle streaming responses
            if response.headers.get("content-type", "").startswith("text/event-stream"):
                async def stream_response():
                    async for chunk in response.aiter_bytes():
                        yield chunk
                
                return StreamingResponse(
                    stream_response(),
                    status_code=response.status_code,
                    headers=response_headers,
                    media_type="text/event-stream"
                )
            
            return Response(
                content=response.content,
                status_code=response.status_code,
                headers=response_headers
            )
            
        except httpx.TimeoutException:
            duration = time.time() - start_time
            logger.error(
                "Request timeout",
                service=service.name,
                upstream_url=upstream_url,
                duration=f"{duration:.3f}s"
            )
            raise HTTPException(
                status_code=504,
                detail=f"Gateway timeout after {duration:.1f}s"
            )
            
        except httpx.ConnectError as e:
            duration = time.time() - start_time
            logger.error(
                "Service connection error",
                service=service.name,
                upstream_url=upstream_url,
                error=str(e),
                duration=f"{duration:.3f}s"
            )
            raise HTTPException(
                status_code=503,
                detail=f"Service {service.name} unavailable"
            )
            
        except Exception as e:
            duration = time.time() - start_time
            logger.error(
                "Proxy error",
                service=service.name,
                upstream_url=upstream_url,
                error=str(e),
                duration=f"{duration:.3f}s"
            )
            raise HTTPException(
                status_code=502,
                detail=f"Gateway error: {str(e)}"
            )
    
    def create_app(self) -> FastAPI:
        """Create and configure the FastAPI application."""
        
        @asynccontextmanager
        async def lifespan(app: FastAPI):
            """Application lifespan manager."""
            logger.info("🚀 Starting TracSeq API Gateway", 
                        version=self.config.version,
                        environment=self.config.environment)
            
            try:
                await self._initialize_components()
                app.state.gateway = self
                app.state.start_time = self.start_time
                yield
            except Exception as e:
                logger.error("❌ Failed to initialize gateway", error=str(e))
                raise
            finally:
                logger.info("🛑 Shutting down TracSeq API Gateway")
                await self._cleanup_components()
        
        self.app = FastAPI(
            title="TracSeq API Gateway",
            description="Production-ready API Gateway for TracSeq LIMS",
            version=self.config.version,
            docs_url="/docs",
            redoc_url="/redoc",
            lifespan=lifespan,
        )
        
        # Add middleware
        self._add_middleware()
        
        # Add routes
        self._add_routes()
        
        return self.app
    
    def _add_middleware(self):
        """Add middleware to the application."""
        if self.app is None:
            raise RuntimeError("FastAPI app not initialized")
            
        # CORS middleware
        if self.config.cors.enabled:
            self.app.add_middleware(
                CORSMiddleware,
                allow_origins=self.config.cors.allow_origins,
                allow_credentials=self.config.cors.allow_credentials,
                allow_methods=self.config.cors.allow_methods,
                allow_headers=self.config.cors.allow_headers,
            )
        
        # Compression middleware
        self.app.add_middleware(GZipMiddleware, minimum_size=1000)
        
        # Custom logging middleware
        @self.app.middleware("http")
        async def logging_middleware(request: Request, call_next):
            start_time = time.time()
            
            response = await call_next(request)
            
            duration = time.time() - start_time
            
            logger.info(
                "HTTP Request",
                method=request.method,
                path=request.url.path,
                status_code=response.status_code,
                duration=f"{duration:.3f}s",
                user_agent=request.headers.get("user-agent", "unknown")
            )
            
            return response
    
    def _add_routes(self):
        """Add API routes."""
        if self.app is None:
            raise RuntimeError("FastAPI app not initialized")
        
        @self.app.get("/")
        async def root():
            """Root endpoint with gateway information."""
            return {
                "service": "TracSeq API Gateway",
                "version": self.config.version,
                "status": "operational",
                "environment": self.config.environment,
                "features": {
                    "microservices": True,
                    "database_connectivity": db_pool is not None,
                    "authentication": True,
                    "rate_limiting": ENHANCED_FEATURES,
                    "circuit_breaker": ENHANCED_FEATURES,
                    "monitoring": ENHANCED_FEATURES
                },
                "services": {
                    name: {
                        "name": service.name,
                        "path_prefix": service.path_prefix,
                        "health_url": service.health_url
                    }
                    for name, service in self.config.services.items()
                },
                "docs": "/docs",
                "health": "/health"
            }
        
        @self.app.get("/health")
        async def health_check():
            """Gateway health check with service status."""
            health_status = await self.monitoring_manager.get_health_status()
            
            # Check database connectivity
            db_healthy = False
            if db_pool:
                try:
                    async with db_pool.acquire() as conn:
                        await conn.execute("SELECT 1")
                    db_healthy = True
                except Exception as e:
                    logger.error(f"Database health check failed: {e}")
            
            gateway_health = {
                "status": "healthy" if db_healthy else "degraded",
                "service": "TracSeq API Gateway",
                "version": self.config.version,
                "uptime": time.time() - self.start_time,
                "environment": self.config.environment,
                "components": {
                    "http_client": self.http_client is not None,
                    "database": db_healthy,
                    "monitoring": True,
                    "circuit_breaker": ENHANCED_FEATURES,
                    "rate_limiter": self.rate_limit_manager is not None
                }
            }
            
            # Combine with service health status
            if health_status:
                gateway_health.update(health_status)
            
            overall_status = "healthy" if gateway_health.get("status") == "healthy" and db_healthy else "unhealthy"
            status_code = 200 if overall_status == "healthy" else 503
            
            return JSONResponse(content=gateway_health, status_code=status_code)
        
        @self.app.get("/services")
        async def list_services():
            """List all available services with their status."""
            services = []
            health_status = await self.monitoring_manager.get_health_status()
            
            for name, endpoint in self.config.services.items():
                if name != "gateway":
                    service_health = health_status.get("checks", {}).get(name, {}) if health_status else {}
                    
                    services.append({
                        "name": name,
                        "display_name": endpoint.name,
                        "path_prefix": endpoint.path_prefix,
                        "base_url": endpoint.base_url,
                        "health_url": endpoint.health_url,
                        "status": service_health.get("status", "unknown"),
                        "last_check": service_health.get("last_check"),
                        "require_auth": endpoint.require_auth,
                        "rate_limit": endpoint.rate_limit
                    })
            
            return {
                "services": services,
                "total": len(services),
                "healthy": sum(1 for s in services if s["status"] == "healthy"),
                "gateway_version": self.config.version
            }
        
        @self.app.get("/gateway/stats")
        async def gateway_stats():
            """Get comprehensive gateway statistics."""
            return {
                "uptime": time.time() - self.start_time,
                "version": self.config.version,
                "environment": self.config.environment,
                "services_count": len(self.config.services),
                "health_checks": await self.monitoring_manager.get_health_status(),
                "circuit_breakers": self.circuit_manager.get_all_status() if self.circuit_manager else {},
                "rate_limits": self.rate_limit_manager.get_status() if self.rate_limit_manager else {},
                "database_connected": db_pool is not None
            }
        
        # WebSocket endpoint for real-time chat
        @self.app.websocket("/ws/chat/{conversation_id}")
        async def websocket_chat(websocket: WebSocket, conversation_id: str):
            """WebSocket endpoint for real-time chat."""
            await websocket.accept()
            
            # Store connection
            if conversation_id not in self.websocket_connections:
                self.websocket_connections[conversation_id] = []
            self.websocket_connections[conversation_id].append(websocket)
            
            try:
                while True:
                    data = await websocket.receive_text()
                    
                    # Process message and forward to chat service
                    chat_service = self.config.services.get("chat")
                    if chat_service:
                        # Forward to chat service (implementation depends on chat service API)
                        pass
                    
                    # Echo for now
                    await websocket.send_text(f"Echo: {data}")
                    
            except WebSocketDisconnect:
                # Remove connection
                if conversation_id in self.websocket_connections:
                    self.websocket_connections[conversation_id].remove(websocket)
                    if not self.websocket_connections[conversation_id]:
                        del self.websocket_connections[conversation_id]
        
        # Main proxy handler
        @self.app.api_route(
            "/api/{service_path:path}",
            methods=["GET", "POST", "PUT", "DELETE", "PATCH", "OPTIONS", "HEAD"]
        )
        async def proxy_api_request(request: Request, service_path: str):
            """Proxy API requests to appropriate microservices."""
            full_path = f"/api/{service_path}"
            
            logger.info(f"DEBUG: Received request for path: {full_path}")
            
            # Handle special case for /api/health - redirect to gateway health
            if full_path == "/api/health":
                health_status = await self.monitoring_manager.get_health_status()
                
                # Check database connectivity
                db_healthy = False
                if db_pool:
                    try:
                        async with db_pool.acquire() as conn:
                            await conn.execute("SELECT 1")
                        db_healthy = True
                    except Exception as e:
                        logger.error(f"Database health check failed: {e}")
                
                gateway_health = {
                    "status": "healthy" if db_healthy else "degraded",
                    "service": "TracSeq API Gateway",
                    "version": self.config.version,
                    "uptime": time.time() - self.start_time,
                    "environment": self.config.environment,
                    "components": {
                        "http_client": self.http_client is not None,
                        "database": db_healthy,
                        "monitoring": True,
                        "circuit_breaker": ENHANCED_FEATURES,
                        "rate_limiter": self.rate_limit_manager is not None
                    }
                }
                
                # Combine with service health status
                if health_status:
                    gateway_health.update(health_status)
                
                overall_status = "healthy" if gateway_health.get("status") == "healthy" and db_healthy else "unhealthy"
                status_code = 200 if overall_status == "healthy" else 503
                
                return JSONResponse(content=gateway_health, status_code=status_code)
            
            # Find target service
            service = self._get_service_for_path(full_path)
            if not service:
                logger.warning("No service found for path", path=full_path)
                raise HTTPException(
                    status_code=404,
                    detail=f"No service found for path: {full_path}"
                )
            
            # Build upstream URL
            upstream_url = self._build_upstream_url(service, full_path, request)
            
            # Check authentication if required
            auth_exempt_paths = ["/api/auth/login", "/api/auth/register", "/api/auth/forgot-password", "/api/auth/reset-password", "/api/auth/verify-email", "/api/health"]
            logger.info(f"DEBUG: Service {service.name} require_auth={service.require_auth}, path={full_path}")
            if service.require_auth and full_path not in auth_exempt_paths:
                auth_header = request.headers.get("authorization")
                logger.info(f"DEBUG: Auth header present: {auth_header is not None}")
                if not auth_header or not auth_header.startswith("Bearer "):
                    logger.warning(f"DEBUG: Authentication failed for path {full_path}")
                    raise HTTPException(
                        status_code=401,
                        detail="Authentication required"
                    )
            
            # Apply circuit breaker if enhanced features are available
            if ENHANCED_FEATURES:
                service_name = next(
                    name for name, ep in self.config.services.items() if ep == service
                )
                
                breaker = self.circuit_manager.get_breaker(
                    service_name,
                    CircuitBreakerConfig(
                        failure_threshold=5,
                        timeout=60,
                        window_size=100
                    )
                )
                
                try:
                    async with breaker:
                        return await self._proxy_request(request, service, upstream_url)
                        
                except CircuitOpenError:
                    logger.warning("Circuit breaker open", service=service_name)
                    raise HTTPException(
                        status_code=503,
                        detail=f"Service {service.name} temporarily unavailable"
                    )
            else:
                # Direct proxy without circuit breaker
                return await self._proxy_request(request, service, upstream_url)
        
        # Catch-all proxy for non-API routes
        @self.app.api_route(
            "/{path:path}",
            methods=["GET", "POST", "PUT", "DELETE", "PATCH", "OPTIONS", "HEAD"]
        )
        async def proxy_fallback_request(request: Request, path: str):
            """Fallback proxy for non-API routes."""
            full_path = f"/{path}"
            
            # Skip if it's a gateway route
            if full_path.startswith(("/health", "/docs", "/redoc", "/openapi.json", "/services", "/gateway")):
                raise HTTPException(status_code=404, detail="Not found")
            
            # Route to lab manager as fallback
            lab_manager = self.config.services.get("lab-manager")
            if lab_manager:
                upstream_url = f"{lab_manager.base_url}{full_path}"
                return await self._proxy_request(request, lab_manager, upstream_url)
            
            raise HTTPException(status_code=404, detail="Not found")


def create_app() -> FastAPI:
    """Create TracSeq API Gateway application."""
    config = get_config()
    gateway = TracSeqAPIGateway(config)
    return gateway.create_app()


# Create the application instance
app = create_app()


if __name__ == "__main__":
    config = get_config()
    
    logger.info("🚀 Starting TracSeq API Gateway",
                host=config.host,
                port=config.port,
                environment=config.environment)
    
    uvicorn.run(
        "api_gateway.unified_main:app",
        host=config.host,
        port=config.port,
        reload=config.is_development,
        log_config=None,
        workers=1,  # Keep at 1 for proper state management
        access_log=config.is_development
    )