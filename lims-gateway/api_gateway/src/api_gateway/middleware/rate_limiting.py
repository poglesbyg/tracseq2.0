#!/usr/bin/env python3
"""
Rate Limiting Middleware for TracSeq 2.0 API Gateway
Centralized rate limiting and throttling
"""

import time
import asyncio
from typing import Callable, Dict, Optional
from collections import defaultdict, deque
from fastapi import Request, Response
from starlette.middleware.base import BaseHTTPMiddleware

from ..core.config import get_config
from ..core.logging import main_logger
from ..core.exceptions import raise_rate_limit_error


class RateLimitMiddleware(BaseHTTPMiddleware):
    """Middleware for rate limiting requests"""
    
    def __init__(self, app, 
                 requests_per_minute: int = None,
                 burst_size: int = None,
                 skip_paths: list = None):
        super().__init__(app)
        self.config = get_config()
        
        # Rate limiting configuration
        self.requests_per_minute = requests_per_minute or self.config.security.rate_limit_requests
        self.window_size = 60  # 1 minute window
        self.burst_size = burst_size or (self.requests_per_minute * 2)
        
        # Paths to skip rate limiting
        self.skip_paths = skip_paths or ["/health", "/metrics"]
        
        # In-memory storage for rate limiting
        # In production, this should use Redis or similar
        self.request_counts: Dict[str, deque] = defaultdict(deque)
        self.burst_counts: Dict[str, int] = defaultdict(int)
        self.last_reset: Dict[str, float] = defaultdict(float)
        
        # Cleanup task
        self._cleanup_task = None
        self._start_cleanup_task()
    
    def _start_cleanup_task(self):
        """Start background cleanup task"""
        if self._cleanup_task is None:
            self._cleanup_task = asyncio.create_task(self._cleanup_old_entries())
    
    async def _cleanup_old_entries(self):
        """Cleanup old rate limiting entries"""
        while True:
            try:
                await asyncio.sleep(300)  # Cleanup every 5 minutes
                current_time = time.time()
                
                # Remove old entries
                keys_to_remove = []
                for key, timestamps in self.request_counts.items():
                    # Remove timestamps older than window
                    while timestamps and current_time - timestamps[0] > self.window_size:
                        timestamps.popleft()
                    
                    # Remove empty entries
                    if not timestamps:
                        keys_to_remove.append(key)
                
                for key in keys_to_remove:
                    del self.request_counts[key]
                    self.burst_counts.pop(key, None)
                    self.last_reset.pop(key, None)
                
                main_logger.debug(f"Rate limiting cleanup: removed {len(keys_to_remove)} entries")
                
            except asyncio.CancelledError:
                break
            except Exception as e:
                main_logger.error(f"Rate limiting cleanup error: {e}")
    
    async def dispatch(self, request: Request, call_next: Callable) -> Response:
        """Process request with rate limiting"""
        
        # Skip rate limiting for certain paths
        if any(request.url.path.startswith(path) for path in self.skip_paths):
            return await call_next(request)
        
        # Get client identifier
        client_id = self._get_client_id(request)
        
        # Check rate limit
        if not self._check_rate_limit(client_id):
            # Calculate retry after
            retry_after = self._calculate_retry_after(client_id)
            
            main_logger.warning(
                f"Rate limit exceeded for client: {client_id}",
                extra={
                    "client_id": client_id,
                    "path": request.url.path,
                    "requests_per_minute": self.requests_per_minute,
                    "retry_after": retry_after
                }
            )
            
            raise_rate_limit_error(
                message=f"Rate limit exceeded: {self.requests_per_minute} requests per minute",
                limit=self.requests_per_minute,
                window=self.window_size,
                retry_after=retry_after
            )
        
        # Record the request
        self._record_request(client_id)
        
        # Process request
        response = await call_next(request)
        
        # Add rate limiting headers
        remaining_requests = self._get_remaining_requests(client_id)
        response.headers["X-RateLimit-Limit"] = str(self.requests_per_minute)
        response.headers["X-RateLimit-Remaining"] = str(remaining_requests)
        response.headers["X-RateLimit-Reset"] = str(int(time.time() + self.window_size))
        
        return response
    
    def _get_client_id(self, request: Request) -> str:
        """Get client identifier for rate limiting"""
        # Try to get user ID from request state (if authenticated)
        if hasattr(request.state, 'user') and request.state.user:
            return f"user:{request.state.user.id}"
        
        # Fallback to IP address
        client_ip = self._get_client_ip(request)
        return f"ip:{client_ip}"
    
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
    
    def _check_rate_limit(self, client_id: str) -> bool:
        """Check if client has exceeded rate limit"""
        current_time = time.time()
        
        # Get request timestamps for this client
        timestamps = self.request_counts[client_id]
        
        # Remove old timestamps
        while timestamps and current_time - timestamps[0] > self.window_size:
            timestamps.popleft()
        
        # Check if under limit
        if len(timestamps) < self.requests_per_minute:
            return True
        
        # Check burst limit (for sudden spikes)
        if self.burst_counts[client_id] < self.burst_size:
            return True
        
        return False
    
    def _record_request(self, client_id: str):
        """Record a request for rate limiting"""
        current_time = time.time()
        
        # Add timestamp
        self.request_counts[client_id].append(current_time)
        
        # Update burst count
        last_reset_time = self.last_reset.get(client_id, 0)
        if current_time - last_reset_time > 60:  # Reset burst every minute
            self.burst_counts[client_id] = 1
            self.last_reset[client_id] = current_time
        else:
            self.burst_counts[client_id] += 1
    
    def _get_remaining_requests(self, client_id: str) -> int:
        """Get remaining requests for client"""
        current_time = time.time()
        timestamps = self.request_counts[client_id]
        
        # Remove old timestamps
        while timestamps and current_time - timestamps[0] > self.window_size:
            timestamps.popleft()
        
        return max(0, self.requests_per_minute - len(timestamps))
    
    def _calculate_retry_after(self, client_id: str) -> int:
        """Calculate retry after time in seconds"""
        timestamps = self.request_counts[client_id]
        if not timestamps:
            return 60
        
        # Time until oldest request expires
        oldest_timestamp = timestamps[0]
        retry_after = int(self.window_size - (time.time() - oldest_timestamp))
        return max(1, retry_after)


class AdaptiveRateLimitMiddleware(RateLimitMiddleware):
    """Advanced rate limiting with adaptive limits based on system load"""
    
    def __init__(self, app, **kwargs):
        super().__init__(app, **kwargs)
        self.system_load_factor = 1.0
        self.load_check_interval = 30  # Check load every 30 seconds
        self.last_load_check = 0
    
    async def dispatch(self, request: Request, call_next: Callable) -> Response:
        """Process request with adaptive rate limiting"""
        
        # Update system load factor periodically
        current_time = time.time()
        if current_time - self.last_load_check > self.load_check_interval:
            self._update_system_load_factor()
            self.last_load_check = current_time
        
        # Adjust rate limit based on system load
        original_limit = self.requests_per_minute
        self.requests_per_minute = int(original_limit * self.system_load_factor)
        
        try:
            return await super().dispatch(request, call_next)
        finally:
            # Restore original limit
            self.requests_per_minute = original_limit
    
    def _update_system_load_factor(self):
        """Update system load factor based on current conditions"""
        try:
            import psutil
            
            # Get CPU usage
            cpu_percent = psutil.cpu_percent(interval=1)
            
            # Get memory usage
            memory = psutil.virtual_memory()
            memory_percent = memory.percent
            
            # Calculate load factor (reduce limits when system is under stress)
            if cpu_percent > 80 or memory_percent > 80:
                self.system_load_factor = 0.5  # Reduce to 50%
            elif cpu_percent > 60 or memory_percent > 60:
                self.system_load_factor = 0.7  # Reduce to 70%
            else:
                self.system_load_factor = 1.0  # Normal limits
            
            main_logger.debug(
                f"System load factor updated: {self.system_load_factor}",
                extra={
                    "cpu_percent": cpu_percent,
                    "memory_percent": memory_percent,
                    "load_factor": self.system_load_factor
                }
            )
            
        except ImportError:
            # psutil not available, keep normal limits
            self.system_load_factor = 1.0
        except Exception as e:
            main_logger.error(f"Error updating system load factor: {e}")
            self.system_load_factor = 1.0