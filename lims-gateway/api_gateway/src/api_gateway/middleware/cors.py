"""
CORS middleware for the TracSeq 2.0 API Gateway.

This module provides centralized CORS configuration and middleware
for handling cross-origin requests with proper security controls.
"""

from typing import List, Optional
from fastapi import FastAPI
from fastapi.middleware.cors import CORSMiddleware

from ..core.config import get_cors_config
from ..core.logging import get_logger


logger = get_logger("api_gateway.middleware.cors")


def setup_cors_middleware(app: FastAPI) -> None:
    """
    Setup CORS middleware with configuration from settings.
    
    Args:
        app: FastAPI application instance
    """
    config = get_cors_config()
    
    logger.info(
        "Setting up CORS middleware",
        allow_origins=config.allow_origins,
        allow_credentials=config.allow_credentials,
        allow_methods=config.allow_methods,
        allow_headers=config.allow_headers,
        max_age=config.max_age
    )
    
    app.add_middleware(
        CORSMiddleware,
        allow_origins=config.allow_origins,
        allow_credentials=config.allow_credentials,
        allow_methods=config.allow_methods,
        allow_headers=config.allow_headers,
        max_age=config.max_age
    )


def get_cors_headers(
    origin: Optional[str] = None,
    request_method: Optional[str] = None,
    request_headers: Optional[List[str]] = None
) -> dict:
    """
    Get CORS headers for manual response handling.
    
    Args:
        origin: Request origin
        request_method: HTTP method
        request_headers: Requested headers
        
    Returns:
        Dictionary of CORS headers
    """
    config = get_cors_config()
    headers = {}
    
    # Check if origin is allowed
    if origin and (
        "*" in config.allow_origins or 
        origin in config.allow_origins
    ):
        headers["Access-Control-Allow-Origin"] = origin
    elif "*" in config.allow_origins:
        headers["Access-Control-Allow-Origin"] = "*"
    
    # Add credentials header if enabled
    if config.allow_credentials:
        headers["Access-Control-Allow-Credentials"] = "true"
    
    # Add allowed methods
    if request_method and (
        "*" in config.allow_methods or 
        request_method in config.allow_methods
    ):
        headers["Access-Control-Allow-Methods"] = ", ".join(config.allow_methods)
    
    # Add allowed headers
    if request_headers:
        allowed_headers = []
        for header in request_headers:
            if "*" in config.allow_headers or header in config.allow_headers:
                allowed_headers.append(header)
        if allowed_headers:
            headers["Access-Control-Allow-Headers"] = ", ".join(allowed_headers)
    elif config.allow_headers:
        headers["Access-Control-Allow-Headers"] = ", ".join(config.allow_headers)
    
    # Add max age
    if config.max_age:
        headers["Access-Control-Max-Age"] = str(config.max_age)
    
    return headers


def is_cors_preflight_request(method: str, headers: dict) -> bool:
    """
    Check if request is a CORS preflight request.
    
    Args:
        method: HTTP method
        headers: Request headers
        
    Returns:
        True if this is a preflight request
    """
    return (
        method == "OPTIONS" and
        "origin" in headers and
        "access-control-request-method" in headers
    )


def validate_cors_request(
    origin: Optional[str],
    method: str,
    headers: Optional[List[str]] = None
) -> tuple[bool, str]:
    """
    Validate CORS request against configuration.
    
    Args:
        origin: Request origin
        method: HTTP method
        headers: Request headers
        
    Returns:
        Tuple of (is_valid, error_message)
    """
    config = get_cors_config()
    
    # Check origin
    if origin and not ("*" in config.allow_origins or origin in config.allow_origins):
        return False, f"Origin '{origin}' not allowed"
    
    # Check method
    if not ("*" in config.allow_methods or method in config.allow_methods):
        return False, f"Method '{method}' not allowed"
    
    # Check headers
    if headers:
        for header in headers:
            if not ("*" in config.allow_headers or header in config.allow_headers):
                return False, f"Header '{header}' not allowed"
    
    return True, ""


class CORSSecurityMiddleware:
    """
    Enhanced CORS middleware with additional security features.
    """
    
    def __init__(self, app: FastAPI):
        self.app = app
        self.config = get_cors_config()
    
    async def __call__(self, scope, receive, send):
        """ASGI middleware implementation."""
        if scope["type"] != "http":
            await self.app(scope, receive, send)
            return
        
        # Get request details
        method = scope["method"]
        headers = dict(scope.get("headers", []))
        origin = headers.get(b"origin", b"").decode()
        
        # Handle preflight requests
        if is_cors_preflight_request(method, headers):
            await self._handle_preflight(scope, receive, send, origin)
            return
        
        # Validate CORS for actual requests
        is_valid, error_msg = validate_cors_request(
            origin, method, 
            headers.get(b"access-control-request-headers", b"").decode().split(",")
        )
        
        if not is_valid:
            logger.warning(
                "CORS validation failed",
                origin=origin,
                method=method,
                error=error_msg
            )
            
            # Send error response
            response = {
                "type": "http.response.start",
                "status": 403,
                "headers": [
                    [b"content-type", b"application/json"],
                    [b"content-length", b"0"]
                ]
            }
            await send(response)
            await send({"type": "http.response.body", "body": b""})
            return
        
        # Continue with request
        await self.app(scope, receive, send)
    
    async def _handle_preflight(self, scope, receive, send, origin: str):
        """Handle CORS preflight requests."""
        headers = get_cors_headers(
            origin=origin,
            request_method=scope.get("method"),
            request_headers=scope.get("headers", {}).get("access-control-request-headers", "").split(",")
        )
        
        response_headers = [
            [b"content-type", b"application/json"],
            [b"content-length", b"0"]
        ]
        
        # Add CORS headers
        for key, value in headers.items():
            response_headers.append([key.lower().encode(), value.encode()])
        
        response = {
            "type": "http.response.start",
            "status": 200,
            "headers": response_headers
        }
        
        await send(response)
        await send({"type": "http.response.body", "body": b""})


# Export commonly used functions and classes
__all__ = [
    "setup_cors_middleware",
    "get_cors_headers",
    "is_cors_preflight_request",
    "validate_cors_request",
    "CORSSecurityMiddleware"
]