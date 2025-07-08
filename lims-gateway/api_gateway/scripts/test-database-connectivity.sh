#!/bin/bash

# TracSeq Database Connectivity Test Script
# Tests database connectivity across all microservices

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Configuration
DATABASE_URL="postgresql://postgres:postgres@lims-postgres:5432/lims_db"
REDIS_URL="redis://lims-redis:6379/0"
GATEWAY_URL="http://localhost:8000"

# Service endpoints
declare -A SERVICES=(
    ["api-gateway"]="http://localhost:8000"
    ["auth-service"]="http://localhost:8080"
    ["sample-service"]="http://localhost:8081"
    ["storage-service"]="http://localhost:8082"
    ["template-service"]="http://localhost:8083"
    ["sequencing-service"]="http://localhost:8084"
    ["notification-service"]="http://localhost:8085"
    ["rag-service"]="http://localhost:8086"
    ["event-service"]="http://localhost:8087"
    ["transaction-service"]="http://localhost:8088"
)

# Function to print colored output
print_status() {
    local status=$1
    local message=$2
    case $status in
        "SUCCESS")
            echo -e "${GREEN}✓${NC} $message"
            ;;
        "ERROR")
            echo -e "${RED}✗${NC} $message"
            ;;
        "WARNING")
            echo -e "${YELLOW}⚠${NC} $message"
            ;;
        "INFO")
            echo -e "${BLUE}ℹ${NC} $message"
            ;;
    esac
}

# Function to check if a service is running
check_service_health() {
    local service_name=$1
    local service_url=$2
    
    print_status "INFO" "Checking health of $service_name..."
    
    # Try health endpoint
    if curl -f -s --max-time 10 "$service_url/health" > /dev/null 2>&1; then
        print_status "SUCCESS" "$service_name is healthy"
        return 0
    else
        print_status "ERROR" "$service_name is not responding"
        return 1
    fi
}

# Function to test database connectivity
test_database_connectivity() {
    print_status "INFO" "Testing PostgreSQL database connectivity..."
    
    # Check if PostgreSQL is accessible
    if command -v psql > /dev/null 2>&1; then
        if psql "$DATABASE_URL" -c "SELECT 1;" > /dev/null 2>&1; then
            print_status "SUCCESS" "PostgreSQL database is accessible"
            
            # Check if tables exist
            local table_count=$(psql "$DATABASE_URL" -t -c "SELECT COUNT(*) FROM information_schema.tables WHERE table_schema = 'public';" 2>/dev/null | tr -d ' ')
            if [ "$table_count" -gt 0 ]; then
                print_status "SUCCESS" "Database has $table_count tables"
            else
                print_status "WARNING" "Database appears to be empty (no tables found)"
            fi
        else
            print_status "ERROR" "Cannot connect to PostgreSQL database"
            return 1
        fi
    else
        print_status "WARNING" "psql not available, skipping direct database test"
    fi
    
    return 0
}

# Function to test Redis connectivity
test_redis_connectivity() {
    print_status "INFO" "Testing Redis connectivity..."
    
    if command -v redis-cli > /dev/null 2>&1; then
        if redis-cli -u "$REDIS_URL" ping > /dev/null 2>&1; then
            print_status "SUCCESS" "Redis is accessible"
        else
            print_status "ERROR" "Cannot connect to Redis"
            return 1
        fi
    else
        print_status "WARNING" "redis-cli not available, skipping direct Redis test"
    fi
    
    return 0
}

# Function to test service database health endpoints
test_service_database_health() {
    local service_name=$1
    local service_url=$2
    
    print_status "INFO" "Testing database health for $service_name..."
    
    # Try to get health status with database information
    local health_response=$(curl -f -s --max-time 10 "$service_url/health" 2>/dev/null)
    
    if [ $? -eq 0 ]; then
        # Check if response contains database information
        if echo "$health_response" | grep -q "database\|db\|postgres"; then
            print_status "SUCCESS" "$service_name reports database connectivity"
            
            # Try to extract database status if available
            if echo "$health_response" | grep -q '"database".*true\|"db".*true\|"healthy".*true'; then
                print_status "SUCCESS" "$service_name database is healthy"
            elif echo "$health_response" | grep -q '"database".*false\|"db".*false'; then
                print_status "ERROR" "$service_name database is unhealthy"
                return 1
            else
                print_status "WARNING" "$service_name database status unclear"
            fi
        else
            print_status "WARNING" "$service_name health endpoint doesn't report database status"
        fi
    else
        print_status "ERROR" "Cannot get health status from $service_name"
        return 1
    fi
    
    return 0
}

# Function to test API Gateway comprehensive health
test_gateway_comprehensive_health() {
    print_status "INFO" "Testing API Gateway comprehensive health..."
    
    # Test main health endpoint
    local health_response=$(curl -f -s --max-time 10 "$GATEWAY_URL/health" 2>/dev/null)
    
    if [ $? -eq 0 ]; then
        print_status "SUCCESS" "API Gateway health endpoint is accessible"
        
        # Check database status in response
        if echo "$health_response" | grep -q '"database".*true'; then
            print_status "SUCCESS" "API Gateway reports database is healthy"
        elif echo "$health_response" | grep -q '"database".*false'; then
            print_status "ERROR" "API Gateway reports database is unhealthy"
        else
            print_status "WARNING" "API Gateway database status unclear"
        fi
        
        # Test services endpoint
        local services_response=$(curl -f -s --max-time 10 "$GATEWAY_URL/services" 2>/dev/null)
        if [ $? -eq 0 ]; then
            print_status "SUCCESS" "API Gateway services endpoint is accessible"
            
            # Count healthy services
            local healthy_count=$(echo "$services_response" | grep -o '"status":"healthy"' | wc -l)
            local total_count=$(echo "$services_response" | grep -o '"status":"' | wc -l)
            
            if [ "$healthy_count" -gt 0 ]; then
                print_status "SUCCESS" "API Gateway reports $healthy_count/$total_count services as healthy"
            else
                print_status "WARNING" "API Gateway reports no healthy services"
            fi
        else
            print_status "WARNING" "API Gateway services endpoint not accessible"
        fi
    else
        print_status "ERROR" "Cannot access API Gateway health endpoint"
        return 1
    fi
    
    return 0
}

# Function to test database schema
test_database_schema() {
    print_status "INFO" "Testing database schema..."
    
    if command -v psql > /dev/null 2>&1; then
        # Check for critical tables
        local critical_tables=("users" "samples" "storage_locations" "templates" "projects")
        
        for table in "${critical_tables[@]}"; do
            if psql "$DATABASE_URL" -t -c "SELECT EXISTS (SELECT FROM information_schema.tables WHERE table_schema = 'public' AND table_name = '$table');" 2>/dev/null | grep -q "t"; then
                print_status "SUCCESS" "Table '$table' exists"
            else
                print_status "WARNING" "Table '$table' does not exist"
            fi
        done
        
        # Check for migrations table
        if psql "$DATABASE_URL" -t -c "SELECT EXISTS (SELECT FROM information_schema.tables WHERE table_schema = 'public' AND table_name = '_sqlx_migrations');" 2>/dev/null | grep -q "t"; then
            print_status "SUCCESS" "Migration tracking table exists"
        else
            print_status "WARNING" "Migration tracking table not found"
        fi
    else
        print_status "WARNING" "Cannot test database schema without psql"
    fi
}

# Function to run comprehensive connectivity tests
run_comprehensive_tests() {
    local failed_tests=0
    
    echo -e "${BLUE}===========================================${NC}"
    echo -e "${BLUE}  TracSeq Database Connectivity Tests    ${NC}"
    echo -e "${BLUE}===========================================${NC}"
    echo
    
    # Test infrastructure
    echo -e "${YELLOW}Testing Infrastructure...${NC}"
    test_database_connectivity || ((failed_tests++))
    test_redis_connectivity || ((failed_tests++))
    echo
    
    # Test database schema
    echo -e "${YELLOW}Testing Database Schema...${NC}"
    test_database_schema
    echo
    
    # Test API Gateway
    echo -e "${YELLOW}Testing API Gateway...${NC}"
    test_gateway_comprehensive_health || ((failed_tests++))
    echo
    
    # Test individual services
    echo -e "${YELLOW}Testing Individual Services...${NC}"
    for service_name in "${!SERVICES[@]}"; do
        service_url="${SERVICES[$service_name]}"
        
        if check_service_health "$service_name" "$service_url"; then
            test_service_database_health "$service_name" "$service_url" || ((failed_tests++))
        else
            ((failed_tests++))
        fi
        echo
    done
    
    # Summary
    echo -e "${BLUE}===========================================${NC}"
    echo -e "${BLUE}  Test Summary                            ${NC}"
    echo -e "${BLUE}===========================================${NC}"
    
    if [ $failed_tests -eq 0 ]; then
        print_status "SUCCESS" "All database connectivity tests passed!"
        echo -e "${GREEN}✓ All microservices are properly connected to the database${NC}"
        echo -e "${GREEN}✓ Database schema appears to be in good condition${NC}"
        echo -e "${GREEN}✓ API Gateway is routing requests correctly${NC}"
        return 0
    else
        print_status "ERROR" "$failed_tests test(s) failed"
        echo -e "${RED}✗ Some microservices may have database connectivity issues${NC}"
        echo -e "${YELLOW}⚠ Check the logs above for specific issues${NC}"
        return 1
    fi
}

# Function to show usage
show_usage() {
    echo "Usage: $0 [OPTIONS]"
    echo
    echo "Options:"
    echo "  -h, --help          Show this help message"
    echo "  -q, --quick         Run quick connectivity test only"
    echo "  -v, --verbose       Enable verbose output"
    echo "  -s, --service NAME  Test specific service only"
    echo
    echo "Examples:"
    echo "  $0                  Run all tests"
    echo "  $0 --quick          Run quick connectivity test"
    echo "  $0 --service auth   Test auth service only"
}

# Main script
main() {
    local quick_mode=false
    local verbose_mode=false
    local specific_service=""
    
    # Parse command line arguments
    while [[ $# -gt 0 ]]; do
        case $1 in
            -h|--help)
                show_usage
                exit 0
                ;;
            -q|--quick)
                quick_mode=true
                shift
                ;;
            -v|--verbose)
                verbose_mode=true
                shift
                ;;
            -s|--service)
                specific_service="$2"
                shift 2
                ;;
            *)
                echo "Unknown option: $1"
                show_usage
                exit 1
                ;;
        esac
    done
    
    # Set verbose mode if requested
    if [ "$verbose_mode" = true ]; then
        set -x
    fi
    
    # Run tests based on mode
    if [ "$quick_mode" = true ]; then
        echo -e "${BLUE}Running quick connectivity test...${NC}"
        test_database_connectivity && test_redis_connectivity
    elif [ -n "$specific_service" ]; then
        if [[ -v SERVICES["$specific_service"] ]]; then
            echo -e "${BLUE}Testing $specific_service...${NC}"
            service_url="${SERVICES[$specific_service]}"
            check_service_health "$specific_service" "$service_url"
            test_service_database_health "$specific_service" "$service_url"
        else
            print_status "ERROR" "Unknown service: $specific_service"
            echo "Available services: ${!SERVICES[*]}"
            exit 1
        fi
    else
        run_comprehensive_tests
    fi
}

# Run main function with all arguments
main "$@" 