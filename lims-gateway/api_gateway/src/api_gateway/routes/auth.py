#!/usr/bin/env python3
"""
Authentication Routes for TracSeq 2.0 API Gateway
Centralized authentication and authorization endpoints
"""

from fastapi import APIRouter, Request, Depends, HTTPException
from fastapi.responses import JSONResponse
from pydantic import BaseModel
from typing import Dict, Any

from ..core.config import get_config, MOCK_USERS
from ..core.logging import security_logger
from ..middleware.auth import create_token, get_current_user
from ..services.proxy import service_proxy


auth_router = APIRouter()


class LoginRequest(BaseModel):
    email: str
    password: str


class LoginResponse(BaseModel):
    token: str
    user: Dict[str, Any]


@auth_router.post("/login", response_model=LoginResponse)
async def login(request: Request, login_data: LoginRequest):
    """User login endpoint with enhanced security logging"""
    
    try:
        # Get client IP for security logging
        client_ip = request.headers.get("X-Forwarded-For", "").split(",")[0].strip()
        if not client_ip:
            client_ip = request.headers.get("X-Real-IP", "")
        if not client_ip and hasattr(request, 'client'):
            client_ip = request.client.host
        
        # Check if user exists
        user = MOCK_USERS.get(login_data.email)
        
        if not user or user["password"] != login_data.password:
            # Log failed authentication attempt
            security_logger.log_auth_attempt(
                user_id=login_data.email,
                success=False,
                ip_address=client_ip
            )
            
            raise HTTPException(
                status_code=401,
                detail="Invalid credentials"
            )
        
        # Generate JWT token
        token = create_token({
            "id": user["id"],
            "email": user["email"],
            "name": user["name"],
            "role": user["role"]
        })
        
        # Log successful authentication
        security_logger.log_auth_attempt(
            user_id=user["id"],
            success=True,
            ip_address=client_ip
        )
        
        return LoginResponse(
            token=token,
            user={
                "id": user["id"],
                "email": user["email"],
                "name": user["name"],
                "role": user["role"]
            }
        )
        
    except HTTPException:
        raise
    except Exception as e:
        security_logger.log_security_event(
            event_type="LOGIN_ERROR",
            severity="medium",
            description=f"Login error: {str(e)}",
            user_id=login_data.email
        )
        raise HTTPException(
            status_code=500,
            detail="Login failed"
        )


@auth_router.get("/me")
async def get_current_user_info(current_user = Depends(get_current_user)):
    """Get current user information"""
    
    if not current_user:
        raise HTTPException(
            status_code=401,
            detail="Not authenticated"
        )
    
    return {
        "id": current_user.id,
        "email": current_user.email,
        "name": current_user.name,
        "role": current_user.role
    }


@auth_router.post("/logout")
async def logout(request: Request, current_user = Depends(get_current_user)):
    """User logout endpoint"""
    
    if current_user:
        security_logger.log_security_event(
            event_type="USER_LOGOUT",
            severity="low",
            description=f"User logged out: {current_user.email}",
            user_id=current_user.id
        )
    
    return {"message": "Logged out successfully"}


@auth_router.get("/health")
async def auth_health():
    """Authentication service health check"""
    
    config = get_config()
    auth_service_health = await service_proxy.check_service_health("auth")
    
    return {
        "status": "healthy",
        "service": "auth",
        "mock_auth_enabled": True,
        "auth_service": auth_service_health,
        "jwt_configured": bool(config.security.jwt_secret_key)
    }