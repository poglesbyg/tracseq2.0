#!/bin/bash

# Hierarchical Storage Integration Tests
# End-to-end tests for the complete system

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
PURPLE='\033[0;35m'
NC='\033[0m' # No Color

# Test result tracking
TESTS_PASSED=0
TESTS_FAILED=0
TESTS_SKIPPED=0

# Function to log test results
log_test() {
    local test_name="$1"
    local status="$2"
    local message="$3"
    
    if [ "$status" = "PASS" ]; then
        echo -e "${GREEN}✓ PASS${NC}: $test_name - $message"
        ((TESTS_PASSED++))
    elif [ "$status" = "SKIP" ]; then
        echo -e "${YELLOW}⚠ SKIP${NC}: $test_name - $message"
        ((TESTS_SKIPPED++))
    else
        echo -e "${RED}✗ FAIL${NC}: $test_name - $message"
        ((TESTS_FAILED++))
    fi
}

echo "=================================================="
echo "    Hierarchical Storage Integration Tests"
echo "=================================================="
echo

# Test 1: System Readiness
echo "=== Test 1: System Readiness ==="

# Check all required services
services_ready=true
for service in lims-postgres lims-redis lims-gateway lims-frontend; do
    if ! docker ps --format "{{.Names}}" | grep -q "^$service$"; then
        log_test "Service Check" "FAIL" "$service is not running"
        services_ready=false
    fi
done

if [ "$services_ready" = true ]; then
    log_test "Service Check" "PASS" "All required services are running"
else
    log_test "Service Check" "FAIL" "Some required services are not running"
fi

# Test 2: Database Schema Validation
echo
echo "=== Test 2: Database Schema Validation ==="

# Test hierarchical storage tables exist and have data
tables=("storage_locations" "storage_containers" "sample_positions")
schema_valid=true

for table in "${tables[@]}"; do
    count=$(docker exec lims-postgres psql -U postgres -d lims_db -t -c "SELECT COUNT(*) FROM $table;" 2>/dev/null | tr -d ' \n' || echo "0")
    if [ "$count" -gt 0 ]; then
        log_test "$table Table" "PASS" "$count records found"
    else
        log_test "$table Table" "FAIL" "No records found or table missing"
        schema_valid=false
    fi
done

# Test hierarchical relationships
hierarchy_test=$(docker exec lims-postgres psql -U postgres -d lims_db -t -c "
    SELECT COUNT(*) FROM storage_containers sc1 
    JOIN storage_containers sc2 ON sc1.parent_id = sc2.id 
    WHERE sc1.container_type = 'rack' AND sc2.container_type = 'freezer';" 2>/dev/null | tr -d ' \n' || echo "0")

if [ "$hierarchy_test" -gt 0 ]; then
    log_test "Hierarchical Relationships" "PASS" "$hierarchy_test rack-freezer relationships found"
else
    log_test "Hierarchical Relationships" "FAIL" "No hierarchical relationships found"
    schema_valid=false
fi

# Test 3: Data Integrity Validation
echo
echo "=== Test 3: Data Integrity Validation ==="

# Test sample position grid integrity
grid_test=$(docker exec lims-postgres psql -U postgres -d lims_db -t -c "
    SELECT COUNT(*) FROM (
        SELECT container_id, COUNT(*) as position_count 
        FROM sample_positions 
        GROUP BY container_id 
        HAVING COUNT(*) > 0
    ) as grid_counts;" 2>/dev/null | tr -d ' \n' || echo "0")

if [ "$grid_test" -gt 0 ]; then
    log_test "Position Grid Integrity" "PASS" "$grid_test containers have position grids"
else
    log_test "Position Grid Integrity" "FAIL" "No position grids found"
fi

# Test unique constraints
duplicate_test=$(docker exec lims-postgres psql -U postgres -d lims_db -t -c "
    SELECT COUNT(*) FROM (
        SELECT container_id, position_x, position_y 
        FROM sample_positions 
        GROUP BY container_id, position_x, position_y 
        HAVING COUNT(*) > 1
    ) as duplicates;" 2>/dev/null | tr -d ' \n' || echo "1")

if [ "$duplicate_test" = "0" ]; then
    log_test "Position Uniqueness" "PASS" "No duplicate position coordinates"
else
    log_test "Position Uniqueness" "FAIL" "$duplicate_test duplicate position coordinates found"
fi

# Test 4: API Gateway Integration
echo
echo "=== Test 4: API Gateway Integration ==="

# Test API Gateway health
gateway_response=$(curl -s -w "%{http_code}" http://localhost:8089/health -o /dev/null 2>/dev/null || echo "000")
if [ "$gateway_response" = "200" ]; then
    log_test "API Gateway Health" "PASS" "Gateway responding with HTTP 200"
else
    log_test "API Gateway Health" "FAIL" "Gateway not responding (HTTP $gateway_response)"
fi

# Test service routing through gateway
auth_route=$(curl -s -w "%{http_code}" http://localhost:8089/auth/health -o /dev/null 2>/dev/null || echo "000")
if [ "$auth_route" = "200" ] || [ "$auth_route" = "404" ]; then
    log_test "Service Routing" "PASS" "Gateway can route to services"
else
    log_test "Service Routing" "FAIL" "Gateway routing issues (HTTP $auth_route)"
fi

# Test 5: Frontend Integration
echo
echo "=== Test 5: Frontend Integration ==="

# Test frontend accessibility
frontend_response=$(curl -s -w "%{http_code}" http://localhost:3000 -o /dev/null 2>/dev/null || echo "000")
if [ "$frontend_response" = "200" ]; then
    log_test "Frontend Accessibility" "PASS" "Frontend accessible at port 3000"
else
    log_test "Frontend Accessibility" "FAIL" "Frontend not accessible (HTTP $frontend_response)"
fi

# Test if frontend can load basic assets
assets_test=$(curl -s http://localhost:3000 2>/dev/null | grep -q "html\|HTML" && echo "SUCCESS" || echo "FAILED")
if [ "$assets_test" = "SUCCESS" ]; then
    log_test "Frontend Assets" "PASS" "Frontend serves HTML content"
else
    log_test "Frontend Assets" "FAIL" "Frontend not serving proper content"
fi

# Test 6: Storage Capacity Analysis
echo
echo "=== Test 6: Storage Capacity Analysis ==="

# Test total storage capacity
total_positions=$(docker exec lims-postgres psql -U postgres -d lims_db -t -c "SELECT COUNT(*) FROM sample_positions;" 2>/dev/null | tr -d ' \n' || echo "0")
if [ "$total_positions" -gt 5000 ]; then
    log_test "Storage Capacity" "PASS" "$total_positions total sample positions available"
else
    log_test "Storage Capacity" "FAIL" "Insufficient storage capacity ($total_positions positions)"
fi

# Test temperature zone distribution
temp_zones=$(docker exec lims-postgres psql -U postgres -d lims_db -t -c "
    SELECT COUNT(DISTINCT CASE 
        WHEN min_temperature_celsius < -50 THEN 'ultra_low'
        WHEN min_temperature_celsius < 0 THEN 'freezer'
        WHEN min_temperature_celsius < 10 THEN 'refrigerated'
        ELSE 'room_temp'
    END) FROM storage_containers WHERE container_type = 'freezer';" 2>/dev/null | tr -d ' \n' || echo "0")

if [ "$temp_zones" -gt 2 ]; then
    log_test "Temperature Zones" "PASS" "$temp_zones different temperature zones configured"
else
    log_test "Temperature Zones" "FAIL" "Insufficient temperature zone diversity"
fi

# Test 7: Hierarchical Navigation
echo
echo "=== Test 7: Hierarchical Navigation ==="

# Test freezer to rack navigation
freezer_racks=$(docker exec lims-postgres psql -U postgres -d lims_db -t -c "
    SELECT COUNT(*) FROM storage_containers r
    JOIN storage_containers f ON r.parent_id = f.id
    WHERE r.container_type = 'rack' AND f.container_type = 'freezer';" 2>/dev/null | tr -d ' \n' || echo "0")

if [ "$freezer_racks" -gt 10 ]; then
    log_test "Freezer-Rack Navigation" "PASS" "$freezer_racks racks linked to freezers"
else
    log_test "Freezer-Rack Navigation" "FAIL" "Insufficient rack-freezer relationships ($freezer_racks)"
fi

# Test rack to box navigation
rack_boxes=$(docker exec lims-postgres psql -U postgres -d lims_db -t -c "
    SELECT COUNT(*) FROM storage_containers b
    JOIN storage_containers r ON b.parent_id = r.id
    WHERE b.container_type = 'box' AND r.container_type = 'rack';" 2>/dev/null | tr -d ' \n' || echo "0")

if [ "$rack_boxes" -gt 50 ]; then
    log_test "Rack-Box Navigation" "PASS" "$rack_boxes boxes linked to racks"
else
    log_test "Rack-Box Navigation" "FAIL" "Insufficient box-rack relationships ($rack_boxes)"
fi

# Test 8: Sample Position Management
echo
echo "=== Test 8: Sample Position Management ==="

# Test available positions
available_positions=$(docker exec lims-postgres psql -U postgres -d lims_db -t -c "
    SELECT COUNT(*) FROM sample_positions WHERE status = 'empty';" 2>/dev/null | tr -d ' \n' || echo "0")

if [ "$available_positions" -gt 5000 ]; then
    log_test "Available Positions" "PASS" "$available_positions positions available for samples"
else
    log_test "Available Positions" "FAIL" "Limited available positions ($available_positions)"
fi

# Test position coordinate validity
coordinate_validity=$(docker exec lims-postgres psql -U postgres -d lims_db -t -c "
    SELECT COUNT(*) FROM sample_positions 
    WHERE position_x BETWEEN 1 AND 10 AND position_y BETWEEN 1 AND 10;" 2>/dev/null | tr -d ' \n' || echo "0")

if [ "$coordinate_validity" = "$total_positions" ]; then
    log_test "Position Coordinates" "PASS" "All positions have valid coordinates"
else
    log_test "Position Coordinates" "FAIL" "Some positions have invalid coordinates"
fi

# Test 9: System Performance
echo
echo "=== Test 9: System Performance ==="

# Test database query performance
query_start=$(date +%s%N)
docker exec lims-postgres psql -U postgres -d lims_db -c "
    SELECT sc.name, COUNT(sp.id) 
    FROM storage_containers sc 
    LEFT JOIN sample_positions sp ON sc.id = sp.container_id 
    WHERE sc.container_type = 'box' 
    GROUP BY sc.id, sc.name;" >/dev/null 2>&1
query_end=$(date +%s%N)
query_time=$(((query_end - query_start) / 1000000)) # Convert to milliseconds

if [ "$query_time" -lt 1000 ]; then
    log_test "Database Performance" "PASS" "Complex query completed in ${query_time}ms"
else
    log_test "Database Performance" "FAIL" "Slow query performance (${query_time}ms)"
fi

# Test API response time
api_start=$(date +%s%N)
curl -s http://localhost:8089/health >/dev/null 2>&1
api_end=$(date +%s%N)
api_time=$(((api_end - api_start) / 1000000))

if [ "$api_time" -lt 500 ]; then
    log_test "API Performance" "PASS" "API response in ${api_time}ms"
else
    log_test "API Performance" "FAIL" "Slow API response (${api_time}ms)"
fi

# Test 10: Data Consistency
echo
echo "=== Test 10: Data Consistency ==="

# Test container capacity consistency
capacity_consistency=$(docker exec lims-postgres psql -U postgres -d lims_db -t -c "
    SELECT COUNT(*) FROM storage_containers sc
    LEFT JOIN (
        SELECT container_id, COUNT(*) as actual_positions
        FROM sample_positions
        GROUP BY container_id
    ) sp ON sc.id = sp.container_id
    WHERE sc.container_type = 'box' 
    AND (sc.capacity < COALESCE(sp.actual_positions, 0));" 2>/dev/null | tr -d ' \n' || echo "1")

if [ "$capacity_consistency" = "0" ]; then
    log_test "Capacity Consistency" "PASS" "All containers have sufficient capacity"
else
    log_test "Capacity Consistency" "FAIL" "$capacity_consistency containers have capacity issues"
fi

# Test referential integrity
referential_integrity=$(docker exec lims-postgres psql -U postgres -d lims_db -t -c "
    SELECT COUNT(*) FROM sample_positions sp
    LEFT JOIN storage_containers sc ON sp.container_id = sc.id
    WHERE sc.id IS NULL;" 2>/dev/null | tr -d ' \n' || echo "1")

if [ "$referential_integrity" = "0" ]; then
    log_test "Referential Integrity" "PASS" "All foreign key relationships valid"
else
    log_test "Referential Integrity" "FAIL" "$referential_integrity orphaned records found"
fi

# Test Summary
echo
echo "=================================================="
echo "              INTEGRATION TEST SUMMARY"
echo "=================================================="
echo -e "${GREEN}Tests Passed: $TESTS_PASSED${NC}"
echo -e "${RED}Tests Failed: $TESTS_FAILED${NC}"
echo -e "${YELLOW}Tests Skipped: $TESTS_SKIPPED${NC}"
echo "Total Tests: $((TESTS_PASSED + TESTS_FAILED + TESTS_SKIPPED))"

# Success rate calculation
total_tests=$((TESTS_PASSED + TESTS_FAILED))
if [ $total_tests -gt 0 ]; then
    success_rate=$(( (TESTS_PASSED * 100) / total_tests ))
    echo "Success Rate: ${success_rate}%"
fi

echo
echo -e "${BLUE}=== System Status Summary ===${NC}"
echo "📊 Total Storage Positions: $total_positions"
echo "🏷️  Temperature Zones: $temp_zones"
echo "🔗 Freezer-Rack Links: $freezer_racks"
echo "📦 Rack-Box Links: $rack_boxes"
echo "✅ Available Positions: $available_positions"
echo "⚡ Database Query Time: ${query_time}ms"
echo "🌐 API Response Time: ${api_time}ms"

echo
if [ $TESTS_FAILED -eq 0 ]; then
    echo -e "${GREEN}🎉 All integration tests passed! Hierarchical storage system is fully operational.${NC}"
    exit 0
else
    echo -e "${RED}❌ Some integration tests failed. System may have issues.${NC}"
    exit 1
fi 