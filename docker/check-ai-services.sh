#!/bin/bash

echo "==============================================="
echo "     TracSeq 2.0 AI Services Status Check"
echo "==============================================="
echo ""

# Colors for output
GREEN='\033[0;32m'
RED='\033[0;31m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
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

echo -e "${BLUE}Core AI Infrastructure:${NC}"
echo "------------------------"
check_service "Ollama LLM Server" "lims-ollama" "/api/version" "11434"
check_service "Enhanced RAG Service" "lims-rag" "/api/v1/health" "8100"
echo ""

echo -e "${BLUE}Cognitive Services:${NC}"
echo "-------------------"
check_service "Cognitive Assistant" "lims-cognitive" "/health" "8015"
echo ""

echo -e "${BLUE}ML Platform Services:${NC}"
echo "---------------------"
check_service "Feature Store" "lims-feature-store" "/health" "8090"
check_service "Model Serving" "lims-model-serving" "/health" "8091"
check_service "MLOps Pipeline" "lims-mlops" "/health" "8092"
check_service "AutoML Service" "lims-automl" "/health" "8093"
check_service "ML Worker" "lims-ml-worker" "" ""
echo ""

# Check Ollama models
echo -e "${BLUE}Ollama Models:${NC}"
echo "--------------"
if docker ps --format "{{.Names}}" | grep -q "^lims-ollama$"; then
    models=$(docker exec lims-ollama ollama list 2>/dev/null | grep -v "NAME" | awk '{print $1}' | tr '\n' ', ' | sed 's/,$//')
    if [ -z "$models" ]; then
        echo -e "${YELLOW}⚠${NC} No models loaded"
    else
        echo -e "${GREEN}✓${NC} Models loaded: $models"
    fi
else
    echo -e "${RED}✗${NC} Ollama not running"
fi
echo ""

# Show AI service endpoints
echo -e "${BLUE}AI Service Endpoints:${NC}"
echo "--------------------"
echo "Ollama API:          http://localhost:11434"
echo "RAG Service:         http://localhost:8100"
echo "Cognitive Assistant: http://localhost:8015"
echo "Feature Store:       http://localhost:8090"
echo "Model Serving:       http://localhost:8091"
echo "MLOps Pipeline:      http://localhost:8092"
echo "AutoML Service:      http://localhost:8093"
echo ""

# Show AI feature status
echo -e "${BLUE}AI Features Status:${NC}"
echo "-------------------"
# Check if RAG service can process documents
if curl -s -f "http://localhost:8100/api/v1/health" > /dev/null 2>&1; then
    echo -e "${GREEN}✓${NC} Document Processing: Available"
    echo -e "${GREEN}✓${NC} Intelligent Queries: Available"
else
    echo -e "${RED}✗${NC} Document Processing: Unavailable"
    echo -e "${RED}✗${NC} Intelligent Queries: Unavailable"
fi

# Check storage AI features
if docker logs lims-storage 2>&1 | grep -q "AI features disabled"; then
    echo -e "${YELLOW}⚠${NC} Storage Optimization AI: Disabled"
    echo -e "${YELLOW}⚠${NC} Predictive Analytics: Disabled"
else
    echo -e "${GREEN}✓${NC} Storage Optimization AI: Enabled"
    echo -e "${GREEN}✓${NC} Predictive Analytics: Enabled"
fi
echo ""

# Memory and resource usage for AI services
echo -e "${BLUE}AI Service Resources:${NC}"
echo "--------------------"
if docker ps --format "{{.Names}}" | grep -q "lims-ollama\|lims-rag\|lims-cognitive\|lims-feature-store\|lims-model-serving\|lims-mlops\|lims-automl"; then
    docker stats --no-stream --format "table {{.Name}}\t{{.CPUPerc}}\t{{.MemUsage}}" | grep -E "NAME|lims-ollama|lims-rag|lims-cognitive|lims-feature-store|lims-model-serving|lims-mlops|lims-automl"
fi
echo ""

echo "===============================================" 