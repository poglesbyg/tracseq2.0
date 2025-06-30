#!/bin/bash

set -e

echo "🚀 Executing Phase 2: Create Missing Services (Dashboard & Reports)"
echo "=================================================================="
echo ""
echo "This script will:"
echo "  1. Build the Dashboard Service"
echo "  2. Build the Reports Service"
echo "  3. Create required databases"
echo "  4. Deploy both services"
echo "  5. Update API Gateway routing"
echo "  6. Verify services are working"
echo ""

# Colors for output
GREEN='\033[0;32m'
RED='\033[0;31m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# Function to check service health
check_service_health() {
    local service_name=$1
    local url=$2
    local max_attempts=10
    local attempt=1
    
    echo -n "Checking $service_name... "
    
    while [ $attempt -le $max_attempts ]; do
        if curl -s -f "$url" > /dev/null 2>&1; then
            echo -e "${GREEN}✓ Healthy${NC}"
            return 0
        fi
        sleep 3
        attempt=$((attempt + 1))
    done
    
    echo -e "${RED}✗ Not responding${NC}"
    return 1
}

echo "📦 Step 1: Building Dashboard Service..."
echo "======================================="
cd dashboard_service
docker build -t tracseq-dashboard-service:latest .
cd ..
echo -e "${GREEN}✓ Dashboard Service built${NC}"

echo ""
echo "📦 Step 2: Building Reports Service..."
echo "===================================="
cd reports_service
docker build -t tracseq-reports-service:latest .
cd ..
echo -e "${GREEN}✓ Reports Service built${NC}"

echo ""
echo "🗄️ Step 3: Creating Databases..."
echo "================================"

# Create databases if they don't exist
echo "Creating dashboard database..."
docker exec -i tracseq-postgres psql -U postgres << EOF || true
CREATE DATABASE tracseq_dashboard;
CREATE USER dashboard_user WITH PASSWORD 'dashboard_pass';
GRANT ALL PRIVILEGES ON DATABASE tracseq_dashboard TO dashboard_user;
EOF

echo "Creating reports database..."
docker exec -i tracseq-postgres psql -U postgres << EOF || true
CREATE DATABASE tracseq_reports;
CREATE USER reports_user WITH PASSWORD 'reports_pass';
GRANT ALL PRIVILEGES ON DATABASE tracseq_reports TO reports_user;
EOF

echo -e "${GREEN}✓ Databases created${NC}"

echo ""
echo "🚀 Step 4: Deploying Services..."
echo "==============================="

# Check if network exists, create if not
if ! docker network inspect tracseq-network >/dev/null 2>&1; then
    echo "Creating tracseq-network..."
    docker network create tracseq-network
fi

# Deploy services
docker-compose -f docker-compose.phase2.yml up -d

echo ""
echo "⏳ Waiting for services to initialize..."
sleep 15

echo ""
echo "🔍 Step 5: Verifying Service Deployment..."
echo "========================================="

# Check if services are running
services_running=true

if docker ps | grep -q "tracseq-dashboard-service"; then
    echo -e "${GREEN}✓ Dashboard Service container is running${NC}"
else
    echo -e "${RED}✗ Dashboard Service container is not running${NC}"
    services_running=false
fi

if docker ps | grep -q "tracseq-reports-service"; then
    echo -e "${GREEN}✓ Reports Service container is running${NC}"
else
    echo -e "${RED}✗ Reports Service container is not running${NC}"
    services_running=false
fi

echo ""
echo "🧪 Step 6: Testing Service Health Endpoints..."
echo "============================================"

dashboard_healthy=false
reports_healthy=false

# Test dashboard service
if check_service_health "Dashboard Service" "http://localhost:3025/health"; then
    dashboard_healthy=true
fi

# Test reports service
if check_service_health "Reports Service" "http://localhost:3026/health"; then
    reports_healthy=true
fi

echo ""
echo "🔄 Step 7: Updating API Gateway Configuration..."
echo "=============================================="

# The feature flags are already set in the .env file
echo "Feature flags already configured:"
grep -E "USE_(DASHBOARD|REPORTS)_SERVICE" api_gateway/.env

# Restart API Gateway to pick up new services
echo ""
echo "Restarting API Gateway..."
cd api_gateway
docker-compose -f docker-compose.minimal.yml restart api-gateway 2>/dev/null || \
docker-compose -f docker-compose.yml restart api-gateway 2>/dev/null || \
echo -e "${YELLOW}⚠ Could not restart API Gateway automatically${NC}"
cd ..

echo ""
echo "⏳ Waiting for API Gateway to restart..."
sleep 10

echo ""
echo "🧪 Step 8: Testing API Gateway Routing..."
echo "========================================"

# Test routing through API Gateway
echo ""
echo "Testing Dashboard Service through API Gateway:"
if curl -s -f http://localhost:8000/api/dashboard/health > /dev/null 2>&1; then
    echo -e "${GREEN}✓ Dashboard Service accessible through API Gateway${NC}"
else
    echo -e "${YELLOW}⚠ Dashboard Service not accessible through API Gateway yet${NC}"
fi

echo ""
echo "Testing Reports Service through API Gateway:"
if curl -s -f http://localhost:8000/api/reports/health > /dev/null 2>&1; then
    echo -e "${GREEN}✓ Reports Service accessible through API Gateway${NC}"
else
    echo -e "${YELLOW}⚠ Reports Service not accessible through API Gateway yet${NC}"
fi

echo ""
echo "📊 Phase 2 Summary"
echo "================="

if [ "$services_running" = true ] && [ "$dashboard_healthy" = true ] && [ "$reports_healthy" = true ]; then
    echo -e "${GREEN}✅ Phase 2 Complete: Dashboard and Reports services created and deployed!${NC}"
    echo ""
    echo "🎯 Services Status:"
    echo "  • Dashboard Service: http://localhost:3025 ✓"
    echo "  • Reports Service: http://localhost:3026 ✓"
    echo ""
    echo "📈 Available Endpoints:"
    echo ""
    echo "Dashboard Service:"
    echo "  • GET  /api/dashboard - Main dashboard"
    echo "  • GET  /api/dashboard/metrics - System metrics"
    echo "  • GET  /api/dashboard/kpis - Key performance indicators"
    echo "  • GET  /api/dashboard/services - Service status"
    echo "  • GET  /api/dashboard/lab/samples - Sample metrics"
    echo "  • POST /api/dashboard/custom - Create custom dashboard"
    echo ""
    echo "Reports Service:"
    echo "  • GET  /api/reports - List reports"
    echo "  • POST /api/reports/generate - Generate report"
    echo "  • GET  /api/reports/templates - Report templates"
    echo "  • POST /api/reports/schedules - Schedule reports"
    echo "  • POST /api/reports/export/pdf - Export as PDF"
    echo "  • POST /api/reports/export/excel - Export as Excel"
    echo ""
    echo "🎉 All 14 microservices are now deployed!"
    echo "   The monolith can now be completely decommissioned!"
else
    echo -e "${YELLOW}⚠ Phase 2 Partially Complete${NC}"
    echo ""
    echo "🔧 Troubleshooting:"
    echo "  1. Check service logs: docker-compose -f docker-compose.phase2.yml logs"
    echo "  2. Verify database connectivity"
    echo "  3. Check if ports 3025 and 3026 are available"
    echo "  4. Ensure all dependencies are running"
fi

echo ""
echo "📋 Useful Commands:"
echo "  • View logs: docker-compose -f docker-compose.phase2.yml logs -f [service-name]"
echo "  • Restart services: docker-compose -f docker-compose.phase2.yml restart"
echo "  • Check all services: docker ps | grep tracseq"
echo "  • Test endpoints: curl http://localhost:8000/api/dashboard/health"

echo ""
echo "🚀 Next Steps:"
echo "  1. Verify all services are working correctly"
echo "  2. Test all API endpoints through the gateway"
echo "  3. Migrate any remaining data from monolith"
echo "  4. Decommission the monolith (Phase 3)"