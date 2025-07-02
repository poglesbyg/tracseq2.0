#!/usr/bin/env python3
"""
Simple MCP Proxy Server for TracSeq 2.0
A lightweight proxy that routes requests between MCP services.
"""

import asyncio
import json
import logging
from datetime import datetime
from typing import Any, Dict, List
import aiohttp
from aiohttp import web

# Configure logging
logging.basicConfig(
    level=logging.INFO,
    format="%(asctime)s - %(name)s - %(levelname)s - %(message)s"
)
logger = logging.getLogger(__name__)

# Service registry
service_registry = {
    "cognitive_assistant": {
        "name": "cognitive_assistant",
        "endpoint": "http://cognitive-assistant-mcp:9501",
        "status": "online",
        "capabilities": ["ask_laboratory_question", "get_proactive_suggestions"]
    },
    "rag_service": {
        "name": "rag_service",
        "endpoint": "http://enhanced-rag-service:8100",
        "status": "online",
        "capabilities": ["extract_laboratory_data", "query_laboratory_knowledge"]
    },
    "storage_optimizer": {
        "name": "storage_optimizer",
        "endpoint": "http://enhanced-storage-service:8005",
        "status": "online",
        "capabilities": ["optimize_storage", "predict_capacity"]
    }
}

# Metrics
metrics = {
    "total_requests": 0,
    "successful_requests": 0,
    "failed_requests": 0
}

async def handle_service_request(request):
    """Handle incoming service requests."""
    try:
        data = await request.json()
        service_name = data.get("service")
        tool = data.get("tool")
        params = data.get("params", {})
        
        logger.info(f"Routing request to {service_name}.{tool}")
        metrics["total_requests"] += 1
        
        if service_name not in service_registry:
            metrics["failed_requests"] += 1
            return web.json_response({
                "success": False,
                "error": f"Service '{service_name}' not found"
            }, status=404)
        
        service = service_registry[service_name]
        
        # Forward request to service
        async with aiohttp.ClientSession() as session:
            try:
                async with session.post(
                    f"{service['endpoint']}/mcp/tools/{tool}",
                    json=params,
                    timeout=aiohttp.ClientTimeout(total=30)
                ) as resp:
                    result = await resp.json()
                    metrics["successful_requests"] += 1
                    
                    return web.json_response({
                        "success": True,
                        "service": service_name,
                        "tool": tool,
                        "result": result
                    })
            except Exception as e:
                logger.error(f"Service call failed: {e}")
                metrics["failed_requests"] += 1
                return web.json_response({
                    "success": False,
                    "error": str(e)
                }, status=500)
                
    except Exception as e:
        logger.error(f"Request handling failed: {e}")
        return web.json_response({
            "success": False,
            "error": str(e)
        }, status=500)

async def handle_health(request):
    """Health check endpoint."""
    return web.json_response({
        "status": "healthy",
        "services": len(service_registry),
        "metrics": metrics,
        "timestamp": datetime.now().isoformat()
    })

async def handle_services(request):
    """List all registered services."""
    return web.json_response({
        "services": service_registry,
        "timestamp": datetime.now().isoformat()
    })

async def handle_websocket(request):
    """Handle WebSocket connections for real-time communication."""
    ws = web.WebSocketResponse()
    await ws.prepare(request)
    
    logger.info("WebSocket connection established")
    
    async for msg in ws:
        if msg.type == aiohttp.WSMsgType.TEXT:
            try:
                data = json.loads(msg.data)
                # Handle WebSocket messages here
                await ws.send_json({
                    "type": "response",
                    "data": f"Received: {data}"
                })
            except Exception as e:
                await ws.send_json({
                    "type": "error",
                    "error": str(e)
                })
        elif msg.type == aiohttp.WSMsgType.ERROR:
            logger.error(f"WebSocket error: {ws.exception()}")
    
    logger.info("WebSocket connection closed")
    return ws

def create_app():
    """Create the web application."""
    app = web.Application()
    
    # Add routes
    app.router.add_post('/mcp/invoke', handle_service_request)
    app.router.add_get('/health', handle_health)
    app.router.add_get('/services', handle_services)
    app.router.add_get('/ws', handle_websocket)
    
    return app

if __name__ == "__main__":
    app = create_app()
    port = 9500
    
    logger.info(f"Starting Simple MCP Proxy Server on port {port}")
    logger.info(f"Registered services: {list(service_registry.keys())}")
    
    web.run_app(app, host='0.0.0.0', port=port) 