#!/bin/bash

echo "==============================================="
echo "     TracSeq 2.0 LIMS Services Status Check"
echo "==============================================="
echo ""

# Colors for output
GREEN='\033[0;32m'
RED='\033[0;31m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# Function to check service status
check_service() {
    local service_name=$1
    local container_name=$2
    local health_endpoint=$3
    local port=$4
    
    # Check if container is running
    if docker ps --format "{{.Names}}" | grep -q "^${container_name}$"; then
        # Get container status
        status=$(docker inspect -f '{{.State.Status}}' ${container_name} 2>/dev/null)
        health=$(docker inspect -f '{{.State.Health.Status}}' ${container_name} 2>/dev/null || echo "no health check")
        
        if [ "$status" = "running" ]; then
            if [ ! -z "$health_endpoint" ] && [ ! -z "$port" ]; then
                # Try to hit health endpoint
                if curl -s -f "http://localhost:${port}${health_endpoint}" > /dev/null 2>&1; then
                    echo -e "${GREEN}✓${NC} ${service_name}: Running (Health: ${health})"
                else
                    echo -e "${YELLOW}⚠${NC} ${service_name}: Running but health check failed"
                fi
            else
                echo -e "${GREEN}✓${NC} ${service_name}: Running (Health: ${health})"
            fi
        else
            echo -e "${RED}✗${NC} ${service_name}: Status: ${status}"
        fi
    else
        echo -e "${RED}✗${NC} ${service_name}: Not running"
    fi
}

echo "Infrastructure Services:"
echo "------------------------"
check_service "PostgreSQL" "lims-postgres" "" "5433"
check_service "Redis" "lims-redis" "" "6380"
echo ""

echo "Core Services:"
echo "--------------"
check_service "API Gateway" "lims-gateway" "/health" "8089"
check_service "Auth Service" "lims-auth" "/health" "8011"
check_service "Sample Service" "lims-samples" "/health" "8012"
check_service "Storage Service" "lims-storage" "/health" "8013"
check_service "Reports Service" "lims-reports" "/health" "8014"
echo ""

echo "AI Services:"
echo "------------"
check_service "RAG Service" "lims-rag" "/api/v1/health" "8100"
echo ""

echo "Frontend:"
echo "---------"
check_service "Frontend" "lims-frontend" "" "3000"
echo ""

# Show port mappings
echo "Port Mappings:"
echo "--------------"
echo "Frontend:        http://localhost:3000"
echo "API Gateway:     http://localhost:8089"
echo "Auth Service:    http://localhost:8011"
echo "Sample Service:  http://localhost:8012"
echo "Storage Service: http://localhost:8013"
echo "Reports Service: http://localhost:8014"
echo "RAG Service:     http://localhost:8100"
echo "PostgreSQL:      localhost:5433"
echo "Redis:           localhost:6380"
echo ""

# Check for exited containers
echo "Recently Exited Containers:"
echo "--------------------------"
docker ps -a --filter "status=exited" --filter "name=lims-" --format "table {{.Names}}\t{{.Status}}\t{{.RunningFor}}" | grep -v "NAMES" | head -5 || echo "None"
echo ""

echo "===============================================" 