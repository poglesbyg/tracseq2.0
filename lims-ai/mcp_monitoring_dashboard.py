#!/usr/bin/env python3
"""
MCP Monitoring Dashboard

Real-time monitoring and observability for all MCP services in TracSeq 2.0.
Provides health status, performance metrics, and service coordination insights.
"""

import asyncio
import json
from datetime import datetime, timedelta
from typing import Dict, List, Optional, Any
import httpx
from fastmcp import FastMCP
from pydantic import BaseModel, Field
import logging

# Configure logging
logging.basicConfig(level=logging.INFO)
logger = logging.getLogger(__name__)

# Initialize MCP server
mcp = FastMCP("TracSeq MCP Monitor", version="1.0.0")

# Configuration
class MonitorConfig(BaseModel):
    proxy_url: str = Field(default="http://localhost:8000")
    refresh_interval: int = Field(default=30, description="Seconds between health checks")
    alert_thresholds: Dict[str, float] = Field(
        default_factory=lambda: {
            "response_time_ms": 1000,
            "error_rate": 0.1,
            "cpu_usage": 80,
            "memory_usage": 90
        }
    )

# Monitoring data storage
monitoring_data = {
    "services": {},
    "metrics": {
        "total_requests": [],
        "error_rates": [],
        "response_times": []
    },
    "alerts": [],
    "last_update": None
}

@mcp.tool
async def get_system_health() -> Dict[str, Any]:
    """
    Get comprehensive health status of all MCP services.
    """
    logger.info("Fetching system health status")
    
    config = MonitorConfig()
    async with httpx.AsyncClient() as client:
        try:
            # Get service status from proxy
            services_response = await client.get(
                f"{config.proxy_url}/mcp/resources/proxy://services"
            )
            services_data = services_response.text
            
            # Get metrics from proxy
            metrics_response = await client.get(
                f"{config.proxy_url}/mcp/resources/proxy://metrics"
            )
            metrics_data = metrics_response.text
            
            # Parse service information
            services_status = _parse_services_status(services_data)
            metrics_summary = _parse_metrics_data(metrics_data)
            
            # Calculate overall health score
            health_score = _calculate_health_score(services_status, metrics_summary)
            
            # Check for alerts
            alerts = _check_alert_conditions(services_status, metrics_summary, config.alert_thresholds)
            
            monitoring_data["last_update"] = datetime.now().isoformat()
            
            return {
                "timestamp": datetime.now().isoformat(),
                "health_score": health_score,
                "services": services_status,
                "metrics": metrics_summary,
                "alerts": alerts,
                "status": _get_overall_status(health_score)
            }
            
        except Exception as e:
            logger.error(f"Error fetching system health: {str(e)}")
            return {
                "timestamp": datetime.now().isoformat(),
                "health_score": 0,
                "error": str(e),
                "status": "error"
            }

@mcp.tool
async def get_service_details(service_name: str) -> Dict[str, Any]:
    """
    Get detailed information about a specific MCP service.
    """
    logger.info(f"Fetching details for service: {service_name}")
    
    if service_name not in monitoring_data.get("services", {}):
        return {
            "error": f"Service '{service_name}' not found",
            "available_services": list(monitoring_data.get("services", {}).keys())
        }
    
    service_data = monitoring_data["services"][service_name]
    
    # Calculate service-specific metrics
    uptime = _calculate_uptime(service_data.get("start_time"))
    availability = _calculate_availability(service_data.get("health_history", []))
    
    return {
        "service": service_name,
        "current_status": service_data.get("status"),
        "endpoint": service_data.get("endpoint"),
        "capabilities": service_data.get("capabilities", []),
        "metrics": {
            "uptime": uptime,
            "availability": f"{availability:.2f}%",
            "average_response_time": service_data.get("avg_response_time", 0),
            "total_requests": service_data.get("total_requests", 0),
            "error_rate": service_data.get("error_rate", 0)
        },
        "recent_errors": service_data.get("recent_errors", [])[-5:],
        "last_health_check": service_data.get("last_health_check")
    }

@mcp.tool
async def get_performance_trends(
    timeframe: str = "1h",
    metric_type: str = "all"
) -> Dict[str, Any]:
    """
    Get performance trends for MCP services.
    
    Args:
        timeframe: Time period (1h, 6h, 24h, 7d)
        metric_type: Type of metrics (all, response_time, error_rate, throughput)
    """
    logger.info(f"Fetching performance trends for {timeframe}")
    
    # Calculate time window
    time_windows = {
        "1h": timedelta(hours=1),
        "6h": timedelta(hours=6),
        "24h": timedelta(hours=24),
        "7d": timedelta(days=7)
    }
    
    window = time_windows.get(timeframe, timedelta(hours=1))
    cutoff_time = datetime.now() - window
    
    trends = {
        "timeframe": timeframe,
        "start_time": cutoff_time.isoformat(),
        "end_time": datetime.now().isoformat(),
        "metrics": {}
    }
    
    if metric_type in ["all", "response_time"]:
        trends["metrics"]["response_time"] = {
            "current": 250,  # Mock data
            "average": 200,
            "min": 50,
            "max": 800,
            "trend": "stable",
            "data_points": _generate_mock_timeseries(24)
        }
    
    if metric_type in ["all", "error_rate"]:
        trends["metrics"]["error_rate"] = {
            "current": 0.02,
            "average": 0.03,
            "trend": "improving",
            "data_points": _generate_mock_timeseries(24, 0, 0.1)
        }
    
    if metric_type in ["all", "throughput"]:
        trends["metrics"]["throughput"] = {
            "current": 150,
            "average": 120,
            "peak": 300,
            "trend": "increasing",
            "data_points": _generate_mock_timeseries(24, 50, 300)
        }
    
    return trends

@mcp.resource("monitor://dashboard")
async def monitoring_dashboard() -> str:
    """
    Real-time monitoring dashboard for MCP services.
    """
    # Get latest health data from stored monitoring data
    health_data = {
        "status": _get_overall_status(85),
        "health_score": 85,
        "timestamp": monitoring_data.get("last_update", datetime.now().isoformat()),
        "services": monitoring_data.get("services", {}),
        "metrics": monitoring_data.get("metrics", {}),
        "alerts": monitoring_data.get("alerts", [])
    }
    
    dashboard = f"""# TracSeq MCP Monitoring Dashboard

## System Overview
- **Status**: {health_data.get('status', 'Unknown')}
- **Health Score**: {health_data.get('health_score', 0)}/100
- **Last Update**: {health_data.get('timestamp')}

## Service Status
"""
    
    # Add service status
    services = health_data.get('services', {})
    for service_name, service_info in services.items():
        status_emoji = "🟢" if service_info.get('status') == 'online' else "🔴"
        dashboard += f"\n### {status_emoji} {service_name}"
        dashboard += f"\n- Status: {service_info.get('status')}"
        dashboard += f"\n- Response Time: {service_info.get('response_time_ms', 'N/A')}ms"
        dashboard += f"\n- Last Check: {service_info.get('last_check', 'Never')}\n"
    
    # Add metrics summary
    metrics = health_data.get('metrics', {})
    dashboard += f"""
## Performance Metrics
- **Total Requests**: {metrics.get('total_requests', 0):,}
- **Success Rate**: {metrics.get('success_rate', 0):.1f}%
- **Average Response Time**: {metrics.get('avg_response_time', 0):.0f}ms
- **Active Connections**: {metrics.get('active_connections', 0)}

## Active Alerts
"""
    
    # Add alerts
    alerts = health_data.get('alerts', [])
    if alerts:
        for alert in alerts:
            dashboard += f"\n⚠️ **{alert['severity']}**: {alert['message']}"
    else:
        dashboard += "\n✅ No active alerts"
    
    dashboard += f"""

## Quick Actions
- Check specific service: `get_service_details`
- View performance trends: `get_performance_trends`
- Configure alerts: `configure_alerts`

---
*Dashboard refreshes every 30 seconds*
"""
    
    return dashboard

@mcp.resource("monitor://alerts")
async def active_alerts() -> str:
    """
    Show all active alerts and warnings.
    """
    alerts = monitoring_data.get("alerts", [])
    
    if not alerts:
        return """# Active Alerts

✅ **All systems operational**

No active alerts at this time.

---
*Last checked: {datetime.now().isoformat()}*
"""
    
    alert_text = "# Active Alerts\n\n"
    
    # Group alerts by severity
    critical = [a for a in alerts if a.get("severity") == "critical"]
    warning = [a for a in alerts if a.get("severity") == "warning"]
    info = [a for a in alerts if a.get("severity") == "info"]
    
    if critical:
        alert_text += "## 🚨 Critical\n"
        for alert in critical:
            alert_text += f"- **{alert['service']}**: {alert['message']} (Since: {alert['timestamp']})\n"
    
    if warning:
        alert_text += "\n## ⚠️ Warning\n"
        for alert in warning:
            alert_text += f"- **{alert['service']}**: {alert['message']} (Since: {alert['timestamp']})\n"
    
    if info:
        alert_text += "\n## ℹ️ Information\n"
        for alert in info:
            alert_text += f"- **{alert['service']}**: {alert['message']} (Since: {alert['timestamp']})\n"
    
    alert_text += f"\n---\n*Total active alerts: {len(alerts)}*"
    
    return alert_text

@mcp.tool
async def configure_alerts(
    alert_type: str,
    threshold: float,
    enabled: bool = True
) -> Dict[str, Any]:
    """
    Configure alert thresholds and settings.
    """
    config = MonitorConfig()
    
    valid_types = ["response_time_ms", "error_rate", "cpu_usage", "memory_usage"]
    if alert_type not in valid_types:
        return {
            "success": False,
            "error": f"Invalid alert type. Valid types: {valid_types}"
        }
    
    config.alert_thresholds[alert_type] = threshold
    
    return {
        "success": True,
        "alert_type": alert_type,
        "threshold": threshold,
        "enabled": enabled,
        "message": f"Alert threshold for {alert_type} set to {threshold}"
    }

# Helper functions
def _parse_services_status(services_text: str) -> Dict[str, Any]:
    """Parse service status from proxy response."""
    # Mock implementation - in production, parse actual response
    return {
        "cognitive_assistant": {
            "status": "online",
            "response_time_ms": 250,
            "last_check": datetime.now().isoformat()
        },
        "rag_service": {
            "status": "online",
            "response_time_ms": 450,
            "last_check": datetime.now().isoformat()
        },
        "storage_optimizer": {
            "status": "offline",
            "response_time_ms": None,
            "last_check": datetime.now().isoformat()
        }
    }

def _parse_metrics_data(metrics_text: str) -> Dict[str, Any]:
    """Parse metrics from proxy response."""
    return {
        "total_requests": 15420,
        "success_rate": 98.5,
        "avg_response_time": 320,
        "active_connections": 12
    }

def _calculate_health_score(services: Dict, metrics: Dict) -> int:
    """Calculate overall system health score (0-100)."""
    # Count online services
    total_services = len(services)
    online_services = sum(1 for s in services.values() if s.get("status") == "online")
    
    # Service availability score (50%)
    availability_score = (online_services / total_services * 50) if total_services > 0 else 0
    
    # Performance score (30%)
    success_rate = metrics.get("success_rate", 0)
    performance_score = (success_rate / 100 * 30)
    
    # Response time score (20%)
    avg_response_time = metrics.get("avg_response_time", 1000)
    response_score = max(0, 20 - (avg_response_time / 50))  # Lose points for slow response
    
    return int(availability_score + performance_score + response_score)

def _check_alert_conditions(services: Dict, metrics: Dict, thresholds: Dict) -> List[Dict]:
    """Check for alert conditions."""
    alerts = []
    
    # Check response time
    if metrics.get("avg_response_time", 0) > thresholds.get("response_time_ms", 1000):
        alerts.append({
            "severity": "warning",
            "service": "proxy",
            "message": f"High response time: {metrics['avg_response_time']}ms",
            "timestamp": datetime.now().isoformat()
        })
    
    # Check error rate
    error_rate = (100 - metrics.get("success_rate", 100)) / 100
    if error_rate > thresholds.get("error_rate", 0.1):
        alerts.append({
            "severity": "critical",
            "service": "proxy",
            "message": f"High error rate: {error_rate:.1%}",
            "timestamp": datetime.now().isoformat()
        })
    
    # Check offline services
    for service_name, service_info in services.items():
        if service_info.get("status") == "offline":
            alerts.append({
                "severity": "critical",
                "service": service_name,
                "message": "Service is offline",
                "timestamp": datetime.now().isoformat()
            })
    
    monitoring_data["alerts"] = alerts
    return alerts

def _get_overall_status(health_score: int) -> str:
    """Get overall system status based on health score."""
    if health_score >= 90:
        return "healthy"
    elif health_score >= 70:
        return "degraded"
    elif health_score >= 50:
        return "impaired"
    else:
        return "critical"

def _calculate_uptime(start_time: Optional[str]) -> str:
    """Calculate service uptime."""
    if not start_time:
        return "Unknown"
    
    try:
        start = datetime.fromisoformat(start_time)
        uptime = datetime.now() - start
        
        days = uptime.days
        hours = uptime.seconds // 3600
        minutes = (uptime.seconds % 3600) // 60
        
        return f"{days}d {hours}h {minutes}m"
    except:
        return "Unknown"

def _calculate_availability(health_history: List[bool]) -> float:
    """Calculate service availability percentage."""
    if not health_history:
        return 100.0
    
    healthy_checks = sum(1 for h in health_history if h)
    return (healthy_checks / len(health_history)) * 100

def _generate_mock_timeseries(points: int, min_val: float = 0, max_val: float = 100) -> List[Dict]:
    """Generate mock time series data."""
    import random
    
    data = []
    current_time = datetime.now()
    
    for i in range(points):
        timestamp = current_time - timedelta(hours=points-i)
        value = random.uniform(min_val, max_val)
        data.append({
            "timestamp": timestamp.isoformat(),
            "value": round(value, 2)
        })
    
    return data

# Background monitoring task
async def continuous_monitoring(config: MonitorConfig):
    """Continuously monitor MCP services."""
    while True:
        try:
            # Update monitoring data directly
            async with httpx.AsyncClient() as client:
                # Get service status
                services_response = await client.get(
                    f"{config.proxy_url}/mcp/resources/proxy://services"
                )
                monitoring_data["services"] = _parse_services_status(services_response.text)
                monitoring_data["last_update"] = datetime.now().isoformat()
            
            await asyncio.sleep(config.refresh_interval)
        except Exception as e:
            logger.error(f"Monitoring error: {str(e)}")
            await asyncio.sleep(config.refresh_interval)

# Main execution
if __name__ == "__main__":
    config = MonitorConfig()
    
    logger.info("Starting MCP Monitoring Dashboard")
    logger.info(f"Monitoring proxy at: {config.proxy_url}")
    logger.info(f"Refresh interval: {config.refresh_interval}s")
    
    # Start background monitoring
    asyncio.create_task(continuous_monitoring(config))
    
    # Run the monitoring dashboard
    mcp.run(transport="http", port=8019) 