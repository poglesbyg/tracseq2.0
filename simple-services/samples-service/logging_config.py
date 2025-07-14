"""
Centralized logging configuration for TracSeq 2.0 microservices
Provides structured JSON logging with request tracking and performance metrics
"""

import json
import logging
import sys
import time
from datetime import datetime
from typing import Any, Dict, Optional
from contextvars import ContextVar
from functools import wraps

import uvicorn
from fastapi import Request, Response
try:
    from fastapi.middleware.base import BaseHTTPMiddleware
except ImportError:
    # Fallback for older FastAPI versions
    from starlette.middleware.base import BaseHTTPMiddleware

# Context variables for request tracking
request_id_var: ContextVar[str] = ContextVar('request_id', default='')
service_name_var: ContextVar[str] = ContextVar('service_name', default='')

class JSONFormatter(logging.Formatter):
    """Custom JSON formatter for structured logging"""
    
    def format(self, record: logging.LogRecord) -> str:
        """Format log record as JSON"""
        
        # Base log structure
        log_entry = {
            "timestamp": datetime.utcnow().isoformat() + "Z",
            "level": record.levelname,
            "service": service_name_var.get() or "unknown",
            "logger": record.name,
            "message": record.getMessage(),
        }
        
        # Add request ID if available
        request_id = request_id_var.get()
        if request_id:
            log_entry["request_id"] = request_id
            
        # Add exception information if present
        if record.exc_info:
            log_entry["exception"] = {
                "type": record.exc_info[0].__name__ if record.exc_info[0] else None,
                "message": str(record.exc_info[1]) if record.exc_info[1] else None,
                "traceback": self.formatException(record.exc_info)
            }
            
        # Add extra fields from log record
        extra_fields = {
            k: v for k, v in record.__dict__.items()
            if k not in {
                'name', 'msg', 'args', 'levelname', 'levelno', 'pathname',
                'filename', 'module', 'lineno', 'funcName', 'created',
                'msecs', 'relativeCreated', 'thread', 'threadName',
                'processName', 'process', 'exc_info', 'exc_text', 'stack_info',
                'getMessage'
            }
        }
        
        if extra_fields:
            log_entry.update(extra_fields)
            
        return json.dumps(log_entry, default=str)

class RequestLoggingMiddleware(BaseHTTPMiddleware):
    """Middleware for logging HTTP requests and responses"""
    
    def __init__(self, app, service_name: str):
        super().__init__(app)
        self.service_name = service_name
        self.logger = logging.getLogger(f"{service_name}.requests")
        
    async def dispatch(self, request: Request, call_next):
        """Log request and response details"""
        
        # Generate request ID
        request_id = f"{self.service_name}-{int(time.time() * 1000)}-{id(request)}"
        request_id_var.set(request_id)
        service_name_var.set(self.service_name)
        
        # Start timing
        start_time = time.time()
        
        # Log incoming request
        self.logger.info(
            "Incoming request",
            extra={
                "http_method": request.method,
                "http_url": str(request.url),
                "http_path": request.url.path,
                "http_query_params": dict(request.query_params),
                "http_headers": dict(request.headers),
                "client_ip": request.client.host if request.client else None,
                "user_agent": request.headers.get("user-agent"),
                "request_id": request_id
            }
        )
        
        # Process request
        try:
            response = await call_next(request)
            
            # Calculate processing time
            processing_time = time.time() - start_time
            
            # Log response
            self.logger.info(
                "Request completed",
                extra={
                    "http_status_code": response.status_code,
                    "processing_time_ms": round(processing_time * 1000, 2),
                    "response_size_bytes": response.headers.get("content-length"),
                    "request_id": request_id
                }
            )
            
            return response
            
        except Exception as e:
            # Calculate processing time for failed requests
            processing_time = time.time() - start_time
            
            # Log error
            self.logger.error(
                "Request failed",
                extra={
                    "error_type": type(e).__name__,
                    "error_message": str(e),
                    "processing_time_ms": round(processing_time * 1000, 2),
                    "request_id": request_id
                },
                exc_info=True
            )
            
            raise

def setup_logging(service_name: str, log_level: str = "INFO") -> logging.Logger:
    """
    Setup structured logging for a service
    
    Args:
        service_name: Name of the service (e.g., "dashboard-service")
        log_level: Logging level (DEBUG, INFO, WARNING, ERROR, CRITICAL)
        
    Returns:
        Configured logger instance
    """
    
    # Set service name in context
    service_name_var.set(service_name)
    
    # Create logger
    logger = logging.getLogger(service_name)
    logger.setLevel(getattr(logging, log_level.upper()))
    
    # Remove existing handlers
    logger.handlers.clear()
    
    # Create console handler with JSON formatter
    console_handler = logging.StreamHandler(sys.stdout)
    console_handler.setFormatter(JSONFormatter())
    logger.addHandler(console_handler)
    
    # Prevent duplicate logs
    logger.propagate = False
    
    # Configure uvicorn logging
    uvicorn_logger = logging.getLogger("uvicorn")
    uvicorn_logger.handlers.clear()
    uvicorn_logger.addHandler(console_handler)
    uvicorn_logger.setLevel(getattr(logging, log_level.upper()))
    
    uvicorn_access_logger = logging.getLogger("uvicorn.access")
    uvicorn_access_logger.handlers.clear()
    uvicorn_access_logger.disabled = True  # We handle access logging in middleware
    
    return logger

def log_performance(operation_name: str):
    """Decorator for logging performance metrics of operations"""
    
    def decorator(func):
        @wraps(func)
        async def async_wrapper(*args, **kwargs):
            logger = logging.getLogger(f"{service_name_var.get()}.performance")
            start_time = time.time()
            
            try:
                result = await func(*args, **kwargs)
                duration = time.time() - start_time
                
                logger.info(
                    f"Operation completed: {operation_name}",
                    extra={
                        "operation": operation_name,
                        "duration_ms": round(duration * 1000, 2),
                        "success": True
                    }
                )
                
                return result
                
            except Exception as e:
                duration = time.time() - start_time
                
                logger.error(
                    f"Operation failed: {operation_name}",
                    extra={
                        "operation": operation_name,
                        "duration_ms": round(duration * 1000, 2),
                        "success": False,
                        "error_type": type(e).__name__,
                        "error_message": str(e)
                    },
                    exc_info=True
                )
                
                raise
                
        @wraps(func)
        def sync_wrapper(*args, **kwargs):
            logger = logging.getLogger(f"{service_name_var.get()}.performance")
            start_time = time.time()
            
            try:
                result = func(*args, **kwargs)
                duration = time.time() - start_time
                
                logger.info(
                    f"Operation completed: {operation_name}",
                    extra={
                        "operation": operation_name,
                        "duration_ms": round(duration * 1000, 2),
                        "success": True
                    }
                )
                
                return result
                
            except Exception as e:
                duration = time.time() - start_time
                
                logger.error(
                    f"Operation failed: {operation_name}",
                    extra={
                        "operation": operation_name,
                        "duration_ms": round(duration * 1000, 2),
                        "success": False,
                        "error_type": type(e).__name__,
                        "error_message": str(e)
                    },
                    exc_info=True
                )
                
                raise
                
        # Return appropriate wrapper based on function type
        import asyncio
        if asyncio.iscoroutinefunction(func):
            return async_wrapper
        else:
            return sync_wrapper
            
    return decorator

def log_database_operation(operation_type: str, table_name: str, record_id: Optional[str] = None):
    """Log database operations for audit trail"""
    
    logger = logging.getLogger(f"{service_name_var.get()}.database")
    
    logger.info(
        f"Database operation: {operation_type}",
        extra={
            "operation_type": operation_type,
            "table_name": table_name,
            "record_id": record_id,
            "request_id": request_id_var.get()
        }
    )

def log_business_event(event_type: str, details: Dict[str, Any]):
    """Log business events for analytics and monitoring"""
    
    logger = logging.getLogger(f"{service_name_var.get()}.business")
    
    logger.info(
        f"Business event: {event_type}",
        extra={
            "event_type": event_type,
            "event_details": details,
            "request_id": request_id_var.get()
        }
    )

def get_logger(name: str) -> logging.Logger:
    """Get a logger instance with service context"""
    
    service_name = service_name_var.get()
    if service_name:
        return logging.getLogger(f"{service_name}.{name}")
    else:
        return logging.getLogger(name)

# Health check logging utility
def log_health_check(service_name: str, status: str, details: Optional[Dict[str, Any]] = None):
    """Log health check results"""
    
    logger = logging.getLogger(f"{service_name}.health")
    
    logger.info(
        f"Health check: {status}",
        extra={
            "health_status": status,
            "health_details": details or {},
            "service_name": service_name
        }
    )

# Example usage and configuration
LOGGING_CONFIG = {
    "version": 1,
    "disable_existing_loggers": False,
    "formatters": {
        "json": {
            "()": JSONFormatter,
        },
    },
    "handlers": {
        "console": {
            "class": "logging.StreamHandler",
            "formatter": "json",
            "stream": "ext://sys.stdout",
        },
    },
    "root": {
        "level": "INFO",
        "handlers": ["console"],
    },
    "loggers": {
        "uvicorn": {
            "level": "INFO",
            "handlers": ["console"],
            "propagate": False,
        },
        "uvicorn.access": {
            "level": "INFO",
            "handlers": [],
            "propagate": False,
        },
    },
} 