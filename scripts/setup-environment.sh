#!/bin/bash
set -euo pipefail

# TracSeq 2.0 Environment Setup Script
# Creates production and staging environment configurations

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Configuration
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(dirname "$SCRIPT_DIR")"
DEPLOY_DIR="$PROJECT_ROOT/deploy"
CONFIGS_DIR="$DEPLOY_DIR/configs"

# Utility functions
log() {
    echo -e "${GREEN}[$(date '+%Y-%m-%d %H:%M:%S')] $1${NC}"
}

warn() {
    echo -e "${YELLOW}[$(date '+%Y-%m-%d %H:%M:%S')] WARNING: $1${NC}"
}

error() {
    echo -e "${RED}[$(date '+%Y-%m-%d %H:%M:%S')] ERROR: $1${NC}"
}

info() {
    echo -e "${BLUE}[$(date '+%Y-%m-%d %H:%M:%S')] INFO: $1${NC}"
}

# Generate secure random string
generate_secret() {
    local length=${1:-32}
    openssl rand -hex "$length" 2>/dev/null || head -c "$length" /dev/urandom | xxd -p -c "$length"
}

# Create production environment file
create_production_env() {
    local env_file="$DEPLOY_DIR/.env.production"
    
    log "Creating production environment configuration..."
    
    # Generate secure secrets
    local postgres_password=$(generate_secret 32)
    local jwt_secret=$(generate_secret 32)
    local redis_password=$(generate_secret 16)
    local grafana_password=$(generate_secret 16)
    local backup_encryption_key=$(generate_secret 32)
    
    cat > "$env_file" << EOF
# ================================
# TracSeq 2.0 Production Environment Configuration
# Generated: $(date -u +%Y-%m-%dT%H:%M:%SZ)
# ================================

# ================================
# DATABASE CONFIGURATION
# ================================
POSTGRES_PASSWORD=$postgres_password
POSTGRES_HOST=postgres-primary
POSTGRES_PORT=5432
POSTGRES_DB=tracseq_prod
POSTGRES_USER=tracseq_admin
DB_MAX_CONNECTIONS=100
DB_MIN_CONNECTIONS=10

# PostgreSQL Performance Tuning
POSTGRES_SHARED_BUFFERS=512MB
POSTGRES_EFFECTIVE_CACHE_SIZE=2GB
POSTGRES_MAINTENANCE_WORK_MEM=128MB
POSTGRES_WAL_BUFFERS=32MB

# ================================
# SECURITY CONFIGURATION
# ================================
JWT_SECRET_KEY=$jwt_secret
JWT_ALGORITHM=HS256
JWT_EXPIRATION_HOURS=24
BCRYPT_COST=12

# Session Configuration
SESSION_TIMEOUT_HOURS=8
MAX_LOGIN_ATTEMPTS=5
LOCKOUT_DURATION_MINUTES=30

# ================================
# REDIS CONFIGURATION
# ================================
REDIS_HOST=redis-primary
REDIS_PORT=6379
REDIS_PASSWORD=$redis_password
REDIS_MAXMEMORY=2gb
REDIS_MAXMEMORY_POLICY=allkeys-lru

# ================================
# EXTERNAL INTEGRATIONS
# ================================

# Email Configuration (CONFIGURE THESE)
SMTP_HOST=smtp.gmail.com
SMTP_PORT=587
SMTP_USERNAME=your-email@gmail.com
SMTP_PASSWORD=your-app-password
SMTP_TLS=true
SMTP_STARTTLS=true
EMAIL_FROM=noreply@yourdomain.com
EMAIL_REPLY_TO=support@yourdomain.com

# SMS Configuration (CONFIGURE THESE)
SMS_PROVIDER=twilio
SMS_API_KEY=your-twilio-api-key
SMS_API_SECRET=your-twilio-api-secret
SMS_FROM=+1234567890

# Slack Integration (CONFIGURE THESE)
SLACK_BOT_TOKEN=xoxb-your-slack-bot-token
SLACK_WEBHOOK_URL=https://hooks.slack.com/services/YOUR/SLACK/WEBHOOK
SLACK_CHANNEL_ALERTS=#tracseq-alerts
SLACK_CHANNEL_NOTIFICATIONS=#tracseq-notifications

# Microsoft Teams Integration (CONFIGURE THESE)
TEAMS_WEBHOOK_URL=https://outlook.office.com/webhook/YOUR-TEAMS-WEBHOOK

# ================================
# AI SERVICES CONFIGURATION
# ================================

# OpenAI Configuration (CONFIGURE THESE)
OPENAI_API_KEY=sk-your-openai-api-key
OPENAI_MODEL=gpt-4
OPENAI_MAX_TOKENS=2000
OPENAI_TEMPERATURE=0.1

# Anthropic Claude Configuration (CONFIGURE THESE)
ANTHROPIC_API_KEY=sk-ant-your-anthropic-key
ANTHROPIC_MODEL=claude-3-sonnet-20240229
ANTHROPIC_MAX_TOKENS=2000

# Vector Database
CHROMA_HOST=chroma
CHROMA_PORT=8000

# ================================
# MONITORING CONFIGURATION
# ================================

# Grafana
GRAFANA_PASSWORD=$grafana_password

# Prometheus
PROMETHEUS_RETENTION_DAYS=30

# Logging
LOG_LEVEL=info
LOG_FORMAT=json

# ================================
# PERFORMANCE TUNING
# ================================

# Rate Limiting
API_RATE_LIMIT_PER_IP=1000
API_RATE_LIMIT_WINDOW_MINUTES=1
BULK_OPERATION_MAX_SIZE=1000

# ================================
# BUSINESS CONFIGURATION
# ================================

# Sample Management
BARCODE_PREFIX=SMPL
AUTO_GENERATE_BARCODES=true
ENABLE_BATCH_OPERATIONS=true
MAX_BATCH_SIZE=1000

# Template Management
MAX_UPLOAD_SIZE_MB=50
ALLOWED_EXTENSIONS=xlsx,xls,csv,json,txt
ENABLE_VERSIONING=true
ENABLE_APPROVAL_WORKFLOW=true
AUTO_BACKUP=true

# Sequencing Configuration
ENABLE_ILLUMINA=true
ENABLE_NANOPORE=true
ENABLE_PACBIO=false
MAX_CONCURRENT_JOBS=10
JOB_TIMEOUT_HOURS=48
AUTO_CLEANUP_DAYS=30

# Quality Control
ENABLE_QC_VALIDATION=true
AUTO_QC_THRESHOLD=0.95
QC_THRESHOLD_Q30=85
QC_THRESHOLD_YIELD=80

# ================================
# SECURITY FEATURES
# ================================

# Multi-tenancy
ENABLE_MULTI_TENANCY=true
DEFAULT_TENANT=tracseq

# Audit Logging
ENABLE_AUDIT_LOGGING=true
AUDIT_LOG_RETENTION_DAYS=2555

# API Security
ENABLE_CORS=true
CORS_ALLOWED_ORIGINS=https://yourdomain.com,https://app.yourdomain.com
ENABLE_CSRF_PROTECTION=true

# ================================
# BACKUP CONFIGURATION
# ================================

# Database Backup
DB_BACKUP_ENABLED=true
DB_BACKUP_SCHEDULE=0 2 * * *
DB_BACKUP_RETENTION_DAYS=30
DB_BACKUP_ENCRYPTION_KEY=$backup_encryption_key

# ================================
# FEATURE FLAGS
# ================================

# Experimental Features
ENABLE_BLOCKCHAIN_AUDIT=false
ENABLE_ADVANCED_AI=true
ENABLE_PREDICTIVE_ANALYTICS=true
ENABLE_AUTOMATED_WORKFLOWS=true

# ================================
# DOMAIN CONFIGURATION
# ================================

# Domain Configuration (CONFIGURE THESE)
DOMAIN_NAME=yourdomain.com
API_DOMAIN=api.yourdomain.com
APP_DOMAIN=app.yourdomain.com

# ================================
# ENVIRONMENT METADATA
# ================================
ENVIRONMENT=production
DEPLOY_DATE=$(date -u +%Y-%m-%dT%H:%M:%SZ)
VERSION=1.0.0

# ================================
# CONTACT INFORMATION
# ================================
ADMIN_EMAIL=admin@yourdomain.com
SUPPORT_EMAIL=support@yourdomain.com
EMERGENCY_PHONE=+1-XXX-XXX-XXXX
EOF

    log "✅ Production environment file created: $env_file"
    warn "Please update the placeholder values in $env_file"
}

# Create staging environment file
create_staging_env() {
    local env_file="$DEPLOY_DIR/.env.staging"
    
    log "Creating staging environment configuration..."
    
    # Generate test secrets (simpler for staging)
    local postgres_password="staging_$(generate_secret 8)"
    local jwt_secret="staging_$(generate_secret 16)"
    
    cat > "$env_file" << EOF
# ================================
# TracSeq 2.0 Staging Environment Configuration
# Generated: $(date -u +%Y-%m-%dT%H:%M:%SZ)
# ================================

# ================================
# DATABASE CONFIGURATION
# ================================
POSTGRES_STAGING_PASSWORD=$postgres_password
POSTGRES_HOST=postgres-staging
POSTGRES_PORT=5432
POSTGRES_DB=tracseq_staging
POSTGRES_USER=tracseq_staging
DB_MAX_CONNECTIONS=50
DB_MIN_CONNECTIONS=5

# ================================
# SECURITY CONFIGURATION (RELAXED FOR TESTING)
# ================================
JWT_SECRET_STAGING=$jwt_secret
JWT_ALGORITHM=HS256
JWT_EXPIRATION_HOURS=2
BCRYPT_COST=8

# Session Configuration
SESSION_TIMEOUT_HOURS=2
MAX_LOGIN_ATTEMPTS=10
LOCKOUT_DURATION_MINUTES=5

# ================================
# REDIS CONFIGURATION
# ================================
REDIS_HOST=redis-staging
REDIS_PORT=6379
REDIS_MAXMEMORY=256mb

# ================================
# EXTERNAL INTEGRATIONS (MOCK/TEST)
# ================================

# Email Configuration (MailHog for testing)
SMTP_HOST=mailhog
SMTP_PORT=1025
SMTP_USERNAME=test
SMTP_PASSWORD=test
SMTP_TLS=false
EMAIL_FROM=staging@tracseq.local

# SMS Configuration (Mock)
SMS_PROVIDER=mock
SMS_API_KEY=mock-api-key
SMS_FROM=+1234567890

# AI Services (Mock or limited)
OPENAI_API_KEY=mock-openai-key
ANTHROPIC_API_KEY=mock-anthropic-key

# ================================
# MONITORING CONFIGURATION
# ================================

# Grafana
GRAFANA_PASSWORD=staging123

# Logging
LOG_LEVEL=debug
LOG_FORMAT=pretty

# ================================
# BUSINESS CONFIGURATION (TESTING)
# ================================

# Sample Management
BARCODE_PREFIX=TEST
AUTO_GENERATE_BARCODES=true
ENABLE_BATCH_OPERATIONS=false
MAX_BATCH_SIZE=100

# Template Management
MAX_UPLOAD_SIZE_MB=10
ALLOWED_EXTENSIONS=xlsx,xls,csv,json
ENABLE_VERSIONING=false
ENABLE_APPROVAL_WORKFLOW=false
AUTO_BACKUP=false

# Sequencing Configuration
ENABLE_ILLUMINA=true
ENABLE_NANOPORE=false
ENABLE_PACBIO=false
MAX_CONCURRENT_JOBS=2
JOB_TIMEOUT_HOURS=1
AUTO_CLEANUP_DAYS=7

# Mock sequencing for testing
ENABLE_MOCK_SEQUENCING=true
MOCK_JOB_DURATION_SECONDS=30

# ================================
# TESTING FLAGS
# ================================

# Enable test features
ENABLE_TEST_ENDPOINTS=true
ENABLE_MOCK_NOTIFICATIONS=true
ENABLE_MOCK_TRANSACTIONS=true
ENABLE_TRANSACTION_LOGGING=true

# ================================
# ENVIRONMENT METADATA
# ================================
ENVIRONMENT=staging
DEPLOY_DATE=$(date -u +%Y-%m-%dT%H:%M:%SZ)
VERSION=1.0.0-staging

# ================================
# TEST CONFIGURATION
# ================================
TEST_ADMIN_EMAIL=admin@test.local
TEST_ADMIN_PASSWORD=test123
TEST_DATABASE_URL=postgresql://tracseq_staging:$postgres_password@postgres-staging:5432/tracseq_staging
EOF

    log "✅ Staging environment file created: $env_file"
}

# Create nginx configuration
create_nginx_config() {
    local nginx_config="$CONFIGS_DIR/nginx.conf"
    
    mkdir -p "$CONFIGS_DIR"
    
    log "Creating nginx configuration..."
    
    cat > "$nginx_config" << 'EOF'
# TracSeq 2.0 Production Nginx Configuration

upstream tracseq_api {
    least_conn;
    server api-gateway:8089 max_fails=3 fail_timeout=30s;
}

upstream tracseq_frontend {
    least_conn;
    server lab-manager-frontend:3000 max_fails=3 fail_timeout=30s;
}

# Rate limiting zones
limit_req_zone $binary_remote_addr zone=api:10m rate=10r/s;
limit_req_zone $binary_remote_addr zone=login:10m rate=5r/m;
limit_req_zone $binary_remote_addr zone=upload:10m rate=2r/m;

# Main server block
server {
    listen 80;
    server_name your-domain.com;
    
    # Security headers
    add_header X-Frame-Options DENY;
    add_header X-Content-Type-Options nosniff;
    add_header X-XSS-Protection "1; mode=block";
    add_header Referrer-Policy "strict-origin-when-cross-origin";
    add_header Content-Security-Policy "default-src 'self'; script-src 'self' 'unsafe-inline'; style-src 'self' 'unsafe-inline';";
    
    # Remove server signature
    server_tokens off;
    
    # Gzip compression
    gzip on;
    gzip_vary on;
    gzip_min_length 1024;
    gzip_types text/plain text/css text/xml text/javascript application/javascript application/json application/xml+rss;
    
    # API routes
    location /api/ {
        limit_req zone=api burst=20 nodelay;
        
        proxy_pass http://tracseq_api;
        proxy_set_header Host $host;
        proxy_set_header X-Real-IP $remote_addr;
        proxy_set_header X-Forwarded-For $proxy_add_x_forwarded_for;
        proxy_set_header X-Forwarded-Proto $scheme;
        
        # Timeouts
        proxy_connect_timeout 30s;
        proxy_send_timeout 30s;
        proxy_read_timeout 30s;
        
        # Buffer settings
        proxy_buffering on;
        proxy_buffer_size 4k;
        proxy_buffers 8 4k;
    }
    
    # Login endpoint with stricter rate limiting
    location /api/auth/login {
        limit_req zone=login burst=5 nodelay;
        
        proxy_pass http://tracseq_api;
        proxy_set_header Host $host;
        proxy_set_header X-Real-IP $remote_addr;
        proxy_set_header X-Forwarded-For $proxy_add_x_forwarded_for;
        proxy_set_header X-Forwarded-Proto $scheme;
    }
    
    # Upload endpoints with file size limits
    location /api/templates/upload {
        limit_req zone=upload burst=2 nodelay;
        
        client_max_body_size 50M;
        proxy_pass http://tracseq_api;
        proxy_set_header Host $host;
        proxy_set_header X-Real-IP $remote_addr;
        proxy_set_header X-Forwarded-For $proxy_add_x_forwarded_for;
        proxy_set_header X-Forwarded-Proto $scheme;
        
        # Extended timeouts for uploads
        proxy_connect_timeout 60s;
        proxy_send_timeout 60s;
        proxy_read_timeout 60s;
    }
    
    # Frontend application
    location / {
        proxy_pass http://tracseq_frontend;
        proxy_set_header Host $host;
        proxy_set_header X-Real-IP $remote_addr;
        proxy_set_header X-Forwarded-For $proxy_add_x_forwarded_for;
        proxy_set_header X-Forwarded-Proto $scheme;
        
        # WebSocket support
        proxy_http_version 1.1;
        proxy_set_header Upgrade $http_upgrade;
        proxy_set_header Connection "upgrade";
    }
    
    # Health check endpoint (no rate limiting)
    location /health {
        proxy_pass http://tracseq_api/api/health;
        access_log off;
    }
    
    # Static files caching
    location ~* \.(js|css|png|jpg|jpeg|gif|ico|svg|woff|woff2|ttf|eot)$ {
        expires 1y;
        add_header Cache-Control "public, immutable";
        access_log off;
    }
    
    # Block sensitive files
    location ~ /\. {
        deny all;
        access_log off;
        log_not_found off;
    }
    
    location ~ \.(env|log|conf)$ {
        deny all;
        access_log off;
        log_not_found off;
    }
}

# SSL configuration (when ready)
# server {
#     listen 443 ssl http2;
#     server_name your-domain.com;
#     
#     ssl_certificate /etc/ssl/certs/your-domain.com.crt;
#     ssl_certificate_key /etc/ssl/private/your-domain.com.key;
#     
#     ssl_protocols TLSv1.2 TLSv1.3;
#     ssl_ciphers ECDHE-RSA-AES128-GCM-SHA256:ECDHE-RSA-AES256-GCM-SHA384;
#     ssl_prefer_server_ciphers off;
#     
#     # HSTS
#     add_header Strict-Transport-Security "max-age=31536000; includeSubDomains" always;
#     
#     # Same location blocks as HTTP version
# }
EOF

    log "✅ Nginx configuration created: $nginx_config"
}

# Create Prometheus configuration
create_prometheus_config() {
    local prometheus_config="$CONFIGS_DIR/prometheus.yml"
    
    mkdir -p "$CONFIGS_DIR"
    
    log "Creating Prometheus configuration..."
    
    cat > "$prometheus_config" << 'EOF'
# TracSeq 2.0 Prometheus Configuration

global:
  scrape_interval: 15s
  evaluation_interval: 15s

rule_files:
  - "rules/*.yml"

scrape_configs:
  # TracSeq Microservices
  - job_name: 'tracseq-auth-service'
    static_configs:
      - targets: ['auth-service:8080']
    metrics_path: '/metrics'
    scrape_interval: 30s
    scrape_timeout: 10s

  - job_name: 'tracseq-sample-service'
    static_configs:
      - targets: ['sample-service:8081']
    metrics_path: '/metrics'
    scrape_interval: 30s

  - job_name: 'tracseq-template-service'
    static_configs:
      - targets: ['template-service:8083']
    metrics_path: '/metrics'
    scrape_interval: 30s

  - job_name: 'tracseq-sequencing-service'
    static_configs:
      - targets: ['sequencing-service:8084']
    metrics_path: '/metrics'
    scrape_interval: 30s

  - job_name: 'tracseq-notification-service'
    static_configs:
      - targets: ['notification-service:8085']
    metrics_path: '/metrics'
    scrape_interval: 30s

  - job_name: 'tracseq-api-gateway'
    static_configs:
      - targets: ['api-gateway:8089']
    metrics_path: '/metrics'
    scrape_interval: 30s

  - job_name: 'tracseq-transaction-service'
    static_configs:
      - targets: ['transaction-service:8088']
    metrics_path: '/metrics'
    scrape_interval: 30s

  # Infrastructure Services
  - job_name: 'postgres'
    static_configs:
      - targets: ['postgres-exporter:9187']
    scrape_interval: 30s

  - job_name: 'redis'
    static_configs:
      - targets: ['redis-exporter:9121']
    scrape_interval: 30s

  - job_name: 'nginx'
    static_configs:
      - targets: ['nginx-exporter:9113']
    scrape_interval: 30s

  # System Metrics
  - job_name: 'node'
    static_configs:
      - targets: ['node-exporter:9100']
    scrape_interval: 30s

alerting:
  alertmanagers:
    - static_configs:
        - targets:
          - alertmanager:9093
EOF

    log "✅ Prometheus configuration created: $prometheus_config"
}

# Create health check script
create_health_check_script() {
    local health_script="$SCRIPT_DIR/comprehensive-health-check.sh"
    
    log "Creating comprehensive health check script..."
    
    cat > "$health_script" << 'EOF'
#!/bin/bash

# TracSeq 2.0 Comprehensive Health Check Script

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m'

# Service configurations
SERVICES=(
    "auth-service:8080"
    "sample-service:8081"
    "template-service:8083"
    "sequencing-service:8084"
    "notification-service:8085"
    "api-gateway:8089"
    "transaction-service:8088"
)

DATABASE_SERVICES=(
    "postgres-primary:5432"
    "redis-primary:6379"
)

MONITORING_SERVICES=(
    "prometheus:9090"
    "grafana:3001"
)

check_service() {
    local service=$1
    local port=$2
    local timeout=${3:-10}
    
    echo -n "Checking $service..."
    
    if timeout "$timeout" bash -c "</dev/tcp/localhost/$port" 2>/dev/null; then
        if curl -f -s "http://localhost:$port/health" >/dev/null 2>&1; then
            echo -e " ${GREEN}✅ HEALTHY${NC}"
            return 0
        else
            echo -e " ${YELLOW}⚠️  PORT OPEN, NO HEALTH ENDPOINT${NC}"
            return 1
        fi
    else
        echo -e " ${RED}❌ UNREACHABLE${NC}"
        return 1
    fi
}

check_database() {
    local service=$1
    local port=$2
    
    echo -n "Checking $service..."
    
    if timeout 5 bash -c "</dev/tcp/localhost/$port" 2>/dev/null; then
        echo -e " ${GREEN}✅ REACHABLE${NC}"
        return 0
    else
        echo -e " ${RED}❌ UNREACHABLE${NC}"
        return 1
    fi
}

main() {
    echo "🏥 TracSeq 2.0 Comprehensive Health Check"
    echo "================================================"
    echo "Started at: $(date)"
    echo
    
    # Check core services
    echo "🔧 Core Services:"
    local core_failed=0
    for service_port in "${SERVICES[@]}"; do
        IFS=':' read -r service port <<< "$service_port"
        if ! check_service "$service" "$port"; then
            ((core_failed++))
        fi
    done
    
    echo
    
    # Check database services
    echo "🗄️  Database Services:"
    local db_failed=0
    for service_port in "${DATABASE_SERVICES[@]}"; do
        IFS=':' read -r service port <<< "$service_port"
        if ! check_database "$service" "$port"; then
            ((db_failed++))
        fi
    done
    
    echo
    
    # Check monitoring services
    echo "📊 Monitoring Services:"
    local monitoring_failed=0
    for service_port in "${MONITORING_SERVICES[@]}"; do
        IFS=':' read -r service port <<< "$service_port"
        if ! check_service "$service" "$port" 15; then
            ((monitoring_failed++))
        fi
    done
    
    echo
    echo "================================================"
    echo "📋 SUMMARY:"
    echo "Core Services: $((${#SERVICES[@]} - core_failed))/${#SERVICES[@]} healthy"
    echo "Database Services: $((${#DATABASE_SERVICES[@]} - db_failed))/${#DATABASE_SERVICES[@]} reachable"
    echo "Monitoring Services: $((${#MONITORING_SERVICES[@]} - monitoring_failed))/${#MONITORING_SERVICES[@]} healthy"
    
    local total_failed=$((core_failed + db_failed + monitoring_failed))
    
    if [ $total_failed -eq 0 ]; then
        echo -e "\n${GREEN}🎉 ALL SYSTEMS OPERATIONAL${NC}"
        exit 0
    elif [ $core_failed -eq 0 ]; then
        echo -e "\n${YELLOW}⚠️  CORE SYSTEMS HEALTHY, SOME AUXILIARY SERVICES DOWN${NC}"
        exit 1
    else
        echo -e "\n${RED}🚨 CRITICAL SERVICES DOWN${NC}"
        exit 2
    fi
}

main "$@"
EOF

    chmod +x "$health_script"
    log "✅ Health check script created: $health_script"
}

# Main function
main() {
    log "🔧 Setting up TracSeq 2.0 deployment environments"
    
    # Create directories
    mkdir -p "$DEPLOY_DIR" "$CONFIGS_DIR"
    
    # Create environment files
    create_production_env
    create_staging_env
    
    # Create configuration files
    create_nginx_config
    create_prometheus_config
    
    # Create utility scripts
    create_health_check_script
    
    log "✅ Environment setup completed!"
    log ""
    log "📋 Next steps:"
    log "1. Review and update $DEPLOY_DIR/.env.production"
    log "2. Configure external services (SMTP, SMS, Slack, AI APIs)"
    log "3. Update domain names in nginx configuration"
    log "4. Run: ./scripts/deploy-production.sh"
    log ""
    log "🧪 For staging deployment:"
    log "1. docker-compose -f deploy/staging/docker-compose.staging.yml up -d"
    log "2. Run: ./scripts/comprehensive-health-check.sh"
    log ""
    log "📊 Monitor deployment:"
    log "• Grafana: http://localhost:3001"
    log "• Prometheus: http://localhost:9090"
    log "• MailHog (staging): http://localhost:8025"
}

# Run main function
main "$@" 
