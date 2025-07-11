# TracSeq 2.0 API Gateway - Deployment Guide

[![Docker](https://img.shields.io/badge/Docker-ready-blue.svg)](https://www.docker.com/)
[![Kubernetes](https://img.shields.io/badge/Kubernetes-ready-green.svg)](https://kubernetes.io/)
[![Production](https://img.shields.io/badge/Production-ready-success.svg)](https://api.tracseq.com)

Complete deployment guide for the TracSeq 2.0 API Gateway modular architecture across different environments and platforms.

## Table of Contents

- [Overview](#overview)
- [Prerequisites](#prerequisites)
- [Environment Configuration](#environment-configuration)
- [Local Development](#local-development)
- [Docker Deployment](#docker-deployment)
- [Kubernetes Deployment](#kubernetes-deployment)
- [Production Deployment](#production-deployment)
- [Monitoring Setup](#monitoring-setup)
- [Security Configuration](#security-configuration)
- [Performance Tuning](#performance-tuning)
- [Troubleshooting](#troubleshooting)
- [Rollback Procedures](#rollback-procedures)

## Overview

The TracSeq 2.0 API Gateway uses a modular architecture designed for scalability, maintainability, and reliability. This guide covers deployment strategies for different environments.

### Architecture Components

```
┌─────────────────────────────────────────────────────────────┐
│                    Load Balancer                            │
├─────────────────────────────────────────────────────────────┤
│                API Gateway Cluster                          │
│  ┌─────────────┬─────────────┬─────────────┐              │
│  │  Gateway 1  │  Gateway 2  │  Gateway 3  │              │
│  │  (Primary)  │  (Replica)  │  (Replica)  │              │
│  └─────────────┴─────────────┴─────────────┘              │
├─────────────────────────────────────────────────────────────┤
│                    Service Mesh                             │
│  Auth │ Sample │ Storage │ Template │ Sequencing │ RAG     │
├─────────────────────────────────────────────────────────────┤
│               Shared Infrastructure                         │
│  Database │ Redis │ Monitoring │ Logging                   │
└─────────────────────────────────────────────────────────────┘
```

## Prerequisites

### System Requirements

#### Minimum Requirements
- **CPU**: 2 cores
- **RAM**: 4GB
- **Storage**: 20GB SSD
- **Network**: 100Mbps

#### Recommended Requirements
- **CPU**: 4+ cores
- **RAM**: 8GB+
- **Storage**: 50GB+ SSD
- **Network**: 1Gbps+

### Software Dependencies

#### Required
- **Python**: 3.9+
- **PostgreSQL**: 12+
- **Docker**: 20.10+
- **Docker Compose**: 2.0+

#### Optional
- **Kubernetes**: 1.20+
- **Helm**: 3.0+
- **Redis**: 6.0+
- **Nginx**: 1.18+

### Network Requirements

#### Ports
- **8000**: API Gateway HTTP
- **8080-8086**: Microservices
- **5432**: PostgreSQL
- **6379**: Redis (optional)
- **9090**: Prometheus (monitoring)
- **3000**: Grafana (monitoring)

#### Firewall Rules
```bash
# Allow inbound traffic
ufw allow 8000/tcp    # API Gateway
ufw allow 22/tcp      # SSH
ufw allow 80/tcp      # HTTP
ufw allow 443/tcp     # HTTPS

# Allow outbound traffic
ufw allow out 5432/tcp  # PostgreSQL
ufw allow out 6379/tcp  # Redis
ufw allow out 53/udp    # DNS
```

## Environment Configuration

### Environment Variables

#### Core Configuration
```bash
# Environment
export ENVIRONMENT=production
export GATEWAY_HOST=0.0.0.0
export GATEWAY_PORT=8000
export GATEWAY_DEBUG=false

# Database
export DATABASE_URL="postgres://user:password@db-host:5432/lims_db"
export DB_POOL_MIN_SIZE=10
export DB_POOL_MAX_SIZE=50
export DB_CONNECTION_TIMEOUT=30

# Security
export JWT_SECRET_KEY="$(openssl rand -base64 32)"
export JWT_ALGORITHM=HS256
export JWT_EXPIRATION_HOURS=8

# Services
export AUTH_SERVICE_URL=http://auth-service:8080
export SAMPLE_SERVICE_URL=http://sample-service:8081
export STORAGE_SERVICE_URL=http://storage-service:8082
export TEMPLATE_SERVICE_URL=http://template-service:8083
export SEQUENCING_SERVICE_URL=http://sequencing-service:8084
export RAG_SERVICE_URL=http://rag-service:8000

# Rate Limiting
export RATE_LIMIT_REQUESTS=100
export RATE_LIMIT_WINDOW=60
export ADAPTIVE_RATE_LIMITING=true

# Monitoring
export ENABLE_METRICS=true
export HEALTH_CHECK_INTERVAL=30
export CIRCUIT_BREAKER_FAILURE_THRESHOLD=5
export CIRCUIT_BREAKER_RECOVERY_TIMEOUT=60

# Logging
export LOG_LEVEL=INFO
export LOG_FILE=/var/log/tracseq/gateway.log
export ENABLE_ACCESS_LOG=true
export ENABLE_SQL_LOGGING=false
```

#### Environment-Specific Files

**Development (.env.development)**
```bash
ENVIRONMENT=development
GATEWAY_DEBUG=true
DATABASE_URL=postgres://postgres:postgres@localhost:5432/lims_dev
LOG_LEVEL=DEBUG
ENABLE_SQL_LOGGING=true
RATE_LIMIT_REQUESTS=1000
```

**Staging (.env.staging)**
```bash
ENVIRONMENT=staging
DATABASE_URL=postgres://user:password@staging-db:5432/lims_staging
LOG_LEVEL=INFO
RATE_LIMIT_REQUESTS=200
CIRCUIT_BREAKER_FAILURE_THRESHOLD=3
```

**Production (.env.production)**
```bash
ENVIRONMENT=production
DATABASE_URL=postgres://user:password@prod-db:5432/lims_prod
LOG_LEVEL=INFO
LOG_FILE=/var/log/tracseq/gateway.log
RATE_LIMIT_REQUESTS=100
CIRCUIT_BREAKER_FAILURE_THRESHOLD=5
```

## Local Development

### Quick Start

```bash
# Clone repository
git clone https://github.com/tracseq/api-gateway.git
cd api-gateway

# Set up virtual environment
python -m venv venv
source venv/bin/activate  # On Windows: venv\Scripts\activate

# Install dependencies
pip install -r requirements.txt
pip install -r requirements-dev.txt

# Set up environment
cp .env.example .env.development
source .env.development

# Start database
docker run -d --name postgres-dev \
  -e POSTGRES_DB=lims_dev \
  -e POSTGRES_USER=postgres \
  -e POSTGRES_PASSWORD=postgres \
  -p 5432:5432 postgres:13

# Run migrations
python -m alembic upgrade head

# Start the gateway
python -m src.api_gateway.main_modular
```

### Development with Docker Compose

```yaml
# docker-compose.dev.yml
version: '3.8'

services:
  api-gateway:
    build:
      context: .
      dockerfile: Dockerfile.dev
    ports:
      - "8000:8000"
    environment:
      - ENVIRONMENT=development
      - DATABASE_URL=postgres://postgres:postgres@db:5432/lims_dev
      - JWT_SECRET_KEY=dev-secret-key
      - LOG_LEVEL=DEBUG
    volumes:
      - ./src:/app/src
      - ./config:/app/config
    depends_on:
      - db
    restart: unless-stopped

  db:
    image: postgres:13
    environment:
      POSTGRES_DB: lims_dev
      POSTGRES_USER: postgres
      POSTGRES_PASSWORD: postgres
    volumes:
      - postgres_dev_data:/var/lib/postgresql/data
    ports:
      - "5432:5432"

  redis:
    image: redis:6-alpine
    ports:
      - "6379:6379"

volumes:
  postgres_dev_data:
```

```bash
# Start development environment
docker-compose -f docker-compose.dev.yml up -d

# View logs
docker-compose -f docker-compose.dev.yml logs -f api-gateway

# Stop environment
docker-compose -f docker-compose.dev.yml down
```

## Docker Deployment

### Single Container Deployment

#### Dockerfile
```dockerfile
FROM python:3.9-slim

LABEL maintainer="TracSeq Team <dev@tracseq.com>"
LABEL version="2.0.0"

# Set working directory
WORKDIR /app

# Install system dependencies
RUN apt-get update && apt-get install -y \
    gcc \
    libpq-dev \
    && rm -rf /var/lib/apt/lists/*

# Copy requirements first for better caching
COPY requirements.txt .
RUN pip install --no-cache-dir -r requirements.txt

# Copy application code
COPY src/ ./src/
COPY config/ ./config/
COPY alembic.ini .
COPY alembic/ ./alembic/

# Create non-root user
RUN groupadd -r tracseq && useradd -r -g tracseq tracseq
RUN chown -R tracseq:tracseq /app
USER tracseq

# Health check
HEALTHCHECK --interval=30s --timeout=30s --start-period=5s --retries=3 \
  CMD curl -f http://localhost:8000/health || exit 1

# Expose port
EXPOSE 8000

# Start application
CMD ["python", "-m", "src.api_gateway.main_modular"]
```

#### Build and Run
```bash
# Build image
docker build -t tracseq/api-gateway:2.0.0 .

# Run container
docker run -d \
  --name api-gateway \
  -p 8000:8000 \
  -e DATABASE_URL="postgres://user:password@host:5432/lims_db" \
  -e JWT_SECRET_KEY="your-secret-key" \
  -e ENVIRONMENT=production \
  --restart unless-stopped \
  tracseq/api-gateway:2.0.0

# Check logs
docker logs -f api-gateway

# Check health
curl http://localhost:8000/health
```

### Multi-Container Deployment

#### docker-compose.yml
```yaml
version: '3.8'

services:
  api-gateway:
    image: tracseq/api-gateway:2.0.0
    ports:
      - "8000:8000"
    environment:
      - ENVIRONMENT=production
      - DATABASE_URL=postgres://postgres:${POSTGRES_PASSWORD}@db:5432/lims_prod
      - JWT_SECRET_KEY=${JWT_SECRET_KEY}
      - REDIS_URL=redis://redis:6379/0
      - LOG_LEVEL=INFO
    depends_on:
      - db
      - redis
    restart: unless-stopped
    networks:
      - tracseq-network
    volumes:
      - ./logs:/var/log/tracseq
    deploy:
      replicas: 3
      resources:
        limits:
          cpus: '0.5'
          memory: 512M
        reservations:
          cpus: '0.25'
          memory: 256M

  db:
    image: postgres:13
    environment:
      POSTGRES_DB: lims_prod
      POSTGRES_USER: postgres
      POSTGRES_PASSWORD: ${POSTGRES_PASSWORD}
    volumes:
      - postgres_data:/var/lib/postgresql/data
      - ./backups:/backups
    networks:
      - tracseq-network
    restart: unless-stopped

  redis:
    image: redis:6-alpine
    command: redis-server --appendonly yes
    volumes:
      - redis_data:/data
    networks:
      - tracseq-network
    restart: unless-stopped

  nginx:
    image: nginx:alpine
    ports:
      - "80:80"
      - "443:443"
    volumes:
      - ./nginx.conf:/etc/nginx/nginx.conf
      - ./ssl:/etc/nginx/ssl
    depends_on:
      - api-gateway
    networks:
      - tracseq-network
    restart: unless-stopped

networks:
  tracseq-network:
    driver: bridge

volumes:
  postgres_data:
  redis_data:
```

#### Environment File (.env)
```bash
# Database
POSTGRES_PASSWORD=secure-password-here

# Security
JWT_SECRET_KEY=your-super-secret-jwt-key-here

# Optional
REDIS_PASSWORD=redis-password-here
```

#### Nginx Configuration
```nginx
# nginx.conf
events {
    worker_connections 1024;
}

http {
    upstream api_gateway {
        server api-gateway:8000;
    }

    server {
        listen 80;
        server_name api.tracseq.com;

        location / {
            proxy_pass http://api_gateway;
            proxy_set_header Host $host;
            proxy_set_header X-Real-IP $remote_addr;
            proxy_set_header X-Forwarded-For $proxy_add_x_forwarded_for;
            proxy_set_header X-Forwarded-Proto $scheme;
        }

        location /health {
            proxy_pass http://api_gateway/health;
            access_log off;
        }
    }
}
```

#### Deployment Commands
```bash
# Start services
docker-compose up -d

# Scale gateway instances
docker-compose up -d --scale api-gateway=3

# View logs
docker-compose logs -f api-gateway

# Update service
docker-compose pull api-gateway
docker-compose up -d api-gateway

# Stop services
docker-compose down
```

## Kubernetes Deployment

### Namespace and ConfigMap

#### namespace.yaml
```yaml
apiVersion: v1
kind: Namespace
metadata:
  name: tracseq
  labels:
    name: tracseq
    environment: production
```

#### configmap.yaml
```yaml
apiVersion: v1
kind: ConfigMap
metadata:
  name: api-gateway-config
  namespace: tracseq
data:
  ENVIRONMENT: "production"
  GATEWAY_HOST: "0.0.0.0"
  GATEWAY_PORT: "8000"
  LOG_LEVEL: "INFO"
  RATE_LIMIT_REQUESTS: "100"
  RATE_LIMIT_WINDOW: "60"
  ENABLE_METRICS: "true"
  HEALTH_CHECK_INTERVAL: "30"
  CIRCUIT_BREAKER_FAILURE_THRESHOLD: "5"
```

### Secrets

#### secrets.yaml
```yaml
apiVersion: v1
kind: Secret
metadata:
  name: api-gateway-secrets
  namespace: tracseq
type: Opaque
data:
  DATABASE_URL: <base64-encoded-database-url>
  JWT_SECRET_KEY: <base64-encoded-jwt-secret>
  REDIS_PASSWORD: <base64-encoded-redis-password>
```

```bash
# Create secrets
kubectl create secret generic api-gateway-secrets \
  --from-literal=DATABASE_URL="postgres://user:password@postgres:5432/lims_prod" \
  --from-literal=JWT_SECRET_KEY="your-secret-key" \
  --namespace=tracseq
```

### Deployment

#### deployment.yaml
```yaml
apiVersion: apps/v1
kind: Deployment
metadata:
  name: api-gateway
  namespace: tracseq
  labels:
    app: api-gateway
    version: v2.0.0
spec:
  replicas: 3
  strategy:
    type: RollingUpdate
    rollingUpdate:
      maxSurge: 1
      maxUnavailable: 0
  selector:
    matchLabels:
      app: api-gateway
  template:
    metadata:
      labels:
        app: api-gateway
        version: v2.0.0
    spec:
      containers:
      - name: api-gateway
        image: tracseq/api-gateway:2.0.0
        ports:
        - containerPort: 8000
          name: http
        env:
        - name: DATABASE_URL
          valueFrom:
            secretKeyRef:
              name: api-gateway-secrets
              key: DATABASE_URL
        - name: JWT_SECRET_KEY
          valueFrom:
            secretKeyRef:
              name: api-gateway-secrets
              key: JWT_SECRET_KEY
        envFrom:
        - configMapRef:
            name: api-gateway-config
        resources:
          requests:
            memory: "256Mi"
            cpu: "250m"
          limits:
            memory: "512Mi"
            cpu: "500m"
        livenessProbe:
          httpGet:
            path: /health
            port: 8000
          initialDelaySeconds: 30
          periodSeconds: 10
          timeoutSeconds: 5
          failureThreshold: 3
        readinessProbe:
          httpGet:
            path: /health
            port: 8000
          initialDelaySeconds: 5
          periodSeconds: 5
          timeoutSeconds: 3
          failureThreshold: 3
        volumeMounts:
        - name: logs
          mountPath: /var/log/tracseq
      volumes:
      - name: logs
        emptyDir: {}
      restartPolicy: Always
```

### Service

#### service.yaml
```yaml
apiVersion: v1
kind: Service
metadata:
  name: api-gateway-service
  namespace: tracseq
  labels:
    app: api-gateway
spec:
  selector:
    app: api-gateway
  ports:
  - name: http
    port: 80
    targetPort: 8000
    protocol: TCP
  type: ClusterIP
```

### Ingress

#### ingress.yaml
```yaml
apiVersion: networking.k8s.io/v1
kind: Ingress
metadata:
  name: api-gateway-ingress
  namespace: tracseq
  annotations:
    kubernetes.io/ingress.class: "nginx"
    nginx.ingress.kubernetes.io/ssl-redirect: "true"
    nginx.ingress.kubernetes.io/rate-limit: "100"
    nginx.ingress.kubernetes.io/rate-limit-window: "1m"
    cert-manager.io/cluster-issuer: "letsencrypt-prod"
spec:
  tls:
  - hosts:
    - api.tracseq.com
    secretName: api-gateway-tls
  rules:
  - host: api.tracseq.com
    http:
      paths:
      - path: /
        pathType: Prefix
        backend:
          service:
            name: api-gateway-service
            port:
              number: 80
```

### Horizontal Pod Autoscaler

#### hpa.yaml
```yaml
apiVersion: autoscaling/v2
kind: HorizontalPodAutoscaler
metadata:
  name: api-gateway-hpa
  namespace: tracseq
spec:
  scaleTargetRef:
    apiVersion: apps/v1
    kind: Deployment
    name: api-gateway
  minReplicas: 3
  maxReplicas: 10
  metrics:
  - type: Resource
    resource:
      name: cpu
      target:
        type: Utilization
        averageUtilization: 70
  - type: Resource
    resource:
      name: memory
      target:
        type: Utilization
        averageUtilization: 80
```

### Deployment Commands

```bash
# Apply all configurations
kubectl apply -f namespace.yaml
kubectl apply -f configmap.yaml
kubectl apply -f secrets.yaml
kubectl apply -f deployment.yaml
kubectl apply -f service.yaml
kubectl apply -f ingress.yaml
kubectl apply -f hpa.yaml

# Check deployment status
kubectl get deployments -n tracseq
kubectl get pods -n tracseq
kubectl get services -n tracseq

# View logs
kubectl logs -f deployment/api-gateway -n tracseq

# Scale deployment
kubectl scale deployment api-gateway --replicas=5 -n tracseq

# Rolling update
kubectl set image deployment/api-gateway api-gateway=tracseq/api-gateway:2.0.1 -n tracseq

# Check rollout status
kubectl rollout status deployment/api-gateway -n tracseq
```

## Production Deployment

### High Availability Setup

#### Load Balancer Configuration
```nginx
# /etc/nginx/nginx.conf
upstream api_gateway_cluster {
    least_conn;
    server gateway-1.tracseq.com:8000 max_fails=3 fail_timeout=30s;
    server gateway-2.tracseq.com:8000 max_fails=3 fail_timeout=30s;
    server gateway-3.tracseq.com:8000 max_fails=3 fail_timeout=30s;
}

server {
    listen 443 ssl http2;
    server_name api.tracseq.com;

    ssl_certificate /etc/ssl/certs/tracseq.crt;
    ssl_certificate_key /etc/ssl/private/tracseq.key;

    location / {
        proxy_pass http://api_gateway_cluster;
        proxy_set_header Host $host;
        proxy_set_header X-Real-IP $remote_addr;
        proxy_set_header X-Forwarded-For $proxy_add_x_forwarded_for;
        proxy_set_header X-Forwarded-Proto $scheme;
        
        # Health check
        proxy_next_upstream error timeout http_500 http_502 http_503;
        proxy_connect_timeout 5s;
        proxy_send_timeout 10s;
        proxy_read_timeout 10s;
    }

    location /health {
        proxy_pass http://api_gateway_cluster/health;
        access_log off;
    }
}
```

#### Database High Availability
```yaml
# PostgreSQL with replication
version: '3.8'

services:
  postgres-primary:
    image: postgres:13
    environment:
      POSTGRES_DB: lims_prod
      POSTGRES_USER: postgres
      POSTGRES_PASSWORD: ${POSTGRES_PASSWORD}
      POSTGRES_REPLICATION_USER: replicator
      POSTGRES_REPLICATION_PASSWORD: ${REPLICATION_PASSWORD}
    command: |
      postgres
      -c wal_level=replica
      -c max_wal_senders=3
      -c max_replication_slots=3
      -c hot_standby=on
    volumes:
      - postgres_primary_data:/var/lib/postgresql/data

  postgres-replica:
    image: postgres:13
    environment:
      POSTGRES_USER: postgres
      POSTGRES_PASSWORD: ${POSTGRES_PASSWORD}
      PGUSER: postgres
      POSTGRES_PRIMARY_USER: replicator
      POSTGRES_PRIMARY_PASSWORD: ${REPLICATION_PASSWORD}
      POSTGRES_PRIMARY_HOST: postgres-primary
    command: |
      bash -c "
      until pg_basebackup --pgdata=/var/lib/postgresql/data -R --slot=replication_slot --host=postgres-primary --port=5432
      do
        echo 'Waiting for primary to connect...'
        sleep 1s
      done
      echo 'Backup done, starting replica...'
      postgres
      "
    depends_on:
      - postgres-primary
    volumes:
      - postgres_replica_data:/var/lib/postgresql/data

volumes:
  postgres_primary_data:
  postgres_replica_data:
```

### SSL/TLS Configuration

#### SSL Certificate Setup
```bash
# Using Let's Encrypt
certbot --nginx -d api.tracseq.com

# Or using custom certificate
openssl req -x509 -newkey rsa:4096 -keyout tracseq.key -out tracseq.crt -days 365 -nodes
```

#### SSL Configuration
```nginx
# Strong SSL configuration
ssl_protocols TLSv1.2 TLSv1.3;
ssl_ciphers ECDHE-RSA-AES256-GCM-SHA512:DHE-RSA-AES256-GCM-SHA512:ECDHE-RSA-AES256-GCM-SHA384:DHE-RSA-AES256-GCM-SHA384;
ssl_prefer_server_ciphers off;
ssl_session_cache shared:SSL:10m;
ssl_session_tickets off;
ssl_stapling on;
ssl_stapling_verify on;

# Security headers
add_header Strict-Transport-Security "max-age=63072000" always;
add_header X-Frame-Options DENY;
add_header X-Content-Type-Options nosniff;
add_header X-XSS-Protection "1; mode=block";
```

### Database Optimization

#### PostgreSQL Configuration
```sql
-- postgresql.conf optimizations
shared_buffers = 256MB
effective_cache_size = 1GB
maintenance_work_mem = 64MB
checkpoint_completion_target = 0.9
wal_buffers = 16MB
default_statistics_target = 100
random_page_cost = 1.1
effective_io_concurrency = 200
work_mem = 4MB
min_wal_size = 1GB
max_wal_size = 4GB
max_connections = 100
```

#### Database Monitoring
```sql
-- Create monitoring user
CREATE USER monitoring WITH PASSWORD 'monitoring_password';
GRANT pg_monitor TO monitoring;

-- Useful monitoring queries
SELECT * FROM pg_stat_activity WHERE state = 'active';
SELECT * FROM pg_stat_database WHERE datname = 'lims_prod';
SELECT * FROM pg_stat_user_tables WHERE relname LIKE 'api_gateway%';
```

## Monitoring Setup

### Prometheus Configuration

#### prometheus.yml
```yaml
global:
  scrape_interval: 15s
  evaluation_interval: 15s

rule_files:
  - "alert_rules.yml"

scrape_configs:
  - job_name: 'api-gateway'
    static_configs:
      - targets: ['api-gateway:8000']
    metrics_path: '/metrics'
    scrape_interval: 10s

  - job_name: 'postgres'
    static_configs:
      - targets: ['postgres-exporter:9187']

alerting:
  alertmanagers:
    - static_configs:
        - targets:
          - alertmanager:9093
```

#### Alert Rules
```yaml
# alert_rules.yml
groups:
- name: api-gateway
  rules:
  - alert: APIGatewayDown
    expr: up{job="api-gateway"} == 0
    for: 1m
    labels:
      severity: critical
    annotations:
      summary: "API Gateway is down"
      description: "API Gateway has been down for more than 1 minute"

  - alert: HighResponseTime
    expr: http_request_duration_seconds{quantile="0.95"} > 1
    for: 5m
    labels:
      severity: warning
    annotations:
      summary: "High response time"
      description: "95th percentile response time is above 1 second"

  - alert: HighErrorRate
    expr: rate(http_requests_total{status=~"5.."}[5m]) > 0.1
    for: 5m
    labels:
      severity: critical
    annotations:
      summary: "High error rate"
      description: "Error rate is above 10%"
```

### Grafana Dashboard

#### dashboard.json
```json
{
  "dashboard": {
    "title": "TracSeq API Gateway",
    "panels": [
      {
        "title": "Request Rate",
        "type": "graph",
        "targets": [
          {
            "expr": "rate(http_requests_total[5m])",
            "legendFormat": "{{method}} {{status}}"
          }
        ]
      },
      {
        "title": "Response Time",
        "type": "graph",
        "targets": [
          {
            "expr": "histogram_quantile(0.95, rate(http_request_duration_seconds_bucket[5m]))",
            "legendFormat": "95th percentile"
          }
        ]
      },
      {
        "title": "Error Rate",
        "type": "singlestat",
        "targets": [
          {
            "expr": "rate(http_requests_total{status=~\"5..\"}[5m]) / rate(http_requests_total[5m])",
            "legendFormat": "Error Rate"
          }
        ]
      }
    ]
  }
}
```

## Security Configuration

### Security Checklist

#### Application Security
- [ ] Strong JWT secret key (32+ characters)
- [ ] Database credentials secured
- [ ] API rate limiting enabled
- [ ] Input validation implemented
- [ ] SQL injection protection
- [ ] XSS protection headers
- [ ] CSRF protection enabled
- [ ] Secure session management

#### Infrastructure Security
- [ ] SSL/TLS certificates configured
- [ ] Firewall rules configured
- [ ] Database access restricted
- [ ] Container security scanning
- [ ] Secrets management implemented
- [ ] Network segmentation
- [ ] Regular security updates
- [ ] Backup encryption

#### Monitoring Security
- [ ] Security event logging
- [ ] Intrusion detection
- [ ] Audit trail maintenance
- [ ] Compliance monitoring
- [ ] Incident response plan
- [ ] Security metrics tracking

### Security Configuration Files

#### Security Headers
```python
# middleware/security.py additions
SECURITY_HEADERS = {
    "Strict-Transport-Security": "max-age=31536000; includeSubDomains",
    "X-Content-Type-Options": "nosniff",
    "X-Frame-Options": "DENY",
    "X-XSS-Protection": "1; mode=block",
    "Referrer-Policy": "strict-origin-when-cross-origin",
    "Content-Security-Policy": "default-src 'self'; script-src 'self' 'unsafe-inline'",
    "Permissions-Policy": "geolocation=(), microphone=(), camera=()"
}
```

## Performance Tuning

### Application Performance

#### Python Optimizations
```python
# uvicorn configuration
uvicorn.run(
    "src.api_gateway.main_modular:app",
    host="0.0.0.0",
    port=8000,
    workers=4,  # CPU cores
    worker_class="uvicorn.workers.UvicornWorker",
    worker_connections=1000,
    max_requests=1000,
    max_requests_jitter=50,
    preload_app=True,
    access_log=True,
    use_colors=False
)
```

#### Database Connection Pooling
```python
# core/database.py optimizations
DATABASE_CONFIG = {
    "pool_size": 20,
    "max_overflow": 30,
    "pool_pre_ping": True,
    "pool_recycle": 3600,
    "echo": False,
    "connect_args": {
        "connect_timeout": 10,
        "server_settings": {
            "application_name": "api_gateway",
            "jit": "off"
        }
    }
}
```

### System Performance

#### OS Optimizations
```bash
# /etc/sysctl.conf
net.core.somaxconn = 1024
net.core.netdev_max_backlog = 5000
net.core.rmem_default = 262144
net.core.rmem_max = 16777216
net.core.wmem_default = 262144
net.core.wmem_max = 16777216
net.ipv4.tcp_rmem = 4096 87380 16777216
net.ipv4.tcp_wmem = 4096 65536 16777216
net.ipv4.tcp_congestion_control = bbr
```

#### Docker Performance
```yaml
# docker-compose.yml optimizations
services:
  api-gateway:
    deploy:
      resources:
        limits:
          cpus: '2.0'
          memory: 1G
        reservations:
          cpus: '1.0'
          memory: 512M
    ulimits:
      nofile:
        soft: 65536
        hard: 65536
    sysctls:
      - net.core.somaxconn=1024
```

## Troubleshooting

### Common Issues

#### 1. Database Connection Issues
```bash
# Check database connectivity
psql -h localhost -U postgres -d lims_prod -c "SELECT 1"

# Check connection pool
curl http://localhost:8000/health/detailed | jq '.database'

# Fix connection pool exhaustion
export DB_POOL_MAX_SIZE=50
export DB_CONNECTION_TIMEOUT=30
```

#### 2. Service Discovery Issues
```bash
# Check service health
curl http://localhost:8000/metrics | jq '.services'

# Test service connectivity
curl http://auth-service:8080/health
curl http://sample-service:8081/health

# Check DNS resolution
nslookup auth-service
```

#### 3. Memory Issues
```bash
# Check memory usage
docker stats api-gateway
kubectl top pods -n tracseq

# Increase memory limits
docker run -m 1g tracseq/api-gateway:2.0.0

# Kubernetes
kubectl patch deployment api-gateway -n tracseq -p '{"spec":{"template":{"spec":{"containers":[{"name":"api-gateway","resources":{"limits":{"memory":"1Gi"}}}]}}}}'
```

#### 4. Performance Issues
```bash
# Check response times
curl -w "@curl-format.txt" -o /dev/null -s http://localhost:8000/health

# Monitor with Apache Bench
ab -n 1000 -c 10 http://localhost:8000/health

# Check bottlenecks
docker exec api-gateway python -m cProfile -o profile.stats main.py
```

### Debugging Tools

#### Log Analysis
```bash
# Follow logs
tail -f /var/log/tracseq/gateway.log

# Search for errors
grep -i error /var/log/tracseq/gateway.log

# JSON log parsing
cat /var/log/tracseq/gateway.log | jq 'select(.level == "ERROR")'
```

#### Performance Profiling
```python
# Add to main_modular.py for debugging
import cProfile
import pstats

def profile_app():
    profiler = cProfile.Profile()
    profiler.enable()
    
    # Your app code here
    
    profiler.disable()
    stats = pstats.Stats(profiler)
    stats.sort_stats('cumulative')
    stats.print_stats(20)
```

## Rollback Procedures

### Docker Rollback
```bash
# Tag current version
docker tag tracseq/api-gateway:2.0.0 tracseq/api-gateway:2.0.0-backup

# Rollback to previous version
docker pull tracseq/api-gateway:1.9.0
docker stop api-gateway
docker rm api-gateway
docker run -d --name api-gateway tracseq/api-gateway:1.9.0

# Verify rollback
curl http://localhost:8000/health
```

### Kubernetes Rollback
```bash
# Check rollout history
kubectl rollout history deployment/api-gateway -n tracseq

# Rollback to previous version
kubectl rollout undo deployment/api-gateway -n tracseq

# Rollback to specific revision
kubectl rollout undo deployment/api-gateway --to-revision=2 -n tracseq

# Check rollback status
kubectl rollout status deployment/api-gateway -n tracseq
```

### Database Rollback
```bash
# Backup current database
pg_dump -h localhost -U postgres lims_prod > backup_$(date +%Y%m%d_%H%M%S).sql

# Restore from backup
psql -h localhost -U postgres lims_prod < backup_20240115_103000.sql

# Run migration rollback
python -m alembic downgrade -1
```

## Best Practices

### Deployment Best Practices
1. **Blue-Green Deployment**: Use two identical environments
2. **Rolling Updates**: Gradual replacement of instances
3. **Health Checks**: Comprehensive health monitoring
4. **Automated Testing**: CI/CD pipeline integration
5. **Monitoring**: Real-time performance monitoring
6. **Backup Strategy**: Regular automated backups
7. **Security Updates**: Regular security patches
8. **Documentation**: Keep deployment docs updated

### Operational Best Practices
1. **Monitoring**: Set up comprehensive monitoring
2. **Alerting**: Configure meaningful alerts
3. **Logging**: Centralized log management
4. **Backup**: Regular database and config backups
5. **Security**: Regular security audits
6. **Performance**: Regular performance tuning
7. **Disaster Recovery**: Documented recovery procedures
8. **Team Training**: Ensure team knows procedures

---

*Last updated: January 15, 2024*
*Version: 2.0.0*

*Context improved by Giga AI*