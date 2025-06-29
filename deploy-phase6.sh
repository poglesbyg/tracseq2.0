#!/bin/bash

# TracSeq 2.0 - Phase 6 Deployment Script
# Production Readiness & Observability

set -euo pipefail

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Functions
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

# Check prerequisites
check_prerequisites() {
    log "Checking prerequisites..."
    
    # Check Docker
    if ! command -v docker &> /dev/null; then
        error "Docker is not installed"
        exit 1
    fi
    
    # Check Docker Compose
    if ! docker compose version &> /dev/null; then
        error "Docker Compose is not installed"
        exit 1
    fi
    
    # Check if monitoring network exists
    if ! docker network ls | grep -q "tracseq-monitoring"; then
        log "Creating monitoring network..."
        docker network create tracseq-monitoring
    fi
    
    log "Prerequisites check passed"
}

# Deploy monitoring stack
deploy_monitoring() {
    log "Deploying Phase 6 Monitoring Stack..."
    
    # Deploy Prometheus and Grafana
    docker compose -f docker-compose.phase6-monitoring.yml up -d prometheus grafana alertmanager
    
    # Wait for services to be ready
    log "Waiting for monitoring services to start..."
    sleep 30
    
    # Deploy exporters
    docker compose -f docker-compose.phase6-monitoring.yml up -d node-exporter postgres-exporter redis-exporter
    
    # Deploy Jaeger for tracing
    docker compose -f docker-compose.phase6-monitoring.yml up -d jaeger
    
    # Deploy ELK stack for logging
    docker compose -f docker-compose.phase6-monitoring.yml up -d elasticsearch logstash kibana
    
    log "Monitoring stack deployed successfully"
}

# Configure security
configure_security() {
    log "Configuring security features..."
    
    # Generate mTLS certificates
    if [[ ! -d "security/mtls/certificates" ]]; then
        log "Generating mTLS certificates..."
        chmod +x security/mtls/generate-certificates.sh
        cd security/mtls && ./generate-certificates.sh && cd ../..
    fi
    
    # Deploy Falco for runtime security (optional)
    # docker compose -f docker-compose.phase6-monitoring.yml up -d falco
    
    log "Security configuration complete"
}

# Create performance optimization configs
create_performance_configs() {
    log "Creating performance optimization configurations..."
    
    # Create connection pool configuration
    cat > config/database-pool.yml << EOF
# Database Connection Pool Configuration
postgres:
  max_connections: 100
  min_connections: 10
  connection_timeout: 30s
  idle_timeout: 900s
  max_lifetime: 3600s

redis:
  pool_size: 50
  connection_timeout: 5s
  read_timeout: 5s
  write_timeout: 5s
EOF

    # Create cache configuration
    cat > config/cache-config.yml << EOF
# Cache Configuration
cache:
  default_ttl: 300s
  max_entries: 10000
  eviction_policy: lru
  
  endpoints:
    - pattern: "/api/samples/*"
      ttl: 600s
    - pattern: "/api/templates/*"
      ttl: 1800s
    - pattern: "/api/storage/locations/*"
      ttl: 3600s
EOF

    log "Performance configurations created"
}

# Health check function
health_check() {
    log "Running health checks..."
    
    local services=(
        "Prometheus:http://localhost:9090/-/healthy"
        "Grafana:http://localhost:3000/api/health"
        "Jaeger:http://localhost:16686/"
        "Elasticsearch:http://localhost:9200/_cluster/health"
        "Kibana:http://localhost:5601/api/status"
        "AlertManager:http://localhost:9093/-/healthy"
    )
    
    for service in "${services[@]}"; do
        IFS=':' read -r name url <<< "$service"
        if curl -sf "$url" > /dev/null 2>&1; then
            log "✅ $name is healthy"
        else
            warn "❌ $name is not responding"
        fi
    done
}

# Display access information
display_access_info() {
    echo ""
    log "🎉 Phase 6 Deployment Complete!"
    echo ""
    info "Access your monitoring dashboards:"
    echo "  📊 Prometheus: http://localhost:9090"
    echo "  📈 Grafana: http://localhost:3000 (admin/admin)"
    echo "  🔍 Jaeger: http://localhost:16686"
    echo "  📝 Kibana: http://localhost:5601"
    echo "  🚨 AlertManager: http://localhost:9093"
    echo ""
    info "Next steps:"
    echo "  1. Import Grafana dashboards from monitoring/grafana/dashboards/"
    echo "  2. Configure alert notification channels in AlertManager"
    echo "  3. Set up log shipping from microservices to Logstash"
    echo "  4. Configure mTLS in each microservice using certificates from security/mtls/certificates/"
    echo ""
}

# Main execution
main() {
    log "Starting TracSeq 2.0 Phase 6 Deployment"
    log "Phase 6: Production Readiness & Observability"
    
    # Check prerequisites
    check_prerequisites
    
    # Create necessary directories
    mkdir -p config logs
    
    # Deploy monitoring stack
    deploy_monitoring
    
    # Configure security
    configure_security
    
    # Create performance configs
    create_performance_configs
    
    # Run health checks
    health_check
    
    # Display access information
    display_access_info
}

# Run main function
main "$@"