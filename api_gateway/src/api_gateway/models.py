"""
API Gateway Models
Pydantic models for API Gateway configuration, requests, and responses
"""

from datetime import datetime
from typing import Dict, List, Optional, Any, Union
from enum import Enum
from pydantic import BaseModel, Field, ConfigDict, validator
from pydantic_settings import BaseSettings


class ServiceStatus(str, Enum):
    """Service health status enumeration"""
    HEALTHY = "healthy"
    DEGRADED = "degraded"
    UNHEALTHY = "unhealthy"
    UNKNOWN = "unknown"


class LoadBalancingStrategy(str, Enum):
    """Load balancing strategy enumeration"""
    ROUND_ROBIN = "round_robin"
    LEAST_CONNECTIONS = "least_connections"
    WEIGHTED_ROUND_ROBIN = "weighted_round_robin"
    IP_HASH = "ip_hash"


class CircuitBreakerState(str, Enum):
    """Circuit breaker state enumeration"""
    CLOSED = "closed"
    OPEN = "open"
    HALF_OPEN = "half_open"


# Configuration Models
class ServiceConfig(BaseModel):
    """Configuration for a microservice"""
    name: str = Field(..., description="Service name")
    url: str = Field(..., description="Service base URL")
    weight: int = Field(default=1, description="Load balancing weight")
    timeout_seconds: int = Field(default=30, description="Request timeout")
    retry_attempts: int = Field(default=3, description="Number of retry attempts")
    enabled: bool = Field(default=True, description="Service enabled status")
    health_check_path: str = Field(default="/health", description="Health check endpoint")
    health_check_interval: int = Field(default=30, description="Health check interval in seconds")


class RouteConfig(BaseModel):
    """Configuration for API routes"""
    path: str = Field(..., description="Route path pattern")
    service_name: str = Field(..., description="Target service name")
    methods: List[str] = Field(default=["GET"], description="Allowed HTTP methods")
    strip_prefix: bool = Field(default=True, description="Strip route prefix from forwarded path")
    require_auth: bool = Field(default=True, description="Require authentication")
    rate_limit: Optional[int] = Field(default=None, description="Rate limit per minute")


class GatewayConfig(BaseSettings):
    """Main gateway configuration"""
    model_config = ConfigDict(env_prefix="GATEWAY_")
    
    host: str = Field(default="0.0.0.0", description="Gateway host")
    port: int = Field(default=8089, description="Gateway port")
    debug: bool = Field(default=False, description="Debug mode")
    
    # Services configuration
    services: Dict[str, ServiceConfig] = Field(default_factory=dict)
    routes: List[RouteConfig] = Field(default_factory=list)
    
    # Load balancing
    load_balancing_strategy: LoadBalancingStrategy = Field(
        default=LoadBalancingStrategy.ROUND_ROBIN,
        description="Load balancing strategy"
    )
    
    # Circuit breaker configuration
    circuit_breaker_enabled: bool = Field(default=True, description="Enable circuit breaker")
    circuit_breaker_failure_threshold: int = Field(default=5, description="Failure threshold")
    circuit_breaker_recovery_timeout: int = Field(default=60, description="Recovery timeout in seconds")
    
    # Rate limiting
    rate_limiting_enabled: bool = Field(default=True, description="Enable rate limiting")
    default_rate_limit: int = Field(default=100, description="Default rate limit per minute")
    
    # Authentication
    jwt_secret_key: str = Field(..., description="JWT secret key")
    jwt_algorithm: str = Field(default="HS256", description="JWT algorithm")
    jwt_expiration_hours: int = Field(default=24, description="JWT expiration in hours")


# Request/Response Models
class HealthCheckResponse(BaseModel):
    """Health check response model"""
    status: ServiceStatus = Field(..., description="Service status")
    timestamp: datetime = Field(default_factory=datetime.utcnow, description="Check timestamp")
    version: str = Field(..., description="Service version")
    uptime_seconds: float = Field(..., description="Service uptime in seconds")
    dependencies: Dict[str, ServiceStatus] = Field(default_factory=dict, description="Dependency statuses")


class ServiceHealthResponse(BaseModel):
    """Individual service health response"""
    service_name: str = Field(..., description="Service name")
    status: ServiceStatus = Field(..., description="Service health status")
    url: str = Field(..., description="Service URL")
    response_time_ms: Optional[float] = Field(default=None, description="Response time in milliseconds")
    last_check: datetime = Field(..., description="Last health check timestamp")
    error_message: Optional[str] = Field(default=None, description="Error message if unhealthy")


class LoadBalancerStats(BaseModel):
    """Load balancer statistics"""
    total_requests: int = Field(..., description="Total requests processed")
    successful_requests: int = Field(..., description="Successful requests")
    failed_requests: int = Field(..., description="Failed requests")
    average_response_time_ms: float = Field(..., description="Average response time")
    active_connections: int = Field(..., description="Active connections")
    service_stats: Dict[str, Dict[str, Union[int, float]]] = Field(
        default_factory=dict,
        description="Per-service statistics"
    )


class CircuitBreakerStatus(BaseModel):
    """Circuit breaker status"""
    service_name: str = Field(..., description="Service name")
    state: CircuitBreakerState = Field(..., description="Circuit breaker state")
    failure_count: int = Field(..., description="Current failure count")
    failure_threshold: int = Field(..., description="Failure threshold")
    last_failure_time: Optional[datetime] = Field(default=None, description="Last failure timestamp")
    next_attempt_time: Optional[datetime] = Field(default=None, description="Next attempt timestamp")


class RateLimitStatus(BaseModel):
    """Rate limit status"""
    client_id: str = Field(..., description="Client identifier")
    requests_count: int = Field(..., description="Current request count")
    limit: int = Field(..., description="Rate limit")
    window_start: datetime = Field(..., description="Rate limit window start")
    window_end: datetime = Field(..., description="Rate limit window end")
    blocked: bool = Field(..., description="Whether client is blocked")


class GatewayMetrics(BaseModel):
    """Gateway metrics response"""
    uptime_seconds: float = Field(..., description="Gateway uptime")
    total_requests: int = Field(..., description="Total requests processed")
    requests_per_second: float = Field(..., description="Current requests per second")
    error_rate: float = Field(..., description="Error rate percentage")
    average_latency_ms: float = Field(..., description="Average latency in milliseconds")
    active_connections: int = Field(..., description="Active connections")
    memory_usage_mb: float = Field(..., description="Memory usage in MB")
    cpu_usage_percent: float = Field(..., description="CPU usage percentage")


class ProxyRequest(BaseModel):
    """Proxy request model"""
    method: str = Field(..., description="HTTP method")
    path: str = Field(..., description="Request path")
    headers: Dict[str, str] = Field(default_factory=dict, description="Request headers")
    query_params: Dict[str, str] = Field(default_factory=dict, description="Query parameters")
    body: Optional[bytes] = Field(default=None, description="Request body")


class ProxyResponse(BaseModel):
    """Proxy response model"""
    status_code: int = Field(..., description="HTTP status code")
    headers: Dict[str, str] = Field(default_factory=dict, description="Response headers")
    body: bytes = Field(..., description="Response body")
    service_name: str = Field(..., description="Source service name")
    response_time_ms: float = Field(..., description="Response time in milliseconds")


class AuthRequest(BaseModel):
    """Authentication request model"""
    username: str = Field(..., description="Username")
    password: str = Field(..., min_length=8, description="Password")


class AuthResponse(BaseModel):
    """Authentication response model"""
    access_token: str = Field(..., description="JWT access token")
    token_type: str = Field(default="bearer", description="Token type")
    expires_in: int = Field(..., description="Token expiration in seconds")
    user_id: str = Field(..., description="User ID")
    roles: List[str] = Field(default_factory=list, description="User roles")


class ErrorResponse(BaseModel):
    """Error response model"""
    error: str = Field(..., description="Error type")
    message: str = Field(..., description="Error message")
    details: Optional[Dict[str, Any]] = Field(default=None, description="Additional error details")
    timestamp: datetime = Field(default_factory=datetime.utcnow, description="Error timestamp")
    trace_id: Optional[str] = Field(default=None, description="Request trace ID")


class ServiceDiscoveryEvent(BaseModel):
    """Service discovery event model"""
    event_type: str = Field(..., description="Event type (register, deregister, update)")
    service_name: str = Field(..., description="Service name")
    service_config: ServiceConfig = Field(..., description="Service configuration")
    timestamp: datetime = Field(default_factory=datetime.utcnow, description="Event timestamp")


# Validation models
class RouteValidationResult(BaseModel):
    """Route validation result"""
    is_valid: bool = Field(..., description="Whether route is valid")
    errors: List[str] = Field(default_factory=list, description="Validation errors")
    warnings: List[str] = Field(default_factory=list, description="Validation warnings")


class ConfigValidationResult(BaseModel):
    """Configuration validation result"""
    is_valid: bool = Field(..., description="Whether configuration is valid")
    errors: List[str] = Field(default_factory=list, description="Validation errors")
    warnings: List[str] = Field(default_factory=list, description="Validation warnings")
    service_count: int = Field(..., description="Number of configured services")
    route_count: int = Field(..., description="Number of configured routes")


# Monitoring models
class AlertRule(BaseModel):
    """Alert rule configuration"""
    name: str = Field(..., description="Alert rule name")
    metric: str = Field(..., description="Metric to monitor")
    threshold: float = Field(..., description="Alert threshold")
    operator: str = Field(..., description="Comparison operator (>, <, >=, <=, ==)")
    duration_seconds: int = Field(default=60, description="Alert duration")
    enabled: bool = Field(default=True, description="Whether alert is enabled")


class Alert(BaseModel):
    """Active alert"""
    rule_name: str = Field(..., description="Alert rule name")
    metric: str = Field(..., description="Metric name")
    current_value: float = Field(..., description="Current metric value")
    threshold: float = Field(..., description="Alert threshold")
    started_at: datetime = Field(..., description="Alert start time")
    message: str = Field(..., description="Alert message")
    severity: str = Field(..., description="Alert severity")


# Custom validators
@validator('url', pre=True)
def validate_url(cls, v):
    """Validate URL format"""
    if not v.startswith(('http://', 'https://')):
        raise ValueError('URL must start with http:// or https://')
    return v


@validator('methods', pre=True)
def validate_methods(cls, v):
    """Validate HTTP methods"""
    valid_methods = {'GET', 'POST', 'PUT', 'DELETE', 'PATCH', 'HEAD', 'OPTIONS'}
    if isinstance(v, list):
        for method in v:
            if method.upper() not in valid_methods:
                raise ValueError(f'Invalid HTTP method: {method}')
        return [method.upper() for method in v]
    return v 
