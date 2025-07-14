#!/bin/bash

# TracSeq 2.0 - Basic End-to-End Testing Script
# Tests complete request flows through API Gateway

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# Configuration
API_GATEWAY_URL="http://localhost:8089"
FRONTEND_PROXY_URL="http://localhost:8085"
DIRECT_SERVICE_BASE="http://localhost"

# Test counters
TOTAL_TESTS=0
PASSED_TESTS=0
FAILED_TESTS=0

# Helper functions
log_info() {
    echo -e "${GREEN}[INFO]${NC} $1"
}

log_warn() {
    echo -e "${YELLOW}[WARN]${NC} $1"
}

log_error() {
    echo -e "${RED}[ERROR]${NC} $1"
}

test_endpoint() {
    local description="$1"
    local url="$2"
    local expected_status="${3:-200}"
    
    TOTAL_TESTS=$((TOTAL_TESTS + 1))
    
    echo -n "Testing: $description... "
    
    response=$(curl -s -w "%{http_code}" -o /tmp/response.json "$url" 2>/dev/null || echo "000")
    
    if [ "$response" = "$expected_status" ]; then
        echo -e "${GREEN}PASS${NC}"
        PASSED_TESTS=$((PASSED_TESTS + 1))
        return 0
    else
        echo -e "${RED}FAIL${NC} (Expected: $expected_status, Got: $response)"
        FAILED_TESTS=$((FAILED_TESTS + 1))
        if [ -f /tmp/response.json ]; then
            echo "Response: $(cat /tmp/response.json)"
        fi
        return 1
    fi
}

test_json_response() {
    local description="$1"
    local url="$2"
    local expected_key="$3"
    
    TOTAL_TESTS=$((TOTAL_TESTS + 1))
    
    echo -n "Testing: $description... "
    
    response=$(curl -s "$url" 2>/dev/null || echo "{}")
    
    if echo "$response" | jq -e ".$expected_key" >/dev/null 2>&1; then
        echo -e "${GREEN}PASS${NC}"
        PASSED_TESTS=$((PASSED_TESTS + 1))
        return 0
    else
        echo -e "${RED}FAIL${NC} (Expected key: $expected_key not found)"
        echo "Response: $response"
        FAILED_TESTS=$((FAILED_TESTS + 1))
        return 1
    fi
}

test_database_operation() {
    local description="$1"
    local method="$2"
    local url="$3"
    local data="$4"
    
    TOTAL_TESTS=$((TOTAL_TESTS + 1))
    
    echo -n "Testing: $description... "
    
    if [ "$method" = "POST" ]; then
        response=$(curl -s -X POST -H "Content-Type: application/json" -d "$data" "$url" 2>/dev/null || echo "{}")
    else
        response=$(curl -s "$url" 2>/dev/null || echo "{}")
    fi
    
    if echo "$response" | jq -e '.' >/dev/null 2>&1; then
        echo -e "${GREEN}PASS${NC}"
        PASSED_TESTS=$((PASSED_TESTS + 1))
        return 0
    else
        echo -e "${RED}FAIL${NC}"
        echo "Response: $response"
        FAILED_TESTS=$((FAILED_TESTS + 1))
        return 1
    fi
}

print_header() {
    echo "=================================================="
    echo "TracSeq 2.0 - Basic End-to-End Testing"
    echo "=================================================="
    echo "API Gateway: $API_GATEWAY_URL"
    echo "Frontend Proxy: $FRONTEND_PROXY_URL"
    echo "=================================================="
    echo
}

print_summary() {
    echo
    echo "=================================================="
    echo "TEST SUMMARY"
    echo "=================================================="
    echo "Total Tests: $TOTAL_TESTS"
    echo -e "Passed: ${GREEN}$PASSED_TESTS${NC}"
    echo -e "Failed: ${RED}$FAILED_TESTS${NC}"
    echo "Success Rate: $(( PASSED_TESTS * 100 / TOTAL_TESTS ))%"
    echo "=================================================="
    
    if [ $FAILED_TESTS -eq 0 ]; then
        log_info "All tests passed! ✅"
        exit 0
    else
        log_error "Some tests failed! ❌"
        exit 1
    fi
}

# Main test execution
main() {
    print_header
    
    # Check if jq is available
    if ! command -v jq &> /dev/null; then
        log_error "jq is required for JSON parsing. Please install it."
        exit 1
    fi
    
    # Test 1: Health Endpoints via API Gateway
    log_info "Testing Health Endpoints via API Gateway..."
    test_endpoint "Dashboard Health" "$API_GATEWAY_URL/api/dashboard/health"
    test_endpoint "Samples Health" "$API_GATEWAY_URL/api/samples/health"
    test_endpoint "Sequencing Health" "$API_GATEWAY_URL/api/sequencing/health"
    test_endpoint "Spreadsheet Health" "$API_GATEWAY_URL/api/spreadsheet/health"
    
    # Test 2: Health Endpoints via Frontend Proxy
    log_info "Testing Health Endpoints via Frontend Proxy..."
    test_endpoint "Dashboard Health (Proxy)" "$FRONTEND_PROXY_URL/api/dashboard/health"
    test_endpoint "Samples Health (Proxy)" "$FRONTEND_PROXY_URL/api/samples/health"
    test_endpoint "Sequencing Health (Proxy)" "$FRONTEND_PROXY_URL/api/sequencing/health"
    test_endpoint "Spreadsheet Health (Proxy)" "$FRONTEND_PROXY_URL/api/spreadsheet/health"
    
    # Test 3: Direct Service Access
    log_info "Testing Direct Service Access..."
    test_endpoint "Dashboard Direct" "$DIRECT_SERVICE_BASE:8080/health"
    test_endpoint "Samples Direct" "$DIRECT_SERVICE_BASE:8081/health"
    test_endpoint "Sequencing Direct" "$DIRECT_SERVICE_BASE:8082/health"
    test_endpoint "Spreadsheet Direct" "$DIRECT_SERVICE_BASE:8083/health"
    
    # Test 4: API Gateway Service Discovery
    log_info "Testing API Gateway Service Discovery..."
    test_json_response "Service Discovery" "$API_GATEWAY_URL/services" "services"
    
    # Test 5: Database Operations via Services
    log_info "Testing Database Operations..."
    test_json_response "Get All Samples" "$API_GATEWAY_URL/api/samples/v1/samples" "data"
    test_json_response "Get All Users" "$API_GATEWAY_URL/api/dashboard/v1/users" "data"
    test_json_response "Get Storage Locations" "$API_GATEWAY_URL/api/dashboard/v1/storage/locations" "data"
    test_json_response "Get Sequencing Jobs" "$API_GATEWAY_URL/api/sequencing/v1/jobs" "data"
    test_json_response "Get Templates" "$API_GATEWAY_URL/api/spreadsheet/v1/templates" "data"
    
    # Test 6: Sample Submission Workflow
    log_info "Testing Sample Submission Workflow..."
    
    # Create a test sample
    sample_data='{
        "name": "Test Sample E2E",
        "sample_type": "DNA",
        "volume": 100.0,
        "concentration": 50.0,
        "storage_location": "Freezer A1",
        "submitter_id": "test-user-1"
    }'
    
    test_database_operation "Create Sample" "POST" "$API_GATEWAY_URL/api/samples/v1/samples" "$sample_data"
    
    # Test 7: Cross-Service Communication
    log_info "Testing Cross-Service Communication..."
    
    # Test that dashboard can access sample data
    test_json_response "Dashboard Sample Access" "$API_GATEWAY_URL/api/dashboard/v1/samples" "data"
    
    # Test 8: Error Handling
    log_info "Testing Error Handling..."
    test_endpoint "Non-existent Endpoint" "$API_GATEWAY_URL/api/nonexistent" "404"
    test_endpoint "Invalid Sample ID" "$API_GATEWAY_URL/api/samples/v1/samples/invalid-id" "404"
    
    # Test 9: Authentication Endpoints
    log_info "Testing Authentication Endpoints..."
    test_endpoint "Auth Status" "$API_GATEWAY_URL/api/auth/v1/status"
    
    # Test 10: Template Operations
    log_info "Testing Template Operations..."
    test_json_response "Get Sample Templates" "$API_GATEWAY_URL/api/spreadsheet/v1/templates/sample" "data"
    
    print_summary
}

# Run tests
main "$@" 