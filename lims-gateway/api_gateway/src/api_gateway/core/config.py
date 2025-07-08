"""
TracSeq API Gateway Configuration
Centralized configuration management for the API Gateway
"""

import os
from dataclasses import dataclass, field
from typing import Dict, List, Optional
from functools import lru_cache

@dataclass
class CORSConfig:
    """CORS configuration"""
    enabled: bool = True
    allow_origins: List[str] = field(default_factory=lambda: [
        "http://localhost:3000",
        "http://localhost:5173", 
        "http://localhost:8000"
    ])
    allow_credentials: bool = True
    allow_methods: List[str] = field(default_factory=lambda: ["*"])
    allow_headers: List[str] = field(default_factory=lambda: ["*"])

@dataclass
class AuthenticationConfig:
    """Authentication configuration"""
    enabled: bool = True
    jwt_secret_key: str = "this-is-a-secure-32-character-jwt-secret-key-change-in-production"
    jwt_algorithm: str = "HS256"
    token_expiry: int = 1800  # 30 minutes
    enable_permissions: bool = True
    permission_config: Dict = field(default_factory=dict)
    security_headers: bool = True

@dataclass
class ServiceEndpoint:
    """Service endpoint configuration"""
    name: str
    base_url: str
    path_prefix: str
    health_check_path: str = "/health"
    timeout: float = 30.0
    require_auth: bool = True
    rate_limit: int = 100
    strip_path_prefix: bool = True
    add_version_prefix: bool = False
    custom_headers: Dict[str, str] = field(default_factory=dict)
    
    @property
    def health_url(self) -> str:
        """Get the full health check URL"""
        return f"{self.base_url}{self.health_check_path}"

@dataclass
class TracSeqAPIGatewayConfig:
    """Main API Gateway configuration"""
    # Basic settings
    version: str = "2.0.0"
    environment: str = "development"
    host: str = "0.0.0.0"
    port: int = 8000
    
    # Request handling
    request_timeout: float = 30.0
    max_concurrent_requests: int = 1000
    
    # Feature flags
    cors: CORSConfig = field(default_factory=CORSConfig)
    authentication: AuthenticationConfig = field(default_factory=AuthenticationConfig)
    
    # Services configuration - UPDATED with correct container names and ports
    services: Dict[str, ServiceEndpoint] = field(default_factory=lambda: {
        "auth": ServiceEndpoint(
            name="Auth Service",
            base_url="http://lims-auth:8000",
            path_prefix="/api/auth",
            health_check_path="/health"
        ),
        "samples": ServiceEndpoint(
            name="Sample Service", 
            base_url="http://lims-samples:8000",
            path_prefix="/api/samples",
            health_check_path="/health"
        ),
        "storage": ServiceEndpoint(
            name="Storage Service",
            base_url="http://lims-storage:8080", 
            path_prefix="/api/storage",
            health_check_path="/health"
        ),
        "templates": ServiceEndpoint(
            name="Template Service",
            base_url="http://tracseq-templates:8083",
            path_prefix="/api/templates", 
            health_check_path="/health",
            require_auth=False,
            strip_path_prefix=True
        ),
        "sequencing": ServiceEndpoint(
            name="Sequencing Service",
            base_url="http://tracseq-sequencing:8084",
            path_prefix="/api/sequencing",
            health_check_path="/health"
        ),
        "notifications": ServiceEndpoint(
            name="Notification Service", 
            base_url="http://tracseq-notification:8085",
            path_prefix="/api/notifications",
            health_check_path="/health"
        ),
        "rag": ServiceEndpoint(
            name="RAG Service",
            base_url="http://tracseq-rag:8000",
            path_prefix="/api/rag",
            health_check_path="/health"
        ),
        "events": ServiceEndpoint(
            name="Event Service",
            base_url="http://tracseq-events:8087", 
            path_prefix="/api/events",
            health_check_path="/health"
        ),
        "transactions": ServiceEndpoint(
            name="Transaction Service",
            base_url="http://tracseq-transactions:8088",
            path_prefix="/api/transactions",
            health_check_path="/health"
        ),
        "projects": ServiceEndpoint(
            name="Project Service",
            base_url="http://tracseq-projects:8101",
            path_prefix="/api/projects",
            health_check_path="/health"
        ),
        "qaqc": ServiceEndpoint(
            name="QA/QC Service", 
            base_url="http://tracseq-qaqc:8103",
            path_prefix="/api/qaqc",
            health_check_path="/health"
        ),
        "library-prep": ServiceEndpoint(
            name="Library Prep Service",
            base_url="http://tracseq-library-prep:8102",
            path_prefix="/api/library-prep",
            health_check_path="/health"
        ),
        "flow-cells": ServiceEndpoint(
            name="Flow Cell Service",
            base_url="http://tracseq-flow-cells:8104", 
            path_prefix="/api/flow-cells",
            health_check_path="/health"
        ),
        "dashboard": ServiceEndpoint(
            name="Dashboard Service",
            base_url="http://lims-reports:8000",
            path_prefix="/api/dashboard",
            health_check_path="/health"
        ),
        "spreadsheets": ServiceEndpoint(
            name="Spreadsheet Service",
            base_url="http://tracseq-spreadsheet:8088",
            path_prefix="/api/spreadsheets", 
            health_check_path="/health"
        ),
        # Fallback service (lab manager handles everything else)
        "lab-manager": ServiceEndpoint(
            name="Lab Manager Service",
            base_url="http://lims-sequencing:8084",  # Lab manager is deployed as sequencing service
            path_prefix="/api",
            health_check_path="/health",
            strip_path_prefix=False
        )
    })
    
    @property
    def is_development(self) -> bool:
        """Check if running in development mode"""
        return self.environment.lower() in ("development", "dev")
    
    @property 
    def is_production(self) -> bool:
        """Check if running in production mode"""
        return self.environment.lower() in ("production", "prod")
    
    def get_service_by_path(self, path: str) -> Optional[ServiceEndpoint]:
        """Get service endpoint by request path"""
        # Sort by path prefix length (longest first) for better matching
        sorted_services = sorted(
            self.services.items(),
            key=lambda x: len(x[1].path_prefix),
            reverse=True
        )
        
        for service_name, endpoint in sorted_services:
            if path.startswith(endpoint.path_prefix):
                return endpoint
        
        # Fallback patterns for common endpoints
        fallback_patterns = {
            "/api/users": "auth",
            "/api/chat": "rag", 
            "/api/reports": "dashboard",
            "/api/qc": "qaqc",  # Alternative path for QA/QC
        }
        
        for pattern, service_name in fallback_patterns.items():
            if path.startswith(pattern):
                service = self.services.get(service_name)
                if service:
                    return service
        
        # Ultimate fallback to lab manager
        return self.services.get("lab-manager")

@lru_cache()
def get_config() -> TracSeqAPIGatewayConfig:
    """Get cached configuration instance"""
    config = TracSeqAPIGatewayConfig()
    
    # Override from environment variables
    config.host = os.getenv("HOST", config.host)
    config.port = int(os.getenv("PORT", str(config.port)))
    config.environment = os.getenv("ENVIRONMENT", config.environment)
    config.version = os.getenv("VERSION", config.version)
    
    # Request settings
    config.request_timeout = float(os.getenv("REQUEST_TIMEOUT", str(config.request_timeout)))
    config.max_concurrent_requests = int(os.getenv("MAX_CONCURRENT_REQUESTS", str(config.max_concurrent_requests)))
    
    # Authentication settings
    config.authentication.jwt_secret_key = os.getenv("JWT_SECRET_KEY", config.authentication.jwt_secret_key)
    config.authentication.jwt_algorithm = os.getenv("JWT_ALGORITHM", config.authentication.jwt_algorithm)
    config.authentication.token_expiry = int(os.getenv("ACCESS_TOKEN_EXPIRE_MINUTES", str(config.authentication.token_expiry // 60)) * 60)
    
    # CORS settings
    cors_origins = os.getenv("CORS__ALLOW_ORIGINS")
    if cors_origins:
        try:
            import json
            config.cors.allow_origins = json.loads(cors_origins)
        except:
            config.cors.allow_origins = cors_origins.split(",")
    
    # Override service URLs from environment
    service_url_mapping = {
        "auth": "AUTH_SERVICE_URL",
        "samples": "SAMPLE_SERVICE_URL", 
        "storage": "STORAGE_SERVICE_URL",
        "templates": "TEMPLATE_SERVICE_URL",
        "sequencing": "SEQUENCING_SERVICE_URL",
        "notifications": "NOTIFICATION_SERVICE_URL",
        "rag": "RAG_SERVICE_URL",
        "events": "EVENT_SERVICE_URL", 
        "transactions": "TRANSACTION_SERVICE_URL",
        "projects": "PROJECT_SERVICE_URL",
        "qaqc": "QAQC_SERVICE_URL",
        "library-prep": "LIBRARY_PREP_SERVICE_URL",
        "flow-cells": "FLOW_CELL_SERVICE_URL",
        "dashboard": "DASHBOARD_SERVICE_URL",
        "spreadsheets": "SPREADSHEET_SERVICE_URL"
    }
    
    for service_name, env_var in service_url_mapping.items():
        service_url = os.getenv(env_var)
        if service_url and service_name in config.services:
            config.services[service_name].base_url = service_url
    
    return config
