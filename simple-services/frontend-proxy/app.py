from fastapi import FastAPI, Request, Response
from fastapi.middleware.cors import CORSMiddleware
import httpx
import uvicorn
import os
import json
import time
import uuid

app = FastAPI(title="Frontend Proxy", version="1.0.0")

# Add CORS middleware to handle frontend requests
app.add_middleware(
    CORSMiddleware,
    allow_origins=["*"],  # In production, specify your frontend domain
    allow_credentials=True,
    allow_methods=["*"],
    allow_headers=["*"],
)

# API Gateway URL
API_GATEWAY_URL = os.getenv("API_GATEWAY_URL", "http://lims-gateway:8000")

# Test users for development
TEST_USERS = {
    "admin.test@tracseq.com": {
        "id": "550e8400-e29b-41d4-a716-446655440002",
        "email": "admin.test@tracseq.com",
        "first_name": "Test",
        "last_name": "Admin",
        "role": "lab_administrator",
        "password": "password123"
    },
    "admin@tracseq.lab": {
        "id": "550e8400-e29b-41d4-a716-446655440001", 
        "email": "admin@tracseq.lab",
        "first_name": "System",
        "last_name": "Administrator",
        "role": "lab_administrator",
        "password": "password123"
    }
}

@app.get("/")
async def root():
    """Root endpoint - redirect to React frontend or provide service info"""
    return {
        "message": "TracSeq 2.0 Laboratory Management System",
        "status": "operational",
        "frontend_ui": "http://localhost:3001",
        "api_gateway": API_GATEWAY_URL,
        "available_routes": {
            "desktop": "/desktop",
            "health": "/health",
            "api": "/api/*"
        }
    }

@app.get("/favicon.ico")
async def favicon():
    """Handle favicon requests"""
    return Response(status_code=204)  # No content

@app.get("/.well-known/appspecific/com.chrome.devtools.json")
async def chrome_devtools():
    """Handle Chrome DevTools requests"""
    return Response(status_code=404)  # Not found

@app.get("/health")
async def health_check():
    """Health check endpoint"""
    return {
        "status": "healthy",
        "service": "frontend-proxy",
        "proxying_to": API_GATEWAY_URL
    }

@app.get("/desktop")
async def desktop_route():
    """Handle desktop route - temporary until real frontend is connected"""
    return {
        "message": "Desktop route - Frontend UI needs to be started",
        "status": "frontend_not_connected",
        "api_gateway": API_GATEWAY_URL,
        "available_apis": {
            "samples": "/api/samples",
            "sequencing": "/api/sequencing/jobs", 
            "dashboard": "/api/dashboard/stats",
            "spreadsheets": "/api/spreadsheets/datasets",
            "templates": "/api/templates"
        }
    }

# Special auth handling
@app.post("/api/auth/login")
async def handle_login(request: Request):
    """Handle login with fallback to test users if real auth fails"""
    body = await request.body()
    
    try:
        # First try the real auth service
        async with httpx.AsyncClient() as client:
            headers = dict(request.headers)
            headers.pop("host", None)
            headers.pop("content-length", None)
            
            response = await client.post(
                f"{API_GATEWAY_URL}/api/auth/login",
                headers=headers,
                content=body,
                timeout=10.0
            )
            
            # If real auth succeeds (200 status AND no error in response), return its response
            if response.status_code == 200:
                try:
                    response_data = response.json()
                    # Check if the response contains an error
                    if "error" not in response_data and "success" in response_data:
                        return Response(
                            content=response.content,
                            status_code=response.status_code,
                            headers=dict(response.headers),
                            media_type=response.headers.get("content-type")
                        )
                except:
                    pass  # Fall through to test auth if response parsing fails
    except:
        pass  # Fall through to test auth
    
    # Parse login request for test auth
    try:
        login_data = json.loads(body.decode())
        email = login_data.get("email")
        password = login_data.get("password")
        
        # Check test users
        if email in TEST_USERS and TEST_USERS[email]["password"] == password:
            user = TEST_USERS[email]
            
            # Create mock successful login response
            response_data = {
                "success": True,
                "data": {
                    "user": {
                        "id": user["id"],
                        "email": user["email"],
                        "first_name": user["first_name"],
                        "last_name": user["last_name"],
                        "role": user["role"]
                    },
                    "access_token": f"mock_token_{uuid.uuid4()}",
                    "expires_at": int(time.time()) + 3600,
                    "session_id": str(uuid.uuid4())
                }
            }
            
            return Response(
                content=json.dumps(response_data),
                status_code=200,
                media_type="application/json"
            )
        else:
            # Invalid credentials
            error_response = {
                "error": {
                    "code": "INVALID_CREDENTIALS",
                    "message": "Invalid email or password",
                    "timestamp": time.time()
                }
            }
            return Response(
                content=json.dumps(error_response),
                status_code=401,
                media_type="application/json"
            )
            
    except Exception as e:
        error_response = {
            "error": {
                "code": "BAD_REQUEST",
                "message": f"Invalid request: {str(e)}",
                "timestamp": time.time()
            }
        }
        return Response(
            content=json.dumps(error_response),
            status_code=400,
            media_type="application/json"
        )

@app.api_route(
    "/api/{path:path}",
    methods=["GET", "POST", "PUT", "DELETE", "PATCH", "OPTIONS", "HEAD"]
)
async def proxy_api_request(request: Request, path: str):
    """Proxy all /api/* requests to the API Gateway"""
    
    # Build target URL
    target_url = f"{API_GATEWAY_URL}/api/{path}"
    
    # Get request data
    body = await request.body()
    
    try:
        async with httpx.AsyncClient() as client:
            # Clean headers to avoid conflicts
            headers = dict(request.headers)
            headers.pop("host", None)
            headers.pop("content-length", None)
            
            response = await client.request(
                method=request.method,
                url=target_url,
                headers=headers,
                content=body,
                params=dict(request.query_params),
                timeout=30.0
            )
            
            # Clean response headers to avoid conflicts
            response_headers = dict(response.headers)
            response_headers.pop("content-length", None)
            response_headers.pop("transfer-encoding", None)
            response_headers.pop("content-encoding", None)  # Remove content-encoding to avoid decoding issues
            response_headers.pop("server", None)  # Remove duplicate server header
            response_headers.pop("date", None)  # Remove duplicate date header
            
            # Add cache-control headers to prevent browser caching issues
            response_headers["cache-control"] = "no-cache, no-store, must-revalidate"
            response_headers["pragma"] = "no-cache"
            response_headers["expires"] = "0"
            
            # httpx automatically decompresses content, so we use .content which is already decompressed
            # Return response with same status code and cleaned headers
            return Response(
                content=response.content,
                status_code=response.status_code,
                headers=response_headers,
                media_type=response.headers.get("content-type")
            )
            
    except httpx.ConnectError:
        return Response(
            content='{"error": "API Gateway unavailable"}',
            status_code=503,
            media_type="application/json"
        )
    except httpx.TimeoutException:
        return Response(
            content='{"error": "API Gateway timeout"}',
            status_code=504,
            media_type="application/json"
        )
    except Exception as e:
        return Response(
            content=f'{{"error": "Proxy error: {str(e)}"}}',
            status_code=502,
            media_type="application/json"
        )

if __name__ == "__main__":
    port = int(os.getenv("PORT", 8080))
    uvicorn.run(app, host="0.0.0.0", port=port) 