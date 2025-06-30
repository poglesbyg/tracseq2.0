"""
Configuration management for TracSeq API Gateway.

Comprehensive settings for routing, authentication, rate limiting,
load balancing, and service discovery.
"""

import os
from functools import lru_cache
from typing import Any, Dict, List, Optional, Union

from pydantic import BaseModel, Field, validator
from pydantic_settings import BaseSettings


class ServiceEndpoint(BaseModel):
    """Service endpoint configuration."""
    name: str
    host: str
    port: int
    path_prefix: str
    health_check_path: str = "/health"
    timeout: int = 30
    retries: int = 3
    circuit_breaker_enabled: bool = True
    load_balancer_weight: int = 100

    @property
    def base_url(self) -> str:
        """Get the base URL for the service."""
        return f"http://{self.host}:{self.port}"

    @property
    def health_url(self) -> str:
        """Get the health check URL for the service."""
        return f"{self.base_url}{self.health_check_path}"


class AuthenticationConfig(BaseModel):
    """Authentication configuration."""
    enabled: bool = True
    jwt_secret_key: str = Field(default="your-secret-key-change-in-production")
    jwt_algorithm: str = "HS256"
    access_token_expire_minutes: int = 30

    # Exempt paths (no authentication required)
    exempt_paths: List[str] = Field(
        default=[
            "/",
            "/health",
            "/metrics",
            "/docs",
            "/openapi.json",
            "/auth/login",
            "/auth/register",
        ]
    )

    # Auth service integration
    auth_service_url: str = "http://auth-service:8080"
    token_validation_cache_ttl: int = 300  # 5 minutes


class RateLimitConfig(BaseModel):
    """Rate limiting configuration."""
    enabled: bool = True
    default_requests_per_minute: int = 100
    default_burst_size: int = 200

    # Per-service rate limits
    service_limits: Dict[str, Dict[str, int]] = Field(
        default={
            "auth": {"requests_per_minute": 50, "burst_size": 100},
            "samples": {"requests_per_minute": 200, "burst_size": 400},
            "storage": {"requests_per_minute": 150, "burst_size": 300},
            "templates": {"requests_per_minute": 100, "burst_size": 200},
            "sequencing": {"requests_per_minute": 300, "burst_size": 600},
            "notifications": {"requests_per_minute": 100, "burst_size": 200},
            "rag": {"requests_per_minute": 80, "burst_size": 160},
        }
    )

    # Redis configuration for rate limiting
    redis_url: str = "redis://localhost:6379/1"
    redis_key_prefix: str = "tracseq:ratelimit"


class LoadBalancerConfig(BaseModel):
    """Load balancer configuration."""
    enabled: bool = True
    algorithm: str = "round_robin"  # round_robin, weighted_round_robin, least_connections
    health_check_interval: int = 30  # seconds
    health_check_timeout: int = 5  # seconds
    unhealthy_threshold: int = 3  # failed health checks before marking unhealthy
    healthy_threshold: int = 2  # successful health checks before marking healthy


class CircuitBreakerConfig(BaseModel):
    """Circuit breaker configuration."""
    enabled: bool = True
    failure_threshold: int = 5  # failures before opening circuit
    recovery_timeout: int = 60  # seconds before attempting to close circuit
    expected_exception: str = "Exception"  # exception class name


class MonitoringConfig(BaseModel):
    """Monitoring and observability configuration."""
    metrics_enabled: bool = True
    tracing_enabled: bool = True
    logging_level: str = "INFO"

    # Prometheus metrics
    metrics_path: str = "/metrics"
    metrics_port: int = 9090

    # Request/response logging
    log_requests: bool = True
    log_responses: bool = True
    log_request_body: bool = False  # Security: disable in production
    log_response_body: bool = False  # Security: disable in production

    # Performance monitoring
    slow_request_threshold: float = 1.0  # seconds


class CORSConfig(BaseModel):
    """CORS configuration."""
    enabled: bool = True
    allow_origins: List[str] = Field(
        default=[
            "http://localhost:3000",
            "http://localhost:8080",
            "http://localhost:8000",
            "https://*.tracseq.com",
        ]
    )
    allow_credentials: bool = True
    allow_methods: List[str] = Field(default=["GET", "POST", "PUT", "DELETE", "OPTIONS"])
    allow_headers: List[str] = Field(default=["*"])
    max_age: int = 86400  # 24 hours


class SecurityConfig(BaseModel):
    """Security configuration."""
    # HTTPS enforcement
    force_https: bool = False

    # Security headers
    security_headers_enabled: bool = True

    # Request size limits
    max_request_size: int = 50 * 1024 * 1024  # 50MB
    max_file_size: int = 100 * 1024 * 1024  # 100MB

    # IP filtering
    ip_whitelist: List[str] = Field(default=[])
    ip_blacklist: List[str] = Field(default=[])

    # API key validation
    api_key_header: str = "X-API-Key"
    api_keys: List[str] = Field(default=[])


class TracSeqAPIGatewayConfig(BaseSettings):
    """Main configuration class for TracSeq API Gateway."""

    # Service metadata
    service_name: str = "TracSeq API Gateway"
    version: str = "0.1.0"
    environment: str = Field(default="development")
    host: str = Field(default="0.0.0.0")
    port: int = Field(default=8000)

    # Service endpoints configuration
    services: Dict[str, ServiceEndpoint] = Field(
        default={
            "auth": ServiceEndpoint(
                name="Auth Service",
                host="auth-service",
                port=8080,
                path_prefix="/auth",
                health_check_path="/api/v1/health"
            ),
            "samples": ServiceEndpoint(
                name="Sample Service",
                host="sample-service",
                port=8081,
                path_prefix="/samples",
                health_check_path="/api/v1/health"
            ),
            "storage": ServiceEndpoint(
                name="Enhanced Storage Service",
                host="enhanced-storage-service",
                port=8082,
                path_prefix="/storage",
                health_check_path="/api/v1/health"
            ),
            "templates": ServiceEndpoint(
                name="Template Service",
                host="template-service",
                port=8083,
                path_prefix="/templates",
                health_check_path="/api/v1/health"
            ),
            "sequencing": ServiceEndpoint(
                name="Sequencing Service",
                host="sequencing-service",
                port=8084,
                path_prefix="/sequencing",
                health_check_path="/api/v1/health"
            ),
            "notifications": ServiceEndpoint(
                name="Notification Service",
                host="notification-service",
                port=8085,
                path_prefix="/notifications",
                health_check_path="/api/v1/health"
            ),
            "rag": ServiceEndpoint(
                name="Enhanced RAG Service",
                host="enhanced-rag-service",
                port=8086,
                path_prefix="/rag",
                health_check_path="/api/v1/health"
            ),
            "events": ServiceEndpoint(
                name="Event Service",
                host="event-service",
                port=8087,
                path_prefix="/events",
                health_check_path="/api/v1/health"
            ),
            "transactions": ServiceEndpoint(
                name="Transaction Service",
                host="transaction-service",
                port=8088,
                path_prefix="/transactions",
                health_check_path="/api/v1/health"
            ),
        }
    )

    # Component configurations
    authentication: AuthenticationConfig = Field(default_factory=AuthenticationConfig)
    rate_limiting: RateLimitConfig = Field(default_factory=RateLimitConfig)
    load_balancer: LoadBalancerConfig = Field(default_factory=LoadBalancerConfig)
    circuit_breaker: CircuitBreakerConfig = Field(default_factory=CircuitBreakerConfig)
    monitoring: MonitoringConfig = Field(default_factory=MonitoringConfig)
    cors: CORSConfig = Field(default_factory=CORSConfig)
    security: SecurityConfig = Field(default_factory=SecurityConfig)

    # Gateway-specific settings
    request_timeout: int = 30  # seconds
    max_concurrent_requests: int = 1000
    enable_request_buffering: bool = True
    enable_response_caching: bool = True
    cache_ttl: int = 300  # 5 minutes

    # Service discovery
    service_discovery_enabled: bool = False
    consul_host: str = "localhost"
    consul_port: int = 8500

    # Database (for gateway metrics and logs)
    database_url: str = Field(
        default="postgresql://gateway_user:gateway_password@localhost:5432/tracseq_gateway"
    )

    # Redis (for caching and rate limiting)
    redis_url: str = "redis://localhost:6379/0"

    @validator("environment")
    def validate_environment(cls, v):
        """Validate environment setting."""
        if v not in ["development", "staging", "production"]:
            raise ValueError("Environment must be development, staging, or production")
        return v

    @validator("port")
    def validate_port(cls, v):
        """Validate port range."""
        if not 1 <= v <= 65535:
            raise ValueError("Port must be between 1 and 65535")
        return v

    def get_service_by_path(self, path: str) -> Optional[ServiceEndpoint]:
        """Get service configuration by request path."""
        for service in self.services.values():
            if path.startswith(service.path_prefix):
                return service
        return None

    def get_upstream_url(self, service_name: str, path: str) -> str:
        """Get the upstream URL for a service and path."""
        service = self.services.get(service_name)
        if not service:
            raise ValueError(f"Service {service_name} not found")

        # Remove the path prefix and construct upstream URL
        upstream_path = path[len(service.path_prefix):] if path.startswith(service.path_prefix) else path
        return f"{service.base_url}{upstream_path}"

    @property
    def is_production(self) -> bool:
        """Check if running in production environment."""
        return self.environment == "production"

    @property
    def is_development(self) -> bool:
        """Check if running in development environment."""
        return self.environment == "development"

    class Config:
        """Pydantic configuration."""
        env_file = ".env"
        env_file_encoding = "utf-8"
        env_nested_delimiter = "__"
        case_sensitive = False


@lru_cache
def get_config() -> TracSeqAPIGatewayConfig:
    """Get cached application configuration."""
    return TracSeqAPIGatewayConfig()


def get_service_configs() -> Dict[str, ServiceEndpoint]:
    """Get all service configurations."""
    return get_config().services


def get_auth_config() -> AuthenticationConfig:
    """Get authentication configuration."""
    return get_config().authentication


def get_rate_limit_config() -> RateLimitConfig:
    """Get rate limiting configuration."""
    return get_config().rate_limiting


def get_monitoring_config() -> MonitoringConfig:
    """Get monitoring configuration."""
    return get_config().monitoring
