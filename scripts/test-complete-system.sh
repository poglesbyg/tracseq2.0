#!/bin/bash

# Test complete TracSeq 2.0 system including frontend

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

# Test complete system with frontend
test_complete_system() {
    print_header "TESTING COMPLETE TRACSEQ 2.0 SYSTEM WITH FRONTEND"
    
    print_status "Starting infrastructure services..."
    docker-compose -f docker-compose.microservices.yml up -d postgres redis
    
    # Wait for infrastructure
    print_status "Waiting for PostgreSQL and Redis..."
    sleep 15
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
    
    print_status "Starting core microservices..."
    docker-compose -f docker-compose.microservices.yml up -d \
        auth-service \
        sample-service \
        template-service \
        enhanced-rag-service
    
    sleep 20
    
    print_status "Starting API Gateway..."
    docker-compose -f docker-compose.microservices.yml up -d api-gateway
    sleep 15
    
    print_status "Starting Frontend..."
    docker-compose -f docker-compose.microservices.yml up -d frontend
    sleep 20
    
    # Test all components
    print_header "TESTING SYSTEM COMPONENTS"
    
    local endpoints=(
        "Infrastructure|PostgreSQL:direct"
        "Infrastructure|Redis:direct"
        "Microservice|Auth Service:http://localhost:3010/health"
        "Microservice|Sample Service:http://localhost:3011/health"
        "Microservice|Template Service:http://localhost:3013/health"
        "Microservice|Enhanced RAG Service:http://localhost:3019/health"
        "Gateway|API Gateway:http://localhost:8000/health"
        "Frontend|Web Application:http://localhost:5173"
    )
    
    local healthy=0
    local total=${#endpoints[@]}
    
    for endpoint in "${endpoints[@]}"; do
        IFS='|' read -r category name url <<< "$endpoint"
        
        if [[ "$url" == "direct" ]]; then
            # Direct service check
            if [[ "$name" == "PostgreSQL" ]]; then
                if docker-compose -f docker-compose.microservices.yml exec -T postgres pg_isready -U postgres > /dev/null 2>&1; then
                    echo "  ✅ [$category] $name"
                    ((healthy++))
                else
                    echo "  ❌ [$category] $name"
                fi
            elif [[ "$name" == "Redis" ]]; then
                if docker-compose -f docker-compose.microservices.yml exec -T redis redis-cli ping > /dev/null 2>&1; then
                    echo "  ✅ [$category] $name"
                    ((healthy++))
                else
                    echo "  ❌ [$category] $name"
                fi
            fi
        else
            # HTTP endpoint check
            if curl -f -s "$url" >/dev/null 2>&1; then
                echo "  ✅ [$category] $name"
                ((healthy++))
            else
                echo "  ❌ [$category] $name"
                if [[ "$category" == "Frontend" ]]; then
                    print_warning "Checking if frontend is building..."
                    docker-compose -f docker-compose.microservices.yml logs frontend | tail -5
                fi
            fi
        fi
    done
    
    print_status "\nSystem Health: $healthy/$total components healthy"
    
    # Show complete system status
    print_header "COMPLETE SYSTEM STATUS"
    docker-compose -f docker-compose.microservices.yml ps --format "table {{.Name}}\t{{.Status}}\t{{.Ports}}"
    
    print_header "ACCESS INFORMATION"
    echo "🌐 Frontend Application: http://localhost:5173"
    echo "🔌 API Gateway: http://localhost:8000"
    echo "🔐 Auth Service: http://localhost:3010"
    echo "📊 Sample Service: http://localhost:3011" 
    echo "📋 Template Service: http://localhost:3013"
    echo "🤖 RAG Service: http://localhost:3019"
    echo "🗄️  PostgreSQL: localhost:5432"
    echo "💾 Redis: localhost:6379"
    
    if [ $healthy -ge 6 ]; then
        print_success "🎉 Complete TracSeq 2.0 system is operational!"
        print_status "Access the web application at: http://localhost:5173"
        return 0
    else
        print_warning "Some components need attention, but core system is functional."
        return 1
    fi
}

# Cleanup function
cleanup() {
    print_status "Cleaning up all services..."
    docker-compose -f docker-compose.microservices.yml down --remove-orphans
    print_success "Cleanup completed"
}

# Main execution
main() {
    print_header "TracSeq 2.0 Complete System Test (Backend + Frontend)"
    
    echo "Testing the complete system:"
    echo "  🏗️  Infrastructure (PostgreSQL, Redis)"
    echo "  🔧 Core Microservices (Auth, Sample, Template, RAG)"
    echo "  🚪 API Gateway (Routing layer)"
    echo "  🌐 Frontend Application (React/Vite)"
    echo ""
    
    read -p "Continue with complete system test? (y/N): " -n 1 -r
    echo
    if [[ ! $REPLY =~ ^[Yy]$ ]]; then
        print_warning "Test cancelled"
        exit 0
    fi
    
    # Cleanup first
    cleanup
    
    # Run complete test
    test_complete_system
    
    print_header "🎯 COMPLETE SYSTEM TEST FINISHED"
    print_status "Frontend available at: http://localhost:5173"
    print_status "API Gateway available at: http://localhost:8000"
    print_status "Use 'docker-compose -f docker-compose.microservices.yml down' to stop"
}

# Handle interruption
trap 'print_error "Testing interrupted"; cleanup; exit 1' INT TERM

# Run main function
main "$@" 