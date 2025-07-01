#!/bin/bash

# TracSeq 2.0 Deployment Status Check Script

set -e

# Colors for output
GREEN='\033[0;32m'
RED='\033[0;31m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

echo -e "${BLUE}====================================================${NC}"
echo -e "${BLUE}     TracSeq 2.0 - Deployment Status Check${NC}"
echo -e "${BLUE}====================================================${NC}"
echo ""

# Function to check service status
check_service() {
    local service_name=$1
    local expected_port=$2
    
    # Check if container is running
    if docker ps --format "{{.Names}}" | grep -q "^$service_name$"; then
        # Get container status
        status=$(docker inspect -f '{{.State.Status}}' "$service_name" 2>/dev/null || echo "unknown")
        
        # Check health status if available
        health=$(docker inspect -f '{{.State.Health.Status}}' "$service_name" 2>/dev/null || echo "no health check")
        
        # Get port mapping
        ports=$(docker port "$service_name" 2>/dev/null | head -1 || echo "no ports exposed")
        
        if [[ "$health" == "healthy" ]]; then
            echo -e "${GREEN}✅ $service_name${NC} - Running ($health) - Port: $ports"
        elif [[ "$status" == "running" ]]; then
            echo -e "${YELLOW}⚠️  $service_name${NC} - Running (health: $health) - Port: $ports"
        else
            echo -e "${RED}❌ $service_name${NC} - Status: $status"
        fi
    else
        # Check if container exists but is not running
        if docker ps -a --format "{{.Names}}" | grep -q "^$service_name$"; then
            status=$(docker inspect -f '{{.State.Status}}' "$service_name" 2>/dev/null || echo "unknown")
            exit_code=$(docker inspect -f '{{.State.ExitCode}}' "$service_name" 2>/dev/null || echo "N/A")
            echo -e "${RED}❌ $service_name${NC} - Status: $status (Exit Code: $exit_code)"
        else
            echo -e "${RED}❌ $service_name${NC} - Not deployed"
        fi
    fi
}

# Infrastructure Services
echo -e "${BLUE}Infrastructure Services:${NC}"
echo "------------------------"
check_service "tracseq-postgres-primary" "5432"
check_service "tracseq-redis-primary" "6379"
echo ""

# Core Services
echo -e "${BLUE}Core Services:${NC}"
echo "------------------------"
check_service "tracseq-auth-service" "8080"
check_service "tracseq-sample-service" "8081"
check_service "tracseq-template-service" "8083"
check_service "tracseq-notification-service" "8085"
check_service "tracseq-sequencing-service" "8084"
check_service "tracseq-transaction-service" "8088"
check_service "tracseq-event-service" "8087"
check_service "tracseq-api-gateway" "8089"
echo ""

# Additional Services
echo -e "${BLUE}Additional Services:${NC}"
echo "------------------------"
check_service "tracseq-storage-service" "8082"
check_service "tracseq-library-details-service" "8086"
check_service "tracseq-qaqc-service" "8090"
check_service "tracseq-reports-service" "8091"
check_service "tracseq-spreadsheet-versioning-service" "8092"
echo ""

# Overall Health Summary
echo -e "${BLUE}====================================================${NC}"
echo -e "${BLUE}Overall Health Summary:${NC}"
echo -e "${BLUE}====================================================${NC}"

total_services=$(docker ps -a --filter "name=tracseq-" --format "{{.Names}}" | wc -l | tr -d ' ')
running_services=$(docker ps --filter "name=tracseq-" --format "{{.Names}}" | wc -l | tr -d ' ')
healthy_services=$(docker ps --filter "name=tracseq-" --format "{{.State}}" | grep -c "healthy" || echo "0")

echo -e "Total Services Deployed: ${BLUE}$total_services${NC}"
echo -e "Running Services: ${GREEN}$running_services${NC}"
echo -e "Healthy Services: ${GREEN}$healthy_services${NC}"
echo -e "Failed/Stopped Services: ${RED}$((total_services - running_services))${NC}"
echo ""

# Show logs for failed services
if [ "$((total_services - running_services))" -gt 0 ]; then
    echo -e "${YELLOW}Failed Service Logs:${NC}"
    echo "------------------------"
    docker ps -a --filter "name=tracseq-" --format "{{.Names}} {{.State}}" | grep -v "running" | while read service_info; do
        service_name=$(echo $service_info | awk '{print $1}')
        if [ ! -z "$service_name" ]; then
            echo -e "${RED}$service_name:${NC}"
            docker logs --tail 5 "$service_name" 2>&1 | sed 's/^/  /'
            echo ""
        fi
    done
fi

echo -e "${BLUE}====================================================${NC}" 