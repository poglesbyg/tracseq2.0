#!/bin/bash

# TracSeq 2.0 Microservices Migration Script
# This script helps with the migration from monolith to microservices

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Default values
MODE="monolith"
ACTION=""
SERVICES=""

# Function to print colored output
print_color() {
    color=$1
    message=$2
    echo -e "${color}${message}${NC}"
}

# Function to show usage
show_usage() {
    echo "TracSeq 2.0 Microservices Migration Tool"
    echo ""
    echo "Usage: $0 [OPTIONS] ACTION"
    echo ""
    echo "Actions:"
    echo "  status              Show current deployment status"
    echo "  start-monolith      Start lab_manager in monolith mode"
    echo "  start-proxy         Start lab_manager in proxy mode"
    echo "  start-microservices Start all microservices"
    echo "  stop                Stop all services"
    echo "  test                Run migration tests"
    echo "  compare             Compare monolith vs microservices responses"
    echo "  migrate             Perform full migration"
    echo ""
    echo "Options:"
    echo "  -h, --help          Show this help message"
    echo "  -s, --services      Specify services (comma-separated)"
    echo "  -f, --force         Force action without confirmation"
    echo ""
    echo "Examples:"
    echo "  $0 status"
    echo "  $0 start-microservices"
    echo "  $0 start-proxy"
    echo "  $0 test -s auth,sample"
}

# Function to check service health
check_service_health() {
    service_name=$1
    port=$2
    
    if curl -f -s "http://localhost:${port}/health" > /dev/null 2>&1; then
        print_color "$GREEN" "✓ ${service_name} is healthy"
        return 0
    else
        print_color "$RED" "✗ ${service_name} is not responding"
        return 1
    fi
}

# Function to show deployment status
show_status() {
    print_color "$BLUE" "=== TracSeq 2.0 Deployment Status ==="
    echo ""
    
    # Check if running in proxy mode
    if [ ! -z "$ENABLE_PROXY_MODE" ] && [ "$ENABLE_PROXY_MODE" = "true" ]; then
        print_color "$YELLOW" "Mode: PROXY (routing to microservices)"
    else
        print_color "$YELLOW" "Mode: MONOLITH (local services)"
    fi
    echo ""
    
    # Check services
    print_color "$BLUE" "Service Status:"
    check_service_health "Lab Manager" 8080
    check_service_health "API Gateway" 8000
    check_service_health "Auth Service" 3010
    check_service_health "Sample Service" 3011
    check_service_health "Sequencing Service" 3012
    check_service_health "Template Service" 3013
    check_service_health "Storage Service" 3014
    check_service_health "Spreadsheet Service" 3015
    check_service_health "PostgreSQL" 5432
    check_service_health "Redis" 6379
}

# Function to start monolith mode
start_monolith() {
    print_color "$BLUE" "Starting Lab Manager in MONOLITH mode..."
    
    # Ensure proxy mode is disabled
    export ENABLE_PROXY_MODE=false
    
    # Start only required services
    docker-compose -f docker-compose.yml up -d postgres redis
    
    # Wait for database
    print_color "$YELLOW" "Waiting for database..."
    sleep 5
    
    # Start lab manager
    cd lab_manager
    cargo run &
    
    print_color "$GREEN" "Lab Manager started in monolith mode"
}

# Function to start proxy mode
start_proxy() {
    print_color "$BLUE" "Starting Lab Manager in PROXY mode..."
    
    # Start all microservices first
    start_microservices
    
    # Wait for services to be ready
    print_color "$YELLOW" "Waiting for microservices to be ready..."
    sleep 10
    
    # Enable proxy mode
    export ENABLE_PROXY_MODE=true
    
    # Start lab manager in proxy mode
    docker-compose -f docker-compose.microservices.yml up -d lab-manager-proxy
    
    print_color "$GREEN" "Lab Manager started in proxy mode"
}

# Function to start microservices
start_microservices() {
    print_color "$BLUE" "Starting all microservices..."
    
    # Start infrastructure services first
    docker-compose -f docker-compose.microservices.yml up -d postgres redis
    
    # Wait for infrastructure
    print_color "$YELLOW" "Waiting for infrastructure services..."
    sleep 10
    
    # Start all microservices
    if [ -z "$SERVICES" ]; then
        docker-compose -f docker-compose.microservices.yml up -d
    else
        # Start specific services
        docker-compose -f docker-compose.microservices.yml up -d $SERVICES
    fi
    
    print_color "$GREEN" "Microservices started"
}

# Function to stop all services
stop_services() {
    print_color "$BLUE" "Stopping all services..."
    
    # Stop docker services
    docker-compose -f docker-compose.microservices.yml down
    docker-compose -f docker-compose.yml down
    
    # Kill any running cargo processes
    pkill -f "cargo run" || true
    
    print_color "$GREEN" "All services stopped"
}

# Function to run migration tests
run_tests() {
    print_color "$BLUE" "Running migration tests..."
    
    # Test health endpoints
    print_color "$YELLOW" "Testing health endpoints..."
    for port in 8080 8000 3010 3011 3012 3013 3014 3015; do
        if curl -f -s "http://localhost:${port}/health" > /dev/null 2>&1; then
            print_color "$GREEN" "✓ Port ${port} health check passed"
        else
            print_color "$RED" "✗ Port ${port} health check failed"
        fi
    done
    
    # Test service discovery
    if [ "$ENABLE_PROXY_MODE" = "true" ]; then
        print_color "$YELLOW" "Testing service discovery..."
        curl -s http://localhost:8080/api/services/discovery | jq .
    fi
    
    print_color "$GREEN" "Tests completed"
}

# Function to compare monolith vs microservices
compare_services() {
    print_color "$BLUE" "Comparing monolith vs microservices responses..."
    
    # Create test data
    test_endpoints=(
        "/health"
        "/api/samples"
        "/api/templates"
        "/api/sequencing/jobs"
    )
    
    for endpoint in "${test_endpoints[@]}"; do
        print_color "$YELLOW" "Testing endpoint: $endpoint"
        
        # Get monolith response
        monolith_response=$(curl -s "http://localhost:8080${endpoint}" || echo "FAILED")
        
        # Get microservices response (through API gateway)
        micro_response=$(curl -s "http://localhost:8000${endpoint}" || echo "FAILED")
        
        # Compare
        if [ "$monolith_response" = "$micro_response" ]; then
            print_color "$GREEN" "✓ Responses match"
        else
            print_color "$RED" "✗ Responses differ"
            echo "Monolith: $monolith_response"
            echo "Microservices: $micro_response"
        fi
    done
}

# Function to perform full migration
perform_migration() {
    print_color "$BLUE" "=== Starting Full Migration Process ==="
    
    # Step 1: Ensure monolith is running
    print_color "$YELLOW" "Step 1: Starting monolith mode..."
    start_monolith
    sleep 5
    
    # Step 2: Start microservices
    print_color "$YELLOW" "Step 2: Starting microservices..."
    start_microservices
    sleep 10
    
    # Step 3: Run comparison tests
    print_color "$YELLOW" "Step 3: Comparing responses..."
    compare_services
    
    # Step 4: Switch to proxy mode
    print_color "$YELLOW" "Step 4: Switching to proxy mode..."
    export ENABLE_PROXY_MODE=true
    docker-compose -f docker-compose.microservices.yml restart lab-manager-proxy
    
    # Step 5: Verify proxy mode
    print_color "$YELLOW" "Step 5: Verifying proxy mode..."
    sleep 5
    run_tests
    
    print_color "$GREEN" "=== Migration completed successfully! ==="
}

# Parse command line arguments
while [[ $# -gt 0 ]]; do
    case $1 in
        -h|--help)
            show_usage
            exit 0
            ;;
        -s|--services)
            SERVICES="$2"
            shift 2
            ;;
        -f|--force)
            FORCE=true
            shift
            ;;
        status)
            ACTION="status"
            shift
            ;;
        start-monolith)
            ACTION="start-monolith"
            shift
            ;;
        start-proxy)
            ACTION="start-proxy"
            shift
            ;;
        start-microservices)
            ACTION="start-microservices"
            shift
            ;;
        stop)
            ACTION="stop"
            shift
            ;;
        test)
            ACTION="test"
            shift
            ;;
        compare)
            ACTION="compare"
            shift
            ;;
        migrate)
            ACTION="migrate"
            shift
            ;;
        *)
            print_color "$RED" "Unknown option: $1"
            show_usage
            exit 1
            ;;
    esac
done

# Execute action
case $ACTION in
    status)
        show_status
        ;;
    start-monolith)
        start_monolith
        ;;
    start-proxy)
        start_proxy
        ;;
    start-microservices)
        start_microservices
        ;;
    stop)
        stop_services
        ;;
    test)
        run_tests
        ;;
    compare)
        compare_services
        ;;
    migrate)
        if [ "$FORCE" != true ]; then
            print_color "$YELLOW" "This will perform a full migration. Continue? (y/N)"
            read -r response
            if [ "$response" != "y" ]; then
                print_color "$RED" "Migration cancelled"
                exit 0
            fi
        fi
        perform_migration
        ;;
    *)
        print_color "$RED" "No action specified"
        show_usage
        exit 1
        ;;
esac 
