"""
Enhanced Authentication Middleware for API Gateway

Provides JWT validation, token caching, security headers, and advanced authentication features.
"""

import asyncio
import time
import jwt
import hashlib
from datetime import datetime, timedelta
from typing import Any, Dict, List, Optional, Set, Tuple
from dataclasses import dataclass
import structlog
from fastapi import Request, Response, HTTPException
from fastapi.security import HTTPBearer, HTTPAuthorizationCredentials
from starlette.middleware.base import BaseHTTPMiddleware
from starlette.types import ASGIApp
import httpx

logger = structlog.get_logger(__name__)


@dataclass
class JWTConfig:
    """JWT configuration settings."""
    secret_key: str
    algorithm: str = "HS256"
    token_expiry: int = 3600  # 1 hour
    refresh_token_expiry: int = 604800  # 7 days
    issuer: str = "tracseq-gateway"
    audience: str = "tracseq-api"
    require_expiry: bool = True
    validate_claims: bool = True


@dataclass
class SecurityConfig:
    """Security configuration for the gateway."""
    enable_cors: bool = True
    cors_origins: List[str] = None
    enable_csrf: bool = True
    csrf_header: str = "X-CSRF-Token"
    security_headers: Dict[str, str] = None
    max_token_age: int = 86400  # 24 hours
    enable_ip_whitelist: bool = False
    ip_whitelist: Set[str] = None
    enable_api_keys: bool = True
    require_https: bool = True


class TokenCache:
    """
    High-performance token cache with TTL and LRU eviction.
    """
    
    def __init__(self, max_size: int = 10000, ttl: int = 300):
        self.max_size = max_size
        self.ttl = ttl
        self.cache: Dict[str, Tuple[Dict[str, Any], float]] = {}
        self.access_order: List[str] = []
        self._lock = asyncio.Lock()
        
    async def get(self, token: str) -> Optional[Dict[str, Any]]:
        """Get token data from cache."""
        async with self._lock:
            if token in self.cache:
                data, expiry = self.cache[token]
                if time.time() < expiry:
                    # Move to end (most recently used)
                    self.access_order.remove(token)
                    self.access_order.append(token)
                    return data
                else:
                    # Expired
                    del self.cache[token]
                    self.access_order.remove(token)
            return None
    
    async def set(self, token: str, data: Dict[str, Any]):
        """Store token data in cache."""
        async with self._lock:
            # Evict oldest if at capacity
            if len(self.cache) >= self.max_size:
                oldest = self.access_order.pop(0)
                del self.cache[oldest]
            
            self.cache[token] = (data, time.time() + self.ttl)
            self.access_order.append(token)
    
    async def invalidate(self, token: str):
        """Remove token from cache."""
        async with self._lock:
            if token in self.cache:
                del self.cache[token]
                self.access_order.remove(token)
    
    async def clear(self):
        """Clear all cached tokens."""
        async with self._lock:
            self.cache.clear()
            self.access_order.clear()


class RevokedTokenStore:
    """
    Store for revoked tokens with Redis backend support.
    """
    
    def __init__(self, redis_client=None):
        self.redis_client = redis_client
        self.local_store: Set[str] = set()
        self._lock = asyncio.Lock()
        
    async def is_revoked(self, token_id: str) -> bool:
        """Check if token is revoked."""
        if self.redis_client:
            return await self.redis_client.sismember("revoked_tokens", token_id)
        else:
            async with self._lock:
                return token_id in self.local_store
    
    async def revoke(self, token_id: str, expiry: int):
        """Add token to revoked list."""
        if self.redis_client:
            await self.redis_client.sadd("revoked_tokens", token_id)
            await self.redis_client.expire("revoked_tokens", expiry)
        else:
            async with self._lock:
                self.local_store.add(token_id)
    
    async def clear_expired(self):
        """Remove expired tokens from local store."""
        # In production, Redis handles expiry automatically
        pass


class JWTValidator:
    """
    Advanced JWT validation with multiple verification strategies.
    """
    
    def __init__(
        self, 
        config: JWTConfig,
        auth_service_url: str,
        token_cache: TokenCache,
        revoked_store: RevokedTokenStore
    ):
        self.config = config
        self.auth_service_url = auth_service_url
        self.token_cache = token_cache
        self.revoked_store = revoked_store
        self.http_client = httpx.AsyncClient(timeout=5.0)
        
    async def validate_token(self, token: str) -> Dict[str, Any]:
        """
        Validate JWT token with caching and revocation check.
        
        Returns:
            Decoded token claims if valid
            
        Raises:
            HTTPException if token is invalid
        """
        # Check cache first
        cached_data = await self.token_cache.get(token)
        if cached_data:
            return cached_data
        
        try:
            # Decode token
            claims = jwt.decode(
                token,
                self.config.secret_key,
                algorithms=[self.config.algorithm],
                audience=self.config.audience,
                issuer=self.config.issuer,
                options={
                    "verify_signature": True,
                    "verify_exp": self.config.require_expiry,
                    "verify_aud": self.config.validate_claims,
                    "verify_iss": self.config.validate_claims
                }
            )
            
            # Check if token is revoked
            token_id = claims.get("jti")
            if token_id and await self.revoked_store.is_revoked(token_id):
                raise HTTPException(status_code=401, detail="Token has been revoked")
            
            # Validate with auth service for sensitive operations
            if claims.get("requires_validation", False):
                validation_result = await self._validate_with_auth_service(token)
                if not validation_result.get("valid", False):
                    raise HTTPException(status_code=401, detail="Token validation failed")
                
                # Merge additional claims from auth service
                claims.update(validation_result.get("additional_claims", {}))
            
            # Cache the validated token
            await self.token_cache.set(token, claims)
            
            return claims
            
        except jwt.ExpiredSignatureError:
            raise HTTPException(status_code=401, detail="Token has expired")
        except jwt.InvalidAudienceError:
            raise HTTPException(status_code=401, detail="Invalid token audience")
        except jwt.InvalidIssuerError:
            raise HTTPException(status_code=401, detail="Invalid token issuer")
        except jwt.InvalidTokenError as e:
            raise HTTPException(status_code=401, detail=f"Invalid token: {str(e)}")
        except Exception as e:
            logger.error("Token validation error", error=str(e))
            raise HTTPException(status_code=500, detail="Authentication service error")
    
    async def _validate_with_auth_service(self, token: str) -> Dict[str, Any]:
        """Validate token with auth service."""
        try:
            response = await self.http_client.post(
                f"{self.auth_service_url}/api/v1/auth/validate",
                json={"token": token}
            )
            
            if response.status_code == 200:
                return response.json()
            else:
                return {"valid": False}
                
        except Exception as e:
            logger.error("Auth service validation failed", error=str(e))
            return {"valid": False}


class AuthenticationMiddleware(BaseHTTPMiddleware):
    """
    Comprehensive authentication middleware with advanced security features.
    """
    
    def __init__(
        self,
        app: ASGIApp,
        jwt_config: JWTConfig,
        security_config: SecurityConfig,
        auth_service_url: str,
        redis_client=None,
        excluded_paths: List[str] = None
    ):
        super().__init__(app)
        self.jwt_config = jwt_config
        self.security_config = security_config
        self.auth_service_url = auth_service_url
        self.redis_client = redis_client
        self.excluded_paths = excluded_paths or ["/health", "/docs", "/redoc", "/openapi.json"]
        
        # Initialize components
        self.token_cache = TokenCache()
        self.revoked_store = RevokedTokenStore(redis_client)
        self.jwt_validator = JWTValidator(
            jwt_config,
            auth_service_url,
            self.token_cache,
            self.revoked_store
        )
        
        # Security headers
        self.security_headers = security_config.security_headers or {
            "X-Content-Type-Options": "nosniff",
            "X-Frame-Options": "DENY",
            "X-XSS-Protection": "1; mode=block",
            "Strict-Transport-Security": "max-age=31536000; includeSubDomains",
            "Content-Security-Policy": "default-src 'self'",
            "Referrer-Policy": "strict-origin-when-cross-origin",
            "Permissions-Policy": "geolocation=(), microphone=(), camera=()"
        }
        
        # API key storage
        self.api_keys: Dict[str, Dict[str, Any]] = {}
        self._load_api_keys()
        
    async def dispatch(self, request: Request, call_next):
        """Process request with authentication and security checks."""
        # Skip authentication for excluded paths
        if any(request.url.path.startswith(path) for path in self.excluded_paths):
            response = await call_next(request)
            return self._add_security_headers(response)
        
        # Check HTTPS requirement
        if self.security_config.require_https and request.url.scheme != "https":
            # Allow localhost for development
            if request.client.host not in ["127.0.0.1", "localhost"]:
                raise HTTPException(status_code=403, detail="HTTPS required")
        
        # IP whitelist check
        if self.security_config.enable_ip_whitelist:
            client_ip = request.client.host
            if client_ip not in self.security_config.ip_whitelist:
                raise HTTPException(status_code=403, detail="IP not whitelisted")
        
        # Extract authentication
        auth_header = request.headers.get("Authorization")
        api_key = request.headers.get("X-API-Key")
        
        # Try API key authentication first
        if self.security_config.enable_api_keys and api_key:
            user_context = await self._validate_api_key(api_key)
            if user_context:
                request.state.user = user_context
                request.state.auth_method = "api_key"
            else:
                raise HTTPException(status_code=401, detail="Invalid API key")
        
        # Try JWT authentication
        elif auth_header and auth_header.startswith("Bearer "):
            token = auth_header[7:]  # Remove "Bearer " prefix
            
            try:
                claims = await self.jwt_validator.validate_token(token)
                
                # Create user context
                user_context = {
                    "user_id": claims.get("sub"),
                    "username": claims.get("username"),
                    "email": claims.get("email"),
                    "roles": claims.get("roles", []),
                    "permissions": claims.get("permissions", []),
                    "token_id": claims.get("jti"),
                    "issued_at": claims.get("iat"),
                    "expires_at": claims.get("exp")
                }
                
                request.state.user = user_context
                request.state.auth_method = "jwt"
                request.state.token = token
                
            except HTTPException:
                raise
            except Exception as e:
                logger.error("Authentication error", error=str(e))
                raise HTTPException(status_code=500, detail="Authentication error")
        
        else:
            # No authentication provided
            raise HTTPException(
                status_code=401, 
                detail="Authentication required",
                headers={"WWW-Authenticate": "Bearer"}
            )
        
        # CSRF protection for state-changing methods
        if self.security_config.enable_csrf and request.method in ["POST", "PUT", "DELETE", "PATCH"]:
            csrf_token = request.headers.get(self.security_config.csrf_header)
            if not csrf_token or not await self._validate_csrf_token(csrf_token, request.state.user):
                raise HTTPException(status_code=403, detail="Invalid CSRF token")
        
        # Add user context to request headers for downstream services
        request.headers.__dict__["_list"].append(
            (b"x-user-id", request.state.user["user_id"].encode())
        )
        request.headers.__dict__["_list"].append(
            (b"x-user-roles", ",".join(request.state.user.get("roles", [])).encode())
        )
        
        # Process request
        response = await call_next(request)
        
        # Add security headers
        response = self._add_security_headers(response)
        
        # Log authentication
        logger.info(
            "Request authenticated",
            user_id=request.state.user["user_id"],
            method=request.method,
            path=request.url.path,
            auth_method=request.state.auth_method
        )
        
        return response
    
    def _add_security_headers(self, response: Response) -> Response:
        """Add security headers to response."""
        for header, value in self.security_headers.items():
            response.headers[header] = value
        return response
    
    async def _validate_api_key(self, api_key: str) -> Optional[Dict[str, Any]]:
        """Validate API key and return user context."""
        # Hash the API key for comparison
        key_hash = hashlib.sha256(api_key.encode()).hexdigest()
        
        if key_hash in self.api_keys:
            key_data = self.api_keys[key_hash]
            
            # Check expiry
            if key_data.get("expires_at") and datetime.now() > key_data["expires_at"]:
                return None
            
            # Check rate limits for API key
            if self.redis_client:
                key = f"api_key_usage:{key_hash}"
                usage = await self.redis_client.incr(key)
                await self.redis_client.expire(key, 3600)  # 1 hour window
                
                if usage > key_data.get("rate_limit", 1000):
                    return None
            
            return {
                "user_id": key_data["user_id"],
                "username": key_data.get("username", "api_user"),
                "roles": key_data.get("roles", []),
                "permissions": key_data.get("permissions", []),
                "api_key_name": key_data.get("name", "Unknown")
            }
        
        return None
    
    async def _validate_csrf_token(self, token: str, user: Dict[str, Any]) -> bool:
        """Validate CSRF token."""
        if self.redis_client:
            expected_token = await self.redis_client.get(f"csrf:{user['user_id']}")
            return expected_token and expected_token.decode() == token
        
        # In development, accept any non-empty token
        return bool(token)
    
    def _load_api_keys(self):
        """Load API keys from configuration or database."""
        # In production, this would load from database
        # Example structure:
        self.api_keys = {
            # hashlib.sha256(b"test_api_key_123").hexdigest(): {
            #     "user_id": "api_user_1",
            #     "username": "test_api_user",
            #     "roles": ["api_user"],
            #     "permissions": ["read:samples", "write:samples"],
            #     "rate_limit": 1000,
            #     "expires_at": datetime.now() + timedelta(days=365)
            # }
        }


class PermissionMiddleware(BaseHTTPMiddleware):
    """
    Fine-grained permission checking middleware.
    """
    
    def __init__(
        self,
        app: ASGIApp,
        permission_config: Dict[str, List[str]]
    ):
        super().__init__(app)
        self.permission_config = permission_config
        
    async def dispatch(self, request: Request, call_next):
        """Check permissions for the request."""
        # Skip if no user context
        if not hasattr(request.state, "user"):
            return await call_next(request)
        
        # Get required permissions for endpoint
        path = request.url.path
        method = request.method
        endpoint_key = f"{method}:{path}"
        
        required_permissions = self._get_required_permissions(endpoint_key)
        
        if required_permissions:
            user_permissions = set(request.state.user.get("permissions", []))
            user_roles = set(request.state.user.get("roles", []))
            
            # Check if user has required permissions
            has_permission = False
            
            for permission in required_permissions:
                if permission in user_permissions:
                    has_permission = True
                    break
                
                # Check role-based permissions
                if permission.startswith("role:"):
                    role = permission[5:]
                    if role in user_roles:
                        has_permission = True
                        break
            
            if not has_permission:
                logger.warning(
                    "Permission denied",
                    user_id=request.state.user["user_id"],
                    required=required_permissions,
                    user_permissions=list(user_permissions),
                    endpoint=endpoint_key
                )
                raise HTTPException(
                    status_code=403,
                    detail=f"Insufficient permissions. Required: {required_permissions}"
                )
        
        return await call_next(request)
    
    def _get_required_permissions(self, endpoint_key: str) -> List[str]:
        """Get required permissions for endpoint."""
        # Direct match
        if endpoint_key in self.permission_config:
            return self.permission_config[endpoint_key]
        
        # Pattern matching (e.g., "GET:/samples/*")
        for pattern, permissions in self.permission_config.items():
            if self._match_pattern(endpoint_key, pattern):
                return permissions
        
        return []
    
    def _match_pattern(self, endpoint: str, pattern: str) -> bool:
        """Match endpoint against pattern with wildcards."""
        import fnmatch
        return fnmatch.fnmatch(endpoint, pattern)