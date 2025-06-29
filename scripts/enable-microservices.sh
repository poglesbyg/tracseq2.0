#!/bin/bash

# TracSeq 2.0 Microservices Enablement Script
# Progressively enable feature flags to migrate away from monolith

set -e  # Exit on any error

# Colors for output
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
RED='\033[0;31m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

print_info() { echo -e "${BLUE}[INFO]${NC} $1"; }
print_success() { echo -e "${GREEN}[SUCCESS]${NC} $1"; }
print_warning() { echo -e "${YELLOW}[WARNING]${NC} $1"; }
print_error() { echo -e "${RED}[ERROR]${NC} $1"; }

print_header() {
    echo -e "\n${BLUE}================================================================${NC}"
    echo -e "${BLUE}$1${NC}"
    echo -e "${BLUE}================================================================${NC}\n"
}

# Configuration
API_GATEWAY_URL="http://localhost:8089"
MONOLITH_URL="http://localhost:3000"

# Check if API Gateway is running
check_api_gateway() {
    print_info "Checking API Gateway status..."
    if curl -s "$API_GATEWAY_URL/health" > /dev/null 2>&1; then
        print_success "API Gateway is running on $API_GATEWAY_URL"
        return 0
    else
        print_error "API Gateway is not running on $API_GATEWAY_URL"
        print_info "Start it with: cd api_gateway && docker-compose -f docker-compose.minimal.yml up -d"
        return 1
    fi
}

# Check if monolith is running
check_monolith() {
    print_info "Checking monolith status..."
    if curl -s "$MONOLITH_URL/health" > /dev/null 2>&1; then
        print_warning "Monolith is still running on $MONOLITH_URL"
        return 0
    else
        print_info "Monolith is not running (this is expected after full migration)"
        return 1
    fi
}

# Test service health
test_service_health() {
    local service_name=$1
    local endpoint=$2
    local timeout=${3:-5}
    
    print_info "Testing $service_name health..."
    if timeout $timeout curl -s "$API_GATEWAY_URL$endpoint" > /dev/null 2>&1; then
        print_success "$service_name is healthy ✅"
        return 0
    else
        print_warning "$service_name is not responding ⚠️"
        return 1
    fi
}

# Get current routing status
show_routing_status() {
    print_header "Current Routing Status"
    
    if curl -s "$API_GATEWAY_URL/routing-status" | jq -e .feature_flags > /dev/null 2>&1; then
        echo "Current feature flags:"
        curl -s "$API_GATEWAY_URL/routing-status" | jq '.feature_flags'
        echo ""
        echo "Microservices status:"
        curl -s "$API_GATEWAY_URL/routing-status" | jq '.routing_configuration.microservices'
    else
        print_warning "Could not get routing status. API Gateway might not be running."
    fi
}

# Phase 1: Enable safe services
enable_safe_services() {
    print_header "PHASE 1: Enabling Safe Services (Low Risk)"
    
    cd api_gateway
    
    # Create .env file with safe service flags enabled
    print_info "Creating .env file with safe service flags..."
    cat > .env << 'EOF'
# TracSeq 2.0 Feature Flags - Safe Services Enabled
# Phase 1: Low-risk services that don't affect core business logic

# SAFE SERVICES (✅ Enable immediately)
USE_NOTIFICATION_SERVICE=true
USE_STORAGE_SERVICE=true
USE_RAG_SERVICE=true

# BUSINESS LOGIC SERVICES (⚠️ Enable with caution)
USE_TEMPLATE_SERVICE=false
USE_AUTH_SERVICE=false
USE_SAMPLE_SERVICE=false
USE_SEQUENCING_SERVICE=false

# API Gateway Configuration
ENVIRONMENT=development
HOST=0.0.0.0
PORT=8000

# Monolith Configuration (fallback)
MONOLITH__HOST=host.docker.internal
MONOLITH__PORT=3000

# CORS Configuration
CORS__ENABLED=true
CORS__ALLOW_ORIGINS=["http://localhost:3000","http://localhost:5173","http://localhost:8000"]

# Monitoring
MONITORING__METRICS_ENABLED=true
MONITORING__LOG_REQUESTS=true
EOF

    print_success "Created .env file with safe services enabled"
    
    # Restart API Gateway to pick up new configuration
    print_info "Restarting API Gateway with new configuration..."
    docker-compose -f docker-compose.minimal.yml restart api-gateway
    
    # Wait for restart
    sleep 5
    
    # Test the services
    print_info "Testing enabled services..."
    test_service_health "Notification Service" "/api/notifications" 
    test_service_health "Storage Service" "/api/storage/locations"
    test_service_health "RAG Service" "/api/rag/health"
    
    cd ..
}

# Phase 2: Enable template service
enable_template_service() {
    print_header "PHASE 2: Enabling Template Service (Medium Risk)"
    
    print_warning "This will enable template management via microservice"
    read -p "Continue? (y/N): " -n 1 -r
    echo
    if [[ ! $REPLY =~ ^[Yy]$ ]]; then
        print_info "Skipping template service enablement"
        return 0
    fi
    
    cd api_gateway
    
    # Update .env to enable template service
    sed -i.bak 's/USE_TEMPLATE_SERVICE=false/USE_TEMPLATE_SERVICE=true/' .env
    
    print_info "Restarting API Gateway with template service enabled..."
    docker-compose -f docker-compose.minimal.yml restart api-gateway
    
    sleep 5
    
    # Test template service
    test_service_health "Template Service" "/api/templates"
    
    cd ..
}

# Phase 3: Enable auth service
enable_auth_service() {
    print_header "PHASE 3: Enabling Auth Service (HIGH RISK)"
    
    print_error "⚠️  WARNING: This affects all user authentication!"
    print_warning "Make sure you have:"
    print_warning "  - Migrated user data to auth service database"
    print_warning "  - Tested JWT token compatibility"
    print_warning "  - Prepared rollback plan"
    print_warning "  - Notified users of potential brief downtime"
    
    read -p "Are you sure you want to enable auth service? (y/N): " -n 1 -r
    echo
    if [[ ! $REPLY =~ ^[Yy]$ ]]; then
        print_info "Skipping auth service enablement"
        return 0
    fi
    
    cd api_gateway
    
    # Update .env to enable auth service
    sed -i.bak 's/USE_AUTH_SERVICE=false/USE_AUTH_SERVICE=true/' .env
    
    print_info "Restarting API Gateway with auth service enabled..."
    docker-compose -f docker-compose.minimal.yml restart api-gateway
    
    sleep 5
    
    # Test auth service
    test_service_health "Auth Service" "/api/auth/health"
    
    cd ..
}

# Phase 4: Enable core business services
enable_core_services() {
    print_header "PHASE 4: Enabling Core Business Services (HIGH RISK)"
    
    print_error "⚠️  WARNING: This affects core laboratory functionality!"
    print_warning "This will enable:"
    print_warning "  - Sample Service (sample management)"
    print_warning "  - Sequencing Service (sequencing workflows)"
    
    read -p "Continue with core services enablement? (y/N): " -n 1 -r
    echo
    if [[ ! $REPLY =~ ^[Yy]$ ]]; then
        print_info "Skipping core services enablement"
        return 0
    fi
    
    cd api_gateway
    
    # Update .env to enable core services
    sed -i.bak 's/USE_SAMPLE_SERVICE=false/USE_SAMPLE_SERVICE=true/' .env
    sed -i.bak 's/USE_SEQUENCING_SERVICE=false/USE_SEQUENCING_SERVICE=true/' .env
    
    print_info "Restarting API Gateway with core services enabled..."
    docker-compose -f docker-compose.minimal.yml restart api-gateway
    
    sleep 5
    
    # Test core services
    test_service_health "Sample Service" "/api/samples"
    test_service_health "Sequencing Service" "/api/sequencing/jobs"
    
    cd ..
}

# Start all microservices
start_all_microservices() {
    print_header "Starting All Microservices"
    
    print_info "Starting all microservices in background..."
    
    # Start enhanced microservices
    if [ -f "docker-compose.yml" ]; then
        docker-compose up -d
    elif [ -f "old_ymls/docker-compose.complete-microservices.yml" ]; then
        docker-compose -f old_ymls/docker-compose.complete-microservices.yml up -d
    else
        print_warning "No complete microservices compose file found"
        print_info "Starting individual services..."
        
        # Start individual services
        for service_dir in auth_service sample_service enhanced_storage_service template_service sequencing_service notification_service; do
            if [ -d "$service_dir" ]; then
                print_info "Starting $service_dir..."
                cd "$service_dir"
                if [ -f "docker-compose.yml" ]; then
                    docker-compose up -d
                fi
                cd ..
            fi
        done
    fi
    
    print_success "All microservices started"
}

# Rollback function
rollback() {
    print_header "ROLLBACK: Disabling All Microservices"
    
    cd api_gateway
    
    # Reset .env to disable all services
    cat > .env << 'EOF'
# TracSeq 2.0 Feature Flags - ROLLBACK MODE
# All services disabled, routing to monolith

USE_NOTIFICATION_SERVICE=false
USE_STORAGE_SERVICE=false
USE_RAG_SERVICE=false
USE_TEMPLATE_SERVICE=false
USE_AUTH_SERVICE=false
USE_SAMPLE_SERVICE=false
USE_SEQUENCING_SERVICE=false

# Ensure monolith routing is active
MONOLITH__HOST=host.docker.internal
MONOLITH__PORT=3000

ENVIRONMENT=development
HOST=0.0.0.0
PORT=8000
CORS__ENABLED=true
EOF

    print_info "Restarting API Gateway in monolith mode..."
    docker-compose -f docker-compose.minimal.yml restart api-gateway
    
    sleep 5
    
    print_success "Rollback complete - all traffic routing to monolith"
    
    cd ..
}

# Monitor migration progress
monitor_migration() {
    print_header "Migration Progress Monitor"
    
    while true; do
        clear
        echo -e "${BLUE}TracSeq 2.0 Migration Dashboard${NC}"
        echo "=================================="
        echo ""
        
        # Check API Gateway
        if curl -s "$API_GATEWAY_URL/health" > /dev/null 2>&1; then
            echo -e "API Gateway: ${GREEN}✅ Running${NC}"
        else
            echo -e "API Gateway: ${RED}❌ Down${NC}"
        fi
        
        # Check monolith
        if curl -s "$MONOLITH_URL/health" > /dev/null 2>&1; then
            echo -e "Monolith: ${YELLOW}⚠️  Still Running${NC}"
        else
            echo -e "Monolith: ${GREEN}✅ Stopped (Migration Complete)${NC}"
        fi
        
        echo ""
        echo "Current routing configuration:"
        
        if curl -s "$API_GATEWAY_URL/routing-status" | jq -e .feature_flags > /dev/null 2>&1; then
            curl -s "$API_GATEWAY_URL/routing-status" | jq '.feature_flags' | while read -r line; do
                if [[ $line == *"true"* ]]; then
                    echo -e "  ${GREEN}$line${NC}"
                else
                    echo -e "  ${YELLOW}$line${NC}"
                fi
            done
        else
            echo "  Could not get routing status"
        fi
        
        echo ""
        echo "Press Ctrl+C to exit monitoring"
        sleep 10
    done
}

# Main menu
show_menu() {
    print_header "TracSeq 2.0 Monolith Elimination Tool"
    
    echo "Choose migration phase:"
    echo ""
    echo "📊 Status & Information:"
    echo "  1) Show current routing status"
    echo "  2) Check service health"
    echo "  3) Monitor migration progress (live)"
    echo ""
    echo "🚀 Migration Phases:"
    echo "  4) Phase 1: Enable safe services (notification, storage, RAG)"
    echo "  5) Phase 2: Enable template service"
    echo "  6) Phase 3: Enable auth service (HIGH RISK)"
    echo "  7) Phase 4: Enable core services (samples, sequencing) (HIGH RISK)"
    echo ""
    echo "🛠️  Infrastructure:"
    echo "  8) Start all microservices"
    echo "  9) Rollback to monolith (disable all flags)"
    echo ""
    echo "  0) Exit"
    echo ""
}

# Main execution
main() {
    # Check prerequisites
    if ! command -v jq &> /dev/null; then
        print_error "jq is required but not installed. Install with: brew install jq (macOS) or apt-get install jq (Ubuntu)"
        exit 1
    fi
    
    if ! command -v curl &> /dev/null; then
        print_error "curl is required but not installed"
        exit 1
    fi
    
    while true; do
        show_menu
        read -p "Enter your choice [0-9]: " choice
        echo ""
        
        case $choice in
            1) show_routing_status ;;
            2) 
                check_api_gateway
                check_monolith
                ;;
            3) monitor_migration ;;
            4) 
                check_api_gateway && enable_safe_services
                ;;
            5) 
                check_api_gateway && enable_template_service
                ;;
            6) 
                check_api_gateway && enable_auth_service
                ;;
            7) 
                check_api_gateway && enable_core_services
                ;;
            8) start_all_microservices ;;
            9) rollback ;;
            0) 
                print_info "Goodbye! 👋"
                exit 0
                ;;
            *) 
                print_error "Invalid option. Please choose 0-9."
                ;;
        esac
        
        echo ""
        read -p "Press Enter to continue..."
    done
}

# Run main function if script is executed directly
if [[ "${BASH_SOURCE[0]}" == "${0}" ]]; then
    main "$@"
fi 