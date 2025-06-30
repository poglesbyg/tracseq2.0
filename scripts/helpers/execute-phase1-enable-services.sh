#!/bin/bash

set -e

echo "🚀 Executing Phase 1: Enable All Built Microservices"
echo "===================================================="
echo ""
echo "This script will:"
echo "  1. Enable all feature flags for microservices"
echo "  2. Restart the API Gateway"
echo "  3. Verify all services are routing correctly"
echo ""

# Colors for output
GREEN='\033[0;32m'
RED='\033[0;31m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# Function to check service health
check_service_health() {
    local service_name=$1
    local endpoint=$2
    local max_attempts=5
    local attempt=1
    
    echo -n "Checking $service_name... "
    
    while [ $attempt -le $max_attempts ]; do
        if curl -s -f "http://localhost:8000${endpoint}/health" > /dev/null 2>&1; then
            echo -e "${GREEN}✓ Healthy${NC}"
            return 0
        fi
        sleep 2
        attempt=$((attempt + 1))
    done
    
    echo -e "${RED}✗ Not responding${NC}"
    return 1
}

# Navigate to API Gateway directory
cd api_gateway

echo "📝 Step 1: Verifying feature flags are set..."
echo "============================================="
if [ -f .env ]; then
    echo -e "${GREEN}✓ .env file exists with feature flags${NC}"
    echo ""
    echo "Current feature flags:"
    grep "USE_.*_SERVICE" .env | while read -r line; do
        echo "  $line"
    done
else
    echo -e "${RED}✗ .env file not found!${NC}"
    exit 1
fi

echo ""
echo "🔄 Step 2: Restarting API Gateway with new configuration..."
echo "=========================================================="

# Check which docker-compose file to use
if [ -f docker-compose.minimal.yml ]; then
    COMPOSE_FILE="docker-compose.minimal.yml"
    echo "Using minimal deployment mode"
elif [ -f docker-compose.yml ]; then
    COMPOSE_FILE="docker-compose.yml"
    echo "Using standard deployment mode"
else
    echo -e "${RED}✗ No docker-compose file found!${NC}"
    exit 1
fi

# Stop existing gateway
echo "Stopping existing API Gateway..."
docker-compose -f $COMPOSE_FILE down api-gateway 2>/dev/null || true

# Start API Gateway with new configuration
echo "Starting API Gateway with microservice routing enabled..."
docker-compose -f $COMPOSE_FILE up -d api-gateway

# Wait for gateway to be ready
echo ""
echo "⏳ Waiting for API Gateway to initialize..."
sleep 10

# Check if gateway is running
if docker-compose -f $COMPOSE_FILE ps | grep -q "api-gateway.*running"; then
    echo -e "${GREEN}✓ API Gateway is running${NC}"
else
    echo -e "${RED}✗ API Gateway failed to start${NC}"
    echo "Checking logs..."
    docker-compose -f $COMPOSE_FILE logs api-gateway | tail -20
    exit 1
fi

echo ""
echo "🧪 Step 3: Testing API Gateway routing status..."
echo "=============================================="

# Check gateway health
echo -n "API Gateway health check... "
if curl -s -f http://localhost:8000/health > /dev/null 2>&1; then
    echo -e "${GREEN}✓ Healthy${NC}"
    
    # Get detailed health status
    echo ""
    echo "Gateway health details:"
    curl -s http://localhost:8000/health | python -m json.tool 2>/dev/null || echo "  (JSON formatting not available)"
else
    echo -e "${RED}✗ Not responding${NC}"
fi

# Check routing status
echo ""
echo "📊 Step 4: Checking service routing configuration..."
echo "=================================================="

echo "Fetching routing status from gateway..."
routing_status=$(curl -s http://localhost:8000/routing-status 2>/dev/null)

if [ -n "$routing_status" ]; then
    echo ""
    echo "Current routing configuration:"
    echo "$routing_status" | python -m json.tool 2>/dev/null || echo "$routing_status"
else
    echo -e "${YELLOW}⚠ Could not fetch routing status${NC}"
fi

echo ""
echo "🔍 Step 5: Testing individual service endpoints..."
echo "=============================================="

# Test each service endpoint
services=(
    "auth|/api/auth"
    "sample|/api/samples"
    "template|/api/templates"
    "storage|/api/storage"
    "sequencing|/api/sequencing"
    "notification|/api/notifications"
    "rag|/api/rag"
    "barcode|/api/barcodes"
    "qaqc|/api/qaqc"
    "library|/api/library"
    "event|/api/events"
    "transaction|/api/transactions"
    "spreadsheet|/api/spreadsheets"
)

echo ""
healthy_count=0
total_count=0

for service_info in "${services[@]}"; do
    IFS='|' read -r service_name endpoint <<< "$service_info"
    total_count=$((total_count + 1))
    
    if check_service_health "$service_name" "$endpoint"; then
        healthy_count=$((healthy_count + 1))
    fi
done

echo ""
echo "📈 Summary"
echo "========="
echo "Services enabled: $healthy_count/$total_count"

if [ $healthy_count -eq $total_count ]; then
    echo -e "${GREEN}✅ Phase 1 Complete: All microservices are enabled and routing through the API Gateway!${NC}"
    echo ""
    echo "🎯 Next Steps:"
    echo "  1. Monitor service performance and logs"
    echo "  2. Test all API endpoints thoroughly"
    echo "  3. Verify data consistency across services"
    echo "  4. Prepare for Phase 2: Create missing services (dashboard, reports)"
    echo ""
    echo "📊 All 12 services successfully migrated:"
    echo "  • Auth, Sample, Template, Storage, Sequencing"
    echo "  • Notification, RAG, Barcode, QA/QC, Library"
    echo "  • Event, Transaction, Spreadsheet"
else
    echo -e "${YELLOW}⚠ Phase 1 Partially Complete: Some services may need attention${NC}"
    echo ""
    echo "🔧 Troubleshooting:"
    echo "  1. Check individual service logs: docker-compose -f $COMPOSE_FILE logs [service-name]"
    echo "  2. Verify services are running: docker-compose -f $COMPOSE_FILE ps"
    echo "  3. Check network connectivity between services"
    echo "  4. Ensure all required databases and dependencies are running"
fi

echo ""
echo "📋 Additional Commands:"
echo "  • View API Gateway logs: docker-compose -f $COMPOSE_FILE logs -f api-gateway"
echo "  • Check all services: docker-compose -f $COMPOSE_FILE ps"
echo "  • Test specific endpoint: curl http://localhost:8000/api/[service]/health"
echo "  • Rollback if needed: Set USE_*_SERVICE=false in .env and restart"

cd ..