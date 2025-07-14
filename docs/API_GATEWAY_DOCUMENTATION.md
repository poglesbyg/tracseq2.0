# TracSeq 2.0 - API Gateway Documentation

## Overview

The TracSeq 2.0 API Gateway serves as the central entry point for all client requests, providing service routing, load balancing, and centralized logging. Built with FastAPI, it offers high performance, automatic documentation, and comprehensive monitoring capabilities.

## Architecture

### System Architecture

```
┌─────────────────┐    ┌─────────────────┐    ┌─────────────────┐
│   Frontend      │    │  External       │    │   Mobile        │
│   (React/Web)   │    │  Applications   │    │   Applications  │
└─────────┬───────┘    └─────────┬───────┘    └─────────┬───────┘
          │                      │                      │
          └──────────────────────┼──────────────────────┘
                                 │
                    ┌─────────────▼─────────────┐
                    │     API Gateway           │
                    │   (Port 8089/8000)        │
                    │   - Request Routing       │
                    │   - Service Discovery     │
                    │   - Load Balancing        │
                    │   - Centralized Logging   │
                    └─────────────┬─────────────┘
                                 │
        ┌────────────────────────┼────────────────────────┐
        │                       │                        │
┌───────▼──────┐    ┌───────────▼──────┐    ┌───────────▼──────┐
│  Dashboard   │    │    Samples       │    │   Sequencing     │
│  Service     │    │    Service       │    │   Service        │
│  (Port 8080) │    │   (Port 8081)    │    │   (Port 8082)    │
└──────────────┘    └──────────────────┘    └──────────────────┘
        │                       │                        │
┌───────▼──────┐    ┌───────────▼──────┐    ┌───────────▼──────┐
│ Spreadsheet  │    │   PostgreSQL     │    │     Redis        │
│ Service      │    │   Database       │    │     Cache        │
│ (Port 8083)  │    │   (Port 5433)    │    │   (Port 6379)    │
└──────────────┘    └──────────────────┘    └──────────────────┘
```

### Gateway Components

#### 1. **Request Router**
- **Purpose**: Routes incoming requests to appropriate backend services
- **Technology**: FastAPI with custom routing logic
- **Features**:
  - Path-based routing
  - Service discovery integration
  - Health check routing
  - Error handling and fallback

#### 2. **Service Discovery**
- **Purpose**: Maintains registry of available services and their health status
- **Endpoint**: `/services`
- **Features**:
  - Real-time service health monitoring
  - Service registration and deregistration
  - Load balancing capabilities
  - Service metadata management

#### 3. **Request Logging**
- **Purpose**: Centralized logging for all API requests
- **Technology**: Structured JSON logging with request tracking
- **Features**:
  - Request/response logging
  - Performance metrics
  - Error tracking
  - Distributed tracing support

#### 4. **CORS Handling**
- **Purpose**: Enable cross-origin requests from frontend applications
- **Configuration**: Configurable origins, methods, and headers
- **Security**: Production-ready CORS policies

## Service Routing Configuration

### Current Service Routes

| Route Pattern | Target Service | Port | Description |
|---------------|---------------|------|-------------|
| `/api/dashboard/*` | Dashboard Service | 8080 | User management, system overview |
| `/api/samples/*` | Samples Service | 8081 | Sample management and tracking |
| `/api/sequencing/*` | Sequencing Service | 8082 | Sequencing jobs and platforms |
| `/api/spreadsheet/*` | Spreadsheet Service | 8083 | Data templates and spreadsheets |
| `/api/templates/*` | Spreadsheet Service | 8083 | Template management |
| `/api/auth/*` | Dashboard Service | 8080 | Authentication and authorization |
| `/api/storage/*` | Dashboard Service | 8080 | Storage location management |

### Routing Logic

```python
SERVICE_ROUTES = {
    "/api/dashboard": "http://dashboard-service:8080",
    "/api/samples": "http://samples-service:8081",
    "/api/sequencing": "http://sequencing-service:8082",
    "/api/spreadsheet": "http://spreadsheet-service:8083",
    "/api/templates": "http://spreadsheet-service:8083",
    "/api/auth": "http://dashboard-service:8080",
    "/api/storage": "http://dashboard-service:8080",
}
```

### Path Transformation

The gateway transforms incoming paths before forwarding to services:

1. **Incoming Request**: `GET /api/samples/v1/samples`
2. **Route Matching**: Matches `/api/samples/*` → `samples-service:8081`
3. **Path Transformation**: `/api/samples/v1/samples` → `/api/v1/samples`
4. **Target URL**: `http://samples-service:8081/api/v1/samples`

## API Gateway Endpoints

### Core Gateway Endpoints

#### **GET /** - Root Information
```bash
curl http://localhost:8089/
```

**Response:**
```json
{
  "service": "TracSeq API Gateway (Simple)",
  "version": "1.0.0",
  "status": "operational",
  "available_routes": [
    "/api/dashboard",
    "/api/samples",
    "/api/sequencing",
    "/api/spreadsheet",
    "/api/templates",
    "/api/auth",
    "/api/storage"
  ],
  "docs": "/docs",
  "health": "/health"
}
```

#### **GET /health** - Gateway Health Check
```bash
curl http://localhost:8089/health
```

**Response:**
```json
{
  "status": "healthy",
  "service": "TracSeq API Gateway (Simple)",
  "version": "1.0.0",
  "timestamp": 1702648800.123,
  "services": [
    "/api/dashboard",
    "/api/samples",
    "/api/sequencing",
    "/api/spreadsheet",
    "/api/templates",
    "/api/auth",
    "/api/storage"
  ]
}
```

#### **GET /services** - Service Discovery
```bash
curl http://localhost:8089/services
```

**Response:**
```json
{
  "gateway": "TracSeq API Gateway (Simple)",
  "services": {
    "/api/dashboard": {
      "url": "http://dashboard-service:8080",
      "status": "healthy",
      "response_time": 0.045
    },
    "/api/samples": {
      "url": "http://samples-service:8081",
      "status": "healthy",
      "response_time": 0.032
    },
    "/api/sequencing": {
      "url": "http://sequencing-service:8082",
      "status": "healthy",
      "response_time": 0.028
    },
    "/api/spreadsheet": {
      "url": "http://spreadsheet-service:8083",
      "status": "healthy",
      "response_time": 0.041
    }
  },
  "total_services": 4,
  "healthy_services": 4
}
```

#### **GET /docs** - Interactive API Documentation
```bash
curl http://localhost:8089/docs
```
- **Purpose**: Swagger UI for interactive API documentation
- **Features**: Test endpoints, view schemas, authentication testing

## Backend Service Endpoints

### Dashboard Service (`/api/dashboard/*`)

#### **GET /api/dashboard/health**
```bash
curl http://localhost:8089/api/dashboard/health
```

#### **GET /api/dashboard/v1/users**
```bash
curl http://localhost:8089/api/dashboard/v1/users
```

**Response:**
```json
{
  "success": true,
  "data": {
    "users": [
      {
        "id": "user-001",
        "name": "Dr. Smith",
        "email": "smith@lab.com",
        "department": "Genomics",
        "role": "researcher",
        "created_at": "2024-01-15T10:00:00Z"
      }
    ]
  }
}
```

#### **GET /api/dashboard/v1/storage/locations**
```bash
curl http://localhost:8089/api/dashboard/v1/storage/locations
```

**Response:**
```json
{
  "success": true,
  "data": {
    "locations": [
      {
        "id": "loc-001",
        "name": "Freezer A1-B2",
        "temperature": -80,
        "capacity": 100,
        "current_usage": 45,
        "status": "operational"
      }
    ]
  }
}
```

### Samples Service (`/api/samples/*`)

#### **GET /api/samples/health**
```bash
curl http://localhost:8089/api/samples/health
```

#### **GET /api/samples/v1/samples**
```bash
curl http://localhost:8089/api/samples/v1/samples
```

**Response:**
```json
{
  "success": true,
  "data": {
    "samples": [
      {
        "id": "sample-001",
        "name": "DNA Sample 001",
        "barcode": "DNA-001-2024",
        "sample_type": "DNA",
        "status": "validated",
        "concentration": 150.5,
        "volume": 2.0,
        "storage_location": "Freezer A1-B2",
        "submitter": "Dr. Smith",
        "department": "Genomics",
        "created_at": "2024-12-15T10:30:00Z",
        "metadata": {
          "patient_id": "P12345",
          "collection_date": "2024-12-14",
          "analysis_type": "WGS",
          "priority": "high"
        }
      }
    ],
    "total_count": 3,
    "pagination": {
      "page": 1,
      "per_page": 50,
      "pages": 1
    }
  }
}
```

#### **POST /api/samples/v1/samples**
```bash
curl -X POST http://localhost:8089/api/samples/v1/samples \
  -H "Content-Type: application/json" \
  -d '{
    "name": "New DNA Sample",
    "sample_type": "DNA",
    "concentration": 125.0,
    "volume": 1.5,
    "storage_location": "Freezer A2-B3",
    "submitter": "Dr. Johnson"
  }'
```

### Sequencing Service (`/api/sequencing/*`)

#### **GET /api/sequencing/health**
```bash
curl http://localhost:8089/api/sequencing/health
```

#### **GET /api/sequencing/v1/jobs**
```bash
curl http://localhost:8089/api/sequencing/v1/jobs
```

**Response:**
```json
{
  "success": true,
  "data": {
    "jobs": [
      {
        "id": "seq-001",
        "name": "WGS Sample Batch 1",
        "status": "running",
        "sample_count": 24,
        "platform": "NovaSeq 6000",
        "run_type": "Paired-end 150bp",
        "progress": 65,
        "submitter": "Dr. Smith",
        "priority": "high"
      }
    ],
    "total_count": 4,
    "status_counts": {
      "running": 1,
      "queued": 1,
      "completed": 1,
      "failed": 1
    }
  }
}
```

#### **GET /api/sequencing/v1/platforms**
```bash
curl http://localhost:8089/api/sequencing/v1/platforms
```

**Response:**
```json
{
  "success": true,
  "data": {
    "platforms": [
      {
        "id": "novaseq6000",
        "name": "NovaSeq 6000",
        "manufacturer": "Illumina",
        "status": "available",
        "throughput": "High",
        "read_lengths": ["50bp", "100bp", "150bp"]
      }
    ]
  }
}
```

### Spreadsheet Service (`/api/spreadsheet/*` and `/api/templates/*`)

#### **GET /api/spreadsheet/health**
```bash
curl http://localhost:8089/api/spreadsheet/health
```

#### **GET /api/spreadsheet/v1/spreadsheets**
```bash
curl http://localhost:8089/api/spreadsheet/v1/spreadsheets
```

**Response:**
```json
{
  "success": true,
  "data": {
    "datasets": [
      {
        "id": "dataset-001",
        "name": "Lab Results Q4 2024",
        "description": "Quarterly laboratory results compilation",
        "file_type": "xlsx",
        "size_mb": 2.5,
        "row_count": 1250,
        "column_count": 15,
        "status": "active"
      }
    ],
    "total_count": 3,
    "active_count": 2,
    "total_size_mb": 9.5
  }
}
```

#### **GET /api/templates/v1/templates**
```bash
curl http://localhost:8089/api/templates/v1/templates
```

**Response:**
```json
{
  "success": true,
  "data": {
    "templates": [
      {
        "id": "template-001",
        "name": "Sample Submission Template",
        "description": "Standard template for sample submissions",
        "version": "1.0",
        "created_at": "2024-01-01T00:00:00Z"
      }
    ]
  }
}
```

## Configuration

### Environment Variables

```bash
# Service URLs (Docker internal networking)
DASHBOARD_SERVICE_URL=http://dashboard-service:8080
SAMPLES_SERVICE_URL=http://samples-service:8081
SEQUENCING_SERVICE_URL=http://sequencing-service:8082
SPREADSHEET_SERVICE_URL=http://spreadsheet-service:8083

# Gateway Configuration
LOG_LEVEL=INFO
CORS_ORIGINS=*
REQUEST_TIMEOUT=30
HEALTH_CHECK_TIMEOUT=5

# Performance Settings
MAX_CONNECTIONS=100
KEEP_ALIVE_TIMEOUT=5
```

### Docker Configuration

```dockerfile
FROM python:3.11-slim

WORKDIR /app

# Install dependencies
COPY requirements.txt .
RUN pip install --no-cache-dir -r requirements.txt

# Copy application
COPY . .

# Create non-root user
RUN useradd -m -u 1000 appuser && chown -R appuser:appuser /app
USER appuser

# Expose port
EXPOSE 8000

# Health check
HEALTHCHECK --interval=30s --timeout=10s --start-period=5s --retries=3 \
    CMD curl -f http://localhost:8000/health || exit 1

# Run application
CMD ["python", "-m", "uvicorn", "app:app", "--host", "0.0.0.0", "--port", "8000"]
```

## Error Handling

### HTTP Status Codes

| Status Code | Description | When It Occurs |
|-------------|-------------|----------------|
| 200 | OK | Successful request |
| 404 | Not Found | No service found for path |
| 502 | Bad Gateway | Gateway error during proxying |
| 503 | Service Unavailable | Backend service connection failed |
| 504 | Gateway Timeout | Backend service timeout |

### Error Response Format

```json
{
  "detail": "Service unavailable: http://samples-service:8081"
}
```

### Error Scenarios

#### **Service Unavailable (503)**
```bash
# When backend service is down
curl http://localhost:8089/api/samples/v1/samples
```

**Response:**
```json
{
  "detail": "Service unavailable: http://samples-service:8081"
}
```

#### **Service Timeout (504)**
```bash
# When backend service takes too long to respond
curl http://localhost:8089/api/samples/v1/samples
```

**Response:**
```json
{
  "detail": "Service timeout: http://samples-service:8081"
}
```

#### **Route Not Found (404)**
```bash
# When requesting non-existent route
curl http://localhost:8089/api/nonexistent/endpoint
```

**Response:**
```json
{
  "detail": "No service found for path: /api/nonexistent/endpoint"
}
```

## Logging and Monitoring

### Structured Logging

The API Gateway uses structured JSON logging for all requests:

```json
{
  "timestamp": "2024-12-15T10:30:00.123Z",
  "level": "INFO",
  "service": "api-gateway",
  "logger": "api-gateway.requests",
  "message": "Incoming request",
  "request_id": "api-gateway-1702648800123-140234567890",
  "http_method": "GET",
  "http_url": "http://localhost:8089/api/samples/v1/samples",
  "http_path": "/api/samples/v1/samples",
  "client_ip": "127.0.0.1",
  "user_agent": "curl/7.68.0"
}
```

### Performance Metrics

```json
{
  "timestamp": "2024-12-15T10:30:00.456Z",
  "level": "INFO",
  "service": "api-gateway",
  "logger": "api-gateway.requests",
  "message": "Request completed",
  "request_id": "api-gateway-1702648800123-140234567890",
  "http_status_code": 200,
  "processing_time_ms": 45.67,
  "response_size_bytes": "1024"
}
```

### Health Check Monitoring

```bash
# Check gateway health
curl http://localhost:8089/health

# Check all services health
curl http://localhost:8089/services
```

## Security

### CORS Configuration

```python
app.add_middleware(
    CORSMiddleware,
    allow_origins=["*"],  # Configure for production
    allow_credentials=True,
    allow_methods=["*"],
    allow_headers=["*"],
)
```

### Production Security Recommendations

1. **CORS Origins**: Restrict to specific frontend domains
2. **Request Rate Limiting**: Implement rate limiting middleware
3. **Authentication**: Add JWT token validation
4. **HTTPS**: Use HTTPS in production
5. **Request Validation**: Validate all incoming requests
6. **Security Headers**: Add security headers middleware

## Deployment

### Docker Compose

```yaml
version: '3.8'

services:
  api-gateway:
    build: ./simple-services/api-gateway
    ports:
      - "8089:8000"
    environment:
      - DASHBOARD_SERVICE_URL=http://dashboard-service:8080
      - SAMPLES_SERVICE_URL=http://samples-service:8081
      - SEQUENCING_SERVICE_URL=http://sequencing-service:8082
      - SPREADSHEET_SERVICE_URL=http://spreadsheet-service:8083
      - LOG_LEVEL=INFO
    depends_on:
      - dashboard-service
      - samples-service
      - sequencing-service
      - spreadsheet-service
    networks:
      - tracseq-network

  dashboard-service:
    build: ./simple-services/dashboard-service
    ports:
      - "8080:8080"
    networks:
      - tracseq-network

  samples-service:
    build: ./simple-services/samples-service
    ports:
      - "8081:8080"
    networks:
      - tracseq-network

  sequencing-service:
    build: ./simple-services/sequencing-service
    ports:
      - "8082:8080"
    networks:
      - tracseq-network

  spreadsheet-service:
    build: ./simple-services/spreadsheet-service
    ports:
      - "8083:8080"
    networks:
      - tracseq-network

networks:
  tracseq-network:
    driver: bridge
```

### Kubernetes Deployment

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
        - name: DASHBOARD_SERVICE_URL
          value: "http://dashboard-service:8080"
        - name: SAMPLES_SERVICE_URL
          value: "http://samples-service:8081"
        - name: SEQUENCING_SERVICE_URL
          value: "http://sequencing-service:8082"
        - name: SPREADSHEET_SERVICE_URL
          value: "http://spreadsheet-service:8083"
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
---
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

## Development Guide

### Adding New Services

1. **Update Service Routes**:
```python
SERVICE_ROUTES = {
    # Existing routes...
    "/api/newservice": "http://newservice:8080",
}
```

2. **Add Environment Variable**:
```bash
NEWSERVICE_URL=http://newservice:8080
```

3. **Update Docker Compose**:
```yaml
services:
  newservice:
    build: ./simple-services/newservice
    ports:
      - "8084:8080"
    networks:
      - tracseq-network
```

### Testing New Routes

```bash
# Test new service health
curl http://localhost:8089/api/newservice/health

# Test new service endpoints
curl http://localhost:8089/api/newservice/v1/data

# Check service discovery
curl http://localhost:8089/services
```

### Custom Middleware

```python
from fastapi import Request, Response
from fastapi.middleware.base import BaseHTTPMiddleware

class CustomMiddleware(BaseHTTPMiddleware):
    async def dispatch(self, request: Request, call_next):
        # Custom logic before request
        response = await call_next(request)
        # Custom logic after request
        return response

app.add_middleware(CustomMiddleware)
```

## Troubleshooting

### Common Issues

#### **Service Connection Refused**
```bash
# Check service status
curl http://localhost:8089/services

# Check individual service
curl http://localhost:8080/health

# Check Docker containers
docker ps
docker logs <container_name>
```

#### **Gateway Timeout**
```bash
# Check service response times
curl -w "%{time_total}\n" http://localhost:8089/api/samples/v1/samples

# Increase timeout in gateway configuration
REQUEST_TIMEOUT=60
```

#### **CORS Issues**
```bash
# Check CORS headers
curl -H "Origin: http://localhost:3000" \
     -H "Access-Control-Request-Method: GET" \
     -H "Access-Control-Request-Headers: X-Requested-With" \
     -X OPTIONS \
     http://localhost:8089/api/samples/v1/samples
```

### Debug Mode

```bash
# Enable debug logging
LOG_LEVEL=DEBUG

# Run with debug mode
uvicorn app:app --host 0.0.0.0 --port 8000 --log-level debug
```

### Performance Monitoring

```bash
# Monitor request performance
curl -w "Total time: %{time_total}s\n" http://localhost:8089/api/samples/v1/samples

# Check gateway metrics
curl http://localhost:8089/services | jq '.services[] | {route: .url, status: .status, response_time: .response_time}'
```

## Best Practices

### Development
1. **Use structured logging** for all requests and responses
2. **Implement health checks** for all services
3. **Add request validation** for all endpoints
4. **Use environment variables** for configuration
5. **Implement graceful error handling**

### Production
1. **Configure CORS** for specific origins
2. **Use HTTPS** for all communications
3. **Implement rate limiting** to prevent abuse
4. **Add authentication** middleware
5. **Monitor service health** continuously
6. **Use load balancing** for high availability

### Security
1. **Validate all inputs** before forwarding
2. **Sanitize headers** to prevent injection
3. **Implement request size limits**
4. **Add security headers** to responses
5. **Log security events** for monitoring

## Conclusion

The TracSeq 2.0 API Gateway provides a robust, scalable, and well-documented entry point for all system interactions. With comprehensive logging, service discovery, and error handling, it forms the foundation for a production-ready laboratory management system.

For additional support or questions about the API Gateway, refer to the interactive documentation at `/docs` or contact the development team.

---

*Generated: December 2024*  
*Version: API Gateway Documentation v1.0*  
*Architecture: Microservices with Centralized Gateway* 