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

# Mock users for development
MOCK_USERS = {
    "admin@tracseq.com": {
        "id": "1",
        "email": "admin@tracseq.com",
        "name": "Admin User",
        "role": "admin",
        "password": "admin123"
    },
    "admin.test@tracseq.com": {
        "id": "1",
        "email": "admin.test@tracseq.com",
        "name": "Admin Test User",
        "role": "admin",
        "password": "admin123"
    },
    "user@tracseq.com": {
        "id": "2",
        "email": "user@tracseq.com",
        "name": "Lab User",
        "role": "user",
        "password": "user123"
    }
}

@router.get("/health")
async def auth_health():
    """Authentication service health check."""
    return {"status": "healthy", "service": "auth"}

@router.post("/login")
async def login(request: Request):
    """User login endpoint - flexible payload handling"""
    try:
        # Try to get JSON body
        body = await request.json()

        # Handle different payload formats
        email = body.get("email") or body.get("username")
        password = body.get("password")

        if not email or not password:
            raise HTTPException(status_code=400, detail="Email and password are required")

        user = MOCK_USERS.get(email)

        if not user or user["password"] != password:
            raise HTTPException(status_code=401, detail="Invalid credentials")

        # Generate JWT token
        token = create_token({
            "id": user["id"],
            "email": user["email"],
            "name": user["name"],
            "role": user["role"]
        })

        return {
            "data": {
                "token": token,
                "user": {
                    "id": user["id"],
                    "email": user["email"],
                    "name": user["name"],
                    "role": user["role"]
                }
            }
        }

    except ValueError:
        raise HTTPException(status_code=400, detail="Invalid JSON payload")
    except Exception as e:
        raise HTTPException(status_code=500, detail=f"Login error: {e!s}")

@router.get("/me")
async def get_current_user_endpoint(request: Request):
    """Get current user info"""
    user = await get_current_user(request)
    if not user:
        raise HTTPException(status_code=401, detail="Not authenticated")
    
    return {
        "id": user.id,
        "email": user.email,
        "name": user.name,
        "role": user.role
    }

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