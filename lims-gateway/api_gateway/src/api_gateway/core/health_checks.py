"""
TracSeq API Gateway - Health Check Service

Comprehensive health monitoring for all microservices in the TracSeq ecosystem.
"""

import asyncio
import logging
import time
from typing import Dict, Any, List, Optional, Callable
from dataclasses import dataclass, field
from datetime import datetime, timedelta
from enum import Enum

import httpx
from api_gateway.core.database import get_db_health_status, get_db_info

logger = logging.getLogger(__name__)


class ServiceStatus(Enum):
    """Service health status levels."""
    HEALTHY = "healthy"
    DEGRADED = "degraded"
    UNHEALTHY = "unhealthy"
    UNKNOWN = "unknown"


@dataclass
class ServiceHealthCheck:
    """Configuration for a service health check."""
    name: str
    url: str
    timeout: float = 5.0
    interval: int = 30
    critical: bool = True
    expected_status: int = 200
    custom_check: Optional[Callable] = None
    metadata: Dict[str, Any] = field(default_factory=dict)


@dataclass
class HealthCheckResult:
    """Result of a health check."""
    service_name: str
    status: ServiceStatus
    response_time: float
    timestamp: datetime
    details: Dict[str, Any] = field(default_factory=dict)
    error: Optional[str] = None


class HealthCheckService:
    """Manages health checks for all TracSeq microservices."""
    
    def __init__(self):
        self.services: Dict[str, ServiceHealthCheck] = {}
        self.results: Dict[str, HealthCheckResult] = {}
        self.history: Dict[str, List[HealthCheckResult]] = {}
        self.monitoring_tasks: Dict[str, asyncio.Task] = {}
        self.http_client: Optional[httpx.AsyncClient] = None
        self.running = False
        
        # Default service configurations
        self._setup_default_services()
    
    def _setup_default_services(self):
        """Setup default health checks for TracSeq microservices."""
        default_services = [
            ServiceHealthCheck(
                name="auth-service",
                url="http://lims-auth:8080/health",
                critical=True,
                metadata={"description": "Authentication and authorization service"}
            ),
            ServiceHealthCheck(
                name="sample-service", 
                url="http://lims-samples:8081/health",
                critical=True,
                metadata={"description": "Sample management service"}
            ),
            ServiceHealthCheck(
                name="storage-service",
                url="http://lims-storage:8082/health",
                critical=True,
                metadata={"description": "Enhanced storage service with AI features"}
            ),
            ServiceHealthCheck(
                name="template-service",
                url="http://lims-templates:8083/health",
                critical=True,
                metadata={"description": "Template management service"}
            ),
            ServiceHealthCheck(
                name="sequencing-service",
                url="http://lims-sequencing:8084/health",
                critical=True,
                metadata={"description": "Sequencing workflow service"}
            ),
            ServiceHealthCheck(
                name="notification-service",
                url="http://lims-notification:8085/health",
                critical=False,
                metadata={"description": "Multi-channel notification service"}
            ),
            ServiceHealthCheck(
                name="rag-service",
                url="http://lims-rag:8086/health",
                critical=False,
                metadata={"description": "AI document processing service"}
            ),
            ServiceHealthCheck(
                name="event-service",
                url="http://lims-events:8087/health",
                critical=False,
                metadata={"description": "Event-driven messaging service"}
            ),
            ServiceHealthCheck(
                name="transaction-service",
                url="http://lims-transactions:8088/health",
                critical=False,
                metadata={"description": "Distributed transaction service"}
            ),
        ]
        
        for service in default_services:
            self.services[service.name] = service
    
    async def start(self):
        """Start the health check service."""
        if self.running:
            return
        
        logger.info("Starting health check service...")
        
        # Initialize HTTP client
        self.http_client = httpx.AsyncClient(
            timeout=httpx.Timeout(connect=5.0, read=10.0, write=5.0, pool=5.0),
            limits=httpx.Limits(max_connections=20, max_keepalive_connections=10)
        )
        
        # Start monitoring tasks for each service
        for service_name, service_config in self.services.items():
            task = asyncio.create_task(
                self._monitor_service(service_name, service_config)
            )
            self.monitoring_tasks[service_name] = task
        
        self.running = True
        logger.info(f"Health check service started monitoring {len(self.services)} services")
    
    async def stop(self):
        """Stop the health check service."""
        if not self.running:
            return
        
        logger.info("Stopping health check service...")
        
        # Cancel all monitoring tasks
        for task in self.monitoring_tasks.values():
            task.cancel()
        
        # Wait for tasks to complete
        if self.monitoring_tasks:
            await asyncio.gather(*self.monitoring_tasks.values(), return_exceptions=True)
        
        # Close HTTP client
        if self.http_client:
            await self.http_client.aclose()
        
        self.running = False
        self.monitoring_tasks.clear()
        logger.info("Health check service stopped")
    
    async def _monitor_service(self, service_name: str, service_config: ServiceHealthCheck):
        """Monitor a single service continuously."""
        while self.running:
            try:
                # Perform health check
                result = await self._check_service_health(service_name, service_config)
                
                # Store result
                self.results[service_name] = result
                
                # Update history
                if service_name not in self.history:
                    self.history[service_name] = []
                
                self.history[service_name].append(result)
                
                # Keep only last 100 results
                if len(self.history[service_name]) > 100:
                    self.history[service_name] = self.history[service_name][-100:]
                
                # Log significant status changes
                if result.status == ServiceStatus.UNHEALTHY and service_config.critical:
                    logger.error(f"Critical service {service_name} is unhealthy: {result.error}")
                elif result.status == ServiceStatus.DEGRADED:
                    logger.warning(f"Service {service_name} is degraded: {result.details}")
                
                # Wait for next check
                await asyncio.sleep(service_config.interval)
                
            except asyncio.CancelledError:
                break
            except Exception as e:
                logger.error(f"Error monitoring service {service_name}: {e}")
                await asyncio.sleep(service_config.interval)
    
    async def _check_service_health(self, service_name: str, service_config: ServiceHealthCheck) -> HealthCheckResult:
        """Check the health of a single service."""
        start_time = time.time()
        
        try:
            if service_config.custom_check:
                # Use custom health check function
                result = await service_config.custom_check()
                response_time = time.time() - start_time
                
                return HealthCheckResult(
                    service_name=service_name,
                    status=ServiceStatus.HEALTHY if result.get("healthy", False) else ServiceStatus.UNHEALTHY,
                    response_time=response_time,
                    timestamp=datetime.now(),
                    details=result
                )
            else:
                # Use HTTP health check
                if not self.http_client:
                    raise RuntimeError("HTTP client not initialized")
                
                response = await self.http_client.get(
                    service_config.url,
                    timeout=service_config.timeout
                )
                
                response_time = time.time() - start_time
                
                # Determine status based on response
                if response.status_code == service_config.expected_status:
                    status = ServiceStatus.HEALTHY
                elif 200 <= response.status_code < 300:
                    status = ServiceStatus.HEALTHY
                elif 400 <= response.status_code < 500:
                    status = ServiceStatus.DEGRADED
                else:
                    status = ServiceStatus.UNHEALTHY
                
                # Parse response details
                details = {}
                try:
                    if response.headers.get("content-type", "").startswith("application/json"):
                        details = response.json()
                    else:
                        details = {"response": response.text[:500]}  # Truncate long responses
                except:
                    details = {"response": "Unable to parse response"}
                
                return HealthCheckResult(
                    service_name=service_name,
                    status=status,
                    response_time=response_time,
                    timestamp=datetime.now(),
                    details=details
                )
                
        except asyncio.TimeoutError:
            response_time = time.time() - start_time
            return HealthCheckResult(
                service_name=service_name,
                status=ServiceStatus.UNHEALTHY,
                response_time=response_time,
                timestamp=datetime.now(),
                error="Request timeout"
            )
        except httpx.ConnectError as e:
            response_time = time.time() - start_time
            return HealthCheckResult(
                service_name=service_name,
                status=ServiceStatus.UNHEALTHY,
                response_time=response_time,
                timestamp=datetime.now(),
                error=f"Connection error: {str(e)}"
            )
        except Exception as e:
            response_time = time.time() - start_time
            return HealthCheckResult(
                service_name=service_name,
                status=ServiceStatus.UNHEALTHY,
                response_time=response_time,
                timestamp=datetime.now(),
                error=f"Unexpected error: {str(e)}"
            )
    
    def add_service(self, service_config: ServiceHealthCheck):
        """Add a new service to monitor."""
        self.services[service_config.name] = service_config
        
        # Start monitoring if service is running
        if self.running:
            task = asyncio.create_task(
                self._monitor_service(service_config.name, service_config)
            )
            self.monitoring_tasks[service_config.name] = task
    
    def remove_service(self, service_name: str):
        """Remove a service from monitoring."""
        if service_name in self.services:
            del self.services[service_name]
        
        if service_name in self.monitoring_tasks:
            self.monitoring_tasks[service_name].cancel()
            del self.monitoring_tasks[service_name]
        
        if service_name in self.results:
            del self.results[service_name]
        
        if service_name in self.history:
            del self.history[service_name]
    
    def get_service_status(self, service_name: str) -> Optional[HealthCheckResult]:
        """Get the current status of a specific service."""
        return self.results.get(service_name)
    
    def get_all_services_status(self) -> Dict[str, HealthCheckResult]:
        """Get the current status of all services."""
        return self.results.copy()
    
    def get_system_health(self) -> Dict[str, Any]:
        """Get overall system health status."""
        if not self.results:
            return {
                "status": ServiceStatus.UNKNOWN.value,
                "message": "No health check results available",
                "services": {},
                "summary": {
                    "total": 0,
                    "healthy": 0,
                    "degraded": 0,
                    "unhealthy": 0,
                    "unknown": 0
                }
            }
        
        # Count services by status
        summary = {
            "total": len(self.results),
            "healthy": 0,
            "degraded": 0,
            "unhealthy": 0,
            "unknown": 0
        }
        
        critical_unhealthy = 0
        
        for service_name, result in self.results.items():
            summary[result.status.value] += 1
            
            # Check if critical service is unhealthy
            service_config = self.services.get(service_name)
            if service_config and service_config.critical and result.status == ServiceStatus.UNHEALTHY:
                critical_unhealthy += 1
        
        # Determine overall system status
        if critical_unhealthy > 0:
            overall_status = ServiceStatus.UNHEALTHY
            message = f"{critical_unhealthy} critical service(s) are unhealthy"
        elif summary["unhealthy"] > 0:
            overall_status = ServiceStatus.DEGRADED
            message = f"{summary['unhealthy']} non-critical service(s) are unhealthy"
        elif summary["degraded"] > 0:
            overall_status = ServiceStatus.DEGRADED
            message = f"{summary['degraded']} service(s) are degraded"
        else:
            overall_status = ServiceStatus.HEALTHY
            message = "All services are healthy"
        
        # Include database health
        db_health = get_db_health_status()
        
        return {
            "status": overall_status.value,
            "message": message,
            "timestamp": datetime.now().isoformat(),
            "database": db_health,
            "services": {
                name: {
                    "status": result.status.value,
                    "response_time": result.response_time,
                    "last_check": result.timestamp.isoformat(),
                    "error": result.error,
                    "critical": self.services.get(name, ServiceHealthCheck("", "")).critical
                }
                for name, result in self.results.items()
            },
            "summary": summary
        }
    
    def get_service_uptime(self, service_name: str, hours: int = 24) -> Dict[str, Any]:
        """Get uptime statistics for a service over the specified period."""
        if service_name not in self.history:
            return {"error": f"No history available for service {service_name}"}
        
        # Filter history by time period
        cutoff_time = datetime.now() - timedelta(hours=hours)
        recent_history = [
            result for result in self.history[service_name]
            if result.timestamp >= cutoff_time
        ]
        
        if not recent_history:
            return {"error": f"No recent history available for service {service_name}"}
        
        # Calculate uptime statistics
        total_checks = len(recent_history)
        healthy_checks = sum(1 for result in recent_history if result.status == ServiceStatus.HEALTHY)
        degraded_checks = sum(1 for result in recent_history if result.status == ServiceStatus.DEGRADED)
        unhealthy_checks = sum(1 for result in recent_history if result.status == ServiceStatus.UNHEALTHY)
        
        uptime_percentage = (healthy_checks / total_checks) * 100 if total_checks > 0 else 0
        
        # Calculate average response time
        response_times = [result.response_time for result in recent_history]
        avg_response_time = sum(response_times) / len(response_times) if response_times else 0
        
        return {
            "service_name": service_name,
            "period_hours": hours,
            "total_checks": total_checks,
            "uptime_percentage": round(uptime_percentage, 2),
            "status_breakdown": {
                "healthy": healthy_checks,
                "degraded": degraded_checks,
                "unhealthy": unhealthy_checks
            },
            "avg_response_time": round(avg_response_time, 3),
            "min_response_time": min(response_times) if response_times else 0,
            "max_response_time": max(response_times) if response_times else 0
        }


# Global health check service instance
_health_service: Optional[HealthCheckService] = None


def get_health_service() -> HealthCheckService:
    """Get the global health check service instance."""
    global _health_service
    if _health_service is None:
        _health_service = HealthCheckService()
    return _health_service


async def start_health_monitoring():
    """Start the global health monitoring service."""
    health_service = get_health_service()
    await health_service.start()


async def stop_health_monitoring():
    """Stop the global health monitoring service."""
    global _health_service
    if _health_service:
        await _health_service.stop()
        _health_service = None 