"""
Enhanced TracSeq API Gateway with Production-Grade Features

Integrates all robustness improvements for a highly available, scalable gateway.
"""

import asyncio
import time
import os
from contextlib import asynccontextmanager
from typing import Any, Dict, Optional
import httpx
import structlog
import uvicorn
import redis.asyncio as redis
from fastapi import FastAPI, Request, Response, HTTPException
from fastapi.middleware.cors import CORSMiddleware
from fastapi.middleware.gzip import GZipMiddleware
from fastapi.responses import JSONResponse
from prometheus_client import make_asgi_app

# Import our enhanced components
from api_gateway.core.config import get_config, TracSeqAPIGatewayConfig
from api_gateway.core.circuit_breaker import CircuitBreakerManager, CircuitBreakerConfig, CircuitOpenError
from api_gateway.core.rate_limiter import RateLimitManager, RateLimitConfig, RateLimitAlgorithm
from api_gateway.core.monitoring import MonitoringManager
from api_gateway.middleware.auth_middleware import (
    AuthenticationMiddleware, PermissionMiddleware,
    JWTConfig, SecurityConfig
)

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


class EnhancedAPIGateway:
    """
    Enhanced API Gateway with production-grade features.
    """
    
    def __init__(self, config: TracSeqAPIGatewayConfig):
        self.config = config
        self.app = None
        self.http_client = None
        self.redis_client = None
        self.circuit_manager = CircuitBreakerManager()
        self.rate_limit_manager = None
        self.monitoring_manager = MonitoringManager()
        
    async def _initialize_components(self):
        """Initialize all gateway components."""
        # Initialize HTTP client with connection pooling
        self.http_client = httpx.AsyncClient(
            timeout=httpx.Timeout(self.config.request_timeout),
            limits=httpx.Limits(
                max_connections=self.config.max_concurrent_requests,
                max_keepalive_connections=100,
                keepalive_expiry=30
            ),
            follow_redirects=False
        )
        
        # Initialize Redis connection if configured
        if self.config.rate_limiting.redis_url:
            self.redis_client = await redis.from_url(
                self.config.rate_limiting.redis_url,
                encoding="utf-8",
                decode_responses=True
            )
            
        # Initialize rate limiting
        self.rate_limit_manager = RateLimitManager(
            redis_client=self.redis_client,
            system_monitor=self.monitoring_manager.system_monitor
        )
        
        # Configure rate limiters for each service
        for service_name, endpoint in self.config.services.items():
            # Service-level rate limiting
            self.rate_limit_manager.configure_limiter(
                service_name,
                RateLimitConfig(
                    requests_per_minute=endpoint.rate_limit,
                    burst_size=endpoint.rate_limit // 6,  # 10 second burst
                    algorithm=RateLimitAlgorithm.TOKEN_BUCKET,
                    per_user=True,
                    per_endpoint=False
                ),
                distributed=bool(self.redis_client)
            )
            
        # Global rate limiting
        self.rate_limit_manager.configure_limiter(
            "global",
            RateLimitConfig(
                requests_per_minute=10000,  # 10k requests per minute globally
                burst_size=1000,
                algorithm=RateLimitAlgorithm.ADAPTIVE,
                per_user=False,
                adaptive_threshold=0.8
            ),
            distributed=bool(self.redis_client)
        )
        
        # Register health checks
        for service_name, endpoint in self.config.services.items():
            check_func = self._create_health_check(service_name, endpoint)
            self.monitoring_manager.register_health_check(
                service_name,
                check_func,
                interval=30,
                critical=True
            )
            
        # Start monitoring
        await self.monitoring_manager.start()
        
        logger.info("Gateway components initialized successfully")
        
    async def _cleanup_components(self):
        """Cleanup gateway components."""
        if self.http_client:
            await self.http_client.aclose()
        
        if self.redis_client:
            await self.redis_client.close()
            
        await self.monitoring_manager.stop()
        
        logger.info("Gateway components cleaned up")
        
    def _create_health_check(self, service_name: str, endpoint):
        """Create health check function for a service."""
        async def check_health():
            try:
                response = await self.http_client.get(
                    endpoint.health_url,
                    timeout=5.0
                )
                
                return {
                    "healthy": response.status_code == 200,
                    "details": {
                        "status_code": response.status_code,
                        "response_time": response.elapsed.total_seconds()
                    }
                }
            except Exception as e:
                return {
                    "healthy": False,
                    "details": {
                        "error": str(e)
                    }
                }
                
        return check_health
        
    def create_app(self) -> FastAPI:
        """Create and configure the FastAPI application."""
        
        @asynccontextmanager
        async def lifespan(app: FastAPI):
            """Application lifespan manager."""
            logger.info("🚀 Starting Enhanced TracSeq API Gateway", 
                        version=self.config.version,
                        environment=self.config.environment)
            
            try:
                await self._initialize_components()
                yield
            except Exception as e:
                logger.error("❌ Failed to initialize gateway", error=str(e))
                raise
            finally:
                logger.info("🛑 Shutting down Enhanced TracSeq API Gateway")
                await self._cleanup_components()
                
        self.app = FastAPI(
            title="TracSeq API Gateway (Enhanced)",
            description="Production-grade API Gateway with advanced features",
            version=self.config.version,
            docs_url="/docs",
            redoc_url="/redoc",
            lifespan=lifespan,
        )
        
        # Add middleware
        self._add_middleware()
        
        # Add routes
        self._add_routes()
        
        # Mount Prometheus metrics endpoint
        metrics_app = make_asgi_app()
        self.app.mount("/metrics", metrics_app)
        
        return self.app
        
    def _add_middleware(self):
        """Add middleware to the application."""
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
        
        # Authentication middleware
        if self.config.authentication.enabled:
            jwt_config = JWTConfig(
                secret_key=self.config.authentication.jwt_secret_key,
                algorithm=self.config.authentication.jwt_algorithm,
                token_expiry=self.config.authentication.token_expiry
            )
            
            security_config = SecurityConfig(
                enable_cors=self.config.cors.enabled,
                cors_origins=self.config.cors.allow_origins,
                security_headers=self.config.authentication.security_headers,
                require_https=self.config.is_production
            )
            
            self.app.add_middleware(
                AuthenticationMiddleware,
                jwt_config=jwt_config,
                security_config=security_config,
                auth_service_url=self.config.services["auth"].base_url,
                redis_client=self.redis_client,
                excluded_paths=["/health", "/docs", "/redoc", "/openapi.json", "/metrics"]
            )
            
        # Permission middleware
        if self.config.authentication.enable_permissions:
            self.app.add_middleware(
                PermissionMiddleware,
                permission_config=self.config.authentication.permission_config
            )
            
    def _add_routes(self):
        """Add API routes."""
        
        @self.app.get("/")
        async def root():
            """Root endpoint with gateway information."""
            return {
                "service": "TracSeq API Gateway (Enhanced)",
                "version": self.config.version,
                "status": "operational",
                "environment": self.config.environment,
                "features": {
                    "circuit_breaker": True,
                    "rate_limiting": True,
                    "authentication": self.config.authentication.enabled,
                    "monitoring": True,
                    "distributed_tracing": True
                },
                "services": list(self.config.services.keys()),
                "docs": "/docs",
                "health": "/health",
                "metrics": "/metrics"
            }
            
        @self.app.get("/health")
        async def health_check():
            """Gateway health check with dependencies."""
            health_status = await self.monitoring_manager.get_health_status()
            
            # Add gateway-specific health info
            health_status["gateway"] = {
                "status": "healthy",
                "version": self.config.version,
                "uptime": time.time() - self.app.state.start_time if hasattr(self.app.state, 'start_time') else 0,
                "redis_connected": self.redis_client is not None and await self._check_redis()
            }
            
            status_code = 200 if health_status["status"] == "healthy" else 503
            return JSONResponse(content=health_status, status_code=status_code)
            
        @self.app.get("/health/services")
        async def service_health():
            """Detailed health status of all backend services."""
            return await self.monitoring_manager.get_health_status()
            
        @self.app.get("/services")
        async def list_services():
            """List all available services with status."""
            services = []
            health_status = await self.monitoring_manager.get_health_status()
            
            for name, endpoint in self.config.services.items():
                service_health = health_status["checks"].get(name, {})
                
                services.append({
                    "name": name,
                    "display_name": endpoint.name,
                    "path_prefix": endpoint.path_prefix,
                    "base_url": endpoint.base_url,
                    "health_url": endpoint.health_url,
                    "status": service_health.get("status", "unknown"),
                    "last_check": service_health.get("last_check"),
                    "circuit_breaker": self.circuit_manager.get_breaker(name).get_status()
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
            return await self.monitoring_manager.get_metrics_summary()
            
        @self.app.get("/gateway/circuit-breakers")
        async def circuit_breaker_status():
            """Get circuit breaker status for all services."""
            return self.circuit_manager.get_all_status()
            
        @self.app.get("/gateway/rate-limits")
        async def rate_limit_status():
            """Get rate limiting configuration and status."""
            return self.rate_limit_manager.get_status()
            
        # Main proxy handler
        @self.app.api_route(
            "/{service_path:path}",
            methods=["GET", "POST", "PUT", "DELETE", "PATCH", "OPTIONS", "HEAD"]
        )
        async def proxy_request(request: Request, service_path: str):
            """Enhanced proxy with circuit breaker, rate limiting, and monitoring."""
            start_time = time.time()
            full_path = f"/{service_path}"
            
            # Extract user context
            user_id = getattr(request.state, "user", {}).get("user_id")
            
            # Find target service
            service_endpoint = self.config.get_service_by_path(full_path)
            if not service_endpoint:
                raise HTTPException(
                    status_code=404,
                    detail=f"No service found for path: {full_path}"
                )
                
            service_name = next(
                name for name, ep in self.config.services.items() 
                if ep == service_endpoint
            )
            
            # Start distributed tracing span
            span = self.monitoring_manager.start_request_span(
                f"gateway.proxy.{service_name}",
                dict(request.headers),
                {
                    "http.method": request.method,
                    "http.path": full_path,
                    "service.name": service_name,
                    "user.id": user_id or "anonymous"
                }
            )
            
            try:
                # Check rate limits
                rate_limit_result = await self.rate_limit_manager.check_rate_limit(
                    service=service_name,
                    endpoint=full_path,
                    user_id=user_id,
                    ip_address=request.client.host
                )
                
                if not rate_limit_result.allowed:
                    self.monitoring_manager.record_rate_limit_exceeded(
                        service_name,
                        "authenticated" if user_id else "anonymous"
                    )
                    
                    return JSONResponse(
                        status_code=429,
                        content={
                            "error": "Rate limit exceeded",
                            "reason": rate_limit_result.reason,
                            "retry_after": rate_limit_result.retry_after
                        },
                        headers={
                            "X-RateLimit-Remaining": str(rate_limit_result.remaining),
                            "X-RateLimit-Reset": rate_limit_result.reset_at.isoformat(),
                            "Retry-After": str(rate_limit_result.retry_after)
                        }
                    )
                    
                # Track active request
                self.monitoring_manager.track_active_request(service_name, 1)
                
                # Get circuit breaker
                breaker = self.circuit_manager.get_breaker(
                    service_name,
                    CircuitBreakerConfig(
                        failure_threshold=5,
                        timeout=60,
                        window_size=100,
                        failure_rate_threshold=0.5
                    )
                )
                
                # Update monitoring with circuit breaker state
                self.monitoring_manager.update_circuit_breaker_state(
                    service_name,
                    {"closed": 0, "open": 1, "half_open": 2}[breaker.state.value]
                )
                
                # Execute request with circuit breaker
                async with breaker:
                    # Build upstream URL
                    upstream_path = full_path[len(service_endpoint.path_prefix):]
                    upstream_url = f"{service_endpoint.base_url}/api/v1{upstream_path}"
                    
                    # Prepare headers with trace context
                    headers = dict(request.headers)
                    self.monitoring_manager.inject_trace_context(headers, span)
                    
                    # Add user context headers
                    if hasattr(request.state, "user"):
                        headers["X-User-Id"] = request.state.user.get("user_id", "")
                        headers["X-User-Roles"] = ",".join(request.state.user.get("roles", []))
                        
                    # Get request body
                    body = await request.body()
                    
                    # Make the request
                    response = await self.http_client.request(
                        method=request.method,
                        url=upstream_url,
                        headers=headers,
                        content=body,
                        params=dict(request.query_params)
                    )
                    
                    # Process response
                    duration = time.time() - start_time
                    
                    # Record metrics
                    await self.monitoring_manager.record_request(
                        service=service_name,
                        endpoint=full_path,
                        method=request.method,
                        duration=duration,
                        status_code=response.status_code,
                        user_id=user_id,
                        span=span
                    )
                    
                    # Add response headers
                    response_headers = dict(response.headers)
                    response_headers["X-Gateway-Response-Time"] = f"{duration:.3f}s"
                    response_headers["X-Gateway-Service"] = service_name
                    
                    if rate_limit_result:
                        response_headers["X-RateLimit-Remaining"] = str(rate_limit_result.remaining)
                        response_headers["X-RateLimit-Reset"] = rate_limit_result.reset_at.isoformat()
                        
                    return Response(
                        content=response.content,
                        status_code=response.status_code,
                        headers=response_headers
                    )
                    
            except CircuitOpenError as e:
                # Circuit is open
                duration = time.time() - start_time
                await self.monitoring_manager.record_request(
                    service=service_name,
                    endpoint=full_path,
                    method=request.method,
                    duration=duration,
                    status_code=503,
                    user_id=user_id,
                    error="Circuit breaker open",
                    span=span
                )
                
                return JSONResponse(
                    status_code=503,
                    content={
                        "error": "Service temporarily unavailable",
                        "reason": "Circuit breaker is open",
                        "service": service_name,
                        "retry_after": 60
                    },
                    headers={"Retry-After": "60"}
                )
                
            except httpx.TimeoutException:
                duration = time.time() - start_time
                await self.monitoring_manager.record_request(
                    service=service_name,
                    endpoint=full_path,
                    method=request.method,
                    duration=duration,
                    status_code=504,
                    user_id=user_id,
                    error="Gateway timeout",
                    span=span
                )
                
                logger.error("Request timeout", service=service_name, path=full_path)
                raise HTTPException(status_code=504, detail="Gateway timeout")
                
            except httpx.ConnectError:
                duration = time.time() - start_time
                await self.monitoring_manager.record_request(
                    service=service_name,
                    endpoint=full_path,
                    method=request.method,
                    duration=duration,
                    status_code=503,
                    user_id=user_id,
                    error="Service unavailable",
                    span=span
                )
                
                logger.error("Service unavailable", service=service_name, path=full_path)
                raise HTTPException(status_code=503, detail="Service unavailable")
                
            except Exception as e:
                duration = time.time() - start_time
                await self.monitoring_manager.record_request(
                    service=service_name,
                    endpoint=full_path,
                    method=request.method,
                    duration=duration,
                    status_code=502,
                    user_id=user_id,
                    error=str(e),
                    span=span
                )
                
                logger.error("Proxy error", service=service_name, error=str(e))
                raise HTTPException(status_code=502, detail="Bad gateway")
                
            finally:
                # Track request completion
                self.monitoring_manager.track_active_request(service_name, -1)
                
                # End span
                if span:
                    span.end()
                    
        # Set start time
        self.app.state.start_time = time.time()
        
    async def _check_redis(self) -> bool:
        """Check Redis connection."""
        try:
            if self.redis_client:
                await self.redis_client.ping()
                return True
        except Exception:
            pass
        return False


def create_enhanced_app() -> FastAPI:
    """Create enhanced API Gateway application."""
    config = get_config()
    gateway = EnhancedAPIGateway(config)
    return gateway.create_app()


# Create the application instance
app = create_enhanced_app()


if __name__ == "__main__":
    config = get_config()
    
    logger.info("🚀 Starting Enhanced TracSeq API Gateway",
                host=config.host,
                port=config.port,
                environment=config.environment)
    
    uvicorn.run(
        "api_gateway.enhanced_main:app",
        host=config.host,
        port=config.port,
        reload=config.is_development,
        log_config=None,
        # Production settings
        workers=4 if config.is_production else 1,
        loop="uvloop" if config.is_production else "asyncio",
        access_log=not config.is_production,
    )