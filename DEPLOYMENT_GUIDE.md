# TracSeq 2.0 Complete Deployment Guide

## 🚀 Quick Start (TL;DR)

**For Production-Ready Services (6/10 services)**:
```bash
# 1. Setup environment
./scripts/setup-environment.sh

# 2. Configure external services (edit .env.production)
nano deploy/.env.production

# 3. Deploy core services
./scripts/deploy-production.sh

# 4. Health check
./scripts/comprehensive-health-check.sh
```

**System URLs after deployment**:
- **API Gateway**: http://localhost:8089
- **Frontend**: http://localhost:3000 (Lab Manager)
- **Grafana**: http://localhost:3001 (admin/your_password)
- **Prometheus**: http://localhost:9090

---

## 📋 Table of Contents

1. [System Overview](#system-overview)
2. [Prerequisites](#prerequisites)
3. [Environment Setup](#environment-setup)
4. [Production Deployment](#production-deployment)
5. [Staging Deployment](#staging-deployment)
6. [Post-Deployment Verification](#post-deployment-verification)
7. [Service Configuration](#service-configuration)
8. [Monitoring Setup](#monitoring-setup)
9. [Troubleshooting](#troubleshooting)
10. [Scaling & Performance](#scaling--performance)

---

## 🎯 System Overview

TracSeq 2.0 is a comprehensive laboratory management system with the following architecture:

### **Production-Ready Services (Deploy First)**
✅ **Auth Service** (95/100) - JWT authentication, RBAC, multi-tenant  
✅ **Sample Service** (80/100) - Barcode generation, batch operations, QC  
✅ **Template Service** (100/100) - Spreadsheet processing, versioning  
✅ **Notification Service** (100/100) - Multi-channel notifications, templates  
✅ **Sequencing Service** (95/100) - Job management, platform integration  
✅ **Transaction Service** (95/100) - Saga pattern, distributed transactions  

### **Phase 2 Services (Minor Fixes Needed)**
⚠️ **API Gateway** (80/100) - Service routing, rate limiting  
⚠️ **Enhanced RAG Service** (70/100) - AI document processing  

### **Phase 3 Services (Significant Work Required)**
🔧 **Enhanced Storage Service** - IoT monitoring, blockchain audit  
🔧 **Event Service** - Redis streams, event replay  

### **Infrastructure Services**
- **PostgreSQL** - Primary database with performance tuning
- **Redis** - Caching, session storage, event streams
- **Prometheus** - Metrics collection and alerting
- **Grafana** - Dashboards and visualization
- **Jaeger** - Distributed tracing
- **Nginx** - Load balancing and SSL termination

---

## 🔧 Prerequisites

### **System Requirements**
- **OS**: Linux (Ubuntu 20.04+), macOS, or Windows with WSL2
- **Memory**: 16GB RAM minimum (32GB recommended)
- **Storage**: 100GB free space minimum
- **CPU**: 4 cores minimum (8 cores recommended)

### **Software Dependencies**
```bash
# Docker & Docker Compose
curl -fsSL https://get.docker.com -o get-docker.sh
sudo sh get-docker.sh
sudo curl -L "https://github.com/docker/compose/releases/latest/download/docker-compose-$(uname -s)-$(uname -m)" -o /usr/local/bin/docker-compose
sudo chmod +x /usr/local/bin/docker-compose

# Additional tools
sudo apt update
sudo apt install -y curl jq openssl git
```

### **Network Requirements**
- **Ports**: 3000, 5432, 6379, 8000-8090, 9090, 3001
- **Internet Access**: Required for Docker images and external APIs
- **Firewall**: Configure to allow internal container communication

---

## ⚙️ Environment Setup

### **1. Clone and Prepare Repository**
```bash
git clone <your-tracseq-repo>
cd tracseq2.0

# Check current status
git status
git branch
```

### **2. Generate Environment Configurations**
```bash
# Create production and staging environments
chmod +x scripts/*.sh
./scripts/setup-environment.sh
```

### **3. Configure External Services**

Edit `deploy/.env.production`:

**Email Configuration (Required)**:
```bash
# Gmail example
SMTP_HOST=smtp.gmail.com
SMTP_PORT=587
SMTP_USERNAME=your-email@gmail.com
SMTP_PASSWORD=your-app-specific-password
EMAIL_FROM=noreply@yourdomain.com
```

**AI Services (Optional but Recommended)**:
```bash
# OpenAI
OPENAI_API_KEY=sk-your-openai-api-key

# Anthropic Claude
ANTHROPIC_API_KEY=sk-ant-your-anthropic-key
```

**Notification Channels (Optional)**:
```bash
# Slack
SLACK_BOT_TOKEN=xoxb-your-slack-bot-token
SLACK_WEBHOOK_URL=https://hooks.slack.com/services/YOUR/WEBHOOK

# SMS (Twilio)
SMS_API_KEY=your-twilio-api-key
SMS_FROM=+1234567890
```

---

## 🚀 Production Deployment

### **Phase 1: Core Infrastructure & Services**

```bash
# Deploy production-ready services
./scripts/deploy-production.sh
```

**What this deploys**:
- PostgreSQL with optimized configuration
- Redis with persistence and caching
- Auth Service (JWT, RBAC, multi-tenant)
- Sample Service (barcode generation, QC)
- Template Service (spreadsheet processing)
- Notification Service (email, SMS, Slack)
- Sequencing Service (job management)
- Transaction Service (saga pattern)

**Expected startup time**: 3-5 minutes

### **Phase 2: API Gateway & Enhanced Services**

```bash
# Deploy API Gateway
docker-compose -f deploy/production/docker-compose.production.yml up -d api-gateway

# Deploy Enhanced RAG Service (if AI keys configured)
docker-compose -f deploy/production/docker-compose.production.yml up -d rag-service chroma
```

### **Phase 3: Monitoring Stack**

```bash
# Deploy monitoring services
docker-compose -f deploy/production/docker-compose.production.yml up -d \
  prometheus grafana jaeger loki
```

### **Phase 4: Frontend Application**

```bash
# Deploy Lab Manager frontend
cd lab_manager
docker-compose up -d frontend
```

---

## 🧪 Staging Deployment

### **Quick Staging Setup**
```bash
# Deploy full staging environment
docker-compose -f deploy/staging/docker-compose.staging.yml up -d

# Wait for services to start
sleep 60

# Run health check
./scripts/comprehensive-health-check.sh
```

**Staging Features**:
- Reduced resource requirements
- Mock external services (MailHog for email testing)
- Debug logging enabled
- Test endpoints enabled
- Relaxed security for testing

**Staging URLs**:
- **API Gateway**: http://localhost:8189
- **Services**: http://localhost:818X (X = service number)
- **MailHog**: http://localhost:8025
- **Grafana**: http://localhost:3002

---

## ✅ Post-Deployment Verification

### **1. Health Checks**
```bash
# Comprehensive health check
./scripts/comprehensive-health-check.sh

# Individual service checks
curl http://localhost:8080/health  # Auth Service
curl http://localhost:8081/health  # Sample Service
curl http://localhost:8083/health  # Template Service
curl http://localhost:8085/health  # Notification Service
curl http://localhost:8084/health  # Sequencing Service
```

### **2. Database Connectivity**
```bash
# Check PostgreSQL
docker exec tracseq-postgres-primary psql -U tracseq_admin -d tracseq_prod -c "SELECT version();"

# Check Redis
docker exec tracseq-redis-primary redis-cli ping
```

### **3. Authentication Test**
```bash
# Test login
curl -X POST http://localhost:8080/api/auth/login \
  -H "Content-Type: application/json" \
  -d '{"email":"admin@lab.local","password":"admin123"}'

# Test with token (replace TOKEN)
curl -H "Authorization: Bearer TOKEN" \
  http://localhost:8080/api/users/me
```

### **4. Integration Test**
```bash
# Run comprehensive tests
python test_microservices.py

# Check logs
docker logs tracseq-auth-service --tail 50
docker logs tracseq-api-gateway --tail 50
```

---

## 🔧 Service Configuration

### **Database Management**

**Create Admin User**:
```bash
# Using the auth service
docker exec tracseq-auth-service /app/create_admin \
  --email admin@lab.local \
  --password admin123 \
  --role LabAdmin
```

**Database Migrations**:
```bash
# Check migration status
docker exec tracseq-postgres-primary psql -U tracseq_admin -d tracseq_prod -c "\dt"

# Manual migration (if needed)
docker exec tracseq-postgres-primary psql -U tracseq_admin -d tracseq_prod -f /migrations/latest.sql
```

### **Service Configuration Updates**

**Update Environment Variables**:
```bash
# Edit production environment
nano deploy/.env.production

# Restart affected services
docker-compose -f deploy/production/docker-compose.production.yml restart auth-service
```

**Scale Services**:
```bash
# Scale based on load
docker-compose -f deploy/production/docker-compose.production.yml up -d --scale auth-service=3
docker-compose -f deploy/production/docker-compose.production.yml up -d --scale sample-service=2
```

---

## 📊 Monitoring Setup

### **Grafana Dashboard Setup**

1. **Access Grafana**: http://localhost:3001
2. **Login**: admin / (password from .env file)
3. **Add Prometheus Data Source**:
   - URL: http://prometheus:9090
   - Access: Server (default)

4. **Import Dashboards**:
   - TracSeq Overview Dashboard
   - Service Performance Dashboard
   - Infrastructure Monitoring Dashboard

### **Prometheus Metrics**

**Key Metrics to Monitor**:
- **Request Rate**: `rate(http_requests_total[5m])`
- **Error Rate**: `rate(http_requests_total{status=~"5.."}[5m])`
- **Response Time**: `histogram_quantile(0.95, http_request_duration_seconds_bucket)`
- **Database Connections**: `postgresql_connections_active`

### **Alerting Rules**

**Critical Alerts**:
- Service down for > 1 minute
- Error rate > 5% for > 5 minutes
- Database connection pool > 80%
- Disk space < 10%

**Warning Alerts**:
- Response time > 1 second
- Memory usage > 80%
- CPU usage > 70%

---

## 🛠️ Troubleshooting

### **Common Issues**

**Service Won't Start**:
```bash
# Check logs
docker logs tracseq-auth-service --tail 100

# Check resource usage
docker stats

# Restart service
docker-compose -f deploy/production/docker-compose.production.yml restart auth-service
```

**Database Connection Errors**:
```bash
# Test database connectivity
docker exec tracseq-postgres-primary pg_isready -U tracseq_admin

# Check connections
docker exec tracseq-postgres-primary psql -U tracseq_admin -d tracseq_prod -c "SELECT * FROM pg_stat_activity;"

# Reset connections
docker-compose restart postgres-primary
```

**Authentication Issues**:
```bash
# Check JWT secret consistency
docker exec tracseq-auth-service env | grep JWT_SECRET

# Test login endpoint
curl -v -X POST http://localhost:8080/api/auth/login \
  -H "Content-Type: application/json" \
  -d '{"email":"admin@lab.local","password":"admin123"}'

# Check user exists
docker exec tracseq-postgres-primary psql -U tracseq_admin -d tracseq_prod -c "SELECT * FROM users LIMIT 5;"
```

**Performance Issues**:
```bash
# Check resource constraints
docker stats --no-stream

# Monitor database performance
docker exec tracseq-postgres-primary psql -U tracseq_admin -d tracseq_prod -c "SELECT * FROM pg_stat_statements ORDER BY total_exec_time DESC LIMIT 10;"

# Check slow queries
docker logs tracseq-postgres-primary | grep "slow query"
```

### **Recovery Procedures**

**Complete System Recovery**:
```bash
# Stop all services
docker-compose -f deploy/production/docker-compose.production.yml down

# Clear volumes (CAUTION: Data loss)
docker volume prune

# Restore from backup
./scripts/restore-backup.sh backup-20240115

# Redeploy
./scripts/deploy-production.sh
```

**Database Recovery**:
```bash
# Restore database from backup
docker exec -i tracseq-postgres-primary psql -U tracseq_admin -d tracseq_prod < backups/20240115/database.sql
```

---

## 📈 Scaling & Performance

### **Horizontal Scaling**

**Scale Individual Services**:
```bash
# Scale auth service (most used)
docker-compose up -d --scale auth-service=3

# Scale sample service for batch operations
docker-compose up -d --scale sample-service=2

# Scale API gateway for load distribution
docker-compose up -d --scale api-gateway=2
```

**Load Balancer Configuration**:
```nginx
# Add to nginx.conf
upstream tracseq_auth {
    least_conn;
    server auth-service-1:8080;
    server auth-service-2:8080;
    server auth-service-3:8080;
}
```

### **Database Optimization**

**Connection Pool Tuning**:
```bash
# Update PostgreSQL configuration
echo "max_connections = 200" >> postgresql.conf
echo "shared_buffers = 1GB" >> postgresql.conf
echo "effective_cache_size = 3GB" >> postgresql.conf
```

**Read Replicas** (for high-read workloads):
```yaml
# Add to docker-compose
postgres-replica:
  image: postgres:15-alpine
  environment:
    POSTGRES_MASTER_SERVICE: postgres-primary
    POSTGRES_REPLICATION_USER: replicator
    POSTGRES_REPLICATION_PASSWORD: replica_password
```

### **Caching Strategy**

**Redis Configuration**:
```bash
# Increase Redis memory
REDIS_MAXMEMORY=4gb

# Configure eviction policy
REDIS_MAXMEMORY_POLICY=allkeys-lru
```

**Application-Level Caching**:
- **Session Cache**: 15-minute TTL
- **User Data Cache**: 5-minute TTL
- **Template Cache**: 1-hour TTL
- **Configuration Cache**: 24-hour TTL

### **Performance Benchmarks**

**Target Performance Metrics**:
- **Response Time**: <100ms (95th percentile)
- **Throughput**: 1,000+ requests/second
- **Availability**: 99.9% uptime
- **Database**: <10ms query time (95th percentile)

**Load Testing**:
```bash
# API Gateway load test
hey -n 10000 -c 100 http://localhost:8089/api/health

# Auth service load test
wrk -t12 -c400 -d30s http://localhost:8080/health

# Database connection test
pgbench -c 10 -T 60 tracseq_prod
```

---

## 🎉 Deployment Complete!

### **System Status Dashboard**

After successful deployment, you should have:

✅ **6 Production-Ready Services** running and healthy  
✅ **PostgreSQL** with optimized configuration  
✅ **Redis** for caching and sessions  
✅ **Monitoring Stack** (Grafana, Prometheus, Jaeger)  
✅ **Comprehensive Health Checks** passing  
✅ **Security** features enabled (JWT, RBAC, audit logging)  

### **Key URLs**
- **Application**: http://localhost:3000
- **API Gateway**: http://localhost:8089
- **Grafana Monitoring**: http://localhost:3001
- **Prometheus Metrics**: http://localhost:9090
- **Jaeger Tracing**: http://localhost:16686

### **Next Steps**

1. **Configure External Services**: Complete SMTP, SMS, Slack, AI API setup
2. **SSL/TLS Setup**: Configure production domain and certificates
3. **Backup Strategy**: Set up automated backups and test recovery
4. **User Training**: Onboard laboratory staff with user guides
5. **Phase 2 Deployment**: Fix and deploy remaining services
6. **Performance Tuning**: Optimize based on actual usage patterns

### **Support Resources**

- **Documentation**: `docs/` directory
- **API Reference**: http://localhost:8089/docs
- **Health Checks**: `./scripts/comprehensive-health-check.sh`
- **Logs**: `docker logs <service-name>`
- **Monitoring**: Grafana dashboards for system overview

---

**🏆 Congratulations! TracSeq 2.0 is now deployed and operational.**

*"Context improved by Giga AI"* 
