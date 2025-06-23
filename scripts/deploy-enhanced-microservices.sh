#!/bin/bash

# TracSeq Enhanced Microservices Deployment Script
# This script deploys the complete enhanced microservices ecosystem

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Configuration
DEPLOYMENT_MODE=${1:-"development"} # development, staging, production
SKIP_BUILD=${SKIP_BUILD:-false}
ENABLE_MONITORING=${ENABLE_MONITORING:-true}
ENABLE_SERVICE_MESH=${ENABLE_SERVICE_MESH:-true}
WAIT_FOR_HEALTH=${WAIT_FOR_HEALTH:-true}

echo -e "${BLUE}🚀 TracSeq Enhanced Microservices Deployment${NC}"
echo -e "${BLUE}Deployment Mode: ${DEPLOYMENT_MODE}${NC}"
echo -e "${BLUE}Skip Build: ${SKIP_BUILD}${NC}"
echo -e "${BLUE}Enable Monitoring: ${ENABLE_MONITORING}${NC}"
echo -e "${BLUE}Enable Service Mesh: ${ENABLE_SERVICE_MESH}${NC}"
echo ""

# Function to print step headers
print_step() {
    echo -e "${GREEN}📋 Step: $1${NC}"
}

# Function to print warnings
print_warning() {
    echo -e "${YELLOW}⚠️  Warning: $1${NC}"
}

# Function to print errors
print_error() {
    echo -e "${RED}❌ Error: $1${NC}"
}

# Function to check if command exists
command_exists() {
    command -v "$1" >/dev/null 2>&1
}

# Validate prerequisites
validate_prerequisites() {
    print_step "Validating Prerequisites"
    
    if ! command_exists docker; then
        print_error "Docker is not installed or not in PATH"
        exit 1
    fi
    
    if ! command_exists docker-compose; then
        print_error "Docker Compose is not installed or not in PATH"
        exit 1
    fi
    
    # Check Docker daemon
    if ! docker info >/dev/null 2>&1; then
        print_error "Docker daemon is not running"
        exit 1
    fi
    
    # Check available disk space (at least 10GB)
    available_space=$(df . | tail -1 | awk '{print $4}')
    if [ "$available_space" -lt 10485760 ]; then # 10GB in KB
        print_warning "Less than 10GB available disk space"
    fi
    
    echo "✅ Prerequisites validated"
}

# Setup environment
setup_environment() {
    print_step "Setting up Environment"
    
    # Create necessary directories
    mkdir -p monitoring/prometheus/rules
    mkdir -p monitoring/grafana/dashboards/{tracseq,infrastructure,apm}
    mkdir -p monitoring/loki
    mkdir -p monitoring/alertmanager
    mkdir -p service-mesh/sidecars
    mkdir -p config-service/config
    mkdir -p logs
    
    # Set environment variables based on deployment mode
    case $DEPLOYMENT_MODE in
        "production")
            export RUST_LOG=info
            export LOG_LEVEL=INFO
            export REPLICAS=3
            ;;
        "staging")
            export RUST_LOG=info
            export LOG_LEVEL=INFO
            export REPLICAS=2
            ;;
        "development"|*)
            export RUST_LOG=debug
            export LOG_LEVEL=DEBUG
            export REPLICAS=1
            ;;
    esac
    
    echo "✅ Environment setup complete"
}

# Build services
build_services() {
    if [ "$SKIP_BUILD" = "true" ]; then
        print_step "Skipping Build Phase"
        return
    fi
    
    print_step "Building Microservices"
    
    # Build core services
    echo "Building core microservices..."
    docker-compose -f docker-compose.enhanced-microservices.yml build \
        config-service \
        auth-service \
        sample-service \
        enhanced-storage-service \
        template-service \
        sequencing-service \
        notification-service \
        enhanced-rag-service \
        event-service \
        transaction-service \
        api-gateway
    
    echo "✅ Core services built"
}

# Deploy infrastructure
deploy_infrastructure() {
    print_step "Deploying Infrastructure Components"
    
    # Deploy databases and Redis first
    echo "Starting databases and Redis..."
    docker-compose -f enhanced_storage_service/docker-compose.minimal.yml up -d postgres redis
    
    # Wait for database
    echo "Waiting for database to be ready..."
    sleep 10
    
    # Deploy monitoring stack if enabled
    if [ "$ENABLE_MONITORING" = "true" ]; then
        echo "Starting monitoring stack..."
        docker-compose -f monitoring/docker-compose.monitoring.yml up -d \
            prometheus \
            grafana \
            jaeger \
            loki
    fi
    
    echo "✅ Infrastructure deployed"
}

# Deploy configuration service
deploy_config_service() {
    print_step "Deploying Configuration Service"
    
    docker-compose -f docker-compose.enhanced-microservices.yml up -d config-service
    
    # Wait for config service
    echo "Waiting for configuration service..."
    wait_for_health "http://localhost:8091/health" "Configuration Service"
    
    echo "✅ Configuration service deployed"
}

# Deploy service mesh
deploy_service_mesh() {
    if [ "$ENABLE_SERVICE_MESH" = "true" ]; then
        print_step "Deploying Service Mesh (Envoy)"
        
        # Create Envoy sidecar configurations if they don't exist
        if [ ! -f "service-mesh/sidecars/auth-service-envoy.yaml" ]; then
            echo "Creating default Envoy sidecar configurations..."
            create_default_sidecar_configs
        fi
        
        docker-compose -f docker-compose.enhanced-microservices.yml up -d envoy-proxy
        
        # Wait for Envoy admin interface
        wait_for_health "http://localhost:9901/stats" "Envoy Proxy"
        
        echo "✅ Service mesh deployed"
    else
        print_step "Skipping Service Mesh Deployment"
    fi
}

# Deploy core microservices
deploy_microservices() {
    print_step "Deploying Core Microservices"
    
    # Deploy services in dependency order
    echo "Deploying auth service..."
    docker-compose -f docker-compose.enhanced-microservices.yml up -d auth-service
    wait_for_health "http://localhost:8080/health" "Auth Service"
    
    echo "Deploying event service..."
    docker-compose -f docker-compose.enhanced-microservices.yml up -d event-service
    wait_for_health "http://localhost:8087/health" "Event Service"
    
    echo "Deploying notification service..."
    docker-compose -f docker-compose.enhanced-microservices.yml up -d notification-service
    wait_for_health "http://localhost:8085/health" "Notification Service"
    
    echo "Deploying enhanced storage service..."
    docker-compose -f docker-compose.enhanced-microservices.yml up -d enhanced-storage-service
    wait_for_health "http://localhost:8082/health" "Enhanced Storage Service"
    
    echo "Deploying sample service..."
    docker-compose -f docker-compose.enhanced-microservices.yml up -d sample-service
    wait_for_health "http://localhost:8081/health" "Sample Service"
    
    echo "Deploying template service..."
    docker-compose -f docker-compose.enhanced-microservices.yml up -d template-service
    wait_for_health "http://localhost:8083/health" "Template Service"
    
    echo "Deploying sequencing service..."
    docker-compose -f docker-compose.enhanced-microservices.yml up -d sequencing-service
    wait_for_health "http://localhost:8084/health" "Sequencing Service"
    
    echo "Deploying RAG service..."
    docker-compose -f docker-compose.enhanced-microservices.yml up -d enhanced-rag-service
    wait_for_health "http://localhost:8086/health" "Enhanced RAG Service"
    
    echo "Deploying transaction service..."
    docker-compose -f docker-compose.enhanced-microservices.yml up -d transaction-service
    wait_for_health "http://localhost:8088/health" "Transaction Service"
    
    echo "Deploying API gateway..."
    docker-compose -f docker-compose.enhanced-microservices.yml up -d api-gateway
    wait_for_health "http://localhost:8089/health" "API Gateway"
    
    echo "✅ Core microservices deployed"
}

# Deploy monitoring components
deploy_monitoring() {
    if [ "$ENABLE_MONITORING" = "true" ]; then
        print_step "Deploying Enhanced Monitoring"
        
        # Deploy additional monitoring components
        docker-compose -f monitoring/docker-compose.monitoring.yml up -d \
            node-exporter \
            cadvisor \
            redis-exporter \
            postgres-exporter \
            alertmanager \
            uptime-kuma
        
        echo "✅ Enhanced monitoring deployed"
    fi
}

# Wait for service health
wait_for_health() {
    local url=$1
    local service_name=$2
    local max_attempts=30
    local attempt=1
    
    if [ "$WAIT_FOR_HEALTH" = "false" ]; then
        return
    fi
    
    echo "Waiting for $service_name to be healthy..."
    
    while [ $attempt -le $max_attempts ]; do
        if curl -f -s "$url" >/dev/null 2>&1; then
            echo "✅ $service_name is healthy"
            return 0
        fi
        
        echo "Attempt $attempt/$max_attempts - waiting for $service_name..."
        sleep 5
        attempt=$((attempt + 1))
    done
    
    print_warning "$service_name health check timed out"
    return 1
}

# Create default sidecar configurations
create_default_sidecar_configs() {
    local services=("auth-service" "sample-service" "enhanced-storage-service")
    
    for service in "${services[@]}"; do
        cat > "service-mesh/sidecars/${service}-envoy.yaml" << EOF
admin:
  access_log_path: /tmp/admin_access.log
  address:
    socket_address:
      protocol: TCP
      address: 0.0.0.0
      port_value: 19000

static_resources:
  listeners:
  - name: listener_0
    address:
      socket_address:
        protocol: TCP
        address: 0.0.0.0
        port_value: 15000
    filter_chains:
    - filters:
      - name: envoy.filters.network.http_connection_manager
        typed_config:
          "@type": type.googleapis.com/envoy.extensions.filters.network.http_connection_manager.v3.HttpConnectionManager
          stat_prefix: ingress_http
          route_config:
            name: local_route
            virtual_hosts:
            - name: local_service
              domains: ["*"]
              routes:
              - match:
                  prefix: "/"
                route:
                  cluster: local_service
          http_filters:
          - name: envoy.filters.http.router

  clusters:
  - name: local_service
    connect_timeout: 0.25s
    type: LOGICAL_DNS
    dns_lookup_family: V4_ONLY
    lb_policy: ROUND_ROBIN
    load_assignment:
      cluster_name: local_service
      endpoints:
      - lb_endpoints:
        - endpoint:
            address:
              socket_address:
                address: $service
                port_value: 808${service: -1}
EOF
    done
}

# Validate deployment
validate_deployment() {
    print_step "Validating Deployment"
    
    local failed_services=()
    
    # Check core services
    local services=(
        "http://localhost:8091/health:Configuration Service"
        "http://localhost:8080/health:Auth Service"
        "http://localhost:8081/health:Sample Service"
        "http://localhost:8082/health:Enhanced Storage Service"
        "http://localhost:8083/health:Template Service"
        "http://localhost:8084/health:Sequencing Service"
        "http://localhost:8085/health:Notification Service"
        "http://localhost:8086/health:Enhanced RAG Service"
        "http://localhost:8087/health:Event Service"
        "http://localhost:8088/health:Transaction Service"
        "http://localhost:8089/health:API Gateway"
    )
    
    # Check monitoring services if enabled
    if [ "$ENABLE_MONITORING" = "true" ]; then
        services+=(
            "http://localhost:9090/-/healthy:Prometheus"
            "http://localhost:3001/api/health:Grafana"
            "http://localhost:16686/:Jaeger"
            "http://localhost:3100/ready:Loki"
        )
    fi
    
    # Check service mesh if enabled
    if [ "$ENABLE_SERVICE_MESH" = "true" ]; then
        services+=(
            "http://localhost:9901/stats:Envoy Proxy"
        )
    fi
    
    for service_info in "${services[@]}"; do
        local url=$(echo "$service_info" | cut -d: -f1)
        local name=$(echo "$service_info" | cut -d: -f2)
        
        if ! curl -f -s "$url" >/dev/null 2>&1; then
            failed_services+=("$name")
        fi
    done
    
    if [ ${#failed_services[@]} -eq 0 ]; then
        echo "✅ All services are healthy"
    else
        print_warning "Some services failed health checks: ${failed_services[*]}"
    fi
}

# Display deployment summary
display_summary() {
    print_step "Deployment Summary"
    
    echo -e "${GREEN}🎉 TracSeq Enhanced Microservices Deployment Complete!${NC}"
    echo ""
    echo "🌐 Service URLs:"
    echo "  - API Gateway: http://localhost:8089"
    echo "  - Configuration Service: http://localhost:8091"
    echo "  - Auth Service: http://localhost:8080"
    echo "  - Sample Service: http://localhost:8081"
    echo "  - Enhanced Storage Service: http://localhost:8082"
    echo "  - Template Service: http://localhost:8083"
    echo "  - Sequencing Service: http://localhost:8084"
    echo "  - Notification Service: http://localhost:8085"
    echo "  - Enhanced RAG Service: http://localhost:8086"
    echo "  - Event Service: http://localhost:8087"
    echo "  - Transaction Service: http://localhost:8088"
    
    if [ "$ENABLE_MONITORING" = "true" ]; then
        echo ""
        echo "📊 Monitoring URLs:"
        echo "  - Prometheus: http://localhost:9090"
        echo "  - Grafana: http://localhost:3001 (admin/tracseq-admin)"
        echo "  - Jaeger: http://localhost:16686"
        echo "  - Loki: http://localhost:3100"
        echo "  - Uptime Kuma: http://localhost:3002"
    fi
    
    if [ "$ENABLE_SERVICE_MESH" = "true" ]; then
        echo ""
        echo "🕸️ Service Mesh URLs:"
        echo "  - Envoy Admin: http://localhost:9901"
        echo "  - Service Mesh Gateway: http://localhost:8090"
    fi
    
    echo ""
    echo "📚 Quick Commands:"
    echo "  - View logs: docker-compose -f docker-compose.enhanced-microservices.yml logs -f [service]"
    echo "  - Scale service: docker-compose -f docker-compose.enhanced-microservices.yml up -d --scale [service]=3"
    echo "  - Stop all: docker-compose -f docker-compose.enhanced-microservices.yml down"
    echo "  - Update config: curl -X PUT http://localhost:8091/configs/[service]/[env]/bulk -d '{...}'"
    echo ""
}

# Main execution
main() {
    validate_prerequisites
    setup_environment
    build_services
    deploy_infrastructure
    deploy_config_service
    deploy_service_mesh
    deploy_microservices
    deploy_monitoring
    validate_deployment
    display_summary
}

# Execute main function
main "$@"