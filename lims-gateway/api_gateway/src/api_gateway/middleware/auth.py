"""
Authentication middleware for JWT validation.
"""

import jwt
import os
from typing import Optional, Dict, Any
from datetime import datetime, timezone
from fastapi import HTTPException, Request
from fastapi.security import HTTPBearer, HTTPAuthorizationCredentials

# JWT configuration
JWT_SECRET = os.getenv("JWT_SECRET", "dev-secret-key")
JWT_ALGORITHM = "HS256"

# Initialize security
security = HTTPBearer(auto_error=False)

class AuthUser:
    """Authenticated user information."""
    def __init__(self, user_id: str, email: str, name: str, role: str):
        self.id = user_id
        self.email = email
        self.name = name
        self.role = role

def decode_token(token: str) -> Optional[Dict[str, Any]]:
    """Decode and validate JWT token."""
    try:
        payload = jwt.decode(token, JWT_SECRET, algorithms=[JWT_ALGORITHM])
        # Check expiration
        if 'exp' in payload:
            if datetime.fromtimestamp(payload['exp'], tz=timezone.utc) < datetime.now(timezone.utc):
                return None
        return payload
    except jwt.InvalidTokenError:
        return None

async def get_current_user(request: Request) -> Optional[AuthUser]:
    """Extract and validate user from request."""
    auth_header = request.headers.get("Authorization", "")
    
    if not auth_header.startswith("Bearer "):
        return None
    
    token = auth_header.split(" ")[1]
    payload = decode_token(token)
    
    if not payload:
        return None
    
    return AuthUser(
        user_id=payload.get("sub", payload.get("id", "")),
        email=payload.get("email", ""),
        name=payload.get("name", ""),
        role=payload.get("role", "user")
    )

async def require_auth(request: Request) -> AuthUser:
    """Require authentication for endpoint."""
    user = await get_current_user(request)
    if not user:
        raise HTTPException(status_code=401, detail="Authentication required")
    return user

def create_token(user_data: Dict[str, Any], expires_in: int = 86400) -> str:
    """Create a JWT token for user."""
    from datetime import timedelta
    
    payload = {
        "sub": user_data.get("id"),
        "email": user_data.get("email"),
        "name": user_data.get("name"),
        "role": user_data.get("role", "user"),
        "exp": datetime.now(timezone.utc) + timedelta(seconds=expires_in)
    }
    
    return jwt.encode(payload, JWT_SECRET, algorithm=JWT_ALGORITHM) 