#!/usr/bin/env python3
"""
Security Middleware for TracSeq 2.0 API Gateway
Centralized security headers and protection mechanisms
"""

import time
import hashlib
from typing import Callable, Dict, Set
from fastapi import Request, Response
from starlette.middleware.base import BaseHTTPMiddleware

from ..core.config import get_config
from ..core.logging import security_logger, main_logger
from ..core.exceptions import raise_auth_error


class SecurityMiddleware(BaseHTTPMiddleware):
    """Middleware for security headers and protection"""
    
    def __init__(self, app):
        super().__init__(app)
        self.config = get_config()
        self.suspicious_patterns = [
            # Common attack patterns
            '<script',
            'javascript:',
            'vbscript:',
            'onload=',
            'onerror=',
            'eval(',
            'document.cookie',
            'alert(',
            'confirm(',
            'prompt(',
            # SQL injection patterns
            'union select',
            'drop table',
            'insert into',
            'delete from',
            'update set',
            # Path traversal
            '../',
            '..\\',
            '/etc/passwd',
            '/etc/shadow',
            # Command injection
            '$()',
            '`',
            '&&',
            '||',
            ';',
        ]
        
        # Track suspicious activity
        self.suspicious_ips: Dict[str, Dict] = {}
        self.blocked_ips: Set[str] = set()
    
    async def dispatch(self, request: Request, call_next: Callable) -> Response:
        """Process request with security checks"""
        
        # Get client IP
        client_ip = self._get_client_ip(request)
        
        # Check if IP is blocked
        if client_ip in self.blocked_ips:
            security_logger.log_security_event(
                event_type="BLOCKED_IP_ACCESS",
                severity="high",
                description=f"Blocked IP attempted access: {client_ip}",
                client_ip=client_ip,
                request_url=str(request.url)
            )
            raise_auth_error("Access denied")
        
        # Perform security checks
        security_score = self._calculate_security_score(request)
        
        # Log suspicious activity
        if security_score > 50:
            self._track_suspicious_activity(client_ip, request, security_score)
        
        # Block if too suspicious
        if security_score > 80:
            self.blocked_ips.add(client_ip)
            security_logger.log_security_event(
                event_type="SUSPICIOUS_ACTIVITY_BLOCKED",
                severity="critical",
                description=f"Blocking IP due to suspicious activity: {client_ip}",
                client_ip=client_ip,
                request_url=str(request.url),
                security_score=security_score
            )
            raise_auth_error("Access denied due to suspicious activity")
        
        # Process request
        response = await call_next(request)
        
        # Add security headers
        self._add_security_headers(response)
        
        return response
    
    def _get_client_ip(self, request: Request) -> str:
        """Extract client IP address from request"""
        # Check for forwarded headers first
        forwarded_for = request.headers.get("X-Forwarded-For")
        if forwarded_for:
            return forwarded_for.split(",")[0].strip()
        
        real_ip = request.headers.get("X-Real-IP")
        if real_ip:
            return real_ip
        
        # Fallback to client host
        if hasattr(request, 'client') and request.client:
            return request.client.host
        
        return "unknown"
    
    def _calculate_security_score(self, request: Request) -> int:
        """Calculate security risk score for request"""
        score = 0
        
        # Check URL for suspicious patterns
        url_lower = str(request.url).lower()
        for pattern in self.suspicious_patterns:
            if pattern in url_lower:
                score += 20
        
        # Check headers for suspicious content
        for header_name, header_value in request.headers.items():
            header_lower = header_value.lower()
            for pattern in self.suspicious_patterns:
                if pattern in header_lower:
                    score += 15
        
        # Check user agent
        user_agent = request.headers.get("User-Agent", "").lower()
        if not user_agent or len(user_agent) < 10:
            score += 10
        
        # Check for common bot patterns
        bot_patterns = ['bot', 'crawler', 'spider', 'scraper']
        if any(pattern in user_agent for pattern in bot_patterns):
            score += 5
        
        # Check for unusual request methods
        if request.method not in ['GET', 'POST', 'PUT', 'DELETE', 'PATCH', 'OPTIONS']:
            score += 10
        
        # Check for missing common headers
        if not request.headers.get("Accept"):
            score += 5
        if not request.headers.get("Accept-Language"):
            score += 5
        
        # Check for suspicious query parameters
        for param_name, param_value in request.query_params.items():
            param_lower = param_value.lower()
            for pattern in self.suspicious_patterns:
                if pattern in param_lower:
                    score += 15
        
        return min(score, 100)  # Cap at 100
    
    def _track_suspicious_activity(self, client_ip: str, request: Request, score: int):
        """Track suspicious activity for IP"""
        current_time = time.time()
        
        if client_ip not in self.suspicious_ips:
            self.suspicious_ips[client_ip] = {
                'first_seen': current_time,
                'last_seen': current_time,
                'request_count': 0,
                'total_score': 0,
                'max_score': 0,
                'suspicious_urls': []
            }
        
        ip_data = self.suspicious_ips[client_ip]
        ip_data['last_seen'] = current_time
        ip_data['request_count'] += 1
        ip_data['total_score'] += score
        ip_data['max_score'] = max(ip_data['max_score'], score)
        
        # Track suspicious URLs
        if len(ip_data['suspicious_urls']) < 10:
            ip_data['suspicious_urls'].append(str(request.url))
        
        # Log suspicious activity
        security_logger.log_security_event(
            event_type="SUSPICIOUS_ACTIVITY",
            severity="medium",
            description=f"Suspicious activity detected from {client_ip}",
            client_ip=client_ip,
            request_url=str(request.url),
            security_score=score,
            total_requests=ip_data['request_count'],
            average_score=ip_data['total_score'] / ip_data['request_count']
        )
    
    def _add_security_headers(self, response: Response):
        """Add security headers to response"""
        
        # Content Security Policy
        csp_policy = (
            "default-src 'self'; "
            "script-src 'self' 'unsafe-inline' 'unsafe-eval'; "
            "style-src 'self' 'unsafe-inline'; "
            "img-src 'self' data: https:; "
            "font-src 'self' data:; "
            "connect-src 'self' ws: wss:; "
            "frame-ancestors 'none';"
        )
        response.headers["Content-Security-Policy"] = csp_policy
        
        # Security headers
        response.headers["X-Content-Type-Options"] = "nosniff"
        response.headers["X-Frame-Options"] = "DENY"
        response.headers["X-XSS-Protection"] = "1; mode=block"
        response.headers["Referrer-Policy"] = "strict-origin-when-cross-origin"
        response.headers["Permissions-Policy"] = "geolocation=(), microphone=(), camera=()"
        
        # HSTS (only for HTTPS)
        if not self.config.is_development:
            response.headers["Strict-Transport-Security"] = "max-age=31536000; includeSubDomains"
        
        # Remove server information
        response.headers.pop("Server", None)
        
        # Add custom security header
        response.headers["X-TracSeq-Security"] = "enabled"


class CSRFProtectionMiddleware(BaseHTTPMiddleware):
    """Middleware for CSRF protection"""
    
    def __init__(self, app, secret_key: str = None):
        super().__init__(app)
        self.config = get_config()
        self.secret_key = secret_key or self.config.security.jwt_secret_key
        self.safe_methods = {'GET', 'HEAD', 'OPTIONS', 'TRACE'}
        self.csrf_header_name = 'X-CSRF-Token'
        self.csrf_cookie_name = 'csrf_token'
    
    async def dispatch(self, request: Request, call_next: Callable) -> Response:
        """Process request with CSRF protection"""
        
        # Skip CSRF protection for safe methods
        if request.method in self.safe_methods:
            response = await call_next(request)
            # Add CSRF token to response for future requests
            csrf_token = self._generate_csrf_token()
            response.set_cookie(
                self.csrf_cookie_name,
                csrf_token,
                httponly=True,
                secure=not self.config.is_development,
                samesite='strict'
            )
            return response
        
        # Check CSRF token for unsafe methods
        csrf_token = request.headers.get(self.csrf_header_name)
        csrf_cookie = request.cookies.get(self.csrf_cookie_name)
        
        if not csrf_token or not csrf_cookie:
            security_logger.log_security_event(
                event_type="CSRF_TOKEN_MISSING",
                severity="medium",
                description="CSRF token missing from request",
                client_ip=self._get_client_ip(request),
                request_url=str(request.url)
            )
            raise_auth_error("CSRF token missing")
        
        if not self._validate_csrf_token(csrf_token, csrf_cookie):
            security_logger.log_security_event(
                event_type="CSRF_TOKEN_INVALID",
                severity="high",
                description="Invalid CSRF token in request",
                client_ip=self._get_client_ip(request),
                request_url=str(request.url)
            )
            raise_auth_error("Invalid CSRF token")
        
        # Process request
        response = await call_next(request)
        
        return response
    
    def _generate_csrf_token(self) -> str:
        """Generate CSRF token"""
        import secrets
        return secrets.token_urlsafe(32)
    
    def _validate_csrf_token(self, header_token: str, cookie_token: str) -> bool:
        """Validate CSRF token"""
        # Simple validation - header and cookie should match
        # In production, you might want more sophisticated validation
        return header_token == cookie_token
    
    def _get_client_ip(self, request: Request) -> str:
        """Extract client IP address from request"""
        forwarded_for = request.headers.get("X-Forwarded-For")
        if forwarded_for:
            return forwarded_for.split(",")[0].strip()
        
        real_ip = request.headers.get("X-Real-IP")
        if real_ip:
            return real_ip
        
        if hasattr(request, 'client') and request.client:
            return request.client.host
        
        return "unknown"