#!/usr/bin/env python3
"""
Logging Middleware for TracSeq 2.0 API Gateway
Centralized request/response logging
"""

import time
import uuid
from typing import Callable
from fastapi import Request, Response
from starlette.middleware.base import BaseHTTPMiddleware
from starlette.responses import Response as StarletteResponse

from ..core.logging import request_logger, main_logger
from ..core.config import get_config


class LoggingMiddleware(BaseHTTPMiddleware):
    """Middleware for logging HTTP requests and responses"""
    
    def __init__(self, app, skip_paths: list = None):
        super().__init__(app)
        self.skip_paths = skip_paths or ["/health", "/metrics"]
        self.config = get_config()
    
    async def dispatch(self, request: Request, call_next: Callable) -> Response:
        """Process request and log details"""
        
        # Skip logging for certain paths
        if any(request.url.path.startswith(path) for path in self.skip_paths):
            return await call_next(request)
        
        # Generate request ID if not present
        request_id = request.headers.get("X-Request-ID", str(uuid.uuid4()))
        request.state.request_id = request_id
        
        # Record start time
        start_time = time.time()
        
        # Extract request details
        request_details = {
            "request_id": request_id,
            "method": request.method,
            "url": str(request.url),
            "path": request.url.path,
            "query_params": dict(request.query_params),
            "headers": dict(request.headers),
            "client_ip": self._get_client_ip(request),
            "user_agent": request.headers.get("User-Agent", ""),
        }
        
        # Log request start
        if self.config.logging.enable_access_log:
            main_logger.info(
                f"Request started: {request.method} {request.url.path}",
                extra=request_details
            )
        
        # Process request
        try:
            response = await call_next(request)
            
            # Calculate response time
            response_time = time.time() - start_time
            
            # Log successful response
            if self.config.logging.enable_access_log:
                request_logger.log_request(
                    request=request,
                    response_status=response.status_code,
                    response_time=response_time,
                    request_id=request_id,
                    response_size=self._get_response_size(response)
                )
            
            # Add response headers
            response.headers["X-Request-ID"] = request_id
            response.headers["X-Response-Time"] = f"{response_time:.3f}s"
            
            return response
            
        except Exception as e:
            # Calculate response time for error
            response_time = time.time() - start_time
            
            # Log error
            request_logger.log_error(
                request=request,
                error=e,
                request_id=request_id,
                response_time=response_time
            )
            
            # Re-raise the exception
            raise
    
    def _get_client_ip(self, request: Request) -> str:
        """Extract client IP address from request"""
        # Check for forwarded headers first
        forwarded_for = request.headers.get("X-Forwarded-For")
        if forwarded_for:
            return forwarded_for.split(",")[0].strip()
        
        real_ip = request.headers.get("X-Real-IP")
        if real_ip:
            return real_ip
        
        # Fallback to client host
        if hasattr(request, 'client') and request.client:
            return request.client.host
        
        return "unknown"
    
    def _get_response_size(self, response: Response) -> int:
        """Get response size in bytes"""
        if hasattr(response, 'body') and response.body:
            return len(response.body)
        
        content_length = response.headers.get("Content-Length")
        if content_length:
            try:
                return int(content_length)
            except ValueError:
                pass
        
        return 0


class RequestIDMiddleware(BaseHTTPMiddleware):
    """Middleware for adding request IDs to all requests"""
    
    async def dispatch(self, request: Request, call_next: Callable) -> Response:
        """Add request ID to request state"""
        
        # Generate or extract request ID
        request_id = request.headers.get("X-Request-ID", str(uuid.uuid4()))
        request.state.request_id = request_id
        
        # Process request
        response = await call_next(request)
        
        # Add request ID to response headers
        response.headers["X-Request-ID"] = request_id
        
        return response