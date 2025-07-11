#!/usr/bin/env python3
"""
Enhanced Configuration Management for TracSeq 2.0 API Gateway
Centralized configuration handling with environment variable support
"""

import os
from typing import Dict, Any, Optional
from pydantic import BaseSettings, Field
from functools import lru_cache


class DatabaseConfig(BaseSettings):
    """Database configuration settings"""
    url: str = Field(default="postgres://postgres:postgres@lims-postgres:5432/lims_db", env="DATABASE_URL")
    pool_min_size: int = Field(default=2, env="DB_POOL_MIN_SIZE")
    pool_max_size: int = Field(default=10, env="DB_POOL_MAX_SIZE")
    pool_timeout: int = Field(default=30, env="DB_POOL_TIMEOUT")
    standardized_mode: bool = Field(default=True, env="STANDARDIZED_DB")
    disable_standardized: bool = Field(default=False, env="DISABLE_STANDARDIZED_DB")
    
    class Config:
        env_prefix = "DB_"


class ServiceConfig(BaseSettings):
    """Microservice URLs configuration"""
    auth_service_url: str = Field(default="http://auth-service:8080", env="AUTH_SERVICE_URL")
    sample_service_url: str = Field(default="http://sample-service:8081", env="SAMPLE_SERVICE_URL")
    storage_service_url: str = Field(default="http://storage-service:8082", env="STORAGE_SERVICE_URL")
    template_service_url: str = Field(default="http://template-service:8083", env="TEMPLATE_SERVICE_URL")
    sequencing_service_url: str = Field(default="http://tracseq-sequencing:8084", env="SEQUENCING_SERVICE_URL")
    notification_service_url: str = Field(default="http://tracseq-notification:8085", env="NOTIFICATION_SERVICE_URL")
    rag_service_url: str = Field(default="http://tracseq-rag:8000", env="RAG_SERVICE_URL")
    event_service_url: str = Field(default="http://tracseq-events:8087", env="EVENT_SERVICE_URL")
    transaction_service_url: str = Field(default="http://tracseq-transactions:8088", env="TRANSACTION_SERVICE_URL")
    cognitive_assistant_url: str = Field(default="http://cognitive-assistant:8000", env="COGNITIVE_ASSISTANT_URL")
    reports_service_url: str = Field(default="http://reports-service:8000", env="REPORTS_SERVICE_URL")
    
    # Lab manager services (using sequencing service as base)
    @property
    def lab_manager_url(self) -> str:
        return os.getenv("LAB_MANAGER_URL", self.sequencing_service_url)
    
    @property
    def project_service_url(self) -> str:
        return os.getenv("PROJECT_SERVICE_URL", self.lab_manager_url)
    
    @property
    def library_prep_service_url(self) -> str:
        return os.getenv("LIBRARY_PREP_SERVICE_URL", self.lab_manager_url)
    
    @property
    def qaqc_service_url(self) -> str:
        return os.getenv("QAQC_SERVICE_URL", self.lab_manager_url)
    
    @property
    def flow_cell_service_url(self) -> str:
        return os.getenv("FLOW_CELL_SERVICE_URL", self.lab_manager_url)


class GatewayConfig(BaseSettings):
    """API Gateway server configuration"""
    host: str = Field(default="0.0.0.0", env="GATEWAY_HOST")
    port: int = Field(default=8000, env="GATEWAY_PORT")
    debug: bool = Field(default=False, env="GATEWAY_DEBUG")
    title: str = Field(default="TracSeq 2.0 API Gateway")
    description: str = Field(default="Central routing hub for TracSeq microservices")
    version: str = Field(default="2.0.0")
    
    # CORS settings
    cors_origins: list = Field(default=["*"], env="CORS_ORIGINS")
    cors_credentials: bool = Field(default=True, env="CORS_CREDENTIALS")
    cors_methods: list = Field(default=["*"], env="CORS_METHODS")
    cors_headers: list = Field(default=["*"], env="CORS_HEADERS")


class SecurityConfig(BaseSettings):
    """Security and authentication configuration"""
    jwt_secret_key: str = Field(default="your-secret-key-here", env="JWT_SECRET_KEY")
    jwt_algorithm: str = Field(default="HS256", env="JWT_ALGORITHM")
    jwt_expiration_hours: int = Field(default=24, env="JWT_EXPIRATION_HOURS")
    
    # Rate limiting
    rate_limit_requests: int = Field(default=100, env="RATE_LIMIT_REQUESTS")
    rate_limit_window: int = Field(default=60, env="RATE_LIMIT_WINDOW")
    
    # API timeouts
    service_timeout: int = Field(default=30, env="SERVICE_TIMEOUT")
    health_check_timeout: int = Field(default=5, env="HEALTH_CHECK_TIMEOUT")


class LoggingConfig(BaseSettings):
    """Logging configuration"""
    log_level: str = Field(default="INFO", env="LOG_LEVEL")
    log_format: str = Field(default="%(asctime)s - %(name)s - %(levelname)s - %(message)s", env="LOG_FORMAT")
    log_file: Optional[str] = Field(default=None, env="LOG_FILE")
    enable_access_log: bool = Field(default=True, env="ENABLE_ACCESS_LOG")
    enable_sql_logging: bool = Field(default=False, env="ENABLE_SQL_LOGGING")


class MonitoringConfig(BaseSettings):
    """Monitoring and metrics configuration"""
    enable_metrics: bool = Field(default=True, env="ENABLE_METRICS")
    metrics_port: int = Field(default=9090, env="METRICS_PORT")
    enable_health_checks: bool = Field(default=True, env="ENABLE_HEALTH_CHECKS")
    health_check_interval: int = Field(default=30, env="HEALTH_CHECK_INTERVAL")
    
    # Circuit breaker settings
    circuit_breaker_failure_threshold: int = Field(default=5, env="CIRCUIT_BREAKER_FAILURE_THRESHOLD")
    circuit_breaker_recovery_timeout: int = Field(default=60, env="CIRCUIT_BREAKER_RECOVERY_TIMEOUT")


class ChatConfig(BaseSettings):
    """AI Chat service configuration"""
    enable_chat: bool = Field(default=True, env="ENABLE_CHAT")
    chat_max_file_size: int = Field(default=50 * 1024 * 1024, env="CHAT_MAX_FILE_SIZE")  # 50MB
    chat_allowed_file_types: list = Field(
        default=["application/pdf", "application/vnd.openxmlformats-officedocument.spreadsheetml.sheet", 
                "application/vnd.ms-excel", "text/csv"],
        env="CHAT_ALLOWED_FILE_TYPES"
    )
    chat_stream_chunk_size: int = Field(default=5, env="CHAT_STREAM_CHUNK_SIZE")
    chat_stream_delay: float = Field(default=0.05, env="CHAT_STREAM_DELAY")


class ApplicationConfig(BaseSettings):
    """Main application configuration aggregating all sub-configurations"""
    
    def __init__(self, **kwargs):
        super().__init__(**kwargs)
        self.database = DatabaseConfig()
        self.services = ServiceConfig()
        self.gateway = GatewayConfig()
        self.security = SecurityConfig()
        self.logging = LoggingConfig()
        self.monitoring = MonitoringConfig()
        self.chat = ChatConfig()
    
    @property
    def is_development(self) -> bool:
        """Check if running in development mode"""
        return os.getenv("ENVIRONMENT", "development").lower() == "development"
    
    @property
    def is_production(self) -> bool:
        """Check if running in production mode"""
        return os.getenv("ENVIRONMENT", "development").lower() == "production"
    
    @property
    def is_testing(self) -> bool:
        """Check if running in testing mode"""
        return os.getenv("ENVIRONMENT", "development").lower() == "testing"
    
    def get_service_url(self, service_name: str) -> str:
        """Get service URL by name"""
        service_urls = {
            "auth": self.services.auth_service_url,
            "sample": self.services.sample_service_url,
            "storage": self.services.storage_service_url,
            "template": self.services.template_service_url,
            "sequencing": self.services.sequencing_service_url,
            "notification": self.services.notification_service_url,
            "rag": self.services.rag_service_url,
            "event": self.services.event_service_url,
            "transaction": self.services.transaction_service_url,
            "cognitive": self.services.cognitive_assistant_url,
            "reports": self.services.reports_service_url,
            "lab_manager": self.services.lab_manager_url,
            "project": self.services.project_service_url,
            "library_prep": self.services.library_prep_service_url,
            "qaqc": self.services.qaqc_service_url,
            "flow_cell": self.services.flow_cell_service_url,
        }
        return service_urls.get(service_name, "")
    
    def to_dict(self) -> Dict[str, Any]:
        """Convert configuration to dictionary"""
        return {
            "database": self.database.dict(),
            "services": self.services.dict(),
            "gateway": self.gateway.dict(),
            "security": self.security.dict(),
            "logging": self.logging.dict(),
            "monitoring": self.monitoring.dict(),
            "chat": self.chat.dict(),
            "environment": {
                "is_development": self.is_development,
                "is_production": self.is_production,
                "is_testing": self.is_testing,
            }
        }


@lru_cache()
def get_config() -> ApplicationConfig:
    """Get cached application configuration"""
    return ApplicationConfig()


# Mock user data for development/testing
MOCK_USERS = {
    "admin@tracseq.com": {
        "id": "1",
        "email": "admin@tracseq.com",
        "name": "Admin User",
        "role": "admin",
        "password": "admin123"  # In production, this would be hashed
    },
    "user@tracseq.com": {
        "id": "2",
        "email": "user@tracseq.com",
        "name": "Lab User",
        "role": "user",
        "password": "user123"
    }
}


# Export commonly used configurations
config = get_config()
