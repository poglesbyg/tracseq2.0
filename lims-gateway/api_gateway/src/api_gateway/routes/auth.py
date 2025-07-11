"""
Authentication routes for the TracSeq 2.0 API Gateway.
"""

import os
import httpx
from typing import Dict, Any
from fastapi import APIRouter, HTTPException, Request

# Import auth functions
from ..middleware.auth import create_token, get_current_user

router = APIRouter()

# Service URLs
AUTH_SERVICE_URL = os.getenv("AUTH_SERVICE_URL", "http://lims-auth:8000")

async def proxy_request(service_url: str, path: str, request: Request, timeout: float = 30.0):
    """Generic proxy function to forward requests to auth service."""
    try:
        async with httpx.AsyncClient() as client:
            # Build target URL
            target_url = f"{service_url}{path}"
            
            # Get request body if present
            body = None
            if request.method in ["POST", "PUT", "PATCH"]:
                body = await request.body()
            
            # Forward headers (excluding host)
            headers = dict(request.headers)
            headers.pop("host", None)
            headers.pop("content-length", None)
            
            # Make the request
            response = await client.request(
                method=request.method,
                url=target_url,
                headers=headers,
                params=request.query_params,
                content=body,
                timeout=timeout
            )
            
            return response.json()
            
    except httpx.ConnectError:
        raise HTTPException(status_code=503, detail=f"Auth service unavailable: {service_url}")
    except httpx.TimeoutException:
        raise HTTPException(status_code=504, detail=f"Auth service timeout: {service_url}")
    except Exception as e:
        raise HTTPException(status_code=502, detail=f"Auth service error: {str(e)}")

@router.get("/health")
async def auth_health():
    """Authentication service health check."""
    try:
        async with httpx.AsyncClient() as client:
            response = await client.get(f"{AUTH_SERVICE_URL}/health", timeout=5.0)
            return response.json()
    except Exception as e:
        raise HTTPException(status_code=503, detail=f"Auth service unavailable: {str(e)}")

@router.post("/login")
async def login(request: Request):
    """User login endpoint"""
    return await proxy_request(AUTH_SERVICE_URL, "/auth/login", request)

@router.get("/me")
async def get_current_user_endpoint(request: Request):
    """Get current user info"""
    return await proxy_request(AUTH_SERVICE_URL, "/auth/me", request)

@router.get("/users/me")
async def proxy_users_me(request: Request):
    """Proxy /api/users/me to auth service /auth/me"""
    try:
        async with httpx.AsyncClient() as client:
            url = f"{AUTH_SERVICE_URL}/auth/me"
            
            response = await client.request(
                method="GET",
                url=url,
                headers=dict(request.headers),
                timeout=30.0
            )
            return response.json()
    except Exception as e:
        raise HTTPException(status_code=502, detail=f"Auth service unavailable: {e!s}") 