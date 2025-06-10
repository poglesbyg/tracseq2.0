#!/bin/bash

# Test script for the Spreadsheet Processing Service
# This script tests the main functionality of the service

BASE_URL="http://localhost:3000/api/spreadsheets"
TEST_DATA_DIR="$(dirname "$0")/../test_data"

echo "=== Spreadsheet Service Test Script ==="
echo "Base URL: $BASE_URL"
echo ""

# Function to create test CSV file
create_test_csv() {
    mkdir -p "$TEST_DATA_DIR"
    cat > "$TEST_DATA_DIR/test_lab_data.csv" << 'EOF'
Sample_ID,Patient_ID,Sample_Type,Collection_Date,Submitter,Department,Priority,Analysis_Type,Storage_Temp,Volume_mL,Concentration_ng_uL,Quality_Score,Notes
LAB20240001,P12345,Blood,2024-01-15,Dr. Smith,Oncology,High,WGS,-80°C,2.5,250.5,8.2,Rush analysis needed
LAB20240002,P23456,Saliva,2024-01-14,Dr. Johnson,Cardiology,Medium,WES,-80°C,1.8,180.3,7.9,Standard processing
LAB20240003,P34567,Tissue,2024-01-13,Dr. Brown,Neurology,High,Targeted Panel,-80°C,3.2,320.7,8.5,Handle with care
LAB20240004,P45678,Blood,2024-01-12,Dr. Davis,Genetics,Low,RNA-Seq,-80°C,1.5,150.2,7.8,Quality check required
LAB20240005,P56789,Urine,2024-01-11,Dr. Smith,Oncology,Medium,WGS,-20°C,4.0,400.1,8.9,Standard processing
EOF
    echo "Created test CSV file: $TEST_DATA_DIR/test_lab_data.csv"
}

# Function to test API endpoint
test_endpoint() {
    local method=$1
    local endpoint=$2
    local description=$3
    local extra_args=$4
    
    echo "Testing: $description"
    echo "  $method $BASE_URL$endpoint"
    
    if [ "$method" = "GET" ]; then
        response=$(curl -s -w "HTTP_%{http_code}" "$BASE_URL$endpoint" $extra_args)
    elif [ "$method" = "POST" ]; then
        response=$(curl -s -w "HTTP_%{http_code}" -X POST "$BASE_URL$endpoint" $extra_args)
    elif [ "$method" = "DELETE" ]; then
        response=$(curl -s -w "HTTP_%{http_code}" -X DELETE "$BASE_URL$endpoint" $extra_args)
    fi
    
    http_code=$(echo "$response" | grep -o "HTTP_[0-9]*" | cut -d_ -f2)
    body=$(echo "$response" | sed 's/HTTP_[0-9]*$//')
    
    if [ "$http_code" -ge 200 ] && [ "$http_code" -lt 300 ]; then
        echo "  ✅ SUCCESS (HTTP $http_code)"
        if [ ${#body} -lt 200 ]; then
            echo "  Response: $body"
        else
            echo "  Response: $(echo "$body" | head -c 100)..."
        fi
    else
        echo "  ❌ FAILED (HTTP $http_code)"
        echo "  Error: $body"
    fi
    echo ""
}

# Function to extract dataset ID from upload response
extract_dataset_id() {
    local response=$1
    echo "$response" | grep -o '"id":"[^"]*"' | cut -d'"' -f4
}

# Check if server is running
echo "1. Checking if server is running..."
if curl -s "$BASE_URL/health" > /dev/null 2>&1; then
    echo "  ✅ Server is running"
else
    echo "  ❌ Server is not responding. Make sure the lab_manager server is running on localhost:3000"
    echo "  Start the server with: cd lab_manager && cargo run"
    exit 1
fi
echo ""

# Test 1: Health check
test_endpoint "GET" "/health" "Service health check"

# Test 2: Get supported file types
test_endpoint "GET" "/supported-types" "Get supported file types"

# Test 3: List datasets (should be empty initially)
test_endpoint "GET" "/datasets" "List datasets (initial)"

# Test 4: Create test file and upload
echo "4. Creating test data file..."
create_test_csv
echo ""

echo "5. Testing file upload..."
if [ -f "$TEST_DATA_DIR/test_lab_data.csv" ]; then
    upload_response=$(curl -s -w "HTTP_%{http_code}" -X POST "$BASE_URL/upload" \
        -F "file=@$TEST_DATA_DIR/test_lab_data.csv" \
        -F "uploaded_by=test_script@lab.com")
    
    http_code=$(echo "$upload_response" | grep -o "HTTP_[0-9]*" | cut -d_ -f2)
    body=$(echo "$upload_response" | sed 's/HTTP_[0-9]*$//')
    
    if [ "$http_code" -ge 200 ] && [ "$http_code" -lt 300 ]; then
        echo "  ✅ File upload successful (HTTP $http_code)"
        dataset_id=$(extract_dataset_id "$body")
        echo "  Dataset ID: $dataset_id"
        echo "  Response: $(echo "$body" | head -c 150)..."
    else
        echo "  ❌ File upload failed (HTTP $http_code)"
        echo "  Error: $body"
        exit 1
    fi
else
    echo "  ❌ Test file not found: $TEST_DATA_DIR/test_lab_data.csv"
    exit 1
fi
echo ""

# Test 5: List datasets (should now have our upload)
test_endpoint "GET" "/datasets" "List datasets (after upload)"

# Test 6: Get specific dataset
if [ -n "$dataset_id" ]; then
    test_endpoint "GET" "/datasets/$dataset_id" "Get specific dataset"
fi

# Test 7: Search tests
echo "7. Testing search functionality..."
echo ""

# Test 7a: Simple text search
test_endpoint "GET" "/search?search_term=LAB20240001&limit=5" "Search by sample ID"

# Test 7b: Department filter
test_endpoint "GET" "/search?filter_Department=Oncology&limit=5" "Filter by department"

# Test 7c: Multiple filters
test_endpoint "GET" "/search?filter_Department=Oncology&filter_Priority=High&limit=5" "Multiple column filters"

# Test 7d: Search within specific dataset
if [ -n "$dataset_id" ]; then
    test_endpoint "GET" "/search?dataset_id=$dataset_id&search_term=blood&limit=5" "Search within specific dataset"
fi

# Test 7e: Combined search and filter
test_endpoint "GET" "/search?search_term=analysis&filter_Sample_Type=Blood&limit=5" "Combined text and column filter"

# Test 8: Pagination test
test_endpoint "GET" "/search?limit=2&offset=1" "Pagination test"

# Test 9: Invalid requests (should fail gracefully)
echo "8. Testing error handling..."
echo ""

test_endpoint "GET" "/datasets/invalid-uuid" "Invalid dataset ID (should return 404)"
test_endpoint "GET" "/search?filter_NonExistentColumn=value" "Filter by non-existent column"

# Test 10: Cleanup (optional - comment out if you want to keep test data)
echo "9. Cleanup..."
if [ -n "$dataset_id" ]; then
    echo "Deleting test dataset: $dataset_id"
    test_endpoint "DELETE" "/datasets/$dataset_id" "Delete test dataset"
else
    echo "No dataset ID to clean up"
fi

# Clean up test file
if [ -f "$TEST_DATA_DIR/test_lab_data.csv" ]; then
    rm "$TEST_DATA_DIR/test_lab_data.csv"
    echo "Cleaned up test file"
fi

echo "=== Test Summary ==="
echo "All tests completed. Check the results above for any failures."
echo ""
echo "If all tests passed, the Spreadsheet Service is working correctly!"
echo ""
echo "Next steps:"
echo "1. Try uploading your own CSV/Excel files"
echo "2. Experiment with different search queries"
echo "3. Integrate the service into your lab workflow" 
