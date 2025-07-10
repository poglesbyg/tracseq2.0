#!/bin/bash

# Hierarchical Storage API Endpoint Tests
# Tests API functionality through the API Gateway

set -e

API_BASE_URL="http://localhost:8089"
TEST_RESULTS=()

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# Test result tracking
TESTS_PASSED=0
TESTS_FAILED=0

# Function to log test results
log_test() {
    local test_name="$1"
    local status="$2"
    local message="$3"
    
    if [ "$status" = "PASS" ]; then
        echo -e "${GREEN}✓ PASS${NC}: $test_name - $message"
        ((TESTS_PASSED++))
    else
        echo -e "${RED}✗ FAIL${NC}: $test_name - $message"
        ((TESTS_FAILED++))
    fi
    
    TEST_RESULTS+=("$status: $test_name - $message")
}

# Function to test HTTP endpoint
test_endpoint() {
    local test_name="$1"
    local method="$2"
    local endpoint="$3"
    local expected_status="$4"
    local data="$5"
    
    echo -e "${YELLOW}Testing:${NC} $test_name"
    
    if [ "$method" = "GET" ]; then
        response=$(curl -s -w "\n%{http_code}" "$API_BASE_URL$endpoint" 2>/dev/null || echo -e "\n000")
    elif [ "$method" = "POST" ]; then
        response=$(curl -s -w "\n%{http_code}" -X POST -H "Content-Type: application/json" -d "$data" "$API_BASE_URL$endpoint" 2>/dev/null || echo -e "\n000")
    fi
    
    # Extract status code (last line)
    status_code=$(echo "$response" | tail -1)
    # Extract body (all but last line)
    body=$(echo "$response" | sed '$d')
    
    if [ "$status_code" = "$expected_status" ]; then
        log_test "$test_name" "PASS" "HTTP $status_code"
        return 0
    else
        log_test "$test_name" "FAIL" "Expected HTTP $expected_status, got $status_code"
        return 1
    fi
}

echo "=========================================="
echo "  Hierarchical Storage API Endpoint Tests"
echo "=========================================="
echo

# Test 1: API Gateway Health Check
echo "=== Test 1: Infrastructure Health ==="
test_endpoint "API Gateway Health" "GET" "/health" "200"

# Test 2: Storage Service Health (if available)
echo
echo "=== Test 2: Storage Service Health ==="
test_endpoint "Storage Service Health" "GET" "/storage/health" "200" || \
    log_test "Storage Service Health" "SKIP" "Service may not be directly accessible"

# Test 3: Database Connectivity Tests
echo
echo "=== Test 3: Database Connectivity ==="

# Test storage locations endpoint
test_endpoint "Storage Locations List" "GET" "/storage/locations" "200" || \
    test_endpoint "Storage Locations List (Alt)" "GET" "/api/storage/locations" "200" || \
    log_test "Storage Locations List" "SKIP" "Endpoint may not be implemented"

# Test storage containers endpoint
test_endpoint "Storage Containers List" "GET" "/storage/containers" "200" || \
    test_endpoint "Storage Containers List (Alt)" "GET" "/api/storage/containers" "200" || \
    log_test "Storage Containers List" "SKIP" "Endpoint may not be implemented"

# Test 4: Hierarchical Navigation Tests
echo
echo "=== Test 4: Hierarchical Navigation ==="

# Test getting containers by type
test_endpoint "Get Freezers" "GET" "/storage/containers?type=freezer" "200" || \
    log_test "Get Freezers" "SKIP" "Endpoint may not be implemented"

test_endpoint "Get Racks" "GET" "/storage/containers?type=rack" "200" || \
    log_test "Get Racks" "SKIP" "Endpoint may not be implemented"

test_endpoint "Get Boxes" "GET" "/storage/containers?type=box" "200" || \
    log_test "Get Boxes" "SKIP" "Endpoint may not be implemented"

# Test 5: Sample Position Tests
echo
echo "=== Test 5: Sample Position Management ==="

# Test available positions
test_endpoint "Get Available Positions" "GET" "/storage/positions/available" "200" || \
    test_endpoint "Get Available Positions (Alt)" "GET" "/api/storage/positions/available" "200" || \
    log_test "Get Available Positions" "SKIP" "Endpoint may not be implemented"

# Test 6: Analytics and Reporting Tests
echo
echo "=== Test 6: Analytics and Reporting ==="

# Test capacity summary
test_endpoint "Capacity Summary" "GET" "/storage/capacity/summary" "200" || \
    test_endpoint "Capacity Summary (Alt)" "GET" "/api/storage/capacity/summary" "200" || \
    log_test "Capacity Summary" "SKIP" "Endpoint may not be implemented"

# Test utilization report
test_endpoint "Utilization Report" "GET" "/storage/utilization" "200" || \
    test_endpoint "Utilization Report (Alt)" "GET" "/api/storage/utilization" "200" || \
    log_test "Utilization Report" "SKIP" "Endpoint may not be implemented"

# Test 7: Error Handling Tests
echo
echo "=== Test 7: Error Handling ==="

# Test invalid endpoints
test_endpoint "Invalid Endpoint" "GET" "/storage/nonexistent" "404"
test_endpoint "Invalid Container ID" "GET" "/storage/containers/invalid-id" "400" || \
    test_endpoint "Invalid Container ID" "GET" "/storage/containers/invalid-id" "404"

# Test 8: Service Integration Tests
echo
echo "=== Test 8: Service Integration ==="

# Test that API Gateway can route to different services
test_endpoint "Auth Service via Gateway" "GET" "/auth/health" "200" || \
    log_test "Auth Service via Gateway" "SKIP" "Auth service may not be accessible"

test_endpoint "Sample Service via Gateway" "GET" "/samples/health" "200" || \
    test_endpoint "Sample Service via Gateway (Alt)" "GET" "/api/samples/health" "200" || \
    log_test "Sample Service via Gateway" "SKIP" "Sample service may not be accessible"

# Test 9: Frontend Accessibility
echo
echo "=== Test 9: Frontend Accessibility ==="

frontend_response=$(curl -s -w "%{http_code}" "http://localhost:3000" -o /dev/null 2>/dev/null || echo "000")
if [ "$frontend_response" = "200" ]; then
    log_test "Frontend Accessibility" "PASS" "Frontend accessible at http://localhost:3000"
else
    log_test "Frontend Accessibility" "FAIL" "Frontend not accessible (HTTP $frontend_response)"
fi

# Test 10: Database Direct Access Test
echo
echo "=== Test 10: Database Direct Access ==="

# Test database connectivity through Docker
db_test=$(docker exec lims-postgres psql -U postgres -d lims_db -c "SELECT COUNT(*) FROM storage_containers;" 2>/dev/null | grep -E '^[0-9]+$' | head -1)
if [ -n "$db_test" ] && [ "$db_test" -gt 0 ]; then
    log_test "Database Direct Access" "PASS" "Database accessible with $db_test containers"
else
    log_test "Database Direct Access" "FAIL" "Cannot access database or no containers found"
fi

# Test Summary
echo
echo "=========================================="
echo "              TEST SUMMARY"
echo "=========================================="
echo -e "${GREEN}Tests Passed: $TESTS_PASSED${NC}"
echo -e "${RED}Tests Failed: $TESTS_FAILED${NC}"
echo "Total Tests: $((TESTS_PASSED + TESTS_FAILED))"
echo

if [ $TESTS_FAILED -eq 0 ]; then
    echo -e "${GREEN}🎉 All tests passed!${NC}"
    exit 0
else
    echo -e "${RED}❌ Some tests failed. Check the output above for details.${NC}"
    exit 1
fi 