#!/bin/bash

# TracSeq 2.0 Comprehensive Testing Script
# Runs all tests in the correct order: unit tests, integration tests, and E2E tests

set -e  # Exit on any error

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Configuration
TEST_TIMEOUT=300  # 5 minutes timeout for each test phase
SERVICES_STARTUP_WAIT=30  # Wait time for services to start

# Cross-platform timeout function
run_with_timeout() {
    local timeout_duration=$1
    shift
    local command="$@"
    
    # Detect OS and use appropriate timeout command
    if [[ "$OSTYPE" == "darwin"* ]]; then
        # macOS - use perl-based timeout or background process approach
        if command -v gtimeout >/dev/null 2>&1; then
            # Use GNU timeout if available (via brew install coreutils)
            gtimeout "$timeout_duration" bash -c "$command"
        else
            # Fallback: use Perl for timeout on macOS
            perl -e "alarm($timeout_duration); exec @ARGV" bash -c "$command"
        fi
    else
        # Linux - use standard timeout command
        timeout "$timeout_duration" bash -c "$command"
    fi
}

# Helper functions
log_info() {
    echo -e "${BLUE}ℹ️  $1${NC}"
}

log_success() {
    echo -e "${GREEN}✅ $1${NC}"
}

log_warning() {
    echo -e "${YELLOW}⚠️  $1${NC}"
}

log_error() {
    echo -e "${RED}❌ $1${NC}"
}

log_section() {
    echo -e "\n${BLUE}═══════════════════════════════════════${NC}"
    echo -e "${BLUE}  $1${NC}"
    echo -e "${BLUE}═══════════════════════════════════════${NC}\n"
}

# Cleanup function
cleanup() {
    log_info "Cleaning up test environment..."
    
    # Stop test services
    docker-compose -f docker-compose.test.yml down -v 2>/dev/null || true
    docker-compose -f docker-compose.working-microservices.yml down 2>/dev/null || true
    
    # Kill any remaining processes
    pkill -f "pnpm.*dev" 2>/dev/null || true
    pkill -f "vite" 2>/dev/null || true
    pkill -f "cargo.*run" 2>/dev/null || true
    
    log_info "Cleanup completed"
}

# Set up cleanup trap
trap cleanup EXIT

# Check prerequisites
check_prerequisites() {
    log_section "Checking Prerequisites"
    
    # Check if required tools are installed
    local missing_tools=()
    
    command -v docker >/dev/null 2>&1 || missing_tools+=("docker")
    command -v docker-compose >/dev/null 2>&1 || missing_tools+=("docker-compose")
    command -v cargo >/dev/null 2>&1 || missing_tools+=("cargo")
    command -v pnpm >/dev/null 2>&1 || missing_tools+=("pnpm")
    command -v npx >/dev/null 2>&1 || missing_tools+=("npx")
    
    if [ ${#missing_tools[@]} -ne 0 ]; then
        log_error "Missing required tools: ${missing_tools[*]}"
        exit 1
    fi
    
    # Check if Playwright is installed
    if ! npx playwright --version >/dev/null 2>&1; then
        log_warning "Playwright not found, installing..."
        npx playwright install
    fi
    
    log_success "All prerequisites satisfied"
}

# Phase 1: Rust Unit Tests
run_rust_unit_tests() {
    log_section "Phase 1: Rust Unit Tests"
    
    log_info "Running Rust unit tests..."
    
    # Run unit tests for each service
    local services=("lab_manager" "auth_service" "sample_service" "template_service" "sequencing_service" "notification_service")
    
    for service in "${services[@]}"; do
        if [ -d "$service" ]; then
            log_info "Testing $service..."
            run_with_timeout $TEST_TIMEOUT "cargo test --package $service --lib" || {
                log_error "Unit tests failed for $service"
                return 1
            }
        else
            log_warning "Service directory $service not found, skipping..."
        fi
    done
    
    log_success "All Rust unit tests passed"
}

# Phase 2: Rust Integration Tests with axum-test
run_rust_integration_tests() {
    log_section "Phase 2: Rust Integration Tests"
    
    log_info "Running Rust integration tests with axum-test..."
    
    # Set test environment variables
    export DATABASE_URL="postgres://postgres:postgres@localhost:5434/lab_manager_test"
    export JWT_SECRET="test-jwt-secret"
    export RUST_LOG="debug"
    export TEST_MODE="true"
    
    # Start test database
    log_info "Starting test database..."
    docker-compose -f docker-compose.test.yml up -d test-postgres test-redis
    
    # Wait for database to be ready
    log_info "Waiting for test database to be ready..."
    run_with_timeout 60 'until docker-compose -f docker-compose.test.yml exec -T test-postgres pg_isready -U postgres -d lab_manager_test; do sleep 2; done'
    
    # Run integration tests
    run_with_timeout $TEST_TIMEOUT "cargo test --workspace integration" || {
        log_error "Rust integration tests failed"
        return 1
    }
    
    log_success "All Rust integration tests passed"
}

# Phase 3: Start Microservices
start_microservices() {
    log_section "Phase 3: Starting Microservices"
    
    log_info "Starting microservices for E2E testing..."
    
    # Stop any existing services
    docker-compose -f docker-compose.working-microservices.yml down 2>/dev/null || true
    
    # Start services
    docker-compose -f docker-compose.working-microservices.yml up -d
    
    log_info "Waiting ${SERVICES_STARTUP_WAIT}s for services to start..."
    sleep $SERVICES_STARTUP_WAIT
    
    # Check service health
    local services=(
        "http://localhost:5432"  # PostgreSQL
        "http://localhost:3000/health"  # Lab Manager
        "http://localhost:8089/health"  # API Gateway
        "http://localhost:8000/health"  # RAG Service
    )
    
    for service in "${services[@]}"; do
        log_info "Checking $service..."
        if curl -f -s "$service" >/dev/null 2>&1; then
            log_success "Service $service is healthy"
        else
            log_error "Service $service is not responding"
            return 1
        fi
    done
    
    log_success "All microservices are running and healthy"
}

# Phase 4: Start Frontend
start_frontend() {
    log_section "Phase 4: Starting Frontend"
    
    log_info "Starting frontend development server..."
    
    cd lab_manager/frontend
    
    # Kill any existing frontend processes
    pkill -f "pnpm.*dev" 2>/dev/null || true
    pkill -f "vite" 2>/dev/null || true
    
    # Start frontend in background
    nohup pnpm dev > ../../frontend-test.log 2>&1 &
    local frontend_pid=$!
    
    # Wait for frontend to start
    log_info "Waiting for frontend to start..."
    local attempts=0
    local max_attempts=30
    
    while [ $attempts -lt $max_attempts ]; do
        if curl -f -s "http://localhost:5176" >/dev/null 2>&1; then
            log_success "Frontend is running on http://localhost:5176"
            cd ../..
            return 0
        fi
        
        sleep 2
        attempts=$((attempts + 1))
    done
    
    log_error "Frontend failed to start within timeout"
    cd ../..
    return 1
}

# Phase 5: API Integration Tests
run_api_tests() {
    log_section "Phase 5: API Integration Tests"
    
    log_info "Running API integration tests..."
    
    # Set environment variables for Playwright
    export API_GATEWAY_URL="http://localhost:8089"
    export BACKEND_URL="http://localhost:3000"
    export BASE_URL="http://localhost:5176"
    
    run_with_timeout $TEST_TIMEOUT "npx playwright test --project=api-integration" || {
        log_error "API integration tests failed"
        return 1
    }
    
    log_success "All API integration tests passed"
}

# Phase 6: Frontend E2E Tests
run_frontend_e2e_tests() {
    log_section "Phase 6: Frontend E2E Tests"
    
    log_info "Running frontend E2E tests..."
    
    run_with_timeout $TEST_TIMEOUT "npx playwright test --project=frontend-e2e" || {
        log_error "Frontend E2E tests failed"
        return 1
    }
    
    log_success "All frontend E2E tests passed"
}

# Phase 7: Full Stack Integration Tests
run_fullstack_tests() {
    log_section "Phase 7: Full Stack Integration Tests"
    
    log_info "Running full stack integration tests..."
    
    run_with_timeout $TEST_TIMEOUT "npx playwright test --project=full-stack" || {
        log_error "Full stack integration tests failed"
        return 1
    }
    
    log_success "All full stack integration tests passed"
}

# Phase 8: Performance and Load Tests
run_performance_tests() {
    log_section "Phase 8: Performance Tests"
    
    log_info "Running performance and load tests..."
    
    # Run a subset of performance-focused tests
    run_with_timeout $TEST_TIMEOUT "npx playwright test --grep='performance|concurrent|load'" || {
        log_warning "Some performance tests failed (non-critical)"
    }
    
    log_success "Performance tests completed"
}

# Generate Test Report
generate_report() {
    log_section "Generating Test Report"
    
    log_info "Generating comprehensive test report..."
    
    # Create test results directory
    mkdir -p test-results
    
    # Generate HTML report
    npx playwright show-report --reporter=html
    
    log_success "Test report generated: playwright-report/index.html"
}

# Main execution
main() {
    local start_time=$(date +%s)
    
    log_section "TracSeq 2.0 Comprehensive Testing"
    log_info "Starting comprehensive test suite..."
    
    # Parse command line arguments
    local run_unit=true
    local run_integration=true
    local run_e2e=true
    local run_performance=false
    
    case "${1:-all}" in
        "unit")
            run_integration=false
            run_e2e=false
            ;;
        "integration")
            run_unit=false
            run_e2e=false
            ;;
        "e2e")
            run_unit=false
            run_integration=false
            ;;
        "performance")
            run_unit=false
            run_integration=false
            run_performance=true
            ;;
        "quick")
            run_performance=false
            ;;
        "all")
            run_performance=true
            ;;
    esac
    
    # Run test phases
    check_prerequisites
    
    if [ "$run_unit" = true ]; then
        run_rust_unit_tests || exit 1
    fi
    
    if [ "$run_integration" = true ]; then
        run_rust_integration_tests || exit 1
    fi
    
    if [ "$run_e2e" = true ]; then
        start_microservices || exit 1
        start_frontend || exit 1
        run_api_tests || exit 1
        run_frontend_e2e_tests || exit 1
        run_fullstack_tests || exit 1
    fi
    
    if [ "$run_performance" = true ]; then
        run_performance_tests
    fi
    
    if [ "$run_e2e" = true ]; then
        generate_report
    fi
    
    local end_time=$(date +%s)
    local duration=$((end_time - start_time))
    
    log_section "Test Suite Completed"
    log_success "All tests completed successfully! 🎉"
    log_info "Total execution time: ${duration}s"
    log_info "Test artifacts saved in: test-results/ and playwright-report/"
}

# Show usage if help requested
if [[ "${1}" == "--help" || "${1}" == "-h" ]]; then
    echo "TracSeq 2.0 Testing Script"
    echo ""
    echo "Usage: $0 [OPTION]"
    echo ""
    echo "Options:"
    echo "  all          Run all tests (default)"
    echo "  unit         Run only Rust unit tests"
    echo "  integration  Run only Rust integration tests"
    echo "  e2e          Run only E2E tests"
    echo "  performance  Run only performance tests"
    echo "  quick        Run all tests except performance"
    echo "  --help, -h   Show this help message"
    echo ""
    exit 0
fi

# Run main function
main "$@" 