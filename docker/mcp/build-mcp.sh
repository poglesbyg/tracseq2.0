#!/bin/bash
# Build script for MCP Docker infrastructure

set -e

echo "Building MCP Docker Infrastructure..."

# Colors for output
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
RED='\033[0;31m'
NC='\033[0m' # No Color

# Get script directory
SCRIPT_DIR="$( cd "$( dirname "${BASH_SOURCE[0]}" )" && pwd )"
PROJECT_ROOT="$( cd "$SCRIPT_DIR/../.." && pwd )"

echo -e "${YELLOW}Building from project root: $PROJECT_ROOT${NC}"

# Create network if it doesn't exist
echo -e "${YELLOW}Creating Docker network...${NC}"
docker network create tracseq-network 2>/dev/null || echo "Network already exists"

# Build MCP Proxy Server
echo -e "${YELLOW}Building MCP Proxy Server...${NC}"
docker build -t tracseq/mcp-proxy:latest \
    -f "$PROJECT_ROOT/lims-ai/mcp-proxy/Dockerfile" \
    "$PROJECT_ROOT/lims-ai/mcp-proxy"

# Build MCP Dashboard
echo -e "${YELLOW}Building MCP Dashboard...${NC}"
docker build -t tracseq/mcp-dashboard:latest \
    -f "$PROJECT_ROOT/lims-ai/mcp-dashboard/Dockerfile" \
    "$PROJECT_ROOT/lims-ai/mcp-dashboard"

# Build Cognitive Assistant MCP
echo -e "${YELLOW}Building Cognitive Assistant MCP...${NC}"
docker build -t tracseq/cognitive-assistant-mcp:latest \
    -f "$PROJECT_ROOT/lims-ai/cognitive_assistant/Dockerfile.mcp" \
    "$PROJECT_ROOT/lims-ai/cognitive_assistant"

echo -e "${GREEN}✓ All MCP Docker images built successfully!${NC}"

# List built images
echo -e "\n${YELLOW}Built images:${NC}"
docker images | grep -E "tracseq/(mcp-|cognitive-)" | head -10

echo -e "\n${GREEN}To start the MCP infrastructure, run:${NC}"
echo "  cd $SCRIPT_DIR && docker-compose -f docker-compose.mcp.yml up -d" 