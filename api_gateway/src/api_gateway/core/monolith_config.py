"""
Monolith routing configuration for TracSeq API Gateway.

This configuration routes all requests to the existing monolith initially,
with feature flags to gradually extract services.
"""

import os
from functools import lru_cache
from typing import Any, Dict, List, Optional

from pydantic import BaseModel, Field
from pydantic_settings import BaseSettings

from .config import (
    ServiceEndpoint, AuthenticationConfig, RateLimitConfig, 
    LoadBalancerConfig, CircuitBreakerConfig, MonitoringConfig, 
    CORSConfig, SecurityConfig
)


class MonolithEndpoint(BaseModel):
    """Monolith endpoint configuration."""
    name: str = "Lab Manager Monolith"
    host: str = "host.docker.internal"  # Routes to host machine
    port: int = 3000
    health_check_path: str = "/health"
    timeout: int = 30
    
    @property
    def base_url(self) -> str:
        """Get the base URL for the monolith."""
        return f"http://{self.host}:{self.port}"
    
    @property
    def health_url(self) -> str:
        """Get the health check URL for the monolith."""
        return f"{self.base_url}{self.health_check_path}"


class ServiceFeatureFlags(BaseModel):
    """Feature flags for gradual service extraction."""
    use_auth_service: bool = Field(default=False, env="USE_AUTH_SERVICE")
    use_sample_service: bool = Field(default=False, env="USE_SAMPLE_SERVICE")
    use_template_service: bool = Field(default=False, env="USE_TEMPLATE_SERVICE")
    use_storage_service: bool = Field(default=False, env="USE_STORAGE_SERVICE")
    use_sequencing_service: bool = Field(default=False, env="USE_SEQUENCING_SERVICE")
    use_notification_service: bool = Field(default=False, env="USE_NOTIFICATION_SERVICE")
    use_rag_service: bool = Field(default=False, env="USE_RAG_SERVICE")


class MonolithRouterConfig(BaseSettings):
    """Configuration for routing between monolith and microservices."""
    
    # Service metadata
    service_name: str = "TracSeq API Gateway (Monolith Router)"
    version: str = "0.1.0"
    environment: str = Field(default="development")
    host: str = Field(default="0.0.0.0")
    port: int = Field(default=8000)
    
    # Monolith configuration
    monolith: MonolithEndpoint = Field(default_factory=MonolithEndpoint)
    
    # Feature flags for service extraction
    feature_flags: ServiceFeatureFlags = Field(default_factory=ServiceFeatureFlags)
    
    # Microservice endpoints (only used when feature flags are enabled)
    microservices: Dict[str, ServiceEndpoint] = Field(
        default={
            "auth": ServiceEndpoint(
                name="Auth Service",
                host="auth-service",
                port=8080,
                path_prefix="/api/auth",
                health_check_path="/health"
            ),
            "samples": ServiceEndpoint(
                name="Sample Service",
                host="sample-service",
                port=8081,
                path_prefix="/api/samples",
                health_check_path="/health"
            ),
            "templates": ServiceEndpoint(
                name="Template Service",
                host="template-service",
                port=8083,
                path_prefix="/api/templates",
                health_check_path="/health"
            ),
            "storage": ServiceEndpoint(
                name="Storage Service",
                host="enhanced-storage-service",
                port=8082,
                path_prefix="/api/storage",
                health_check_path="/health"
            ),
            "sequencing": ServiceEndpoint(
                name="Sequencing Service",
                host="sequencing-service",
                port=8084,
                path_prefix="/api/sequencing",
                health_check_path="/health"
            ),
            "notifications": ServiceEndpoint(
                name="Notification Service",
                host="notification-service",
                port=8085,
                path_prefix="/api/notifications",
                health_check_path="/health"
            ),
            "rag": ServiceEndpoint(
                name="RAG Service",
                host="enhanced-rag-service",
                port=8086,
                path_prefix="/api/rag",
                health_check_path="/health"
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
    request_timeout: int = 30
    max_concurrent_requests: int = 1000
    
    def route_request(self, path: str) -> tuple[str, str]:
        """
        Determine where to route a request based on path and feature flags.
        
        Returns:
            tuple: (service_type, base_url) where service_type is 'monolith' or 'microservice'
        """
        # API endpoints that can be routed to microservices
        routing_rules = [
            ("/api/auth", "auth", self.feature_flags.use_auth_service),
            ("/api/samples", "samples", self.feature_flags.use_sample_service),
            ("/api/templates", "templates", self.feature_flags.use_template_service),
            ("/api/storage", "storage", self.feature_flags.use_storage_service),
            ("/api/sequencing", "sequencing", self.feature_flags.use_sequencing_service),
            ("/api/notifications", "notifications", self.feature_flags.use_notification_service),
            ("/api/rag", "rag", self.feature_flags.use_rag_service),
        ]
        
        # Check if any microservice should handle this request
        for path_prefix, service_name, is_enabled in routing_rules:
            if path.startswith(path_prefix) and is_enabled:
                service = self.microservices.get(service_name)
                if service:
                    return ("microservice", service.base_url)
        
        # Default: route to monolith
        return ("monolith", self.monolith.base_url)
    
    def get_service_status(self) -> Dict[str, Any]:
        """Get the current routing status of all services."""
        return {
            "monolith": {
                "name": self.monolith.name,
                "base_url": self.monolith.base_url,
                "health_url": self.monolith.health_url,
                "active": True
            },
            "microservices": {
                "auth": {
                    "enabled": self.feature_flags.use_auth_service,
                    "url": self.microservices["auth"].base_url if self.feature_flags.use_auth_service else None
                },
                "samples": {
                    "enabled": self.feature_flags.use_sample_service,
                    "url": self.microservices["samples"].base_url if self.feature_flags.use_sample_service else None
                },
                "templates": {
                    "enabled": self.feature_flags.use_template_service,
                    "url": self.microservices["templates"].base_url if self.feature_flags.use_template_service else None
                },
                "storage": {
                    "enabled": self.feature_flags.use_storage_service,
                    "url": self.microservices["storage"].base_url if self.feature_flags.use_storage_service else None
                },
                "sequencing": {
                    "enabled": self.feature_flags.use_sequencing_service,
                    "url": self.microservices["sequencing"].base_url if self.feature_flags.use_sequencing_service else None
                },
                "notifications": {
                    "enabled": self.feature_flags.use_notification_service,
                    "url": self.microservices["notifications"].base_url if self.feature_flags.use_notification_service else None
                },
                "rag": {
                    "enabled": self.feature_flags.use_rag_service,
                    "url": self.microservices["rag"].base_url if self.feature_flags.use_rag_service else None
                }
            }
        }
    
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


@lru_cache()
def get_monolith_config() -> MonolithRouterConfig:
    """Get cached monolith router configuration."""
    return MonolithRouterConfig() 
