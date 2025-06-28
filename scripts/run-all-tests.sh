#!/bin/bash

# TracSeq 2.0 Comprehensive Test Runner

set -e

# Colors for output
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
RED='\033[0;31m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Test results tracking
TOTAL_TESTS=0
PASSED_TESTS=0
FAILED_TESTS=0
FAILED_SERVICES=()

# Export DATABASE_URL for tests
export DATABASE_URL="${DATABASE_URL:-postgresql://tracseq:tracseq@localhost:5432/tracseq_test}"
export TEST_DATABASE_URL="$DATABASE_URL"
export RUST_LOG="${RUST_LOG:-warn}"

echo -e "${BLUE}═══════════════════════════════════════════════════════════════${NC}"
echo -e "${BLUE}          TracSeq 2.0 Comprehensive Test Suite${NC}"
echo -e "${BLUE}═══════════════════════════════════════════════════════════════${NC}"
echo ""

# Function to run tests for a service
run_service_tests() {
    local service=$1
    local features=$2
    
    echo -e "${YELLOW}🧪 Testing $service...${NC}"
    
    if [ -n "$features" ]; then
        echo -e "   Features: $features"
        if cargo test -p $service --features "$features" --no-fail-fast 2>&1 | tee test_output.tmp; then
            echo -e "${GREEN}✅ $service tests passed${NC}"
            ((PASSED_TESTS++))
        else
            echo -e "${RED}❌ $service tests failed${NC}"
            ((FAILED_TESTS++))
            FAILED_SERVICES+=("$service")
        fi
    else
        if cargo test -p $service --no-fail-fast 2>&1 | tee test_output.tmp; then
            echo -e "${GREEN}✅ $service tests passed${NC}"
            ((PASSED_TESTS++))
        else
            echo -e "${RED}❌ $service tests failed${NC}"
            ((FAILED_TESTS++))
            FAILED_SERVICES+=("$service")
        fi
    fi
    
    # Extract test count
    local test_count=$(grep -E "test result:|running [0-9]+ test" test_output.tmp | tail -1 || echo "")
    if [ -n "$test_count" ]; then
        echo -e "   $test_count"
    fi
    
    ((TOTAL_TESTS++))
    echo ""
    rm -f test_output.tmp
}

# Check prerequisites
echo -e "${YELLOW}📋 Checking prerequisites...${NC}"

if ! command -v cargo &> /dev/null; then
    echo -e "${RED}❌ Cargo not found. Please install Rust.${NC}"
    exit 1
fi

if ! pg_isready -h localhost -p 5432 > /dev/null 2>&1; then
    echo -e "${RED}❌ PostgreSQL is not running. Please start PostgreSQL.${NC}"
    echo -e "${YELLOW}   You can use: docker-compose up -d postgres${NC}"
    exit 1
fi

echo -e "${GREEN}✅ Prerequisites satisfied${NC}"
echo ""

# Clean build artifacts
echo -e "${YELLOW}🧹 Cleaning build artifacts...${NC}"
cargo clean
echo -e "${GREEN}✅ Clean complete${NC}"
echo ""

# Build all services first
echo -e "${YELLOW}🔨 Building all services...${NC}"
if cargo build --workspace --all-features; then
    echo -e "${GREEN}✅ Build successful${NC}"
else
    echo -e "${RED}❌ Build failed${NC}"
    exit 1
fi
echo ""

# Run tests for each service
echo -e "${BLUE}═══════════════════════════════════════════════════════════════${NC}"
echo -e "${BLUE}                    Running Service Tests${NC}"
echo -e "${BLUE}═══════════════════════════════════════════════════════════════${NC}"
echo ""

# Core services
run_service_tests "auth_service" ""
run_service_tests "sample_service" ""
run_service_tests "sequencing_service" ""
run_service_tests "notification_service" ""
run_service_tests "qaqc_service" ""
run_service_tests "library_details_service" ""
run_service_tests "enhanced_storage_service" ""
run_service_tests "spreadsheet_versioning_service" ""
run_service_tests "template_service" ""
run_service_tests "event_service" ""
run_service_tests "transaction_service" "database-persistence"

# Library crates
run_service_tests "circuit-breaker-lib" ""

# Integration tests
echo -e "${BLUE}═══════════════════════════════════════════════════════════════${NC}"
echo -e "${BLUE}                  Running Integration Tests${NC}"
echo -e "${BLUE}═══════════════════════════════════════════════════════════════${NC}"
echo ""

echo -e "${YELLOW}🧪 Running workspace integration tests...${NC}"
if cargo test --workspace --test '*' --no-fail-fast 2>&1 | tee test_output.tmp; then
    echo -e "${GREEN}✅ Integration tests passed${NC}"
else
    echo -e "${RED}❌ Integration tests failed${NC}"
    ((FAILED_TESTS++))
fi
rm -f test_output.tmp
echo ""

# Run doctests
echo -e "${YELLOW}📚 Running documentation tests...${NC}"
if cargo test --workspace --doc --no-fail-fast; then
    echo -e "${GREEN}✅ Documentation tests passed${NC}"
else
    echo -e "${RED}❌ Documentation tests failed${NC}"
    ((FAILED_TESTS++))
fi
echo ""

# Summary
echo -e "${BLUE}═══════════════════════════════════════════════════════════════${NC}"
echo -e "${BLUE}                        Test Summary${NC}"
echo -e "${BLUE}═══════════════════════════════════════════════════════════════${NC}"
echo ""

echo -e "Total services tested: $TOTAL_TESTS"
echo -e "${GREEN}Passed: $PASSED_TESTS${NC}"
echo -e "${RED}Failed: $FAILED_TESTS${NC}"

if [ ${#FAILED_SERVICES[@]} -gt 0 ]; then
    echo ""
    echo -e "${RED}Failed services:${NC}"
    for service in "${FAILED_SERVICES[@]}"; do
        echo -e "  - $service"
    done
fi

echo ""

# Generate test report
echo -e "${YELLOW}📊 Generating test report...${NC}"
mkdir -p target/test-reports

cat > target/test-reports/test-summary.json << EOF
{
  "timestamp": "$(date -u +"%Y-%m-%dT%H:%M:%SZ")",
  "total_services": $TOTAL_TESTS,
  "passed": $PASSED_TESTS,
  "failed": $FAILED_TESTS,
  "failed_services": [$(printf '"%s",' "${FAILED_SERVICES[@]}" | sed 's/,$//')],
  "rust_version": "$(rustc --version)",
  "database_url": "$DATABASE_URL"
}
EOF

echo -e "${GREEN}✅ Test report saved to target/test-reports/test-summary.json${NC}"
echo ""

# Check code coverage if tarpaulin is installed
if command -v cargo-tarpaulin &> /dev/null; then
    echo -e "${YELLOW}📊 Running code coverage analysis...${NC}"
    cargo tarpaulin --workspace --out Html --output-dir target/coverage || echo -e "${YELLOW}⚠️  Coverage analysis failed${NC}"
    echo -e "${GREEN}✅ Coverage report saved to target/coverage/index.html${NC}"
fi

echo ""

# Exit with appropriate code
if [ $FAILED_TESTS -eq 0 ]; then
    echo -e "${GREEN}🎉 All tests passed successfully!${NC}"
    exit 0
else
    echo -e "${RED}❌ Some tests failed. Please check the output above.${NC}"
    exit 1
fi