# TracSeq 2.0 Microservices Testing Guide

This guide provides instructions for testing the microservices migration and verifying the proxy functionality.

## Prerequisites

- Docker and Docker Compose installed
- Rust toolchain installed
- PostgreSQL client tools (optional, for database verification)
- `jq` command-line tool (for JSON parsing)

## Testing Scenarios

### 1. Monolith Mode Testing

Start the system in monolith mode (default):

```bash
# Start only required infrastructure
docker-compose -f docker-compose.yml up -d postgres redis

# Start lab_manager without proxy mode
export ENABLE_PROXY_MODE=false
cd lab_manager
cargo run
```

Test endpoints:
```bash
# Health check
curl http://localhost:8080/health

# Sample endpoints
curl http://localhost:8080/api/samples
curl -X POST http://localhost:8080/api/samples \
  -H "Content-Type: application/json" \
  -d '{"name": "Test Sample", "type": "DNA"}'

# Auth endpoints (if using local auth)
curl -X POST http://localhost:8080/api/auth/login \
  -H "Content-Type: application/json" \
  -d '{"username": "admin", "password": "admin123"}'
```

### 2. Microservices Mode Testing

Start all microservices:

```bash
# Use the microservices docker-compose
docker-compose -f docker-compose.microservices.yml up -d

# Wait for services to be ready
sleep 30

# Check service health
./scripts/migrate-to-microservices.sh status
```

Test individual microservices:
```bash
# Auth service
curl http://localhost:3010/health

# Sample service
curl http://localhost:3011/health

# Sequencing service
curl http://localhost:3012/health

# Template service
curl http://localhost:3013/health
```

### 3. Proxy Mode Testing

Enable proxy mode and test routing:

```bash
# Set proxy mode environment variable
export ENABLE_PROXY_MODE=true

# Restart lab_manager or use docker-compose
docker-compose -f docker-compose.microservices.yml up -d lab-manager-proxy

# Check service discovery
curl http://localhost:8080/api/services/discovery | jq .

# Check microservices health through proxy
curl http://localhost:8080/api/services/health | jq .
```

### 4. API Gateway Testing

Test the Python API Gateway:

```bash
# API Gateway health
curl http://localhost:8000/health

# Test routing through API Gateway
curl http://localhost:8000/api/v1/samples
curl http://localhost:8000/api/v1/auth/status
```

## Automated Testing Script

Run the complete test suite:

```bash
# Run all migration tests
./scripts/migrate-to-microservices.sh test

# Compare monolith vs microservices responses
./scripts/migrate-to-microservices.sh compare

# Perform full migration test
./scripts/migrate-to-microservices.sh migrate
```

## Performance Testing

### Load Testing with curl

```bash
# Simple load test
for i in {1..100}; do
  curl -s http://localhost:8080/api/samples &
done
wait
```

### Using Apache Bench (ab)

```bash
# Test monolith performance
ab -n 1000 -c 10 http://localhost:8080/api/samples

# Test microservices performance
ab -n 1000 -c 10 http://localhost:8000/api/v1/samples
```

## Debugging

### Check Logs

```bash
# View all service logs
docker-compose -f docker-compose.microservices.yml logs -f

# View specific service logs
docker-compose -f docker-compose.microservices.yml logs -f auth-service
docker-compose -f docker-compose.microservices.yml logs -f sample-service
docker-compose -f docker-compose.microservices.yml logs -f lab-manager-proxy
```

### Database Verification

```bash
# Connect to PostgreSQL
docker exec -it tracseq20_postgres_1 psql -U postgres

# List databases
\l

# Connect to a specific database
\c tracseq_samples

# List tables
\dt
```

### Network Debugging

```bash
# Check service connectivity
docker-compose -f docker-compose.microservices.yml exec lab-manager-proxy ping auth-service
docker-compose -f docker-compose.microservices.yml exec lab-manager-proxy curl http://auth-service:8080/health
```

## Common Issues and Solutions

### Issue: Services not responding

**Solution:**
```bash
# Check if services are running
docker-compose -f docker-compose.microservices.yml ps

# Restart specific service
docker-compose -f docker-compose.microservices.yml restart auth-service

# Check service logs for errors
docker-compose -f docker-compose.microservices.yml logs auth-service | grep ERROR
```

### Issue: Database connection errors

**Solution:**
```bash
# Ensure databases are created
docker exec -it tracseq20_postgres_1 psql -U postgres -c "\l" | grep tracseq

# Re-run initialization script if needed
docker exec -it tracseq20_postgres_1 psql -U postgres -f /docker-entrypoint-initdb.d/init.sql
```

### Issue: Proxy mode not working

**Solution:**
```bash
# Verify environment variable
echo $ENABLE_PROXY_MODE

# Check proxy service initialization
docker-compose -f docker-compose.microservices.yml logs lab-manager-proxy | grep "Proxy mode"

# Test service discovery endpoint
curl http://localhost:8080/api/services/discovery
```

## Integration Testing

### Test Data Flow

1. **Create a sample through the proxy**:
```bash
# Create sample
SAMPLE_ID=$(curl -X POST http://localhost:8080/api/samples \
  -H "Content-Type: application/json" \
  -d '{"name": "Integration Test Sample", "type": "RNA"}' | jq -r '.id')

echo "Created sample: $SAMPLE_ID"
```

2. **Retrieve the sample**:
```bash
# Get sample details
curl http://localhost:8080/api/samples/$SAMPLE_ID | jq .
```

3. **Update the sample**:
```bash
# Update sample
curl -X PUT http://localhost:8080/api/samples/$SAMPLE_ID \
  -H "Content-Type: application/json" \
  -d '{"status": "processed"}' | jq .
```

### Test Cross-Service Communication

```bash
# Test auth service integration
TOKEN=$(curl -X POST http://localhost:8080/api/auth/login \
  -H "Content-Type: application/json" \
  -d '{"username": "test", "password": "test123"}' | jq -r '.token')

# Use token for authenticated requests
curl http://localhost:8080/api/samples \
  -H "Authorization: Bearer $TOKEN"
```

## Monitoring

### Health Dashboard

Create a simple monitoring script:

```bash
#!/bin/bash
# monitor.sh

while true; do
  clear
  echo "=== TracSeq 2.0 Service Health ==="
  echo "Time: $(date)"
  echo ""
  
  # Check each service
  for port in 8080 8000 3010 3011 3012 3013 3014 3015; do
    if curl -f -s "http://localhost:${port}/health" > /dev/null 2>&1; then
      echo "Port $port: ✓ Healthy"
    else
      echo "Port $port: ✗ Unhealthy"
    fi
  done
  
  sleep 5
done
```

## Next Steps

1. Implement comprehensive integration tests
2. Set up continuous monitoring
3. Configure load balancing for high availability
4. Implement service mesh for advanced traffic management
5. Set up distributed tracing with Jaeger
6. Configure centralized logging with ELK stack