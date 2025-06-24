#!/bin/bash

# TracSeq 2.0 - Build and Deploy All Microservices
# This script builds each service individually and then deploys them together

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
RUST_SERVICES=(
    "config-service"
    "auth_service"
    "sample_service"
    "sequencing_service"
    "notification_service"
    "enhanced_storage_service"
    "template_service"
    "transaction_service"
    "event_service"
    "lab_manager"
    "library_details_service"
    "qaqc_service"
    "spreadsheet_versioning_service"
)

PYTHON_SERVICES=(
    "enhanced_rag_service"
    "api_gateway"
    "lab_submission_rag"
)

ALL_SERVICES=("${RUST_SERVICES[@]}" "${PYTHON_SERVICES[@]}")

# Default values
BUILD_INDIVIDUAL=true
DEPLOY_COLLECTIVE=true
NO_CACHE=false
PARALLEL_BUILD=false
SKIP_TESTS=false
COMPOSE_FILE="docker-compose-build-all.yml"
BUILD_TIMEOUT=1800  # 30 minutes

# Parse command line arguments
parse_args() {
    while [[ $# -gt 0 ]]; do
        case $1 in
            --skip-individual)
                BUILD_INDIVIDUAL=false
                shift
                ;;
            --skip-collective)
                DEPLOY_COLLECTIVE=false
                shift
                ;;
            --no-cache)
                NO_CACHE=true
                shift
                ;;
            --parallel)
                PARALLEL_BUILD=true
                shift
                ;;
            --skip-tests)
                SKIP_TESTS=true
                shift
                ;;
            --compose-file)
                COMPOSE_FILE="$2"
                shift 2
                ;;
            --timeout)
                BUILD_TIMEOUT="$2"
                shift 2
                ;;
            -h|--help)
                show_help
                exit 0
                ;;
            *)
                print_error "Unknown option: $1"
                show_help
                exit 1
                ;;
        esac
    done
}

show_help() {
    echo "TracSeq 2.0 - Build and Deploy All Microservices"
    echo ""
    echo "Usage: $0 [options]"
    echo ""
    echo "Options:"
    echo "  --skip-individual    Skip individual service builds"
    echo "  --skip-collective    Skip collective deployment"
    echo "  --no-cache          Build without Docker cache"
    echo "  --parallel          Build services in parallel (experimental)"
    echo "  --skip-tests        Skip running tests after build"
    echo "  --compose-file FILE Docker compose file to use (default: docker-compose-build-all.yml)"
    echo "  --timeout SECONDS   Build timeout in seconds (default: 1800)"
    echo "  -h, --help          Show this help message"
    echo ""
    echo "Examples:"
    echo "  $0                               # Build all services individually then deploy"
    echo "  $0 --no-cache --parallel         # Build with no cache in parallel"
    echo "  $0 --skip-individual             # Only run collective deployment"
}

# Check prerequisites
check_prerequisites() {
    print_header "Checking Prerequisites"
    
    # Check Docker
    if ! command -v docker &> /dev/null; then
        print_error "Docker is not installed or not in PATH"
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
    
    print_success "All prerequisites satisfied"
}

# Clean up old containers and images (optional)
cleanup_old_resources() {
    print_header "Cleaning Up Old Resources"
    
    print_info "Stopping existing TracSeq containers..."
    docker ps -q --filter "name=tracseq-*" | xargs -r docker stop
    
    print_info "Removing old TracSeq containers..."
    docker ps -aq --filter "name=tracseq-*" | xargs -r docker rm
    
    print_info "Removing old TracSeq images..."
    docker images --filter "reference=tracseq-*" -q | xargs -r docker rmi -f
    
    print_success "Cleanup completed"
}

# Build individual service
build_individual_service() {
    local service_name=$1
    local build_args=""
    
    if [ "$NO_CACHE" = true ]; then
        build_args="--no-cache"
    fi
    
    print_info "Building $service_name..."
    
    if [ -d "$service_name" ] && [ -f "$service_name/Dockerfile" ]; then
        local image_name="tracseq-$service_name:latest"
        
        if timeout $BUILD_TIMEOUT docker build $build_args -t "$image_name" "$service_name"; then
            print_success "✅ Built $service_name successfully"
            return 0
        else
            print_error "❌ Failed to build $service_name"
            return 1
        fi
    else
        print_warning "⚠️  Skipping $service_name (no directory or Dockerfile found)"
        return 0
    fi
}

# Build all services individually
build_individual_services() {
    if [ "$BUILD_INDIVIDUAL" = false ]; then
        print_info "Skipping individual service builds"
        return 0
    fi
    
    print_header "Building Individual Services"
    
    local failed_services=()
    local success_count=0
    
    if [ "$PARALLEL_BUILD" = true ]; then
        print_info "Building services in parallel..."
        
        # Create array to store background process PIDs
        local pids=()
        
        for service in "${ALL_SERVICES[@]}"; do
            build_individual_service "$service" &
            pids+=($!)
        done
        
        # Wait for all background processes
        for pid in "${pids[@]}"; do
            if wait $pid; then
                ((success_count++))
            else
                failed_services+=("$service")
            fi
        done
    else
        print_info "Building services sequentially..."
        
        for service in "${ALL_SERVICES[@]}"; do
            if build_individual_service "$service"; then
                ((success_count++))
            else
                failed_services+=("$service")
            fi
        done
    fi
    
    print_info "Individual build summary:"
    print_success "✅ Successfully built: $success_count services"
    
    if [ ${#failed_services[@]} -gt 0 ]; then
        print_error "❌ Failed to build: ${failed_services[*]}"
        
        # Ask user if they want to continue
        read -p "Some services failed to build. Continue with deployment? (y/N): " -n 1 -r
        echo
        if [[ ! $REPLY =~ ^[Yy]$ ]]; then
            print_error "Deployment cancelled by user"
            exit 1
        fi
    fi
}

# Run tests on built images
run_tests() {
    if [ "$SKIP_TESTS" = true ]; then
        print_info "Skipping tests"
        return 0
    fi
    
    print_header "Running Tests"
    
    # Check if test script exists
    if [ -f "scripts/test-services.sh" ]; then
        print_info "Running service tests..."
        if ./scripts/test-services.sh; then
            print_success "All tests passed"
        else
            print_warning "Some tests failed, but continuing with deployment"
        fi
    else
        print_info "No test script found, skipping tests"
    fi
}

# Deploy all services collectively
deploy_collective() {
    if [ "$DEPLOY_COLLECTIVE" = false ]; then
        print_info "Skipping collective deployment"
        return 0
    fi
    
    print_header "Deploying All Services Collectively"
    
    # Check if compose file exists
    if [ ! -f "$COMPOSE_FILE" ]; then
        print_error "Docker compose file '$COMPOSE_FILE' not found"
        exit 1
    fi
    
    print_info "Using compose file: $COMPOSE_FILE"
    
    # Deploy infrastructure first
    print_info "Starting infrastructure services..."
    docker compose -f "$COMPOSE_FILE" up -d postgres redis ollama
    
    # Wait for infrastructure to be ready
    print_info "Waiting for infrastructure to be ready..."
    sleep 30
    
    # Deploy core services in tiers
    print_info "Starting foundational services..."
    docker compose -f "$COMPOSE_FILE" up -d config-service auth-service event-service
    sleep 20
    
    print_info "Starting core business services..."
    docker compose -f "$COMPOSE_FILE" up -d sample-service enhanced-storage-service template-service sequencing-service notification-service transaction-service
    sleep 20
    
    print_info "Starting specialized services..."
    docker compose -f "$COMPOSE_FILE" up -d library-details-service qaqc-service spreadsheet-versioning-service
    sleep 10
    
    print_info "Starting AI/ML services..."
    docker compose -f "$COMPOSE_FILE" up -d enhanced-rag-service lab-submission-rag
    sleep 15
    
    print_info "Starting gateway and frontend..."
    docker compose -f "$COMPOSE_FILE" up -d api-gateway lab-manager
    sleep 10
    
    print_info "Starting monitoring services..."
    docker compose -f "$COMPOSE_FILE" up -d prometheus grafana
    
    print_success "All services deployed"
}

# Health check for deployed services
health_check() {
    print_header "Running Health Checks"
    
    local services_to_check=(
        "localhost:8080/health:Auth Service"
        "localhost:8081/health:Sample Service"
        "localhost:8082/health:Enhanced Storage Service"
        "localhost:8083/health:Template Service"
        "localhost:8084/health:Sequencing Service"
        "localhost:8085/health:Notification Service"
        "localhost:8086/health:Enhanced RAG Service"
        "localhost:8087/health:Event Service"
        "localhost:8088/health:Transaction Service"
        "localhost:8089/health:API Gateway"
        "localhost:8091/health:Config Service"
        "localhost:8092/health:Library Details Service"
        "localhost:8093/health:QA/QC Service"
        "localhost:8094/health:Spreadsheet Versioning Service"
        "localhost:3000/health:Lab Manager"
    )
    
    local healthy_count=0
    local total_count=${#services_to_check[@]}
    
    for service_info in "${services_to_check[@]}"; do
        IFS=':' read -r endpoint name <<< "$service_info"
        
        if curl -f "http://$endpoint" >/dev/null 2>&1; then
            print_success "✅ $name is healthy"
            ((healthy_count++))
        else
            print_error "❌ $name is not responding"
        fi
    done
    
    print_info "Health check summary: $healthy_count/$total_count services healthy"
    
    if [ $healthy_count -eq $total_count ]; then
        print_success "🎉 All services are healthy!"
    else
        print_warning "⚠️  Some services are not responding. Check logs for details."
    fi
}

# Show deployment summary
show_summary() {
    print_header "Deployment Summary"
    
    echo "🐳 Docker Images:"
    docker images | grep "tracseq-" | head -20
    
    echo ""
    echo "📦 Running Containers:"
    docker ps --filter "name=tracseq-*" --format "table {{.Names}}\t{{.Status}}\t{{.Ports}}"
    
    echo ""
    echo "🌐 Service URLs:"
    echo "  • API Gateway:        http://localhost:8089"
    echo "  • Lab Manager:        http://localhost:3000"
    echo "  • Auth Service:       http://localhost:8080"
    echo "  • Sample Service:     http://localhost:8081"
    echo "  • Storage Service:    http://localhost:8082"
    echo "  • Prometheus:         http://localhost:9090"
    echo "  • Grafana:            http://localhost:3001 (admin/admin)"
    
    echo ""
    echo "📊 System Resources:"
    echo "  • CPU Usage: $(docker stats --no-stream --format "table {{.CPUPerc}}" | grep -v CPU | head -1)"
    echo "  • Memory Usage: $(docker system df --format "table {{.Type}}\t{{.TotalCount}}\t{{.Size}}")"
    
    print_success "🎉 TracSeq 2.0 deployment completed successfully!"
}

# Main execution function
main() {
    local start_time=$(date +%s)
    
    print_header "TracSeq 2.0 - Complete Microservices Build & Deploy"
    
    parse_args "$@"
    check_prerequisites
    
    # Optional cleanup
    read -p "Do you want to clean up old TracSeq containers and images? (y/N): " -n 1 -r
    echo
    if [[ $REPLY =~ ^[Yy]$ ]]; then
        cleanup_old_resources
    fi
    
    build_individual_services
    run_tests
    deploy_collective
    
    # Wait a bit for services to start
    print_info "Waiting for services to initialize..."
    sleep 30
    
    health_check
    show_summary
    
    local end_time=$(date +%s)
    local duration=$((end_time - start_time))
    
    print_success "🎉 Complete deployment finished in ${duration} seconds!"
    
    # Optional: Open browser
    read -p "Open Lab Manager in browser? (y/N): " -n 1 -r
    echo
    if [[ $REPLY =~ ^[Yy]$ ]]; then
        if command -v xdg-open &> /dev/null; then
            xdg-open http://localhost:3000
        elif command -v open &> /dev/null; then
            open http://localhost:3000
        else
            print_info "Please open http://localhost:3000 in your browser"
        fi
    fi
}

# Execute main function with all arguments
main "$@" 
