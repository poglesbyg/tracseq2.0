#!/usr/bin/env python3
"""
Enhanced Logging System for TracSeq 2.0 API Gateway
Centralized logging configuration with structured logging support
"""

import sys
import json
import logging
import logging.handlers
from datetime import datetime
from typing import Dict, Any, Optional
from pathlib import Path
from functools import wraps

from .config import get_config


class StructuredFormatter(logging.Formatter):
    """Custom formatter for structured JSON logging"""
    
    def __init__(self, include_extra: bool = True):
        super().__init__()
        self.include_extra = include_extra
    
    def format(self, record: logging.LogRecord) -> str:
        """Format log record as structured JSON"""
        log_entry = {
            "timestamp": datetime.fromtimestamp(record.created).isoformat(),
            "level": record.levelname,
            "logger": record.name,
            "message": record.getMessage(),
            "module": record.module,
            "function": record.funcName,
            "line": record.lineno,
        }
        
        # Add exception information if present
        if record.exc_info:
            log_entry["exception"] = self.formatException(record.exc_info)
        
        # Add extra fields if enabled
        if self.include_extra:
            extra_fields = {}
            for key, value in record.__dict__.items():
                if key not in {
                    'name', 'msg', 'args', 'levelname', 'levelno', 'pathname', 
                    'filename', 'module', 'exc_info', 'exc_text', 'stack_info',
                    'lineno', 'funcName', 'created', 'msecs', 'relativeCreated',
                    'thread', 'threadName', 'processName', 'process', 'message'
                }:
                    extra_fields[key] = value
            
            if extra_fields:
                log_entry["extra"] = extra_fields
        
        return json.dumps(log_entry, default=str)


class RequestLogger:
    """Logger for HTTP requests and responses"""
    
    def __init__(self, logger_name: str = "api_gateway.requests"):
        self.logger = logging.getLogger(logger_name)
    
    def log_request(self, request, response_status: int, response_time: float, **kwargs):
        """Log HTTP request details"""
        self.logger.info(
            "HTTP Request",
            extra={
                "request_method": request.method,
                "request_url": str(request.url),
                "request_headers": dict(request.headers),
                "response_status": response_status,
                "response_time_ms": round(response_time * 1000, 2),
                "client_ip": getattr(request, 'client', {}).get('host', 'unknown'),
                **kwargs
            }
        )
    
    def log_error(self, request, error: Exception, **kwargs):
        """Log HTTP request errors"""
        self.logger.error(
            f"HTTP Request Error: {str(error)}",
            extra={
                "request_method": request.method,
                "request_url": str(request.url),
                "error_type": type(error).__name__,
                "error_message": str(error),
                **kwargs
            },
            exc_info=True
        )


class ServiceLogger:
    """Logger for microservice interactions"""
    
    def __init__(self, logger_name: str = "api_gateway.services"):
        self.logger = logging.getLogger(logger_name)
    
    def log_service_call(self, service_name: str, method: str, url: str, 
                        status_code: int, response_time: float, **kwargs):
        """Log service call details"""
        self.logger.info(
            f"Service Call: {service_name}",
            extra={
                "service_name": service_name,
                "method": method,
                "url": url,
                "status_code": status_code,
                "response_time_ms": round(response_time * 1000, 2),
                **kwargs
            }
        )
    
    def log_service_error(self, service_name: str, error: Exception, **kwargs):
        """Log service call errors"""
        self.logger.error(
            f"Service Error: {service_name} - {str(error)}",
            extra={
                "service_name": service_name,
                "error_type": type(error).__name__,
                "error_message": str(error),
                **kwargs
            },
            exc_info=True
        )
    
    def log_service_health(self, service_name: str, is_healthy: bool, **kwargs):
        """Log service health status"""
        level = logging.INFO if is_healthy else logging.WARNING
        self.logger.log(
            level,
            f"Service Health: {service_name} - {'Healthy' if is_healthy else 'Unhealthy'}",
            extra={
                "service_name": service_name,
                "is_healthy": is_healthy,
                **kwargs
            }
        )


class DatabaseLogger:
    """Logger for database operations"""
    
    def __init__(self, logger_name: str = "api_gateway.database"):
        self.logger = logging.getLogger(logger_name)
    
    def log_query(self, query: str, params: tuple = None, execution_time: float = None, **kwargs):
        """Log database query"""
        if get_config().logging.enable_sql_logging:
            self.logger.debug(
                "Database Query",
                extra={
                    "query": query,
                    "params": params,
                    "execution_time_ms": round(execution_time * 1000, 2) if execution_time else None,
                    **kwargs
                }
            )
    
    def log_connection_event(self, event_type: str, **kwargs):
        """Log database connection events"""
        self.logger.info(
            f"Database Connection: {event_type}",
            extra={
                "event_type": event_type,
                **kwargs
            }
        )
    
    def log_error(self, error: Exception, query: str = None, **kwargs):
        """Log database errors"""
        self.logger.error(
            f"Database Error: {str(error)}",
            extra={
                "error_type": type(error).__name__,
                "error_message": str(error),
                "query": query,
                **kwargs
            },
            exc_info=True
        )


class SecurityLogger:
    """Logger for security events"""
    
    def __init__(self, logger_name: str = "api_gateway.security"):
        self.logger = logging.getLogger(logger_name)
    
    def log_auth_attempt(self, user_id: str, success: bool, ip_address: str = None, **kwargs):
        """Log authentication attempts"""
        level = logging.INFO if success else logging.WARNING
        self.logger.log(
            level,
            f"Authentication {'Success' if success else 'Failed'}: {user_id}",
            extra={
                "user_id": user_id,
                "auth_success": success,
                "ip_address": ip_address,
                **kwargs
            }
        )
    
    def log_authorization_failure(self, user_id: str, resource: str, action: str, **kwargs):
        """Log authorization failures"""
        self.logger.warning(
            f"Authorization Failed: {user_id} -> {action} on {resource}",
            extra={
                "user_id": user_id,
                "resource": resource,
                "action": action,
                **kwargs
            }
        )
    
    def log_security_event(self, event_type: str, severity: str, description: str, **kwargs):
        """Log general security events"""
        level_map = {
            "low": logging.INFO,
            "medium": logging.WARNING,
            "high": logging.ERROR,
            "critical": logging.CRITICAL
        }
        level = level_map.get(severity.lower(), logging.INFO)
        
        self.logger.log(
            level,
            f"Security Event: {event_type} - {description}",
            extra={
                "event_type": event_type,
                "severity": severity,
                "description": description,
                **kwargs
            }
        )


def setup_logging() -> Dict[str, logging.Logger]:
    """Setup and configure logging for the application"""
    config = get_config()
    
    # Set root logger level
    root_logger = logging.getLogger()
    root_logger.setLevel(getattr(logging, config.logging.log_level.upper()))
    
    # Clear existing handlers
    root_logger.handlers.clear()
    
    # Create formatters
    if config.is_production:
        formatter = StructuredFormatter()
    else:
        formatter = logging.Formatter(
            fmt=config.logging.log_format,
            datefmt="%Y-%m-%d %H:%M:%S"
        )
    
    # Console handler
    console_handler = logging.StreamHandler(sys.stdout)
    console_handler.setFormatter(formatter)
    root_logger.addHandler(console_handler)
    
    # File handler (if configured)
    if config.logging.log_file:
        log_file_path = Path(config.logging.log_file)
        log_file_path.parent.mkdir(parents=True, exist_ok=True)
        
        file_handler = logging.handlers.RotatingFileHandler(
            filename=log_file_path,
            maxBytes=10 * 1024 * 1024,  # 10MB
            backupCount=5
        )
        file_handler.setFormatter(formatter)
        root_logger.addHandler(file_handler)
    
    # Create specialized loggers
    loggers = {
        "main": logging.getLogger("api_gateway.main"),
        "requests": RequestLogger(),
        "services": ServiceLogger(),
        "database": DatabaseLogger(),
        "security": SecurityLogger(),
    }
    
    return loggers


def log_function_call(logger: logging.Logger = None):
    """Decorator to log function calls"""
    def decorator(func):
        @wraps(func)
        async def async_wrapper(*args, **kwargs):
            func_logger = logger or logging.getLogger(f"api_gateway.{func.__module__}")
            func_logger.debug(
                f"Function Call: {func.__name__}",
                extra={
                    "function": func.__name__,
                    "module": func.__module__,
                    "args_count": len(args),
                    "kwargs_keys": list(kwargs.keys())
                }
            )
            try:
                result = await func(*args, **kwargs)
                func_logger.debug(
                    f"Function Success: {func.__name__}",
                    extra={
                        "function": func.__name__,
                        "result_type": type(result).__name__
                    }
                )
                return result
            except Exception as e:
                func_logger.error(
                    f"Function Error: {func.__name__} - {str(e)}",
                    extra={
                        "function": func.__name__,
                        "error_type": type(e).__name__,
                        "error_message": str(e)
                    },
                    exc_info=True
                )
                raise
        
        @wraps(func)
        def sync_wrapper(*args, **kwargs):
            func_logger = logger or logging.getLogger(f"api_gateway.{func.__module__}")
            func_logger.debug(
                f"Function Call: {func.__name__}",
                extra={
                    "function": func.__name__,
                    "module": func.__module__,
                    "args_count": len(args),
                    "kwargs_keys": list(kwargs.keys())
                }
            )
            try:
                result = func(*args, **kwargs)
                func_logger.debug(
                    f"Function Success: {func.__name__}",
                    extra={
                        "function": func.__name__,
                        "result_type": type(result).__name__
                    }
                )
                return result
            except Exception as e:
                func_logger.error(
                    f"Function Error: {func.__name__} - {str(e)}",
                    extra={
                        "function": func.__name__,
                        "error_type": type(e).__name__,
                        "error_message": str(e)
                    },
                    exc_info=True
                )
                raise
        
        return async_wrapper if hasattr(func, '__code__') and func.__code__.co_flags & 0x80 else sync_wrapper
    
    return decorator


# Global logger instances
loggers = setup_logging()
main_logger = loggers["main"]
request_logger = loggers["requests"]
service_logger = loggers["services"]
database_logger = loggers["database"]
security_logger = loggers["security"]