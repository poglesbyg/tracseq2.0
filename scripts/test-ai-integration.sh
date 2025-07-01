#!/bin/bash

echo "==============================================="
echo "     TracSeq 2.0 AI Integration Test"
echo "==============================================="
echo ""

# Colors for output
GREEN='\033[0;32m'
RED='\033[0;31m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Test results counter
PASSED=0
FAILED=0

# Function to test an endpoint
test_endpoint() {
    local name=$1
    local url=$2
    local method=$3
    local data=$4
    local expected_field=$5
    
    echo -e "${BLUE}Testing:${NC} $name"
    
    if [ "$method" = "GET" ]; then
        response=$(curl -s -w "\n%{http_code}" "$url")
    else
        response=$(curl -s -w "\n%{http_code}" -X "$method" -H "Content-Type: application/json" -d "$data" "$url")
    fi
    
    http_code=$(echo "$response" | tail -n1)
    body=$(echo "$response" | sed '$d')
    
    if [ "$http_code" = "200" ] || [ "$http_code" = "201" ]; then
        if [ ! -z "$expected_field" ] && echo "$body" | jq -e ".$expected_field" > /dev/null 2>&1; then
            echo -e "${GREEN}✓ PASSED${NC} - HTTP $http_code with expected field '$expected_field'"
            ((PASSED++))
        elif [ -z "$expected_field" ]; then
            echo -e "${GREEN}✓ PASSED${NC} - HTTP $http_code"
            ((PASSED++))
        else
            echo -e "${RED}✗ FAILED${NC} - HTTP $http_code but missing expected field '$expected_field'"
            echo "Response: $body" | jq . 2>/dev/null || echo "$body"
            ((FAILED++))
        fi
    else
        echo -e "${RED}✗ FAILED${NC} - HTTP $http_code"
        echo "Response: $body" | jq . 2>/dev/null || echo "$body"
        ((FAILED++))
    fi
    echo ""
}

echo -e "${BLUE}1. Core Infrastructure Tests${NC}"
echo "================================"

# Test Ollama
test_endpoint "Ollama API Version" "http://localhost:11434/api/version" "GET" "" "version"

# Test Ollama model generation
test_endpoint "Ollama Generate" "http://localhost:11434/api/generate" "POST" \
    '{"model": "llama3.2:3b", "prompt": "What is DNA?", "stream": false}' "response"

# Test RAG Service
test_endpoint "RAG Service Health" "http://localhost:8100/api/v1/health" "GET" "" "status"

# Test Cognitive Assistant
test_endpoint "Cognitive Assistant Health" "http://localhost:8015/health" "GET" "" "status"

echo -e "${BLUE}2. AI Query Tests${NC}"
echo "==================="

# Test Cognitive Assistant Query
test_endpoint "Cognitive Assistant Query" "http://localhost:8015/ask" "POST" \
    '{"query": "What are the storage requirements for RNA samples?"}' "response"

# Test Feature Store (if running)
if curl -s -f "http://localhost:8090/health" > /dev/null 2>&1; then
    test_endpoint "Feature Store Health" "http://localhost:8090/health" "GET" "" ""
fi

echo -e "${BLUE}3. Integration Tests${NC}"
echo "====================="

# Test if storage service has AI features enabled
echo -e "${BLUE}Testing:${NC} Storage Service AI Features"
storage_logs=$(docker logs lims-storage 2>&1 | tail -20)
if echo "$storage_logs" | grep -q "AI features disabled"; then
    echo -e "${YELLOW}⚠ WARNING${NC} - Storage AI features are disabled"
else
    echo -e "${GREEN}✓ PASSED${NC} - Storage AI features are enabled"
    ((PASSED++))
fi
echo ""

# Test document processing capability
echo -e "${BLUE}Testing:${NC} Document Processing Readiness"
rag_endpoints=$(curl -s http://localhost:8100/openapi.json | jq '.paths | keys' 2>/dev/null)
if echo "$rag_endpoints" | grep -q "/upload"; then
    echo -e "${GREEN}✓ PASSED${NC} - Document upload endpoint available"
    ((PASSED++))
else
    echo -e "${RED}✗ FAILED${NC} - Document upload endpoint not found"
    ((FAILED++))
fi
echo ""

echo "==============================================="
echo -e "${BLUE}Test Summary:${NC}"
echo -e "  ${GREEN}Passed:${NC} $PASSED"
echo -e "  ${RED}Failed:${NC} $FAILED"
echo ""

if [ $FAILED -eq 0 ]; then
    echo -e "${GREEN}✓ All AI components are fully integrated!${NC}"
    exit 0
else
    echo -e "${YELLOW}⚠ Some AI components need attention${NC}"
    exit 1
fi 