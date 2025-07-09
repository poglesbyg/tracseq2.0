# TracSeq 2.0 Frontend - Docker Development Guide

This guide explains how to use Docker for frontend development with hot reloading, proper networking, and integration with the API gateway.

## 🚀 Quick Start

### Option 1: Using the Development Script (Recommended)

```bash
# Start frontend only (requires API gateway running separately)
./docker-dev.sh start

# Start frontend + API gateway together
./docker-dev.sh full

# View logs
./docker-dev.sh logs

# Open shell in container
./docker-dev.sh shell

# Stop containers
./docker-dev.sh stop
```

### Option 2: Using Docker Compose Directly

```bash
# Start frontend only
docker-compose -f docker-compose.dev.yml up -d frontend-dev

# Start frontend + API gateway
docker-compose -f docker-compose.dev.yml --profile api-gateway up -d

# View logs
docker-compose -f docker-compose.dev.yml logs -f

# Stop containers
docker-compose -f docker-compose.dev.yml down
```

## 📋 Prerequisites

1. **Docker Desktop** installed and running
2. **API Gateway** running on port 8000 (or use the included API gateway profile)
3. **Node.js 18+** (for local development comparison)

## 🏗️ Architecture

### Development Setup

```
┌─────────────────────┐    ┌─────────────────────┐
│   Frontend Docker   │    │   API Gateway       │
│   (Port 5173)       │◄──►│   (Port 8000)       │
│                     │    │                     │
│   - Hot Reloading   │    │   - Mock Users      │
│   - Volume Mounts   │    │   - Proxy Routes    │
│   - Vite Dev Server │    │   - WebSocket       │
└─────────────────────┘    └─────────────────────┘
```

### Docker Network

- **Network Name**: `tracseq-network`
- **Driver**: Bridge
- **Frontend Container**: `tracseq-frontend-dev`
- **API Gateway Container**: `tracseq-api-gateway-dev` (optional)

## 🔧 Configuration

### Environment Variables

The Docker setup uses these environment variables:

```bash
# Development mode
NODE_ENV=development
VITE_DEV_MODE=true
VITE_DEBUG_MODE=true

# API Configuration
VITE_API_URL=                              # Empty for proxy
VITE_API_BASE_URL=                         # Empty for proxy
VITE_API_GATEWAY_URL=http://localhost:8000 # Direct API gateway access
VITE_WS_URL=ws://localhost:8000/ws         # WebSocket URL
```

### Volume Mounts

The following directories are mounted for hot reloading:

```yaml
volumes:
  - ./src:/app/src                    # Source code
  - ./public:/app/public              # Public assets
  - ./index.html:/app/index.html      # HTML template
  - ./vite.config.ts:/app/vite.config.ts  # Vite config
  - ./tailwind.config.js:/app/tailwind.config.js  # Tailwind config
  - /app/node_modules                 # Exclude node_modules
```

## 🛠️ Development Workflow

### 1. Start Development Environment

```bash
# Option A: Start frontend only (API gateway running separately)
./docker-dev.sh start

# Option B: Start everything together
./docker-dev.sh full
```

### 2. Access Applications

- **Frontend**: http://localhost:5173
- **API Gateway**: http://localhost:8000 (if using full mode)
- **API Health**: http://localhost:5173/api/health (proxied)

### 3. Development Features

- ✅ **Hot Reloading**: File changes trigger automatic reloads
- ✅ **Source Maps**: Full debugging support
- ✅ **Proxy Configuration**: API requests proxied to gateway
- ✅ **WebSocket Support**: Real-time chat functionality
- ✅ **Volume Mounts**: Instant file sync between host and container

### 4. Testing Authentication

```bash
# Test login with mock users
curl -X POST http://localhost:5173/api/auth/login \
  -H "Content-Type: application/json" \
  -d '{"email":"admin@tracseq.com","password":"admin123"}'
```

### 5. Debugging

```bash
# View container logs
./docker-dev.sh logs

# Open shell in container
./docker-dev.sh shell

# Check container status
docker ps | grep tracseq
```

## 🔍 Troubleshooting

### Common Issues

#### 1. Port Already in Use

```bash
# Check what's using port 5173
lsof -i :5173

# Kill process if needed
kill -9 <PID>
```

#### 2. API Gateway Connection Issues

```bash
# Check if API gateway is running
curl http://localhost:8000/health

# Check Docker network
docker network ls | grep tracseq
```

#### 3. Hot Reloading Not Working

```bash
# Rebuild the container
./docker-dev.sh build

# Check volume mounts
docker inspect tracseq-frontend-dev | grep -A 10 "Mounts"
```

#### 4. Node Modules Issues

```bash
# Clean rebuild
./docker-dev.sh clean
./docker-dev.sh build
./docker-dev.sh start
```

### Debug Commands

```bash
# Check container logs
docker logs tracseq-frontend-dev

# Inspect container
docker inspect tracseq-frontend-dev

# Check network connectivity
docker exec tracseq-frontend-dev ping host.docker.internal

# Check environment variables
docker exec tracseq-frontend-dev env | grep VITE
```

## 📊 Performance Considerations

### Container Resource Limits

The development setup is optimized for:
- **Memory**: No specific limits (uses host memory)
- **CPU**: Uses all available cores
- **File Watching**: Polling enabled for Docker compatibility

### Build Optimization

- Uses `.dockerignore` to exclude unnecessary files
- Multi-stage builds for production
- Efficient layer caching

## 🔄 Switching Between Local and Docker

### From Local to Docker

```bash
# Stop local dev server
# Ctrl+C in terminal running npm run dev

# Start Docker development
./docker-dev.sh start
```

### From Docker to Local

```bash
# Stop Docker containers
./docker-dev.sh stop

# Start local development
npm run dev
```

## 📝 Available Scripts

| Script | Description |
|--------|-------------|
| `./docker-dev.sh start` | Start frontend container |
| `./docker-dev.sh full` | Start frontend + API gateway |
| `./docker-dev.sh stop` | Stop all containers |
| `./docker-dev.sh restart` | Restart containers |
| `./docker-dev.sh logs` | View container logs |
| `./docker-dev.sh shell` | Open container shell |
| `./docker-dev.sh build` | Build development image |
| `./docker-dev.sh clean` | Clean up resources |
| `./docker-dev.sh help` | Show help message |

## 🌟 Best Practices

1. **Use the development script** for easier management
2. **Monitor container logs** during development
3. **Use volume mounts** for instant file sync
4. **Keep containers running** for faster development cycles
5. **Clean up regularly** to avoid disk space issues

## 🔗 Integration with Existing Services

### API Gateway Integration

The Docker setup automatically connects to:
- **Local API Gateway**: `http://localhost:8000`
- **Docker API Gateway**: `http://host.docker.internal:8000`

### Authentication Flow

1. Frontend makes login request to `/api/auth/login`
2. Vite proxy forwards to API gateway
3. API gateway returns JWT token
4. Frontend stores token and uses for subsequent requests

### WebSocket Support

- Chat functionality works through WebSocket proxy
- Real-time features fully supported
- Automatic reconnection on connection loss

## 🚀 Production Deployment

For production deployment, use the standard `Dockerfile`:

```bash
# Build production image
docker build -t tracseq-frontend:latest .

# Run production container
docker run -p 3000:8000 tracseq-frontend:latest
```

## 📞 Support

If you encounter issues:

1. Check the troubleshooting section above
2. Review container logs: `./docker-dev.sh logs`
3. Verify network connectivity
4. Ensure API gateway is running
5. Try rebuilding: `./docker-dev.sh clean && ./docker-dev.sh build`

---

*Happy Docker Development! 🐳* 