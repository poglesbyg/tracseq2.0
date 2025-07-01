#!/bin/bash

echo "🚀 TracSeq 2.0 - MCP Setup Script"
echo "================================="
echo ""

# Colors
GREEN='\033[0;32m'
BLUE='\033[0;34m'
YELLOW='\033[1;33m'
RED='\033[0;31m'
NC='\033[0m'

# Check Python version
echo -e "${BLUE}Checking Python version...${NC}"
python_version=$(python3 --version 2>&1 | awk '{print $2}')
echo "Python version: $python_version"

# Create virtual environment if needed
if [ ! -d "venv-mcp" ]; then
    echo -e "\n${BLUE}Creating virtual environment for MCP...${NC}"
    python3 -m venv venv-mcp
    echo -e "${GREEN}✓ Virtual environment created${NC}"
fi

# Activate virtual environment
echo -e "\n${BLUE}Activating virtual environment...${NC}"
source venv-mcp/bin/activate || {
    echo -e "${RED}Failed to activate virtual environment${NC}"
    exit 1
}

# Install MCP dependencies
echo -e "\n${BLUE}Installing MCP dependencies...${NC}"
pip install --upgrade pip > /dev/null 2>&1
pip install fastmcp anthropic openai httpx pydantic > /dev/null 2>&1

if pip show fastmcp > /dev/null 2>&1; then
    echo -e "${GREEN}✓ FastMCP installed successfully${NC}"
    fastmcp_version=$(pip show fastmcp | grep Version | awk '{print $2}')
    echo "  Version: $fastmcp_version"
else
    echo -e "${RED}✗ Failed to install FastMCP${NC}"
    exit 1
fi

# Test MCP functionality
echo -e "\n${BLUE}Testing MCP functionality...${NC}"

# Create a simple test script
cat > test_mcp_basic.py << 'EOF'
#!/usr/bin/env python3
"""Quick MCP functionality test"""

try:
    from fastmcp import FastMCP, Context
    print("✓ FastMCP import successful")
    
    # Create a simple MCP server
    mcp = FastMCP("Test Server")
    
    @mcp.tool
    async def test_tool(message: str, ctx: Context) -> str:
        return f"MCP received: {message}"
    
    print("✓ MCP server created")
    print("✓ Tool registered")
    print("\nMCP is ready for integration!")
    
except ImportError as e:
    print(f"✗ Import error: {e}")
    exit(1)
except Exception as e:
    print(f"✗ Error: {e}")
    exit(1)
EOF

python test_mcp_basic.py
rm test_mcp_basic.py

# Show next steps
echo -e "\n${GREEN}===============================================${NC}"
echo -e "${GREEN}MCP Setup Complete!${NC}"
echo -e "${GREEN}===============================================${NC}"
echo ""
echo "Next steps:"
echo "1. Activate the MCP environment:"
echo -e "   ${YELLOW}source venv-mcp/bin/activate${NC}"
echo ""
echo "2. Test existing MCP services:"
echo -e "   ${YELLOW}python lims-ai/enhanced_rag_service/fastmcp_enhanced_rag_server.py --http${NC}"
echo ""
echo "3. Run the integration example:"
echo -e "   ${YELLOW}python scripts/mcp-integration-example.py${NC}"
echo ""
echo "4. Read the documentation:"
echo -e "   ${YELLOW}docs/MCP_INTEGRATION_STRATEGY.md${NC}"
echo -e "   ${YELLOW}docs/MCP_QUICK_REFERENCE.md${NC}"
echo ""
echo -e "${BLUE}MCP will enhance TracSeq 2.0 with:${NC}"
echo "  • Better AI model coordination"
echo "  • Unified service communication"
echo "  • Built-in progress tracking"
echo "  • Context-aware conversations"
echo "" 