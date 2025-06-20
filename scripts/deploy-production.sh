#!/bin/bash
set -euo pipefail

# TracSeq 2.0 Production Deployment Script
# Comprehensive automated deployment with safety checks and rollback capability

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Configuration
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(dirname "$SCRIPT_DIR")"
COMPOSE_FILE="$PROJECT_ROOT/deploy/production/docker-compose.production.yml"
ENV_FILE="$PROJECT_ROOT/deploy/.env.production"
BACKUP_DIR="$PROJECT_ROOT/backups/$(date +%Y%m%d_%H%M%S)"
LOG_FILE="$PROJECT_ROOT/logs/deployment-$(date +%Y%m%d_%H%M%S).log"

# Deployment configuration
PRODUCTION_READY_SERVICES=(
    "postgres-primary"
    "redis-primary"
    "auth-service"
    "sample-service"
    "template-service"
    "notification-service"
    "sequencing-service"
    "transaction-service"
)

PHASE2_SERVICES=(
    "api-gateway"
    "rag-service"
    "chroma"
)

MONITORING_SERVICES=(
    "prometheus"
    "grafana"
    "jaeger"
    "loki"
)

# Utility functions
log() {
    echo -e "${GREEN}[$(date '+%Y-%m-%d %H:%M:%S')] $1${NC}" | tee -a "$LOG_FILE"
}

warn() {
    echo -e "${YELLOW}[$(date '+%Y-%m-%d %H:%M:%S')] WARNING: $1${NC}" | tee -a "$LOG_FILE"
}

error() {
    echo -e "${RED}[$(date '+%Y-%m-%d %H:%M:%S')] ERROR: $1${NC}" | tee -a "$LOG_FILE"
}

info() {
    echo -e "${BLUE}[$(date '+%Y-%m-%d %H:%M:%S')] INFO: $1${NC}" | tee -a "$LOG_FILE"
}

# Check if running as root
check_permissions() {
    if [[ $EUID -eq 0 ]]; then
        warn "Running as root. Consider using a non-root user with docker permissions."
    fi
}

# Pre-deployment checks
pre_deployment_checks() {
    log "🔍 Running pre-deployment checks..."
    
    # Check if Docker is running
    if ! docker info >/dev/null 2>&1; then
        error "Docker is not running. Please start Docker first."
        exit 1
    fi
    
    # Check if docker-compose is available
    if ! command -v docker-compose >/dev/null 2>&1; then
        error "docker-compose is not installed."
        exit 1
    fi
    
    # Check if required files exist
    if [[ ! -f "$COMPOSE_FILE" ]]; then
        error "Docker compose file not found: $COMPOSE_FILE"
        exit 1
    fi
    
    if [[ ! -f "$ENV_FILE" ]]; then
        error "Environment file not found: $ENV_FILE"
        exit 1
    fi
    
    # Check available disk space (require at least 10GB)
    available_space=$(df / | tail -1 | awk '{print $4}')
    required_space=$((10 * 1024 * 1024)) # 10GB in KB
    
    if [[ $available_space -lt $required_space ]]; then
        error "Insufficient disk space. Required: 10GB, Available: $((available_space / 1024 / 1024))GB"
        exit 1
    fi
    
    # Validate environment file
    if ! grep -q "POSTGRES_PASSWORD" "$ENV_FILE"; then
        error "POSTGRES_PASSWORD not found in environment file"
        exit 1
    fi
    
    if ! grep -q "JWT_SECRET_KEY" "$ENV_FILE"; then
        error "JWT_SECRET_KEY not found in environment file"
        exit 1
    fi
    
    log "✅ Pre-deployment checks passed"
}

# Create backup
create_backup() {
    log "💾 Creating backup..."
    mkdir -p "$BACKUP_DIR"
    mkdir -p "$(dirname "$LOG_FILE")"
    
    # Check if PostgreSQL is running
    if docker ps --format "table {{.Names}}" | grep -q "tracseq-postgres-primary"; then
        log "Creating database backup..."
        if docker exec tracseq-postgres-primary pg_dump -U tracseq_admin tracseq_prod > "$BACKUP_DIR/database.sql" 2>/dev/null; then
            log "✅ Database backup created: $BACKUP_DIR/database.sql"
        else
            warn "Failed to create database backup (database might not exist yet)"
        fi
    else
        info "PostgreSQL not running, skipping database backup"
    fi
    
    # Backup configuration files
    cp "$ENV_FILE" "$BACKUP_DIR/env.backup"
    cp "$COMPOSE_FILE" "$BACKUP_DIR/compose.backup"
    
    log "✅ Backup completed: $BACKUP_DIR"
}

# Wait for service health
wait_for_service_health() {
    local service=$1
    local port=$2
    local max_attempts=${3:-30}
    local attempt=1
    
    log "⏳ Waiting for $service to be healthy (port $port)..."
    
    while [[ $attempt -le $max_attempts ]]; do
        if curl -f -s "http://localhost:$port/health" >/dev/null 2>&1; then
            log "✅ $service is healthy"
            return 0
        fi
        
        info "Attempt $attempt/$max_attempts: $service not ready yet..."
        sleep 10
        ((attempt++))
    done
    
    error "$service failed to become healthy after $max_attempts attempts"
    return 1
}

# Deploy services
deploy_services() {
    local services=("$@")
    log "📦 Deploying services: ${services[*]}"
    
    for service in "${services[@]}"; do
        info "Starting $service..."
        if docker-compose -f "$COMPOSE_FILE" --env-file "$ENV_FILE" up -d "$service"; then
            log "✅ $service started successfully"
        else
            error "Failed to start $service"
            return 1
        fi
    done
}

# Health check for all services
comprehensive_health_check() {
    log "🏥 Running comprehensive health checks..."
    
    local services=(
        "auth-service:8080"
        "sample-service:8081"
        "template-service:8083"
        "sequencing-service:8084"
        "notification-service:8085"
    )
    
    local failed_services=()
    
    for service_port in "${services[@]}"; do
        IFS=':' read -r service port <<< "$service_port"
        
        if ! wait_for_service_health "$service" "$port" 10; then
            failed_services+=("$service")
        fi
    done
    
    if [[ ${#failed_services[@]} -eq 0 ]]; then
        log "✅ All services are healthy"
        return 0
    else
        error "Failed services: ${failed_services[*]}"
        return 1
    fi
}

# Rollback function
rollback() {
    warn "🔄 Initiating rollback..."
    
    # Stop all services
    docker-compose -f "$COMPOSE_FILE" --env-file "$ENV_FILE" down
    
    # Restore database if backup exists
    if [[ -f "$BACKUP_DIR/database.sql" ]]; then
        log "Restoring database from backup..."
        docker-compose -f "$COMPOSE_FILE" --env-file "$ENV_FILE" up -d postgres-primary
        sleep 30
        docker exec -i tracseq-postgres-primary psql -U tracseq_admin -d tracseq_prod < "$BACKUP_DIR/database.sql"
    fi
    
    error "Rollback completed. Check logs for details."
}

# Cleanup function
cleanup() {
    log "🧹 Cleaning up..."
    
    # Remove unused Docker images
    docker image prune -f
    
    # Remove old log files (keep last 10)
    find "$(dirname "$LOG_FILE")" -name "deployment-*.log" -type f | sort -r | tail -n +11 | xargs rm -f
    
    log "✅ Cleanup completed"
}

# Signal handlers
trap 'error "Deployment interrupted"; rollback; exit 1' INT TERM

# Main deployment function
main() {
    log "🚀 Starting TracSeq 2.0 Production Deployment"
    log "📊 Deployment started at $(date)"
    log "📁 Backup directory: $BACKUP_DIR"
    log "📝 Log file: $LOG_FILE"
    
    # Check permissions
    check_permissions
    
    # Pre-deployment checks
    pre_deployment_checks
    
    # Create backup
    create_backup
    
    # Phase 1: Core Infrastructure
    log "📦 Phase 1: Deploying core infrastructure..."
    if ! deploy_services "postgres-primary" "redis-primary"; then
        error "Phase 1 failed"
        rollback
        exit 1
    fi
    
    # Wait for infrastructure
    log "⏳ Waiting for infrastructure to be ready..."
    sleep 30
    
    # Check PostgreSQL health
    if ! wait_for_service_health "postgres-primary" "5432" 20; then
        error "PostgreSQL failed to start"
        rollback
        exit 1
    fi
    
    # Phase 2: Core Services
    log "📦 Phase 2: Deploying core services..."
    if ! deploy_services "${PRODUCTION_READY_SERVICES[@]:2}"; then  # Skip postgres and redis
        error "Phase 2 failed"
        rollback
        exit 1
    fi
    
    # Wait for services to be healthy
    if ! comprehensive_health_check; then
        error "Core services health check failed"
        rollback
        exit 1
    fi
    
    # Phase 3: API Gateway and Enhanced Services
    log "📦 Phase 3: Deploying API Gateway and enhanced services..."
    if ! deploy_services "${PHASE2_SERVICES[@]}"; then
        warn "Phase 3 failed, but core services are running"
        # Don't rollback here as core services are working
    fi
    
    # Phase 4: Monitoring (Optional)
    log "📦 Phase 4: Deploying monitoring stack..."
    if ! deploy_services "${MONITORING_SERVICES[@]}"; then
        warn "Monitoring deployment failed, but application services are running"
    fi
    
    # Final health check
    log "🏥 Running final health checks..."
    if comprehensive_health_check; then
        log "✅ TracSeq 2.0 deployment completed successfully!"
        
        # Display service URLs
        log "🌐 Service URLs:"
        log "   API Gateway: http://localhost:8089"
        log "   Auth Service: http://localhost:8080"
        log "   Sample Service: http://localhost:8081"
        log "   Template Service: http://localhost:8083"
        log "   Sequencing Service: http://localhost:8084"
        log "   Notification Service: http://localhost:8085"
        
        # Display monitoring URLs
        log "📊 Monitoring URLs:"
        log "   Grafana Dashboard: http://localhost:3001"
        log "   Prometheus: http://localhost:9090"
        log "   Jaeger Tracing: http://localhost:16686"
        
        log "📝 Deployment log: $LOG_FILE"
        log "💾 Backup location: $BACKUP_DIR"
        
    else
        warn "Some services failed health checks, but deployment continued"
    fi
    
    # Cleanup
    cleanup
    
    log "🎉 Deployment process completed at $(date)"
}

# Script options
case "${1:-deploy}" in
    "deploy")
        main
        ;;
    "rollback")
        if [[ -z "${2:-}" ]]; then
            error "Please specify backup directory for rollback"
            exit 1
        fi
        BACKUP_DIR="$2"
        rollback
        ;;
    "health-check")
        comprehensive_health_check
        ;;
    "cleanup")
        cleanup
        ;;
    *)
        echo "Usage: $0 [deploy|rollback <backup-dir>|health-check|cleanup]"
        exit 1
        ;;
esac 
