#!/usr/bin/env python3
"""
TracSeq 2.0 - Security Testing Framework
Tests authentication, authorization, JWT security, and vulnerability scanning
"""

import asyncio
import aiohttp
import json
import time
import uuid
import base64
import hashlib
import hmac
from datetime import datetime, timedelta
from typing import Dict, List, Optional, Any, Tuple
from dataclasses import dataclass, asdict
import argparse
import jwt
import secrets
import string

# Security test configuration
SECURITY_TEST_CONFIG = {
    'api_gateway_url': 'http://localhost:8089',
    'frontend_proxy_url': 'http://localhost:8085',
    'auth_endpoints': {
        'login': '/api/auth/login',
        'register': '/api/auth/register',
        'logout': '/api/auth/logout',
        'refresh': '/api/auth/refresh',
        'profile': '/api/auth/profile'
    },
    'protected_endpoints': [
        '/api/samples/v1/samples',
        '/api/dashboard/v1/users',
        '/api/sequencing/v1/jobs',
        '/api/spreadsheet/v1/templates'
    ],
    'test_users': {
        'admin': {'username': 'admin', 'password': 'admin123', 'role': 'admin'},
        'user': {'username': 'testuser', 'password': 'user123', 'role': 'user'},
        'readonly': {'username': 'readonly', 'password': 'readonly123', 'role': 'readonly'}
    },
    'jwt_secret': 'test-secret-key-for-testing',
    'session_timeout': 3600,  # 1 hour
    'max_login_attempts': 5,
    'rate_limit_requests': 100,
    'rate_limit_window': 60  # 1 minute
}

@dataclass
class SecurityTestResult:
    """Security test result data structure"""
    test_name: str
    start_time: datetime
    end_time: datetime
    duration: float
    total_tests: int
    passed_tests: int
    failed_tests: int
    vulnerabilities: List[str]
    security_issues: List[str]
    recommendations: List[str]
    test_details: Dict[str, Any]

@dataclass
class AuthenticationResult:
    """Authentication test result"""
    success: bool
    token: Optional[str]
    user_info: Optional[Dict[str, Any]]
    error: Optional[str]
    response_time: float

class SecurityTester:
    """Comprehensive security testing framework"""
    
    def __init__(self, config: Dict[str, Any]):
        self.config = config
        self.session_tokens = {}
        self.test_results = []
        
    async def make_request(self, session: aiohttp.ClientSession, method: str, url: str, 
                          data: Optional[Dict] = None, headers: Optional[Dict] = None,
                          auth_token: Optional[str] = None) -> Tuple[int, Dict[str, Any], float]:
        """Make HTTP request with security context"""
        start_time = time.time()
        
        # Add authorization header if token provided
        if auth_token:
            if not headers:
                headers = {}
            headers['Authorization'] = f'Bearer {auth_token}'
            
        try:
            if method.upper() == 'GET':
                async with session.get(url, headers=headers) as response:
                    response_time = time.time() - start_time
                    try:
                        response_data = await response.json()
                    except:
                        response_data = {'text': await response.text()}
                    return response.status, response_data, response_time
                    
            elif method.upper() == 'POST':
                async with session.post(url, json=data, headers=headers) as response:
                    response_time = time.time() - start_time
                    try:
                        response_data = await response.json()
                    except:
                        response_data = {'text': await response.text()}
                    return response.status, response_data, response_time
                    
            elif method.upper() == 'PUT':
                async with session.put(url, json=data, headers=headers) as response:
                    response_time = time.time() - start_time
                    try:
                        response_data = await response.json()
                    except:
                        response_data = {'text': await response.text()}
                    return response.status, response_data, response_time
                    
            elif method.upper() == 'DELETE':
                async with session.delete(url, headers=headers) as response:
                    response_time = time.time() - start_time
                    try:
                        response_data = await response.json()
                    except:
                        response_data = {'text': await response.text()}
                    return response.status, response_data, response_time
                    
        except Exception as e:
            response_time = time.time() - start_time
            return 0, {'error': str(e)}, response_time
            
    async def test_authentication_flow(self) -> SecurityTestResult:
        """Test authentication flow including login, logout, and token validation"""
        print("🔐 Running Authentication Flow Tests...")
        
        start_time = datetime.utcnow()
        vulnerabilities = []
        security_issues = []
        recommendations = []
        test_details = {}
        total_tests = 0
        passed_tests = 0
        
        async with aiohttp.ClientSession() as session:
            # Test 1: Valid login
            print("  Test 1: Valid user login...")
            total_tests += 1
            
            login_data = {
                'username': self.config['test_users']['user']['username'],
                'password': self.config['test_users']['user']['password']
            }
            
            status, response, response_time = await self.make_request(
                session, 'POST', 
                f"{self.config['api_gateway_url']}{self.config['auth_endpoints']['login']}", 
                login_data
            )
            
            if status == 200 and 'token' in response:
                passed_tests += 1
                self.session_tokens['user'] = response['token']
                test_details['valid_login'] = {
                    'status': 'PASS',
                    'response_time': response_time,
                    'token_received': True
                }
                print(f"    ✅ Valid login successful ({response_time:.3f}s)")
            else:
                security_issues.append("Valid login failed - authentication system may be broken")
                test_details['valid_login'] = {
                    'status': 'FAIL',
                    'response_time': response_time,
                    'error': response.get('error', 'Unknown error')
                }
                print(f"    ❌ Valid login failed: {response.get('error', 'Unknown error')}")
                
            # Test 2: Invalid credentials
            print("  Test 2: Invalid credentials...")
            total_tests += 1
            
            invalid_login_data = {
                'username': 'invalid_user',
                'password': 'wrong_password'
            }
            
            status, response, response_time = await self.make_request(
                session, 'POST', 
                f"{self.config['api_gateway_url']}{self.config['auth_endpoints']['login']}", 
                invalid_login_data
            )
            
            if status == 401 or status == 403:
                passed_tests += 1
                test_details['invalid_login'] = {
                    'status': 'PASS',
                    'response_time': response_time,
                    'properly_rejected': True
                }
                print(f"    ✅ Invalid credentials properly rejected ({response_time:.3f}s)")
            else:
                vulnerabilities.append("Invalid credentials not properly rejected - potential security vulnerability")
                test_details['invalid_login'] = {
                    'status': 'FAIL',
                    'response_time': response_time,
                    'unexpected_status': status
                }
                print(f"    ❌ Invalid credentials not properly rejected (status: {status})")
                
            # Test 3: SQL Injection attempt
            print("  Test 3: SQL injection protection...")
            total_tests += 1
            
            injection_data = {
                'username': "admin'; DROP TABLE users; --",
                'password': "' OR '1'='1"
            }
            
            status, response, response_time = await self.make_request(
                session, 'POST', 
                f"{self.config['api_gateway_url']}{self.config['auth_endpoints']['login']}", 
                injection_data
            )
            
            if status == 401 or status == 403 or status == 400:
                passed_tests += 1
                test_details['sql_injection'] = {
                    'status': 'PASS',
                    'response_time': response_time,
                    'injection_blocked': True
                }
                print(f"    ✅ SQL injection attempt blocked ({response_time:.3f}s)")
            else:
                vulnerabilities.append("SQL injection not properly blocked - critical security vulnerability")
                test_details['sql_injection'] = {
                    'status': 'FAIL',
                    'response_time': response_time,
                    'injection_not_blocked': True
                }
                print(f"    ❌ SQL injection not properly blocked (status: {status})")
                
            # Test 4: Password brute force protection
            print("  Test 4: Brute force protection...")
            total_tests += 1
            
            brute_force_attempts = 0
            max_attempts = self.config['max_login_attempts']
            
            for attempt in range(max_attempts + 2):
                brute_force_data = {
                    'username': self.config['test_users']['user']['username'],
                    'password': f'wrong_password_{attempt}'
                }
                
                status, response, response_time = await self.make_request(
                    session, 'POST', 
                    f"{self.config['api_gateway_url']}{self.config['auth_endpoints']['login']}", 
                    brute_force_data
                )
                
                brute_force_attempts += 1
                
                # After max attempts, should be rate limited
                if attempt >= max_attempts and status == 429:
                    passed_tests += 1
                    test_details['brute_force'] = {
                        'status': 'PASS',
                        'attempts': brute_force_attempts,
                        'rate_limited': True
                    }
                    print(f"    ✅ Brute force protection active after {brute_force_attempts} attempts")
                    break
                    
                await asyncio.sleep(0.1)  # Small delay between attempts
                
            else:
                vulnerabilities.append("Brute force protection not implemented - security vulnerability")
                test_details['brute_force'] = {
                    'status': 'FAIL',
                    'attempts': brute_force_attempts,
                    'rate_limited': False
                }
                print(f"    ❌ Brute force protection not active after {brute_force_attempts} attempts")
                
            # Test 5: Token validation
            print("  Test 5: JWT token validation...")
            total_tests += 1
            
            if 'user' in self.session_tokens:
                # Test with valid token
                status, response, response_time = await self.make_request(
                    session, 'GET', 
                    f"{self.config['api_gateway_url']}{self.config['auth_endpoints']['profile']}", 
                    auth_token=self.session_tokens['user']
                )
                
                if status == 200:
                    passed_tests += 1
                    test_details['token_validation'] = {
                        'status': 'PASS',
                        'response_time': response_time,
                        'valid_token_accepted': True
                    }
                    print(f"    ✅ Valid token accepted ({response_time:.3f}s)")
                else:
                    security_issues.append("Valid JWT token not accepted")
                    test_details['token_validation'] = {
                        'status': 'FAIL',
                        'response_time': response_time,
                        'valid_token_rejected': True
                    }
                    print(f"    ❌ Valid token rejected (status: {status})")
            else:
                security_issues.append("No valid token available for testing")
                test_details['token_validation'] = {
                    'status': 'FAIL',
                    'error': 'No valid token available'
                }
                print(f"    ❌ No valid token available for testing")
                
            # Test 6: Invalid token handling
            print("  Test 6: Invalid token handling...")
            total_tests += 1
            
            invalid_token = "invalid.jwt.token"
            
            status, response, response_time = await self.make_request(
                session, 'GET', 
                f"{self.config['api_gateway_url']}{self.config['auth_endpoints']['profile']}", 
                auth_token=invalid_token
            )
            
            if status == 401 or status == 403:
                passed_tests += 1
                test_details['invalid_token'] = {
                    'status': 'PASS',
                    'response_time': response_time,
                    'invalid_token_rejected': True
                }
                print(f"    ✅ Invalid token properly rejected ({response_time:.3f}s)")
            else:
                vulnerabilities.append("Invalid JWT token not properly rejected")
                test_details['invalid_token'] = {
                    'status': 'FAIL',
                    'response_time': response_time,
                    'invalid_token_accepted': True
                }
                print(f"    ❌ Invalid token not properly rejected (status: {status})")
                
        end_time = datetime.utcnow()
        
        # Add recommendations based on findings
        if vulnerabilities:
            recommendations.append("Implement proper input validation and sanitization")
            recommendations.append("Add rate limiting for authentication endpoints")
            recommendations.append("Implement proper JWT token validation")
            
        if security_issues:
            recommendations.append("Review authentication system implementation")
            recommendations.append("Ensure proper error handling for security scenarios")
            
        return SecurityTestResult(
            test_name="Authentication Flow",
            start_time=start_time,
            end_time=end_time,
            duration=(end_time - start_time).total_seconds(),
            total_tests=total_tests,
            passed_tests=passed_tests,
            failed_tests=total_tests - passed_tests,
            vulnerabilities=vulnerabilities,
            security_issues=security_issues,
            recommendations=recommendations,
            test_details=test_details
        )
        
    async def test_authorization_rbac(self) -> SecurityTestResult:
        """Test role-based access control and authorization"""
        print("🔒 Running Authorization and RBAC Tests...")
        
        start_time = datetime.utcnow()
        vulnerabilities = []
        security_issues = []
        recommendations = []
        test_details = {}
        total_tests = 0
        passed_tests = 0
        
        async with aiohttp.ClientSession() as session:
            # First, authenticate different user roles
            user_tokens = {}
            
            for role, user_data in self.config['test_users'].items():
                login_data = {
                    'username': user_data['username'],
                    'password': user_data['password']
                }
                
                status, response, response_time = await self.make_request(
                    session, 'POST', 
                    f"{self.config['api_gateway_url']}{self.config['auth_endpoints']['login']}", 
                    login_data
                )
                
                if status == 200 and 'token' in response:
                    user_tokens[role] = response['token']
                    
            # Test 1: Admin access to protected endpoints
            print("  Test 1: Admin access to protected endpoints...")
            total_tests += 1
            
            if 'admin' in user_tokens:
                admin_access_results = []
                
                for endpoint in self.config['protected_endpoints']:
                    status, response, response_time = await self.make_request(
                        session, 'GET', 
                        f"{self.config['api_gateway_url']}{endpoint}", 
                        auth_token=user_tokens['admin']
                    )
                    
                    admin_access_results.append({
                        'endpoint': endpoint,
                        'status': status,
                        'response_time': response_time,
                        'access_granted': status == 200
                    })
                    
                successful_access = sum(1 for r in admin_access_results if r['access_granted'])
                
                if successful_access == len(self.config['protected_endpoints']):
                    passed_tests += 1
                    test_details['admin_access'] = {
                        'status': 'PASS',
                        'endpoints_accessed': successful_access,
                        'total_endpoints': len(self.config['protected_endpoints']),
                        'results': admin_access_results
                    }
                    print(f"    ✅ Admin access granted to all {successful_access} endpoints")
                else:
                    security_issues.append("Admin user cannot access all protected endpoints")
                    test_details['admin_access'] = {
                        'status': 'FAIL',
                        'endpoints_accessed': successful_access,
                        'total_endpoints': len(self.config['protected_endpoints']),
                        'results': admin_access_results
                    }
                    print(f"    ❌ Admin access denied to some endpoints ({successful_access}/{len(self.config['protected_endpoints'])})")
            else:
                security_issues.append("Admin user authentication failed")
                test_details['admin_access'] = {
                    'status': 'FAIL',
                    'error': 'Admin authentication failed'
                }
                print(f"    ❌ Admin user authentication failed")
                
            # Test 2: Regular user access restrictions
            print("  Test 2: Regular user access restrictions...")
            total_tests += 1
            
            if 'user' in user_tokens:
                user_access_results = []
                
                for endpoint in self.config['protected_endpoints']:
                    status, response, response_time = await self.make_request(
                        session, 'GET', 
                        f"{self.config['api_gateway_url']}{endpoint}", 
                        auth_token=user_tokens['user']
                    )
                    
                    user_access_results.append({
                        'endpoint': endpoint,
                        'status': status,
                        'response_time': response_time,
                        'access_granted': status == 200
                    })
                    
                # Regular users should have limited access
                restricted_access = sum(1 for r in user_access_results if not r['access_granted'])
                
                if restricted_access > 0:
                    passed_tests += 1
                    test_details['user_access'] = {
                        'status': 'PASS',
                        'endpoints_restricted': restricted_access,
                        'total_endpoints': len(self.config['protected_endpoints']),
                        'results': user_access_results
                    }
                    print(f"    ✅ Regular user access properly restricted ({restricted_access} endpoints restricted)")
                else:
                    vulnerabilities.append("Regular user has unrestricted access to all endpoints")
                    test_details['user_access'] = {
                        'status': 'FAIL',
                        'endpoints_restricted': restricted_access,
                        'total_endpoints': len(self.config['protected_endpoints']),
                        'results': user_access_results
                    }
                    print(f"    ❌ Regular user has unrestricted access to all endpoints")
            else:
                security_issues.append("Regular user authentication failed")
                test_details['user_access'] = {
                    'status': 'FAIL',
                    'error': 'User authentication failed'
                }
                print(f"    ❌ Regular user authentication failed")
                
            # Test 3: Readonly user restrictions
            print("  Test 3: Readonly user restrictions...")
            total_tests += 1
            
            if 'readonly' in user_tokens:
                readonly_access_results = []
                
                # Test GET requests (should be allowed)
                for endpoint in self.config['protected_endpoints']:
                    status, response, response_time = await self.make_request(
                        session, 'GET', 
                        f"{self.config['api_gateway_url']}{endpoint}", 
                        auth_token=user_tokens['readonly']
                    )
                    
                    readonly_access_results.append({
                        'endpoint': endpoint,
                        'method': 'GET',
                        'status': status,
                        'response_time': response_time,
                        'access_granted': status == 200
                    })
                    
                # Test POST requests (should be denied)
                post_data = {'test': 'data'}
                for endpoint in self.config['protected_endpoints']:
                    status, response, response_time = await self.make_request(
                        session, 'POST', 
                        f"{self.config['api_gateway_url']}{endpoint}", 
                        data=post_data,
                        auth_token=user_tokens['readonly']
                    )
                    
                    readonly_access_results.append({
                        'endpoint': endpoint,
                        'method': 'POST',
                        'status': status,
                        'response_time': response_time,
                        'access_granted': status == 200
                    })
                    
                get_allowed = sum(1 for r in readonly_access_results if r['method'] == 'GET' and r['access_granted'])
                post_denied = sum(1 for r in readonly_access_results if r['method'] == 'POST' and not r['access_granted'])
                
                if get_allowed > 0 and post_denied > 0:
                    passed_tests += 1
                    test_details['readonly_access'] = {
                        'status': 'PASS',
                        'get_allowed': get_allowed,
                        'post_denied': post_denied,
                        'results': readonly_access_results
                    }
                    print(f"    ✅ Readonly user properly restricted (GET: {get_allowed} allowed, POST: {post_denied} denied)")
                else:
                    vulnerabilities.append("Readonly user permissions not properly enforced")
                    test_details['readonly_access'] = {
                        'status': 'FAIL',
                        'get_allowed': get_allowed,
                        'post_denied': post_denied,
                        'results': readonly_access_results
                    }
                    print(f"    ❌ Readonly user permissions not properly enforced")
            else:
                security_issues.append("Readonly user authentication failed")
                test_details['readonly_access'] = {
                    'status': 'FAIL',
                    'error': 'Readonly user authentication failed'
                }
                print(f"    ❌ Readonly user authentication failed")
                
            # Test 4: Unauthorized access (no token)
            print("  Test 4: Unauthorized access protection...")
            total_tests += 1
            
            unauthorized_results = []
            
            for endpoint in self.config['protected_endpoints']:
                status, response, response_time = await self.make_request(
                    session, 'GET', 
                    f"{self.config['api_gateway_url']}{endpoint}"
                )
                
                unauthorized_results.append({
                    'endpoint': endpoint,
                    'status': status,
                    'response_time': response_time,
                    'access_denied': status == 401 or status == 403
                })
                
            properly_denied = sum(1 for r in unauthorized_results if r['access_denied'])
            
            if properly_denied == len(self.config['protected_endpoints']):
                passed_tests += 1
                test_details['unauthorized_access'] = {
                    'status': 'PASS',
                    'endpoints_denied': properly_denied,
                    'total_endpoints': len(self.config['protected_endpoints']),
                    'results': unauthorized_results
                }
                print(f"    ✅ Unauthorized access properly denied for all {properly_denied} endpoints")
            else:
                vulnerabilities.append("Some endpoints accessible without authentication")
                test_details['unauthorized_access'] = {
                    'status': 'FAIL',
                    'endpoints_denied': properly_denied,
                    'total_endpoints': len(self.config['protected_endpoints']),
                    'results': unauthorized_results
                }
                print(f"    ❌ Unauthorized access not properly denied ({properly_denied}/{len(self.config['protected_endpoints'])} denied)")
                
        end_time = datetime.utcnow()
        
        # Add recommendations based on findings
        if vulnerabilities:
            recommendations.append("Implement proper role-based access control (RBAC)")
            recommendations.append("Ensure all protected endpoints require authentication")
            recommendations.append("Review and fix authorization logic")
            
        if security_issues:
            recommendations.append("Fix user authentication system")
            recommendations.append("Ensure proper user role management")
            
        return SecurityTestResult(
            test_name="Authorization and RBAC",
            start_time=start_time,
            end_time=end_time,
            duration=(end_time - start_time).total_seconds(),
            total_tests=total_tests,
            passed_tests=passed_tests,
            failed_tests=total_tests - passed_tests,
            vulnerabilities=vulnerabilities,
            security_issues=security_issues,
            recommendations=recommendations,
            test_details=test_details
        )
        
    async def test_input_validation_security(self) -> SecurityTestResult:
        """Test input validation and injection attack prevention"""
        print("🛡️ Running Input Validation Security Tests...")
        
        start_time = datetime.utcnow()
        vulnerabilities = []
        security_issues = []
        recommendations = []
        test_details = {}
        total_tests = 0
        passed_tests = 0
        
        async with aiohttp.ClientSession() as session:
            # Get a valid token for testing
            auth_token = None
            if 'user' in self.session_tokens:
                auth_token = self.session_tokens['user']
            else:
                # Try to get a token
                login_data = {
                    'username': self.config['test_users']['user']['username'],
                    'password': self.config['test_users']['user']['password']
                }
                
                status, response, response_time = await self.make_request(
                    session, 'POST', 
                    f"{self.config['api_gateway_url']}{self.config['auth_endpoints']['login']}", 
                    login_data
                )
                
                if status == 200 and 'token' in response:
                    auth_token = response['token']
                    
            # Test 1: XSS (Cross-Site Scripting) prevention
            print("  Test 1: XSS attack prevention...")
            total_tests += 1
            
            xss_payloads = [
                "<script>alert('XSS')</script>",
                "javascript:alert('XSS')",
                "<img src=x onerror=alert('XSS')>",
                "';alert('XSS');//"
            ]
            
            xss_results = []
            
            for payload in xss_payloads:
                sample_data = {
                    'name': payload,
                    'sample_type': 'DNA',
                    'volume': 100.0,
                    'concentration': 50.0
                }
                
                status, response, response_time = await self.make_request(
                    session, 'POST', 
                    f"{self.config['api_gateway_url']}/api/samples/v1/samples", 
                    data=sample_data,
                    auth_token=auth_token
                )
                
                xss_results.append({
                    'payload': payload,
                    'status': status,
                    'response_time': response_time,
                    'blocked': status == 400 or status == 422
                })
                
            blocked_xss = sum(1 for r in xss_results if r['blocked'])
            
            if blocked_xss == len(xss_payloads):
                passed_tests += 1
                test_details['xss_prevention'] = {
                    'status': 'PASS',
                    'payloads_blocked': blocked_xss,
                    'total_payloads': len(xss_payloads),
                    'results': xss_results
                }
                print(f"    ✅ XSS attacks properly blocked ({blocked_xss}/{len(xss_payloads)})")
            else:
                vulnerabilities.append("XSS attacks not properly blocked")
                test_details['xss_prevention'] = {
                    'status': 'FAIL',
                    'payloads_blocked': blocked_xss,
                    'total_payloads': len(xss_payloads),
                    'results': xss_results
                }
                print(f"    ❌ XSS attacks not properly blocked ({blocked_xss}/{len(xss_payloads)})")
                
            # Test 2: SQL Injection prevention
            print("  Test 2: SQL injection prevention...")
            total_tests += 1
            
            sql_payloads = [
                "'; DROP TABLE samples; --",
                "' OR '1'='1",
                "1; DELETE FROM users; --",
                "' UNION SELECT * FROM users --"
            ]
            
            sql_results = []
            
            for payload in sql_payloads:
                sample_data = {
                    'name': f"Sample {payload}",
                    'sample_type': payload,
                    'volume': 100.0,
                    'concentration': 50.0
                }
                
                status, response, response_time = await self.make_request(
                    session, 'POST', 
                    f"{self.config['api_gateway_url']}/api/samples/v1/samples", 
                    data=sample_data,
                    auth_token=auth_token
                )
                
                sql_results.append({
                    'payload': payload,
                    'status': status,
                    'response_time': response_time,
                    'blocked': status == 400 or status == 422
                })
                
            blocked_sql = sum(1 for r in sql_results if r['blocked'])
            
            if blocked_sql == len(sql_payloads):
                passed_tests += 1
                test_details['sql_injection_prevention'] = {
                    'status': 'PASS',
                    'payloads_blocked': blocked_sql,
                    'total_payloads': len(sql_payloads),
                    'results': sql_results
                }
                print(f"    ✅ SQL injection attacks properly blocked ({blocked_sql}/{len(sql_payloads)})")
            else:
                vulnerabilities.append("SQL injection attacks not properly blocked")
                test_details['sql_injection_prevention'] = {
                    'status': 'FAIL',
                    'payloads_blocked': blocked_sql,
                    'total_payloads': len(sql_payloads),
                    'results': sql_results
                }
                print(f"    ❌ SQL injection attacks not properly blocked ({blocked_sql}/{len(sql_payloads)})")
                
            # Test 3: Command injection prevention
            print("  Test 3: Command injection prevention...")
            total_tests += 1
            
            command_payloads = [
                "; ls -la",
                "| cat /etc/passwd",
                "&& rm -rf /",
                "`whoami`"
            ]
            
            command_results = []
            
            for payload in command_payloads:
                sample_data = {
                    'name': f"Sample{payload}",
                    'sample_type': 'DNA',
                    'volume': 100.0,
                    'concentration': 50.0
                }
                
                status, response, response_time = await self.make_request(
                    session, 'POST', 
                    f"{self.config['api_gateway_url']}/api/samples/v1/samples", 
                    data=sample_data,
                    auth_token=auth_token
                )
                
                command_results.append({
                    'payload': payload,
                    'status': status,
                    'response_time': response_time,
                    'blocked': status == 400 or status == 422
                })
                
            blocked_commands = sum(1 for r in command_results if r['blocked'])
            
            if blocked_commands == len(command_payloads):
                passed_tests += 1
                test_details['command_injection_prevention'] = {
                    'status': 'PASS',
                    'payloads_blocked': blocked_commands,
                    'total_payloads': len(command_payloads),
                    'results': command_results
                }
                print(f"    ✅ Command injection attacks properly blocked ({blocked_commands}/{len(command_payloads)})")
            else:
                vulnerabilities.append("Command injection attacks not properly blocked")
                test_details['command_injection_prevention'] = {
                    'status': 'FAIL',
                    'payloads_blocked': blocked_commands,
                    'total_payloads': len(command_payloads),
                    'results': command_results
                }
                print(f"    ❌ Command injection attacks not properly blocked ({blocked_commands}/{len(command_payloads)})")
                
            # Test 4: Large payload handling
            print("  Test 4: Large payload handling...")
            total_tests += 1
            
            large_payload = "A" * 10000  # 10KB payload
            
            large_data = {
                'name': large_payload,
                'sample_type': 'DNA',
                'volume': 100.0,
                'concentration': 50.0
            }
            
            status, response, response_time = await self.make_request(
                session, 'POST', 
                f"{self.config['api_gateway_url']}/api/samples/v1/samples", 
                data=large_data,
                auth_token=auth_token
            )
            
            if status == 413 or status == 400:  # Payload too large or bad request
                passed_tests += 1
                test_details['large_payload'] = {
                    'status': 'PASS',
                    'payload_size': len(large_payload),
                    'response_time': response_time,
                    'properly_rejected': True
                }
                print(f"    ✅ Large payload properly rejected ({len(large_payload)} bytes)")
            else:
                vulnerabilities.append("Large payloads not properly limited")
                test_details['large_payload'] = {
                    'status': 'FAIL',
                    'payload_size': len(large_payload),
                    'response_time': response_time,
                    'accepted': status == 200
                }
                print(f"    ❌ Large payload not properly rejected (status: {status})")
                
        end_time = datetime.utcnow()
        
        # Add recommendations based on findings
        if vulnerabilities:
            recommendations.append("Implement comprehensive input validation and sanitization")
            recommendations.append("Add payload size limits to prevent DoS attacks")
            recommendations.append("Use parameterized queries to prevent SQL injection")
            recommendations.append("Implement proper output encoding to prevent XSS")
            
        return SecurityTestResult(
            test_name="Input Validation Security",
            start_time=start_time,
            end_time=end_time,
            duration=(end_time - start_time).total_seconds(),
            total_tests=total_tests,
            passed_tests=passed_tests,
            failed_tests=total_tests - passed_tests,
            vulnerabilities=vulnerabilities,
            security_issues=security_issues,
            recommendations=recommendations,
            test_details=test_details
        )
        
    async def test_session_security(self) -> SecurityTestResult:
        """Test session management and security"""
        print("🔐 Running Session Security Tests...")
        
        start_time = datetime.utcnow()
        vulnerabilities = []
        security_issues = []
        recommendations = []
        test_details = {}
        total_tests = 0
        passed_tests = 0
        
        async with aiohttp.ClientSession() as session:
            # Test 1: Session timeout
            print("  Test 1: Session timeout handling...")
            total_tests += 1
            
            # Create a token with short expiration for testing
            test_payload = {
                'username': 'testuser',
                'role': 'user',
                'exp': datetime.utcnow() + timedelta(seconds=1)  # 1 second expiration
            }
            
            try:
                expired_token = jwt.encode(test_payload, self.config['jwt_secret'], algorithm='HS256')
                
                # Wait for token to expire
                await asyncio.sleep(2)
                
                status, response, response_time = await self.make_request(
                    session, 'GET', 
                    f"{self.config['api_gateway_url']}{self.config['auth_endpoints']['profile']}", 
                    auth_token=expired_token
                )
                
                if status == 401 or status == 403:
                    passed_tests += 1
                    test_details['session_timeout'] = {
                        'status': 'PASS',
                        'response_time': response_time,
                        'expired_token_rejected': True
                    }
                    print(f"    ✅ Expired token properly rejected ({response_time:.3f}s)")
                else:
                    vulnerabilities.append("Expired tokens not properly rejected")
                    test_details['session_timeout'] = {
                        'status': 'FAIL',
                        'response_time': response_time,
                        'expired_token_accepted': True
                    }
                    print(f"    ❌ Expired token not properly rejected (status: {status})")
                    
            except Exception as e:
                security_issues.append(f"JWT token handling error: {str(e)}")
                test_details['session_timeout'] = {
                    'status': 'FAIL',
                    'error': str(e)
                }
                print(f"    ❌ JWT token handling error: {str(e)}")
                
            # Test 2: Token tampering detection
            print("  Test 2: Token tampering detection...")
            total_tests += 1
            
            if 'user' in self.session_tokens:
                # Tamper with the token
                original_token = self.session_tokens['user']
                tampered_token = original_token[:-10] + "tamperedXX"
                
                status, response, response_time = await self.make_request(
                    session, 'GET', 
                    f"{self.config['api_gateway_url']}{self.config['auth_endpoints']['profile']}", 
                    auth_token=tampered_token
                )
                
                if status == 401 or status == 403:
                    passed_tests += 1
                    test_details['token_tampering'] = {
                        'status': 'PASS',
                        'response_time': response_time,
                        'tampered_token_rejected': True
                    }
                    print(f"    ✅ Tampered token properly rejected ({response_time:.3f}s)")
                else:
                    vulnerabilities.append("Tampered tokens not properly detected")
                    test_details['token_tampering'] = {
                        'status': 'FAIL',
                        'response_time': response_time,
                        'tampered_token_accepted': True
                    }
                    print(f"    ❌ Tampered token not properly rejected (status: {status})")
            else:
                security_issues.append("No valid token available for tampering test")
                test_details['token_tampering'] = {
                    'status': 'FAIL',
                    'error': 'No valid token available'
                }
                print(f"    ❌ No valid token available for tampering test")
                
            # Test 3: Logout functionality
            print("  Test 3: Logout functionality...")
            total_tests += 1
            
            if 'user' in self.session_tokens:
                # Test logout
                status, response, response_time = await self.make_request(
                    session, 'POST', 
                    f"{self.config['api_gateway_url']}{self.config['auth_endpoints']['logout']}", 
                    auth_token=self.session_tokens['user']
                )
                
                if status == 200:
                    # Try to use the token after logout
                    status2, response2, response_time2 = await self.make_request(
                        session, 'GET', 
                        f"{self.config['api_gateway_url']}{self.config['auth_endpoints']['profile']}", 
                        auth_token=self.session_tokens['user']
                    )
                    
                    if status2 == 401 or status2 == 403:
                        passed_tests += 1
                        test_details['logout_functionality'] = {
                            'status': 'PASS',
                            'logout_response_time': response_time,
                            'token_invalidated': True
                        }
                        print(f"    ✅ Logout properly invalidates token ({response_time:.3f}s)")
                    else:
                        vulnerabilities.append("Logout does not properly invalidate tokens")
                        test_details['logout_functionality'] = {
                            'status': 'FAIL',
                            'logout_response_time': response_time,
                            'token_still_valid': True
                        }
                        print(f"    ❌ Logout does not properly invalidate token")
                else:
                    security_issues.append("Logout endpoint not working properly")
                    test_details['logout_functionality'] = {
                        'status': 'FAIL',
                        'logout_response_time': response_time,
                        'logout_failed': True
                    }
                    print(f"    ❌ Logout endpoint not working (status: {status})")
            else:
                security_issues.append("No valid token available for logout test")
                test_details['logout_functionality'] = {
                    'status': 'FAIL',
                    'error': 'No valid token available'
                }
                print(f"    ❌ No valid token available for logout test")
                
        end_time = datetime.utcnow()
        
        # Add recommendations based on findings
        if vulnerabilities:
            recommendations.append("Implement proper JWT token validation and expiration")
            recommendations.append("Add token blacklisting for logout functionality")
            recommendations.append("Implement proper session timeout handling")
            
        if security_issues:
            recommendations.append("Fix JWT token handling implementation")
            recommendations.append("Ensure proper logout functionality")
            
        return SecurityTestResult(
            test_name="Session Security",
            start_time=start_time,
            end_time=end_time,
            duration=(end_time - start_time).total_seconds(),
            total_tests=total_tests,
            passed_tests=passed_tests,
            failed_tests=total_tests - passed_tests,
            vulnerabilities=vulnerabilities,
            security_issues=security_issues,
            recommendations=recommendations,
            test_details=test_details
        )
        
    def print_security_test_result(self, result: SecurityTestResult):
        """Print formatted security test result"""
        print(f"\n{'='*70}")
        print(f"SECURITY TEST RESULT: {result.test_name}")
        print(f"{'='*70}")
        print(f"Duration: {result.duration:.2f}s")
        print(f"Total Tests: {result.total_tests}")
        print(f"Passed: {result.passed_tests}")
        print(f"Failed: {result.failed_tests}")
        print(f"Success Rate: {(result.passed_tests/result.total_tests)*100:.1f}%")
        
        if result.vulnerabilities:
            print(f"\n🚨 VULNERABILITIES FOUND:")
            for vuln in result.vulnerabilities:
                print(f"  - {vuln}")
                
        if result.security_issues:
            print(f"\n⚠️  SECURITY ISSUES:")
            for issue in result.security_issues:
                print(f"  - {issue}")
                
        if result.recommendations:
            print(f"\n💡 RECOMMENDATIONS:")
            for rec in result.recommendations:
                print(f"  - {rec}")
                
        print(f"\n📊 Test Details:")
        for test_name, details in result.test_details.items():
            status_icon = "✅" if details.get('status') == 'PASS' else "❌"
            print(f"  {status_icon} {test_name}: {details.get('status', 'UNKNOWN')}")
            
    def save_security_results(self, results: List[SecurityTestResult], filename: str):
        """Save security test results to JSON file"""
        results_data = []
        for result in results:
            results_data.append(asdict(result))
            
        with open(filename, 'w') as f:
            json.dump(results_data, f, indent=2, default=str)
            
        print(f"\n📊 Security test results saved to {filename}")
        
    async def run_all_security_tests(self):
        """Run all security tests"""
        print("🔒 Starting TracSeq 2.0 Security Testing")
        print("=" * 70)
        
        results = []
        
        # 1. Authentication Flow Tests
        result = await self.test_authentication_flow()
        self.print_security_test_result(result)
        results.append(result)
        
        # 2. Authorization and RBAC Tests
        result = await self.test_authorization_rbac()
        self.print_security_test_result(result)
        results.append(result)
        
        # 3. Input Validation Security Tests
        result = await self.test_input_validation_security()
        self.print_security_test_result(result)
        results.append(result)
        
        # 4. Session Security Tests
        result = await self.test_session_security()
        self.print_security_test_result(result)
        results.append(result)
        
        # Save results
        timestamp = datetime.utcnow().strftime("%Y%m%d_%H%M%S")
        self.save_security_results(results, f"security_test_results_{timestamp}.json")
        
        # Print comprehensive summary
        total_tests = sum(r.total_tests for r in results)
        total_passed = sum(r.passed_tests for r in results)
        total_vulnerabilities = sum(len(r.vulnerabilities) for r in results)
        total_issues = sum(len(r.security_issues) for r in results)
        
        print(f"\n🔒 Security Testing Complete!")
        print(f"=" * 70)
        print(f"Total Test Categories: {len(results)}")
        print(f"Total Tests: {total_tests}")
        print(f"Passed Tests: {total_passed}")
        print(f"Failed Tests: {total_tests - total_passed}")
        print(f"Overall Success Rate: {(total_passed/total_tests)*100:.1f}%")
        print(f"Vulnerabilities Found: {total_vulnerabilities}")
        print(f"Security Issues Found: {total_issues}")
        
        if total_vulnerabilities > 0 or total_issues > 0:
            print(f"\n⚠️  SECURITY ATTENTION REQUIRED!")
            print(f"Please review the detailed results and implement recommended fixes.")
        else:
            print(f"\n✅ No critical security vulnerabilities found!")
            
        print(f"Results saved with timestamp: {timestamp}")
        
        return results

async def main():
    """Main entry point"""
    parser = argparse.ArgumentParser(description='TracSeq 2.0 Security Testing')
    parser.add_argument('--test', choices=['auth', 'rbac', 'input', 'session', 'all'], 
                       default='all', help='Test type to run')
    
    args = parser.parse_args()
    
    tester = SecurityTester(SECURITY_TEST_CONFIG)
    
    if args.test == 'all':
        await tester.run_all_security_tests()
    elif args.test == 'auth':
        result = await tester.test_authentication_flow()
        tester.print_security_test_result(result)
    elif args.test == 'rbac':
        result = await tester.test_authorization_rbac()
        tester.print_security_test_result(result)
    elif args.test == 'input':
        result = await tester.test_input_validation_security()
        tester.print_security_test_result(result)
    elif args.test == 'session':
        result = await tester.test_session_security()
        tester.print_security_test_result(result)

if __name__ == "__main__":
    asyncio.run(main()) 