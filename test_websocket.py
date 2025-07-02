
import asyncio
import websockets
import json

async def test_websocket():
    uri = "ws://localhost:9500/ws"
    try:
        async with websockets.connect(uri) as websocket:
            # Send handshake
            await websocket.send(json.dumps({
                "type": "handshake",
                "client_id": "test_client"
            }))
            
            # Receive response
            response = await websocket.recv()
            print(f"Connected! Response: {response}")
            
            return True
    except Exception as e:
        print(f"WebSocket test failed: {e}")
        return False

# Test the connection
result = asyncio.run(test_websocket())
print(f"WebSocket test: {'PASSED' if result else 'FAILED'}")

