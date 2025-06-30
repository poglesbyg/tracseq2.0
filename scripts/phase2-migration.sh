#!/bin/bash

# TracSeq 2.0 Phase 2 Migration Script
# Progressive microservices enablement

set -e

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
WORKSPACE_DIR="/workspace"
API_GATEWAY_DIR="$WORKSPACE_DIR/api_gateway"
API_GATEWAY_URL="http://localhost:8089"

# Function to check service health
check_service_health() {
    local service_name=$1
    local url=$2
    local max_retries=10
    local retry_count=0
    
    while [ $retry_count -lt $max_retries ]; do
        if curl -s -f "$url" > /dev/null 2>&1; then
            print_success "$service_name is healthy at $url"
            return 0
        fi
        retry_count=$((retry_count + 1))
        print_info "Waiting for $service_name... ($retry_count/$max_retries)"
        sleep 3
    done
    
    print_error "$service_name failed to respond at $url"
    return 1
}

# Phase 2 Week 1: Start safe microservices
start_safe_microservices() {
    print_header "Phase 2 Week 1: Starting Safe Microservices"
    
    cd "$WORKSPACE_DIR"
    
    # Start the microservices
    print_info "Starting microservices infrastructure..."
    
    # First start the databases
    docker-compose -f docker-compose.microservices.yml up -d postgres redis
    sleep 10
    
    # Start the safe services
    print_info "Starting notification service..."
    docker-compose -f docker-compose.microservices.yml up -d notification-service
    
    print_info "Starting storage service..."
    docker-compose -f docker-compose.microservices.yml up -d enhanced-storage-service
    
    print_info "Starting RAG service..."
    docker-compose -f docker-compose.microservices.yml up -d enhanced-rag-service
    
    # Wait for services to be ready
    sleep 10
    
    # Check health of services
    check_service_health "Notification Service" "http://localhost:3016/health"
    check_service_health "Storage Service" "http://localhost:3014/health"
    check_service_health "RAG Service" "http://localhost:3019/health"
}

# Start API Gateway
start_api_gateway() {
    print_header "Starting API Gateway"
    
    cd "$API_GATEWAY_DIR"
    
    # Start the API Gateway
    print_info "Starting API Gateway with safe services enabled..."
    docker-compose -f docker-compose.minimal.yml up -d
    
    # Wait for API Gateway to be ready
    sleep 5
    
    check_service_health "API Gateway" "$API_GATEWAY_URL/health"
}

# Test enabled services through API Gateway
test_enabled_services() {
    print_header "Testing Enabled Services Through API Gateway"
    
    # Test notification service
    print_info "Testing notification service..."
    if curl -s -f "$API_GATEWAY_URL/api/notifications" > /dev/null 2>&1; then
        print_success "✅ Notification service routing working"
    else
        print_warning "⚠️ Notification service routing may not be configured"
    fi
    
    # Test storage service
    print_info "Testing storage service..."
    if curl -s -f "$API_GATEWAY_URL/api/storage/locations" > /dev/null 2>&1; then
        print_success "✅ Storage service routing working"
    else
        print_warning "⚠️ Storage service routing may not be configured"
    fi
    
    # Test RAG service
    print_info "Testing RAG service..."
    if curl -s -f "$API_GATEWAY_URL/api/rag/health" > /dev/null 2>&1; then
        print_success "✅ RAG service routing working"
    else
        print_warning "⚠️ RAG service routing may not be configured"
    fi
}

# Show routing status
show_routing_status() {
    print_header "Current Routing Status"
    
    print_info "Fetching routing status from API Gateway..."
    
    # Try to get routing status
    response=$(curl -s "$API_GATEWAY_URL/routing-status" 2>/dev/null || echo "{}")
    
    if [ "$response" != "{}" ]; then
        echo "$response" | python3 -m json.tool 2>/dev/null || echo "$response"
    else
        print_warning "Could not fetch routing status. API Gateway might not be fully configured."
    fi
}

# Create migration log
create_migration_log() {
    local log_file="$WORKSPACE_DIR/phase2_migration_log.md"
    
    cat > "$log_file" << EOF
# Phase 2 Migration Log

## Week 1: Safe Services Enabled
Date: $(date)

### Services Enabled:
- ✅ Notification Service (Port: 3016)
- ✅ Storage Service (Port: 3014)  
- ✅ RAG Service (Port: 3019)

### API Gateway Configuration:
- URL: http://localhost:8089
- Feature Flags:
  - USE_NOTIFICATION_SERVICE=true
  - USE_STORAGE_SERVICE=true
  - USE_RAG_SERVICE=true

### Next Steps:
1. Monitor services for 24-48 hours
2. Verify no degradation in functionality
3. Proceed to Week 2: Enable Template Service

### Commands for verification:
\`\`\`bash
# Check service health
curl http://localhost:8089/health

# Check routing status
curl http://localhost:8089/routing-status

# Test notification service
curl http://localhost:8089/api/notifications

# Test storage service
curl http://localhost:8089/api/storage/locations

# Test RAG service
curl http://localhost:8089/api/rag/health
\`\`\`
EOF

    print_success "Migration log created at: $log_file"
}

# Main execution
main() {
    print_header "TracSeq 2.0 Phase 2 Migration - Week 1"
    
    print_info "This script will enable safe microservices:"
    print_info "  - Notification Service"
    print_info "  - Storage Service"
    print_info "  - RAG Service"
    echo ""
    
    # Start microservices
    start_safe_microservices
    
    # Start API Gateway
    start_api_gateway
    
    # Test services
    test_enabled_services
    
    # Show routing status
    show_routing_status
    
    # Create migration log
    create_migration_log
    
    print_header "Phase 2 Week 1 Complete!"
    
    print_success "Safe services have been enabled successfully!"
    print_info ""
    print_info "Next steps:"
    print_info "1. Monitor services for 24-48 hours"
    print_info "2. Check logs: docker-compose -f docker-compose.microservices.yml logs -f"
    print_info "3. Verify functionality through frontend"
    print_info "4. Once stable, proceed to Week 2 (Template Service)"
    print_info ""
    print_info "To view service status: curl $API_GATEWAY_URL/routing-status"
}

# Run main function
main "$@"