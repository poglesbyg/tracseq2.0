# TracSeq 2.0 Environment Variables Documentation

This document provides comprehensive documentation for all environment variables used in the TracSeq 2.0 Laboratory Management System.

## Table of Contents

1. [Quick Start](#quick-start)
2. [General Configuration](#general-configuration)
3. [Database Configuration](#database-configuration)
4. [Cache Configuration](#cache-configuration)
5. [Authentication & Security](#authentication--security)
6. [Service Ports](#service-ports)
7. [AI & RAG Configuration](#ai--rag-configuration)
8. [Storage Configuration](#storage-configuration)
9. [Notification Configuration](#notification-configuration)
10. [Monitoring & Logging](#monitoring--logging)
11. [Sample Management](#sample-management)
12. [Sequencing Configuration](#sequencing-configuration)
13. [Template Management](#template-management)
14. [External Integrations](#external-integrations)
15. [Performance Settings](#performance-settings)
16. [Feature Flags](#feature-flags)
17. [Production-Only Settings](#production-only-settings)
18. [Security Best Practices](#security-best-practices)

---

## Quick Start

### Development Environment

```bash
# Copy the development environment file
cp config/development.env .env

# Or set key variables manually
export DATABASE_URL="postgres://postgres:postgres@postgres:5432/lab_manager"
export JWT_SECRET="dev-secret-key-change-in-production"
export RUST_LOG="debug"
```

### Production Environment

```bash
# Copy the production environment file
cp config/production.env .env

# Set sensitive variables via environment or secrets
export POSTGRES_PASSWORD="your-secure-password"
export JWT_SECRET="your-jwt-secret-key"
export REDIS_PASSWORD="your-redis-password"
```

---

## General Configuration

### Core Environment Variables

| Variable | Description | Default | Required | Example |
|----------|-------------|---------|----------|---------|
| `NODE_ENV` | Application environment | `development` | No | `production` |
| `RUST_LOG` | Rust logging level | `info` | No | `debug`, `info`, `warn`, `error` |
| `DEBUG_MODE` | Enable debug features | `false` | No | `true`, `false` |

### Usage Example

```bash
# Development
NODE_ENV=development
RUST_LOG=debug
DEBUG_MODE=true

# Production
NODE_ENV=production
RUST_LOG=info
DEBUG_MODE=false
```

---

## Database Configuration

### PostgreSQL Settings

| Variable | Description | Default | Required | Example |
|----------|-------------|---------|----------|---------|
| `DATABASE_URL` | Complete PostgreSQL connection string | - | Yes | `postgres://user:pass@host:5432/db` |
| `POSTGRES_HOST` | PostgreSQL server hostname | `localhost` | No | `postgres-primary` |
| `POSTGRES_PORT` | PostgreSQL server port | `5432` | No | `5432` |
| `POSTGRES_DB` | Database name | `lab_manager` | No | `tracseq_prod` |
| `POSTGRES_USER` | Database username | `postgres` | No | `tracseq_admin` |
| `POSTGRES_PASSWORD` | Database password | - | Yes | `secure-password` |

### Connection Pool Settings

| Variable | Description | Default | Required | Example |
|----------|-------------|---------|----------|---------|
| `DB_MAX_CONNECTIONS` | Maximum database connections | `50` | No | `100` |
| `DB_MIN_CONNECTIONS` | Minimum database connections | `10` | No | `20` |
| `DB_POOL_TIMEOUT` | Connection timeout in seconds | `30` | No | `60` |

### Usage Example

```bash
# Development
DATABASE_URL=postgres://postgres:postgres@postgres:5432/lab_manager
DB_MAX_CONNECTIONS=50
DB_MIN_CONNECTIONS=10

# Production
DATABASE_URL=postgres://tracseq_admin:${POSTGRES_PASSWORD}@postgres-primary:5432/tracseq_prod
DB_MAX_CONNECTIONS=100
DB_MIN_CONNECTIONS=20
```

---

## Cache Configuration

### Redis Settings

| Variable | Description | Default | Required | Example |
|----------|-------------|---------|----------|---------|
| `REDIS_URL` | Complete Redis connection string | - | Yes | `redis://redis:6379` |
| `REDIS_HOST` | Redis server hostname | `localhost` | No | `redis-primary` |
| `REDIS_PORT` | Redis server port | `6379` | No | `6379` |
| `REDIS_PASSWORD` | Redis password | - | No | `redis-password` |
| `REDIS_DB` | Redis database number | `0` | No | `0` |

### Usage Example

```bash
# Development (no password)
REDIS_URL=redis://redis:6379
REDIS_HOST=redis
REDIS_PORT=6379

# Production (with password)
REDIS_URL=redis://:${REDIS_PASSWORD}@redis-primary:6379
REDIS_PASSWORD=secure-redis-password
```

---

## Authentication & Security

### JWT Configuration

| Variable | Description | Default | Required | Example |
|----------|-------------|---------|----------|---------|
| `JWT_SECRET` | Secret key for JWT signing | - | Yes | `your-secret-key` |
| `JWT_EXPIRY_HOURS` | JWT token expiry time | `24` | No | `8` |
| `JWT_REFRESH_EXPIRY_DAYS` | Refresh token expiry | `7` | No | `1` |

### Security Settings

| Variable | Description | Default | Required | Example |
|----------|-------------|---------|----------|---------|
| `BCRYPT_COST` | Bcrypt hashing cost | `10` | No | `12` |
| `SESSION_TIMEOUT_HOURS` | Session timeout | `8` | No | `4` |
| `MAX_LOGIN_ATTEMPTS` | Maximum login attempts | `5` | No | `3` |
| `ENABLE_MULTI_TENANCY` | Enable multi-tenant support | `true` | No | `true`, `false` |

### CORS Settings

| Variable | Description | Default | Required | Example |
|----------|-------------|---------|----------|---------|
| `CORS_ORIGINS` | Allowed CORS origins | `*` | No | `https://app.example.com,https://api.example.com` |
| `CORS_CREDENTIALS` | Allow credentials in CORS | `true` | No | `true`, `false` |

### Usage Example

```bash
# Development
JWT_SECRET=dev-secret-key-change-in-production
JWT_EXPIRY_HOURS=24
BCRYPT_COST=10
CORS_ORIGINS=http://localhost:3000,http://localhost:5173

# Production
JWT_SECRET=${JWT_SECRET_FROM_VAULT}
JWT_EXPIRY_HOURS=8
BCRYPT_COST=12
CORS_ORIGINS=https://tracseq.yourdomain.com
```

---

## Service Ports

### Core Services

| Variable | Description | Default | Required | Example |
|----------|-------------|---------|----------|---------|
| `AUTH_SERVICE_PORT` | Authentication service port | `8080` | No | `8080` |
| `SAMPLE_SERVICE_PORT` | Sample management service port | `8081` | No | `8081` |
| `STORAGE_SERVICE_PORT` | Storage service port | `8082` | No | `8082` |
| `TEMPLATE_SERVICE_PORT` | Template service port | `8083` | No | `8083` |
| `SEQUENCING_SERVICE_PORT` | Sequencing service port | `8084` | No | `8084` |
| `NOTIFICATION_SERVICE_PORT` | Notification service port | `8085` | No | `8085` |
| `RAG_SERVICE_PORT` | RAG service port | `8086` | No | `8086` |
| `EVENT_SERVICE_PORT` | Event service port | `8087` | No | `8087` |
| `TRANSACTION_SERVICE_PORT` | Transaction service port | `8088` | No | `8088` |
| `API_GATEWAY_PORT` | API Gateway port | `8089` | No | `8089` |

### Frontend Services

| Variable | Description | Default | Required | Example |
|----------|-------------|---------|----------|---------|
| `FRONTEND_DEV_PORT` | Frontend development port | `5173` | No | `5173` |
| `FRONTEND_PORT` | Frontend production port | `3000` | No | `3000` |
| `BACKEND_DEV_PORT` | Backend development port | `3000` | No | `3000` |

---

## AI & RAG Configuration

### Ollama Configuration

| Variable | Description | Default | Required | Example |
|----------|-------------|---------|----------|---------|
| `USE_OLLAMA` | Enable Ollama integration | `true` | No | `true`, `false` |
| `OLLAMA_BASE_URL` | Ollama server URL | `http://ollama:11434` | No | `http://ollama:11434` |
| `OLLAMA_MODEL` | Ollama model name | `llama3.2:3b` | No | `llama3.2:3b` |
| `OLLAMA_PORT` | Ollama service port | `11434` | No | `11434` |

### LLM Settings

| Variable | Description | Default | Required | Example |
|----------|-------------|---------|----------|---------|
| `LLM_TEMPERATURE` | LLM temperature setting | `0.7` | No | `0.3` |
| `MAX_TOKENS` | Maximum tokens per request | `2048` | No | `2048` |
| `RAG_CHUNK_SIZE` | RAG document chunk size | `1000` | No | `1000` |
| `RAG_CHUNK_OVERLAP` | RAG chunk overlap | `200` | No | `200` |

### Vector Database

| Variable | Description | Default | Required | Example |
|----------|-------------|---------|----------|---------|
| `VECTOR_DB_URL` | Vector database URL | `http://chromadb:8000` | No | `http://chromadb:8000` |
| `CHROMA_PERSIST_DIRECTORY` | ChromaDB persistence directory | `/app/chroma_data` | No | `/app/chroma_data` |

### Usage Example

```bash
# Development
USE_OLLAMA=true
OLLAMA_BASE_URL=http://ollama:11434
OLLAMA_MODEL=llama3.2:3b
LLM_TEMPERATURE=0.7

# Production
USE_OLLAMA=true
OLLAMA_BASE_URL=http://ollama:11434
OLLAMA_MODEL=llama3.2:3b
LLM_TEMPERATURE=0.3
```

---

## Storage Configuration

### File Storage

| Variable | Description | Default | Required | Example |
|----------|-------------|---------|----------|---------|
| `STORAGE_PATH` | Main storage directory | `/app/storage` | No | `/app/storage` |
| `UPLOAD_PATH` | File upload directory | `/app/uploads` | No | `/app/uploads` |
| `MAX_FILE_SIZE` | Maximum file upload size | `100MB` | No | `50MB` |
| `ALLOWED_FILE_TYPES` | Allowed file extensions | `pdf,doc,docx,txt,csv,xlsx` | No | `pdf,doc,docx` |

### Storage Service Features

| Variable | Description | Default | Required | Example |
|----------|-------------|---------|----------|---------|
| `ENABLE_IOT_INTEGRATION` | Enable IoT sensor integration | `true` | No | `true`, `false` |
| `ENABLE_BLOCKCHAIN_AUDIT` | Enable blockchain audit trail | `false` | No | `true`, `false` |
| `ENABLE_ANALYTICS` | Enable storage analytics | `true` | No | `true`, `false` |
| `TEMP_CHECK_INTERVAL_MINUTES` | Temperature check interval | `5` | No | `1` |
| `ALERT_THRESHOLD_CELSIUS` | Temperature alert threshold | `2.0` | No | `1.0` |

### Usage Example

```bash
# Development
STORAGE_PATH=/app/storage
MAX_FILE_SIZE=100MB
ENABLE_IOT_INTEGRATION=true
ENABLE_BLOCKCHAIN_AUDIT=false

# Production
STORAGE_PATH=/app/storage
MAX_FILE_SIZE=50MB
ENABLE_IOT_INTEGRATION=true
ENABLE_BLOCKCHAIN_AUDIT=true
```

---

## Notification Configuration

### Email Settings

| Variable | Description | Default | Required | Example |
|----------|-------------|---------|----------|---------|
| `SMTP_HOST` | SMTP server hostname | - | Yes | `smtp.gmail.com` |
| `SMTP_PORT` | SMTP server port | `587` | No | `587`, `465`, `25` |
| `SMTP_USERNAME` | SMTP username | - | Yes | `your-email@gmail.com` |
| `SMTP_PASSWORD` | SMTP password | - | Yes | `your-app-password` |
| `SMTP_FROM` | From email address | - | Yes | `noreply@yourdomain.com` |
| `SMTP_SECURE` | Use TLS/SSL | `true` | No | `true`, `false` |

### Slack Integration

| Variable | Description | Default | Required | Example |
|----------|-------------|---------|----------|---------|
| `SLACK_BOT_TOKEN` | Slack bot token | - | No | `xoxb-your-token` |
| `SLACK_CHANNEL` | Default Slack channel | `#lab-notifications` | No | `#alerts` |
| `SLACK_WEBHOOK_URL` | Slack webhook URL | - | No | `https://hooks.slack.com/...` |

### Usage Example

```bash
# Development (using MailHog)
SMTP_HOST=mailhog
SMTP_PORT=1025
SMTP_SECURE=false

# Production
SMTP_HOST=smtp.gmail.com
SMTP_PORT=587
SMTP_USERNAME=your-email@gmail.com
SMTP_PASSWORD=${SMTP_PASSWORD}
SMTP_SECURE=true
```

---

## Monitoring & Logging

### Logging Configuration

| Variable | Description | Default | Required | Example |
|----------|-------------|---------|----------|---------|
| `LOG_LEVEL` | Logging level | `info` | No | `debug`, `info`, `warn`, `error` |
| `LOG_FORMAT` | Log format | `json` | No | `json`, `plain` |
| `LOG_FILE` | Log file path | `/app/logs/app.log` | No | `/app/logs/app.log` |
| `ENABLE_REQUEST_LOGGING` | Enable HTTP request logging | `true` | No | `true`, `false` |

### Metrics and Monitoring

| Variable | Description | Default | Required | Example |
|----------|-------------|---------|----------|---------|
| `METRICS_ENABLED` | Enable metrics collection | `true` | No | `true`, `false` |
| `METRICS_PORT` | Metrics endpoint port | `9090` | No | `9090` |
| `PROMETHEUS_URL` | Prometheus server URL | `http://prometheus:9090` | No | `http://prometheus:9090` |

### Health Checks

| Variable | Description | Default | Required | Example |
|----------|-------------|---------|----------|---------|
| `HEALTH_CHECK_ENABLED` | Enable health checks | `true` | No | `true`, `false` |
| `HEALTH_CHECK_INTERVAL` | Health check interval (seconds) | `30` | No | `30` |

### Usage Example

```bash
# Development
LOG_LEVEL=debug
LOG_FORMAT=json
ENABLE_REQUEST_LOGGING=true
METRICS_ENABLED=true

# Production
LOG_LEVEL=info
LOG_FORMAT=json
ENABLE_REQUEST_LOGGING=false
METRICS_ENABLED=true
```

---

## Sample Management

### Sample Service Settings

| Variable | Description | Default | Required | Example |
|----------|-------------|---------|----------|---------|
| `BARCODE_PREFIX` | Sample barcode prefix | `SMPL` | No | `PROD`, `DEV` |
| `AUTO_GENERATE_BARCODES` | Auto-generate sample barcodes | `true` | No | `true`, `false` |
| `MAX_BATCH_SIZE` | Maximum batch processing size | `1000` | No | `500` |
| `ENABLE_QC_VALIDATION` | Enable quality control validation | `true` | No | `true`, `false` |

### Sample Types

| Variable | Description | Default | Required | Example |
|----------|-------------|---------|----------|---------|
| `SUPPORTED_SAMPLE_TYPES` | Supported sample types | `DNA,RNA,Protein,Tissue` | No | `DNA,RNA` |
| `DEFAULT_SAMPLE_TYPE` | Default sample type | `DNA` | No | `DNA` |

### Usage Example

```bash
# Development
BARCODE_PREFIX=DEV
AUTO_GENERATE_BARCODES=true
MAX_BATCH_SIZE=1000

# Production
BARCODE_PREFIX=PROD
AUTO_GENERATE_BARCODES=true
MAX_BATCH_SIZE=500
```

---

## Sequencing Configuration

### Sequencing Platforms

| Variable | Description | Default | Required | Example |
|----------|-------------|---------|----------|---------|
| `SUPPORTED_PLATFORMS` | Supported sequencing platforms | `Illumina,PacBio,Oxford_Nanopore` | No | `Illumina` |
| `DEFAULT_PLATFORM` | Default sequencing platform | `Illumina` | No | `Illumina` |

### Sequencing Settings

| Variable | Description | Default | Required | Example |
|----------|-------------|---------|----------|---------|
| `MIN_COVERAGE` | Minimum sequencing coverage | `30` | No | `30` |
| `MAX_COVERAGE` | Maximum sequencing coverage | `100` | No | `100` |
| `DEFAULT_COVERAGE` | Default sequencing coverage | `50` | No | `50` |

### Usage Example

```bash
SUPPORTED_PLATFORMS=Illumina,PacBio,Oxford_Nanopore
DEFAULT_PLATFORM=Illumina
MIN_COVERAGE=30
MAX_COVERAGE=100
DEFAULT_COVERAGE=50
```

---

## Template Management

### Template Service

| Variable | Description | Default | Required | Example |
|----------|-------------|---------|----------|---------|
| `TEMPLATE_STORAGE_PATH` | Template storage directory | `/app/templates` | No | `/app/templates` |
| `ENABLE_TEMPLATE_VERSIONING` | Enable template versioning | `true` | No | `true`, `false` |
| `MAX_TEMPLATE_SIZE` | Maximum template file size | `10MB` | No | `5MB` |

### Usage Example

```bash
TEMPLATE_STORAGE_PATH=/app/templates
ENABLE_TEMPLATE_VERSIONING=true
MAX_TEMPLATE_SIZE=10MB
```

---

## External Integrations

### Cloud Storage (AWS S3)

| Variable | Description | Default | Required | Example |
|----------|-------------|---------|----------|---------|
| `AWS_ACCESS_KEY_ID` | AWS access key ID | - | No | `AKIAIOSFODNN7EXAMPLE` |
| `AWS_SECRET_ACCESS_KEY` | AWS secret access key | - | No | `wJalrXUtnFEMI/K7MDENG/bPxRfiCYEXAMPLEKEY` |
| `AWS_REGION` | AWS region | `us-east-1` | No | `us-west-2` |
| `AWS_S3_BUCKET` | S3 bucket name | - | No | `tracseq-storage` |

### Laboratory Equipment Integration

| Variable | Description | Default | Required | Example |
|----------|-------------|---------|----------|---------|
| `ENABLE_EQUIPMENT_INTEGRATION` | Enable equipment integration | `false` | No | `true`, `false` |
| `EQUIPMENT_API_ENDPOINT` | Equipment API endpoint | - | No | `https://lab-equipment.example.com/api` |

### ERP Integration

| Variable | Description | Default | Required | Example |
|----------|-------------|---------|----------|---------|
| `ENABLE_ERP_INTEGRATION` | Enable ERP integration | `false` | No | `true`, `false` |
| `ERP_API_ENDPOINT` | ERP API endpoint | - | No | `https://erp.example.com/api` |
| `ERP_API_KEY` | ERP API key | - | No | `your-erp-api-key` |

### Usage Example

```bash
# AWS S3 Integration
AWS_ACCESS_KEY_ID=${AWS_ACCESS_KEY_ID}
AWS_SECRET_ACCESS_KEY=${AWS_SECRET_ACCESS_KEY}
AWS_REGION=us-east-1
AWS_S3_BUCKET=tracseq-storage

# Equipment Integration
ENABLE_EQUIPMENT_INTEGRATION=true
EQUIPMENT_API_ENDPOINT=https://lab-equipment.example.com/api
```

---

## Performance Settings

### Connection Pools

| Variable | Description | Default | Required | Example |
|----------|-------------|---------|----------|---------|
| `MAX_CONCURRENT_CONNECTIONS` | Maximum concurrent connections | `100` | No | `200` |
| `CONNECTION_TIMEOUT` | Connection timeout (seconds) | `30` | No | `60` |
| `REQUEST_TIMEOUT` | Request timeout (seconds) | `60` | No | `120` |

### Caching

| Variable | Description | Default | Required | Example |
|----------|-------------|---------|----------|---------|
| `CACHE_TTL` | Cache time-to-live (seconds) | `3600` | No | `7200` |
| `ENABLE_QUERY_CACHE` | Enable query result caching | `true` | No | `true`, `false` |
| `ENABLE_RESPONSE_CACHE` | Enable response caching | `true` | No | `true`, `false` |

### Rate Limiting

| Variable | Description | Default | Required | Example |
|----------|-------------|---------|----------|---------|
| `RATE_LIMIT_ENABLED` | Enable rate limiting | `true` | No | `true`, `false` |
| `RATE_LIMIT_REQUESTS` | Requests per window | `100` | No | `1000` |
| `RATE_LIMIT_WINDOW` | Rate limit window (seconds) | `60` | No | `60` |

### Usage Example

```bash
# Development
MAX_CONCURRENT_CONNECTIONS=100
CONNECTION_TIMEOUT=30
CACHE_TTL=3600
RATE_LIMIT_REQUESTS=100

# Production
MAX_CONCURRENT_CONNECTIONS=200
CONNECTION_TIMEOUT=60
CACHE_TTL=7200
RATE_LIMIT_REQUESTS=1000
```

---

## Feature Flags

### Core Features

| Variable | Description | Default | Required | Example |
|----------|-------------|---------|----------|---------|
| `ENABLE_ADVANCED_ANALYTICS` | Enable advanced analytics | `true` | No | `true`, `false` |
| `ENABLE_ML_PREDICTIONS` | Enable ML predictions | `true` | No | `true`, `false` |
| `ENABLE_REAL_TIME_MONITORING` | Enable real-time monitoring | `true` | No | `true`, `false` |
| `ENABLE_AUDIT_LOGGING` | Enable audit logging | `true` | No | `true`, `false` |

### Beta Features

| Variable | Description | Default | Required | Example |
|----------|-------------|---------|----------|---------|
| `ENABLE_BETA_UI` | Enable beta UI features | `false` | No | `true`, `false` |
| `ENABLE_BETA_API` | Enable beta API features | `false` | No | `true`, `false` |

### Usage Example

```bash
# Development
ENABLE_ADVANCED_ANALYTICS=true
ENABLE_ML_PREDICTIONS=true
ENABLE_BETA_UI=true
ENABLE_BETA_API=true

# Production
ENABLE_ADVANCED_ANALYTICS=true
ENABLE_ML_PREDICTIONS=true
ENABLE_BETA_UI=false
ENABLE_BETA_API=false
```

---

## Production-Only Settings

### Backup & Recovery

| Variable | Description | Default | Required | Example |
|----------|-------------|---------|----------|---------|
| `ENABLE_AUTOMATED_BACKUPS` | Enable automated backups | `true` | No | `true`, `false` |
| `BACKUP_INTERVAL_HOURS` | Backup interval in hours | `6` | No | `6` |
| `BACKUP_RETENTION_DAYS` | Backup retention in days | `30` | No | `30` |
| `BACKUP_STORAGE_PATH` | Backup storage directory | `/app/backups` | No | `/app/backups` |

### SSL/TLS Configuration

| Variable | Description | Default | Required | Example |
|----------|-------------|---------|----------|---------|
| `ENABLE_SSL` | Enable SSL/TLS | `true` | No | `true`, `false` |
| `SSL_CERT_PATH` | SSL certificate path | `/app/ssl/cert.pem` | No | `/app/ssl/cert.pem` |
| `SSL_KEY_PATH` | SSL private key path | `/app/ssl/key.pem` | No | `/app/ssl/key.pem` |
| `FORCE_HTTPS` | Force HTTPS redirects | `true` | No | `true`, `false` |

### Compliance & Auditing

| Variable | Description | Default | Required | Example |
|----------|-------------|---------|----------|---------|
| `ENABLE_AUDIT_TRAIL` | Enable audit trail | `true` | No | `true`, `false` |
| `AUDIT_LOG_RETENTION_DAYS` | Audit log retention | `2555` | No | `2555` |
| `ENABLE_DATA_ENCRYPTION` | Enable data encryption | `true` | No | `true`, `false` |
| `ENABLE_FIELD_LEVEL_ENCRYPTION` | Enable field-level encryption | `true` | No | `true`, `false` |

### Regulatory Compliance

| Variable | Description | Default | Required | Example |
|----------|-------------|---------|----------|---------|
| `ENABLE_HIPAA_COMPLIANCE` | Enable HIPAA compliance | `false` | No | `true`, `false` |
| `ENABLE_GDPR_COMPLIANCE` | Enable GDPR compliance | `true` | No | `true`, `false` |
| `ENABLE_SOX_COMPLIANCE` | Enable SOX compliance | `false` | No | `true`, `false` |

---

## Security Best Practices

### Environment Variable Security

1. **Never commit sensitive values to version control**
2. **Use environment-specific files** (development.env, production.env)
3. **Use secrets management** in production (Docker secrets, Kubernetes secrets, HashiCorp Vault)
4. **Rotate secrets regularly**
5. **Use strong, unique passwords**
6. **Validate environment variables** on application startup

### Recommended Secrets Management

```bash
# Using Docker Secrets
docker secret create jwt_secret jwt_secret.txt
docker secret create postgres_password postgres_password.txt

# Using Kubernetes Secrets
kubectl create secret generic tracseq-secrets \
  --from-literal=jwt-secret=your-jwt-secret \
  --from-literal=postgres-password=your-postgres-password

# Using HashiCorp Vault
vault kv put secret/tracseq \
  jwt_secret=your-jwt-secret \
  postgres_password=your-postgres-password
```

### Environment Variable Validation

```rust
// Example Rust validation
use std::env;

fn validate_env_vars() -> Result<(), String> {
    let required_vars = vec![
        "DATABASE_URL",
        "JWT_SECRET",
        "REDIS_URL",
    ];
    
    for var in required_vars {
        env::var(var).map_err(|_| format!("Missing required environment variable: {}", var))?;
    }
    
    Ok(())
}
```

### Development vs Production Differences

| Setting | Development | Production |
|---------|-------------|------------|
| `RUST_LOG` | `debug` | `info` |
| `DEBUG_MODE` | `true` | `false` |
| `BCRYPT_COST` | `10` | `12` |
| `JWT_EXPIRY_HOURS` | `24` | `8` |
| `SESSION_TIMEOUT_HOURS` | `8` | `4` |
| `MAX_LOGIN_ATTEMPTS` | `5` | `3` |
| `ENABLE_REQUEST_LOGGING` | `true` | `false` |
| `ENABLE_BETA_FEATURES` | `true` | `false` |

---

## Troubleshooting

### Common Issues

1. **Database Connection Errors**
   - Check `DATABASE_URL` format
   - Verify database is running
   - Check network connectivity

2. **Authentication Failures**
   - Verify `JWT_SECRET` is set
   - Check token expiry settings
   - Validate CORS configuration

3. **Service Discovery Issues**
   - Check service port configurations
   - Verify Docker network settings
   - Validate service URLs

4. **Performance Issues**
   - Adjust connection pool settings
   - Tune cache configurations
   - Review rate limiting settings

### Environment Variable Debugging

```bash
# List all environment variables
printenv | grep -E "(DATABASE|JWT|REDIS|SMTP)"

# Test specific variable
echo $DATABASE_URL

# Validate database connection
psql $DATABASE_URL -c "SELECT version();"

# Test Redis connection
redis-cli -u $REDIS_URL ping
```

---

## Migration Guide

### Upgrading from Previous Versions

1. **Review new environment variables**
2. **Update configuration files**
3. **Test in development environment**
4. **Migrate production settings**
5. **Validate all services**

### Configuration Migration Script

```bash
#!/bin/bash
# migrate-config.sh

# Backup existing configuration
cp .env .env.backup

# Update environment variables
sed -i 's/OLD_VAR_NAME/NEW_VAR_NAME/g' .env

# Validate configuration
./scripts/validate-config.sh

echo "Configuration migration completed"
```

---

This documentation should be kept up-to-date as new environment variables are added or existing ones are modified. For the most current information, always refer to the actual configuration files in the `config/` directory. 