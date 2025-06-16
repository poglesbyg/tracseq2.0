# Lab Manager - Production Deployment Guide

This directory contains all the configuration files and documentation needed to deploy Lab Manager in production environments.

## 📁 Files Overview

- `nginx.conf` - Nginx configuration for serving frontend and proxying API requests
- `docker-compose.production.yml` - Production Docker Compose configuration
- `production.env.example` - Example environment variables file
- `backups/` - Directory for database backups

## 🚀 Quick Start

### 1. Environment Setup

```bash
# Copy the example environment file
cp deploy/production.env.example deploy/production.env

# Edit the environment file with your production values
vim deploy/production.env
```

**⚠️ Important:** Set a strong password for `POSTGRES_PASSWORD` and update other security-related variables.

### 2. Deploy with Docker Compose

```bash
# Deploy the full stack
docker-compose -f deploy/docker-compose.production.yml --env-file deploy/production.env up -d

# Check service status
docker-compose -f deploy/docker-compose.production.yml ps

# View logs
docker-compose -f deploy/docker-compose.production.yml logs -f app
```

### 3. Verify Deployment

```bash
# Check application health
curl http://localhost/health

# Test API endpoints
curl http://localhost/api/dashboard/stats

# Access the frontend
open http://localhost
```

## 🏗️ Architecture Overview

### Full Stack Deployment

```
┌─────────────┐    ┌─────────────┐    ┌─────────────┐
│   Internet  │    │    Nginx    │    │ Lab Manager │
│             │────▶│  (Port 80)  │────▶│ Backend     │
│             │    │             │    │ (Port 3000) │
└─────────────┘    └─────────────┘    └─────────────┘
                            │                │
                            ▼                ▼
                   ┌─────────────┐    ┌─────────────┐
                   │  Frontend   │    │ PostgreSQL  │
                   │   Static    │    │  Database   │
                   │   Files     │    │             │
                   └─────────────┘    └─────────────┘
```

### Service Components

- **Nginx**: Serves React frontend, proxies API calls to backend
- **Lab Manager Backend**: Rust/Axum API server with business logic
- **PostgreSQL**: Primary database for all application data
- **Redis** (Optional): Caching and session management
- **Backup Service**: Automated database backups

## 🔐 Security Considerations

### 1. Database Security
- Use strong passwords for database user
- Enable SSL/TLS for database connections in production
- Regular security updates for PostgreSQL

### 2. Application Security
- Keep Docker images updated
- Configure proper firewall rules
- Use SSL/TLS for web traffic (add reverse proxy if needed)

### 3. Access Control
- Limit database access to application containers only
- Use Docker secrets for sensitive environment variables
- Regular backup and disaster recovery testing

## 📊 Monitoring & Maintenance

### Health Checks

All services include health checks:

```bash
# Check all service health
docker-compose -f deploy/docker-compose.production.yml ps

# Detailed health status
docker inspect $(docker-compose -f deploy/docker-compose.production.yml ps -q app) | jq '.[0].State.Health'
```

### Logs

```bash
# Application logs
docker-compose -f deploy/docker-compose.production.yml logs app

# Database logs
docker-compose -f deploy/docker-compose.production.yml logs db

# Nginx access logs
docker-compose -f deploy/docker-compose.production.yml exec app tail -f /var/log/nginx/access.log
```

### Backups

```bash
# Manual backup
docker-compose -f deploy/docker-compose.production.yml --profile backup run --rm backup

# Automated backups (add to crontab)
0 2 * * * cd /path/to/lab_manager && docker-compose -f deploy/docker-compose.production.yml --profile backup run --rm backup
```

## 🔧 CI/CD Integration

### GitHub Actions

The project includes automated deployment via GitHub Actions:

1. **Trigger**: Push to `main` branch or manual workflow dispatch
2. **Build**: Multi-architecture Docker images (amd64, arm64)
3. **Deploy**: Automated deployment to development/staging/production
4. **Variants**: Supports full-stack, API-only, and microservices deployments

### Manual Image Build

```bash
# Build production image
docker build -f Dockerfile.deploy -t lab-manager:latest .

# Tag for registry
docker tag lab-manager:latest your-registry/lab-manager:v1.0.0

# Push to registry
docker push your-registry/lab-manager:v1.0.0
```

## 🛠️ Configuration Options

### Environment Variables

| Variable | Description | Required | Default |
|----------|-------------|----------|---------|
| `POSTGRES_PASSWORD` | Database password | ✅ | - |
| `POSTGRES_USER` | Database username | ❌ | postgres |
| `POSTGRES_DB` | Database name | ❌ | lab_manager |
| `LAB_MANAGER_IMAGE` | Docker image tag | ❌ | lab-manager:latest |
| `RUST_LOG` | Log level | ❌ | info |

### Service Profiles

Optional services can be enabled with Docker Compose profiles:

```bash
# Enable backup service
docker-compose -f deploy/docker-compose.production.yml --profile backup up -d

# Enable logging service
docker-compose -f deploy/docker-compose.production.yml --profile logging up -d

# Enable all optional services
docker-compose -f deploy/docker-compose.production.yml --profile backup --profile logging up -d
```

## 🌐 Reverse Proxy Setup (Optional)

For production environments with SSL/TLS, add a reverse proxy:

### Traefik Example

```yaml
# traefik-docker-compose.yml
version: '3.8'
services:
  traefik:
    image: traefik:v2.10
    command:
      - --entrypoints.web.address=:80
      - --entrypoints.websecure.address=:443
      - --certificatesresolvers.myresolver.acme.tlschallenge=true
      - --certificatesresolvers.myresolver.acme.email=admin@yourdomain.com
      - --certificatesresolvers.myresolver.acme.storage=/letsencrypt/acme.json
    ports:
      - "80:80"
      - "443:443"
    volumes:
      - /var/run/docker.sock:/var/run/docker.sock:ro
      - letsencrypt:/letsencrypt
    labels:
      - "traefik.http.routers.lab-manager.rule=Host(\`lab-manager.yourdomain.com\`)"
      - "traefik.http.routers.lab-manager.tls.certresolver=myresolver"
```

## 📞 Troubleshooting

### Common Issues

#### 1. Database Connection Failed
```bash
# Check database logs
docker-compose -f deploy/docker-compose.production.yml logs db

# Verify environment variables
docker-compose -f deploy/docker-compose.production.yml config
```

#### 2. Frontend Not Loading
```bash
# Check nginx configuration
docker-compose -f deploy/docker-compose.production.yml exec app nginx -t

# Check frontend build
docker-compose -f deploy/docker-compose.production.yml exec app ls -la /var/www/html/
```

#### 3. API Requests Failing
```bash
# Check backend logs
docker-compose -f deploy/docker-compose.production.yml logs app

# Test direct backend connection
curl http://localhost:3000/health
```

### Recovery Procedures

#### Database Recovery
```bash
# Restore from backup
docker-compose -f deploy/docker-compose.production.yml exec db psql -U postgres lab_manager < /backups/lab_manager_YYYYMMDD_HHMMSS.sql
```

#### Application Recovery
```bash
# Restart services
docker-compose -f deploy/docker-compose.production.yml restart

# Full redeploy
docker-compose -f deploy/docker-compose.production.yml down
docker-compose -f deploy/docker-compose.production.yml pull
docker-compose -f deploy/docker-compose.production.yml up -d
```

## 📚 Additional Resources

- [Lab Manager Documentation](../README.md)
- [Development Setup](../README.md#development)
- [API Documentation](../docs/API.md)
- [Contributing Guide](../CONTRIBUTING.md)

---

**⚠️ Security Notice**: Always review and update security settings before production deployment. Regularly update dependencies and monitor for security advisories. 
