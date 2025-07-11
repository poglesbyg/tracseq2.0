#!/usr/bin/env python3
"""
Enhanced Proxy Service for TracSeq 2.0 API Gateway
Centralized service routing with circuit breaker and load balancing
"""

import time
import asyncio
from typing import Dict, List, Optional, Any
from urllib.parse import urljoin, urlparse
from contextlib import asynccontextmanager

import httpx
from fastapi import Request, Response, HTTPException
from fastapi.responses import StreamingResponse, JSONResponse

from ..core.config import get_config
from ..core.logging import service_logger, main_logger
from ..core.exceptions import raise_service_error, raise_circuit_breaker_error
from ..core.circuit_breaker import CircuitBreaker


class ServiceProxy:
    """Enhanced service proxy with circuit breaker and health monitoring"""
    
    def __init__(self):
        self.config = get_config()
        self.circuit_breakers: Dict[str, CircuitBreaker] = {}
        self.service_health: Dict[str, Dict] = {}
        self.last_health_check: Dict[str, float] = {}
        self.health_check_interval = 30  # seconds
        
        # Initialize circuit breakers for each service
        self._initialize_circuit_breakers()
    
    def _initialize_circuit_breakers(self):
        """Initialize circuit breakers for all services"""
        services = [
            "auth", "sample", "storage", "template", "sequencing",
            "notification", "rag", "event", "transaction", "cognitive",
            "reports", "lab_manager", "project", "library_prep", "qaqc", "flow_cell"
        ]
        
        for service_name in services:
            self.circuit_breakers[service_name] = CircuitBreaker(
                failure_threshold=self.config.monitoring.circuit_breaker_failure_threshold,
                recovery_timeout=self.config.monitoring.circuit_breaker_recovery_timeout,
                name=service_name
            )
    
    async def proxy_request(self, 
                           service_name: str,
                           request: Request,
                           path: str = None,
                           timeout: int = None) -> Response:
        """Proxy request to a microservice with circuit breaker protection"""
        
        # Get service URL
        service_url = self.config.get_service_url(service_name)
        if not service_url:
            raise_service_error(service_name, f"Service URL not configured for {service_name}")
        
        # Get circuit breaker
        circuit_breaker = self.circuit_breakers.get(service_name)
        if not circuit_breaker:
            raise_service_error(service_name, f"Circuit breaker not found for {service_name}")
        
        # Check circuit breaker state
        if circuit_breaker.is_open():
            raise_circuit_breaker_error(service_name)
        
        # Build target URL
        target_path = path or request.url.path
        if target_path.startswith(f"/api/{service_name}"):
            # Remove service prefix for internal routing
            target_path = target_path.replace(f"/api/{service_name}", "")
        
        target_url = urljoin(service_url, target_path.lstrip("/"))
        
        # Add query parameters
        if request.url.query:
            target_url += f"?{request.url.query}"
        
        start_time = time.time()
        
        try:
            # Make the request
            async with httpx.AsyncClient(timeout=timeout or self.config.security.service_timeout) as client:
                
                # Prepare request data
                request_data = await self._prepare_request_data(request)
                
                # Make the proxied request
                response = await client.request(
                    method=request.method,
                    url=target_url,
                    headers=request_data["headers"],
                    params=request_data["params"],
                    content=request_data["content"],
                    timeout=timeout or self.config.security.service_timeout
                )
                
                response_time = time.time() - start_time
                
                # Record success in circuit breaker
                circuit_breaker.record_success()
                
                # Log successful request
                service_logger.log_service_call(
                    service_name=service_name,
                    method=request.method,
                    url=target_url,
                    status_code=response.status_code,
                    response_time=response_time
                )
                
                # Create response
                return await self._create_response(response)
                
        except httpx.TimeoutException as e:
            response_time = time.time() - start_time
            circuit_breaker.record_failure()
            
            service_logger.log_service_error(
                service_name=service_name,
                error=e,
                url=target_url,
                response_time=response_time
            )
            
            raise_service_error(
                service_name=service_name,
                message=f"Service timeout after {response_time:.2f}s",
                status_code=504,
                original_error=e
            )
            
        except httpx.ConnectError as e:
            response_time = time.time() - start_time
            circuit_breaker.record_failure()
            
            service_logger.log_service_error(
                service_name=service_name,
                error=e,
                url=target_url,
                response_time=response_time
            )
            
            raise_service_error(
                service_name=service_name,
                message=f"Service unavailable: {service_name}",
                status_code=503,
                original_error=e
            )
            
        except httpx.HTTPStatusError as e:
            response_time = time.time() - start_time
            
            # Only record as failure for 5xx errors
            if e.response.status_code >= 500:
                circuit_breaker.record_failure()
            
            service_logger.log_service_error(
                service_name=service_name,
                error=e,
                url=target_url,
                response_time=response_time,
                status_code=e.response.status_code
            )
            
            raise_service_error(
                service_name=service_name,
                message=f"Service error: {e.response.status_code}",
                status_code=e.response.status_code,
                original_error=e
            )
            
        except Exception as e:
            response_time = time.time() - start_time
            circuit_breaker.record_failure()
            
            service_logger.log_service_error(
                service_name=service_name,
                error=e,
                url=target_url,
                response_time=response_time
            )
            
            raise_service_error(
                service_name=service_name,
                message=f"Unexpected error: {str(e)}",
                status_code=502,
                original_error=e
            )
    
    async def _prepare_request_data(self, request: Request) -> Dict[str, Any]:
        """Prepare request data for proxying"""
        
        # Get headers (filter out hop-by-hop headers)
        headers = dict(request.headers)
        hop_by_hop_headers = {
            'connection', 'keep-alive', 'proxy-authenticate',
            'proxy-authorization', 'te', 'trailers', 'transfer-encoding',
            'upgrade', 'host', 'content-length'
        }
        
        filtered_headers = {
            k: v for k, v in headers.items()
            if k.lower() not in hop_by_hop_headers
        }
        
        # Get query parameters
        params = dict(request.query_params)
        
        # Get request body
        content = None
        if request.method in ['POST', 'PUT', 'PATCH']:
            content = await request.body()
        
        return {
            "headers": filtered_headers,
            "params": params,
            "content": content
        }
    
    async def _create_response(self, httpx_response: httpx.Response) -> Response:
        """Create FastAPI response from httpx response"""
        
        # Filter response headers
        response_headers = dict(httpx_response.headers)
        hop_by_hop_headers = {
            'connection', 'keep-alive', 'proxy-authenticate',
            'proxy-authorization', 'te', 'trailers', 'transfer-encoding',
            'upgrade'
        }
        
        filtered_headers = {
            k: v for k, v in response_headers.items()
            if k.lower() not in hop_by_hop_headers
        }
        
        # Handle different content types
        content_type = httpx_response.headers.get('content-type', '')
        
        if 'application/json' in content_type:
            try:
                content = httpx_response.json()
                return JSONResponse(
                    content=content,
                    status_code=httpx_response.status_code,
                    headers=filtered_headers
                )
            except:
                # Fallback to text if JSON parsing fails
                pass
        
        # Handle streaming responses
        if 'text/event-stream' in content_type:
            async def generate():
                async for chunk in httpx_response.aiter_bytes():
                    yield chunk
            
            return StreamingResponse(
                generate(),
                status_code=httpx_response.status_code,
                headers=filtered_headers,
                media_type=content_type
            )
        
        # Default response
        return Response(
            content=httpx_response.content,
            status_code=httpx_response.status_code,
            headers=filtered_headers,
            media_type=content_type
        )
    
    async def check_service_health(self, service_name: str) -> Dict[str, Any]:
        """Check health of a specific service"""
        
        current_time = time.time()
        last_check = self.last_health_check.get(service_name, 0)
        
        # Return cached result if checked recently
        if current_time - last_check < self.health_check_interval:
            return self.service_health.get(service_name, {"healthy": False, "cached": True})
        
        service_url = self.config.get_service_url(service_name)
        if not service_url:
            return {"healthy": False, "error": "Service URL not configured"}
        
        health_url = urljoin(service_url, "/health")
        
        try:
            async with httpx.AsyncClient(timeout=self.config.security.health_check_timeout) as client:
                response = await client.get(health_url)
                
                health_data = {
                    "healthy": response.status_code == 200,
                    "status_code": response.status_code,
                    "response_time": response.elapsed.total_seconds(),
                    "last_check": current_time,
                    "circuit_breaker_state": self.circuit_breakers[service_name].state.value if service_name in self.circuit_breakers else "unknown"
                }
                
                try:
                    health_data["details"] = response.json()
                except:
                    health_data["details"] = {"message": response.text}
                
                self.service_health[service_name] = health_data
                self.last_health_check[service_name] = current_time
                
                service_logger.log_service_health(
                    service_name=service_name,
                    is_healthy=health_data["healthy"],
                    response_time=health_data["response_time"]
                )
                
                return health_data
                
        except Exception as e:
            health_data = {
                "healthy": False,
                "error": str(e),
                "last_check": current_time,
                "circuit_breaker_state": self.circuit_breakers[service_name].state.value if service_name in self.circuit_breakers else "unknown"
            }
            
            self.service_health[service_name] = health_data
            self.last_health_check[service_name] = current_time
            
            service_logger.log_service_health(
                service_name=service_name,
                is_healthy=False,
                error=str(e)
            )
            
            return health_data
    
    async def get_all_service_health(self) -> Dict[str, Any]:
        """Get health status of all services"""
        
        services = [
            "auth", "sample", "storage", "template", "sequencing",
            "notification", "rag", "event", "transaction", "cognitive",
            "reports", "lab_manager", "project", "library_prep", "qaqc", "flow_cell"
        ]
        
        health_results = {}
        
        # Check all services in parallel
        tasks = []
        for service_name in services:
            task = asyncio.create_task(self.check_service_health(service_name))
            tasks.append((service_name, task))
        
        for service_name, task in tasks:
            try:
                health_results[service_name] = await task
            except Exception as e:
                health_results[service_name] = {
                    "healthy": False,
                    "error": str(e)
                }
        
        # Calculate overall health
        healthy_services = sum(1 for health in health_results.values() if health.get("healthy", False))
        total_services = len(health_results)
        
        return {
            "overall_health": "healthy" if healthy_services == total_services else "degraded" if healthy_services > 0 else "unhealthy",
            "healthy_services": healthy_services,
            "total_services": total_services,
            "services": health_results,
            "timestamp": time.time()
        }
    
    def get_circuit_breaker_status(self) -> Dict[str, Any]:
        """Get status of all circuit breakers"""
        
        status = {}
        for service_name, circuit_breaker in self.circuit_breakers.items():
            status[service_name] = {
                "state": circuit_breaker.state.value,
                "failure_count": circuit_breaker.failure_count,
                "last_failure_time": circuit_breaker.last_failure_time,
                "next_attempt_time": circuit_breaker.next_attempt_time
            }
        
        return status


# Global service proxy instance
service_proxy = ServiceProxy()
