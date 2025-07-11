# TracSeq 2.0 API Gateway - Modular Architecture

[![Python 3.9+](https://img.shields.io/badge/python-3.9+-blue.svg)](https://www.python.org/downloads/)
[![FastAPI](https://img.shields.io/badge/FastAPI-0.104+-green.svg)](https://fastapi.tiangolo.com/)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)

A modern, modular API Gateway for the TracSeq 2.0 Laboratory Information Management System (LIMS), providing centralized routing, authentication, monitoring, and security for all microservices.

## 🚀 Quick Start

### Prerequisites

- Python 3.9+
- PostgreSQL 12+
- Docker (optional, for containerized deployment)

### Installation

```bash
# Clone the repository
git clone <repository-url>
cd lims-gateway/api_gateway

# Install dependencies
pip install -r requirements.txt

# Set environment variables
export DATABASE_URL="postgres://postgres:postgres@localhost:5432/lims_db"
export JWT_SECRET_KEY="your-secret-key-here"
export ENVIRONMENT="development"

# Run the modular gateway
python -m src.api_gateway.main_modular
```

### Docker Deployment

```bash
# Build the container
docker build -t tracseq-gateway .

# Run with environment variables
docker run -p 8000:8000 \
  -e DATABASE_URL="postgres://postgres:postgres@host.docker.internal:5432/lims_db" \
  -e JWT_SECRET_KEY="your-secret-key" \
  -e ENVIRONMENT="production" \
  tracseq-gateway
```

## 📋 Table of Contents

- [Architecture Overview](#architecture-overview)
- [Core Components](#core-components)
- [Configuration](#configuration)
- [API Endpoints](#api-endpoints)
- [Security](#security)
- [Monitoring & Observability](#monitoring--observability)
- [Development Guide](#development-guide)
- [Deployment](#deployment)
- [Troubleshooting](#troubleshooting)

## 🏗️ Architecture Overview

The TracSeq 2.0 API Gateway follows a modular, layered architecture designed for maintainability, scalability, and reliability.

```
┌─────────────────────────────────────────────────────────────┐
│                     Client Applications                      │
├─────────────────────────────────────────────────────────────┤
│                    API Gateway (Port 8000)                  │
│  ┌─────────────┬─────────────┬─────────────┬─────────────┐  │
│  │   Auth      │   Samples   │   Storage   │     RAG     │  │
│  │   Routes    │   Routes    │   Routes    │   Routes    │  │
│  └─────────────┴─────────────┴─────────────┴─────────────┘  │
│  ┌─────────────────────────────────────────────────────────┐  │
│  │              Middleware Layer                          │  │
│  │  Security │ Auth │ Rate Limit │ Logging │ CORS       │  │
│  └─────────────────────────────────────────────────────────┘  │
│  ┌─────────────────────────────────────────────────────────┐  │
│  │               Service Layer                            │  │
│  │  Proxy │ Circuit Breaker │ Health Check │ Discovery   │  │
│  └─────────────────────────────────────────────────────────┘  │
│  ┌─────────────────────────────────────────────────────────┐  │
│  │                Core Layer                              │  │
│  │  Config │ Logging │ Database │ Exceptions             │  │
│  └─────────────────────────────────────────────────────────┘  │
├─────────────────────────────────────────────────────────────┤
│                     Microservices                           │
│  Auth │ Sample │ Storage │ Template │ Sequencing │ RAG     │
└─────────────────────────────────────────────────────────────┘
```

### Key Architectural Principles

1. **Separation of Concerns**: Each layer has a specific responsibility
2. **Dependency Injection**: Clean dependency management throughout
3. **Circuit Breaker Pattern**: Automatic failover for failing services
4. **Comprehensive Logging**: Structured logging for all operations
5. **Security by Design**: Multiple security layers and threat detection

## 🧩 Core Components

### 1. Core Infrastructure (`core/`)

#### Configuration Management
```python
from api_gateway.core.config import get_config

config = get_config()
database_url = config.database.url
jwt_secret = config.security.jwt_secret_key
```

**Features:**
- Environment-based configuration
- Type-safe configuration access
- Hierarchical configuration structure
- Validation with Pydantic

#### Logging System
```python
from api_gateway.core.logging import service_logger, security_logger

# Log service calls
service_logger.log_service_call("auth", "POST", "/login", 200, 0.5)

# Log security events
security_logger.log_auth_attempt("user@example.com", True, "192.168.1.1")
```

**Features:**
- Structured JSON logging
- Specialized loggers for different domains
- Request/response correlation
- Security event tracking

#### Database Layer
```python
from api_gateway.core.database import get_db_connection

async with get_db_connection() as conn:
    users = await conn.fetch("SELECT * FROM users WHERE active = true")
```

**Features:**
- Connection pooling with health monitoring
- Automatic reconnection and failover
- Query performance tracking
- Transaction management

#### Exception Handling
```python
from api_gateway.core.exceptions import ServiceException, raise_auth_error

# Raise service-specific exceptions
raise ServiceException("auth", "Service unavailable", 503)

# Raise authentication errors
raise_auth_error("Invalid credentials", user_id="user123")
```

**Features:**
- Custom exception hierarchy
- Standardized error responses
- Automatic error logging
- Security event integration

### 2. Middleware Layer (`middleware/`)

#### Authentication Middleware
```python
from fastapi import Depends
from api_gateway.middleware.auth import get_current_user

@app.get("/protected")
async def protected_endpoint(user = Depends(get_current_user)):
    return {"user_id": user.id, "email": user.email}
```

#### Security Middleware
```python
from api_gateway.middleware.security import SecurityMiddleware

app.add_middleware(SecurityMiddleware)
```

**Features:**
- Comprehensive security headers
- Threat detection and blocking
- Suspicious activity monitoring
- CSRF protection

#### Rate Limiting
```python
from api_gateway.middleware.rate_limiting import RateLimitMiddleware

app.add_middleware(RateLimitMiddleware, requests_per_minute=100)
```

**Features:**
- Per-user and per-IP rate limiting
- Adaptive limits based on system load
- Burst protection
- Circuit breaker integration

### 3. Service Layer (`services/`)

#### Proxy Service
```python
from api_gateway.services.proxy import service_proxy

# Proxy request with circuit breaker protection
response = await service_proxy.proxy_request("auth", request)

# Check service health
health = await service_proxy.check_service_health("auth")
```

**Features:**
- Circuit breaker protection
- Automatic service discovery
- Health monitoring
- Load balancing support

### 4. Route Organization (`routes/`)

#### Modular Routes
```python
from fastapi import APIRouter

# Domain-specific router
auth_router = APIRouter()

@auth_router.post("/login")
async def login(credentials: LoginRequest):
    # Authentication logic
    pass
```

**Features:**
- Domain-specific organization
- Consistent route patterns
- Proper dependency injection
- Clear separation of concerns

## ⚙️ Configuration

### Environment Variables

#### Core Configuration
```bash
# Environment
ENVIRONMENT=development|production|testing

# Server Configuration
GATEWAY_HOST=0.0.0.0
GATEWAY_PORT=8000
GATEWAY_DEBUG=false

# Database Configuration
DATABASE_URL=postgres://user:password@host:port/database
DB_POOL_MIN_SIZE=2
DB_POOL_MAX_SIZE=10

# Security Configuration
JWT_SECRET_KEY=your-secret-key-here
JWT_ALGORITHM=HS256
JWT_EXPIRATION_HOURS=24

# Rate Limiting
RATE_LIMIT_REQUESTS=100
RATE_LIMIT_WINDOW=60

# Logging
LOG_LEVEL=INFO
LOG_FILE=/var/log/tracseq/gateway.log
ENABLE_ACCESS_LOG=true
ENABLE_SQL_LOGGING=false

# Monitoring
ENABLE_METRICS=true
HEALTH_CHECK_INTERVAL=30
CIRCUIT_BREAKER_FAILURE_THRESHOLD=5
CIRCUIT_BREAKER_RECOVERY_TIMEOUT=60
```

#### Service URLs
```bash
# Microservice URLs
AUTH_SERVICE_URL=http://auth-service:8080
SAMPLE_SERVICE_URL=http://sample-service:8081
STORAGE_SERVICE_URL=http://storage-service:8082
TEMPLATE_SERVICE_URL=http://template-service:8083
SEQUENCING_SERVICE_URL=http://sequencing-service:8084
RAG_SERVICE_URL=http://rag-service:8000
```

#### CORS Configuration
```bash
# CORS Settings
CORS_ORIGINS=["http://localhost:3000","https://tracseq.example.com"]
CORS_CREDENTIALS=true
CORS_METHODS=["*"]
CORS_HEADERS=["*"]
```

### Configuration Files

#### Development Configuration
```yaml
# config/development.yml
database:
  url: "postgres://postgres:postgres@localhost:5432/lims_dev"
  pool_min_size: 2
  pool_max_size: 5

logging:
  log_level: "DEBUG"
  enable_access_log: true
  enable_sql_logging: true

security:
  jwt_secret_key: "dev-secret-key"
  rate_limit_requests: 1000

monitoring:
  enable_metrics: true
  circuit_breaker_failure_threshold: 3
```

#### Production Configuration
```yaml
# config/production.yml
database:
  url: "${DATABASE_URL}"
  pool_min_size: 10
  pool_max_size: 50

logging:
  log_level: "INFO"
  log_file: "/var/log/tracseq/gateway.log"
  enable_access_log: true
  enable_sql_logging: false

security:
  jwt_secret_key: "${JWT_SECRET_KEY}"
  rate_limit_requests: 100

monitoring:
  enable_metrics: true
  circuit_breaker_failure_threshold: 5
  circuit_breaker_recovery_timeout: 60
```

## 🔗 API Endpoints

### Authentication Endpoints

#### POST /api/auth/login
```bash
curl -X POST http://localhost:8000/api/auth/login \
  -H "Content-Type: application/json" \
  -d '{"email": "admin@tracseq.com", "password": "admin123"}'
```

**Response:**
```json
{
  "token": "eyJ0eXAiOiJKV1QiLCJhbGciOiJIUzI1NiJ9...",
  "user": {
    "id": "1",
    "email": "admin@tracseq.com",
    "name": "Admin User",
    "role": "admin"
  }
}
```

#### GET /api/auth/me
```bash
curl -X GET http://localhost:8000/api/auth/me \
  -H "Authorization: Bearer <token>"
```

### Health Check Endpoints

#### GET /health
```bash
curl http://localhost:8000/health
```

**Response:**
```json
{
  "status": "healthy",
  "service": "api-gateway",
  "version": "2.0.0"
}
```

#### GET /health/detailed
```bash
curl http://localhost:8000/health/detailed
```

**Response:**
```json
{
  "status": "healthy",
  "service": "api-gateway",
  "version": "2.0.0",
  "services": {
    "auth": {"healthy": true, "response_time": 0.045},
    "sample": {"healthy": true, "response_time": 0.032},
    "storage": {"healthy": false, "error": "Connection timeout"}
  },
  "circuit_breakers": {
    "auth": {"state": "closed", "failure_count": 0},
    "storage": {"state": "open", "failure_count": 5}
  },
  "database": "connected"
}
```

### Metrics Endpoint

#### GET /metrics
```bash
curl http://localhost:8000/metrics
```

**Response:**
```json
{
  "gateway": {
    "version": "2.0.0",
    "uptime": "2h 45m 30s",
    "environment": "production"
  },
  "services": {
    "overall_health": "degraded",
    "healthy_services": 4,
    "total_services": 5
  },
  "circuit_breakers": {
    "auth": {"state": "closed"},
    "sample": {"state": "closed"},
    "storage": {"state": "open"}
  }
}
```

### Proxy Endpoints

All microservice endpoints are automatically proxied through the gateway:

```bash
# Auth service
curl http://localhost:8000/api/auth/users

# Sample service  
curl http://localhost:8000/api/samples

# Storage service
curl http://localhost:8000/api/storage/locations

# RAG service
curl http://localhost:8000/api/rag/query
```

## 🔒 Security

### Security Features

1. **JWT Authentication**: Secure token-based authentication
2. **Rate Limiting**: Adaptive rate limiting with burst protection
3. **Security Headers**: Comprehensive security headers (CSP, HSTS, etc.)
4. **Threat Detection**: Automatic detection and blocking of malicious requests
5. **CSRF Protection**: Cross-site request forgery protection
6. **Input Validation**: Comprehensive input validation and sanitization

### Security Headers

The gateway automatically adds security headers to all responses:

```http
Content-Security-Policy: default-src 'self'; script-src 'self' 'unsafe-inline'
X-Content-Type-Options: nosniff
X-Frame-Options: DENY
X-XSS-Protection: 1; mode=block
Strict-Transport-Security: max-age=31536000; includeSubDomains
Referrer-Policy: strict-origin-when-cross-origin
```

### Threat Detection

The security middleware monitors for:

- SQL injection attempts
- XSS attacks
- Path traversal attempts
- Command injection
- Suspicious user agents
- Unusual request patterns

### Rate Limiting

Rate limiting is applied at multiple levels:

```python
# Per-user rate limiting
@app.middleware("http")
async def rate_limit_middleware(request: Request, call_next):
    # 100 requests per minute per authenticated user
    pass

# Per-IP rate limiting  
@app.middleware("http")
async def ip_rate_limit_middleware(request: Request, call_next):
    # 200 requests per minute per IP address
    pass
```

## 📊 Monitoring & Observability

### Logging

#### Structured Logging
```python
# All logs are structured JSON in production
{
  "timestamp": "2024-01-15T10:30:00.000Z",
  "level": "INFO",
  "logger": "api_gateway.services",
  "message": "Service Call: auth",
  "extra": {
    "service_name": "auth",
    "method": "POST",
    "url": "http://auth:8080/login",
    "status_code": 200,
    "response_time_ms": 45.2
  }
}
```

#### Log Categories
- **Request/Response**: All HTTP requests and responses
- **Service Calls**: Microservice interactions
- **Security Events**: Authentication, authorization, threats
- **Database Operations**: Database queries and connections
- **System Events**: Startup, shutdown, configuration changes

### Health Monitoring

#### Service Health Checks
```python
# Automatic health monitoring
health_status = await service_proxy.get_all_service_health()

# Individual service health
auth_health = await service_proxy.check_service_health("auth")
```

#### Circuit Breaker Monitoring
```python
# Circuit breaker status
cb_status = service_proxy.get_circuit_breaker_status()

# Per-service circuit breaker state
auth_cb = cb_status["auth"]
print(f"Auth service: {auth_cb['state']}")  # closed/open/half-open
```

### Metrics Collection

#### Built-in Metrics
- Request count and response times
- Service health and availability
- Circuit breaker state changes
- Rate limiting statistics
- Error rates and types
- Database connection pool status

#### Custom Metrics
```python
from api_gateway.core.logging import service_logger

# Custom business metrics
service_logger.log_service_call(
    service_name="sample",
    method="POST",
    url="/samples",
    status_code=201,
    response_time=0.123,
    custom_metric="samples_created",
    custom_value=1
)
```

## 🛠️ Development Guide

### Setting Up Development Environment

```bash
# Clone repository
git clone <repository-url>
cd lims-gateway/api_gateway

# Create virtual environment
python -m venv venv
source venv/bin/activate  # On Windows: venv\Scripts\activate

# Install dependencies
pip install -r requirements.txt
pip install -r requirements-dev.txt

# Set up pre-commit hooks
pre-commit install

# Set environment variables
cp .env.example .env
# Edit .env with your configuration

# Run tests
pytest

# Run the development server
python -m src.api_gateway.main_modular
```

### Project Structure

```
api_gateway/
├── src/
│   └── api_gateway/
│       ├── core/                 # Core infrastructure
│       │   ├── config.py        # Configuration management
│       │   ├── logging.py       # Logging system
│       │   ├── database.py      # Database layer
│       │   ├── exceptions.py    # Exception handling
│       │   └── circuit_breaker.py # Circuit breaker
│       ├── middleware/           # Middleware components
│       │   ├── auth.py          # Authentication
│       │   ├── security.py      # Security headers
│       │   ├── rate_limiting.py # Rate limiting
│       │   ├── logging.py       # Request logging
│       │   └── cors.py          # CORS handling
│       ├── services/            # Service layer
│       │   ├── proxy.py         # Service proxy
│       │   └── health.py        # Health monitoring
│       ├── routes/              # Route handlers
│       │   ├── auth.py          # Auth endpoints
│       │   ├── samples.py       # Sample endpoints
│       │   ├── storage.py       # Storage endpoints
│       │   ├── rag.py           # RAG endpoints
│       │   └── proxy.py         # Proxy routes
│       ├── models/              # Data models
│       ├── utils/               # Utility functions
│       ├── app.py               # Application factory
│       └── main_modular.py      # Entry point
├── tests/                       # Test suite
├── config/                      # Configuration files
├── docs/                        # Documentation
├── requirements.txt             # Dependencies
└── Dockerfile                   # Container definition
```

### Adding New Routes

1. **Create a new router module:**
```python
# routes/new_feature.py
from fastapi import APIRouter

new_feature_router = APIRouter()

@new_feature_router.get("/items")
async def get_items():
    return {"items": []}
```

2. **Register the router:**
```python
# routes/__init__.py
from .new_feature import new_feature_router

def setup_routes(app: FastAPI):
    app.include_router(new_feature_router, prefix="/api/new-feature")
```

### Adding New Middleware

1. **Create middleware class:**
```python
# middleware/custom.py
from starlette.middleware.base import BaseHTTPMiddleware

class CustomMiddleware(BaseHTTPMiddleware):
    async def dispatch(self, request: Request, call_next):
        # Pre-processing
        response = await call_next(request)
        # Post-processing
        return response
```

2. **Register middleware:**
```python
# app.py
def _setup_middleware(app: FastAPI):
    app.add_middleware(CustomMiddleware)
```

### Testing

#### Unit Tests
```python
# tests/test_auth.py
import pytest
from fastapi.testclient import TestClient
from api_gateway.app import create_app

@pytest.fixture
def client():
    app = create_app()
    return TestClient(app)

def test_login(client):
    response = client.post("/api/auth/login", json={
        "email": "admin@tracseq.com",
        "password": "admin123"
    })
    assert response.status_code == 200
    assert "token" in response.json()
```

#### Integration Tests
```python
# tests/test_integration.py
import pytest
from api_gateway.services.proxy import service_proxy

@pytest.mark.asyncio
async def test_service_health():
    health = await service_proxy.check_service_health("auth")
    assert "healthy" in health
```

#### Running Tests
```bash
# Run all tests
pytest

# Run with coverage
pytest --cov=api_gateway

# Run specific test file
pytest tests/test_auth.py

# Run with verbose output
pytest -v
```

## 🚀 Deployment

### Docker Deployment

#### Dockerfile
```dockerfile
FROM python:3.9-slim

WORKDIR /app

COPY requirements.txt .
RUN pip install --no-cache-dir -r requirements.txt

COPY src/ ./src/
COPY config/ ./config/

EXPOSE 8000

CMD ["python", "-m", "src.api_gateway.main_modular"]
```

#### Docker Compose
```yaml
version: '3.8'

services:
  api-gateway:
    build: .
    ports:
      - "8000:8000"
    environment:
      - DATABASE_URL=postgres://postgres:postgres@db:5432/lims_db
      - JWT_SECRET_KEY=your-secret-key
      - ENVIRONMENT=production
    depends_on:
      - db
    
  db:
    image: postgres:13
    environment:
      POSTGRES_DB: lims_db
      POSTGRES_USER: postgres
      POSTGRES_PASSWORD: postgres
    volumes:
      - postgres_data:/var/lib/postgresql/data

volumes:
  postgres_data:
```

### Kubernetes Deployment

#### Deployment Manifest
```yaml
apiVersion: apps/v1
kind: Deployment
metadata:
  name: api-gateway
spec:
  replicas: 3
  selector:
    matchLabels:
      app: api-gateway
  template:
    metadata:
      labels:
        app: api-gateway
    spec:
      containers:
      - name: api-gateway
        image: tracseq/api-gateway:latest
        ports:
        - containerPort: 8000
        env:
        - name: DATABASE_URL
          valueFrom:
            secretKeyRef:
              name: database-secret
              key: url
        - name: JWT_SECRET_KEY
          valueFrom:
            secretKeyRef:
              name: jwt-secret
              key: key
        resources:
          requests:
            memory: "256Mi"
            cpu: "250m"
          limits:
            memory: "512Mi"
            cpu: "500m"
        livenessProbe:
          httpGet:
            path: /health
            port: 8000
          initialDelaySeconds: 30
          periodSeconds: 10
        readinessProbe:
          httpGet:
            path: /health
            port: 8000
          initialDelaySeconds: 5
          periodSeconds: 5
```

#### Service Manifest
```yaml
apiVersion: v1
kind: Service
metadata:
  name: api-gateway-service
spec:
  selector:
    app: api-gateway
  ports:
  - protocol: TCP
    port: 80
    targetPort: 8000
  type: LoadBalancer
```

### Production Considerations

#### Environment Variables
```bash
# Production environment variables
ENVIRONMENT=production
GATEWAY_HOST=0.0.0.0
GATEWAY_PORT=8000

# Database (use connection pooling)
DATABASE_URL=postgres://user:pass@db-cluster:5432/lims_prod
DB_POOL_MIN_SIZE=10
DB_POOL_MAX_SIZE=50

# Security (use strong secrets)
JWT_SECRET_KEY=<strong-random-secret>
JWT_EXPIRATION_HOURS=8

# Rate limiting (adjust based on load)
RATE_LIMIT_REQUESTS=100
RATE_LIMIT_WINDOW=60

# Logging (structured logging for production)
LOG_LEVEL=INFO
LOG_FILE=/var/log/tracseq/gateway.log
ENABLE_ACCESS_LOG=true
ENABLE_SQL_LOGGING=false

# Monitoring
ENABLE_METRICS=true
HEALTH_CHECK_INTERVAL=30
```

#### Performance Tuning
```bash
# Increase worker processes
WORKERS=4

# Optimize database connections
DB_POOL_MIN_SIZE=10
DB_POOL_MAX_SIZE=50
DB_CONNECTION_TIMEOUT=30

# Configure rate limiting
RATE_LIMIT_REQUESTS=200
ADAPTIVE_RATE_LIMITING=true

# Enable caching
ENABLE_RESPONSE_CACHE=true
CACHE_TTL=300
```

## 🔧 Troubleshooting

### Common Issues

#### 1. Database Connection Issues
```bash
# Check database connectivity
curl http://localhost:8000/health/detailed

# Common solutions:
# - Verify DATABASE_URL format
# - Check database server status
# - Verify network connectivity
# - Check firewall rules
```

#### 2. Service Discovery Issues
```bash
# Check service health
curl http://localhost:8000/metrics

# Common solutions:
# - Verify service URLs in configuration
# - Check service availability
# - Review circuit breaker status
# - Check network connectivity between services
```

#### 3. Authentication Issues
```bash
# Test authentication
curl -X POST http://localhost:8000/api/auth/login \
  -H "Content-Type: application/json" \
  -d '{"email": "admin@tracseq.com", "password": "admin123"}'

# Common solutions:
# - Verify JWT_SECRET_KEY configuration
# - Check user credentials
# - Review token expiration settings
# - Check authentication middleware configuration
```

#### 4. Rate Limiting Issues
```bash
# Check rate limit headers
curl -I http://localhost:8000/api/samples

# Look for headers:
# X-RateLimit-Limit: 100
# X-RateLimit-Remaining: 95
# X-RateLimit-Reset: 1642234567

# Common solutions:
# - Adjust rate limit configuration
# - Check client IP detection
# - Review rate limiting middleware settings
```

### Debugging

#### Enable Debug Logging
```bash
export LOG_LEVEL=DEBUG
export ENABLE_SQL_LOGGING=true
python -m src.api_gateway.main_modular
```

#### Check Circuit Breaker Status
```python
from api_gateway.services.proxy import service_proxy

# Get circuit breaker status
status = service_proxy.get_circuit_breaker_status()
print(status)
```

#### Monitor Service Health
```bash
# Continuous health monitoring
watch -n 5 'curl -s http://localhost:8000/health/detailed | jq .'
```

### Performance Monitoring

#### Request Metrics
```bash
# Monitor request performance
tail -f /var/log/tracseq/gateway.log | grep "response_time_ms"
```

#### Database Performance
```bash
# Monitor database connections
curl -s http://localhost:8000/metrics | jq '.database'
```

#### Memory Usage
```bash
# Monitor memory usage
ps aux | grep python
top -p $(pgrep -f main_modular)
```

## 📚 Additional Resources

- [FastAPI Documentation](https://fastapi.tiangolo.com/)
- [Pydantic Documentation](https://pydantic-docs.helpmanual.io/)
- [PostgreSQL Documentation](https://www.postgresql.org/docs/)
- [Docker Documentation](https://docs.docker.com/)
- [Kubernetes Documentation](https://kubernetes.io/docs/)

## 🤝 Contributing

1. Fork the repository
2. Create a feature branch (`git checkout -b feature/new-feature`)
3. Make your changes
4. Add tests for new functionality
5. Run the test suite (`pytest`)
6. Commit your changes (`git commit -am 'Add new feature'`)
7. Push to the branch (`git push origin feature/new-feature`)
8. Create a Pull Request

## 📄 License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

## 📞 Support

For support and questions:
- Create an issue in the repository
- Contact the development team
- Check the troubleshooting guide above

---

*Context improved by Giga AI*
