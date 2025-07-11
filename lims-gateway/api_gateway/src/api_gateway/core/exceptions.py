"""
Custom exception hierarchy for the TracSeq 2.0 API Gateway.

This module provides a comprehensive set of custom exceptions with standardized
error responses, proper HTTP status codes, and detailed error information.
"""

import traceback
from typing import Dict, Any, Optional, List
from fastapi import HTTPException, Request
from fastapi.responses import JSONResponse
from pydantic import BaseModel


class ErrorDetail(BaseModel):
    """Detailed error information."""
    code: str
    message: str
    field: Optional[str] = None
    context: Optional[Dict[str, Any]] = None


class ErrorResponse(BaseModel):
    """Standardized error response format."""
    success: bool = False
    error: str
    message: str
    details: Optional[List[ErrorDetail]] = None
    correlation_id: Optional[str] = None
    timestamp: str
    path: Optional[str] = None
    method: Optional[str] = None


class BaseGatewayException(Exception):
    """Base exception for all gateway-specific exceptions."""
    
    def __init__(
        self,
        message: str,
        error_code: str = "GATEWAY_ERROR",
        status_code: int = 500,
        details: Optional[List[ErrorDetail]] = None,
        context: Optional[Dict[str, Any]] = None
    ):
        super().__init__(message)
        self.message = message
        self.error_code = error_code
        self.status_code = status_code
        self.details = details or []
        self.context = context or {}
    
    def to_dict(self) -> Dict[str, Any]:
        """Convert exception to dictionary format."""
        return {
            "error": self.error_code,
            "message": self.message,
            "status_code": self.status_code,
            "details": [detail.dict() for detail in self.details],
            "context": self.context
        }


class ValidationException(BaseGatewayException):
    """Exception for validation errors."""
    
    def __init__(
        self,
        message: str = "Validation failed",
        field: Optional[str] = None,
        details: Optional[List[ErrorDetail]] = None,
        context: Optional[Dict[str, Any]] = None
    ):
        super().__init__(
            message=message,
            error_code="VALIDATION_ERROR",
            status_code=400,
            details=details,
            context=context
        )
        self.field = field


class AuthenticationException(BaseGatewayException):
    """Exception for authentication errors."""
    
    def __init__(
        self,
        message: str = "Authentication failed",
        details: Optional[List[ErrorDetail]] = None,
        context: Optional[Dict[str, Any]] = None
    ):
        super().__init__(
            message=message,
            error_code="AUTHENTICATION_ERROR",
            status_code=401,
            details=details,
            context=context
        )


class AuthorizationException(BaseGatewayException):
    """Exception for authorization errors."""
    
    def __init__(
        self,
        message: str = "Access denied",
        resource: Optional[str] = None,
        action: Optional[str] = None,
        details: Optional[List[ErrorDetail]] = None,
        context: Optional[Dict[str, Any]] = None
    ):
        super().__init__(
            message=message,
            error_code="AUTHORIZATION_ERROR",
            status_code=403,
            details=details,
            context=context
        )
        self.resource = resource
        self.action = action


class ResourceNotFoundException(BaseGatewayException):
    """Exception for resource not found errors."""
    
    def __init__(
        self,
        message: str = "Resource not found",
        resource_type: Optional[str] = None,
        resource_id: Optional[str] = None,
        details: Optional[List[ErrorDetail]] = None,
        context: Optional[Dict[str, Any]] = None
    ):
        super().__init__(
            message=message,
            error_code="RESOURCE_NOT_FOUND",
            status_code=404,
            details=details,
            context=context
        )
        self.resource_type = resource_type
        self.resource_id = resource_id


class ConflictException(BaseGatewayException):
    """Exception for resource conflict errors."""
    
    def __init__(
        self,
        message: str = "Resource conflict",
        resource_type: Optional[str] = None,
        conflict_field: Optional[str] = None,
        details: Optional[List[ErrorDetail]] = None,
        context: Optional[Dict[str, Any]] = None
    ):
        super().__init__(
            message=message,
            error_code="RESOURCE_CONFLICT",
            status_code=409,
            details=details,
            context=context
        )
        self.resource_type = resource_type
        self.conflict_field = conflict_field


class RateLimitException(BaseGatewayException):
    """Exception for rate limit exceeded errors."""
    
    def __init__(
        self,
        message: str = "Rate limit exceeded",
        limit: Optional[int] = None,
        window: Optional[int] = None,
        retry_after: Optional[int] = None,
        details: Optional[List[ErrorDetail]] = None,
        context: Optional[Dict[str, Any]] = None
    ):
        super().__init__(
            message=message,
            error_code="RATE_LIMIT_EXCEEDED",
            status_code=429,
            details=details,
            context=context
        )
        self.limit = limit
        self.window = window
        self.retry_after = retry_after


class ServiceUnavailableException(BaseGatewayException):
    """Exception for service unavailable errors."""
    
    def __init__(
        self,
        message: str = "Service temporarily unavailable",
        service_name: Optional[str] = None,
        retry_after: Optional[int] = None,
        details: Optional[List[ErrorDetail]] = None,
        context: Optional[Dict[str, Any]] = None
    ):
        super().__init__(
            message=message,
            error_code="SERVICE_UNAVAILABLE",
            status_code=503,
            details=details,
            context=context
        )
        self.service_name = service_name
        self.retry_after = retry_after


class DatabaseException(BaseGatewayException):
    """Exception for database-related errors."""
    
    def __init__(
        self,
        message: str = "Database error",
        operation: Optional[str] = None,
        table: Optional[str] = None,
        details: Optional[List[ErrorDetail]] = None,
        context: Optional[Dict[str, Any]] = None
    ):
        super().__init__(
            message=message,
            error_code="DATABASE_ERROR",
            status_code=500,
            details=details,
            context=context
        )
        self.operation = operation
        self.table = table


class ExternalServiceException(BaseGatewayException):
    """Exception for external service errors."""
    
    def __init__(
        self,
        message: str = "External service error",
        service_name: Optional[str] = None,
        service_url: Optional[str] = None,
        status_code: int = 502,
        details: Optional[List[ErrorDetail]] = None,
        context: Optional[Dict[str, Any]] = None
    ):
        super().__init__(
            message=message,
            error_code="EXTERNAL_SERVICE_ERROR",
            status_code=status_code,
            details=details,
            context=context
        )
        self.service_name = service_name
        self.service_url = service_url


class CircuitBreakerException(BaseGatewayException):
    """Exception for circuit breaker open state."""
    
    def __init__(
        self,
        message: str = "Circuit breaker is open",
        service_name: Optional[str] = None,
        details: Optional[List[ErrorDetail]] = None,
        context: Optional[Dict[str, Any]] = None
    ):
        super().__init__(
            message=message,
            error_code="CIRCUIT_BREAKER_OPEN",
            status_code=503,
            details=details,
            context=context
        )
        self.service_name = service_name


class TimeoutException(BaseGatewayException):
    """Exception for timeout errors."""
    
    def __init__(
        self,
        message: str = "Request timeout",
        timeout_seconds: Optional[float] = None,
        operation: Optional[str] = None,
        details: Optional[List[ErrorDetail]] = None,
        context: Optional[Dict[str, Any]] = None
    ):
        super().__init__(
            message=message,
            error_code="TIMEOUT_ERROR",
            status_code=408,
            details=details,
            context=context
        )
        self.timeout_seconds = timeout_seconds
        self.operation = operation


class ConfigurationException(BaseGatewayException):
    """Exception for configuration errors."""
    
    def __init__(
        self,
        message: str = "Configuration error",
        config_key: Optional[str] = None,
        details: Optional[List[ErrorDetail]] = None,
        context: Optional[Dict[str, Any]] = None
    ):
        super().__init__(
            message=message,
            error_code="CONFIGURATION_ERROR",
            status_code=500,
            details=details,
            context=context
        )
        self.config_key = config_key


class BusinessLogicException(BaseGatewayException):
    """Exception for business logic violations."""
    
    def __init__(
        self,
        message: str = "Business logic violation",
        rule: Optional[str] = None,
        details: Optional[List[ErrorDetail]] = None,
        context: Optional[Dict[str, Any]] = None
    ):
        super().__init__(
            message=message,
            error_code="BUSINESS_LOGIC_ERROR",
            status_code=422,
            details=details,
            context=context
        )
        self.rule = rule


def create_error_response(
    exception: BaseGatewayException,
    request: Optional[Request] = None,
    correlation_id: Optional[str] = None
) -> JSONResponse:
    """Create a standardized error response from an exception."""
    from datetime import datetime
    from .logging import get_correlation_id
    
    # Use provided correlation ID or get from context
    if correlation_id is None:
        correlation_id = get_correlation_id()
    
    error_response = ErrorResponse(
        error=exception.error_code,
        message=exception.message,
        details=[ErrorDetail(**detail.dict()) for detail in exception.details] if exception.details else None,
        correlation_id=correlation_id,
        timestamp=datetime.utcnow().isoformat() + "Z",
        path=str(request.url.path) if request else None,
        method=request.method if request else None
    )
    
    return JSONResponse(
        status_code=exception.status_code,
        content=error_response.dict(exclude_none=True)
    )


def create_http_exception(
    exception: BaseGatewayException,
    headers: Optional[Dict[str, str]] = None
) -> HTTPException:
    """Create a FastAPI HTTPException from a gateway exception."""
    return HTTPException(
        status_code=exception.status_code,
        detail=exception.to_dict(),
        headers=headers
    )


def handle_unexpected_error(
    error: Exception,
    request: Optional[Request] = None,
    correlation_id: Optional[str] = None
) -> JSONResponse:
    """Handle unexpected errors and create standardized response."""
    from datetime import datetime
    from .logging import get_correlation_id, get_logger
    
    logger = get_logger("api_gateway.exceptions")
    
    # Use provided correlation ID or get from context
    if correlation_id is None:
        correlation_id = get_correlation_id()
    
    # Log the unexpected error
    logger.error(
        "Unexpected error occurred",
        error_type=type(error).__name__,
        error_message=str(error),
        correlation_id=correlation_id,
        traceback=traceback.format_exc()
    )
    
    # Create generic error response
    error_response = ErrorResponse(
        error="INTERNAL_SERVER_ERROR",
        message="An unexpected error occurred. Please try again later.",
        correlation_id=correlation_id,
        timestamp=datetime.utcnow().isoformat() + "Z",
        path=str(request.url.path) if request else None,
        method=request.method if request else None
    )
    
    return JSONResponse(
        status_code=500,
        content=error_response.dict(exclude_none=True)
    )


# Common error detail creators
def create_validation_detail(
    field: str,
    message: str,
    value: Any = None
) -> ErrorDetail:
    """Create a validation error detail."""
    return ErrorDetail(
        code="VALIDATION_ERROR",
        message=message,
        field=field,
        context={"value": value} if value is not None else None
    )


def create_missing_field_detail(field: str) -> ErrorDetail:
    """Create a missing field error detail."""
    return ErrorDetail(
        code="MISSING_FIELD",
        message=f"Field '{field}' is required",
        field=field
    )


def create_invalid_format_detail(
    field: str,
    expected_format: str,
    actual_value: Any = None
) -> ErrorDetail:
    """Create an invalid format error detail."""
    return ErrorDetail(
        code="INVALID_FORMAT",
        message=f"Field '{field}' must be in format: {expected_format}",
        field=field,
        context={"expected_format": expected_format, "actual_value": actual_value}
    )


def create_resource_not_found_detail(
    resource_type: str,
    resource_id: str
) -> ErrorDetail:
    """Create a resource not found error detail."""
    return ErrorDetail(
        code="RESOURCE_NOT_FOUND",
        message=f"{resource_type} with ID '{resource_id}' not found",
        context={"resource_type": resource_type, "resource_id": resource_id}
    )


def create_duplicate_resource_detail(
    resource_type: str,
    field: str,
    value: Any
) -> ErrorDetail:
    """Create a duplicate resource error detail."""
    return ErrorDetail(
        code="DUPLICATE_RESOURCE",
        message=f"{resource_type} with {field} '{value}' already exists",
        field=field,
        context={"resource_type": resource_type, "field": field, "value": value}
    )


# Export commonly used exceptions and functions
__all__ = [
    # Base exception
    "BaseGatewayException",
    
    # Specific exceptions
    "ValidationException",
    "AuthenticationException",
    "AuthorizationException",
    "ResourceNotFoundException",
    "ConflictException",
    "RateLimitException",
    "ServiceUnavailableException",
    "DatabaseException",
    "ExternalServiceException",
    "CircuitBreakerException",
    "TimeoutException",
    "ConfigurationException",
    "BusinessLogicException",
    
    # Response models
    "ErrorDetail",
    "ErrorResponse",
    
    # Utility functions
    "create_error_response",
    "create_http_exception",
    "handle_unexpected_error",
    
    # Error detail creators
    "create_validation_detail",
    "create_missing_field_detail",
    "create_invalid_format_detail",
    "create_resource_not_found_detail",
    "create_duplicate_resource_detail",
]