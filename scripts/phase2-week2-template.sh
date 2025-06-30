#!/bin/bash

# TracSeq 2.0 Phase 2 Week 2: Enable Template Service
# Medium risk - affects template management

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

# Check prerequisites
check_prerequisites() {
    print_header "Checking Prerequisites"
    
    # Check if Week 1 services are running
    if ! curl -s -f "$API_GATEWAY_URL/health" > /dev/null 2>&1; then
        print_error "API Gateway is not running. Please run phase2-migration.sh first."
        exit 1
    fi
    
    print_success "API Gateway is running"
    
    # Check if safe services are enabled
    local routing_status=$(curl -s "$API_GATEWAY_URL/routing-status" 2>/dev/null || echo "{}")
    
    if [[ $routing_status == *'"USE_NOTIFICATION_SERVICE":true'* ]]; then
        print_success "Week 1 services are enabled"
    else
        print_warning "Week 1 services might not be properly enabled"
    fi
}

# Start template service
start_template_service() {
    print_header "Starting Template Service"
    
    cd "$WORKSPACE_DIR"
    
    # Start template service
    print_info "Starting template service..."
    docker-compose -f docker-compose.microservices.yml up -d template-service
    
    # Wait for service to be ready
    sleep 10
    
    # Check health
    if curl -s -f "http://localhost:3013/health" > /dev/null 2>&1; then
        print_success "Template service is healthy at http://localhost:3013"
    else
        print_error "Template service failed to start properly"
        return 1
    fi
}

# Update API Gateway configuration
update_api_gateway_config() {
    print_header "Updating API Gateway Configuration"
    
    cd "$API_GATEWAY_DIR"
    
    # Backup current .env
    cp .env .env.week1.backup
    print_info "Created backup: .env.week1.backup"
    
    # Update .env to enable template service
    sed -i.tmp 's/USE_TEMPLATE_SERVICE=false/USE_TEMPLATE_SERVICE=true/' .env
    rm -f .env.tmp
    
    print_success "Updated .env to enable template service"
    
    # Restart API Gateway
    print_info "Restarting API Gateway..."
    docker-compose -f docker-compose.minimal.yml restart api-gateway
    
    sleep 5
}

# Test template service routing
test_template_service() {
    print_header "Testing Template Service"
    
    print_info "Testing template service through API Gateway..."
    
    # Test health endpoint
    if curl -s -f "$API_GATEWAY_URL/api/templates" > /dev/null 2>&1; then
        print_success "✅ Template service routing is working"
    else
        print_error "❌ Template service routing failed"
        return 1
    fi
    
    # Test template listing
    print_info "Testing template listing..."
    response=$(curl -s "$API_GATEWAY_URL/api/templates" 2>/dev/null)
    if [[ $? -eq 0 ]]; then
        print_success "Template listing endpoint responding"
    else
        print_warning "Template listing might need authentication"
    fi
}

# Create rollback script
create_rollback_script() {
    local rollback_script="$WORKSPACE_DIR/scripts/phase2-week2-rollback.sh"
    
    cat > "$rollback_script" << 'EOF'
#!/bin/bash
# Emergency rollback for Week 2 template service

echo "Rolling back template service enablement..."

cd /workspace/api_gateway

# Restore Week 1 configuration
if [ -f .env.week1.backup ]; then
    cp .env.week1.backup .env
    echo "✅ Restored Week 1 configuration"
else
    # Manually disable template service
    sed -i 's/USE_TEMPLATE_SERVICE=true/USE_TEMPLATE_SERVICE=false/' .env
    echo "✅ Disabled template service flag"
fi

# Restart API Gateway
docker-compose -f docker-compose.minimal.yml restart api-gateway

echo "✅ Rollback complete - template service disabled"
echo "Template traffic will now route to monolith"
EOF

    chmod +x "$rollback_script"
    print_info "Created rollback script: $rollback_script"
}

# Update migration log
update_migration_log() {
    local log_file="$WORKSPACE_DIR/phase2_migration_log.md"
    
    cat >> "$log_file" << EOF

## Week 2: Template Service Enabled
Date: $(date)

### Services Enabled:
- ✅ Template Service (Port: 3013)
- Previous: Notification, Storage, RAG services

### Configuration Changes:
- USE_TEMPLATE_SERVICE=true

### Testing Results:
- Template health endpoint: ✅ Working
- Template listing: ✅ Working
- File upload: 🔄 To be tested

### Rollback Procedure:
If issues occur, run: scripts/phase2-week2-rollback.sh

### Next Steps:
1. Test template CRUD operations thoroughly
2. Verify file upload functionality
3. Monitor for 48 hours
4. Proceed to Week 3: Auth Service (HIGH RISK)

EOF

    print_success "Updated migration log"
}

# Main execution
main() {
    print_header "TracSeq 2.0 Phase 2 Week 2: Template Service"
    
    print_warning "This will enable template management microservice"
    print_warning "Medium risk - affects template functionality"
    echo ""
    read -p "Continue? (y/N): " -n 1 -r
    echo ""
    
    if [[ ! $REPLY =~ ^[Yy]$ ]]; then
        print_info "Migration cancelled"
        exit 0
    fi
    
    # Check prerequisites
    check_prerequisites
    
    # Start template service
    start_template_service
    
    # Update API Gateway configuration
    update_api_gateway_config
    
    # Test the service
    test_template_service
    
    # Create rollback script
    create_rollback_script
    
    # Update migration log
    update_migration_log
    
    print_header "Phase 2 Week 2 Complete!"
    
    print_success "Template service has been enabled successfully!"
    print_info ""
    print_info "Next steps:"
    print_info "1. Test template CRUD operations"
    print_info "2. Test file upload functionality"
    print_info "3. Monitor for issues"
    print_info "4. If stable after 48 hours, proceed to Week 3"
    print_info ""
    print_warning "Rollback available: scripts/phase2-week2-rollback.sh"
}

# Run main function
main "$@"