# TracSeq 2.0 - API Gateway Quick Reference

## Quick Start

### Gateway URL
```bash
# Development
http://localhost:8089

# Production
https://api.tracseq.com
```

### Essential Endpoints

| Endpoint | Purpose | Example |
|----------|---------|---------|
| `GET /` | Gateway info | `curl http://localhost:8089/` |
| `GET /health` | Gateway health | `curl http://localhost:8089/health` |
| `GET /services` | Service discovery | `curl http://localhost:8089/services` |
| `GET /docs` | API documentation | `http://localhost:8089/docs` |

## Service Routes

### Route Patterns
```bash
/api/dashboard/*    → dashboard-service:8080
/api/samples/*      → samples-service:8081
/api/sequencing/*   → sequencing-service:8082
/api/spreadsheet/*  → spreadsheet-service:8083
/api/templates/*    → spreadsheet-service:8083
/api/auth/*         → dashboard-service:8080
/api/storage/*      → dashboard-service:8080
```

### Common API Calls

#### Samples
```bash
# Get all samples
curl http://localhost:8089/api/samples/v1/samples

# Create sample
curl -X POST http://localhost:8089/api/samples/v1/samples \
  -H "Content-Type: application/json" \
  -d '{"name": "Test Sample", "sample_type": "DNA"}'
```

#### Sequencing
```bash
# Get sequencing jobs
curl http://localhost:8089/api/sequencing/v1/jobs

# Get platforms
curl http://localhost:8089/api/sequencing/v1/platforms
```

#### Dashboard
```bash
# Get users
curl http://localhost:8089/api/dashboard/v1/users

# Get storage locations
curl http://localhost:8089/api/dashboard/v1/storage/locations
```

#### Templates
```bash
# Get templates
curl http://localhost:8089/api/templates/v1/templates

# Get spreadsheets
curl http://localhost:8089/api/spreadsheet/v1/spreadsheets
```

## Error Codes

| Code | Meaning | Action |
|------|---------|--------|
| 404 | Route not found | Check route pattern |
| 502 | Gateway error | Check gateway logs |
| 503 | Service unavailable | Check service health |
| 504 | Gateway timeout | Check service performance |

## Development Commands

### Check Service Health
```bash
# All services
curl http://localhost:8089/services | jq

# Individual service
curl http://localhost:8089/api/samples/health
```

### Test Performance
```bash
# Response time
curl -w "Time: %{time_total}s\n" http://localhost:8089/api/samples/v1/samples

# Service discovery with timing
curl -w "Total: %{time_total}s\n" http://localhost:8089/services
```

### Debug Issues
```bash
# Check Docker containers
docker ps

# Check gateway logs
docker logs tracseq-api-gateway

# Check service logs
docker logs tracseq-samples-service
```

## Configuration

### Environment Variables
```bash
# Service URLs
DASHBOARD_SERVICE_URL=http://dashboard-service:8080
SAMPLES_SERVICE_URL=http://samples-service:8081
SEQUENCING_SERVICE_URL=http://sequencing-service:8082
SPREADSHEET_SERVICE_URL=http://spreadsheet-service:8083

# Gateway settings
LOG_LEVEL=INFO
REQUEST_TIMEOUT=30
CORS_ORIGINS=*
```

### Docker Compose
```yaml
api-gateway:
  build: ./simple-services/api-gateway
  ports:
    - "8089:8000"
  environment:
    - LOG_LEVEL=INFO
  depends_on:
    - dashboard-service
    - samples-service
    - sequencing-service
    - spreadsheet-service
```

## Adding New Services

### 1. Update Routes
```python
# In simple-services/api-gateway/app.py
SERVICE_ROUTES = {
    # Existing routes...
    "/api/newservice": "http://newservice:8080",
}
```

### 2. Add Environment Variable
```bash
NEWSERVICE_URL=http://newservice:8080
```

### 3. Update Docker Compose
```yaml
newservice:
  build: ./simple-services/newservice
  ports:
    - "8084:8080"
  networks:
    - tracseq-network
```

### 4. Test New Service
```bash
# Test health
curl http://localhost:8089/api/newservice/health

# Check service discovery
curl http://localhost:8089/services
```

## Common Patterns

### Request/Response Format
```json
{
  "success": true,
  "data": {
    "items": [...],
    "total_count": 10,
    "pagination": {
      "page": 1,
      "per_page": 50
    }
  }
}
```

### Error Format
```json
{
  "detail": "Service unavailable: http://samples-service:8081"
}
```

### Health Check Format
```json
{
  "status": "healthy",
  "service": "service-name",
  "timestamp": "2024-12-15T10:30:00.123Z",
  "version": "1.0.0"
}
```

## Troubleshooting

### Service Not Responding
```bash
# 1. Check service discovery
curl http://localhost:8089/services

# 2. Check individual service
curl http://localhost:8081/health

# 3. Check Docker containers
docker ps | grep tracseq

# 4. Check logs
docker logs tracseq-samples-service
```

### CORS Issues
```bash
# Test CORS
curl -H "Origin: http://localhost:3000" \
     -H "Access-Control-Request-Method: GET" \
     -X OPTIONS \
     http://localhost:8089/api/samples/v1/samples
```

### Performance Issues
```bash
# Check response times
curl -w "DNS: %{time_namelookup}s, Connect: %{time_connect}s, Total: %{time_total}s\n" \
     http://localhost:8089/api/samples/v1/samples

# Monitor service health
watch -n 5 'curl -s http://localhost:8089/services | jq ".healthy_services"'
```

## Useful Tools

### jq for JSON Processing
```bash
# Pretty print service discovery
curl -s http://localhost:8089/services | jq

# Extract service statuses
curl -s http://localhost:8089/services | jq '.services[] | {route: .url, status: .status}'

# Count healthy services
curl -s http://localhost:8089/services | jq '.healthy_services'
```

### httpie for API Testing
```bash
# Install httpie
pip install httpie

# GET request
http GET localhost:8089/api/samples/v1/samples

# POST request
http POST localhost:8089/api/samples/v1/samples name="Test Sample" sample_type="DNA"
```

### curl with JSON
```bash
# POST with JSON
curl -X POST http://localhost:8089/api/samples/v1/samples \
  -H "Content-Type: application/json" \
  -d '{"name": "Test Sample", "sample_type": "DNA"}'

# PUT with JSON
curl -X PUT http://localhost:8089/api/samples/v1/samples/sample-001 \
  -H "Content-Type: application/json" \
  -d '{"status": "validated"}'
```

## Monitoring

### Health Checks
```bash
# Gateway health
curl http://localhost:8089/health

# All services health
curl http://localhost:8089/services

# Individual service health
curl http://localhost:8089/api/samples/health
```

### Performance Monitoring
```bash
# Response time monitoring
curl -w "@curl-format.txt" http://localhost:8089/api/samples/v1/samples

# Create curl-format.txt:
echo "     time_namelookup:  %{time_namelookup}\n
        time_connect:  %{time_connect}\n
     time_appconnect:  %{time_appconnect}\n
    time_pretransfer:  %{time_pretransfer}\n
       time_redirect:  %{time_redirect}\n
  time_starttransfer:  %{time_starttransfer}\n
                     ----------\n
          time_total:  %{time_total}\n" > curl-format.txt
```

### Log Monitoring
```bash
# Follow gateway logs
docker logs -f tracseq-api-gateway

# Follow all service logs
docker-compose logs -f

# Filter logs by service
docker-compose logs -f api-gateway
```

## Security

### CORS Configuration
```python
# Development (permissive)
allow_origins=["*"]

# Production (restrictive)
allow_origins=["https://app.tracseq.com", "https://tracseq.com"]
```

### Authentication Headers
```bash
# With JWT token
curl -H "Authorization: Bearer <token>" \
     http://localhost:8089/api/samples/v1/samples

# With API key
curl -H "X-API-Key: <api-key>" \
     http://localhost:8089/api/samples/v1/samples
```

## Best Practices

### Development
1. Always check service health before testing
2. Use structured logging for debugging
3. Test with different HTTP methods
4. Validate request/response formats
5. Handle errors gracefully

### Production
1. Use HTTPS for all communications
2. Implement proper authentication
3. Monitor service health continuously
4. Set up alerting for service failures
5. Use load balancing for high availability

### Testing
1. Test individual services directly
2. Test through API Gateway
3. Test error scenarios
4. Test performance under load
5. Test service discovery functionality

---

*Quick Reference Guide v1.0*  
*For detailed documentation, see: API_GATEWAY_DOCUMENTATION.md* 