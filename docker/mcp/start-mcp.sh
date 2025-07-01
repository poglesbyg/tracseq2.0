#!/bin/bash
# Start script for MCP-enabled TracSeq 2.0

set -e

# Colors for output
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
RED='\033[0;31m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

echo -e "${BLUE}Starting TracSeq 2.0 with MCP Integration${NC}"
echo "================================================"

# Get script directory
SCRIPT_DIR="$( cd "$( dirname "${BASH_SOURCE[0]}" )" && pwd )"
DOCKER_DIR="$( cd "$SCRIPT_DIR/.." && pwd )"

# Check if Docker is running
if ! docker info > /dev/null 2>&1; then
    echo -e "${RED}Docker is not running. Please start Docker first.${NC}"
    exit 1
fi

# Build MCP images if needed
if [[ "$1" == "--build" ]] || [[ "$1" == "-b" ]]; then
    echo -e "${YELLOW}Building MCP Docker images...${NC}"
    "$SCRIPT_DIR/build-mcp.sh"
    echo ""
fi

# Create network if it doesn't exist
echo -e "${YELLOW}Creating Docker network...${NC}"
docker network create tracseq-network 2>/dev/null || echo "Network already exists"

# Start services
echo -e "${YELLOW}Starting MCP infrastructure...${NC}"
cd "$DOCKER_DIR"

# Start base services first
echo -e "${YELLOW}Starting base services (Postgres, Redis, Ollama, ChromaDB)...${NC}"
docker-compose -f docker-compose.with-mcp.yml up -d postgres redis ollama chromadb

# Wait for base services
echo -e "${YELLOW}Waiting for base services to be ready...${NC}"
sleep 10

# Start MCP infrastructure
echo -e "${YELLOW}Starting MCP services...${NC}"
docker-compose -f docker-compose.with-mcp.yml up -d consul mcp-proxy

# Wait for MCP proxy
echo -e "${YELLOW}Waiting for MCP proxy to be ready...${NC}"
sleep 5

# Start MCP services
echo -e "${YELLOW}Starting MCP-enabled services...${NC}"
docker-compose -f docker-compose.with-mcp.yml up -d cognitive-assistant-mcp mcp-dashboard

# Start application services
echo -e "${YELLOW}Starting application services...${NC}"
docker-compose -f docker-compose.with-mcp.yml up -d

# Show status
echo -e "\n${GREEN}✓ TracSeq 2.0 with MCP is starting up!${NC}"
echo -e "\n${BLUE}Service URLs:${NC}"
echo -e "  • MCP Dashboard:      ${GREEN}http://localhost:7890${NC}"
echo -e "  • MCP Proxy:          ${GREEN}ws://localhost:9500${NC}"
echo -e "  • Consul UI:          ${GREEN}http://localhost:8500${NC}"
echo -e "  • Lab Manager:        ${GREEN}http://localhost:8080${NC}"
echo -e "  • Cognitive Assistant: ${GREEN}http://localhost:8015${NC}"
echo -e "  • Enhanced Storage:    ${GREEN}http://localhost:8005${NC}"
echo -e "  • Enhanced RAG:        ${GREEN}http://localhost:8100${NC}"

echo -e "\n${YELLOW}Checking service health...${NC}"
sleep 5

# Check service health
services=("mcp-proxy" "mcp-dashboard" "cognitive-assistant-mcp" "consul")
all_healthy=true

for service in "${services[@]}"; do
    if docker ps --format "table {{.Names}}\t{{.Status}}" | grep -q "$service.*healthy"; then
        echo -e "  ✓ $service: ${GREEN}healthy${NC}"
    else
        echo -e "  ✗ $service: ${RED}not healthy yet${NC}"
        all_healthy=false
    fi
done

if [ "$all_healthy" = true ]; then
    echo -e "\n${GREEN}All MCP services are healthy!${NC}"
else
    echo -e "\n${YELLOW}Some services are still starting. Check again with:${NC}"
    echo "  docker ps"
fi

echo -e "\n${BLUE}Useful commands:${NC}"
echo "  • View logs:        docker-compose -f docker-compose.with-mcp.yml logs -f [service-name]"
echo "  • Stop all:         docker-compose -f docker-compose.with-mcp.yml down"
echo "  • View MCP metrics: curl http://localhost:9590/metrics"
echo "  • MCP Dashboard:    open http://localhost:7890" 