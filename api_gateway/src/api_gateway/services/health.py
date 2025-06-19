"""
Health Service for TracSeq API Gateway

Monitors health of backend microservices and provides aggregated health status.
"""

import asyncio
import time
from typing import Dict, Any, List
from datetime import datetime, timedelta

import httpx
import structlog

from api_gateway.core.config import TracSeqAPIGatewayConfig, ServiceEndpoint

logger = structlog.get_logger(__name__)


class HealthService:
    """Service for monitoring health of backend microservices."""
    
    def __init__(self, http_client: httpx.AsyncClient, config: TracSeqAPIGatewayConfig):
        self.http_client = http_client
        self.config = config
        self.health_status: Dict[str, Dict[str, Any]] = {}
        self.monitoring_task: asyncio.Task = None
        self.monitoring_active = False
        
    async def start_health_monitoring(self):
        """Start continuous health monitoring of services."""
        if self.monitoring_active:
            return
            
        self.monitoring_active = True
        self.monitoring_task = asyncio.create_task(self._health_monitoring_loop())
        logger.info("Health monitoring started")
        
    async def stop_health_monitoring(self):
        """Stop health monitoring."""
        self.monitoring_active = False
        if self.monitoring_task:
            self.monitoring_task.cancel()
            try:
                await self.monitoring_task
            except asyncio.CancelledError:
                pass
        logger.info("Health monitoring stopped")
        
    async def _health_monitoring_loop(self):
        """Background task for continuous health monitoring."""
        while self.monitoring_active:
            try:
                await self._check_all_services()
                await asyncio.sleep(self.config.load_balancer.health_check_interval)
            except asyncio.CancelledError:
                break
            except Exception as e:
                logger.error("Error in health monitoring loop", error=str(e))
                await asyncio.sleep(5)  # Short delay before retrying
                
    async def _check_all_services(self):
        """Check health of all configured services."""
        tasks = []
        
        for service_name, service_config in self.config.services.items():
            task = asyncio.create_task(
                self._check_service_health(service_name, service_config)
            )
            tasks.append(task)
            
        # Wait for all health checks to complete
        results = await asyncio.gather(*tasks, return_exceptions=True)
        
        # Process results
        for i, (service_name, _) in enumerate(self.config.services.items()):
            result = results[i]
            if isinstance(result, Exception):
                logger.error("Health check failed", 
                           service=service_name, 
                           error=str(result))
                self.health_status[service_name] = {
                    "status": "error",
                    "error": str(result),
                    "last_checked": datetime.utcnow().isoformat(),
                    "consecutive_failures": self.health_status.get(service_name, {}).get("consecutive_failures", 0) + 1
                }
            else:
                self.health_status[service_name] = result
                
    async def _check_service_health(self, service_name: str, service_config: ServiceEndpoint) -> Dict[str, Any]:
        """Check health of a single service."""
        start_time = time.time()
        
        try:
            response = await self.http_client.get(
                service_config.health_url,
                timeout=self.config.load_balancer.health_check_timeout
            )
            
            duration = time.time() - start_time
            
            # Determine health status
            if response.status_code == 200:
                status = "healthy"
                consecutive_failures = 0
            elif 200 <= response.status_code < 300:
                status = "healthy"
                consecutive_failures = 0
            elif 400 <= response.status_code < 500:
                status = "degraded"
                consecutive_failures = self.health_status.get(service_name, {}).get("consecutive_failures", 0) + 1
            else:
                status = "unhealthy"
                consecutive_failures = self.health_status.get(service_name, {}).get("consecutive_failures", 0) + 1
            
            # Parse response if possible
            health_data = {}
            try:
                health_data = response.json()
            except:
                health_data = {"raw_response": response.text[:200]}
            
            return {
                "name": service_config.name,
                "service_key": service_name,
                "status": status,
                "status_code": response.status_code,
                "response_time_ms": round(duration * 1000, 2),
                "url": service_config.health_url,
                "last_checked": datetime.utcnow().isoformat(),
                "consecutive_failures": consecutive_failures,
                "health_data": health_data
            }
            
        except httpx.TimeoutException:
            consecutive_failures = self.health_status.get(service_name, {}).get("consecutive_failures", 0) + 1
            return {
                "name": service_config.name,
                "service_key": service_name,
                "status": "timeout",
                "error": f"Health check timed out after {self.config.load_balancer.health_check_timeout}s",
                "url": service_config.health_url,
                "last_checked": datetime.utcnow().isoformat(),
                "consecutive_failures": consecutive_failures
            }
            
        except httpx.ConnectError:
            consecutive_failures = self.health_status.get(service_name, {}).get("consecutive_failures", 0) + 1
            return {
                "name": service_config.name,
                "service_key": service_name,
                "status": "unreachable",
                "error": "Cannot connect to service",
                "url": service_config.health_url,
                "last_checked": datetime.utcnow().isoformat(),
                "consecutive_failures": consecutive_failures
            }
            
        except Exception as e:
            consecutive_failures = self.health_status.get(service_name, {}).get("consecutive_failures", 0) + 1
            return {
                "name": service_config.name,
                "service_key": service_name,
                "status": "error",
                "error": str(e),
                "url": service_config.health_url,
                "last_checked": datetime.utcnow().isoformat(),
                "consecutive_failures": consecutive_failures
            }
    
    async def get_gateway_health(self) -> Dict[str, Any]:
        """Get the health status of the gateway itself."""
        
        start_time = time.time()
        
        # Basic gateway health
        gateway_health = {
            "status": "healthy",
            "service": "TracSeq API Gateway",
            "version": self.config.version,
            "environment": self.config.environment,
            "timestamp": datetime.utcnow().isoformat(),
            "uptime_seconds": int(time.time() - start_time),
        }
        
        # Add service summary
        service_summary = self._get_service_summary()
        gateway_health.update(service_summary)
        
        # Determine overall gateway status
        if service_summary["unhealthy_services"] > 0:
            gateway_health["status"] = "degraded"
        elif service_summary["total_services"] == 0:
            gateway_health["status"] = "unknown"
            
        return gateway_health
    
    async def get_all_services_health(self) -> Dict[str, Any]:
        """Get detailed health status of all services."""
        
        # Trigger immediate health check if no recent data
        if not self.health_status or self._is_health_data_stale():
            await self._check_all_services()
        
        service_summary = self._get_service_summary()
        
        return {
            "gateway": {
                "status": "healthy",
                "version": self.config.version,
                "monitoring_active": self.monitoring_active,
                "last_health_check": datetime.utcnow().isoformat()
            },
            "services": self.health_status,
            "summary": service_summary
        }
    
    def _get_service_summary(self) -> Dict[str, Any]:
        """Get summary statistics of service health."""
        
        total_services = len(self.config.services)
        healthy_services = 0
        degraded_services = 0
        unhealthy_services = 0
        
        for service_health in self.health_status.values():
            status = service_health.get("status", "unknown")
            if status == "healthy":
                healthy_services += 1
            elif status in ["degraded", "timeout"]:
                degraded_services += 1
            else:
                unhealthy_services += 1
        
        return {
            "total_services": total_services,
            "healthy_services": healthy_services,
            "degraded_services": degraded_services,
            "unhealthy_services": unhealthy_services,
            "health_percentage": round((healthy_services / total_services * 100), 1) if total_services > 0 else 0
        }
    
    def _is_health_data_stale(self) -> bool:
        """Check if health data is stale and needs refresh."""
        
        if not self.health_status:
            return True
            
        # Check if any service health data is older than 2x the check interval
        stale_threshold = datetime.utcnow() - timedelta(
            seconds=self.config.load_balancer.health_check_interval * 2
        )
        
        for service_health in self.health_status.values():
            last_checked_str = service_health.get("last_checked")
            if last_checked_str:
                try:
                    last_checked = datetime.fromisoformat(last_checked_str.replace('Z', '+00:00'))
                    if last_checked.replace(tzinfo=None) < stale_threshold:
                        return True
                except:
                    return True
            else:
                return True
                
        return False
    
    def get_service_health(self, service_name: str) -> Dict[str, Any]:
        """Get health status of a specific service."""
        
        if service_name not in self.health_status:
            return {
                "name": service_name,
                "status": "unknown",
                "error": "Service not found or health check not performed yet"
            }
            
        return self.health_status[service_name]
    
    def is_service_healthy(self, service_name: str) -> bool:
        """Check if a specific service is healthy."""
        
        service_health = self.get_service_health(service_name)
        return service_health.get("status") == "healthy" 
