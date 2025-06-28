#!/bin/bash

# TracSeq 2.0 Comprehensive Test Runner with axum-test Integration
# Enhanced to include Rust (with axum-test), Python, Frontend, and E2E testing

set -e

# Colors for output
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
RED='\033[0;31m'
BLUE='\033[0;34m'
PURPLE='\033[0;35m'
CYAN='\033[0;36m'
NC='\033[0m' # No Color

# Test results tracking
TOTAL_PHASES=0
PASSED_PHASES=0
FAILED_PHASES=0
FAILED_SERVICES=()
DOCKER_STARTED=false

# Configuration
DOCKER_COMPOSE_FILE="${DOCKER_COMPOSE_FILE:-docker-compose.simple.yml}"
TEST_DATABASE_PORT="${TEST_DATABASE_PORT:-5433}"
TEST_TIMEOUT="${TEST_TIMEOUT:-300}"

# Export environment variables for tests
export DATABASE_URL="postgresql://tracseq_admin:tracseq_secure_password@localhost:${TEST_DATABASE_PORT}/tracseq_main"
export TEST_DATABASE_URL="$DATABASE_URL"
export RUST_LOG="${RUST_LOG:-warn}"
export RUST_BACKTRACE="${RUST_BACKTRACE:-1}"
export TEST_MODE="true"
export PYTHONPATH=".:$PYTHONPATH"

# Function to log with timestamps
log() {
    echo -e "[$(date '+%Y-%m-%d %H:%M:%S')] $1"
}

log_success() {
    log "${GREEN}✅ $1${NC}"
}

log_info() {
    log "${BLUE}ℹ️  $1${NC}"
}

log_warning() {
    log "${YELLOW}⚠️  $1${NC}"
}

log_error() {
    log "${RED}❌ $1${NC}"
}

log_section() {
    echo ""
    echo -e "${BLUE}═══════════════════════════════════════════════════════════════${NC}"
    echo -e "${BLUE}  $1${NC}"
    echo -e "${BLUE}═══════════════════════════════════════════════════════════════${NC}"
}

# Function to run with timeout
run_with_timeout() {
    local timeout=$1
    local command=$2
    
    timeout "$timeout" bash -c "$command"
}

# Function to check if service is healthy
check_service_health() {
    local service=$1
    local port=$2
    local path=${3:-"/health"}
    local max_attempts=30
    local attempt=0
    
    log_info "Checking health of $service on port $port..."
    
    while [ $attempt -lt $max_attempts ]; do
        if curl -f -s "http://localhost:$port$path" > /dev/null 2>&1; then
            log_success "$service is healthy"
            return 0
        fi
        
        attempt=$((attempt + 1))
        sleep 2
        echo -n "."
    done
    
    log_error "$service health check failed after $max_attempts attempts"
    return 1
}

# Setup Docker services
setup_docker_services() {
    log_section "Setting up Database Services"
    
    # Check if we already have running PostgreSQL and Redis containers
    if docker ps --format "table {{.Names}}" | grep -q "tracseq-test-postgres" && \
       docker ps --format "table {{.Names}}" | grep -q "tracseq-test-redis"; then
        log_success "Found existing PostgreSQL and Redis containers"
        log_info "Using existing containers:"
        log_info "  - PostgreSQL: tracseq-test-postgres (port 5433)"
        log_info "  - Redis: tracseq-test-redis (port 6380)"
        
        # Test connections
        if docker exec tracseq-test-postgres pg_isready -U tracseq_admin -d tracseq_main >/dev/null 2>&1; then
            log_success "PostgreSQL connection verified"
        else
            log_warning "PostgreSQL connection issues detected"
        fi
        
        export DATABASE_URL="postgresql://tracseq_admin:tracseq_secure_password@localhost:5433/tracseq_main"
        export TEST_DATABASE_URL="$DATABASE_URL"
        export REDIS_URL="redis://:tracseq_redis_password@localhost:6380"
        
        DOCKER_STARTED=true
        return 0
    fi
    
    # Try to start simple containers if complex compose fails
    log_info "Starting simple PostgreSQL and Redis containers..."
    
    # Start PostgreSQL
    if ! docker run -d --name tracseq-test-postgres \
        -e POSTGRES_DB=tracseq_main \
        -e POSTGRES_USER=tracseq_admin \
        -e POSTGRES_PASSWORD=tracseq_secure_password \
        -p 5433:5432 \
        postgres:15-alpine >/dev/null 2>&1; then
        log_warning "Failed to start new PostgreSQL container (may already exist)"
    fi
    
    # Start Redis
    if ! docker run -d --name tracseq-test-redis \
        -p 6380:6379 \
        redis:7-alpine \
        redis-server --requirepass tracseq_redis_password >/dev/null 2>&1; then
        log_warning "Failed to start new Redis container (may already exist)"
    fi
    
    # Wait for services to be ready
    log_info "Waiting for database services to be ready..."
    sleep 5
    
    # Test connections
    if docker exec tracseq-test-postgres pg_isready -U tracseq_admin -d tracseq_main >/dev/null 2>&1; then
        log_success "PostgreSQL is ready"
        export DATABASE_URL="postgresql://tracseq_admin:tracseq_secure_password@localhost:5433/tracseq_main"
        export TEST_DATABASE_URL="$DATABASE_URL"
        DOCKER_STARTED=true
    else
        log_error "PostgreSQL is not responding"
        return 1
    fi
    
    if docker exec tracseq-test-redis redis-cli -a tracseq_redis_password ping >/dev/null 2>&1; then
        log_success "Redis is ready"
        export REDIS_URL="redis://:tracseq_redis_password@localhost:6380"
    else
        log_warning "Redis connection issues"
    fi
}

# Function to run Rust tests with axum-test
run_rust_tests() {
    log_section "Phase 1: Rust Services Testing (with axum-test)"
    
    local rust_services=(
        "auth_service"
        "sample_service"
        "sequencing_service"
        "notification_service"
        "qaqc_service"
        "library_details_service"
        "enhanced_storage_service"
        "spreadsheet_versioning_service"
        "template_service"
        "event_service"
        "transaction_service"
        "circuit-breaker-lib"
        "test-helpers"
    )
    
    local passed=0
    local failed=0
    
    # Clean build artifacts
    log_info "Cleaning Rust build artifacts..."
    cargo clean
    
    # Build all services first
    log_info "Building all Rust services..."
    if cargo build --workspace --all-features; then
        log_success "Rust build successful"
    else
        log_error "Rust build failed"
        return 1
    fi
    
    # Run tests for each service
    for service in "${rust_services[@]}"; do
        log_info "Testing $service with axum-test integration..."
        
        if [ "$service" = "transaction_service" ]; then
            # Special handling for transaction_service with features
            if cargo test -p "$service" --features "database-persistence" --no-fail-fast 2>&1 | tee "target/test-$service.log"; then
                log_success "$service tests passed (with database-persistence)"
                ((passed++))
            else
                log_error "$service tests failed"
                ((failed++))
                FAILED_SERVICES+=("$service")
            fi
        else
            # Standard test execution
            if cargo test -p "$service" --no-fail-fast 2>&1 | tee "target/test-$service.log"; then
                log_success "$service tests passed"
                ((passed++))
            else
                log_error "$service tests failed"
                ((failed++))
                FAILED_SERVICES+=("$service")
            fi
        fi
    done
    
    # Run integration tests
    log_info "Running workspace integration tests..."
    if cargo test --workspace --test '*' --no-fail-fast 2>&1 | tee "target/integration-tests.log"; then
        log_success "Integration tests passed"
    else
        log_error "Integration tests failed"
        ((failed++))
    fi
    
    # Run doctests
    log_info "Running documentation tests..."
    if cargo test --workspace --doc --no-fail-fast 2>&1 | tee "target/doctests.log"; then
        log_success "Documentation tests passed"
    else
        log_error "Documentation tests failed"  
        ((failed++))
    fi
    
    log_info "Rust Testing Summary: $passed passed, $failed failed"
    
    if [ $failed -eq 0 ]; then
        ((PASSED_PHASES++))
        log_success "Rust testing phase completed successfully"
    else
        ((FAILED_PHASES++))
        log_error "Rust testing phase failed"
    fi
    
    ((TOTAL_PHASES++))
}

# Function to run Python tests  
run_python_tests() {
    log_section "Phase 2: Python AI Services Testing"
    
    # Check if uv is available
    local use_uv=false
    if command -v uv >/dev/null 2>&1; then
        use_uv=true
        log_info "Using uv for Python package management"
    fi
    
    # Run the dedicated Python test script
    if [ -f "scripts/test-python.sh" ]; then
        log_info "Running Python test suite..."
        if ./scripts/test-python.sh --verbose; then
            log_success "Python testing completed successfully"
            ((PASSED_PHASES++))
        else
            log_error "Python testing failed"
            ((FAILED_PHASES++))
        fi
    else
        log_warning "Python test script not found, skipping Python tests"
    fi
    
    ((TOTAL_PHASES++))
}

# Function to run frontend tests
run_frontend_tests() {
    log_section "Phase 3: Frontend Testing"
    
    if [ -d "frontend" ]; then
        cd frontend
        
        # Install dependencies
        log_info "Installing frontend dependencies..."
        if command -v pnpm >/dev/null 2>&1; then
            pnpm install
        elif command -v npm >/dev/null 2>&1; then
            npm install
        else
            log_warning "No package manager found, skipping frontend tests"
            cd ..
            return
        fi
        
        # Run tests
        log_info "Running frontend tests..."
        if command -v pnpm >/dev/null 2>&1; then
            if pnpm test --run 2>&1 | tee "../target/frontend-tests.log"; then
                log_success "Frontend tests passed"
                ((PASSED_PHASES++))
            else
                log_error "Frontend tests failed"
                ((FAILED_PHASES++))
            fi
        else
            log_warning "Frontend tests skipped - pnpm not available"
        fi
        
        cd ..
    else
        log_warning "Frontend directory not found, skipping frontend tests"
    fi
    
    ((TOTAL_PHASES++))
}

# Function to run E2E tests
run_e2e_tests() {
    log_section "Phase 4: End-to-End Testing"
    
    # Start all services for E2E testing
    log_info "Starting all services for E2E testing..."
    if docker-compose -f "$DOCKER_COMPOSE_FILE" up -d; then
        log_success "All services started"
        
        # Wait for services to be healthy
        check_service_health "api-gateway" "8089" "/health" || return 1
        check_service_health "frontend" "3000" "/" || return 1
        
        # Run Playwright tests if available
        if [ -f "playwright.config.ts" ]; then
            log_info "Running Playwright E2E tests..."
            if command -v pnpm >/dev/null 2>&1; then
                if pnpm exec playwright test --reporter=html 2>&1 | tee "target/e2e-tests.log"; then
                    log_success "E2E tests passed"
                    ((PASSED_PHASES++))
                else
                    log_error "E2E tests failed"
                    ((FAILED_PHASES++))
                fi
            else
                log_warning "pnpm not available, skipping E2E tests"
            fi
        else
            log_warning "Playwright config not found, skipping E2E tests"
        fi
    else
        log_error "Failed to start all services for E2E testing"
        ((FAILED_PHASES++))
    fi
    
    ((TOTAL_PHASES++))
}

# Function to generate comprehensive test report
generate_test_report() {
    log_section "Generating Test Report"
    
    mkdir -p target/test-reports
    
    # Create comprehensive JSON report
    cat > target/test-reports/comprehensive-test-summary.json << EOF
{
  "timestamp": "$(date -u +"%Y-%m-%dT%H:%M:%SZ")",
  "test_run": {
    "total_phases": $TOTAL_PHASES,
    "passed_phases": $PASSED_PHASES,
    "failed_phases": $FAILED_PHASES,
    "success_rate": $(echo "scale=2; $PASSED_PHASES * 100 / $TOTAL_PHASES" | bc -l 2>/dev/null || echo "0")
  },
  "failed_services": [$(printf '"%s",' "${FAILED_SERVICES[@]}" | sed 's/,$//')],
  "environment": {
    "rust_version": "$(rustc --version)",
    "python_version": "$(python3 --version 2>/dev/null || echo 'Not available')",
    "node_version": "$(node --version 2>/dev/null || echo 'Not available')",
    "database_url": "$DATABASE_URL",
    "docker_compose_file": "$DOCKER_COMPOSE_FILE"
  },
  "phases": {
    "rust_services": "Rust microservices with axum-test integration",
    "python_services": "Python AI services with FastMCP and RAG",
    "frontend": "React frontend testing",
    "e2e": "End-to-end testing with Playwright"
  }
}
EOF

    # Create markdown report
    cat > target/test-reports/comprehensive-test-summary.md << EOF
# TracSeq 2.0 Comprehensive Test Report

**Generated:** $(date -u +"%Y-%m-%d %H:%M:%S UTC")

## Test Summary
- **Total Phases:** $TOTAL_PHASES
- **Passed:** $PASSED_PHASES ✅
- **Failed:** $FAILED_PHASES ❌
- **Success Rate:** $(echo "scale=1; $PASSED_PHASES * 100 / $TOTAL_PHASES" | bc -l 2>/dev/null || echo "0")%

## Test Phases

### Phase 1: Rust Services (axum-test)
- **Purpose:** Test all Rust microservices with axum-test integration
- **Services:** auth_service, sample_service, enhanced_storage_service, etc.
- **Features:** HTTP endpoint testing, database integration, error handling

### Phase 2: Python AI Services  
- **Purpose:** Test Python services including RAG, FastMCP, and API Gateway
- **Coverage:** Unit tests, integration tests, FastMCP server validation
- **Tools:** pytest, httpx, FastAPI TestClient

### Phase 3: Frontend Testing
- **Purpose:** Test React frontend components and integration
- **Framework:** Vitest/Jest with React Testing Library
- **Coverage:** Component rendering, user interactions, API integration

### Phase 4: End-to-End Testing
- **Purpose:** Full system testing across all services
- **Framework:** Playwright with multi-browser support
- **Scenarios:** User workflows, service integration, data flow

## Environment
- **Rust:** $(rustc --version)
- **Python:** $(python3 --version 2>/dev/null || echo 'Not available')
- **Node.js:** $(node --version 2>/dev/null || echo 'Not available')
- **Database:** $DATABASE_URL
- **Docker Compose:** $DOCKER_COMPOSE_FILE

$(if [ ${#FAILED_SERVICES[@]} -gt 0 ]; then
    echo "## Failed Services"
    for service in "${FAILED_SERVICES[@]}"; do
        echo "- $service"
    done
fi)
EOF

    log_success "Test reports generated:"
    log_info "  - JSON: target/test-reports/comprehensive-test-summary.json"
    log_info "  - Markdown: target/test-reports/comprehensive-test-summary.md"
}

# Function to run code coverage analysis
run_coverage_analysis() {
    log_section "Code Coverage Analysis"
    
    # Rust coverage with tarpaulin
    if command -v cargo-tarpaulin >/dev/null 2>&1; then
        log_info "Running Rust code coverage with tarpaulin..."
        cargo tarpaulin --workspace --out Html --output-dir target/coverage/rust || log_warning "Rust coverage analysis failed"
        log_success "Rust coverage report: target/coverage/rust/index.html"
    else
        log_info "cargo-tarpaulin not installed, install with: cargo install cargo-tarpaulin"
    fi
    
    # Python coverage (if pytest-cov available)
    if command -v uv >/dev/null 2>&1; then
        log_info "Python coverage included in test-python.sh execution"
    fi
}

# Cleanup function
cleanup() {
    log_info "Performing cleanup..."
    docker-compose -f "$DOCKER_COMPOSE_FILE" down --remove-orphans 2>/dev/null || true
    
    # Remove temporary files
    rm -f test_output.tmp
    
    # Ensure target directory permissions
    chmod -R u+w target/ 2>/dev/null || true
}

# Trap cleanup on exit
trap cleanup EXIT

# Main execution
main() {
    log_section "TracSeq 2.0 Comprehensive Test Suite with axum-test"
    
    echo -e "${PURPLE}🚀 Enhanced Architecture Testing${NC}"
    echo -e "${CYAN}   ✅ Rust microservices with axum-test integration${NC}"
    echo -e "${CYAN}   🐍 Python AI services with FastMCP${NC}"
    echo -e "${CYAN}   ⚛️  React frontend testing${NC}"
    echo -e "${CYAN}   🎭 End-to-end testing with Playwright${NC}"
    echo ""
    
    # Check prerequisites
    log_section "Checking Prerequisites"
    
    if ! command -v cargo &> /dev/null; then
        log_error "Cargo not found. Please install Rust."
        exit 1
    fi
    
    if ! command -v docker &> /dev/null; then
        log_error "Docker not found. Please install Docker."
        exit 1
    fi
    
    if ! command -v docker-compose &> /dev/null; then
        log_error "Docker Compose not found. Please install Docker Compose."
        exit 1
    fi
    
    log_success "Prerequisites satisfied"
    
    # Create target directory for test outputs
    mkdir -p target/test-reports
    
    # Start Docker services
    setup_docker_services || {
        log_error "Failed to start Docker services"
        exit 1
    }
    
    # Run test phases
    run_rust_tests
    run_python_tests
    run_frontend_tests
    run_e2e_tests
    
    # Generate reports
    generate_test_report
    run_coverage_analysis
    
    # Final summary
    log_section "Final Test Summary"
    
    echo -e "📊 **Test Results:**"
    echo -e "   Total Phases: $TOTAL_PHASES"
    echo -e "   ${GREEN}✅ Passed: $PASSED_PHASES${NC}"
    echo -e "   ${RED}❌ Failed: $FAILED_PHASES${NC}"
    
    if [ ${#FAILED_SERVICES[@]} -gt 0 ]; then
        echo ""
        echo -e "${RED}Failed Services:${NC}"
        for service in "${FAILED_SERVICES[@]}"; do
            echo -e "  - $service"
        done
    fi
    
    echo ""
    
    # Exit with appropriate code
    if [ $FAILED_PHASES -eq 0 ]; then
        log_success "🎉 All test phases completed successfully!"
        echo -e "${GREEN}TracSeq 2.0 is ready for deployment! 🚀${NC}"
        exit 0
    else
        log_error "Some test phases failed. Please check the reports above."
        echo -e "${RED}Please resolve issues before deployment. 🔧${NC}"
        exit 1
    fi
}

# Handle command line arguments
case "${1:-all}" in
    "rust")
        log_info "Running only Rust tests..."
        setup_docker_services
        run_rust_tests
        generate_test_report
        ;;
    "python")
        log_info "Running only Python tests..."
        setup_docker_services
        run_python_tests
        generate_test_report
        ;;
    "frontend")
        log_info "Running only frontend tests..."
        run_frontend_tests
        generate_test_report
        ;;
    "e2e")
        log_info "Running only E2E tests..."
        setup_docker_services
        run_e2e_tests
        generate_test_report
        ;;
    "all"|"")
        main
        ;;
    "--help"|"-h")
        echo "TracSeq 2.0 Comprehensive Test Runner"
        echo ""
        echo "Usage: $0 [OPTION]"
        echo ""
        echo "Options:"
        echo "  all (default)  Run all test phases"
        echo "  rust          Run only Rust tests (with axum-test)"
        echo "  python        Run only Python AI service tests"
        echo "  frontend      Run only frontend tests"
        echo "  e2e           Run only end-to-end tests"
        echo "  --help, -h    Show this help message"
        echo ""
        echo "Environment Variables:"
        echo "  DOCKER_COMPOSE_FILE  Docker compose file to use (default: docker-compose.simple.yml)"
        echo "  TEST_DATABASE_PORT   PostgreSQL port for testing (default: 5433)"
        echo "  TEST_TIMEOUT         Test timeout in seconds (default: 300)"
        exit 0
        ;;
    *)
        log_error "Unknown option: $1"
        echo "Use --help for usage information"
        exit 1
        ;;
esac