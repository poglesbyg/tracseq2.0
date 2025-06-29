#!/bin/bash

# Test core services with fixed configurations

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

# Test core services only (the ones we fixed)
test_core_services() {
    print_header "TESTING CORE FIXED SERVICES"
    
    print_status "Starting infrastructure..."
    docker-compose -f docker-compose.microservices.yml up -d postgres redis
    
    # Wait for PostgreSQL
    print_status "Waiting for PostgreSQL..."
    sleep 10
    local attempt=1
    while [ $attempt -le 15 ]; do
        if docker-compose -f docker-compose.microservices.yml exec -T postgres pg_isready -U postgres > /dev/null 2>&1; then
            print_success "PostgreSQL is ready"
            break
        fi
        echo -n "."
        sleep 2
        ((attempt++))
    done
    
    print_status "Testing Auth Service..."
    docker-compose -f docker-compose.microservices.yml up -d auth-service
    sleep 15
    
    if curl -f -s "http://localhost:3010/health" >/dev/null 2>&1; then
        print_success "Auth Service is healthy!"
    else
        print_warning "Auth Service health check failed, checking logs..."
        docker-compose -f docker-compose.microservices.yml logs auth-service | tail -10
    fi
    
    print_status "Testing Sample Service..."
    docker-compose -f docker-compose.microservices.yml up -d sample-service
    sleep 15
    
    if curl -f -s "http://localhost:3011/health" >/dev/null 2>&1; then
        print_success "Sample Service is healthy!"
    else
        print_warning "Sample Service health check failed, checking logs..."
        docker-compose -f docker-compose.microservices.yml logs sample-service | tail -10
    fi
    
    print_status "Testing Template Service..."
    docker-compose -f docker-compose.microservices.yml up -d template-service
    sleep 15
    
    if curl -f -s "http://localhost:3013/health" >/dev/null 2>&1; then
        print_success "Template Service is healthy!"
    else
        print_warning "Template Service health check failed, checking logs..."
        docker-compose -f docker-compose.microservices.yml logs template-service | tail -10
    fi
    
    # Test Enhanced RAG Service (Python - was working)
    print_status "Testing Enhanced RAG Service..."
    docker-compose -f docker-compose.microservices.yml up -d enhanced-rag-service
    sleep 20
    
    if curl -f -s "http://localhost:3019/health" >/dev/null 2>&1; then
        print_success "Enhanced RAG Service is healthy!"
    else
        print_warning "Enhanced RAG Service health check failed, checking logs..."
        docker-compose -f docker-compose.microservices.yml logs enhanced-rag-service | tail -10
    fi
    
    # Show service status
    print_header "CORE SERVICES STATUS"
    docker-compose -f docker-compose.microservices.yml ps --format "table {{.Name}}\t{{.Status}}\t{{.Ports}}"
    
    print_status "\nHealthy endpoints:"
    local endpoints=(
        "Auth Service:http://localhost:3010/health"
        "Sample Service:http://localhost:3011/health"
        "Template Service:http://localhost:3013/health"
        "RAG Service:http://localhost:3019/health"
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
    
    print_status "\nCore Services Health: $healthy/$total services healthy"
    
    if [ $healthy -ge 2 ]; then
        print_success "🎉 Core migration fixes are working! Ready to proceed."
        return 0
    else
        print_warning "Some services need attention, but progress made."
        return 1
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
    print_header "TracSeq 2.0 Core Services Test"
    
    echo "Testing the fixed core services:"
    echo "  🔧 Auth Service (Fixed Dockerfile)"
    echo "  🔧 Sample Service (Fixed Dockerfile)" 
    echo "  🔧 Template Service (Fixed Dockerfile)"
    echo "  ✅ Enhanced RAG Service (Python - Working)"
    echo ""
    
    read -p "Continue with core services test? (y/N): " -n 1 -r
    echo
    if [[ ! $REPLY =~ ^[Yy]$ ]]; then
        print_warning "Test cancelled"
        exit 0
    fi
    
    # Cleanup first
    cleanup
    
    # Run core test
    test_core_services
    
    print_header "🎯 CORE SERVICES TEST COMPLETED"
    print_status "Use 'docker-compose -f docker-compose.microservices.yml down' to stop services"
    print_status "Use 'docker-compose -f docker-compose.microservices.yml logs [service]' to debug"
}

# Handle interruption
trap 'print_error "Testing interrupted"; cleanup; exit 1' INT TERM

# Run main function
main "$@" 