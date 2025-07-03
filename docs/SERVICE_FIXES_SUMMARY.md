# Service Fixes Summary

## Date: 2025-07-03

## Overview
Fixed issues with three services that were failing health checks:
1. Transaction Service
2. Template Service  
3. Notification Service

## Root Cause
The services had port misconfigurations between:
- The ports the services were actually listening on
- The ports configured in Docker Compose environment variables
- The ports mapped in Docker Compose
- The ports used in Dockerfile HEALTHCHECK commands

## Fixes Applied

### 1. Transaction Service
- **Issue**: Service listens on port 8088, but Docker Compose was setting `TRANSACTION_PORT: "8000"`
- **Fix**: Updated environment variable to `PORT: "8088"` and port mapping to `8017:8088`
- **Status**: ✅ Healthy

### 2. Template Service
- **Issue**: Service uses `TEMPLATE_PORT` environment variable (not `PORT`) and listens on port 8083
- **Fix**: 
  - Updated environment variable from `PORT: "8000"` to `TEMPLATE_PORT: "8083"`
  - Updated port mapping from `8018:8000` to `8018:8083`
  - Updated Dockerfile EXPOSE from 8000 to 8083
  - Updated Dockerfile HEALTHCHECK to use port 8083
- **Status**: ✅ Healthy

### 3. Notification Service
- **Issue**: Service listens on port 8085, but Docker Compose was setting `PORT: "8000"`
- **Fix**: 
  - Updated environment variable from `PORT: "8000"` to `PORT: "8085"`
  - Updated port mapping from `8015:8000` to `8015:8085`
  - Updated Dockerfile EXPOSE from 8000 to 8085
  - Updated Dockerfile HEALTHCHECK to use port 8085
- **Status**: ✅ Healthy

## Files Modified
1. `docker/docker-compose.enhanced-services.yml` - Updated port configurations
2. `lims-enhanced/notification_service/Dockerfile` - Updated EXPOSE and HEALTHCHECK
3. `lims-core/template_service/Dockerfile` - Updated EXPOSE and HEALTHCHECK

## Verification
All services are now healthy and responding correctly:
- Transaction Service: http://localhost:8017/health ✅
- Notification Service: http://localhost:8015/health ✅
- Template Service: http://localhost:8018/health ✅

## Additional Issue: Template Upload Endpoint (404 Error)

### Root Cause
The template upload endpoint was returning 404 due to multiple routing issues:
1. **Lab Manager Proxy**: Wrong port (8000 instead of 8083) and incorrect path prefix
2. **API Gateway**: Forwarding to wrong path (`/api/v1/upload` instead of `/templates/upload`)
3. **Frontend Nginx**: Using wrong container name (`api-gateway` instead of `lims-gateway`)

### Fixes Applied
1. **Lab Manager** (`lims-laboratory/lab_manager/src/services/proxy_service.rs`):
   - Updated template service URL from port 8000 to 8083
2. **Lab Manager** (`lims-laboratory/lab_manager/src/handlers/proxy_handlers.rs`):
   - Changed path format from `/api/templates/{path}` to `/templates/{path}`
3. **API Gateway** (`lims-gateway/api_gateway/src/api_gateway/simple_main.py`):
   - Updated TEMPLATE_SERVICE_URL default port from 8000 to 8083
   - Changed proxy path from `/api/v1/{path}` to `/templates/{path}`
4. **Frontend** (`lims-ui/nginx.conf`):
   - Updated upstream server from `api-gateway:8000` to `lims-gateway:8000`

### Result
✅ Template upload endpoint now working: `POST /api/templates/upload` returns 200 OK

## Commands Used
```bash
# Recreate containers with new configuration
docker-compose -f docker-compose.yml -f docker-compose.enhanced-services.yml up -d --force-recreate notification-service template-service transaction-service

# Rebuild services with updated Dockerfiles
docker-compose -f docker-compose.yml -f docker-compose.enhanced-services.yml build notification-service template-service

# Rebuild and recreate API Gateway
cd docker
docker-compose build api-gateway
docker-compose up -d --force-recreate api-gateway

# Rebuild and recreate Frontend
docker-compose build frontend
docker-compose up -d --force-recreate frontend

# Check health status
docker ps --format "table {{.Names}}\t{{.Status}}\t{{.Ports}}" | grep -E "lims-notification|lims-templates|lims-transactions"

# Test template upload endpoint
curl -X POST http://localhost:3000/api/templates/upload -H "Content-Type: application/json" -d '{"test": "data"}'
``` 