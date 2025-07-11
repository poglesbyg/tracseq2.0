"""
Comprehensive logging configuration for the TracSeq 2.0 API Gateway.

This module provides structured logging with JSON formatting, correlation IDs,
and specialized loggers for different components (requests, services, database, security).
"""

import json
import logging
import logging.handlers
import sys
import uuid
from contextlib import contextmanager
from datetime import datetime
from typing import Dict, Any, Optional
from pathlib import Path

from .config import get_logging_config


# Global correlation ID for request tracing
_correlation_id: Optional[str] = None


def get_correlation_id() -> Optional[str]:
    """Get the current correlation ID for request tracing."""
    return _correlation_id


def set_correlation_id(correlation_id: Optional[str]) -> None:
    """Set the correlation ID for request tracing."""
    global _correlation_id
    _correlation_id = correlation_id


def generate_correlation_id() -> str:
    """Generate a new correlation ID."""
    return str(uuid.uuid4())


@contextmanager
def correlation_context(correlation_id: Optional[str] = None):
    """Context manager for correlation ID scoping."""
    if correlation_id is None:
        correlation_id = generate_correlation_id()
    
    old_id = get_correlation_id()
    set_correlation_id(correlation_id)
    try:
        yield correlation_id
    finally:
        set_correlation_id(old_id)


class JSONFormatter(logging.Formatter):
    """Custom JSON formatter for structured logging."""
    
    def format(self, record):
        log_entry = {
            "timestamp": datetime.utcfromtimestamp(record.created).isoformat() + "Z",
            "level": record.levelname,
            "logger": record.name,
            "message": record.getMessage(),
            "service": "api-gateway",
            "version": "2.0.0"
        }
        
        # Add correlation ID if available
        correlation_id = get_correlation_id()
        if correlation_id:
            log_entry["correlation_id"] = correlation_id
        
        # Add exception info if present
        if record.exc_info:
            log_entry["exception"] = self.formatException(record.exc_info)
        
        # Add extra fields
        for key, value in record.__dict__.items():
            if key not in ('name', 'msg', 'args', 'levelname', 'levelno', 'pathname', 
                          'filename', 'module', 'lineno', 'funcName', 'created', 'msecs', 
                          'relativeCreated', 'thread', 'threadName', 'processName', 
                          'process', 'getMessage', 'exc_info', 'exc_text', 'stack_info'):
                log_entry[key] = value
        
        return json.dumps(log_entry)


class TextFormatter(logging.Formatter):
    """Custom text formatter for human-readable logging."""
    
    def __init__(self):
        super().__init__(
            fmt='%(asctime)s [%(levelname)s] %(name)s: %(message)s',
            datefmt='%Y-%m-%d %H:%M:%S'
        )


def setup_logging() -> None:
    """Setup comprehensive logging configuration."""
    config = get_logging_config()
    
    # Clear existing handlers
    logging.root.handlers = []
    
    # Set root logger level
    logging.root.setLevel(getattr(logging, config.log_level))
    
    # Create formatters
    if config.log_format == "json":
        formatter = JSONFormatter()
    else:
        formatter = TextFormatter()
    
    # Console handler
    console_handler = logging.StreamHandler(sys.stdout)
    console_handler.setFormatter(formatter)
    logging.root.addHandler(console_handler)
    
    # File handler (if configured)
    if config.log_file:
        log_path = Path(config.log_file)
        log_path.parent.mkdir(parents=True, exist_ok=True)
        
        if config.log_rotation:
            # Parse log_max_size (e.g., "100MB" -> 100 * 1024 * 1024)
            max_size_str = config.log_max_size.upper()
            if max_size_str.endswith('MB'):
                max_bytes = int(max_size_str[:-2]) * 1024 * 1024
            elif max_size_str.endswith('KB'):
                max_bytes = int(max_size_str[:-2]) * 1024
            elif max_size_str.endswith('GB'):
                max_bytes = int(max_size_str[:-2]) * 1024 * 1024 * 1024
            else:
                max_bytes = int(max_size_str)
            
            file_handler = logging.handlers.RotatingFileHandler(
                filename=config.log_file,
                maxBytes=max_bytes,
                backupCount=config.log_backup_count,
                encoding='utf-8'
            )
        else:
            file_handler = logging.FileHandler(
                filename=config.log_file,
                encoding='utf-8'
            )
        
        file_handler.setFormatter(formatter)
        logging.root.addHandler(file_handler)
    
    # Silence noisy loggers
    logging.getLogger("uvicorn.access").setLevel(logging.WARNING)
    logging.getLogger("httpx").setLevel(logging.WARNING)
    logging.getLogger("asyncio").setLevel(logging.WARNING)
    
    # Enable SQL logging if configured
    if config.enable_sql_logging:
        logging.getLogger("sqlalchemy.engine").setLevel(logging.INFO)
        logging.getLogger("sqlalchemy.pool").setLevel(logging.INFO)
    else:
        logging.getLogger("sqlalchemy.engine").setLevel(logging.WARNING)
        logging.getLogger("sqlalchemy.pool").setLevel(logging.WARNING)


class BaseLogger:
    """Base class for specialized loggers."""
    
    def __init__(self, name: str):
        self.logger = logging.getLogger(name)
    
    def debug(self, message: str, **kwargs):
        extra = kwargs if kwargs else {}
        self.logger.debug(message, extra=extra)
    
    def info(self, message: str, **kwargs):
        extra = kwargs if kwargs else {}
        self.logger.info(message, extra=extra)
    
    def warning(self, message: str, **kwargs):
        extra = kwargs if kwargs else {}
        self.logger.warning(message, extra=extra)
    
    def error(self, message: str, **kwargs):
        extra = kwargs if kwargs else {}
        self.logger.error(message, extra=extra)
    
    def critical(self, message: str, **kwargs):
        extra = kwargs if kwargs else {}
        self.logger.critical(message, extra=extra)
    
    def exception(self, message: str, **kwargs):
        extra = kwargs if kwargs else {}
        self.logger.exception(message, extra=extra)


class RequestLogger(BaseLogger):
    """Specialized logger for HTTP requests."""
    
    def __init__(self):
        super().__init__("api_gateway.requests")
    
    def log_request(self, method: str, path: str, client_ip: str, 
                   user_agent: Optional[str] = None, **kwargs):
        """Log incoming HTTP request."""
        self.info(
            "HTTP request received",
            method=method,
            path=path,
            client_ip=client_ip,
            user_agent=user_agent or "",
            **kwargs
        )
    
    def log_response(self, method: str, path: str, status_code: int, 
                    duration_ms: float, **kwargs):
        """Log HTTP response."""
        self.info(
            "HTTP response sent",
            method=method,
            path=path,
            status_code=status_code,
            duration_ms=duration_ms,
            **kwargs
        )
    
    def log_error(self, method: str, path: str, status_code: int, 
                 error_message: str, **kwargs):
        """Log HTTP error response."""
        self.error(
            "HTTP error response",
            method=method,
            path=path,
            status_code=status_code,
            error_message=error_message,
            **kwargs
        )


class ServiceLogger(BaseLogger):
    """Specialized logger for service interactions."""
    
    def __init__(self):
        super().__init__("api_gateway.services")
    
    def log_service_call(self, service_name: str, method: str, url: str, **kwargs):
        """Log outgoing service call."""
        self.info(
            "Service call initiated",
            service_name=service_name,
            method=method,
            url=url,
            **kwargs
        )
    
    def log_service_response(self, service_name: str, method: str, url: str, 
                           status_code: int, duration_ms: float, **kwargs):
        """Log service response."""
        self.info(
            "Service response received",
            service_name=service_name,
            method=method,
            url=url,
            status_code=status_code,
            duration_ms=duration_ms,
            **kwargs
        )
    
    def log_service_error(self, service_name: str, method: str, url: str, 
                         error_message: str, **kwargs):
        """Log service error."""
        self.error(
            "Service call failed",
            service_name=service_name,
            method=method,
            url=url,
            error_message=error_message,
            **kwargs
        )
    
    def log_circuit_breaker_open(self, service_name: str, **kwargs):
        """Log circuit breaker opening."""
        self.warning(
            "Circuit breaker opened",
            service_name=service_name,
            **kwargs
        )
    
    def log_circuit_breaker_close(self, service_name: str, **kwargs):
        """Log circuit breaker closing."""
        self.info(
            "Circuit breaker closed",
            service_name=service_name,
            **kwargs
        )


class DatabaseLogger(BaseLogger):
    """Specialized logger for database operations."""
    
    def __init__(self):
        super().__init__("api_gateway.database")
    
    def log_query(self, query: str, params: Optional[Dict[str, Any]] = None, **kwargs):
        """Log database query."""
        self.debug(
            "Database query executed",
            query=query,
            params=params or {},
            **kwargs
        )
    
    def log_query_error(self, query: str, error_message: str, 
                       params: Optional[Dict[str, Any]] = None, **kwargs):
        """Log database query error."""
        self.error(
            "Database query failed",
            query=query,
            error_message=error_message,
            params=params or {},
            **kwargs
        )
    
    def log_connection_error(self, error_message: str, **kwargs):
        """Log database connection error."""
        self.error(
            "Database connection failed",
            error_message=error_message,
            **kwargs
        )
    
    def log_transaction_start(self, **kwargs):
        """Log transaction start."""
        self.debug("Database transaction started", **kwargs)
    
    def log_transaction_commit(self, **kwargs):
        """Log transaction commit."""
        self.debug("Database transaction committed", **kwargs)
    
    def log_transaction_rollback(self, error_message: Optional[str] = None, **kwargs):
        """Log transaction rollback."""
        self.warning(
            "Database transaction rolled back",
            error_message=error_message or "",
            **kwargs
        )


class SecurityLogger(BaseLogger):
    """Specialized logger for security events."""
    
    def __init__(self):
        super().__init__("api_gateway.security")
    
    def log_authentication_attempt(self, user_id: Optional[str] = None, 
                                  email: Optional[str] = None, 
                                  client_ip: Optional[str] = None, **kwargs):
        """Log authentication attempt."""
        self.info(
            "Authentication attempt",
            user_id=user_id or "",
            email=email or "",
            client_ip=client_ip or "",
            **kwargs
        )
    
    def log_authentication_success(self, user_id: str, 
                                  email: Optional[str] = None, 
                                  client_ip: Optional[str] = None, **kwargs):
        """Log successful authentication."""
        self.info(
            "Authentication successful",
            user_id=user_id,
            email=email or "",
            client_ip=client_ip or "",
            **kwargs
        )
    
    def log_authentication_failure(self, reason: str, 
                                  email: Optional[str] = None, 
                                  client_ip: Optional[str] = None, **kwargs):
        """Log authentication failure."""
        self.warning(
            "Authentication failed",
            reason=reason,
            email=email or "",
            client_ip=client_ip or "",
            **kwargs
        )
    
    def log_authorization_failure(self, user_id: str, resource: str, 
                                 action: str, client_ip: Optional[str] = None, **kwargs):
        """Log authorization failure."""
        self.warning(
            "Authorization failed",
            user_id=user_id,
            resource=resource,
            action=action,
            client_ip=client_ip or "",
            **kwargs
        )
    
    def log_rate_limit_exceeded(self, client_ip: str, endpoint: str, 
                               limit: int, **kwargs):
        """Log rate limit exceeded."""
        self.warning(
            "Rate limit exceeded",
            client_ip=client_ip,
            endpoint=endpoint,
            limit=limit,
            **kwargs
        )
    
    def log_suspicious_activity(self, activity_type: str, client_ip: str, 
                               details: str, **kwargs):
        """Log suspicious activity."""
        self.error(
            "Suspicious activity detected",
            activity_type=activity_type,
            client_ip=client_ip,
            details=details,
            **kwargs
        )


class PerformanceLogger(BaseLogger):
    """Specialized logger for performance metrics."""
    
    def __init__(self):
        super().__init__("api_gateway.performance")
    
    def log_slow_request(self, method: str, path: str, duration_ms: float, 
                        threshold_ms: float = 1000, **kwargs):
        """Log slow request."""
        self.warning(
            "Slow request detected",
            method=method,
            path=path,
            duration_ms=duration_ms,
            threshold_ms=threshold_ms,
            **kwargs
        )
    
    def log_memory_usage(self, memory_mb: float, **kwargs):
        """Log memory usage."""
        self.info(
            "Memory usage",
            memory_mb=memory_mb,
            **kwargs
        )
    
    def log_cpu_usage(self, cpu_percent: float, **kwargs):
        """Log CPU usage."""
        self.info(
            "CPU usage",
            cpu_percent=cpu_percent,
            **kwargs
        )


# Global logger instances
request_logger = RequestLogger()
service_logger = ServiceLogger()
database_logger = DatabaseLogger()
security_logger = SecurityLogger()
performance_logger = PerformanceLogger()


def get_logger(name: str) -> BaseLogger:
    """Get a logger instance by name."""
    return BaseLogger(name)


def get_request_logger() -> RequestLogger:
    """Get the request logger instance."""
    return request_logger


def get_service_logger() -> ServiceLogger:
    """Get the service logger instance."""
    return service_logger


def get_database_logger() -> DatabaseLogger:
    """Get the database logger instance."""
    return database_logger


def get_security_logger() -> SecurityLogger:
    """Get the security logger instance."""
    return security_logger


def get_performance_logger() -> PerformanceLogger:
    """Get the performance logger instance."""
    return performance_logger


# Export commonly used functions and classes
__all__ = [
    "setup_logging",
    "get_logger",
    "get_request_logger",
    "get_service_logger", 
    "get_database_logger",
    "get_security_logger",
    "get_performance_logger",
    "correlation_context",
    "get_correlation_id",
    "set_correlation_id",
    "generate_correlation_id",
    "BaseLogger",
    "RequestLogger",
    "ServiceLogger",
    "DatabaseLogger",
    "SecurityLogger",
    "PerformanceLogger"
]