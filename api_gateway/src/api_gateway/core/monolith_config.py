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
    AuthenticationConfig,
    CircuitBreakerConfig,
    CORSConfig,
    LoadBalancerConfig,
    MonitoringConfig,
    RateLimitConfig,
    SecurityConfig,
    ServiceEndpoint,
)


class MonolithEndpoint(BaseModel):
    """Monolith endpoint configuration."""
    name: str = "Lab Manager Monolith"
    host: str = "host.docker.internal"  # Routes to host for backend access
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


class ServiceFeatureFlags(BaseSettings):
    """Feature flags for gradual service extraction."""
    use_auth_service: bool = Field(default=False)
    use_sample_service: bool = Field(default=False)
    use_template_service: bool = Field(default=False)
    use_storage_service: bool = Field(default=False)
    use_sequencing_service: bool = Field(default=False)
    use_notification_service: bool = Field(default=False)
    use_rag_service: bool = Field(default=False)
    # Additional services from Phase 1
    use_barcode_service: bool = Field(default=False)
    use_qaqc_service: bool = Field(default=False)
    use_library_service: bool = Field(default=False)
    use_event_service: bool = Field(default=False)
    use_transaction_service: bool = Field(default=False)
    use_spreadsheet_service: bool = Field(default=False)
    # Phase 2 services
    use_dashboard_service: bool = Field(default=False)
    use_reports_service: bool = Field(default=False)

    class Config:
        """Pydantic configuration for reading environment variables."""
        env_file = ".env"
        env_file_encoding = "utf-8"
        case_sensitive = False


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
            # Additional services
            "barcode": ServiceEndpoint(
                name="Barcode Service",
                host="barcode-service",
                port=3020,
                path_prefix="/api/barcodes",
                health_check_path="/health"
            ),
            "qaqc": ServiceEndpoint(
                name="QA/QC Service",
                host="qaqc-service",
                port=3018,
                path_prefix="/api/qaqc",
                health_check_path="/health"
            ),
            "library": ServiceEndpoint(
                name="Library Details Service",
                host="library-details-service",
                port=3021,
                path_prefix="/api/library",
                health_check_path="/health"
            ),
            "event": ServiceEndpoint(
                name="Event Service",
                host="event-service",
                port=3017,
                path_prefix="/api/events",
                health_check_path="/health"
            ),
            "transaction": ServiceEndpoint(
                name="Transaction Service",
                host="transaction-service",
                port=8088,
                path_prefix="/api/transactions",
                health_check_path="/health"
            ),
            "spreadsheet": ServiceEndpoint(
                name="Spreadsheet Service",
                host="spreadsheet-versioning-service",
                port=3015,
                path_prefix="/api/spreadsheets",
                health_check_path="/health"
            ),
            # Phase 2 services
            "dashboard": ServiceEndpoint(
                name="Dashboard Service",
                host="dashboard-service",
                port=3025,
                path_prefix="/api/dashboard",
                health_check_path="/health"
            ),
            "reports": ServiceEndpoint(
                name="Reports Service",
                host="reports-service",
                port=3026,
                path_prefix="/api/reports",
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
            # Additional service routes
            ("/api/barcodes", "barcode", self.feature_flags.use_barcode_service),
            ("/api/qaqc", "qaqc", self.feature_flags.use_qaqc_service),
            ("/api/library", "library", self.feature_flags.use_library_service),
            ("/api/events", "event", self.feature_flags.use_event_service),
            ("/api/transactions", "transaction", self.feature_flags.use_transaction_service),
            ("/api/spreadsheets", "spreadsheet", self.feature_flags.use_spreadsheet_service),
            # Phase 2 service routes
            ("/api/dashboard", "dashboard", self.feature_flags.use_dashboard_service),
            ("/api/reports", "reports", self.feature_flags.use_reports_service),
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
                },
                # Additional services status
                "barcode": {
                    "enabled": self.feature_flags.use_barcode_service,
                    "url": self.microservices["barcode"].base_url if self.feature_flags.use_barcode_service else None
                },
                "qaqc": {
                    "enabled": self.feature_flags.use_qaqc_service,
                    "url": self.microservices["qaqc"].base_url if self.feature_flags.use_qaqc_service else None
                },
                "library": {
                    "enabled": self.feature_flags.use_library_service,
                    "url": self.microservices["library"].base_url if self.feature_flags.use_library_service else None
                },
                "event": {
                    "enabled": self.feature_flags.use_event_service,
                    "url": self.microservices["event"].base_url if self.feature_flags.use_event_service else None
                },
                "transaction": {
                    "enabled": self.feature_flags.use_transaction_service,
                    "url": self.microservices["transaction"].base_url if self.feature_flags.use_transaction_service else None
                },
                "spreadsheet": {
                    "enabled": self.feature_flags.use_spreadsheet_service,
                    "url": self.microservices["spreadsheet"].base_url if self.feature_flags.use_spreadsheet_service else None
                },
                # Phase 2 services status
                "dashboard": {
                    "enabled": self.feature_flags.use_dashboard_service,
                    "url": self.microservices["dashboard"].base_url if self.feature_flags.use_dashboard_service else None
                },
                "reports": {
                    "enabled": self.feature_flags.use_reports_service,
                    "url": self.microservices["reports"].base_url if self.feature_flags.use_reports_service else None
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


@lru_cache
def get_monolith_config() -> MonolithRouterConfig:
    """Get cached monolith router configuration."""
    return MonolithRouterConfig()
