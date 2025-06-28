# TracSeq 2.0 - Complete Docker Build System

## 🎯 Overview

This build system enables you to build each microservice individually **and** deploy them all together seamlessly. You now have complete control over your microservices architecture with robust build, test, and deployment capabilities.

## 📁 Build System Components

### 🔧 **Individual Service Builders**
- `scripts/build-individual-service.sh` - Linux/macOS script for building single services
- `scripts/build-individual-service.ps1` - PowerShell script for Windows users
- `config-service/Dockerfile` - Added missing Dockerfile for config service

### 🚀 **Collective Deployment**
- `scripts/build-and-deploy-all.sh` - Complete orchestration script
- `docker-compose-build-all.yml` - Unified compose file for all services
- `scripts/init-databases.sql` - PostgreSQL initialization script

### 📋 **Service Coverage**
✅ **All 16 microservices are supported:**

**Rust Services:**
- `config-service` *(newly added Dockerfile)*
- `auth_service`
- `sample_service`
- `sequencing_service`
- `notification_service`
- `enhanced_storage_service`
- `template_service`
- `transaction_service`
- `event_service`
- `lab_manager`
- `library_details_service`
- `qaqc_service`
- `spreadsheet_versioning_service`

**Python Services:**
- `enhanced_rag_service`
- `api_gateway`
- `lab_submission_rag`

---

## 🚀 Quick Start

### **Option 1: Build Everything at Once**
```bash
# Linux/macOS
./scripts/build-and-deploy-all.sh

# Windows (PowerShell)
# Use individual scripts or existing PowerShell deployment scripts
```

### **Option 2: Build Individual Services**
```bash
# Linux/macOS - Build a single service
./scripts/build-individual-service.sh auth_service
./scripts/build-individual-service.sh enhanced_storage_service --no-cache

# Windows - Build a single service
.\scripts\build-individual-service.ps1 -ServiceName auth_service
.\scripts\build-individual-service.ps1 -ServiceName enhanced_storage_service -NoCache
```

### **Option 3: Use Existing Docker Compose**
```bash
# Build all services using docker-compose
docker compose -f docker-compose-build-all.yml build

# Deploy all services
docker compose -f docker-compose-build-all.yml up -d
```

---

## 📖 Detailed Usage Guide

### 🔨 **Building Individual Services**

#### **Linux/macOS (Bash)**
```bash
# Basic build
./scripts/build-individual-service.sh <service-name>

# Build with options
./scripts/build-individual-service.sh <service-name> [options]

# Examples:
./scripts/build-individual-service.sh auth_service
./scripts/build-individual-service.sh auth_service --tag v1.0.0
./scripts/build-individual-service.sh enhanced_storage_service --no-cache
./scripts/build-individual-service.sh api_gateway --build-arg ENV=production
./scripts/build-individual-service.sh lab_manager --push --verbose
```

**Available Options:**
- `-t, --tag TAG` - Image tag (default: latest)
- `-f, --file FILE` - Dockerfile name (default: Dockerfile) 
- `--no-cache` - Build without cache
- `--push` - Push image to registry after build
- `--build-arg ARG` - Pass build argument to Docker
- `-v, --verbose` - Verbose output
- `-h, --help` - Show help message

#### **Windows (PowerShell)**
```powershell
# Basic build
.\scripts\build-individual-service.ps1 -ServiceName <service-name>

# Build with options
.\scripts\build-individual-service.ps1 -ServiceName <service-name> [options]

# Examples:
.\scripts\build-individual-service.ps1 -ServiceName auth_service
.\scripts\build-individual-service.ps1 -ServiceName auth_service -ImageTag v1.0.0
.\scripts\build-individual-service.ps1 -ServiceName enhanced_storage_service -NoCache
.\scripts\build-individual-service.ps1 -ServiceName api_gateway -BuildArgs @('ENV=production')
```

**Available Parameters:**
- `-ServiceName` - Name of the service to build (required)
- `-ImageTag` - Image tag (default: latest)
- `-Dockerfile` - Dockerfile name (default: Dockerfile)
- `-NoCache` - Build without cache
- `-Push` - Push image to registry after build
- `-BuildArgs` - Array of build arguments
- `-Verbose` - Verbose output
- `-Help` - Show help message

### 🏗️ **Complete Build & Deploy System**

#### **Full Orchestration Script**
```bash
# Build everything and deploy
./scripts/build-and-deploy-all.sh

# Build with specific options
./scripts/build-and-deploy-all.sh --no-cache --parallel
./scripts/build-and-deploy-all.sh --skip-individual  # Only deploy
./scripts/build-and-deploy-all.sh --skip-collective  # Only build individually
```

**Available Options:**
- `--skip-individual` - Skip individual service builds
- `--skip-collective` - Skip collective deployment
- `--no-cache` - Build without Docker cache
- `--parallel` - Build services in parallel (experimental)
- `--skip-tests` - Skip running tests after build
- `--compose-file FILE` - Docker compose file to use
- `--timeout SECONDS` - Build timeout in seconds (default: 1800)
- `-h, --help` - Show help message

#### **What the Full Script Does:**
1. **Prerequisites Check** - Verifies Docker and Docker Compose
2. **Optional Cleanup** - Removes old containers/images
3. **Individual Builds** - Builds each service separately with error handling
4. **Testing** - Runs tests on built images (if test script exists)
5. **Tiered Deployment** - Deploys services in proper dependency order:
   - Infrastructure (PostgreSQL, Redis, Ollama)
   - Foundational Services (Config, Auth, Events)
   - Core Business Services (Sample, Storage, Templates, etc.)
   - Specialized Services (Library Details, QA/QC, etc.)
   - AI/ML Services (Enhanced RAG, Lab Submission RAG)
   - Gateway & Frontend (API Gateway, Lab Manager)
   - Monitoring (Prometheus, Grafana)
6. **Health Checks** - Verifies all services are running
7. **Summary** - Shows deployment status and service URLs

---

## 🐳 Docker Compose Usage

### **Using the Unified Compose File**
```bash
# Build all services
docker compose -f docker-compose-build-all.yml build

# Build specific services
docker compose -f docker-compose-build-all.yml build auth-service sample-service

# Build without cache
docker compose -f docker-compose-build-all.yml build --no-cache

# Deploy all services
docker compose -f docker-compose-build-all.yml up -d

# Deploy specific tier
docker compose -f docker-compose-build-all.yml up -d postgres redis auth-service

# View logs
docker compose -f docker-compose-build-all.yml logs -f

# Stop all services
docker compose -f docker-compose-build-all.yml down

# Stop and remove volumes
docker compose -f docker-compose-build-all.yml down --volumes
```

### **Service Dependency Order**
The compose file deploys services in tiers with proper dependencies:

1. **Infrastructure** → PostgreSQL, Redis, Ollama
2. **Foundation** → Config Service, Auth Service, Event Service  
3. **Core Business** → Sample, Storage, Template, Sequencing, Notification, Transaction
4. **Specialized** → Library Details, QA/QC, Spreadsheet Versioning
5. **AI/ML** → Enhanced RAG, Lab Submission RAG
6. **Gateway** → API Gateway, Lab Manager
7. **Monitoring** → Prometheus, Grafana

---

## 🔧 Development Workflows

### **Scenario 1: Working on a Single Service**
```bash
# Build and test just the service you're working on
./scripts/build-individual-service.sh auth_service --no-cache --verbose

# Test the service individually
docker run --rm -p 8080:8080 tracseq-auth-service:latest

# Deploy into existing stack
docker compose -f docker-compose-build-all.yml up -d auth-service
```

### **Scenario 2: Testing New Features**
```bash
# Build specific services with new tag
./scripts/build-individual-service.sh auth_service --tag feature-branch
./scripts/build-individual-service.sh sample_service --tag feature-branch

# Deploy with new images (modify compose file tags)
docker compose -f docker-compose-build-all.yml up -d
```

### **Scenario 3: Fresh Complete Deployment**
```bash
# Complete clean deployment
./scripts/build-and-deploy-all.sh --no-cache

# Or step by step
docker compose -f docker-compose-build-all.yml down --volumes
./scripts/build-and-deploy-all.sh
```

### **Scenario 4: Production Deployment**
```bash
# Build for production with optimizations
./scripts/build-individual-service.sh lab_manager --build-arg ENV=production --tag production
./scripts/build-individual-service.sh api_gateway --build-arg ENV=production --tag production

# Deploy production stack
docker compose -f docker-compose-build-all.yml up -d
```

---

## 🌐 Service URLs & Health Checks

After deployment, access services at:

| Service | URL | Description |
|---------|-----|-------------|
| Lab Manager | http://localhost:3000 | Main web interface |
| API Gateway | http://localhost:8089 | Unified API endpoint |
| Auth Service | http://localhost:8080 | Authentication |
| Sample Service | http://localhost:8081 | Sample management |
| Enhanced Storage | http://localhost:8082 | Storage with AI |
| Template Service | http://localhost:8083 | Template management |
| Sequencing Service | http://localhost:8084 | Sequencing workflows |
| Notification Service | http://localhost:8085 | Notifications |
| Enhanced RAG | http://localhost:8086 | AI document processing |
| Event Service | http://localhost:8087 | Event handling |
| Transaction Service | http://localhost:8088 | Workflow orchestration |
| Config Service | http://localhost:8091 | Configuration |
| Library Details | http://localhost:8092 | Library management |
| QA/QC Service | http://localhost:8093 | Quality control |
| Spreadsheet Versioning | http://localhost:8094 | Version control |
| Lab Submission RAG | http://localhost:8095 | RAG processing |
| Prometheus | http://localhost:9090 | Metrics |
| Grafana | http://localhost:3001 | Dashboards (admin/admin) |

### **Health Check Commands**
```bash
# Check individual service health
curl http://localhost:8080/health  # Auth Service
curl http://localhost:8081/health  # Sample Service
curl http://localhost:8089/health  # API Gateway

# Check all services at once
for port in 8080 8081 8082 8083 8084 8085 8086 8087 8088 8089 8091 8092 8093 8094 8095 3000; do
  echo -n "Port $port: "
  curl -s -o /dev/null -w "%{http_code}" http://localhost:$port/health || echo "FAIL"
  echo
done
```

---

## 🛠️ Troubleshooting

### **Common Issues & Solutions**

#### **Build Failures**
```bash
# Build with verbose output to see detailed errors
./scripts/build-individual-service.sh <service> --verbose

# Build without cache to avoid cache issues
./scripts/build-individual-service.sh <service> --no-cache

# Check Dockerfile exists
ls -la <service>/Dockerfile
```

#### **Service Won't Start**
```bash
# Check service logs
docker compose -f docker-compose-build-all.yml logs <service-name>

# Check if service is running
docker ps --filter "name=tracseq-*"

# Check health status
docker compose -f docker-compose-build-all.yml ps
```

#### **Database Issues**
```bash
# Reset database
docker compose -f docker-compose-build-all.yml down --volumes
docker compose -f docker-compose-build-all.yml up -d postgres

# Check database connection
docker exec -it tracseq-postgres psql -U postgres -d tracseq_db -c "SELECT 1;"
```

#### **Network Issues**
```bash
# Check network connectivity
docker network ls | grep tracseq
docker network inspect <network-name>

# Restart networking
docker compose -f docker-compose-build-all.yml down
docker compose -f docker-compose-build-all.yml up -d
```

#### **Resource Issues**
```bash
# Check system resources
docker stats
docker system df

# Clean up unused resources
docker system prune -a
docker volume prune
```

### **Debug Commands**
```bash
# Enter a running container
docker exec -it tracseq-<service-name> /bin/bash

# Check service environment
docker exec tracseq-<service-name> env

# Check service processes
docker exec tracseq-<service-name> ps aux

# View real-time logs
docker compose -f docker-compose-build-all.yml logs -f <service-name>
```

---

## 📊 Performance & Optimization

### **Build Performance Tips**
1. **Use Docker BuildKit**: `export DOCKER_BUILDKIT=1`
2. **Parallel Builds**: Use `--parallel` flag in build script
3. **Layer Caching**: Build dependencies separately from source code
4. **Multi-stage Builds**: All Dockerfiles use optimized multi-stage builds

### **Runtime Performance**
1. **Resource Limits**: Configure in docker-compose file
2. **Health Checks**: Monitor service health automatically
3. **Monitoring**: Use Prometheus + Grafana for observability

### **Scaling Options**
```bash
# Scale specific services
docker compose -f docker-compose-build-all.yml up -d --scale sample-service=3
docker compose -f docker-compose-build-all.yml up -d --scale api-gateway=2
```

---

## 🔄 CI/CD Integration

### **GitHub Actions Example**
```yaml
name: Build TracSeq Services
on: [push, pull_request]

jobs:
  build:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v3
      
      - name: Build Individual Services
        run: |
          for service in auth_service sample_service enhanced_storage_service; do
            ./scripts/build-individual-service.sh $service --no-cache
          done
      
      - name: Deploy Stack
        run: ./scripts/build-and-deploy-all.sh --skip-individual
      
      - name: Run Health Checks
        run: |
          sleep 60
          ./scripts/health-check.sh
```

### **Production Deployment**
```bash
# Tag for production
./scripts/build-individual-service.sh lab_manager --tag $(git rev-parse --short HEAD)

# Push to registry
./scripts/build-individual-service.sh lab_manager --tag production --push
```

---

## 📝 Maintenance

### **Regular Maintenance Tasks**
```bash
# Update base images
docker pull postgres:15-alpine
docker pull redis:7-alpine
docker pull rustlang/rust:nightly-slim

# Rebuild services with updated base images
./scripts/build-and-deploy-all.sh --no-cache

# Clean up old images
docker image prune -a
```

### **Backup & Recovery**
```bash
# Backup databases
docker exec tracseq-postgres pg_dump -U postgres tracseq_db > backup.sql

# Backup volumes
docker run --rm -v tracseq_postgres_data:/data -v $(pwd):/backup alpine tar czf /backup/postgres-backup.tar.gz /data
```

---

## 🎉 Summary

You now have a **complete, production-ready Docker build system** that supports:

✅ **Individual service builds** with comprehensive options  
✅ **Collective deployment** with proper service dependencies  
✅ **Cross-platform support** (Linux, macOS, Windows)  
✅ **Health monitoring** and validation  
✅ **Development workflows** for efficient iteration  
✅ **Production deployment** capabilities  
✅ **Troubleshooting tools** and comprehensive documentation  

**The system handles all 16 microservices** and provides flexible deployment strategies for any development or production scenario.

*Context added by Giga llm-integration* 
