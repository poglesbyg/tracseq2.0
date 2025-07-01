# MCP Docker Infrastructure

## Overview

This directory contains the Docker configuration for TracSeq 2.0's Model Context Protocol (MCP) infrastructure. MCP provides a standardized communication protocol for AI model interactions, enabling better coordination, monitoring, and scaling of AI services.

## Architecture

```
┌─────────────────────────────────────────────────────────────┐
│                       MCP Dashboard                           │
│                    (localhost:7890)                           │
└───────────────────────────┬──────────────────────────────────┘
                            │
┌───────────────────────────▼──────────────────────────────────┐
│                      MCP Proxy Server                         │
│                    (localhost:9500)                           │
│              • Service Discovery (Consul)                     │
│              • Load Balancing                                 │
│              • Circuit Breaking                               │
│              • Monitoring & Metrics                           │
└──────┬────────────────────┬────────────────────┬─────────────┘
       │                    │                    │
┌──────▼──────┐    ┌───────▼──────┐    ┌───────▼──────┐
│  Cognitive  │    │     RAG      │    │ Notification │
│  Assistant  │    │   Service    │    │   Service    │
│    MCP      │    │     MCP      │    │     MCP      │
│   (:9501)   │    │   (:9502)    │    │   (:9503)    │
└─────────────┘    └──────────────┘    └──────────────┘
```

## Components

### 1. MCP Proxy Server
- **Port**: 9500 (WebSocket), 9590 (Metrics)
- **Purpose**: Central coordination point for all MCP services
- **Features**:
  - Service discovery and registration
  - Request routing and load balancing
  - Circuit breaking and fault tolerance
  - Metrics collection and monitoring

### 2. MCP Dashboard
- **Port**: 7890 (HTTP)
- **Purpose**: Real-time monitoring and management UI
- **Features**:
  - Service health visualization
  - Request/response monitoring
  - Performance metrics
  - Service topology view

### 3. Cognitive Assistant MCP
- **Port**: 9501 (WebSocket)
- **Purpose**: AI-powered laboratory assistant
- **Features**:
  - Natural language processing
  - Laboratory workflow assistance
  - Integration with Ollama LLM
  - Context-aware responses

### 4. Consul
- **Port**: 8500 (HTTP), 8600 (DNS)
- **Purpose**: Service discovery and configuration
- **Features**:
  - Dynamic service registration
  - Health checking
  - Key/value storage
  - Multi-datacenter support

## Quick Start

### Build and Start All Services

```bash
# Build MCP images (first time or after changes)
./docker/mcp/build-mcp.sh

# Start all services with MCP
./docker/mcp/start-mcp.sh --build

# Or start without rebuilding
./docker/mcp/start-mcp.sh
```

### Individual Commands

```bash
# Start only MCP infrastructure
cd docker
docker-compose -f mcp/docker-compose.mcp.yml up -d

# Start TracSeq with MCP integration
docker-compose -f docker-compose.with-mcp.yml up -d

# View logs
docker-compose -f docker-compose.with-mcp.yml logs -f mcp-proxy

# Stop all services
docker-compose -f docker-compose.with-mcp.yml down
```

## Service URLs

| Service | URL | Purpose |
|---------|-----|---------|
| MCP Dashboard | http://localhost:7890 | Monitoring UI |
| MCP Proxy API | ws://localhost:9500 | WebSocket API |
| MCP Metrics | http://localhost:9590/metrics | Prometheus metrics |
| Consul UI | http://localhost:8500 | Service discovery UI |
| Cognitive Assistant | ws://localhost:9501 | AI assistant API |

## Configuration

### Environment Variables

Each MCP service can be configured through environment variables:

```yaml
# MCP Proxy
MCP_PROXY_PORT: 9500
LOG_LEVEL: info
METRICS_ENABLED: true
SERVICE_DISCOVERY_ENABLED: true
CONSUL_HOST: consul
CONSUL_PORT: 8500

# MCP Dashboard
MCP_PROXY_URL: ws://mcp-proxy:9500
FLASK_ENV: production

# Cognitive Assistant MCP
MCP_SERVICE_NAME: cognitive_assistant
MCP_SERVICE_PORT: 9501
OLLAMA_API_URL: http://ollama:11434
DATABASE_URL: postgresql://...
```

### Service Registration

Services automatically register with Consul on startup:

```json
{
  "ID": "cognitive-assistant-mcp-1",
  "Name": "cognitive-assistant-mcp",
  "Tags": ["mcp", "ai", "assistant"],
  "Address": "cognitive-assistant-mcp",
  "Port": 9501,
  "Check": {
    "Type": "http",
    "URL": "http://cognitive-assistant-mcp:9501/health",
    "Interval": "10s"
  }
}
```

## Development

### Adding a New MCP Service

1. Create service implementation:
```python
# my_service_mcp.py
from mcp import MCPServer, method

class MyServiceMCP(MCPServer):
    @method("my_service.process")
    async def process(self, data):
        # Implementation
        return {"result": "processed"}
```

2. Create Dockerfile:
```dockerfile
FROM python:3.11-slim
WORKDIR /app
COPY requirements.txt .
RUN pip install -r requirements.txt
COPY . .
CMD ["python", "my_service_mcp.py"]
```

3. Add to docker-compose:
```yaml
my-service-mcp:
  build: ./my-service
  ports:
    - "9504:9504"
  environment:
    - MCP_SERVICE_NAME=my_service
    - MCP_PROXY_URL=ws://mcp-proxy:9500
  depends_on:
    - mcp-proxy
```

### Testing MCP Services

```bash
# Test MCP proxy health
curl http://localhost:9590/metrics

# Test service registration
curl http://localhost:8500/v1/catalog/services

# Connect to MCP service via WebSocket
wscat -c ws://localhost:9500
> {"jsonrpc": "2.0", "method": "cognitive_assistant.query", "params": {"query": "test"}, "id": 1}
```

## Monitoring

### Prometheus Metrics

MCP services expose metrics at `/metrics`:

- `mcp_requests_total` - Total requests processed
- `mcp_request_duration_seconds` - Request latency
- `mcp_active_connections` - Current WebSocket connections
- `mcp_service_health` - Service health status

### Health Checks

All services implement health endpoints:

```bash
# Check individual service health
curl http://localhost:7890/health
curl http://localhost:8500/v1/health/service/mcp-proxy

# Docker health status
docker ps --format "table {{.Names}}\t{{.Status}}"
```

## Troubleshooting

### Common Issues

1. **Service not registering with Consul**
   - Check Consul is running: `docker ps | grep consul`
   - Verify network connectivity: `docker exec mcp-proxy ping consul`
   - Check service logs: `docker logs cognitive-assistant-mcp`

2. **WebSocket connection failures**
   - Verify proxy is running: `curl http://localhost:9590/metrics`
   - Check firewall/port availability
   - Review proxy logs: `docker logs mcp-proxy`

3. **Dashboard not loading**
   - Check dashboard health: `curl http://localhost:7890/health`
   - Verify proxy connection in logs
   - Clear browser cache

### Debug Commands

```bash
# View all MCP logs
docker-compose -f docker-compose.with-mcp.yml logs -f | grep mcp

# Inspect service configuration
docker inspect mcp-proxy | jq '.[0].Config.Env'

# Test WebSocket connection
pip install websocket-client
python -c "import websocket; ws = websocket.create_connection('ws://localhost:9500'); print('Connected')"

# Check Consul service catalog
curl http://localhost:8500/v1/catalog/services | jq
```

## Performance Tuning

### Resource Limits

Configure resource limits in docker-compose:

```yaml
mcp-proxy:
  deploy:
    resources:
      limits:
        cpus: '2.0'
        memory: 2G
      reservations:
        cpus: '1.0'
        memory: 1G
```

### Scaling

Scale MCP services horizontally:

```bash
# Scale cognitive assistant instances
docker-compose -f docker-compose.with-mcp.yml up -d --scale cognitive-assistant-mcp=3
```

## Security

### Network Isolation

MCP services run on an isolated Docker network:

```yaml
networks:
  tracseq-network:
    driver: bridge
    ipam:
      config:
        - subnet: 172.20.0.0/16
```

### TLS Configuration (Production)

For production, enable TLS:

```yaml
mcp-proxy:
  volumes:
    - ./certs:/certs
  environment:
    - MCP_TLS_ENABLED=true
    - MCP_TLS_CERT=/certs/server.crt
    - MCP_TLS_KEY=/certs/server.key
```

## Backup and Recovery

### Consul Data

Backup Consul's key/value store:

```bash
# Backup
docker exec consul consul snapshot save /consul/data/backup.snap
docker cp consul:/consul/data/backup.snap ./consul-backup.snap

# Restore
docker cp ./consul-backup.snap consul:/consul/data/backup.snap
docker exec consul consul snapshot restore /consul/data/backup.snap
```

### Service Configurations

All service configurations are stored in:
- Environment variables (docker-compose files)
- Consul key/value store
- Docker volumes

Regular backups should include these components.

*Context improved by Giga AI* 