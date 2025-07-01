#!/bin/bash
# Test script for MCP Docker infrastructure

set -e

# Colors for output
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
RED='\033[0;31m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

echo -e "${BLUE}Testing MCP Docker Infrastructure${NC}"
echo "===================================="

# Track test results
TESTS_PASSED=0
TESTS_FAILED=0

# Function to run a test
run_test() {
    local test_name=$1
    local test_command=$2
    
    echo -n "Testing $test_name... "
    
    if eval "$test_command" > /dev/null 2>&1; then
        echo -e "${GREEN}✓ PASSED${NC}"
        ((TESTS_PASSED++))
    else
        echo -e "${RED}✗ FAILED${NC}"
        ((TESTS_FAILED++))
    fi
}

# Function to check if service is healthy
check_health() {
    local service=$1
    docker ps --filter "name=$service" --filter "health=healthy" --format "{{.Names}}" | grep -q "$service"
}

# 1. Check Docker is running
run_test "Docker daemon" "docker info"

# 2. Check Docker Compose
run_test "Docker Compose" "docker-compose version"

# 3. Check network exists
run_test "Docker network" "docker network ls | grep -q tracseq-network"

# 4. Check services are running
echo -e "\n${YELLOW}Checking service status...${NC}"

services=("consul" "mcp-proxy" "mcp-dashboard" "cognitive-assistant-mcp" "postgres" "redis" "ollama" "chromadb")

for service in "${services[@]}"; do
    run_test "$service running" "docker ps | grep -q $service"
done

# 5. Wait for services to be healthy
echo -e "\n${YELLOW}Waiting for services to be healthy...${NC}"
sleep 10

# 6. Check service health
echo -e "\n${YELLOW}Checking service health...${NC}"

run_test "Consul health" "check_health consul"
run_test "MCP Proxy health" "check_health mcp-proxy"
run_test "MCP Dashboard health" "check_health mcp-dashboard"
run_test "Cognitive Assistant MCP health" "check_health cognitive-assistant-mcp"

# 7. Check service endpoints
echo -e "\n${YELLOW}Testing service endpoints...${NC}"

# Test Consul UI
run_test "Consul UI (8500)" "curl -f -s http://localhost:8500/v1/status/leader"

# Test MCP Proxy metrics
run_test "MCP Proxy metrics (9590)" "curl -f -s http://localhost:9590/metrics | grep -q mcp_"

# Test MCP Dashboard
run_test "MCP Dashboard (7890)" "curl -f -s http://localhost:7890/health"

# Test PostgreSQL
run_test "PostgreSQL (5432)" "docker exec postgres pg_isready -U postgres"

# Test Redis
run_test "Redis (6379)" "docker exec redis redis-cli ping | grep -q PONG"

# Test Ollama
run_test "Ollama API (11434)" "curl -f -s http://localhost:11434/api/version"

# Test ChromaDB
run_test "ChromaDB (8000)" "curl -f -s http://localhost:8000/api/v1/heartbeat"

# 8. Test service discovery
echo -e "\n${YELLOW}Testing service discovery...${NC}"

# Check if services are registered in Consul
run_test "Services registered in Consul" "curl -s http://localhost:8500/v1/catalog/services | grep -q cognitive-assistant-mcp"

# 9. Test MCP WebSocket connectivity
echo -e "\n${YELLOW}Testing MCP WebSocket connectivity...${NC}"

# Create a simple WebSocket test
cat > /tmp/test_ws.py << 'EOF'
import asyncio
import websockets
import json
import sys

async def test_websocket():
    try:
        async with websockets.connect("ws://localhost:9500") as ws:
            # Send a ping
            await ws.send(json.dumps({"type": "ping", "id": 1}))
            # Wait for response
            response = await asyncio.wait_for(ws.recv(), timeout=5)
            data = json.loads(response)
            if "pong" in str(data).lower() or "healthy" in str(data).lower():
                sys.exit(0)
            else:
                sys.exit(1)
    except Exception as e:
        print(f"Error: {e}")
        sys.exit(1)

asyncio.run(test_websocket())
EOF

if command -v python3 > /dev/null 2>&1; then
    run_test "MCP WebSocket connection" "python3 /tmp/test_ws.py"
else
    echo -e "${YELLOW}Skipping WebSocket test (Python not available)${NC}"
fi

# 10. Test MCP tool call
echo -e "\n${YELLOW}Testing MCP tool calls...${NC}"

# Test calling the cognitive assistant
cat > /tmp/test_mcp_call.sh << 'EOF'
curl -s -X POST http://localhost:7890/test-service \
  -H "Content-Type: application/json" \
  -d '{"service": "cognitive_assistant", "method": "ping"}' \
  | grep -q "success"
EOF

run_test "MCP tool call via dashboard" "bash /tmp/test_mcp_call.sh"

# 11. Check logs for errors
echo -e "\n${YELLOW}Checking logs for errors...${NC}"

# Check for critical errors in logs
for service in "${services[@]}"; do
    if docker logs "$service" 2>&1 | tail -20 | grep -i "error\|fatal\|panic" > /dev/null 2>&1; then
        echo -e "${RED}✗ Errors found in $service logs${NC}"
        ((TESTS_FAILED++))
    else
        echo -e "${GREEN}✓ No critical errors in $service logs${NC}"
        ((TESTS_PASSED++))
    fi
done

# 12. Test data persistence
echo -e "\n${YELLOW}Testing data persistence...${NC}"

# Test if volumes are properly mounted
run_test "Cognitive logs volume" "docker exec cognitive-assistant-mcp ls /app/logs"
run_test "Consul data volume" "docker exec consul ls /consul/data"

# Summary
echo -e "\n${BLUE}Test Summary${NC}"
echo "============"
echo -e "Tests passed: ${GREEN}$TESTS_PASSED${NC}"
echo -e "Tests failed: ${RED}$TESTS_FAILED${NC}"

if [ $TESTS_FAILED -eq 0 ]; then
    echo -e "\n${GREEN}✓ All tests passed! MCP Docker infrastructure is working correctly.${NC}"
    exit 0
else
    echo -e "\n${RED}✗ Some tests failed. Please check the logs and configuration.${NC}"
    echo -e "\nDebug commands:"
    echo "  docker-compose -f docker-compose.with-mcp.yml logs"
    echo "  docker ps -a"
    echo "  docker network ls"
    exit 1
fi 