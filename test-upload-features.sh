#!/bin/bash

# TracSeq 2.0 Upload Features Test Script

set -e

# Colors
GREEN='\033[0;32m'
RED='\033[0;31m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m'

# Configuration
API_URL="http://localhost:8089"

echo -e "${BLUE}TracSeq 2.0 Upload Features Test${NC}"
echo "================================="
echo ""

# Function to test endpoint
test_endpoint() {
    local method=$1
    local url=$2
    local desc=$3
    
    echo -e "${YELLOW}Testing:${NC} $desc"
    
    if [ "$method" = "GET" ]; then
        code=$(curl -s -o /dev/null -w "%{http_code}" "$url")
    else
        code=$(curl -s -o /dev/null -w "%{http_code}" -X "$method" "$url")
    fi
    
    if [[ "$code" =~ ^2[0-9][0-9]$ ]]; then
        echo -e "  ${GREEN}✓ Success${NC} (HTTP $code)"
    else
        echo -e "  ${RED}✗ Failed${NC} (HTTP $code)"
    fi
}

# Check services
echo "1. Checking Services"
echo "-------------------"

echo -n "API Gateway: "
if curl -s -f "$API_URL/health" > /dev/null 2>&1; then
    echo -e "${GREEN}✓ Running${NC}"
else
    echo -e "${RED}✗ Not Running${NC}"
    echo "Please start services first!"
    exit 1
fi

echo ""
echo "2. Testing Endpoints"
echo "-------------------"

test_endpoint "GET" "$API_URL/health" "Health Check"
test_endpoint "GET" "$API_URL/api/dashboard/stats" "Dashboard Stats"
test_endpoint "GET" "$API_URL/api/samples" "List Samples"
test_endpoint "GET" "$API_URL/api/templates" "List Templates"

echo ""
echo "3. Testing Upload Features"
echo "-------------------------"

# Create test file
echo "Creating test CSV file..."
mkdir -p test-uploads
cat > test-uploads/test.csv << EOF
Sample ID,Name,Type
S001,Test Sample 1,DNA
S002,Test Sample 2,RNA
EOF

# Test upload
echo -e "${YELLOW}Testing:${NC} CSV Upload"
response=$(curl -s -w "\n%{http_code}" -X POST -F "file=@test-uploads/test.csv" "$API_URL/api/spreadsheets/upload")
code=$(echo "$response" | tail -n1)

if [[ "$code" =~ ^2[0-9][0-9]$ ]]; then
    echo -e "  ${GREEN}✓ Upload Success${NC} (HTTP $code)"
else
    echo -e "  ${RED}✗ Upload Failed${NC} (HTTP $code)"
fi

echo ""
echo -e "${BLUE}Test Complete!${NC}"

# Cleanup
rm -rf test-uploads