# TracSeq 2.0 API Gateway - API Reference

[![Version](https://img.shields.io/badge/version-2.0.0-blue.svg)](https://github.com/tracseq/api-gateway)
[![API Status](https://img.shields.io/badge/API-stable-green.svg)](https://api.tracseq.com/health)

Complete API reference for the TracSeq 2.0 API Gateway, including all endpoints, request/response formats, authentication, and error handling.

## Table of Contents

- [Base URL](#base-url)
- [Authentication](#authentication)
- [Rate Limiting](#rate-limiting)
- [Error Handling](#error-handling)
- [Gateway Endpoints](#gateway-endpoints)
- [Authentication Endpoints](#authentication-endpoints)
- [Proxy Endpoints](#proxy-endpoints)
- [Response Formats](#response-formats)
- [Status Codes](#status-codes)
- [Examples](#examples)

## Base URL

```
Production:  https://api.tracseq.com
Development: http://localhost:8000
```

## Authentication

### JWT Bearer Token

All protected endpoints require a valid JWT token in the Authorization header:

```http
Authorization: Bearer <token>
```

### Token Format

```json
{
  "user_id": "123",
  "email": "user@example.com",
  "role": "admin|user|viewer",
  "exp": 1642234567,
  "iat": 1642230967
}
```

### Authentication Flow

1. **Login** with credentials to receive a JWT token
2. **Include token** in Authorization header for subsequent requests
3. **Refresh token** before expiration (default: 24 hours)

## Rate Limiting

### Limits

- **Authenticated users**: 100 requests per minute
- **Anonymous users**: 50 requests per minute
- **Burst allowance**: 20 additional requests

### Headers

Rate limit information is included in response headers:

```http
X-RateLimit-Limit: 100
X-RateLimit-Remaining: 95
X-RateLimit-Reset: 1642234567
X-RateLimit-Retry-After: 60
```

### Adaptive Rate Limiting

The system automatically adjusts limits based on:
- System load
- Service health
- User behavior patterns
- Time of day

## Error Handling

### Standard Error Response

```json
{
  "error": {
    "code": "VALIDATION_ERROR",
    "message": "Invalid request data",
    "details": {
      "field": "email",
      "issue": "Invalid email format"
    },
    "request_id": "req_123456789",
    "timestamp": "2024-01-15T10:30:00Z"
  }
}
```

### Error Categories

- **Client Errors (4xx)**: Invalid requests, authentication issues
- **Server Errors (5xx)**: Internal errors, service unavailable
- **Gateway Errors (502-504)**: Upstream service issues

## Gateway Endpoints

### Health Check

#### GET /health

Basic health check endpoint.

**Response:**
```json
{
  "status": "healthy",
  "service": "api-gateway",
  "version": "2.0.0",
  "timestamp": "2024-01-15T10:30:00Z"
}
```

**Status Codes:**
- `200`: Service is healthy
- `503`: Service is unhealthy

---

#### GET /health/detailed

Detailed health check with service status.

**Response:**
```json
{
  "status": "healthy",
  "service": "api-gateway",
  "version": "2.0.0",
  "timestamp": "2024-01-15T10:30:00Z",
  "services": {
    "auth": {
      "healthy": true,
      "response_time": 0.045,
      "last_check": "2024-01-15T10:29:30Z"
    },
    "sample": {
      "healthy": true,
      "response_time": 0.032,
      "last_check": "2024-01-15T10:29:30Z"
    },
    "storage": {
      "healthy": false,
      "error": "Connection timeout",
      "last_check": "2024-01-15T10:29:30Z"
    }
  },
  "circuit_breakers": {
    "auth": {
      "state": "closed",
      "failure_count": 0,
      "last_failure": null
    },
    "storage": {
      "state": "open",
      "failure_count": 5,
      "last_failure": "2024-01-15T10:25:00Z"
    }
  },
  "database": {
    "status": "connected",
    "pool_size": 10,
    "active_connections": 3
  }
}
```

**Status Codes:**
- `200`: Gateway is healthy (may include unhealthy services)
- `503`: Gateway is unhealthy

---

### Metrics

#### GET /metrics

System metrics and performance data.

**Response:**
```json
{
  "gateway": {
    "version": "2.0.0",
    "uptime": "2h 45m 30s",
    "environment": "production",
    "requests_total": 15420,
    "requests_per_minute": 125.5,
    "avg_response_time": 0.089
  },
  "services": {
    "overall_health": "degraded",
    "healthy_services": 4,
    "total_services": 5,
    "service_stats": {
      "auth": {
        "requests": 3240,
        "avg_response_time": 0.045,
        "error_rate": 0.001
      },
      "sample": {
        "requests": 5680,
        "avg_response_time": 0.032,
        "error_rate": 0.002
      }
    }
  },
  "circuit_breakers": {
    "auth": {"state": "closed"},
    "sample": {"state": "closed"},
    "storage": {"state": "open"}
  },
  "rate_limiting": {
    "requests_blocked": 45,
    "top_blocked_ips": ["192.168.1.100", "10.0.0.50"],
    "adaptive_limits_active": true
  }
}
```

**Status Codes:**
- `200`: Metrics retrieved successfully

---

### Configuration

#### GET /config

Gateway configuration information (non-sensitive).

**Authentication:** Required

**Response:**
```json
{
  "gateway": {
    "version": "2.0.0",
    "environment": "production",
    "features": {
      "rate_limiting": true,
      "circuit_breaker": true,
      "metrics": true,
      "security_headers": true
    }
  },
  "services": {
    "auth": {
      "url": "http://auth-service:8080",
      "timeout": 30,
      "retries": 3
    },
    "sample": {
      "url": "http://sample-service:8081",
      "timeout": 30,
      "retries": 3
    }
  },
  "rate_limits": {
    "default": 100,
    "burst": 20,
    "window": 60
  }
}
```

**Status Codes:**
- `200`: Configuration retrieved successfully
- `401`: Authentication required
- `403`: Insufficient permissions

## Authentication Endpoints

### Login

#### POST /api/auth/login

Authenticate user and receive JWT token.

**Request:**
```json
{
  "email": "admin@tracseq.com",
  "password": "admin123"
}
```

**Response:**
```json
{
  "token": "eyJ0eXAiOiJKV1QiLCJhbGciOiJIUzI1NiJ9...",
  "user": {
    "id": "1",
    "email": "admin@tracseq.com",
    "name": "Admin User",
    "role": "admin",
    "permissions": ["read", "write", "admin"]
  },
  "expires_at": "2024-01-16T10:30:00Z"
}
```

**Status Codes:**
- `200`: Authentication successful
- `400`: Invalid request format
- `401`: Invalid credentials
- `429`: Rate limit exceeded

---

### Current User

#### GET /api/auth/me

Get current user information.

**Authentication:** Required

**Response:**
```json
{
  "user": {
    "id": "1",
    "email": "admin@tracseq.com",
    "name": "Admin User",
    "role": "admin",
    "permissions": ["read", "write", "admin"],
    "last_login": "2024-01-15T10:30:00Z"
  }
}
```

**Status Codes:**
- `200`: User information retrieved
- `401`: Invalid or expired token

---

### Refresh Token

#### POST /api/auth/refresh

Refresh JWT token before expiration.

**Authentication:** Required

**Response:**
```json
{
  "token": "eyJ0eXAiOiJKV1QiLCJhbGciOiJIUzI1NiJ9...",
  "expires_at": "2024-01-16T10:30:00Z"
}
```

**Status Codes:**
- `200`: Token refreshed successfully
- `401`: Invalid or expired token

---

### Logout

#### POST /api/auth/logout

Logout user and invalidate token.

**Authentication:** Required

**Response:**
```json
{
  "message": "Logged out successfully"
}
```

**Status Codes:**
- `200`: Logout successful
- `401`: Invalid or expired token

## Proxy Endpoints

All microservice endpoints are accessible through the gateway with the `/api/` prefix.

### Service Routing

| Service | Prefix | Target | Example |
|---------|--------|--------|---------|
| Auth | `/api/auth/*` | `auth-service:8080` | `/api/auth/users` |
| Sample | `/api/samples/*` | `sample-service:8081` | `/api/samples/list` |
| Storage | `/api/storage/*` | `storage-service:8082` | `/api/storage/locations` |
| Template | `/api/templates/*` | `template-service:8083` | `/api/templates/validate` |
| Sequencing | `/api/sequencing/*` | `sequencing-service:8084` | `/api/sequencing/jobs` |
| RAG | `/api/rag/*` | `rag-service:8000` | `/api/rag/query` |

### Example Proxy Requests

#### Sample Service

```bash
# List all samples
GET /api/samples

# Get specific sample
GET /api/samples/123

# Create new sample
POST /api/samples
{
  "name": "Sample001",
  "type": "DNA",
  "location": "A1"
}
```

#### Storage Service

```bash
# List storage locations
GET /api/storage/locations

# Get location details
GET /api/storage/locations/freezer-1

# Update location
PUT /api/storage/locations/freezer-1
{
  "temperature": -80,
  "capacity": 100
}
```

#### RAG Service

```bash
# Query documents
POST /api/rag/query
{
  "query": "What are the sample preparation protocols?",
  "context": "laboratory"
}

# Upload document
POST /api/rag/documents/upload
Content-Type: multipart/form-data
```

## Response Formats

### Success Response

```json
{
  "data": {
    // Response data
  },
  "meta": {
    "request_id": "req_123456789",
    "timestamp": "2024-01-15T10:30:00Z",
    "version": "2.0.0"
  }
}
```

### Error Response

```json
{
  "error": {
    "code": "ERROR_CODE",
    "message": "Human readable error message",
    "details": {
      // Additional error details
    },
    "request_id": "req_123456789",
    "timestamp": "2024-01-15T10:30:00Z"
  }
}
```

### Paginated Response

```json
{
  "data": [
    // Array of items
  ],
  "pagination": {
    "page": 1,
    "per_page": 20,
    "total": 150,
    "total_pages": 8,
    "has_next": true,
    "has_prev": false
  },
  "meta": {
    "request_id": "req_123456789",
    "timestamp": "2024-01-15T10:30:00Z"
  }
}
```

## Status Codes

### Success Codes

- `200 OK`: Request successful
- `201 Created`: Resource created successfully
- `202 Accepted`: Request accepted for processing
- `204 No Content`: Request successful, no content to return

### Client Error Codes

- `400 Bad Request`: Invalid request format or data
- `401 Unauthorized`: Authentication required or invalid
- `403 Forbidden`: Insufficient permissions
- `404 Not Found`: Resource not found
- `409 Conflict`: Resource conflict
- `422 Unprocessable Entity`: Validation error
- `429 Too Many Requests`: Rate limit exceeded

### Server Error Codes

- `500 Internal Server Error`: Unexpected server error
- `502 Bad Gateway`: Upstream service error
- `503 Service Unavailable`: Service temporarily unavailable
- `504 Gateway Timeout`: Upstream service timeout

### Gateway-Specific Codes

- `520 Unknown Error`: Unknown upstream error
- `521 Service Down`: Upstream service is down
- `522 Connection Timed Out`: Connection to upstream service timed out
- `523 Origin Unreachable`: Cannot reach upstream service

## Examples

### Authentication Flow

```bash
# 1. Login
curl -X POST http://localhost:8000/api/auth/login \
  -H "Content-Type: application/json" \
  -d '{
    "email": "admin@tracseq.com",
    "password": "admin123"
  }'

# Response:
{
  "token": "eyJ0eXAiOiJKV1QiLCJhbGciOiJIUzI1NiJ9...",
  "user": {
    "id": "1",
    "email": "admin@tracseq.com",
    "name": "Admin User",
    "role": "admin"
  }
}

# 2. Use token for authenticated requests
curl -X GET http://localhost:8000/api/auth/me \
  -H "Authorization: Bearer eyJ0eXAiOiJKV1QiLCJhbGciOiJIUzI1NiJ9..."
```

### Error Handling

```bash
# Request with invalid data
curl -X POST http://localhost:8000/api/samples \
  -H "Content-Type: application/json" \
  -H "Authorization: Bearer <token>" \
  -d '{
    "name": "",
    "type": "INVALID_TYPE"
  }'

# Error response:
{
  "error": {
    "code": "VALIDATION_ERROR",
    "message": "Invalid request data",
    "details": {
      "name": "Field is required",
      "type": "Invalid sample type"
    },
    "request_id": "req_123456789",
    "timestamp": "2024-01-15T10:30:00Z"
  }
}
```

### Rate Limiting

```bash
# Check rate limit headers
curl -I http://localhost:8000/api/samples \
  -H "Authorization: Bearer <token>"

# Response headers:
HTTP/1.1 200 OK
X-RateLimit-Limit: 100
X-RateLimit-Remaining: 95
X-RateLimit-Reset: 1642234567
X-RateLimit-Retry-After: 60
```

### Circuit Breaker

```bash
# When service is down
curl http://localhost:8000/api/storage/locations

# Response:
{
  "error": {
    "code": "SERVICE_UNAVAILABLE",
    "message": "Storage service is temporarily unavailable",
    "details": {
      "service": "storage",
      "circuit_breaker": "open",
      "retry_after": 60
    },
    "request_id": "req_123456789",
    "timestamp": "2024-01-15T10:30:00Z"
  }
}
```

### Health Monitoring

```bash
# Check overall health
curl http://localhost:8000/health/detailed

# Monitor specific service
curl http://localhost:8000/metrics | jq '.services.service_stats.auth'

# Response:
{
  "requests": 3240,
  "avg_response_time": 0.045,
  "error_rate": 0.001,
  "circuit_breaker": "closed"
}
```

### Bulk Operations

```bash
# Batch create samples
curl -X POST http://localhost:8000/api/samples/batch \
  -H "Content-Type: application/json" \
  -H "Authorization: Bearer <token>" \
  -d '{
    "samples": [
      {"name": "Sample001", "type": "DNA"},
      {"name": "Sample002", "type": "RNA"},
      {"name": "Sample003", "type": "Protein"}
    ]
  }'

# Response:
{
  "data": {
    "created": 3,
    "failed": 0,
    "results": [
      {"id": "123", "name": "Sample001", "status": "created"},
      {"id": "124", "name": "Sample002", "status": "created"},
      {"id": "125", "name": "Sample003", "status": "created"}
    ]
  }
}
```

## WebSocket Support

### Real-time Updates

```javascript
// Connect to WebSocket
const ws = new WebSocket('ws://localhost:8000/ws');

// Authentication
ws.send(JSON.stringify({
  type: 'auth',
  token: 'eyJ0eXAiOiJKV1QiLCJhbGciOiJIUzI1NiJ9...'
}));

// Subscribe to updates
ws.send(JSON.stringify({
  type: 'subscribe',
  channels: ['samples', 'storage']
}));

// Receive updates
ws.onmessage = (event) => {
  const data = JSON.parse(event.data);
  console.log('Update:', data);
};
```

### WebSocket Events

```json
{
  "type": "update",
  "channel": "samples",
  "event": "created",
  "data": {
    "id": "123",
    "name": "Sample001",
    "type": "DNA"
  },
  "timestamp": "2024-01-15T10:30:00Z"
}
```

## SDK Examples

### Python SDK

```python
from tracseq_sdk import TracSeqClient

# Initialize client
client = TracSeqClient(
    base_url="http://localhost:8000",
    api_key="your-api-key"
)

# Authenticate
client.login("admin@tracseq.com", "admin123")

# Use services
samples = client.samples.list()
storage = client.storage.get_locations()
```

### JavaScript SDK

```javascript
import { TracSeqClient } from '@tracseq/sdk';

// Initialize client
const client = new TracSeqClient({
  baseUrl: 'http://localhost:8000',
  apiKey: 'your-api-key'
});

// Authenticate
await client.auth.login('admin@tracseq.com', 'admin123');

// Use services
const samples = await client.samples.list();
const storage = await client.storage.getLocations();
```

## Changelog

### Version 2.0.0

- **Breaking Changes:**
  - Migrated from monolithic to modular architecture
  - Updated authentication flow
  - Changed error response format

- **New Features:**
  - Circuit breaker protection
  - Adaptive rate limiting
  - Enhanced security headers
  - Structured logging
  - Health monitoring

- **Improvements:**
  - Better error handling
  - Improved performance
  - Enhanced documentation
  - Better testing coverage

### Version 1.9.0

- Added WebSocket support
- Improved rate limiting
- Enhanced monitoring

## Support

For API support and questions:

- **Documentation**: [https://docs.tracseq.com](https://docs.tracseq.com)
- **Status Page**: [https://status.tracseq.com](https://status.tracseq.com)
- **Support Email**: support@tracseq.com
- **GitHub Issues**: [https://github.com/tracseq/api-gateway/issues](https://github.com/tracseq/api-gateway/issues)

---

*Last updated: January 15, 2024*
*API Version: 2.0.0*

*Context improved by Giga AI*