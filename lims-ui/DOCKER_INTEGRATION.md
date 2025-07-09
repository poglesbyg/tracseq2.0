# TracSeq 2.0 Frontend - Docker Integration Guide

This guide explains how the frontend Docker development environment integrates with the existing TracSeq 2.0 LIMS microservices ecosystem.

## 🏗️ Architecture Integration

### Current LIMS Ecosystem

The TracSeq 2.0 system consists of multiple microservices running in Docker containers:

```
┌─────────────────────────────────────────────────────────────────────────────────┐
│                           TracSeq 2.0 LIMS Ecosystem                            │
├─────────────────────────────────────────────────────────────────────────────────┤
│                                                                                 │
│  ┌─────────────────┐    ┌─────────────────┐    ┌─────────────────┐           │
│  │   Frontend Dev  │    │   API Gateway   │    │   Auth Service  │           │
│  │   Port: 5173    │◄──►│   Port: 8089    │◄──►│   Port: 8011    │           │
│  │                 │    │   (lims-gateway)│    │   (lims-auth)   │           │
│  └─────────────────┘    └─────────────────┘    └─────────────────┘           │
│                                   │                                            │
│  ┌─────────────────┐    ┌─────────▼─────────┐    ┌─────────────────┐         │
│  │ Sample Service  │    │   PostgreSQL      │    │ Storage Service │         │
│  │   Port: 8012    │◄──►│   Port: 5433      │◄──►│   Port: 8013    │         │
│  │  (lims-samples) │    │  (lims-postgres)  │    │ (lims-storage)  │         │
│  └─────────────────┘    └───────────────────┘    └─────────────────┘         │
│                                   │                                            │
│  ┌─────────────────┐    ┌─────────▼─────────┐    ┌─────────────────┐         │
│  │   RAG Service   │    │      Redis        │    │ Reports Service │         │
│  │   Port: 8100    │    │   Port: 6380      │    │   Port: 8014    │         │
│  │  (tracseq-rag)  │    │   (lims-redis)    │    │ (lims-reports)  │         │
│  └─────────────────┘    └───────────────────┘    └─────────────────┘         │
│                                                                                 │
│  ┌─────────────────┐    ┌─────────────────┐    ┌─────────────────┐           │
│  │ Ollama AI       │    │ Notification    │    │ Event Service   │           │
│  │   Port: 11434   │    │   Port: 8015    │    │   Port: 8016    │           │
│  │ (tracseq-ollama)│    │(tracseq-notify) │    │(tracseq-events) │           │
│  └─────────────────┘    └─────────────────┘    └─────────────────┘           │
│                                                                                 │
│                        Network: docker_lims-network                            │
└─────────────────────────────────────────────────────────────────────────────────┘
```

### Integration Points

| Service | Container Name | Internal Port | External Port | Status |
|---------|----------------|---------------|---------------|--------|
| **API Gateway** | `lims-gateway` | 8000 | 8089 | ✅ Healthy |
| **Frontend Dev** | `tracseq-frontend-dev` | 5173 | 5173 | ✅ Healthy |
| **Auth Service** | `lims-auth` | 8000 | 8011 | ✅ Healthy |
| **Sample Service** | `lims-samples` | 8000 | 8012 | ✅ Healthy |
| **Storage Service** | `lims-storage` | 8080 | 8013 | ✅ Healthy |
| **Reports Service** | `lims-reports` | 8000 | 8014 | ✅ Healthy |
| **PostgreSQL** | `lims-postgres` | 5432 | 5433 | ✅ Healthy |
| **Redis** | `lims-redis` | 6379 | 6380 | ✅ Healthy |
| **RAG Service** | `tracseq-rag` | 8000 | 8100 | ✅ Healthy |
| **Ollama AI** | `tracseq-ollama` | 11434 | 11434 | ✅ Healthy |

## 🔧 Configuration Details

### Network Configuration

The frontend development container connects to the existing `docker_lims-network`:

```yaml
networks:
  docker_lims-network:
    external: true  # Uses existing network
```

### Environment Variables

```bash
# Development Environment
NODE_ENV=development
DOCKER_ENV=true

# API Configuration (internal network)
VITE_API_URL=                              # Empty for proxy
VITE_API_BASE_URL=                         # Empty for proxy  
VITE_API_GATEWAY_URL=http://lims-gateway:8000  # Internal service name
VITE_WS_URL=ws://lims-gateway:8000/ws      # WebSocket connection

# Development Features
VITE_DEV_MODE=true
VITE_DEBUG_MODE=true
```

### Vite Proxy Configuration

```typescript
// vite.config.ts
const isDocker = process.env.DOCKER_ENV === 'true'
const apiTarget = isDocker 
  ? 'http://lims-gateway:8000'      // Internal Docker network
  : 'http://localhost:8089'         // External host access

export default defineConfig({
  server: {
    host: '0.0.0.0',
    proxy: {
      '/api': {
        target: apiTarget,
        changeOrigin: true,
        secure: false,
      },
      '/ws': {
        target: apiTarget.replace('http:', 'ws:'),
        ws: true,
        changeOrigin: true,
      },
    },
  },
})
```

## 🚀 Development Workflow

### 1. Starting the Development Environment

```bash
# Option 1: Use the development script (recommended)
./docker-dev.sh start

# Option 2: Use Docker Compose directly
docker-compose -f docker-compose.dev.yml up -d frontend-dev

# Option 3: Include API gateway if not running
./docker-dev.sh full
```

### 2. Accessing Services

| Service | URL | Description |
|---------|-----|-------------|
| **Frontend** | http://localhost:5173 | Development frontend with hot reload |
| **API Health** | http://localhost:5173/api/health | Proxied health check |
| **Login** | http://localhost:5173/api/auth/login | Authentication endpoint |
| **Direct API** | http://localhost:8089/api/health | Direct API gateway access |
| **Database** | localhost:5433 | PostgreSQL database |
| **Cache** | localhost:6380 | Redis cache |

### 3. Testing Integration

```bash
# Test API connectivity
curl http://localhost:5173/api/health

# Test authentication
curl -X POST http://localhost:5173/api/auth/login \
  -H "Content-Type: application/json" \
  -d '{"email":"admin@tracseq.com","password":"admin123"}'

# Test direct API gateway
curl http://localhost:8089/api/health

# Check container status
docker ps | grep -E "(tracseq|lims)"
```

## 🔍 Monitoring and Debugging

### Container Status

```bash
# Check all LIMS containers
docker ps --format "table {{.Names}}\t{{.Image}}\t{{.Status}}\t{{.Ports}}"

# Check specific frontend container
docker ps | grep tracseq-frontend-dev

# View frontend logs
docker logs tracseq-frontend-dev -f

# Check network connectivity
docker exec tracseq-frontend-dev ping lims-gateway
```

### Health Checks

```bash
# Frontend health (through proxy)
curl http://localhost:5173/api/health

# API Gateway health (direct)
curl http://localhost:8089/api/health

# Database connectivity
docker exec lims-postgres pg_isready -U postgres

# Redis connectivity  
docker exec lims-redis redis-cli ping
```

### Development Logs

```bash
# View all service logs
docker-compose logs -f

# View specific service logs
docker logs lims-gateway -f
docker logs tracseq-frontend-dev -f
docker logs lims-auth -f

# View proxy logs
docker logs tracseq-frontend-dev | grep "Proxying"
```

## 🛠️ Troubleshooting

### Common Issues

#### 1. Frontend Container Unhealthy

```bash
# Check container logs
docker logs tracseq-frontend-dev

# Restart container
docker-compose -f docker-compose.dev.yml restart frontend-dev

# Rebuild if needed
docker-compose -f docker-compose.dev.yml up -d --build frontend-dev
```

#### 2. API Proxy Not Working

```bash
# Verify API gateway is running
docker ps | grep lims-gateway

# Test direct API gateway connection
curl http://localhost:8089/api/health

# Check network connectivity
docker exec tracseq-frontend-dev ping lims-gateway

# Verify environment variables
docker exec tracseq-frontend-dev env | grep DOCKER_ENV
```

#### 3. Network Connection Issues

```bash
# Check if container is on correct network
docker inspect tracseq-frontend-dev | grep -A 5 "Networks"

# Verify network exists
docker network ls | grep docker_lims-network

# Check other containers on network
docker network inspect docker_lims-network
```

#### 4. Port Conflicts

```bash
# Check what's using port 5173
lsof -i :5173

# Stop conflicting services
docker stop tracseq-frontend  # Old frontend if running

# Use different port if needed
docker-compose -f docker-compose.dev.yml up -d \
  -p "5174:5173" frontend-dev
```

### Debug Mode

```bash
# Enable debug logging
docker-compose -f docker-compose.dev.yml down
VITE_DEBUG_MODE=true docker-compose -f docker-compose.dev.yml up -d frontend-dev

# View debug logs
docker logs tracseq-frontend-dev | grep -E "(debug|error|proxy)"
```

## 🔄 Integration with Existing Services

### Authentication Flow

1. **Frontend** makes login request to `/api/auth/login`
2. **Vite Proxy** forwards to `lims-gateway:8000/api/auth/login`
3. **API Gateway** routes to `lims-auth:8000/auth/login`
4. **Auth Service** validates credentials against PostgreSQL
5. **Response** flows back through the chain with JWT token

### Data Flow

1. **Frontend** requests data via `/api/samples`
2. **API Gateway** authenticates request
3. **Gateway** routes to appropriate service (e.g., `lims-samples:8000`)
4. **Service** queries PostgreSQL database
5. **Response** returns through proxy to frontend

### Real-time Features

- **WebSocket** connections for chat/notifications
- **Event streaming** through the event service
- **Live updates** for sample status changes
- **Real-time monitoring** of storage conditions

## 📊 Performance Considerations

### Resource Usage

```bash
# Monitor container resources
docker stats tracseq-frontend-dev

# Check memory usage
docker exec tracseq-frontend-dev free -h

# Monitor network traffic
docker exec tracseq-frontend-dev netstat -i
```

### Optimization Tips

1. **Use volume mounts** for instant file sync
2. **Enable hot reloading** for faster development
3. **Proxy API calls** to avoid CORS issues
4. **Share Docker network** for direct service communication
5. **Cache node_modules** to speed up rebuilds

## 🔒 Security Considerations

### Network Security

- Frontend container isolated on internal Docker network
- No direct external access to internal services
- All API calls go through authenticated gateway
- Database and Redis not exposed externally

### Development Security

```yaml
# Security configuration
security_opt:
  - no-new-privileges:true
user: "1000:1000"  # Non-root user
read_only: false   # Allow writes for development
```

## 🚀 Production Deployment

For production deployment, the configuration changes:

```bash
# Production environment variables
NODE_ENV=production
DOCKER_ENV=false
VITE_API_URL=https://api.tracseq.com
VITE_API_GATEWAY_URL=https://gateway.tracseq.com

# Production build
docker build -f Dockerfile -t tracseq-frontend:latest .

# Production deployment
docker run -p 80:80 tracseq-frontend:latest
```

## 📝 Quick Reference

### Essential Commands

```bash
# Start development
./docker-dev.sh start

# View logs
./docker-dev.sh logs

# Stop containers
./docker-dev.sh stop

# Restart with rebuild
./docker-dev.sh restart

# Open container shell
./docker-dev.sh shell

# Clean up resources
./docker-dev.sh clean
```

### Service URLs

```bash
# Development
Frontend:     http://localhost:5173
API Gateway:  http://localhost:8089
Database:     localhost:5433
Redis:        localhost:6380

# Internal (Docker network)
API Gateway:  http://lims-gateway:8000
Auth:         http://lims-auth:8000
Samples:      http://lims-samples:8000
Storage:      http://lims-storage:8080
```

### Environment Check

```bash
# Verify all services are running
docker ps | grep -E "(lims|tracseq)" | wc -l  # Should show ~15+ containers

# Check frontend integration
curl -s http://localhost:5173/api/health | jq .status  # Should return "healthy"

# Verify authentication
curl -s -X POST http://localhost:5173/api/auth/login \
  -H "Content-Type: application/json" \
  -d '{"email":"admin@tracseq.com","password":"admin123"}' | jq .data.token
```

---

*The frontend development environment is now fully integrated with the TracSeq 2.0 LIMS ecosystem! 🎉* 