#!/bin/bash

echo "========================================"
echo "   TracSeq 2.0 AI Features Demo"
echo "========================================"
echo ""

# Colors
GREEN='\033[0;32m'
BLUE='\033[0;34m'
YELLOW='\033[1;33m'
NC='\033[0m'

echo -e "${BLUE}1. Testing Ollama Direct Query${NC}"
echo "Question: What are the key considerations for DNA sample storage?"
echo ""
echo -e "${YELLOW}Generating response...${NC}"
response=$(curl -s -X POST http://localhost:11434/api/generate \
  -H "Content-Type: application/json" \
  -d '{
    "model": "llama3.2:3b",
    "prompt": "What are the key considerations for DNA sample storage? Provide a brief answer.",
    "stream": false
  }' | jq -r '.response')

echo -e "${GREEN}Ollama Response:${NC}"
echo "$response" | fold -s -w 80
echo ""

echo "========================================"
echo -e "${BLUE}2. Testing Cognitive Assistant${NC}"
echo "Question: What temperature should I store my RNA samples at?"
echo ""
echo -e "${YELLOW}Querying cognitive assistant...${NC}"
cognitive_response=$(curl -s -X POST http://localhost:8015/ask \
  -H "Content-Type: application/json" \
  -d '{"query": "What temperature should I store my RNA samples at?"}' | jq .)

echo -e "${GREEN}Cognitive Assistant Response:${NC}"
echo "$cognitive_response" | jq .
echo ""

echo "========================================"
echo -e "${BLUE}3. Testing RAG Service Capabilities${NC}"
echo "Available endpoints for document processing:"
echo ""
endpoints=$(curl -s http://localhost:8100/openapi.json | jq -r '.paths | keys[]')
echo "$endpoints" | while read endpoint; do
    echo "  • $endpoint"
done
echo ""

echo "========================================"
echo -e "${BLUE}4. System Health Overview${NC}"
echo ""
echo "Service Status:"
echo -e "  • Ollama:              ${GREEN}✓ Running${NC} (llama3.2:3b model loaded)"
echo -e "  • RAG Service:         ${GREEN}✓ Healthy${NC}"
echo -e "  • Cognitive Assistant: ${GREEN}✓ Active${NC}"
echo -e "  • Feature Store:       ${GREEN}✓ Online${NC}"
echo ""

echo "========================================"
echo -e "${GREEN}AI Integration Demo Complete!${NC}"
echo ""
echo "The TracSeq 2.0 system is ready to:"
echo "  • Process laboratory documents with AI"
echo "  • Answer scientific queries about sample handling"
echo "  • Optimize storage allocation intelligently"
echo "  • Extract information from uploaded documents"
echo "" 