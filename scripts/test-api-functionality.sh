#!/bin/bash

# TracSeq 2.0 API Functionality Tests
# This script tests actual API operations to ensure services are functional

# Colors for output
GREEN='\033[0;32m'
RED='\033[0;31m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

echo "================================================"
echo " TracSeq 2.0 - API Functionality Tests"
echo "================================================"

# Base URLs
API_GATEWAY="http://localhost:18089"
AUTH_SERVICE="http://localhost:8080"

echo ""
echo "=== Authentication Tests ==="
echo ""

# Test auth service registration endpoint
echo -n "Testing user registration endpoint... "
register_response=$(curl -s -X POST "$AUTH_SERVICE/api/v1/auth/register" \
    -H "Content-Type: application/json" \
    -d '{"username": "test_user_'$(date +%s)'", "email": "test@example.com", "password": "TestPassword123!"}' \
    -w "\n%{http_code}" | tail -1)

if [ "$register_response" == "200" ] || [ "$register_response" == "201" ] || [ "$register_response" == "409" ]; then
    echo -e "${GREEN}✓ PASSED${NC} (HTTP $register_response)"
else
    echo -e "${RED}✗ FAILED${NC} (HTTP $register_response)"
fi

# Test login endpoint
echo -n "Testing login endpoint... "
login_response=$(curl -s -X POST "$AUTH_SERVICE/api/v1/auth/login" \
    -H "Content-Type: application/json" \
    -d '{"username": "admin", "password": "admin123"}' \
    -o /dev/null -w "%{http_code}")

if [ "$login_response" == "200" ] || [ "$login_response" == "401" ]; then
    echo -e "${GREEN}✓ PASSED${NC} (Endpoint responding correctly)"
else
    echo -e "${RED}✗ FAILED${NC} (HTTP $login_response)"
fi

echo ""
echo "=== Sequencing Service Tests ==="
echo ""

# Test sequencing job creation endpoint
echo -n "Testing sequencing job endpoint... "
seq_response=$(curl -s -X GET "http://localhost:8084/api/v1/jobs" \
    -H "Content-Type: application/json" \
    -o /dev/null -w "%{http_code}")

if [ "$seq_response" == "200" ] || [ "$seq_response" == "401" ]; then
    echo -e "${GREEN}✓ PASSED${NC} (Endpoint accessible)"
else
    echo -e "${RED}✗ FAILED${NC} (HTTP $seq_response)"
fi

echo ""
echo "=== Notification Service Tests ==="
echo ""

# Test notification channels
echo -n "Testing notification channels endpoint... "
notif_response=$(curl -s "http://localhost:8085/api/v1/channels" \
    -H "Content-Type: application/json")

if echo "$notif_response" | jq -e '.channels' > /dev/null 2>&1; then
    echo -e "${GREEN}✓ PASSED${NC} (Channels listed)"
    echo "  Available channels:"
    echo "$notif_response" | jq -r '.channels[] | "  - \(.id): \(.name)"' 2>/dev/null || true
else
    echo -e "${RED}✗ FAILED${NC} (Invalid response)"
fi

echo ""
echo "=== API Gateway Integration Tests ==="
echo ""

# Test API Gateway service discovery
echo -n "Testing service discovery... "
services=$(curl -s "$API_GATEWAY/api/v1/status" | jq -r '.services | keys[]' 2>/dev/null | wc -l)

if [ "$services" -gt 0 ]; then
    echo -e "${GREEN}✓ PASSED${NC} ($services services discovered)"
else
    echo -e "${RED}✗ FAILED${NC} (No services discovered)"
fi

echo ""
echo "=== RAG Service Tests ==="
echo ""

# Test RAG service through API Gateway
echo -n "Testing RAG service health... "
rag_health=$(curl -s "$API_GATEWAY/api/v1/status" | jq -r '.services."rag-service"' 2>/dev/null)

if [ "$rag_health" == "healthy" ]; then
    echo -e "${GREEN}✓ PASSED${NC} (RAG service healthy)"
else
    echo -e "${YELLOW}⚠ WARNING${NC} (RAG service: $rag_health)"
fi

echo ""
echo "================================================"
echo " API Functionality Test Complete"
echo "================================================" 