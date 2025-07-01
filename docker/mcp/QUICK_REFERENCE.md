# MCP Docker Quick Reference

## 🚀 Quick Start

```bash
# Clone and setup
git clone https://github.com/your-org/tracseq2.0
cd tracseq2.0

# Configure environment
cd docker/mcp && cp env.example .env
# Edit .env with your values

# Build and start
./build-mcp.sh && ./start-mcp.sh

# Test everything
./test-mcp-docker.sh
```

## 🌐 Service URLs

| Service | URL | Credentials |
|---------|-----|-------------|
| MCP Dashboard | http://localhost:7890 | No auth (dev) |
| MCP WebSocket | ws://localhost:9500 | - |
| Consul UI | http://localhost:8500 | - |
| Metrics | http://localhost:9590/metrics | - |

## 🛠️ Common Commands

### Service Management
```bash
# Start all services
docker-compose -f docker-compose.with-mcp.yml up -d

# Stop all services
docker-compose -f docker-compose.with-mcp.yml down

# Restart specific service
docker-compose -f docker-compose.with-mcp.yml restart mcp-proxy

# Scale a service
docker-compose -f docker-compose.with-mcp.yml up -d --scale rag-service-mcp=3

# View service status
docker ps --format "table {{.Names}}\t{{.Status}}"
```

### Logs and Debugging
```bash
# View all logs
docker-compose -f docker-compose.with-mcp.yml logs -f

# View specific service logs
docker logs -f mcp-proxy

# View last 100 lines
docker logs --tail 100 cognitive-assistant-mcp

# Search logs for errors
docker-compose logs | grep -i error

# Enter container shell
docker exec -it mcp-proxy /bin/bash
```

### Health Checks
```bash
# Check all health statuses
for service in mcp-proxy mcp-dashboard cognitive-assistant-mcp; do
  echo -n "$service: "
  docker inspect $service | jq -r '.[0].State.Health.Status'
done

# Test specific endpoint
curl -f http://localhost:7890/health
curl -f http://localhost:9590/metrics | grep up
```

### Resource Management
```bash
# View resource usage
docker stats

# Clean up unused resources
docker system prune -a

# View disk usage
docker system df

# Remove specific volumes
docker volume rm tracseq_cognitive-logs
```

## 🔧 Configuration

### Key Environment Variables
```bash
# General
LOG_LEVEL=info|debug|warn|error
NODE_ENV=development|production

# MCP
MCP_PROXY_PORT=9500
MCP_ENABLED=true|false

# Database
DATABASE_URL=postgresql://user:pass@host:5432/db

# AI Services
OLLAMA_API_URL=http://ollama:11434
OLLAMA_MODEL=llama3.2:3b

# Security
JWT_SECRET=your-secret-key
API_KEY=your-api-key
```

### Service Ports
| Service | Internal | External |
|---------|----------|----------|
| MCP Proxy | 9500 | 9500 |
| MCP Dashboard | 7890 | 7890 |
| Cognitive Assistant | 9501 | - |
| RAG Service (HTTP) | 8100 | 8100 |
| RAG Service (MCP) | 9502 | - |
| Consul | 8500 | 8500 |

## 🐛 Troubleshooting

### Service Won't Start
```bash
# Check logs
docker logs mcp-proxy

# Check port availability
lsof -i :9500

# Recreate service
docker-compose -f docker-compose.with-mcp.yml up -d --force-recreate mcp-proxy
```

### Connection Issues
```bash
# Test network
docker exec mcp-proxy ping consul

# Check DNS
docker exec mcp-proxy nslookup cognitive-assistant-mcp

# Inspect network
docker network inspect tracseq-network
```

### Performance Issues
```bash
# Check resource limits
docker inspect mcp-proxy | jq '.[0].HostConfig.Resources'

# Monitor in real-time
docker stats --no-stream

# Check disk I/O
iostat -x 1
```

## 📊 Monitoring

### Prometheus Metrics
```bash
# Key metrics to monitor
curl -s http://localhost:9590/metrics | grep -E "mcp_requests_total|mcp_request_duration|mcp_active_connections"
```

### Consul Services
```bash
# List all services
curl http://localhost:8500/v1/catalog/services | jq

# Check service health
curl http://localhost:8500/v1/health/service/cognitive-assistant-mcp | jq
```

### Dashboard API
```bash
# Get service status
curl http://localhost:7890/api/services | jq

# Test MCP connection
curl -X POST http://localhost:7890/test-service \
  -H "Content-Type: application/json" \
  -d '{"service": "cognitive_assistant", "method": "ping"}'
```

## 🚨 Emergency Procedures

### Service Recovery
```bash
# Full restart
docker-compose -f docker-compose.with-mcp.yml down
docker-compose -f docker-compose.with-mcp.yml up -d

# Reset specific service
docker-compose -f docker-compose.with-mcp.yml rm -f -s -v mcp-proxy
docker-compose -f docker-compose.with-mcp.yml up -d mcp-proxy
```

### Data Backup
```bash
# Backup Consul data
docker exec consul consul snapshot save /consul/data/backup.snap
docker cp consul:/consul/data/backup.snap ./consul-backup-$(date +%Y%m%d).snap

# Backup volumes
docker run --rm -v cognitive-logs:/data -v $(pwd):/backup alpine tar czf /backup/cognitive-logs-$(date +%Y%m%d).tar.gz -C /data .
```

### Rollback
```bash
# Tag current images
docker tag tracseq/mcp-proxy:latest tracseq/mcp-proxy:rollback

# Restore from backup tag
docker-compose -f docker-compose.with-mcp.yml down
docker tag tracseq/mcp-proxy:rollback tracseq/mcp-proxy:latest
docker-compose -f docker-compose.with-mcp.yml up -d
```

## 🚀 Production Deployment

```bash
# Use production compose file
cd docker/mcp
docker-compose -f docker-compose.mcp.prod.yml up -d

# Enable TLS
export MCP_TLS_ENABLED=true
# Place certs in docker/mcp/certs/

# Set production environment
export NODE_ENV=production
export LOG_LEVEL=warn
```

## 📋 Checklists

### Pre-deployment
- [ ] Environment variables configured
- [ ] Database migrations run
- [ ] SSL certificates in place (prod)
- [ ] Resource limits set
- [ ] Backup procedures tested

### Post-deployment
- [ ] All services healthy
- [ ] Endpoints responding
- [ ] Logs clean of errors
- [ ] Monitoring active
- [ ] Performance acceptable

---

**Help**: `./docker/mcp/README.md` | **Issues**: Check logs first!

*Context improved by Giga AI* 