#!/bin/bash

# TracSeq 2.0 Production Deployment Script
# This script handles the complete deployment of the TracSeq 2.0 system

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Print functions
print_info() {
    echo -e "${BLUE}ℹ️  $1${NC}"
}

print_success() {
    echo -e "${GREEN}✅ $1${NC}"
}

print_warning() {
    echo -e "${YELLOW}⚠️  $1${NC}"
}

print_error() {
    echo -e "${RED}❌ $1${NC}"
}

print_header() {
    echo ""
    echo -e "${BLUE}================================================${NC}"
    echo -e "${BLUE} $1${NC}"
    echo -e "${BLUE}================================================${NC}"
    echo ""
}

# Configuration
PROJECT_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
DOCKER_COMPOSE_FILE="$PROJECT_ROOT/docker/production/docker-compose.production.yml"
ENV_FILE="$PROJECT_ROOT/docker/production/.env"

# Services grouped by deployment phase
INFRASTRUCTURE_SERVICES=("postgres-primary" "redis-primary")
CORE_SERVICES=("auth-service" "event-service")
BUSINESS_SERVICES=("sample-service" "template-service" "notification-service" "sequencing-service" "transaction-service")
STORAGE_SERVICES=("storage-service")
API_SERVICES=("api-gateway")
AI_SERVICES=("rag-service" "chroma")
MONITORING_SERVICES=("prometheus" "grafana" "jaeger" "loki")

# Check prerequisites
check_prerequisites() {
    print_header "Checking Prerequisites"
    
    # Check Docker
    if ! command -v docker &> /dev/null; then
        print_error "Docker is not installed"
        exit 1
    fi
    
    # Check Docker Compose
    if ! docker compose version &> /dev/null; then
        print_error "Docker Compose is not available"
        exit 1
    fi
    
    # Check if Docker is running
    if ! docker info &> /dev/null; then
        print_error "Docker daemon is not running"
        exit 1
    fi
    
    # Check if environment file exists
    if [[ ! -f "$ENV_FILE" ]]; then
        print_error "Environment file not found: $ENV_FILE"
        exit 1
    fi
    
    print_success "All prerequisites satisfied"
}

# Prepare services for standalone deployment
prepare_services() {
    print_header "Preparing Services for Deployment"
    
    # Copy standalone Cargo.toml files for Rust services
    cd "$PROJECT_ROOT/lims-core"
    
    for service in auth_service sample_service template_service notification_service \
                   sequencing_service transaction_service enhanced_storage_service \
                   event_service spreadsheet_versioning_service; do
        if [ -f "$service/Cargo.toml.standalone" ]; then
            print_info "Preparing $service..."
            cp "$service/Cargo.toml.standalone" "$service/Cargo.toml"
        fi
    done
    
    print_success "Services prepared for deployment"
}

# Build services
build_services() {
    local services=("$@")
    print_header "Building Services: ${services[*]}"
    
    cd "$PROJECT_ROOT/docker/production"
    
    if docker compose -f docker-compose.production.yml build --parallel "${services[@]}"; then
        print_success "Services built successfully"
    else
        print_error "Failed to build services"
        return 1
    fi
}

# Deploy services
deploy_services() {
    local services=("$@")
    print_info "Deploying services: ${services[*]}"
    
    cd "$PROJECT_ROOT/docker/production"
    
    if docker compose -f docker-compose.production.yml up -d "${services[@]}"; then
        print_success "Services deployed successfully"
    else
        print_error "Failed to deploy services"
        return 1
    fi
}

# Wait for service to be healthy
wait_for_health() {
    local service=$1
    local url=$2
    local max_attempts=${3:-30}
    local attempt=1
    
    print_info "Waiting for $service to be healthy..."
    
    while [[ $attempt -le $max_attempts ]]; do
        if curl -f -s "$url" >/dev/null 2>&1; then
            print_success "$service is healthy"
            return 0
        fi
        
        echo -n "."
        sleep 5
        ((attempt++))
    done
    
    echo ""
    print_error "$service failed to become healthy"
    return 1
}

# Main deployment function
main() {
    print_header "TracSeq 2.0 Production Deployment"
    print_info "Starting deployment at $(date)"
    
    # Check prerequisites
    check_prerequisites
    
    # Prepare services
    prepare_services
    
    # Phase 1: Infrastructure
    print_header "Phase 1: Infrastructure Services"
    deploy_services "${INFRASTRUCTURE_SERVICES[@]}"
    sleep 30  # Wait for databases to initialize
    
    # Phase 2: Core Services
    print_header "Phase 2: Core Services"
    build_services "${CORE_SERVICES[@]}"
    deploy_services "${CORE_SERVICES[@]}"
    
    wait_for_health "auth-service" "http://localhost:8080/health"
    wait_for_health "event-service" "http://localhost:8087/health"
    
    # Phase 3: Business Services
    print_header "Phase 3: Business Services"
    build_services "${BUSINESS_SERVICES[@]}"
    deploy_services "${BUSINESS_SERVICES[@]}"
    
    # Phase 4: Storage Services (if available)
    print_header "Phase 4: Storage Services"
    if docker compose -f "$DOCKER_COMPOSE_FILE" config --services | grep -q "storage-service"; then
        build_services "${STORAGE_SERVICES[@]}"
        deploy_services "${STORAGE_SERVICES[@]}"
    else
        print_info "Storage service not configured in this deployment"
    fi
    
    # Phase 5: API Gateway
    print_header "Phase 5: API Gateway"
    build_services "${API_SERVICES[@]}"
    deploy_services "${API_SERVICES[@]}"
    
    # Phase 6: AI Services (optional)
    print_header "Phase 6: AI Services (Optional)"
    print_info "Skipping AI services for now (can be enabled later)"
    
    # Phase 7: Monitoring (optional)
    print_header "Phase 7: Monitoring Stack (Optional)"
    read -p "Deploy monitoring stack? (y/N): " -n 1 -r
    echo
    if [[ $REPLY =~ ^[Yy]$ ]]; then
        deploy_services "${MONITORING_SERVICES[@]}"
    fi
    
    # Final status
    print_header "Deployment Status"
    cd "$PROJECT_ROOT/docker/production"
    docker compose -f docker-compose.production.yml ps
    
    print_header "Service URLs"
    echo "🌐 Application Services:"
    echo "   • API Gateway:        http://localhost:8089"
    echo "   • Auth Service:       http://localhost:8080"
    echo "   • Sample Service:     http://localhost:8081"
    echo "   • Template Service:   http://localhost:8083"
    echo "   • Sequencing Service: http://localhost:8084"
    echo "   • Notification:       http://localhost:8085"
    echo ""
    echo "📊 Infrastructure:"
    echo "   • PostgreSQL:         localhost:15432"
    echo "   • Redis:              localhost:6379"
    echo ""
    
    if [[ $REPLY =~ ^[Yy]$ ]]; then
        echo "📈 Monitoring:"
        echo "   • Prometheus:         http://localhost:9090"
        echo "   • Grafana:            http://localhost:3001"
        echo "   • Jaeger:             http://localhost:16686"
    fi
    
    print_success "TracSeq 2.0 deployment completed successfully!"
}

# Script execution
case "${1:-deploy}" in
    "deploy")
        main
        ;;
    "status")
        cd "$PROJECT_ROOT/docker/production"
        docker compose -f docker-compose.production.yml ps
        ;;
    "logs")
        cd "$PROJECT_ROOT/docker/production"
        docker compose -f docker-compose.production.yml logs -f "${2:-}"
        ;;
    "stop")
        cd "$PROJECT_ROOT/docker/production"
        docker compose -f docker-compose.production.yml stop
        ;;
    "down")
        cd "$PROJECT_ROOT/docker/production"
        docker compose -f docker-compose.production.yml down
        ;;
    *)
        echo "Usage: $0 [deploy|status|logs [service]|stop|down]"
        exit 1
        ;;
esac 