#!/usr/bin/env python3
"""
MCP Real-time Monitor
WebSocket client for real-time communication with MCP services.
"""

import asyncio
import json
import logging
from datetime import datetime
import websockets
from typing import Dict, Any, Optional

logging.basicConfig(level=logging.INFO)
logger = logging.getLogger(__name__)

class MCPRealtimeMonitor:
    def __init__(self, mcp_proxy_ws_url="ws://localhost:9500/ws"):
        self.ws_url = mcp_proxy_ws_url
        self.websocket: Optional[websockets.WebSocketClientProtocol] = None
        self.subscriptions = set()
        self.message_handlers = {}
        
    async def connect(self):
        """Connect to the MCP proxy WebSocket."""
        try:
            self.websocket = await websockets.connect(self.ws_url)
            logger.info(f"Connected to MCP proxy at {self.ws_url}")
            
            # Send initial handshake
            await self.send_message({
                "type": "handshake",
                "client_id": "monitor_client",
                "capabilities": ["subscribe", "publish", "query"]
            })
            
        except Exception as e:
            logger.error(f"Failed to connect: {e}")
            raise
            
    async def disconnect(self):
        """Disconnect from the WebSocket."""
        if self.websocket:
            await self.websocket.close()
            logger.info("Disconnected from MCP proxy")
            
    async def send_message(self, message: Dict[str, Any]):
        """Send a message through the WebSocket."""
        if not self.websocket:
            raise RuntimeError("Not connected to MCP proxy")
            
        await self.websocket.send(json.dumps(message))
        logger.debug(f"Sent: {message}")
        
    async def receive_messages(self):
        """Continuously receive messages from the WebSocket."""
        if not self.websocket:
            raise RuntimeError("Not connected to MCP proxy")
            
        async for message in self.websocket:
            try:
                data = json.loads(message)
                await self.handle_message(data)
            except json.JSONDecodeError:
                logger.error(f"Invalid JSON received: {message}")
            except Exception as e:
                logger.error(f"Error handling message: {e}")
                
    async def handle_message(self, data: Dict[str, Any]):
        """Handle incoming messages based on type."""
        msg_type = data.get("type")
        
        if msg_type == "event":
            await self.handle_event(data)
        elif msg_type == "response":
            await self.handle_response(data)
        elif msg_type == "error":
            logger.error(f"Error from server: {data.get('error')}")
        else:
            logger.info(f"Received: {data}")
            
        # Call custom handlers if registered
        if msg_type in self.message_handlers:
            await self.message_handlers[msg_type](data)
            
    async def handle_event(self, data: Dict[str, Any]):
        """Handle real-time events."""
        event_type = data.get("event_type")
        event_data = data.get("data", {})
        
        logger.info(f"Event: {event_type}")
        
        # Handle specific event types
        if event_type == "service_registered":
            logger.info(f"New service registered: {event_data.get('service_name')}")
        elif event_type == "service_health_changed":
            service = event_data.get("service")
            status = event_data.get("status")
            logger.warning(f"Service {service} health changed to: {status}")
        elif event_type == "sample_processed":
            sample_id = event_data.get("sample_id")
            logger.info(f"Sample {sample_id} has been processed")
            
    async def handle_response(self, data: Dict[str, Any]):
        """Handle responses to our requests."""
        request_id = data.get("request_id")
        result = data.get("result")
        
        logger.info(f"Response for request {request_id}: {result}")
        
    async def subscribe_to_events(self, event_types: list):
        """Subscribe to specific event types."""
        await self.send_message({
            "type": "subscribe",
            "event_types": event_types,
            "timestamp": datetime.now().isoformat()
        })
        
        self.subscriptions.update(event_types)
        logger.info(f"Subscribed to events: {event_types}")
        
    async def query_service_status(self, service_name: str):
        """Query the status of a specific service."""
        await self.send_message({
            "type": "query",
            "query_type": "service_status",
            "service": service_name,
            "request_id": f"query_{datetime.now().timestamp()}"
        })
        
    async def invoke_service_realtime(self, service: str, tool: str, params: Dict[str, Any]):
        """Invoke a service tool through WebSocket for real-time response."""
        request_id = f"invoke_{datetime.now().timestamp()}"
        
        await self.send_message({
            "type": "invoke",
            "request_id": request_id,
            "service": service,
            "tool": tool,
            "params": params,
            "stream": True  # Request streaming response
        })
        
        return request_id
        
    def register_handler(self, message_type: str, handler):
        """Register a custom message handler."""
        self.message_handlers[message_type] = handler
        
    async def monitor_sample_workflow(self, sample_id: str):
        """Monitor a sample through its workflow in real-time."""
        # Subscribe to sample-related events
        await self.subscribe_to_events([
            "sample_received",
            "sample_processed",
            "sample_stored",
            "analysis_complete",
            "report_generated"
        ])
        
        # Start monitoring
        await self.send_message({
            "type": "monitor",
            "entity_type": "sample",
            "entity_id": sample_id,
            "start_monitoring": True
        })
        
        logger.info(f"Started monitoring sample: {sample_id}")

# Example usage functions
async def example_real_time_monitoring():
    """Example of using the real-time monitor."""
    monitor = MCPRealtimeMonitor()
    
    try:
        # Connect to MCP proxy
        await monitor.connect()
        
        # Subscribe to all service events
        await monitor.subscribe_to_events([
            "service_registered",
            "service_health_changed",
            "workflow_started",
            "workflow_completed"
        ])
        
        # Query specific service status
        await monitor.query_service_status("sample_analyzer")
        
        # Monitor a specific sample
        await monitor.monitor_sample_workflow("SMPL-20240702-001")
        
        # Listen for messages
        await monitor.receive_messages()
        
    except KeyboardInterrupt:
        logger.info("Shutting down monitor...")
    finally:
        await monitor.disconnect()

async def example_streaming_analysis():
    """Example of streaming analysis results."""
    monitor = MCPRealtimeMonitor()
    
    # Define a handler for streaming results
    async def handle_stream(data):
        if data.get("type") == "stream_chunk":
            chunk = data.get("chunk", "")
            print(chunk, end="", flush=True)
    
    monitor.register_handler("stream_chunk", handle_stream)
    
    try:
        await monitor.connect()
        
        # Request streaming analysis
        request_id = await monitor.invoke_service_realtime(
            service="sample_analyzer",
            tool="analyze_sample_stream",
            params={
                "sample_id": "SMPL-12345",
                "stream_results": True
            }
        )
        
        # Receive streaming response
        await monitor.receive_messages()
        
    finally:
        await monitor.disconnect()

if __name__ == "__main__":
    # Run the example
    asyncio.run(example_real_time_monitoring()) 