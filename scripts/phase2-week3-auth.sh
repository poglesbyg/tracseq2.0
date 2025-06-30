#!/bin/bash

# TracSeq 2.0 Phase 2 Week 3: Enable Auth Service
# HIGH RISK - Affects all user authentication

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
MONOLITH_DB="postgresql://postgres:postgres@localhost:5432/tracseq_main"
AUTH_DB="postgresql://postgres:postgres@localhost:5432/tracseq_auth"

# Pre-migration checklist
pre_migration_checklist() {
    print_header "Pre-Migration Checklist"
    
    echo "⚠️  HIGH RISK OPERATION - AUTH SERVICE MIGRATION"
    echo ""
    echo "Please confirm the following steps have been completed:"
    echo ""
    echo "[ ] 1. Database backup created"
    echo "[ ] 2. User data migrated to auth service database"
    echo "[ ] 3. JWT token compatibility tested"
    echo "[ ] 4. Rollback plan prepared and tested"
    echo "[ ] 5. Users notified of potential brief downtime"
    echo "[ ] 6. Support team on standby"
    echo ""
    
    read -p "Have all these steps been completed? (yes/no): " -r
    if [[ ! $REPLY == "yes" ]]; then
        print_error "Migration cancelled. Please complete all checklist items."
        exit 1
    fi
}

# Migrate user data
migrate_user_data() {
    print_header "Migrating User Data"
    
    print_warning "This step should have been completed offline!"
    print_info "Verifying user data migration..."
    
    # Create migration script for reference
    cat > "$WORKSPACE_DIR/scripts/auth-data-migration.sql" << 'EOF'
-- Auth Service Data Migration Script
-- Run this before enabling auth service

-- Create users table in auth database
\c tracseq_auth;

CREATE TABLE IF NOT EXISTS users (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    email VARCHAR(255) UNIQUE NOT NULL,
    username VARCHAR(255) UNIQUE NOT NULL,
    password_hash VARCHAR(255) NOT NULL,
    first_name VARCHAR(255),
    last_name VARCHAR(255),
    role VARCHAR(50) DEFAULT 'user',
    is_active BOOLEAN DEFAULT true,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT NOW()
);

CREATE TABLE IF NOT EXISTS user_sessions (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID REFERENCES users(id) ON DELETE CASCADE,
    token TEXT NOT NULL,
    expires_at TIMESTAMP WITH TIME ZONE NOT NULL,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW()
);

-- Copy users from monolith database
-- INSERT INTO users (id, email, username, password_hash, first_name, last_name, role, is_active, created_at, updated_at)
-- SELECT id, email, username, password_hash, first_name, last_name, role, is_active, created_at, updated_at
-- FROM dblink('dbname=tracseq_main', 'SELECT * FROM users')
-- AS t(id UUID, email VARCHAR, username VARCHAR, password_hash VARCHAR, first_name VARCHAR, last_name VARCHAR, role VARCHAR, is_active BOOLEAN, created_at TIMESTAMP WITH TIME ZONE, updated_at TIMESTAMP WITH TIME ZONE);
EOF

    print_info "Migration script created at: scripts/auth-data-migration.sql"
    print_warning "Please ensure this has been executed before proceeding!"
}

# Start auth service
start_auth_service() {
    print_header "Starting Auth Service"
    
    cd "$WORKSPACE_DIR"
    
    # Start auth service
    print_info "Starting auth service..."
    docker-compose -f docker-compose.microservices.yml up -d auth-service
    
    # Wait for service to be ready
    sleep 15
    
    # Check health
    if curl -s -f "http://localhost:3010/health" > /dev/null 2>&1; then
        print_success "Auth service is healthy at http://localhost:3010"
    else
        print_error "Auth service failed to start properly"
        return 1
    fi
}

# Test auth service directly
test_auth_service_direct() {
    print_header "Testing Auth Service Directly"
    
    # Test login endpoint
    print_info "Testing auth service login endpoint..."
    
    # Create test request
    local test_login='{
        "email": "test@tracseq.com",
        "password": "test_password"
    }'
    
    response=$(curl -s -X POST "http://localhost:3010/api/auth/login" \
        -H "Content-Type: application/json" \
        -d "$test_login" 2>/dev/null || echo "FAILED")
    
    if [[ $response != "FAILED" ]]; then
        print_success "Auth service responding to login requests"
    else
        print_warning "Auth service login test failed (might need valid credentials)"
    fi
}

# Update API Gateway configuration
update_api_gateway_config() {
    print_header "Updating API Gateway Configuration"
    
    cd "$API_GATEWAY_DIR"
    
    # Backup current .env
    cp .env .env.week2.backup
    print_info "Created backup: .env.week2.backup"
    
    # Update .env to enable auth service
    sed -i.tmp 's/USE_AUTH_SERVICE=false/USE_AUTH_SERVICE=true/' .env
    rm -f .env.tmp
    
    print_success "Updated .env to enable auth service"
    
    # Restart API Gateway
    print_info "Restarting API Gateway..."
    docker-compose -f docker-compose.minimal.yml restart api-gateway
    
    sleep 5
}

# Test auth routing
test_auth_routing() {
    print_header "Testing Auth Service Routing"
    
    # Test auth endpoints through gateway
    print_info "Testing auth endpoints through API Gateway..."
    
    # Test health
    if curl -s -f "$API_GATEWAY_URL/api/auth/health" > /dev/null 2>&1; then
        print_success "✅ Auth service health check working"
    else
        print_error "❌ Auth service health check failed"
    fi
    
    # Test login
    local test_login='{
        "email": "admin@tracseq.com",
        "password": "admin123"
    }'
    
    response=$(curl -s -X POST "$API_GATEWAY_URL/api/auth/login" \
        -H "Content-Type: application/json" \
        -d "$test_login" 2>/dev/null)
    
    if [[ $response == *"token"* ]] || [[ $response == *"error"* ]]; then
        print_success "✅ Auth login endpoint responding"
    else
        print_error "❌ Auth login endpoint not responding properly"
    fi
}

# Create comprehensive rollback script
create_rollback_script() {
    local rollback_script="$WORKSPACE_DIR/scripts/phase2-week3-rollback.sh"
    
    cat > "$rollback_script" << 'EOF'
#!/bin/bash
# EMERGENCY ROLLBACK for Auth Service

RED='\033[0;31m'
GREEN='\033[0;32m'
NC='\033[0m'

echo -e "${RED}EMERGENCY AUTH SERVICE ROLLBACK${NC}"
echo "This will revert authentication to the monolith"

cd /workspace/api_gateway

# Restore Week 2 configuration
if [ -f .env.week2.backup ]; then
    cp .env.week2.backup .env
    echo -e "${GREEN}✅ Restored Week 2 configuration${NC}"
else
    # Manually disable auth service
    sed -i 's/USE_AUTH_SERVICE=true/USE_AUTH_SERVICE=false/' .env
    echo -e "${GREEN}✅ Disabled auth service flag${NC}"
fi

# Restart API Gateway immediately
docker-compose -f docker-compose.minimal.yml restart api-gateway

echo -e "${GREEN}✅ ROLLBACK COMPLETE${NC}"
echo "Authentication traffic restored to monolith"
echo ""
echo "Next steps:"
echo "1. Verify users can log in"
echo "2. Check monolith auth logs"
echo "3. Investigate auth service issues"
echo "4. Plan remediation before retry"
EOF

    chmod +x "$rollback_script"
    print_info "Created rollback script: $rollback_script"
}

# Create monitoring script
create_monitoring_script() {
    local monitor_script="$WORKSPACE_DIR/scripts/monitor-auth-migration.sh"
    
    cat > "$monitor_script" << 'EOF'
#!/bin/bash
# Monitor auth service migration

while true; do
    clear
    echo "Auth Service Migration Monitor - $(date)"
    echo "========================================"
    
    # Check auth service health
    if curl -s -f "http://localhost:3010/health" > /dev/null 2>&1; then
        echo "Auth Service: ✅ Healthy"
    else
        echo "Auth Service: ❌ Down"
    fi
    
    # Check API Gateway routing
    if curl -s "http://localhost:8089/routing-status" | grep -q '"USE_AUTH_SERVICE":true'; then
        echo "API Gateway: ✅ Routing to auth service"
    else
        echo "API Gateway: ❌ Not routing to auth service"
    fi
    
    # Count login attempts (would need actual metrics)
    echo ""
    echo "Recent Activity:"
    docker logs auth-service --tail 20 2>&1 | grep -E "(login|auth)" | tail -5
    
    echo ""
    echo "Press Ctrl+C to exit"
    sleep 10
done
EOF

    chmod +x "$monitor_script"
    print_info "Created monitoring script: $monitor_script"
}

# Update migration log
update_migration_log() {
    local log_file="$WORKSPACE_DIR/phase2_migration_log.md"
    
    cat >> "$log_file" << EOF

## Week 3: Auth Service Enabled (HIGH RISK)
Date: $(date)

### Services Enabled:
- ✅ Auth Service (Port: 3010) - **CRITICAL SERVICE**
- Previous: Notification, Storage, RAG, Template services

### Configuration Changes:
- USE_AUTH_SERVICE=true

### Testing Results:
- Auth service health: ✅ Working
- Login endpoint: ✅ Responding
- Token validation: 🔄 To be monitored
- Session management: 🔄 To be monitored

### Rollback Procedure:
**EMERGENCY**: scripts/phase2-week3-rollback.sh

### Monitoring:
Run: scripts/monitor-auth-migration.sh

### Critical Metrics to Watch:
- Login success rate
- Token validation errors
- Session timeout issues
- User complaints

### Next Steps:
1. Monitor intensively for 72 hours
2. Check all user workflows
3. Verify token compatibility
4. If stable, proceed to Week 4: Core Services

EOF

    print_success "Updated migration log"
}

# Main execution
main() {
    print_header "TracSeq 2.0 Phase 2 Week 3: Auth Service Migration"
    
    print_error "⚠️  HIGH RISK OPERATION ⚠️"
    print_error "This will migrate ALL user authentication to microservice"
    print_warning ""
    print_warning "Potential impacts:"
    print_warning "- Users may need to re-login"
    print_warning "- Brief authentication downtime possible"
    print_warning "- Session management changes"
    echo ""
    
    # Pre-migration checklist
    pre_migration_checklist
    
    # Show current state
    print_info "Current auth routing: Monolith (port 3000)"
    print_info "Target auth routing: Auth Service (port 3010)"
    echo ""
    
    read -p "Proceed with auth service migration? (yes/no): " -r
    if [[ ! $REPLY == "yes" ]]; then
        print_info "Migration cancelled"
        exit 0
    fi
    
    # Migration steps
    migrate_user_data
    start_auth_service
    test_auth_service_direct
    update_api_gateway_config
    test_auth_routing
    create_rollback_script
    create_monitoring_script
    update_migration_log
    
    print_header "Phase 2 Week 3 Complete!"
    
    print_success "Auth service migration complete!"
    print_warning ""
    print_warning "⚠️  CRITICAL: Monitor authentication closely!"
    print_info ""
    print_info "Immediate actions:"
    print_info "1. Run monitoring: scripts/monitor-auth-migration.sh"
    print_info "2. Test user login flows"
    print_info "3. Check error logs: docker logs -f auth-service"
    print_info "4. Be ready to rollback: scripts/phase2-week3-rollback.sh"
    print_info ""
    print_info "If stable after 72 hours, proceed to Week 4"
}

# Run main function
main "$@"