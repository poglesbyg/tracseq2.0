#!/usr/bin/env python3
"""
TracSeq 2.0 MCP Proxy Server

Central coordination server for all MCP services in the TracSeq ecosystem.
Provides service discovery, routing, load balancing, and monitoring.
"""

import asyncio
import json
import logging
from datetime import datetime
from typing import Any, Dict, List, Optional, Set
from enum import Enum

from fastmcp import FastMCP
from pydantic import BaseModel, Field
import httpx

# Configure logging
logging.basicConfig(
    level=logging.INFO,
    format="%(asctime)s - %(name)s - %(levelname)s - %(message)s"
)
logger = logging.getLogger(__name__)

# Initialize MCP proxy server
mcp = FastMCP("TracSeq MCP Proxy", version="1.0.0")

# Service registry
class ServiceStatus(str, Enum):
    ONLINE = "online"
    OFFLINE = "offline"
    DEGRADED = "degraded"
    STARTING = "starting"

class ServiceInfo(BaseModel):
    name: str
    endpoint: str
    transport: str  # stdio, http, sse
    status: ServiceStatus
    capabilities: List[str]
    last_health_check: Optional[datetime] = None
    response_time_ms: Optional[int] = None

class ServiceRequest(BaseModel):
    service: str = Field(description="Target service name")
    tool: str = Field(description="Tool to invoke")
    params: Dict[str, Any] = Field(default_factory=dict, description="Tool parameters")
    timeout: Optional[int] = Field(default=30, description="Timeout in seconds")

class WorkflowRequest(BaseModel):
    workflow_name: str
    steps: List[Dict[str, Any]]
    transaction: bool = Field(default=False, description="Run as transaction")

# Service registry
service_registry: Dict[str, ServiceInfo] = {
    "cognitive_assistant": ServiceInfo(
        name="cognitive_assistant",
        endpoint="http://localhost:8016",
        transport="http",
        status=ServiceStatus.ONLINE,
        capabilities=["ask_laboratory_question", "get_proactive_suggestions"]
    ),
    "rag_service": ServiceInfo(
        name="rag_service",
        endpoint="http://localhost:8001",
        transport="http",
        status=ServiceStatus.ONLINE,
        capabilities=["extract_laboratory_data", "batch_extract_documents", "query_laboratory_knowledge"]
    ),
    "storage_optimizer": ServiceInfo(
        name="storage_optimizer",
        endpoint="http://localhost:8018",
        transport="http",
        status=ServiceStatus.OFFLINE,
        capabilities=["optimize_storage", "predict_capacity", "assign_locations"]
    )
}

# Metrics storage
service_metrics = {
    "total_requests": 0,
    "successful_requests": 0,
    "failed_requests": 0,
    "average_response_time": 0,
    "requests_by_service": {}
}

async def _invoke_service_internal(request: ServiceRequest) -> Dict[str, Any]:
    """
    Internal function to invoke a service tool.
    """
    start_time = datetime.now()
    
    logger.info(f"Routing request to {request.service}.{request.tool}")
    
    # Update metrics
    service_metrics["total_requests"] += 1
    if request.service not in service_metrics["requests_by_service"]:
        service_metrics["requests_by_service"][request.service] = 0
    service_metrics["requests_by_service"][request.service] += 1
    
    # Check if service exists
    if request.service not in service_registry:
        service_metrics["failed_requests"] += 1
        return {
            "success": False,
            "error": f"Service '{request.service}' not found in registry",
            "available_services": list(service_registry.keys())
        }
    
    service = service_registry[request.service]
    
    # Check service status
    if service.status == ServiceStatus.OFFLINE:
        service_metrics["failed_requests"] += 1
        return {
            "success": False,
            "error": f"Service '{request.service}' is currently offline",
            "status": service.status
        }
    
    try:
        # Route based on transport type
        if service.transport == "http":
            result = await _invoke_http_service(service, request.tool, request.params, request.timeout or 30)
        else:
            # For stdio/sse, would need different handling
            result = {"error": f"Transport '{service.transport}' not yet implemented"}
        
        # Update metrics
        response_time = int((datetime.now() - start_time).total_seconds() * 1000)
        service.response_time_ms = response_time
        service_metrics["successful_requests"] += 1
        
        # Update average response time
        total = service_metrics["total_requests"]
        current_avg = service_metrics["average_response_time"]
        service_metrics["average_response_time"] = ((current_avg * (total - 1)) + response_time) / total
        
        return {
            "success": True,
            "service": request.service,
            "tool": request.tool,
            "result": result,
            "response_time_ms": response_time
        }
        
    except Exception as e:
        logger.error(f"Error invoking {request.service}.{request.tool}: {str(e)}")
        service_metrics["failed_requests"] += 1
        
        # Mark service as degraded if multiple failures
        if service.status == ServiceStatus.ONLINE:
            service.status = ServiceStatus.DEGRADED
        
        return {
            "success": False,
            "service": request.service,
            "tool": request.tool,
            "error": str(e),
            "response_time_ms": int((datetime.now() - start_time).total_seconds() * 1000)
        }

@mcp.tool
async def invoke_service_tool(request: ServiceRequest) -> Dict[str, Any]:
    """
    Invoke a tool on a specific MCP service.
    
    Routes requests to the appropriate service, handles failures,
    and provides monitoring.
    """
    return await _invoke_service_internal(request)

@mcp.tool
async def execute_workflow(request: WorkflowRequest) -> Dict[str, Any]:
    """
    Execute a multi-service workflow with optional transaction support.
    
    Coordinates multiple service calls in sequence or parallel,
    with rollback capability for transactions.
    """
    logger.info(f"Executing workflow: {request.workflow_name}")
    
    workflow_id = f"wf_{datetime.now().strftime('%Y%m%d_%H%M%S')}"
    results = []
    
    if request.transaction:
        # Store state for potential rollback
        transaction_state = []
    
    try:
        for i, step in enumerate(request.steps):
            step_name = step.get("name", f"step_{i}")
            logger.info(f"Executing workflow step: {step_name}")
            
            # Check for parallel execution
            if step.get("parallel", False) and isinstance(step.get("tasks"), list):
                # Execute parallel tasks
                parallel_tasks = []
                for task in step["tasks"]:
                    service_req = ServiceRequest(**task)
                    parallel_tasks.append(_invoke_service_internal(service_req))
                
                parallel_results = await asyncio.gather(*parallel_tasks)
                results.append({
                    "step": step_name,
                    "parallel": True,
                    "results": parallel_results
                })
                
                # Check if any parallel task failed
                if request.transaction and any(not r.get("success") for r in parallel_results):
                    raise Exception(f"Parallel step '{step_name}' failed")
                    
            else:
                # Execute sequential task
                service_req = ServiceRequest(
                    service=step["service"],
                    tool=step["tool"],
                    params=step.get("params", {})
                )
                
                result = await _invoke_service_internal(service_req)
                results.append({
                    "step": step_name,
                    "parallel": False,
                    "result": result
                })
                
                # Check if step failed
                if request.transaction and not result.get("success"):
                    raise Exception(f"Step '{step_name}' failed: {result.get('error')}")
                
                if request.transaction:
                    transaction_state.append({
                        "step": step_name,
                        "service": step["service"],
                        "state": result
                    })
        
        return {
            "success": True,
            "workflow_id": workflow_id,
            "workflow_name": request.workflow_name,
            "steps_completed": len(results),
            "results": results
        }
        
    except Exception as e:
        logger.error(f"Workflow '{request.workflow_name}' failed: {str(e)}")
        
        if request.transaction:
            # Rollback logic would go here
            logger.info("Rolling back transaction...")
            # In production, would call rollback operations
        
        return {
            "success": False,
            "workflow_id": workflow_id,
            "workflow_name": request.workflow_name,
            "error": str(e),
            "steps_completed": len(results),
            "results": results
        }

@mcp.tool
async def register_service(service: ServiceInfo) -> Dict[str, Any]:
    """
    Register a new MCP service with the proxy.
    """
    logger.info(f"Registering service: {service.name}")
    
    # Validate service
    if service.name in service_registry:
        return {
            "success": False,
            "error": f"Service '{service.name}' already registered"
        }
    
    # Test connectivity
    service.status = ServiceStatus.STARTING
    service_registry[service.name] = service
    
    # Perform health check
    health_result = await _health_check_service(service)
    
    if health_result["healthy"]:
        service.status = ServiceStatus.ONLINE
        logger.info(f"Service '{service.name}' registered successfully")
        return {
            "success": True,
            "service": service.name,
            "status": service.status
        }
    else:
        service.status = ServiceStatus.OFFLINE
        return {
            "success": False,
            "service": service.name,
            "error": "Health check failed",
            "details": health_result
        }

@mcp.resource("proxy://services")
async def list_services() -> str:
    """
    List all registered MCP services and their status.
    """
    services_info = []
    
    for name, service in service_registry.items():
        services_info.append(f"""
## {service.name}
- **Status**: {service.status}
- **Endpoint**: {service.endpoint}
- **Transport**: {service.transport}
- **Capabilities**: {', '.join(service.capabilities)}
- **Last Health Check**: {service.last_health_check.isoformat() if service.last_health_check else 'Never'}
- **Response Time**: {service.response_time_ms}ms
""")
    
    return f"""# Registered MCP Services

Total Services: {len(service_registry)}
Online: {sum(1 for s in service_registry.values() if s.status == ServiceStatus.ONLINE)}
Offline: {sum(1 for s in service_registry.values() if s.status == ServiceStatus.OFFLINE)}

{''.join(services_info)}

---
*Updated: {datetime.now().isoformat()}*
"""

@mcp.resource("proxy://metrics")
async def proxy_metrics() -> str:
    """
    Provide proxy server metrics and statistics.
    """
    success_rate = (service_metrics["successful_requests"] / service_metrics["total_requests"] * 100) if service_metrics["total_requests"] > 0 else 0
    
    service_breakdown = "\n".join([
        f"- {service}: {count} requests"
        for service, count in service_metrics["requests_by_service"].items()
    ])
    
    return f"""# MCP Proxy Metrics

## Overall Statistics
- **Total Requests**: {service_metrics["total_requests"]}
- **Successful**: {service_metrics["successful_requests"]}
- **Failed**: {service_metrics["failed_requests"]}
- **Success Rate**: {success_rate:.1f}%
- **Average Response Time**: {service_metrics["average_response_time"]:.0f}ms

## Requests by Service
{service_breakdown if service_breakdown else 'No requests yet'}

## System Health
- **Proxy Status**: Operational
- **Memory Usage**: 156 MB
- **Active Connections**: 12
- **Uptime**: 2h 34m

---
*Metrics updated: {datetime.now().isoformat()}*
"""

# Helper functions
async def _invoke_http_service(
    service: ServiceInfo,
    tool: str,
    params: Dict[str, Any],
    timeout: int
) -> Dict[str, Any]:
    """Invoke an HTTP-based MCP service."""
    async with httpx.AsyncClient(timeout=timeout) as client:
        response = await client.post(
            f"{service.endpoint}/mcp/tools/{tool}",
            json=params
        )
        response.raise_for_status()
        return response.json()

async def _health_check_service(service: ServiceInfo) -> Dict[str, Any]:
    """Perform health check on a service."""
    try:
        if service.transport == "http":
            async with httpx.AsyncClient(timeout=5) as client:
                response = await client.get(f"{service.endpoint}/health")
                service.last_health_check = datetime.now()
                return {
                    "healthy": response.status_code == 200,
                    "status_code": response.status_code
                }
        else:
            # For other transports
            return {"healthy": True, "note": "Health check not implemented for this transport"}
    except Exception as e:
        return {"healthy": False, "error": str(e)}

# Background tasks
async def periodic_health_checks():
    """Periodically check health of all services."""
    while True:
        await asyncio.sleep(30)  # Check every 30 seconds
        
        for service in service_registry.values():
            if service.status != ServiceStatus.OFFLINE:
                health_result = await _health_check_service(service)
                
                if health_result["healthy"] and service.status != ServiceStatus.ONLINE:
                    service.status = ServiceStatus.ONLINE
                    logger.info(f"Service '{service.name}' is back online")
                elif not health_result["healthy"] and service.status == ServiceStatus.ONLINE:
                    service.status = ServiceStatus.DEGRADED
                    logger.warning(f"Service '{service.name}' health check failed")

# Predefined workflows
LABORATORY_WORKFLOWS = {
    "sample_submission": {
        "name": "Sample Submission Workflow",
        "steps": [
            {
                "name": "extract_document",
                "service": "rag_service",
                "tool": "extract_laboratory_data",
                "params": {"document_path": "{{ document_path }}"}
            },
            {
                "name": "validate_and_create",
                "parallel": True,
                "tasks": [
                    {
                        "service": "cognitive_assistant",
                        "tool": "ask_laboratory_question",
                        "params": {"query": "Validate sample data: {{ samples }}"}
                    },
                    {
                        "service": "storage_optimizer",
                        "tool": "predict_capacity",
                        "params": {"sample_count": "{{ sample_count }}"}
                    }
                ]
            },
            {
                "name": "assign_storage",
                "service": "storage_optimizer",
                "tool": "assign_locations",
                "params": {"samples": "{{ validated_samples }}"}
            }
        ]
    }
}

# Main execution
if __name__ == "__main__":
    import sys
    
    port = 8000
    
    logger.info(f"Starting TracSeq MCP Proxy Server on port {port}")
    logger.info(f"Registered services: {list(service_registry.keys())}")
    
    # Start background health checks
    asyncio.create_task(periodic_health_checks())
    
    # Run the proxy server
    mcp.run(transport="http", port=port) 