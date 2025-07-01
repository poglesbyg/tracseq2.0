#!/bin/bash

# TracSeq 2.0 Deployment Test Suite
# This script tests all deployed services to ensure they're functioning correctly

# Don't exit on error for arithmetic operations
set +e

# Colors for output
GREEN='\033[0;32m'
RED='\033[0;31m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

echo "================================================"
echo " TracSeq 2.0 - Deployment Test Suite"
echo "================================================"

# Test results tracking
PASSED=0
FAILED=0
WARNINGS=0

# Function to test a service endpoint
test_endpoint() {
    local name=$1
    local url=$2
    local expected_status=${3:-200}
    
    echo -n "Testing $name... "
    
    response=$(curl -s -o /dev/null -w "%{http_code}" "$url" || echo "000")
    
    if [ "$response" == "$expected_status" ]; then
        echo -e "${GREEN}✓ PASSED${NC} (HTTP $response)"
        PASSED=$((PASSED + 1))
    else
        echo -e "${RED}✗ FAILED${NC} (Expected: $expected_status, Got: $response)"
        FAILED=$((FAILED + 1))
    fi
}

# Function to test JSON health endpoint
test_health_endpoint() {
    local name=$1
    local url=$2
    
    echo -n "Testing $name health... "
    
    response=$(curl -s "$url" | jq -r '.status' 2>/dev/null || echo "error")
    
    if [ "$response" == "healthy" ]; then
        echo -e "${GREEN}✓ HEALTHY${NC}"
        PASSED=$((PASSED + 1))
    elif [ "$response" == "unhealthy" ]; then
        echo -e "${YELLOW}⚠ UNHEALTHY${NC}"
        WARNINGS=$((WARNINGS + 1))
    else
        echo -e "${RED}✗ FAILED${NC} (No response or invalid JSON)"
        FAILED=$((FAILED + 1))
    fi
}

echo ""
echo "=== Core Service Health Checks ==="
echo ""

# Test individual services
test_health_endpoint "API Gateway" "http://localhost:18089/health"
test_health_endpoint "Auth Service" "http://localhost:8080/health"
test_health_endpoint "Sample Service" "http://localhost:8081/health"
test_health_endpoint "Sequencing Service" "http://localhost:8084/health"
test_health_endpoint "Notification Service" "http://localhost:8085/health"
test_health_endpoint "Transaction Service" "http://localhost:8088/health"

echo ""
echo "=== API Gateway Route Tests ==="
echo ""

# Test API Gateway routes
test_endpoint "API Gateway Root" "http://localhost:18089/"
test_endpoint "API Gateway Status" "http://localhost:18089/api/v1/status"

echo ""
echo "=== Database Connectivity Tests ==="
echo ""

# Test database connectivity through services
echo -n "Testing database connectivity... "
db_test=$(docker exec tracseq-postgres-primary psql -U tracseq_admin -d tracseq_prod -c "SELECT 1" 2>&1 | grep -c "1 row" || echo 0)
if [ "$db_test" -eq 1 ]; then
    echo -e "${GREEN}✓ PASSED${NC}"
    PASSED=$((PASSED + 1))
else
    echo -e "${RED}✗ FAILED${NC}"
    FAILED=$((FAILED + 1))
fi

echo ""
echo "=== Service Integration Tests ==="
echo ""

# Test service-to-service communication
echo -n "Testing API Gateway → RAG Service integration... "
rag_status=$(curl -s "http://localhost:18089/api/v1/status" | jq -r '.services."rag-service"' 2>/dev/null || echo "error")
if [ "$rag_status" == "healthy" ]; then
    echo -e "${GREEN}✓ PASSED${NC}"
    PASSED=$((PASSED + 1))
else
    echo -e "${RED}✗ FAILED${NC}"
    FAILED=$((FAILED + 1))
fi

echo ""
echo "=== Container Health Status ==="
echo ""

# Check Docker container health status
echo "Container Health Summary:"
docker ps --format "table {{.Names}}\t{{.Status}}" | grep -E "tracseq|lims" | while read line; do
    if echo "$line" | grep -q "healthy"; then
        echo -e "  ${GREEN}✓${NC} $line"
    elif echo "$line" | grep -q "unhealthy"; then
        echo -e "  ${RED}✗${NC} $line"
    else
        echo "    $line"
    fi
done

echo ""
echo "================================================"
echo " Test Summary"
echo "================================================"
echo -e "  ${GREEN}Passed:${NC} $PASSED"
echo -e "  ${YELLOW}Warnings:${NC} $WARNINGS"
echo -e "  ${RED}Failed:${NC} $FAILED"
echo ""

if [ $FAILED -eq 0 ]; then
    echo -e "${GREEN}All tests passed! Deployment is successful.${NC}"
    exit 0
else
    echo -e "${RED}Some tests failed. Please check the logs for more details.${NC}"
    exit 1
fi 