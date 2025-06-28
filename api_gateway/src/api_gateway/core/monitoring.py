"""
Advanced Monitoring and Observability for API Gateway

Provides comprehensive metrics, distributed tracing, and health monitoring.
"""

import asyncio
import time
import psutil
import aiohttp
from datetime import datetime, timedelta
from typing import Any, Dict, List, Optional, Tuple, Callable
from dataclasses import dataclass, field
from collections import defaultdict, deque
import structlog
from prometheus_client import Counter, Histogram, Gauge, Info
from opentelemetry import trace, metrics
from opentelemetry.trace import Status, StatusCode
from opentelemetry.propagate import extract, inject
from opentelemetry.trace.propagation.tracecontext import TraceContextTextMapPropagator
import json

logger = structlog.get_logger(__name__)


# Prometheus metrics
gateway_requests = Counter(
    'gateway_requests_total',
    'Total number of requests',
    ['method', 'service', 'endpoint', 'status']
)

gateway_request_duration = Histogram(
    'gateway_request_duration_seconds',
    'Request duration in seconds',
    ['method', 'service', 'endpoint'],
    buckets=(0.005, 0.01, 0.025, 0.05, 0.1, 0.25, 0.5, 1, 2.5, 5, 10)
)

gateway_active_requests = Gauge(
    'gateway_active_requests',
    'Number of active requests',
    ['service']
)

gateway_circuit_breaker_state = Gauge(
    'gateway_circuit_breaker_state',
    'Circuit breaker state (0=closed, 1=open, 2=half-open)',
    ['service']
)

gateway_rate_limit_exceeded = Counter(
    'gateway_rate_limit_exceeded_total',
    'Number of rate limit exceeded events',
    ['service', 'user_type']
)

gateway_auth_failures = Counter(
    'gateway_auth_failures_total',
    'Number of authentication failures',
    ['reason']
)

gateway_service_health = Gauge(
    'gateway_service_health',
    'Service health status (1=healthy, 0=unhealthy)',
    ['service']
)

gateway_info = Info(
    'gateway_info',
    'Gateway information'
)


@dataclass
class ServiceMetrics:
    """Metrics for a specific service."""
    request_count: int = 0
    error_count: int = 0
    success_count: int = 0
    total_duration: float = 0.0
    response_times: deque = field(default_factory=lambda: deque(maxlen=1000))
    error_rates: deque = field(default_factory=lambda: deque(maxlen=100))
    last_error: Optional[datetime] = None
    last_success: Optional[datetime] = None
    
    def calculate_p95(self) -> float:
        """Calculate 95th percentile response time."""
        if not self.response_times:
            return 0.0
        sorted_times = sorted(self.response_times)
        index = int(len(sorted_times) * 0.95)
        return sorted_times[index] if index < len(sorted_times) else sorted_times[-1]
    
    def calculate_error_rate(self) -> float:
        """Calculate current error rate."""
        if not self.error_rates:
            return 0.0
        return sum(self.error_rates) / len(self.error_rates)


class DistributedTracer:
    """
    Distributed tracing implementation using OpenTelemetry.
    """
    
    def __init__(self, service_name: str = "api-gateway"):
        self.service_name = service_name
        self.tracer = trace.get_tracer(__name__)
        self.propagator = TraceContextTextMapPropagator()
        
    def start_span(
        self, 
        name: str, 
        context: Optional[Dict[str, str]] = None,
        attributes: Optional[Dict[str, Any]] = None
    ) -> trace.Span:
        """Start a new span with optional parent context."""
        # Extract parent context from headers if provided
        parent_context = None
        if context:
            parent_context = extract(context)
            
        # Start span with parent context
        span = self.tracer.start_span(
            name,
            context=parent_context,
            attributes=attributes or {}
        )
        
        # Add default attributes
        span.set_attribute("service.name", self.service_name)
        span.set_attribute("span.kind", "server")
        
        return span
    
    def inject_context(self, headers: Dict[str, str], span: trace.Span):
        """Inject trace context into headers for propagation."""
        context = trace.set_span_in_context(span)
        inject(headers, context=context)
    
    def record_exception(self, span: trace.Span, exception: Exception):
        """Record exception in span."""
        span.record_exception(exception)
        span.set_status(Status(StatusCode.ERROR, str(exception)))
    
    def add_event(self, span: trace.Span, name: str, attributes: Dict[str, Any]):
        """Add event to span."""
        span.add_event(name, attributes=attributes)


class HealthChecker:
    """
    Advanced health checking system with dependency monitoring.
    """
    
    def __init__(self):
        self.checks: Dict[str, Callable] = {}
        self.results: Dict[str, Dict[str, Any]] = {}
        self.check_intervals: Dict[str, int] = {}
        self._tasks: Dict[str, asyncio.Task] = {}
        
    def register_check(
        self, 
        name: str, 
        check_func: Callable, 
        interval: int = 30,
        critical: bool = True
    ):
        """Register a health check."""
        self.checks[name] = check_func
        self.check_intervals[name] = interval
        self.results[name] = {
            "status": "unknown",
            "last_check": None,
            "critical": critical,
            "details": {}
        }
        
    async def start(self):
        """Start health check tasks."""
        for name in self.checks:
            if name not in self._tasks:
                self._tasks[name] = asyncio.create_task(
                    self._run_check_loop(name)
                )
                
    async def stop(self):
        """Stop health check tasks."""
        for task in self._tasks.values():
            task.cancel()
        await asyncio.gather(*self._tasks.values(), return_exceptions=True)
        self._tasks.clear()
        
    async def _run_check_loop(self, name: str):
        """Run health check in a loop."""
        while True:
            try:
                result = await self._perform_check(name)
                self.results[name] = result
                
                # Update Prometheus metric
                gateway_service_health.labels(service=name).set(
                    1 if result["status"] == "healthy" else 0
                )
                
            except Exception as e:
                logger.error(f"Health check error for {name}", error=str(e))
                self.results[name] = {
                    "status": "error",
                    "last_check": datetime.now(),
                    "error": str(e),
                    "critical": self.results[name].get("critical", True)
                }
                
            await asyncio.sleep(self.check_intervals[name])
            
    async def _perform_check(self, name: str) -> Dict[str, Any]:
        """Perform a single health check."""
        check_func = self.checks[name]
        start_time = time.time()
        
        try:
            result = await check_func()
            duration = time.time() - start_time
            
            return {
                "status": "healthy" if result.get("healthy", False) else "unhealthy",
                "last_check": datetime.now(),
                "duration": duration,
                "details": result.get("details", {}),
                "critical": self.results[name].get("critical", True)
            }
            
        except Exception as e:
            duration = time.time() - start_time
            return {
                "status": "unhealthy",
                "last_check": datetime.now(),
                "duration": duration,
                "error": str(e),
                "critical": self.results[name].get("critical", True)
            }
            
    def get_status(self) -> Dict[str, Any]:
        """Get overall health status."""
        all_healthy = all(
            r["status"] == "healthy" 
            for r in self.results.values() 
            if r.get("critical", True)
        )
        
        return {
            "status": "healthy" if all_healthy else "unhealthy",
            "timestamp": datetime.now().isoformat(),
            "checks": self.results
        }


class MetricsCollector:
    """
    Advanced metrics collection and aggregation.
    """
    
    def __init__(self):
        self.service_metrics: Dict[str, ServiceMetrics] = defaultdict(ServiceMetrics)
        self.endpoint_metrics: Dict[str, Dict[str, Any]] = defaultdict(lambda: {
            "count": 0,
            "total_time": 0.0,
            "errors": 0,
            "status_codes": defaultdict(int)
        })
        self.user_metrics: Dict[str, Dict[str, Any]] = defaultdict(lambda: {
            "requests": 0,
            "errors": 0,
            "rate_limited": 0,
            "last_request": None
        })
        self._lock = asyncio.Lock()
        
    async def record_request(
        self,
        service: str,
        endpoint: str,
        method: str,
        duration: float,
        status_code: int,
        user_id: Optional[str] = None,
        error: Optional[str] = None
    ):
        """Record request metrics."""
        async with self._lock:
            # Service metrics
            metrics = self.service_metrics[service]
            metrics.request_count += 1
            metrics.total_duration += duration
            metrics.response_times.append(duration)
            
            if status_code >= 400:
                metrics.error_count += 1
                metrics.error_rates.append(1)
                metrics.last_error = datetime.now()
            else:
                metrics.success_count += 1
                metrics.error_rates.append(0)
                metrics.last_success = datetime.now()
                
            # Endpoint metrics
            endpoint_key = f"{method}:{endpoint}"
            endpoint_data = self.endpoint_metrics[endpoint_key]
            endpoint_data["count"] += 1
            endpoint_data["total_time"] += duration
            endpoint_data["status_codes"][status_code] += 1
            if status_code >= 400:
                endpoint_data["errors"] += 1
                
            # User metrics
            if user_id:
                user_data = self.user_metrics[user_id]
                user_data["requests"] += 1
                user_data["last_request"] = datetime.now()
                if status_code >= 400:
                    user_data["errors"] += 1
                if status_code == 429:
                    user_data["rate_limited"] += 1
                    
        # Update Prometheus metrics
        gateway_requests.labels(
            method=method,
            service=service,
            endpoint=endpoint,
            status=str(status_code)
        ).inc()
        
        gateway_request_duration.labels(
            method=method,
            service=service,
            endpoint=endpoint
        ).observe(duration)
        
    async def get_service_metrics(self, service: str) -> Dict[str, Any]:
        """Get metrics for a specific service."""
        async with self._lock:
            metrics = self.service_metrics.get(service, ServiceMetrics())
            
            return {
                "request_count": metrics.request_count,
                "error_count": metrics.error_count,
                "success_count": metrics.success_count,
                "average_response_time": (
                    metrics.total_duration / metrics.request_count 
                    if metrics.request_count > 0 else 0
                ),
                "p95_response_time": metrics.calculate_p95(),
                "error_rate": metrics.calculate_error_rate(),
                "last_error": metrics.last_error.isoformat() if metrics.last_error else None,
                "last_success": metrics.last_success.isoformat() if metrics.last_success else None
            }
            
    async def get_endpoint_metrics(self, top_n: int = 10) -> List[Dict[str, Any]]:
        """Get top N endpoints by request count."""
        async with self._lock:
            sorted_endpoints = sorted(
                self.endpoint_metrics.items(),
                key=lambda x: x[1]["count"],
                reverse=True
            )[:top_n]
            
            return [
                {
                    "endpoint": endpoint,
                    "count": data["count"],
                    "average_time": data["total_time"] / data["count"] if data["count"] > 0 else 0,
                    "error_rate": data["errors"] / data["count"] if data["count"] > 0 else 0,
                    "status_distribution": dict(data["status_codes"])
                }
                for endpoint, data in sorted_endpoints
            ]
            
    async def get_user_metrics(self, user_id: str) -> Dict[str, Any]:
        """Get metrics for a specific user."""
        async with self._lock:
            return dict(self.user_metrics.get(user_id, {}))


class SystemMonitor:
    """
    System resource monitoring.
    """
    
    def __init__(self):
        self.cpu_history = deque(maxlen=60)  # 1 minute of data
        self.memory_history = deque(maxlen=60)
        self.disk_history = deque(maxlen=60)
        self.network_history = deque(maxlen=60)
        self._monitoring_task = None
        
    async def start(self):
        """Start system monitoring."""
        self._monitoring_task = asyncio.create_task(self._monitor_loop())
        
    async def stop(self):
        """Stop system monitoring."""
        if self._monitoring_task:
            self._monitoring_task.cancel()
            await self._monitoring_task
            
    async def _monitor_loop(self):
        """Monitor system resources."""
        while True:
            try:
                # CPU usage
                cpu_percent = psutil.cpu_percent(interval=1)
                self.cpu_history.append(cpu_percent)
                
                # Memory usage
                memory = psutil.virtual_memory()
                self.memory_history.append(memory.percent)
                
                # Disk usage
                disk = psutil.disk_usage('/')
                self.disk_history.append(disk.percent)
                
                # Network I/O
                net_io = psutil.net_io_counters()
                self.network_history.append({
                    "bytes_sent": net_io.bytes_sent,
                    "bytes_recv": net_io.bytes_recv,
                    "packets_sent": net_io.packets_sent,
                    "packets_recv": net_io.packets_recv
                })
                
                await asyncio.sleep(1)
                
            except Exception as e:
                logger.error("System monitoring error", error=str(e))
                await asyncio.sleep(5)
                
    async def get_metrics(self) -> Dict[str, Any]:
        """Get current system metrics."""
        return {
            "cpu_usage": self.cpu_history[-1] if self.cpu_history else 0,
            "cpu_average": sum(self.cpu_history) / len(self.cpu_history) if self.cpu_history else 0,
            "memory_usage": self.memory_history[-1] if self.memory_history else 0,
            "memory_average": sum(self.memory_history) / len(self.memory_history) if self.memory_history else 0,
            "disk_usage": self.disk_history[-1] if self.disk_history else 0,
            "network": self.network_history[-1] if self.network_history else {}
        }
        
    def get_alerts(self) -> List[Dict[str, Any]]:
        """Get system alerts based on thresholds."""
        alerts = []
        
        # CPU alert
        if self.cpu_history and self.cpu_history[-1] > 80:
            alerts.append({
                "type": "cpu",
                "severity": "warning" if self.cpu_history[-1] < 90 else "critical",
                "message": f"High CPU usage: {self.cpu_history[-1]:.1f}%",
                "value": self.cpu_history[-1]
            })
            
        # Memory alert
        if self.memory_history and self.memory_history[-1] > 85:
            alerts.append({
                "type": "memory",
                "severity": "warning" if self.memory_history[-1] < 95 else "critical",
                "message": f"High memory usage: {self.memory_history[-1]:.1f}%",
                "value": self.memory_history[-1]
            })
            
        # Disk alert
        if self.disk_history and self.disk_history[-1] > 90:
            alerts.append({
                "type": "disk",
                "severity": "critical",
                "message": f"Low disk space: {self.disk_history[-1]:.1f}% used",
                "value": self.disk_history[-1]
            })
            
        return alerts


class MonitoringManager:
    """
    Central monitoring management system.
    """
    
    def __init__(self):
        self.tracer = DistributedTracer()
        self.health_checker = HealthChecker()
        self.metrics_collector = MetricsCollector()
        self.system_monitor = SystemMonitor()
        
        # Set gateway info
        gateway_info.info({
            'version': '2.0.0',
            'environment': 'production',
            'start_time': datetime.now().isoformat()
        })
        
    async def start(self):
        """Start all monitoring components."""
        await self.health_checker.start()
        await self.system_monitor.start()
        logger.info("Monitoring manager started")
        
    async def stop(self):
        """Stop all monitoring components."""
        await self.health_checker.stop()
        await self.system_monitor.stop()
        logger.info("Monitoring manager stopped")
        
    def start_request_span(
        self, 
        operation: str, 
        headers: Dict[str, str],
        attributes: Dict[str, Any]
    ) -> trace.Span:
        """Start a request span."""
        return self.tracer.start_span(operation, context=headers, attributes=attributes)
        
    def inject_trace_context(self, headers: Dict[str, str], span: trace.Span):
        """Inject trace context into headers."""
        self.tracer.inject_context(headers, span)
        
    async def record_request(
        self,
        service: str,
        endpoint: str,
        method: str,
        duration: float,
        status_code: int,
        user_id: Optional[str] = None,
        error: Optional[str] = None,
        span: Optional[trace.Span] = None
    ):
        """Record request metrics and tracing."""
        # Record metrics
        await self.metrics_collector.record_request(
            service, endpoint, method, duration, status_code, user_id, error
        )
        
        # Update span if provided
        if span:
            span.set_attribute("http.status_code", status_code)
            span.set_attribute("http.response.duration", duration)
            if error:
                self.tracer.record_exception(span, Exception(error))
                
    def register_health_check(
        self, 
        name: str, 
        check_func: Callable,
        interval: int = 30,
        critical: bool = True
    ):
        """Register a health check."""
        self.health_checker.register_check(name, check_func, interval, critical)
        
    async def get_health_status(self) -> Dict[str, Any]:
        """Get overall health status."""
        return self.health_checker.get_status()
        
    async def get_metrics_summary(self) -> Dict[str, Any]:
        """Get comprehensive metrics summary."""
        system_metrics = await self.system_monitor.get_metrics()
        
        # Get service metrics
        service_metrics = {}
        for service in ["auth", "samples", "storage", "templates", "sequencing", "notifications", "rag"]:
            service_metrics[service] = await self.metrics_collector.get_service_metrics(service)
            
        return {
            "timestamp": datetime.now().isoformat(),
            "system": system_metrics,
            "services": service_metrics,
            "top_endpoints": await self.metrics_collector.get_endpoint_metrics(),
            "alerts": self.system_monitor.get_alerts()
        }
        
    def update_circuit_breaker_state(self, service: str, state: int):
        """Update circuit breaker state metric."""
        gateway_circuit_breaker_state.labels(service=service).set(state)
        
    def record_rate_limit_exceeded(self, service: str, user_type: str = "authenticated"):
        """Record rate limit exceeded event."""
        gateway_rate_limit_exceeded.labels(service=service, user_type=user_type).inc()
        
    def record_auth_failure(self, reason: str):
        """Record authentication failure."""
        gateway_auth_failures.labels(reason=reason).inc()
        
    def track_active_request(self, service: str, delta: int):
        """Track active requests (increment/decrement)."""
        if delta > 0:
            gateway_active_requests.labels(service=service).inc(delta)
        else:
            gateway_active_requests.labels(service=service).dec(abs(delta))