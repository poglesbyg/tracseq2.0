#!/usr/bin/env python3
"""
Enhanced Exception Handling for TracSeq 2.0 API Gateway
Centralized error handling with custom exceptions and error responses
"""

import traceback
from typing import Dict, Any, Optional, List, Union
from datetime import datetime
from fastapi import HTTPException, Request, Response
from fastapi.responses import JSONResponse
from fastapi.exceptions import RequestValidationError
from starlette.exceptions import HTTPException as StarletteHTTPException
import logging

from .logging import main_logger, security_logger


class TracSeqException(Exception):
    """Base exception for TracSeq API Gateway"""
    
    def __init__(self, message: str, error_code: str = None, details: Dict[str, Any] = None):
        super().__init__(message)
        self.message = message
        self.error_code = error_code or self.__class__.__name__
        self.details = details or {}
        self.timestamp = datetime.utcnow()
    
    def to_dict(self) -> Dict[str, Any]:
        """Convert exception to dictionary"""
        return {
            "error_code": self.error_code,
            "message": self.message,
            "details": self.details,
            "timestamp": self.timestamp.isoformat(),
        }


class ServiceException(TracSeqException):
    """Exception for microservice-related errors"""
    
    def __init__(self, service_name: str, message: str, status_code: int = 502, 
                 original_error: Exception = None):
        super().__init__(message, f"SERVICE_{service_name.upper()}_ERROR")
        self.service_name = service_name
        self.status_code = status_code
        self.original_error = original_error
        self.details.update({
            "service_name": service_name,
            "status_code": status_code,
            "original_error": str(original_error) if original_error else None
        })


class DatabaseException(TracSeqException):
    """Exception for database-related errors"""
    
    def __init__(self, message: str, query: str = None, original_error: Exception = None):
        super().__init__(message, "DATABASE_ERROR")
        self.query = query
        self.original_error = original_error
        self.details.update({
            "query": query,
            "original_error": str(original_error) if original_error else None
        })


class AuthenticationException(TracSeqException):
    """Exception for authentication-related errors"""
    
    def __init__(self, message: str = "Authentication failed", user_id: str = None):
        super().__init__(message, "AUTHENTICATION_ERROR")
        self.user_id = user_id
        self.details.update({
            "user_id": user_id
        })


class AuthorizationException(TracSeqException):
    """Exception for authorization-related errors"""
    
    def __init__(self, message: str = "Access denied", user_id: str = None, 
                 resource: str = None, action: str = None):
        super().__init__(message, "AUTHORIZATION_ERROR")
        self.user_id = user_id
        self.resource = resource
        self.action = action
        self.details.update({
            "user_id": user_id,
            "resource": resource,
            "action": action
        })


class ValidationException(TracSeqException):
    """Exception for validation errors"""
    
    def __init__(self, message: str, field_errors: List[Dict[str, Any]] = None):
        super().__init__(message, "VALIDATION_ERROR")
        self.field_errors = field_errors or []
        self.details.update({
            "field_errors": self.field_errors
        })


class RateLimitException(TracSeqException):
    """Exception for rate limiting errors"""
    
    def __init__(self, message: str = "Rate limit exceeded", limit: int = None, 
                 window: int = None, retry_after: int = None):
        super().__init__(message, "RATE_LIMIT_EXCEEDED")
        self.limit = limit
        self.window = window
        self.retry_after = retry_after
        self.details.update({
            "limit": limit,
            "window": window,
            "retry_after": retry_after
        })


class FileProcessingException(TracSeqException):
    """Exception for file processing errors"""
    
    def __init__(self, message: str, filename: str = None, file_type: str = None):
        super().__init__(message, "FILE_PROCESSING_ERROR")
        self.filename = filename
        self.file_type = file_type
        self.details.update({
            "filename": filename,
            "file_type": file_type
        })


class CircuitBreakerException(TracSeqException):
    """Exception for circuit breaker errors"""
    
    def __init__(self, service_name: str, message: str = "Service circuit breaker is open"):
        super().__init__(message, "CIRCUIT_BREAKER_OPEN")
        self.service_name = service_name
        self.details.update({
            "service_name": service_name
        })


class ConfigurationException(TracSeqException):
    """Exception for configuration errors"""
    
    def __init__(self, message: str, config_key: str = None):
        super().__init__(message, "CONFIGURATION_ERROR")
        self.config_key = config_key
        self.details.update({
            "config_key": config_key
        })


class ErrorHandler:
    """Centralized error handler for the API Gateway"""
    
    def __init__(self):
        self.logger = main_logger
        self.security_logger = security_logger
    
    def create_error_response(self, 
                            status_code: int,
                            message: str,
                            error_code: str = None,
                            details: Dict[str, Any] = None,
                            request_id: str = None) -> JSONResponse:
        """Create standardized error response"""
        
        error_response = {
            "success": False,
            "error": {
                "code": error_code or f"HTTP_{status_code}",
                "message": message,
                "timestamp": datetime.utcnow().isoformat(),
            }
        }
        
        if details:
            error_response["error"]["details"] = details
        
        if request_id:
            error_response["error"]["request_id"] = request_id
        
        return JSONResponse(
            status_code=status_code,
            content=error_response
        )
    
    def handle_tracseq_exception(self, exc: TracSeqException, request: Request = None) -> JSONResponse:
        """Handle TracSeq custom exceptions"""
        
        # Determine status code based on exception type
        status_code_map = {
            AuthenticationException: 401,
            AuthorizationException: 403,
            ValidationException: 400,
            RateLimitException: 429,
            FileProcessingException: 400,
            CircuitBreakerException: 503,
            ConfigurationException: 500,
            ServiceException: 502,
            DatabaseException: 500,
        }
        
        status_code = status_code_map.get(type(exc), 500)
        
        # Log the exception
        self.logger.error(
            f"TracSeq Exception: {exc.error_code} - {exc.message}",
            extra={
                "error_code": exc.error_code,
                "exception_type": type(exc).__name__,
                "details": exc.details,
                "request_url": str(request.url) if request else None,
                "request_method": request.method if request else None,
            },
            exc_info=True
        )
        
        # Log security events for auth-related exceptions
        if isinstance(exc, (AuthenticationException, AuthorizationException)):
            self.security_logger.log_security_event(
                event_type=exc.error_code,
                severity="medium",
                description=exc.message,
                user_id=exc.details.get("user_id"),
                request_url=str(request.url) if request else None
            )
        
        return self.create_error_response(
            status_code=status_code,
            message=exc.message,
            error_code=exc.error_code,
            details=exc.details,
            request_id=getattr(request.state, 'request_id', None) if request else None
        )
    
    def handle_http_exception(self, exc: HTTPException, request: Request = None) -> JSONResponse:
        """Handle FastAPI HTTP exceptions"""
        
        self.logger.warning(
            f"HTTP Exception: {exc.status_code} - {exc.detail}",
            extra={
                "status_code": exc.status_code,
                "detail": exc.detail,
                "request_url": str(request.url) if request else None,
                "request_method": request.method if request else None,
            }
        )
        
        return self.create_error_response(
            status_code=exc.status_code,
            message=exc.detail,
            error_code=f"HTTP_{exc.status_code}",
            request_id=getattr(request.state, 'request_id', None) if request else None
        )
    
    def handle_validation_exception(self, exc: RequestValidationError, request: Request = None) -> JSONResponse:
        """Handle FastAPI validation exceptions"""
        
        field_errors = []
        for error in exc.errors():
            field_errors.append({
                "field": ".".join(str(x) for x in error.get("loc", [])),
                "message": error.get("msg", "Validation error"),
                "type": error.get("type", "validation_error"),
                "input": error.get("input"),
            })
        
        self.logger.warning(
            f"Validation Error: {len(field_errors)} field(s) failed validation",
            extra={
                "field_errors": field_errors,
                "request_url": str(request.url) if request else None,
                "request_method": request.method if request else None,
                "request_body": getattr(exc, 'body', None),
            }
        )
        
        return self.create_error_response(
            status_code=400,
            message="Request validation failed",
            error_code="VALIDATION_ERROR",
            details={"field_errors": field_errors},
            request_id=getattr(request.state, 'request_id', None) if request else None
        )
    
    def handle_general_exception(self, exc: Exception, request: Request = None) -> JSONResponse:
        """Handle general unhandled exceptions"""
        
        # Get traceback for debugging
        tb_str = traceback.format_exc()
        
        self.logger.error(
            f"Unhandled Exception: {type(exc).__name__} - {str(exc)}",
            extra={
                "exception_type": type(exc).__name__,
                "exception_message": str(exc),
                "traceback": tb_str,
                "request_url": str(request.url) if request else None,
                "request_method": request.method if request else None,
            },
            exc_info=True
        )
        
        # Don't expose internal error details in production
        from .config import get_config
        config = get_config()
        
        if config.is_production:
            message = "An internal server error occurred"
            details = None
        else:
            message = f"{type(exc).__name__}: {str(exc)}"
            details = {"traceback": tb_str}
        
        return self.create_error_response(
            status_code=500,
            message=message,
            error_code="INTERNAL_SERVER_ERROR",
            details=details,
            request_id=getattr(request.state, 'request_id', None) if request else None
        )


# Global error handler instance
error_handler = ErrorHandler()


def setup_exception_handlers(app):
    """Setup exception handlers for FastAPI app"""
    
    @app.exception_handler(TracSeqException)
    async def tracseq_exception_handler(request: Request, exc: TracSeqException):
        return error_handler.handle_tracseq_exception(exc, request)
    
    @app.exception_handler(HTTPException)
    async def http_exception_handler(request: Request, exc: HTTPException):
        return error_handler.handle_http_exception(exc, request)
    
    @app.exception_handler(StarletteHTTPException)
    async def starlette_http_exception_handler(request: Request, exc: StarletteHTTPException):
        return error_handler.handle_http_exception(
            HTTPException(status_code=exc.status_code, detail=exc.detail), 
            request
        )
    
    @app.exception_handler(RequestValidationError)
    async def validation_exception_handler(request: Request, exc: RequestValidationError):
        return error_handler.handle_validation_exception(exc, request)
    
    @app.exception_handler(Exception)
    async def general_exception_handler(request: Request, exc: Exception):
        return error_handler.handle_general_exception(exc, request)


# Utility functions for raising common exceptions
def raise_service_error(service_name: str, message: str, status_code: int = 502, 
                       original_error: Exception = None) -> None:
    """Raise a service exception"""
    raise ServiceException(service_name, message, status_code, original_error)


def raise_database_error(message: str, query: str = None, original_error: Exception = None) -> None:
    """Raise a database exception"""
    raise DatabaseException(message, query, original_error)


def raise_auth_error(message: str = "Authentication failed", user_id: str = None) -> None:
    """Raise an authentication exception"""
    raise AuthenticationException(message, user_id)


def raise_authorization_error(message: str = "Access denied", user_id: str = None, 
                            resource: str = None, action: str = None) -> None:
    """Raise an authorization exception"""
    raise AuthorizationException(message, user_id, resource, action)


def raise_validation_error(message: str, field_errors: List[Dict[str, Any]] = None) -> None:
    """Raise a validation exception"""
    raise ValidationException(message, field_errors)


def raise_rate_limit_error(message: str = "Rate limit exceeded", limit: int = None, 
                          window: int = None, retry_after: int = None) -> None:
    """Raise a rate limit exception"""
    raise RateLimitException(message, limit, window, retry_after)


def raise_file_processing_error(message: str, filename: str = None, file_type: str = None) -> None:
    """Raise a file processing exception"""
    raise FileProcessingException(message, filename, file_type)


def raise_circuit_breaker_error(service_name: str, 
                               message: str = "Service circuit breaker is open") -> None:
    """Raise a circuit breaker exception"""
    raise CircuitBreakerException(service_name, message)


def raise_config_error(message: str, config_key: str = None) -> None:
    """Raise a configuration exception"""
    raise ConfigurationException(message, config_key)