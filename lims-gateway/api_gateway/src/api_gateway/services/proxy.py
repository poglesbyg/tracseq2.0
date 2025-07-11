"""
Enhanced Proxy Service for TracSeq API Gateway

Handles request forwarding, response transformation, error handling,
circuit breaker patterns, and health monitoring.
"""

import asyncio
import json
import time
from datetime import datetime, timedelta
from enum import Enum
from typing import Any, Dict, Optional, List
from dataclasses import dataclass

import httpx
from fastapi import HTTPException, Request, Response

from ..core.config import get_service_config, get_monitoring_config
from ..core.logging import get_service_logger, correlation_context
from ..core.exceptions import (
    ExternalServiceException,
    CircuitBreakerException,
    TimeoutException,
    ServiceUnavailableException
)


class CircuitState(Enum):
    """Circuit breaker states."""
    CLOSED = "closed"
    OPEN = "open"
    HALF_OPEN = "half_open"


@dataclass
class ServiceHealth:
    """Service health information."""
    name: str
    status: str
    response_time_ms: float
    last_check: datetime
    error_count: int
    success_count: int
    url: str


class CircuitBreaker:
    """Circuit breaker implementation for service resilience."""
    
    def __init__(self, service_name: str, failure_threshold: int = 5, 
                 recovery_timeout: int = 60, half_open_max_calls: int = 3):
        self.service_name = service_name
        self.failure_threshold = failure_threshold
        self.recovery_timeout = recovery_timeout
        self.half_open_max_calls = half_open_max_calls
        
        self.state = CircuitState.CLOSED
        self.failure_count = 0
        self.last_failure_time = None
        self.half_open_calls = 0
        
        self.logger = get_service_logger()
    
    async def call(self, func, *args, **kwargs):
        """Execute function with circuit breaker protection."""
        if self.state == CircuitState.OPEN:
            if self._should_attempt_reset():
                self.state = CircuitState.HALF_OPEN
                self.half_open_calls = 0
                self.logger.info(
                    "Circuit breaker transitioning to half-open",
                    service_name=self.service_name
                )
            else:
                raise CircuitBreakerException(
                    f"Circuit breaker is open for {self.service_name}",
                    service_name=self.service_name
                )
        
        if self.state == CircuitState.HALF_OPEN:
            if self.half_open_calls >= self.half_open_max_calls:
                raise CircuitBreakerException(
                    f"Circuit breaker half-open limit reached for {self.service_name}",
                    service_name=self.service_name
                )
            self.half_open_calls += 1
        
        try:
            result = await func(*args, **kwargs)
            self._on_success()
            return result
        except Exception as e:
            self._on_failure()
            raise e
    
    def _should_attempt_reset(self) -> bool:
        """Check if circuit breaker should attempt reset."""
        if self.last_failure_time is None:
            return True
        
        return (datetime.now() - self.last_failure_time).total_seconds() > self.recovery_timeout
    
    def _on_success(self):
        """Handle successful call."""
        if self.state == CircuitState.HALF_OPEN:
            self.state = CircuitState.CLOSED
            self.logger.log_circuit_breaker_close(self.service_name)
        
        self.failure_count = 0
        self.half_open_calls = 0
    
    def _on_failure(self):
        """Handle failed call."""
        self.failure_count += 1
        self.last_failure_time = datetime.now()
        
        if self.state == CircuitState.HALF_OPEN:
            self.state = CircuitState.OPEN
            self.logger.log_circuit_breaker_open(self.service_name)
        elif self.failure_count >= self.failure_threshold:
            self.state = CircuitState.OPEN
            self.logger.log_circuit_breaker_open(self.service_name)


class EnhancedProxyService:
    """Enhanced service for proxying requests to microservices with resilience patterns."""

    def __init__(self, http_client: httpx.AsyncClient):
        self.http_client = http_client
        self.service_config = get_service_config()
        self.monitoring_config = get_monitoring_config()
        self.logger = get_service_logger()
        
        # Circuit breakers for each service
        self.circuit_breakers = {}
        
        # Service health tracking
        self.service_health = {}
        
        # Request statistics
        self.request_count = 0
        self.error_count = 0
        self.service_stats = {}
        
        # Initialize circuit breakers
        self._initialize_circuit_breakers()
    
    def _initialize_circuit_breakers(self):
        """Initialize circuit breakers for all services."""
        services = [
            ("auth", self.service_config.auth_service_url),
            ("sample", self.service_config.sample_service_url),
            ("storage", self.service_config.storage_service_url),
            ("template", self.service_config.template_service_url),
            ("sequencing", self.service_config.sequencing_service_url),
            ("rag", self.service_config.rag_service_url),
            ("notification", self.service_config.notification_service_url),
        ]
        
        for service_name, service_url in services:
            self.circuit_breakers[service_name] = CircuitBreaker(
                service_name=service_name,
                failure_threshold=self.monitoring_config.circuit_breaker_failure_threshold,
                recovery_timeout=self.monitoring_config.circuit_breaker_recovery_timeout,
                half_open_max_calls=self.monitoring_config.circuit_breaker_half_open_max_calls
            )
    
    async def proxy_request(
        self,
        request: Request,
        service_name: str,
        service_url: str,
        path: str,
        response: Response
    ) -> Response:
        """Proxy a request to the target service with circuit breaker protection."""
        
        start_time = time.time()
        self.request_count += 1
        
        # Get or create circuit breaker
        circuit_breaker = self.circuit_breakers.get(
            service_name, 
            CircuitBreaker(service_name)
        )
        
        try:
            # Use circuit breaker to protect the call
            return await circuit_breaker.call(
                self._make_request,
                request, service_name, service_url, path, response, start_time
            )
        except CircuitBreakerException:
            self.error_count += 1
            self._update_service_stats(service_name, False, time.time() - start_time)
            raise
        except Exception as e:
            self.error_count += 1
            self._update_service_stats(service_name, False, time.time() - start_time)
            raise
    
    async def _make_request(
        self,
        request: Request,
        service_name: str,
        service_url: str,
        path: str,
        response: Response,
        start_time: float
    ) -> Response:
        """Make the actual HTTP request to the service."""
        
        # Build upstream URL
        upstream_url = f"{service_url}{path}"
        
        # Prepare headers
        headers = self._prepare_headers(dict(request.headers))
        
        # Get request body
        body = await request.body()
        
        # Log request
        self.logger.log_service_call(
            service_name=service_name,
            method=request.method,
            url=upstream_url
        )
        
        try:
            # Make upstream request with timeout and retries
            upstream_response = await self._make_http_request(
                method=request.method,
                url=upstream_url,
                headers=headers,
                content=body,
                params=dict(request.query_params),
                timeout=self.service_config.service_timeout
            )
            
            # Calculate request duration
            duration = time.time() - start_time
            
            # Log response
            self.logger.log_service_response(
                service_name=service_name,
                method=request.method,
                url=upstream_url,
                status_code=upstream_response.status_code,
                duration_ms=duration * 1000
            )
            
            # Update statistics
            self._update_service_stats(service_name, True, duration)
            
            # Prepare response headers
            response_headers = self._prepare_response_headers(dict(upstream_response.headers))
            
            # Return response
            return Response(
                content=upstream_response.content,
                status_code=upstream_response.status_code,
                headers=response_headers
            )
            
        except httpx.TimeoutException:
            duration = time.time() - start_time
            self.logger.log_service_error(
                service_name=service_name,
                method=request.method,
                url=upstream_url,
                error_message=f"Request timeout after {self.service_config.service_timeout}s"
            )
            
            raise TimeoutException(
                f"Service {service_name} did not respond within {self.service_config.service_timeout}s",
                timeout_seconds=self.service_config.service_timeout,
                operation="proxy_request"
            )
        
        except httpx.ConnectError:
            duration = time.time() - start_time
            self.logger.log_service_error(
                service_name=service_name,
                method=request.method,
                url=upstream_url,
                error_message="Connection failed"
            )
            
            raise ServiceUnavailableException(
                f"Service {service_name} is temporarily unavailable",
                service_name=service_name
            )
        
        except httpx.HTTPStatusError as e:
            duration = time.time() - start_time
            
            # For 4xx errors, don't treat as service failure
            if 400 <= e.response.status_code < 500:
                self.logger.log_service_response(
                    service_name=service_name,
                    method=request.method,
                    url=upstream_url,
                    status_code=e.response.status_code,
                    duration_ms=duration * 1000
                )
                
                # Forward the error response
                return Response(
                    content=e.response.content,
                    status_code=e.response.status_code,
                    headers=self._prepare_response_headers(dict(e.response.headers))
                )
            else:
                # 5xx errors are service failures
                self.logger.log_service_error(
                    service_name=service_name,
                    method=request.method,
                    url=upstream_url,
                    error_message=f"HTTP {e.response.status_code} error"
                )
                
                raise ExternalServiceException(
                    f"Service {service_name} returned error {e.response.status_code}",
                    service_name=service_name,
                    service_url=upstream_url,
                    status_code=e.response.status_code
                )
    
    async def _make_http_request(self, **kwargs) -> httpx.Response:
        """Make HTTP request with retry logic."""
        last_exception = None
        
        for attempt in range(self.service_config.service_retries + 1):
            try:
                return await self.http_client.request(**kwargs)
            except (httpx.ConnectError, httpx.TimeoutException) as e:
                last_exception = e
                if attempt < self.service_config.service_retries:
                    await asyncio.sleep(self.service_config.service_retry_delay * (2 ** attempt))
                continue
            except httpx.HTTPStatusError as e:
                # Don't retry on HTTP errors
                raise e
        
        # If we get here, all retries failed
        if last_exception:
            raise last_exception
        else:
            raise httpx.ConnectError("All retry attempts failed")
    
    def _prepare_headers(self, headers: Dict[str, str]) -> Dict[str, str]:
        """Prepare request headers for upstream service."""
        # Remove hop-by-hop headers
        hop_by_hop_headers = {
            "connection", "keep-alive", "proxy-authenticate",
            "proxy-authorization", "te", "trailers", "transfer-encoding",
            "upgrade", "proxy-connection"
        }
        
        cleaned_headers = {
            k: v for k, v in headers.items()
            if k.lower() not in hop_by_hop_headers
        }
        
        # Add gateway headers
        cleaned_headers["X-Gateway"] = "TracSeq-API-Gateway"
        cleaned_headers["X-Gateway-Version"] = "2.0.0"
        cleaned_headers["X-Forwarded-By"] = "TracSeq-Gateway"
        
        return cleaned_headers
    
    def _prepare_response_headers(self, headers: Dict[str, str]) -> Dict[str, str]:
        """Prepare response headers from upstream service."""
        # Remove hop-by-hop headers
        hop_by_hop_headers = {
            "connection", "keep-alive", "proxy-authenticate",
            "proxy-authorization", "te", "trailers", "transfer-encoding",
            "upgrade", "proxy-connection"
        }
        
        cleaned_headers = {
            k: v for k, v in headers.items()
            if k.lower() not in hop_by_hop_headers
        }
        
        # Add gateway response headers
        cleaned_headers["X-Gateway"] = "TracSeq-API-Gateway"
        cleaned_headers["X-Gateway-Request-ID"] = str(self.request_count)
        
        return cleaned_headers
    
    def _update_service_stats(self, service_name: str, success: bool, duration: float):
        """Update service statistics."""
        if service_name not in self.service_stats:
            self.service_stats[service_name] = {
                "total_requests": 0,
                "successful_requests": 0,
                "failed_requests": 0,
                "total_duration": 0.0,
                "avg_duration": 0.0
            }
        
        stats = self.service_stats[service_name]
        stats["total_requests"] += 1
        stats["total_duration"] += duration
        stats["avg_duration"] = stats["total_duration"] / stats["total_requests"]
        
        if success:
            stats["successful_requests"] += 1
        else:
            stats["failed_requests"] += 1
    
    async def health_check_service(self, service_name: str, service_url: str) -> ServiceHealth:
        """Check health of a specific service."""
        health_url = f"{service_url}/health"
        
        try:
            start_time = time.time()
            
            response = await self.http_client.get(
                health_url,
                timeout=self.service_config.service_timeout
            )
            
            duration = time.time() - start_time
            
            status = "healthy" if response.status_code == 200 else "unhealthy"
            
            health = ServiceHealth(
                name=service_name,
                status=status,
                response_time_ms=round(duration * 1000, 2),
                last_check=datetime.now(),
                error_count=self.service_stats.get(service_name, {}).get("failed_requests", 0),
                success_count=self.service_stats.get(service_name, {}).get("successful_requests", 0),
                url=health_url
            )
            
            self.service_health[service_name] = health
            return health
            
        except httpx.TimeoutException:
            health = ServiceHealth(
                name=service_name,
                status="timeout",
                response_time_ms=self.service_config.service_timeout * 1000,
                last_check=datetime.now(),
                error_count=self.service_stats.get(service_name, {}).get("failed_requests", 0),
                success_count=self.service_stats.get(service_name, {}).get("successful_requests", 0),
                url=health_url
            )
            
            self.service_health[service_name] = health
            return health
            
        except httpx.ConnectError:
            health = ServiceHealth(
                name=service_name,
                status="unreachable",
                response_time_ms=0,
                last_check=datetime.now(),
                error_count=self.service_stats.get(service_name, {}).get("failed_requests", 0),
                success_count=self.service_stats.get(service_name, {}).get("successful_requests", 0),
                url=health_url
            )
            
            self.service_health[service_name] = health
            return health
            
        except Exception as e:
            health = ServiceHealth(
                name=service_name,
                status="error",
                response_time_ms=0,
                last_check=datetime.now(),
                error_count=self.service_stats.get(service_name, {}).get("failed_requests", 0),
                success_count=self.service_stats.get(service_name, {}).get("successful_requests", 0),
                url=health_url
            )
            
            self.service_health[service_name] = health
            return health
    
    async def health_check_all_services(self) -> Dict[str, ServiceHealth]:
        """Check health of all configured services."""
        services = [
            ("auth", self.service_config.auth_service_url),
            ("sample", self.service_config.sample_service_url),
            ("storage", self.service_config.storage_service_url),
            ("template", self.service_config.template_service_url),
            ("sequencing", self.service_config.sequencing_service_url),
            ("rag", self.service_config.rag_service_url),
            ("notification", self.service_config.notification_service_url),
        ]
        
        health_checks = []
        for service_name, service_url in services:
            health_checks.append(self.health_check_service(service_name, service_url))
        
        results = await asyncio.gather(*health_checks, return_exceptions=True)
        
        health_status = {}
        for i, result in enumerate(results):
            service_name = services[i][0]
            if isinstance(result, Exception):
                health_status[service_name] = ServiceHealth(
                    name=service_name,
                    status="error",
                    response_time_ms=0,
                    last_check=datetime.now(),
                    error_count=0,
                    success_count=0,
                    url=f"{services[i][1]}/health"
                )
            else:
                health_status[service_name] = result
        
        return health_status
    
    def get_stats(self) -> Dict[str, Any]:
        """Get comprehensive proxy service statistics."""
        error_rate = (self.error_count / self.request_count * 100) if self.request_count > 0 else 0
        
        # Circuit breaker states
        circuit_states = {
            name: cb.state.value for name, cb in self.circuit_breakers.items()
        }
        
        return {
            "total_requests": self.request_count,
            "total_errors": self.error_count,
            "error_rate_percent": round(error_rate, 2),
            "success_rate_percent": round(100 - error_rate, 2),
            "service_stats": self.service_stats,
            "circuit_breaker_states": circuit_states,
            "service_health": {
                name: {
                    "status": health.status,
                    "response_time_ms": health.response_time_ms,
                    "last_check": health.last_check.isoformat(),
                    "error_count": health.error_count,
                    "success_count": health.success_count
                }
                for name, health in self.service_health.items()
            }
        }


# For backward compatibility, keep the original class name as an alias
ProxyService = EnhancedProxyService


# Export commonly used classes
__all__ = [
    "ProxyService",
    "EnhancedProxyService",
    "CircuitBreaker",
    "CircuitState",
    "ServiceHealth"
]
