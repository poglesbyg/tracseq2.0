#!/bin/bash

# Test script for QAQC Service
# This script tests the QAQC service functionality

set -e

echo "🧪 Testing QAQC Service Functionality"
echo "======================================"

# Configuration
QAQC_URL="http://localhost:8103"
API_GATEWAY_URL="http://localhost:8089"

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# Test counter
TESTS_PASSED=0
TESTS_TOTAL=0

# Helper function to run test
run_test() {
    local test_name="$1"
    local test_command="$2"
    local expected_status="$3"
    
    echo -n "Testing $test_name... "
    TESTS_TOTAL=$((TESTS_TOTAL + 1))
    
    if eval "$test_command" > /dev/null 2>&1; then
        if [ "$expected_status" = "success" ]; then
            echo -e "${GREEN}PASS${NC}"
            TESTS_PASSED=$((TESTS_PASSED + 1))
        else
            echo -e "${RED}FAIL${NC} (expected failure but got success)"
        fi
    else
        if [ "$expected_status" = "fail" ]; then
            echo -e "${GREEN}PASS${NC} (expected failure)"
            TESTS_PASSED=$((TESTS_PASSED + 1))
        else
            echo -e "${RED}FAIL${NC}"
        fi
    fi
}

# Test 1: Health Check (Direct)
echo -e "\n${YELLOW}1. Direct Service Tests${NC}"
run_test "QAQC Service Health Check" "curl -f $QAQC_URL/health" "success"

# Test 2: API Gateway Integration
echo -e "\n${YELLOW}2. API Gateway Integration Tests${NC}"
run_test "QAQC via API Gateway Health" "curl -f $API_GATEWAY_URL/api/qaqc/health" "success"

# Test 3: QC Dashboard
echo -e "\n${YELLOW}3. QC Dashboard Tests${NC}"
run_test "QC Dashboard Endpoint" "curl -f $QAQC_URL/api/v1/qc/dashboard" "success"

# Test 4: QC Reviews
echo -e "\n${YELLOW}4. QC Reviews Tests${NC}"
run_test "List QC Reviews" "curl -f $QAQC_URL/api/v1/qc/reviews" "success"

# Test 5: QC Metrics
echo -e "\n${YELLOW}5. QC Metrics Tests${NC}"
run_test "Recent QC Metrics" "curl -f $QAQC_URL/api/v1/qc/metrics/recent" "success"
run_test "QC Metric Trends" "curl -f $QAQC_URL/api/v1/qc/metrics/trends" "success"

# Test 6: Control Samples
echo -e "\n${YELLOW}6. Control Samples Tests${NC}"
run_test "List Control Samples" "curl -f $QAQC_URL/api/v1/qc/control-samples" "success"

# Test 7: API Gateway Routing
echo -e "\n${YELLOW}7. API Gateway Routing Tests${NC}"
run_test "QC Dashboard via Gateway" "curl -f $API_GATEWAY_URL/api/qaqc/api/v1/qc/dashboard" "success"
run_test "QC Reviews via Gateway" "curl -f $API_GATEWAY_URL/api/qaqc/api/v1/qc/reviews" "success"

# Test 8: Service Discovery
echo -e "\n${YELLOW}8. Service Discovery Tests${NC}"
run_test "Gateway Service List" "curl -f $API_GATEWAY_URL/services | grep -q qaqc" "success"

# Test 9: Error Handling
echo -e "\n${YELLOW}9. Error Handling Tests${NC}"
run_test "Invalid Endpoint" "curl -f $QAQC_URL/api/v1/qc/invalid-endpoint" "fail"
run_test "Invalid QC Review ID" "curl -f $QAQC_URL/api/v1/qc/reviews/invalid-uuid" "fail"

# Test 10: Performance Tests
echo -e "\n${YELLOW}10. Performance Tests${NC}"
run_test "Response Time < 1s" "timeout 1 curl -f $QAQC_URL/health" "success"

# Summary
echo -e "\n${YELLOW}Test Summary${NC}"
echo "============="
echo "Tests Passed: $TESTS_PASSED/$TESTS_TOTAL"

if [ $TESTS_PASSED -eq $TESTS_TOTAL ]; then
    echo -e "${GREEN}✅ All tests passed! QAQC service is working correctly.${NC}"
    exit 0
else
    echo -e "${RED}❌ Some tests failed. Please check the QAQC service configuration.${NC}"
    exit 1
fi 