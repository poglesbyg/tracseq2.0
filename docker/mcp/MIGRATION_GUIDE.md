# MCP Docker Migration Guide

## Overview

This guide walks you through migrating your TracSeq 2.0 installation from a traditional deployment to the MCP-enabled Docker infrastructure.

## Prerequisites

- Docker 20.10+ and Docker Compose 2.0+ installed
- Access to existing TracSeq 2.0 database
- Backup of all critical data
- Understanding of MCP architecture (see MCP_INTEGRATION_STRATEGY.md)

## Migration Steps

### Step 1: Backup Existing Data

```bash
# 1. Backup PostgreSQL database
pg_dump -h localhost -U tracseq_user -d tracseq_db > backup_$(date +%Y%m%d).sql

# 2. Backup uploaded files and documents
tar -czf documents_backup_$(date +%Y%m%d).tar.gz /path/to/documents

# 3. Backup AI models (if using local models)
tar -czf models_backup_$(date +%Y%m%d).tar.gz /path/to/models

# 4. Export environment variables
env | grep -E "TRACSEQ|DATABASE|JWT|OLLAMA" > env_backup_$(date +%Y%m%d).txt
```

### Step 2: Prepare Environment

1. **Clone the repository** (if not already done):
```bash
git clone https://github.com/your-org/tracseq2.0
cd tracseq2.0
```

2. **Create environment file**:
```bash
cd docker/mcp
cp env.example .env
# Edit .env with your specific values
nano .env
```

3. **Update configuration**:
```bash
# Update database connection string
DATABASE_URL=postgresql://postgres:postgres@postgres:5432/tracseq

# Update service URLs to use Docker service names
AUTH_SERVICE_URL=http://auth-service:8001
SAMPLE_SERVICE_URL=http://sample-service:8002
# etc.
```

### Step 3: Build MCP Docker Images

```bash
# Build all MCP images
./docker/mcp/build-mcp.sh

# Verify images were created
docker images | grep tracseq
```

### Step 4: Migrate Database

1. **Start only the database service**:
```bash
cd docker
docker-compose -f docker-compose.with-mcp.yml up -d postgres
```

2. **Import existing data**:
```bash
# Wait for PostgreSQL to be ready
sleep 10

# Create database if needed
docker exec postgres createdb -U postgres tracseq

# Import data
docker exec -i postgres psql -U postgres tracseq < backup_20240115.sql
```

3. **Run migrations** (if any):
```bash
# For each service that has migrations
docker-compose -f docker-compose.with-mcp.yml run --rm lab-manager sqlx migrate run
```

### Step 5: Migrate File Storage

1. **Create volume directories**:
```bash
sudo mkdir -p /opt/tracseq/{documents,models,logs,storage}
sudo chown -R 1000:1000 /opt/tracseq
```

2. **Copy existing files**:
```bash
# Extract document backup
tar -xzf documents_backup_20240115.tar.gz -C /opt/tracseq/documents

# Extract model backup
tar -xzf models_backup_20240115.tar.gz -C /opt/tracseq/models
```

### Step 6: Start MCP Infrastructure

```bash
# Start all services with MCP
./docker/mcp/start-mcp.sh

# Monitor startup
docker-compose -f docker-compose.with-mcp.yml logs -f
```

### Step 7: Verify Services

```bash
# Run the test script
./docker/mcp/test-mcp-docker.sh

# Check individual services
curl http://localhost:7890/health  # MCP Dashboard
curl http://localhost:8500/v1/catalog/services  # Consul
curl http://localhost:9590/metrics  # MCP Proxy metrics
```

### Step 8: Update Application Configuration

1. **Update frontend configuration**:
```javascript
// Update API endpoints to use the gateway
const API_BASE_URL = process.env.REACT_APP_API_URL || 'http://localhost:8089';
const MCP_DASHBOARD_URL = process.env.REACT_APP_MCP_DASHBOARD || 'http://localhost:7890';
```

2. **Update service clients** to use MCP when available:
```python
# Python services
if os.environ.get('MCP_ENABLED', 'false').lower() == 'true':
    from mcp_client import MCPClient
    client = MCPClient(os.environ.get('MCP_PROXY_URL'))
else:
    # Fallback to HTTP
    client = HTTPClient()
```

3. **Update Rust services**:
```rust
// Check if MCP is enabled
if env::var("MCP_ENABLED").unwrap_or_default() == "true" {
    let mcp_bridge = MCPBridge::new(config.mcp);
    // Use MCP for AI operations
} else {
    // Use traditional HTTP calls
}
```

### Step 9: Configure Load Balancer/Proxy

If you're using Nginx or another reverse proxy:

```nginx
# /etc/nginx/sites-available/tracseq
upstream mcp_dashboard {
    server localhost:7890;
}

upstream api_gateway {
    server localhost:8089;
}

server {
    listen 443 ssl http2;
    server_name tracseq.example.com;

    # MCP Dashboard
    location /mcp/ {
        proxy_pass http://mcp_dashboard/;
        proxy_http_version 1.1;
        proxy_set_header Upgrade $http_upgrade;
        proxy_set_header Connection "upgrade";
    }

    # API Gateway
    location /api/ {
        proxy_pass http://api_gateway/;
    }

    # WebSocket for MCP
    location /ws/ {
        proxy_pass http://localhost:9500/;
        proxy_http_version 1.1;
        proxy_set_header Upgrade $http_upgrade;
        proxy_set_header Connection "upgrade";
    }
}
```

### Step 10: Monitor and Optimize

1. **Check logs for errors**:
```bash
docker-compose -f docker-compose.with-mcp.yml logs | grep ERROR
```

2. **Monitor resource usage**:
```bash
docker stats
```

3. **Set up alerts** (optional):
```bash
# Access Consul UI
open http://localhost:8500

# View MCP Dashboard
open http://localhost:7890
```

## Rollback Plan

If you need to rollback:

```bash
# 1. Stop all services
docker-compose -f docker-compose.with-mcp.yml down

# 2. Restore database
psql -h localhost -U tracseq_user -d tracseq_db < backup_20240115.sql

# 3. Restore files
rm -rf /path/to/documents/*
tar -xzf documents_backup_20240115.tar.gz -C /path/to/documents

# 4. Start original services
# (Use your original startup method)
```

## Common Issues and Solutions

### Issue 1: Services Can't Connect
**Problem**: Services showing connection refused errors.
**Solution**:
```bash
# Check if services are on the same network
docker network inspect tracseq-network

# Restart services
docker-compose -f docker-compose.with-mcp.yml restart
```

### Issue 2: Database Migration Fails
**Problem**: Migration scripts fail to run.
**Solution**:
```bash
# Run migrations manually
docker exec -it lab-manager bash
sqlx migrate run

# Or revert and retry
sqlx migrate revert
sqlx migrate run
```

### Issue 3: MCP Services Not Registering
**Problem**: Services not appearing in Consul.
**Solution**:
```bash
# Check Consul logs
docker logs consul

# Manually register service
curl -X PUT -d @service.json http://localhost:8500/v1/agent/service/register
```

### Issue 4: Performance Issues
**Problem**: Services running slower than before.
**Solution**:
```bash
# Increase resource limits in docker-compose.yml
deploy:
  resources:
    limits:
      memory: 2G
      cpus: '2.0'

# Enable caching
ENABLE_CACHE=true
CACHE_TTL=3600
```

## Post-Migration Checklist

- [ ] All services are healthy
- [ ] Database contains all data
- [ ] Files are accessible
- [ ] Authentication works
- [ ] API endpoints respond correctly
- [ ] MCP Dashboard shows all services
- [ ] WebSocket connections work
- [ ] Monitoring is active
- [ ] Backups are configured
- [ ] Documentation is updated

## Performance Tuning

After migration, optimize performance:

1. **Database**:
```sql
-- Update statistics
ANALYZE;

-- Check slow queries
SELECT query, calls, mean_time 
FROM pg_stat_statements 
ORDER BY mean_time DESC 
LIMIT 10;
```

2. **Docker**:
```bash
# Prune unused resources
docker system prune -a

# Optimize compose file
# Use .env for variables
# Set appropriate restart policies
# Configure health checks
```

3. **MCP**:
```yaml
# Tune MCP proxy settings
MAX_CONCURRENT_REQUESTS: 200
CIRCUIT_BREAKER_THRESHOLD: 10
REQUEST_TIMEOUT_MS: 60000
```

## Next Steps

1. **Set up monitoring**:
   - Configure Prometheus/Grafana
   - Set up log aggregation
   - Configure alerts

2. **Enable additional features**:
   - Service mesh capabilities
   - A/B testing
   - Canary deployments

3. **Scale services**:
   ```bash
   docker-compose up -d --scale cognitive-assistant-mcp=3
   ```

4. **Implement CI/CD**:
   - Automated testing
   - Container scanning
   - Automated deployments

## Support

If you encounter issues:

1. Check the logs: `docker-compose logs [service-name]`
2. Review the documentation in `docker/mcp/README.md`
3. Run diagnostics: `./docker/mcp/test-mcp-docker.sh`
4. Contact support with:
   - Error messages
   - Service logs
   - Environment details

---

*Migration guide for TracSeq 2.0 MCP Docker infrastructure* 