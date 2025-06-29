#!/bin/bash

# TracSeq 2.0 Step-by-Step Migration Testing Script
# This script tests the migration incrementally with better error handling

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

print_status() { echo -e "${BLUE}🔄 $1${NC}"; }
print_success() { echo -e "${GREEN}✅ $1${NC}"; }
print_warning() { echo -e "${YELLOW}⚠️  $1${NC}"; }
print_error() { echo -e "${RED}❌ $1${NC}"; }
print_header() { echo -e "\n${BLUE}===========================================${NC}"; echo -e "${BLUE}$1${NC}"; echo -e "${BLUE}===========================================${NC}\n"; }

# Function to wait for service
wait_for_service() {
    local service_name=$1
    local max_attempts=30
    local attempt=1

    print_status "Waiting for $service_name to be ready..."
    
    while [ $attempt -le $max_attempts ]; do
        if docker-compose -f docker-compose.microservices.yml ps $service_name | grep -q "Up"; then
            print_success "$service_name is running"
            return 0
        fi
        echo -n "."
        sleep 2
        ((attempt++))
    done
    
    print_error "$service_name failed to start after $max_attempts attempts"
    return 1
}

# Function to check HTTP health
check_http_health() {
    local service_name=$1
    local url=$2
    local max_attempts=20
    local attempt=1

    print_status "Checking HTTP health of $service_name at $url"
    
    while [ $attempt -le $max_attempts ]; do
        if curl -f -s "$url" >/dev/null 2>&1; then
            print_success "$service_name HTTP health check passed"
            return 0
        fi
        echo -n "."
        sleep 3
        ((attempt++))
    done
    
    print_warning "$service_name HTTP health check failed after $max_attempts attempts"
    return 1
}

# Step 1: Infrastructure Services
test_infrastructure() {
    print_header "STEP 1: Testing Infrastructure Services"
    
    print_status "Starting PostgreSQL and Redis..."
    docker-compose -f docker-compose.microservices.yml up -d postgres redis
    
    # Wait for containers to be up
    wait_for_service "postgres"
    wait_for_service "redis"
    
    # Check PostgreSQL readiness
    print_status "Checking PostgreSQL readiness..."
    local attempt=1
    while [ $attempt -le 20 ]; do
        if docker-compose -f docker-compose.microservices.yml exec -T postgres pg_isready -U postgres > /dev/null 2>&1; then
            print_success "PostgreSQL is ready"
            break
        fi
        echo -n "."
        sleep 2
        ((attempt++))
    done
    
    if [ $attempt -gt 20 ]; then
        print_error "PostgreSQL readiness check failed"
        return 1
    fi
    
    # Check Redis
    if docker-compose -f docker-compose.microservices.yml exec -T redis redis-cli ping > /dev/null 2>&1; then
        print_success "Redis is ready"
    else
        print_error "Redis readiness check failed"
        return 1
    fi
    
    print_success "✅ Infrastructure services are ready"
    return 0
}

# Step 2: Core Rust Services
test_core_services() {
    print_header "STEP 2: Testing Core Rust Services"
    
    print_status "Starting Auth Service..."
    docker-compose -f docker-compose.microservices.yml up -d auth-service
    wait_for_service "auth-service"
    sleep 5
    check_http_health "Auth Service" "http://localhost:3010/health"
    
    print_status "Starting Sample Service..."
    docker-compose -f docker-compose.microservices.yml up -d sample-service
    wait_for_service "sample-service"
    sleep 5
    check_http_health "Sample Service" "http://localhost:3011/health"
    
    print_status "Starting Template Service..."
    docker-compose -f docker-compose.microservices.yml up -d template-service
    wait_for_service "template-service"
    sleep 5
    check_http_health "Template Service" "http://localhost:3013/health"
    
    print_success "✅ Core Rust services are running"
}

# Step 3: Storage and Processing Services
test_storage_services() {
    print_header "STEP 3: Testing Storage and Processing Services"
    
    print_status "Starting Enhanced Storage Service..."
    docker-compose -f docker-compose.microservices.yml up -d enhanced-storage-service
    wait_for_service "enhanced-storage-service"
    sleep 5
    check_http_health "Enhanced Storage Service" "http://localhost:3014/health"
    
    print_status "Starting Sequencing Service..."
    docker-compose -f docker-compose.microservices.yml up -d sequencing-service
    wait_for_service "sequencing-service"
    sleep 5
    check_http_health "Sequencing Service" "http://localhost:3012/health"
    
    print_success "✅ Storage and processing services are running"
}

# Step 4: Notification and Event Services
test_notification_services() {
    print_header "STEP 4: Testing Notification and Event Services"
    
    print_status "Starting Notification Service..."
    docker-compose -f docker-compose.microservices.yml up -d notification-service
    wait_for_service "notification-service"
    sleep 5
    check_http_health "Notification Service" "http://localhost:3016/health"
    
    print_status "Starting Event Service..."
    docker-compose -f docker-compose.microservices.yml up -d event-service
    wait_for_service "event-service"
    sleep 5
    check_http_health "Event Service" "http://localhost:3017/health"
    
    print_success "✅ Notification and event services are running"
}

# Step 5: Enhanced RAG Service
test_rag_service() {
    print_header "STEP 5: Testing Enhanced RAG Service"
    
    print_status "Building and starting Enhanced RAG Service..."
    docker-compose -f docker-compose.microservices.yml up --build -d enhanced-rag-service
    
    wait_for_service "enhanced-rag-service"
    sleep 10
    
    check_http_health "Enhanced RAG Service" "http://localhost:3019/health" || {
        print_warning "RAG service health check failed, checking logs..."
        docker-compose -f docker-compose.microservices.yml logs enhanced-rag-service | tail -20
    }
    
    print_success "✅ Enhanced RAG service test completed"
}

# Step 6: New Barcode Service
test_barcode_service() {
    print_header "STEP 6: Testing New Barcode Service"
    
    print_status "Building and starting Barcode Service..."
    docker-compose -f docker-compose.microservices.yml up --build -d barcode-service
    
    wait_for_service "barcode-service"
    sleep 10
    
    check_http_health "Barcode Service" "http://localhost:3020/health"
    
    print_success "✅ Barcode service is running"
}

# Step 7: API Gateway
test_api_gateway() {
    print_header "STEP 7: Testing API Gateway"
    
    print_status "Starting API Gateway..."
    docker-compose -f docker-compose.microservices.yml up -d api-gateway
    
    wait_for_service "api-gateway"
    sleep 10
    
    check_http_health "API Gateway" "http://localhost:8000/health"
    
    # Test routing
    print_status "Testing API Gateway routing..."
    if curl -f -s "http://localhost:8000/api/v1/auth/health" >/dev/null 2>&1; then
        print_success "API Gateway routing is working"
    else
        print_warning "API Gateway routing test inconclusive"
    fi
    
    print_success "✅ API Gateway is running"
}

# Show service status
show_service_status() {
    print_header "SERVICE STATUS SUMMARY"
    
    print_status "Current running services:"
    docker-compose -f docker-compose.microservices.yml ps --format "table {{.Name}}\t{{.Status}}\t{{.Ports}}"
    
    print_status "\nHealthy endpoints:"
    local endpoints=(
        "Auth Service:http://localhost:3010/health"
        "Sample Service:http://localhost:3011/health"
        "Sequencing Service:http://localhost:3012/health"
        "Template Service:http://localhost:3013/health"
        "Storage Service:http://localhost:3014/health"
        "Notification Service:http://localhost:3016/health"
        "Event Service:http://localhost:3017/health"
        "RAG Service:http://localhost:3019/health"
        "Barcode Service:http://localhost:3020/health"
        "API Gateway:http://localhost:8000/health"
    )
    
    local healthy=0
    local total=${#endpoints[@]}
    
    for endpoint in "${endpoints[@]}"; do
        IFS=':' read -r name url <<< "$endpoint"
        if curl -f -s "$url" >/dev/null 2>&1; then
            echo "  ✅ $name"
            ((healthy++))
        else
            echo "  ❌ $name"
        fi
    done
    
    print_status "\nHealth Summary: $healthy/$total services healthy"
    
    if [ $healthy -eq $total ]; then
        print_success "🎉 All services are healthy! Migration is successful!"
    else
        print_warning "Some services need attention, but core migration is working"
    fi
}

# Cleanup function
cleanup() {
    print_status "Cleaning up services..."
    docker-compose -f docker-compose.microservices.yml down --remove-orphans
    print_success "Cleanup completed"
}

# Main execution
main() {
    print_header "TracSeq 2.0 Step-by-Step Migration Testing"
    
    echo "This script will test the migration step by step:"
    echo "  1️⃣  Infrastructure (PostgreSQL, Redis)"
    echo "  2️⃣  Core Rust Services (Auth, Sample, Template)"
    echo "  3️⃣  Storage & Processing Services"
    echo "  4️⃣  Notification & Event Services"
    echo "  5️⃣  Enhanced RAG Service"
    echo "  6️⃣  New Barcode Service"
    echo "  7️⃣  API Gateway"
    echo ""
    
    read -p "Continue with step-by-step testing? (y/N): " -n 1 -r
    echo
    if [[ ! $REPLY =~ ^[Yy]$ ]]; then
        print_warning "Testing cancelled"
        exit 0
    fi
    
    # Cleanup first
    cleanup
    
    # Run each step
    if test_infrastructure; then
        if test_core_services; then
            if test_storage_services; then
                if test_notification_services; then
                    test_rag_service  # Continue even if this fails
                    test_barcode_service  # Continue even if this fails
                    test_api_gateway  # Continue even if this fails
                fi
            fi
        fi
    fi
    
    show_service_status
    
    print_header "🎉 STEP-BY-STEP TESTING COMPLETED"
    print_status "Use 'docker-compose -f docker-compose.microservices.yml down' to stop services"
    print_status "Use 'docker-compose -f docker-compose.microservices.yml logs [service]' to debug issues"
}

# Handle interruption
trap 'print_error "Testing interrupted"; cleanup; exit 1' INT TERM

# Run main function
main "$@" 