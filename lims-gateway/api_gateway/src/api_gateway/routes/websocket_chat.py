"""
WebSocket chat endpoint for real-time communication.
"""

import json
import asyncio
from typing import Dict, Set
from datetime import datetime

from fastapi import WebSocket, WebSocketDisconnect, HTTPException
from fastapi import Depends

# Store active connections
class ConnectionManager:
    def __init__(self):
        self.active_connections: Dict[str, Set[WebSocket]] = {}
    
    async def connect(self, websocket: WebSocket, conversation_id: str):
        await websocket.accept()
        if conversation_id not in self.active_connections:
            self.active_connections[conversation_id] = set()
        self.active_connections[conversation_id].add(websocket)
    
    def disconnect(self, websocket: WebSocket, conversation_id: str):
        if conversation_id in self.active_connections:
            self.active_connections[conversation_id].discard(websocket)
            if not self.active_connections[conversation_id]:
                del self.active_connections[conversation_id]
    
    async def send_personal_message(self, message: str, websocket: WebSocket):
        await websocket.send_text(message)
    
    async def broadcast(self, message: str, conversation_id: str, exclude: WebSocket | None = None):
        if conversation_id in self.active_connections:
            for connection in self.active_connections[conversation_id]:
                if exclude is None or connection != exclude:
                    await connection.send_text(message)

# Global connection manager
manager = ConnectionManager()

async def websocket_chat_endpoint(
    websocket: WebSocket,
    conversation_id: str,
    db_pool,
    auth_user=None
):
    """WebSocket endpoint for real-time chat."""
    await manager.connect(websocket, conversation_id)
    
    try:
        # Send connection confirmation
        await websocket.send_json({
            "type": "connection",
            "status": "connected",
            "conversation_id": conversation_id,
            "user": {
                "id": auth_user.id if auth_user else "anonymous",
                "name": auth_user.name if auth_user else "Anonymous User"
            } if auth_user else None,
            "timestamp": datetime.utcnow().isoformat()
        })
        
        while True:
            # Receive message from client
            data = await websocket.receive_text()
            message_data = json.loads(data)
            
            # Handle different message types
            if message_data.get("type") == "message":
                # Store message in database
                if db_pool:
                    try:
                        async with db_pool.acquire() as conn:
                            await conn.execute("""
                                INSERT INTO chat_messages (
                                    conversation_id, role, content, metadata, created_at
                                ) VALUES ($1, $2, $3, $4, $5)
                            """, 
                            conversation_id,
                            "user",
                            message_data.get("content", ""),
                            json.dumps({
                                "user_id": auth_user.id if auth_user else "anonymous",
                                "websocket": True
                            }),
                            datetime.utcnow()
                            )
                    except Exception as e:
                        print(f"Error saving message: {e}")
                
                # Broadcast message to other participants
                broadcast_data = {
                    "type": "message",
                    "conversation_id": conversation_id,
                    "user": {
                        "id": auth_user.id if auth_user else "anonymous",
                        "name": auth_user.name if auth_user else "Anonymous User"
                    },
                    "content": message_data.get("content", ""),
                    "timestamp": datetime.utcnow().isoformat()
                }
                
                await manager.broadcast(
                    json.dumps(broadcast_data),
                    conversation_id,
                    exclude=websocket
                )
                
                # Echo back to sender with confirmation
                await websocket.send_json({
                    **broadcast_data,
                    "status": "sent"
                })
                
            elif message_data.get("type") == "typing":
                # Broadcast typing indicator
                await manager.broadcast(
                    json.dumps({
                        "type": "typing",
                        "user": {
                            "id": auth_user.id if auth_user else "anonymous",
                            "name": auth_user.name if auth_user else "Anonymous User"
                        },
                        "typing": message_data.get("typing", False)
                    }),
                    conversation_id,
                    exclude=websocket
                )
                
            elif message_data.get("type") == "ping":
                # Respond to ping
                await websocket.send_json({
                    "type": "pong",
                    "timestamp": datetime.utcnow().isoformat()
                })
                
    except WebSocketDisconnect:
        manager.disconnect(websocket, conversation_id)
        # Notify other participants
        await manager.broadcast(
            json.dumps({
                "type": "user_disconnected",
                "user": {
                    "id": auth_user.id if auth_user else "anonymous",
                    "name": auth_user.name if auth_user else "Anonymous User"
                } if auth_user else None,
                "timestamp": datetime.utcnow().isoformat()
            }),
            conversation_id
        )
    except Exception as e:
        print(f"WebSocket error: {e}")
        manager.disconnect(websocket, conversation_id) 