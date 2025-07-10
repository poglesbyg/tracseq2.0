#!/bin/bash

# Container Deployment Tests
# Tests Docker container deployment and service health

# set -e # Don't exit on first failure for tests

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
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
}

echo "=========================================="
echo "    Container Deployment Tests"
echo "=========================================="
echo

# Test 1: Required Containers Running
echo "=== Test 1: Container Status ==="

# Check PostgreSQL
if docker ps --format "table {{.Names}}\t{{.Status}}" | grep -q "lims-postgres.*Up"; then
    log_test "PostgreSQL Container" "PASS" "lims-postgres is running"
else
    log_test "PostgreSQL Container" "FAIL" "lims-postgres is not running"
fi

# Check Redis
if docker ps --format "table {{.Names}}\t{{.Status}}" | grep -q "lims-redis.*Up"; then
    log_test "Redis Container" "PASS" "lims-redis is running"
else
    log_test "Redis Container" "FAIL" "lims-redis is not running"
fi

# Check API Gateway
if docker ps --format "table {{.Names}}\t{{.Status}}" | grep -q "lims-gateway.*Up"; then
    log_test "API Gateway Container" "PASS" "lims-gateway is running"
else
    log_test "API Gateway Container" "FAIL" "lims-gateway is not running"
fi

# Check Frontend
if docker ps --format "table {{.Names}}\t{{.Status}}" | grep -q "lims-frontend.*Up"; then
    log_test "Frontend Container" "PASS" "lims-frontend is running"
else
    log_test "Frontend Container" "FAIL" "lims-frontend is not running"
fi

# Check Auth Service
if docker ps --format "table {{.Names}}\t{{.Status}}" | grep -q "lims-auth.*Up"; then
    log_test "Auth Service Container" "PASS" "lims-auth is running"
else
    log_test "Auth Service Container" "FAIL" "lims-auth is not running"
fi

# Check Sample Service
if docker ps --format "table {{.Names}}\t{{.Status}}" | grep -q "lims-samples.*Up"; then
    log_test "Sample Service Container" "PASS" "lims-samples is running"
else
    log_test "Sample Service Container" "FAIL" "lims-samples is not running"
fi

# Test 2: Network Connectivity
echo
echo "=== Test 2: Network Connectivity ==="

# Check if containers can communicate
network_test=$(docker exec lims-gateway ping -c 1 lims-postgres 2>/dev/null && echo "SUCCESS" || echo "FAILED")
if [ "$network_test" = "SUCCESS" ]; then
    log_test "Container Network" "PASS" "Containers can communicate via Docker network"
else
    log_test "Container Network" "FAIL" "Network connectivity issues between containers"
fi

# Test 3: Service Health Checks
echo
echo "=== Test 3: Service Health Checks ==="

# PostgreSQL Health
pg_health=$(docker exec lims-postgres pg_isready -U postgres 2>/dev/null && echo "HEALTHY" || echo "UNHEALTHY")
if [ "$pg_health" = "HEALTHY" ]; then
    log_test "PostgreSQL Health" "PASS" "Database is accepting connections"
else
    log_test "PostgreSQL Health" "FAIL" "Database is not responding"
fi

# Redis Health
redis_health=$(docker exec lims-redis redis-cli ping 2>/dev/null | grep -q "PONG" && echo "HEALTHY" || echo "UNHEALTHY")
if [ "$redis_health" = "HEALTHY" ]; then
    log_test "Redis Health" "PASS" "Redis is responding to ping"
else
    log_test "Redis Health" "FAIL" "Redis is not responding"
fi

# API Gateway Health
gateway_health=$(curl -s -f http://localhost:8089/health >/dev/null 2>&1 && echo "HEALTHY" || echo "UNHEALTHY")
if [ "$gateway_health" = "HEALTHY" ]; then
    log_test "API Gateway Health" "PASS" "API Gateway is responding"
else
    log_test "API Gateway Health" "FAIL" "API Gateway is not responding"
fi

# Frontend Health
frontend_health=$(curl -s -f http://localhost:3000 >/dev/null 2>&1 && echo "HEALTHY" || echo "UNHEALTHY")
if [ "$frontend_health" = "HEALTHY" ]; then
    log_test "Frontend Health" "PASS" "Frontend is accessible"
else
    log_test "Frontend Health" "FAIL" "Frontend is not accessible"
fi

# Test 4: Port Accessibility
echo
echo "=== Test 4: Port Accessibility ==="

# Test key ports
ports=("5433:PostgreSQL" "6380:Redis" "8089:API Gateway" "3000:Frontend" "8011:Auth Service" "8012:Sample Service")

for port_info in "${ports[@]}"; do
    port=$(echo $port_info | cut -d: -f1)
    service=$(echo $port_info | cut -d: -f2)
    
    if nc -z localhost $port 2>/dev/null; then
        log_test "$service Port ($port)" "PASS" "Port is accessible"
    else
        log_test "$service Port ($port)" "FAIL" "Port is not accessible"
    fi
done

# Test 5: Volume Mounts
echo
echo "=== Test 5: Volume Mounts ==="

# Check PostgreSQL data persistence
pg_volume=$(docker inspect lims-postgres --format '{{range .Mounts}}{{.Type}}:{{.Destination}} {{end}}' | grep -q "/var/lib/postgresql/data" && echo "MOUNTED" || echo "NOT_MOUNTED")
if [ "$pg_volume" = "MOUNTED" ]; then
    log_test "PostgreSQL Volume" "PASS" "Data volume is mounted"
else
    log_test "PostgreSQL Volume" "FAIL" "Data volume is not mounted"
fi

# Test 6: Container Resource Usage
echo
echo "=== Test 6: Container Resource Usage ==="

# Check memory usage (basic test)
high_memory_containers=$(docker stats --no-stream --format "table {{.Name}}\t{{.MemUsage}}" | grep -E "(GB|[5-9][0-9][0-9]MB)" | wc -l)
if [ "$high_memory_containers" -lt 3 ]; then
    log_test "Memory Usage" "PASS" "Containers using reasonable memory"
else
    log_test "Memory Usage" "FAIL" "Some containers using excessive memory"
fi

# Test 7: Docker Compose Configuration
echo
echo "=== Test 7: Docker Compose Configuration ==="

# Check if docker-compose.yml is valid
if docker-compose config >/dev/null 2>&1; then
    log_test "Docker Compose Config" "PASS" "Configuration is valid"
else
    log_test "Docker Compose Config" "FAIL" "Configuration has errors"
fi

# Test 8: Image Build Status
echo
echo "=== Test 8: Image Build Status ==="

# Check if required images exist
images=("docker-postgres" "docker-api-gateway" "docker-frontend" "docker-auth-service" "docker-sample-service" "docker-storage-service")

for image in "${images[@]}"; do
    if docker images --format "table {{.Repository}}" | grep -q "^$image$"; then
        log_test "$image Image" "PASS" "Image exists"
    else
        log_test "$image Image" "FAIL" "Image not found"
    fi
done

# Test 9: Container Logs Check
echo
echo "=== Test 9: Container Logs Check ==="

# Check for error patterns in logs
error_containers=()
for container in lims-postgres lims-redis lims-gateway lims-frontend lims-auth lims-samples; do
    if docker logs $container 2>&1 | grep -qi "error\|fatal\|exception" | head -1 >/dev/null; then
        error_containers+=($container)
    fi
done

if [ ${#error_containers[@]} -eq 0 ]; then
    log_test "Container Logs" "PASS" "No critical errors in container logs"
else
    log_test "Container Logs" "FAIL" "Errors found in: ${error_containers[*]}"
fi

# Test 10: Storage Service Build Test
echo
echo "=== Test 10: Storage Service Build Test ==="

# Test if the storage service can be built successfully
if docker build -t test-storage-build -f ../lims-enhanced/enhanced_storage_service/Dockerfile.working ../lims-enhanced/enhanced_storage_service >/dev/null 2>&1; then
    log_test "Storage Service Build" "PASS" "Storage service builds successfully"
    docker rmi test-storage-build >/dev/null 2>&1
else
    log_test "Storage Service Build" "FAIL" "Storage service build failed"
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

# Additional Information
echo -e "${BLUE}=== Container Status Summary ===${NC}"
docker ps --format "table {{.Names}}\t{{.Status}}\t{{.Ports}}" | grep "lims-"

echo
echo -e "${BLUE}=== Resource Usage Summary ===${NC}"
docker stats --no-stream --format "table {{.Name}}\t{{.CPUPerc}}\t{{.MemUsage}}" | grep "lims-"

if [ $TESTS_FAILED -eq 0 ]; then
    echo
    echo -e "${GREEN}🎉 All container deployment tests passed!${NC}"
    exit 0
else
    echo
    echo -e "${RED}❌ Some deployment tests failed. Check the output above for details.${NC}"
    exit 1
fi 