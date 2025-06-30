#!/bin/bash

echo "========================================================="
echo "     TracSeq 2.0 LIMS - Complete System Status Check"
echo "========================================================="
echo ""

# Colors for output
GREEN='\033[0;32m'
RED='\033[0;31m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
CYAN='\033[0;36m'
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

# Count running services
TOTAL_SERVICES=0
RUNNING_SERVICES=0

count_service() {
    ((TOTAL_SERVICES++))
    if docker ps --format "{{.Names}}" | grep -q "^$1$"; then
        ((RUNNING_SERVICES++))
    fi
}

echo -e "${CYAN}Infrastructure Services:${NC}"
echo "------------------------"
check_service "PostgreSQL" "lims-postgres" "" "5433"
check_service "Redis" "lims-redis" "" "6380"
count_service "lims-postgres"
count_service "lims-redis"
echo ""

echo -e "${CYAN}Core Services:${NC}"
echo "--------------"
check_service "API Gateway" "lims-gateway" "/health" "8089"
check_service "Auth Service" "lims-auth" "/health" "8011"
check_service "Sample Service" "lims-samples" "/health" "8012"
check_service "Storage Service" "lims-storage" "/health" "8013"
check_service "Reports Service" "lims-reports" "/health" "8014"
count_service "lims-gateway"
count_service "lims-auth"
count_service "lims-samples"
count_service "lims-storage"
count_service "lims-reports"
echo ""

echo -e "${CYAN}AI Infrastructure:${NC}"
echo "------------------"
check_service "Ollama LLM Server" "lims-ollama" "/api/version" "11434"
check_service "Enhanced RAG Service" "lims-rag" "/api/v1/health" "8100"
check_service "Cognitive Assistant" "lims-cognitive" "/health" "8015"
check_service "Feature Store" "lims-feature-store" "/health" "8090"
count_service "lims-ollama"
count_service "lims-rag"
count_service "lims-cognitive"
count_service "lims-feature-store"
echo ""

echo -e "${CYAN}Frontend:${NC}"
echo "---------"
check_service "Frontend" "lims-frontend" "" "3000"
count_service "lims-frontend"
echo ""

# Summary statistics
echo -e "${BLUE}System Summary:${NC}"
echo "---------------"
echo -e "Total Services: ${TOTAL_SERVICES}"
echo -e "Running Services: ${GREEN}${RUNNING_SERVICES}${NC}"
echo -e "Success Rate: ${GREEN}$(( RUNNING_SERVICES * 100 / TOTAL_SERVICES ))%${NC}"
echo ""

# Show system capabilities based on running services
echo -e "${BLUE}Available Capabilities:${NC}"
echo "----------------------"

# Basic capabilities
if docker ps --format "{{.Names}}" | grep -q "lims-gateway" && docker ps --format "{{.Names}}" | grep -q "lims-auth"; then
    echo -e "${GREEN}✓${NC} Authentication & Authorization"
else
    echo -e "${RED}✗${NC} Authentication & Authorization"
fi

if docker ps --format "{{.Names}}" | grep -q "lims-samples"; then
    echo -e "${GREEN}✓${NC} Sample Management"
else
    echo -e "${RED}✗${NC} Sample Management"
fi

if docker ps --format "{{.Names}}" | grep -q "lims-storage"; then
    echo -e "${GREEN}✓${NC} Storage Tracking"
else
    echo -e "${RED}✗${NC} Storage Tracking"
fi

if docker ps --format "{{.Names}}" | grep -q "lims-reports"; then
    echo -e "${GREEN}✓${NC} Report Generation"
else
    echo -e "${RED}✗${NC} Report Generation"
fi

# AI capabilities
if docker ps --format "{{.Names}}" | grep -q "lims-rag" && docker ps --format "{{.Names}}" | grep -q "lims-ollama"; then
    echo -e "${GREEN}✓${NC} AI Document Processing"
    echo -e "${GREEN}✓${NC} Intelligent Queries"
else
    echo -e "${RED}✗${NC} AI Document Processing"
    echo -e "${RED}✗${NC} Intelligent Queries"
fi

if docker ps --format "{{.Names}}" | grep -q "lims-cognitive"; then
    echo -e "${GREEN}✓${NC} Cognitive Laboratory Assistant"
else
    echo -e "${RED}✗${NC} Cognitive Laboratory Assistant"
fi

if docker ps --format "{{.Names}}" | grep -q "lims-feature-store"; then
    echo -e "${GREEN}✓${NC} ML Feature Management"
else
    echo -e "${RED}✗${NC} ML Feature Management"
fi

echo ""

# Quick access URLs
echo -e "${BLUE}Quick Access URLs:${NC}"
echo "------------------"
echo "Frontend:            http://localhost:3000"
echo "API Gateway:         http://localhost:8089"
echo "RAG Service:         http://localhost:8100"
echo "Cognitive Assistant: http://localhost:8015"
echo "Feature Store:       http://localhost:8090"
echo ""

# Memory usage summary
echo -e "${BLUE}Resource Usage Summary:${NC}"
echo "----------------------"
if docker ps -q | wc -l > 0; then
    docker stats --no-stream --format "table {{.Name}}\t{{.CPUPerc}}\t{{.MemUsage}}" | head -15
fi

echo ""
echo "=========================================================" 