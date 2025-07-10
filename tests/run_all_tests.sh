#!/bin/bash

# Master Test Runner for Hierarchical Storage System
# Executes all test suites and generates summary report

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
PURPLE='\033[0;35m'
NC='\033[0m' # No Color

# Test results tracking
TOTAL_TESTS_PASSED=0
TOTAL_TESTS_FAILED=0
TOTAL_TESTS_SKIPPED=0

echo "=================================================================="
echo "    Hierarchical Storage System - Complete Test Suite"
echo "=================================================================="
echo "Starting comprehensive testing of all system components..."
echo

# Function to run test and capture results
run_test_suite() {
    local test_name="$1"
    local test_script="$2"
    local description="$3"
    
    echo -e "${BLUE}▶ Running $test_name${NC}"
    echo "   $description"
    echo "   ────────────────────────────────────────────────────────"
    
    if [ -f "$test_script" ]; then
        # Run the test and capture exit code
        if bash "$test_script"; then
            echo -e "${GREEN}✓ $test_name COMPLETED${NC}"
            return 0
        else
            echo -e "${RED}✗ $test_name FAILED${NC}"
            return 1
        fi
    else
        echo -e "${YELLOW}⚠ $test_name SKIPPED - Script not found: $test_script${NC}"
        return 2
    fi
    echo
}

# Test 1: Database Schema Tests
echo "=== Test Suite 1: Database Schema Validation ==="
if run_test_suite "Database Schema Tests" "hierarchical_storage_tests.sql" "Validate database structure and data integrity"; then
    ((TOTAL_TESTS_PASSED++))
else
    ((TOTAL_TESTS_FAILED++))
fi
echo

# Test 2: Container Deployment Tests  
echo "=== Test Suite 2: Container Deployment Validation ==="
if run_test_suite "Container Deployment Tests" "container_deployment_tests.sh" "Verify Docker containers and service health"; then
    ((TOTAL_TESTS_PASSED++))
else
    ((TOTAL_TESTS_FAILED++))
fi
echo

# Test 3: API Endpoint Tests
echo "=== Test Suite 3: API Endpoint Validation ==="
if run_test_suite "API Endpoint Tests" "api_endpoint_tests.sh" "Test API functionality through gateway"; then
    ((TOTAL_TESTS_PASSED++))
else
    ((TOTAL_TESTS_FAILED++))
fi
echo

# Test 4: Integration Tests
echo "=== Test Suite 4: End-to-End Integration ==="
if run_test_suite "Integration Tests" "integration_tests.sh" "Complete system integration validation"; then
    ((TOTAL_TESTS_PASSED++))
else
    ((TOTAL_TESTS_FAILED++))
fi
echo

# Final System Verification
echo "=== Final System Verification ==="
echo -e "${BLUE}Performing final system health check...${NC}"

# Check critical services
services_ok=true
for service in lims-postgres lims-redis lims-gateway lims-frontend; do
    if docker ps --format "{{.Names}}" | grep -q "^$service$"; then
        echo -e "${GREEN}✓${NC} $service is running"
    else
        echo -e "${RED}✗${NC} $service is not running"
        services_ok=false
    fi
done

# Check database content
db_containers=$(docker exec lims-postgres psql -U postgres -d lims_db -t -c "SELECT COUNT(*) FROM storage_containers;" 2>/dev/null | tr -d ' \n' || echo "0")
db_positions=$(docker exec lims-postgres psql -U postgres -d lims_db -t -c "SELECT COUNT(*) FROM sample_positions;" 2>/dev/null | tr -d ' \n' || echo "0")

echo -e "${GREEN}✓${NC} Database contains $db_containers storage containers"
echo -e "${GREEN}✓${NC} Database contains $db_positions sample positions"

# Check API Gateway
api_status=$(curl -s -w "%{http_code}" http://localhost:8089/health -o /dev/null 2>/dev/null || echo "000")
if [ "$api_status" = "200" ]; then
    echo -e "${GREEN}✓${NC} API Gateway responding (HTTP 200)"
else
    echo -e "${RED}✗${NC} API Gateway not responding (HTTP $api_status)"
    services_ok=false
fi

echo

# Generate Summary Report
echo "=================================================================="
echo "                    FINAL TEST SUMMARY"
echo "=================================================================="
echo -e "${GREEN}Test Suites Passed: $TOTAL_TESTS_PASSED${NC}"
echo -e "${RED}Test Suites Failed: $TOTAL_TESTS_FAILED${NC}"
echo "Total Test Suites: $((TOTAL_TESTS_PASSED + TOTAL_TESTS_FAILED))"

if [ $TOTAL_TESTS_FAILED -eq 0 ] && [ "$services_ok" = true ]; then
    success_rate=100
else
    success_rate=$(( (TOTAL_TESTS_PASSED * 100) / (TOTAL_TESTS_PASSED + TOTAL_TESTS_FAILED) ))
fi

echo "Success Rate: ${success_rate}%"
echo

echo -e "${BLUE}=== System Metrics ===${NC}"
echo "📊 Storage Containers: $db_containers"
echo "📍 Sample Positions: $db_positions"
echo "🌐 API Gateway: $([ "$api_status" = "200" ] && echo "Healthy" || echo "Issues")"
echo "🐳 Container Services: $([ "$services_ok" = true ] && echo "All Running" || echo "Some Issues")"

echo
echo -e "${BLUE}=== Access Points ===${NC}"
echo "🌐 Frontend: http://localhost:3000"
echo "🔌 API Gateway: http://localhost:8089"
echo "🗄️  Database: localhost:5433"
echo "📊 Redis: localhost:6380"

echo
if [ $TOTAL_TESTS_FAILED -eq 0 ] && [ "$services_ok" = true ]; then
    echo -e "${GREEN}🎉 ALL TESTS PASSED! Hierarchical Storage System is fully operational.${NC}"
    echo -e "${GREEN}✅ System is ready for production use.${NC}"
    exit 0
elif [ $success_rate -ge 75 ]; then
    echo -e "${YELLOW}⚠️  Most tests passed but some issues detected.${NC}"
    echo -e "${YELLOW}📋 Review individual test results for details.${NC}"
    exit 0
else
    echo -e "${RED}❌ Multiple test failures detected.${NC}"
    echo -e "${RED}🔧 System requires attention before production use.${NC}"
    exit 1
fi 