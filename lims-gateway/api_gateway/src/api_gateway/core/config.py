"""
Configuration management for the TracSeq 2.0 API Gateway.

This module provides a comprehensive configuration system with
environment variable support and sensible defaults.
"""

import os
from dataclasses import dataclass
from typing import Dict, List, Optional, Any
from pydantic import BaseModel, Field, validator


class DatabaseConfig(BaseModel):
    """Database configuration settings."""
    
    url: str = Field(
        default="postgres://postgres:postgres@localhost:5432/lims_db",
        description="PostgreSQL connection URL"
    )
    pool_min_size: int = Field(
        default=2,
        description="Minimum number of connections in the pool"
    )
    pool_max_size: int = Field(
        default=10,
        description="Maximum number of connections in the pool"
    )
    connection_timeout: int = Field(
        default=30,
        description="Connection timeout in seconds"
    )
    pool_recycle: int = Field(
        default=3600,
        description="Pool recycle time in seconds"
    )
    echo: bool = Field(
        default=False,
        description="Enable SQL query logging"
    )


class SecurityConfig(BaseModel):
    """Security configuration settings."""
    
    jwt_secret_key: str = Field(
        default="dev-secret-key-change-in-production",
        description="JWT signing secret key"
    )
    jwt_algorithm: str = Field(
        default="HS256",
        description="JWT signing algorithm"
    )
    jwt_expiration_hours: int = Field(
        default=24,
        description="JWT token expiration time in hours"
    )
    rate_limit_requests: int = Field(
        default=100,
        description="Rate limit requests per window"
    )
    rate_limit_window: int = Field(
        default=60,
        description="Rate limit window in seconds"
    )
    adaptive_rate_limiting: bool = Field(
        default=True,
        description="Enable adaptive rate limiting"
    )
    enable_csrf_protection: bool = Field(
        default=True,
        description="Enable CSRF protection"
    )
    
    @validator('jwt_secret_key')
    def validate_jwt_secret(cls, v):
        if len(v) < 32:
            raise ValueError('JWT secret key must be at least 32 characters long')
        return v


class GatewayConfig(BaseModel):
    """API Gateway configuration settings."""
    
    host: str = Field(
        default="0.0.0.0",
        description="Gateway host address"
    )
    port: int = Field(
        default=8000,
        description="Gateway port number"
    )
    debug: bool = Field(
        default=False,
        description="Enable debug mode"
    )
    workers: int = Field(
        default=1,
        description="Number of worker processes"
    )
    max_request_size: int = Field(
        default=16 * 1024 * 1024,  # 16MB
        description="Maximum request size in bytes"
    )
    request_timeout: int = Field(
        default=30,
        description="Request timeout in seconds"
    )


class ServiceConfig(BaseModel):
    """Microservices configuration."""
    
    auth_service_url: str = Field(
        default="http://lims-auth:8000",
        description="Authentication service URL"
    )
    sample_service_url: str = Field(
        default="http://sample-service:8081",
        description="Sample service URL"
    )
    storage_service_url: str = Field(
        default="http://storage-service:8082",
        description="Storage service URL"
    )
    template_service_url: str = Field(
        default="http://lims-templates:8000",
        description="Template service URL"
    )
    sequencing_service_url: str = Field(
        default="http://sequencing-service:8084",
        description="Sequencing service URL"
    )
    rag_service_url: str = Field(
        default="http://rag-service:8000",
        description="RAG service URL"
    )
    notification_service_url: str = Field(
        default="http://notification-service:8085",
        description="Notification service URL"
    )
    
    # Service timeouts and retries
    service_timeout: int = Field(
        default=30,
        description="Default service timeout in seconds"
    )
    service_retries: int = Field(
        default=3,
        description="Default number of service retries"
    )
    service_retry_delay: float = Field(
        default=1.0,
        description="Delay between service retries in seconds"
    )


class LoggingConfig(BaseModel):
    """Logging configuration settings."""
    
    log_level: str = Field(
        default="INFO",
        description="Logging level"
    )
    log_file: Optional[str] = Field(
        default=None,
        description="Log file path"
    )
    enable_access_log: bool = Field(
        default=True,
        description="Enable access logging"
    )
    enable_sql_logging: bool = Field(
        default=False,
        description="Enable SQL query logging"
    )
    log_format: str = Field(
        default="json",
        description="Log format: json or text"
    )
    log_rotation: bool = Field(
        default=True,
        description="Enable log rotation"
    )
    log_max_size: str = Field(
        default="100MB",
        description="Maximum log file size"
    )
    log_backup_count: int = Field(
        default=5,
        description="Number of backup log files to keep"
    )
    
    @validator('log_level')
    def validate_log_level(cls, v):
        valid_levels = ['DEBUG', 'INFO', 'WARNING', 'ERROR', 'CRITICAL']
        if v.upper() not in valid_levels:
            raise ValueError(f'Log level must be one of: {valid_levels}')
        return v.upper()


class MonitoringConfig(BaseModel):
    """Monitoring and observability configuration."""
    
    enable_metrics: bool = Field(
        default=True,
        description="Enable metrics collection"
    )
    metrics_port: int = Field(
        default=9090,
        description="Metrics server port"
    )
    health_check_interval: int = Field(
        default=30,
        description="Health check interval in seconds"
    )
    circuit_breaker_failure_threshold: int = Field(
        default=5,
        description="Circuit breaker failure threshold"
    )
    circuit_breaker_recovery_timeout: int = Field(
        default=60,
        description="Circuit breaker recovery timeout in seconds"
    )
    circuit_breaker_half_open_max_calls: int = Field(
        default=3,
        description="Maximum calls in half-open state"
    )
    enable_tracing: bool = Field(
        default=False,
        description="Enable distributed tracing"
    )
    tracing_sample_rate: float = Field(
        default=0.1,
        description="Tracing sample rate (0.0 to 1.0)"
    )


class CORSConfig(BaseModel):
    """CORS configuration settings."""
    
    allow_origins: List[str] = Field(
        default=["http://localhost:3000", "http://localhost:8080"],
        description="Allowed CORS origins"
    )
    allow_credentials: bool = Field(
        default=True,
        description="Allow credentials in CORS requests"
    )
    allow_methods: List[str] = Field(
        default=["*"],
        description="Allowed CORS methods"
    )
    allow_headers: List[str] = Field(
        default=["*"],
        description="Allowed CORS headers"
    )
    max_age: int = Field(
        default=600,
        description="CORS preflight cache max age in seconds"
    )


class ChatConfig(BaseModel):
    """Chat and AI configuration settings."""
    
    enable_chat: bool = Field(
        default=True,
        description="Enable chat functionality"
    )
    chat_model: str = Field(
        default="gpt-3.5-turbo",
        description="Chat model to use"
    )
    chat_max_tokens: int = Field(
        default=2000,
        description="Maximum tokens per chat response"
    )
    chat_temperature: float = Field(
        default=0.7,
        description="Chat model temperature"
    )
    chat_timeout: int = Field(
        default=30,
        description="Chat request timeout in seconds"
    )


class AppConfig(BaseModel):
    """Main application configuration."""
    
    environment: str = Field(
        default="development",
        description="Application environment"
    )
    app_name: str = Field(
        default="TracSeq API Gateway",
        description="Application name"
    )
    app_version: str = Field(
        default="2.0.0",
        description="Application version"
    )
    debug: bool = Field(
        default=False,
        description="Global debug mode"
    )
    testing: bool = Field(
        default=False,
        description="Testing mode"
    )
    
    # Sub-configurations
    database: DatabaseConfig = Field(default_factory=DatabaseConfig)
    security: SecurityConfig = Field(default_factory=SecurityConfig)
    gateway: GatewayConfig = Field(default_factory=GatewayConfig)
    services: ServiceConfig = Field(default_factory=ServiceConfig)
    logging: LoggingConfig = Field(default_factory=LoggingConfig)
    monitoring: MonitoringConfig = Field(default_factory=MonitoringConfig)
    cors: CORSConfig = Field(default_factory=CORSConfig)
    chat: ChatConfig = Field(default_factory=ChatConfig)
    
    @validator('environment')
    def validate_environment(cls, v):
        valid_envs = ['development', 'staging', 'production', 'testing']
        if v.lower() not in valid_envs:
            raise ValueError(f'Environment must be one of: {valid_envs}')
        return v.lower()
    
    def is_production(self) -> bool:
        """Check if running in production environment."""
        return self.environment == "production"
    
    def is_development(self) -> bool:
        """Check if running in development environment."""
        return self.environment == "development"
    
    def is_testing(self) -> bool:
        """Check if running in testing environment."""
        return self.environment == "testing" or self.testing

    def get_service_by_path(self, path: str):
        """
        Find the appropriate service endpoint for a given path.
        
        This method provides routing logic for the API Gateway.
        It imports the routing module to avoid circular dependencies.
        
        Args:
            path: The request path to match
            
        Returns:
            ServiceEndpoint if found, None otherwise
        """
        from api_gateway.core.routing import get_service_by_path
        return get_service_by_path(path)


# Global configuration instance
_config: Optional[AppConfig] = None


def load_config_from_env() -> AppConfig:
    """Load configuration from environment variables."""
    # Database configuration
    db_config = DatabaseConfig(
        url=os.getenv("DATABASE_URL", "postgres://postgres:postgres@localhost:5432/lims_db"),
        pool_min_size=int(os.getenv("DB_POOL_MIN_SIZE", "2")),
        pool_max_size=int(os.getenv("DB_POOL_MAX_SIZE", "10")),
        connection_timeout=int(os.getenv("DB_CONNECTION_TIMEOUT", "30")),
        pool_recycle=int(os.getenv("DB_POOL_RECYCLE", "3600")),
        echo=os.getenv("DB_ECHO", "false").lower() == "true"
    )
    
    # Security configuration
    security_config = SecurityConfig(
        jwt_secret_key=os.getenv("JWT_SECRET_KEY", "dev-secret-key-change-in-production"),
        jwt_algorithm=os.getenv("JWT_ALGORITHM", "HS256"),
        jwt_expiration_hours=int(os.getenv("JWT_EXPIRATION_HOURS", "24")),
        rate_limit_requests=int(os.getenv("RATE_LIMIT_REQUESTS", "100")),
        rate_limit_window=int(os.getenv("RATE_LIMIT_WINDOW", "60")),
        adaptive_rate_limiting=os.getenv("ADAPTIVE_RATE_LIMITING", "true").lower() == "true",
        enable_csrf_protection=os.getenv("ENABLE_CSRF_PROTECTION", "true").lower() == "true"
    )
    
    # Gateway configuration
    gateway_config = GatewayConfig(
        host=os.getenv("GATEWAY_HOST", "0.0.0.0"),
        port=int(os.getenv("GATEWAY_PORT", "8000")),
        debug=os.getenv("GATEWAY_DEBUG", "false").lower() == "true",
        workers=int(os.getenv("GATEWAY_WORKERS", "1")),
        max_request_size=int(os.getenv("MAX_REQUEST_SIZE", str(16 * 1024 * 1024))),
        request_timeout=int(os.getenv("REQUEST_TIMEOUT", "30"))
    )
    
    # Service configuration
    service_config = ServiceConfig(
        auth_service_url=os.getenv("AUTH_SERVICE_URL", "http://lims-auth:8000"),
        sample_service_url=os.getenv("SAMPLE_SERVICE_URL", "http://sample-service:8081"),
        storage_service_url=os.getenv("STORAGE_SERVICE_URL", "http://storage-service:8082"),
        template_service_url=os.getenv("TEMPLATE_SERVICE_URL", "http://lims-templates:8000"),
        sequencing_service_url=os.getenv("SEQUENCING_SERVICE_URL", "http://sequencing-service:8084"),
        rag_service_url=os.getenv("RAG_SERVICE_URL", "http://rag-service:8000"),
        notification_service_url=os.getenv("NOTIFICATION_SERVICE_URL", "http://notification-service:8085"),
        service_timeout=int(os.getenv("SERVICE_TIMEOUT", "30")),
        service_retries=int(os.getenv("SERVICE_RETRIES", "3")),
        service_retry_delay=float(os.getenv("SERVICE_RETRY_DELAY", "1.0"))
    )
    
    # Logging configuration
    logging_config = LoggingConfig(
        log_level=os.getenv("LOG_LEVEL", "INFO").upper(),
        log_file=os.getenv("LOG_FILE"),
        enable_access_log=os.getenv("ENABLE_ACCESS_LOG", "true").lower() == "true",
        enable_sql_logging=os.getenv("ENABLE_SQL_LOGGING", "false").lower() == "true",
        log_format=os.getenv("LOG_FORMAT", "json"),
        log_rotation=os.getenv("LOG_ROTATION", "true").lower() == "true",
        log_max_size=os.getenv("LOG_MAX_SIZE", "100MB"),
        log_backup_count=int(os.getenv("LOG_BACKUP_COUNT", "5"))
    )
    
    # Monitoring configuration
    monitoring_config = MonitoringConfig(
        enable_metrics=os.getenv("ENABLE_METRICS", "true").lower() == "true",
        metrics_port=int(os.getenv("METRICS_PORT", "9090")),
        health_check_interval=int(os.getenv("HEALTH_CHECK_INTERVAL", "30")),
        circuit_breaker_failure_threshold=int(os.getenv("CIRCUIT_BREAKER_FAILURE_THRESHOLD", "5")),
        circuit_breaker_recovery_timeout=int(os.getenv("CIRCUIT_BREAKER_RECOVERY_TIMEOUT", "60")),
        circuit_breaker_half_open_max_calls=int(os.getenv("CIRCUIT_BREAKER_HALF_OPEN_MAX_CALLS", "3")),
        enable_tracing=os.getenv("ENABLE_TRACING", "false").lower() == "true",
        tracing_sample_rate=float(os.getenv("TRACING_SAMPLE_RATE", "0.1"))
    )
    
    # CORS configuration
    cors_origins = os.getenv("CORS_ORIGINS", "http://localhost:3000,http://localhost:8080").split(",")
    cors_methods = os.getenv("CORS_METHODS", "*").split(",") if os.getenv("CORS_METHODS") != "*" else ["*"]
    cors_headers = os.getenv("CORS_HEADERS", "*").split(",") if os.getenv("CORS_HEADERS") != "*" else ["*"]
    
    cors_config = CORSConfig(
        allow_origins=cors_origins,
        allow_credentials=os.getenv("CORS_CREDENTIALS", "true").lower() == "true",
        allow_methods=cors_methods,
        allow_headers=cors_headers,
        max_age=int(os.getenv("CORS_MAX_AGE", "600"))
    )
    
    # Chat configuration
    chat_config = ChatConfig(
        enable_chat=os.getenv("ENABLE_CHAT", "true").lower() == "true",
        chat_model=os.getenv("CHAT_MODEL", "gpt-3.5-turbo"),
        chat_max_tokens=int(os.getenv("CHAT_MAX_TOKENS", "2000")),
        chat_temperature=float(os.getenv("CHAT_TEMPERATURE", "0.7")),
        chat_timeout=int(os.getenv("CHAT_TIMEOUT", "30"))
    )
    
    # Main application configuration
    app_config = AppConfig(
        environment=os.getenv("ENVIRONMENT", "development").lower(),
        app_name=os.getenv("APP_NAME", "TracSeq API Gateway"),
        app_version=os.getenv("APP_VERSION", "2.0.0"),
        debug=os.getenv("DEBUG", "false").lower() == "true",
        testing=os.getenv("TESTING", "false").lower() == "true",
        database=db_config,
        security=security_config,
        gateway=gateway_config,
        services=service_config,
        logging=logging_config,
        monitoring=monitoring_config,
        cors=cors_config,
        chat=chat_config
    )
    
    return app_config


def get_config() -> AppConfig:
    """
    Get the global configuration instance.
    
    This function implements a singleton pattern to ensure configuration
    is loaded only once and reused throughout the application.
    
    Returns:
        AppConfig: The application configuration instance
    """
    global _config
    if _config is None:
        _config = load_config_from_env()
    return _config


def reload_config() -> AppConfig:
    """
    Reload the configuration from environment variables.
    
    This is useful for testing or when configuration needs to be updated
    at runtime.
    
    Returns:
        AppConfig: The reloaded configuration instance
    """
    global _config
    _config = load_config_from_env()
    return _config


def get_database_config() -> DatabaseConfig:
    """Get database configuration."""
    return get_config().database


def get_security_config() -> SecurityConfig:
    """Get security configuration."""
    return get_config().security


def get_gateway_config() -> GatewayConfig:
    """Get gateway configuration."""
    return get_config().gateway


def get_service_config() -> ServiceConfig:
    """Get service configuration."""
    return get_config().services


def get_logging_config() -> LoggingConfig:
    """Get logging configuration."""
    return get_config().logging


def get_monitoring_config() -> MonitoringConfig:
    """Get monitoring configuration."""
    return get_config().monitoring


def get_cors_config() -> CORSConfig:
    """Get CORS configuration."""
    return get_config().cors


def get_chat_config() -> ChatConfig:
    """Get chat configuration."""
    return get_config().chat


# Legacy support for the existing simple_main.py configuration
def get_legacy_config() -> Dict[str, Any]:
    """
    Get configuration in the format expected by simple_main.py.
    
    This function provides backward compatibility with the existing
    monolithic configuration while we transition to the modular architecture.
    
    Returns:
        Dict[str, Any]: Configuration dictionary in legacy format
    """
    config = get_config()
    
    return {
        # Gateway settings
        "GATEWAY_HOST": config.gateway.host,
        "GATEWAY_PORT": config.gateway.port,
        "GATEWAY_DEBUG": config.gateway.debug,
        
        # Service URLs
        "AUTH_SERVICE_URL": config.services.auth_service_url,
        "SAMPLE_SERVICE_URL": config.services.sample_service_url,
        "STORAGE_SERVICE_URL": config.services.storage_service_url,
        "TEMPLATE_SERVICE_URL": config.services.template_service_url,
        "SEQUENCING_SERVICE_URL": config.services.sequencing_service_url,
        "RAG_SERVICE_URL": config.services.rag_service_url,
        "NOTIFICATION_SERVICE_URL": config.services.notification_service_url,
        
        # Database settings
        "DATABASE_URL": config.database.url,
        "DB_POOL_MIN_SIZE": config.database.pool_min_size,
        "DB_POOL_MAX_SIZE": config.database.pool_max_size,
        
        # Security settings
        "JWT_SECRET_KEY": config.security.jwt_secret_key,
        "JWT_ALGORITHM": config.security.jwt_algorithm,
        "JWT_EXPIRATION_HOURS": config.security.jwt_expiration_hours,
        
        # Logging settings
        "LOG_LEVEL": config.logging.log_level,
        "LOG_FILE": config.logging.log_file,
        "ENABLE_ACCESS_LOG": config.logging.enable_access_log,
        "ENABLE_SQL_LOGGING": config.logging.enable_sql_logging,
        
        # Monitoring settings
        "ENABLE_METRICS": config.monitoring.enable_metrics,
        "HEALTH_CHECK_INTERVAL": config.monitoring.health_check_interval,
        "CIRCUIT_BREAKER_FAILURE_THRESHOLD": config.monitoring.circuit_breaker_failure_threshold,
        "CIRCUIT_BREAKER_RECOVERY_TIMEOUT": config.monitoring.circuit_breaker_recovery_timeout,
        
        # CORS settings
        "CORS_ORIGINS": config.cors.allow_origins,
        "CORS_CREDENTIALS": config.cors.allow_credentials,
        "CORS_METHODS": config.cors.allow_methods,
        "CORS_HEADERS": config.cors.allow_headers,
        
        # Environment
        "ENVIRONMENT": config.environment,
        "DEBUG": config.debug,
        "TESTING": config.testing,
    }


# Export commonly used configurations for easy access
__all__ = [
    "AppConfig",
    "DatabaseConfig", 
    "SecurityConfig",
    "GatewayConfig",
    "ServiceConfig",
    "LoggingConfig",
    "MonitoringConfig",
    "CORSConfig",
    "ChatConfig",
    "get_config",
    "reload_config",
    "get_database_config",
    "get_security_config", 
    "get_gateway_config",
    "get_service_config",
    "get_logging_config",
    "get_monitoring_config",
    "get_cors_config",
    "get_chat_config",
    "get_legacy_config"
]
