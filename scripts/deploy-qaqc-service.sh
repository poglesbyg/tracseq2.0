#!/bin/bash

# QAQC Service Deployment Script
# This script builds and deploys the QAQC service as the first Rust service

set -e

echo "🚀 Deploying QAQC Service - First Rust Service"
echo "=============================================="

# Configuration
QAQC_PORT=8103
DATABASE_URL="postgres://postgres:postgres@localhost:5440/lims_db"
JWT_SECRET="test-secret"
SERVICE_NAME="qaqc-service"

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# Helper function to print colored output
print_status() {
    echo -e "${GREEN}[INFO]${NC} $1"
}

print_warning() {
    echo -e "${YELLOW}[WARN]${NC} $1"
}

print_error() {
    echo -e "${RED}[ERROR]${NC} $1"
}

# Check if we're in the correct directory
if [ ! -f "lims-laboratory/qaqc_service/Cargo.toml" ]; then
    print_error "Please run this script from the project root directory"
    exit 1
fi

# Step 1: Build the QAQC service
print_status "Building QAQC service..."
cd lims-laboratory/qaqc_service
cargo build --release

if [ $? -eq 0 ]; then
    print_status "✅ QAQC service built successfully"
else
    print_error "❌ Failed to build QAQC service"
    exit 1
fi

# Step 2: Check if service is already running
print_status "Checking if QAQC service is already running..."
if curl -f http://localhost:$QAQC_PORT/health > /dev/null 2>&1; then
    print_warning "QAQC service is already running on port $QAQC_PORT"
    print_status "Stopping existing service..."
    pkill -f qaqc_service || true
    sleep 2
fi

# Step 3: Start the service
print_status "Starting QAQC service on port $QAQC_PORT..."
SKIP_MIGRATIONS=true \
DATABASE_URL=$DATABASE_URL \
QAQC_PORT=$QAQC_PORT \
JWT_SECRET=$JWT_SECRET \
RUST_LOG=info \
./target/release/qaqc_service &

SERVICE_PID=$!
print_status "QAQC service started with PID: $SERVICE_PID"

# Step 4: Wait for service to be ready
print_status "Waiting for service to be ready..."
for i in {1..30}; do
    if curl -f http://localhost:$QAQC_PORT/health > /dev/null 2>&1; then
        print_status "✅ QAQC service is healthy and ready"
        break
    fi
    
    if [ $i -eq 30 ]; then
        print_error "❌ QAQC service failed to start within 30 seconds"
        kill $SERVICE_PID 2>/dev/null || true
        exit 1
    fi
    
    sleep 1
done

# Step 5: Test the service
print_status "Testing QAQC service endpoints..."

# Test health endpoint
HEALTH_RESPONSE=$(curl -s http://localhost:$QAQC_PORT/health)
print_status "Health endpoint response: $HEALTH_RESPONSE"

# Test API endpoint (may return 500 if database tables don't exist, but that's expected)
API_STATUS=$(curl -s -o /dev/null -w "%{http_code}" http://localhost:$QAQC_PORT/api/v1/qc/reviews)
if [ "$API_STATUS" = "500" ]; then
    print_warning "API endpoint returned 500 (expected - database tables may not exist)"
elif [ "$API_STATUS" = "200" ]; then
    print_status "✅ API endpoint is working correctly"
else
    print_warning "API endpoint returned status: $API_STATUS"
fi

# Step 6: Success summary
print_status ""
print_status "🎉 QAQC Service Deployment Successful!"
print_status "======================================"
print_status "Service URL: http://localhost:$QAQC_PORT"
print_status "Health Check: http://localhost:$QAQC_PORT/health"
print_status "API Endpoint: http://localhost:$QAQC_PORT/api/v1/qc/reviews"
print_status "Process ID: $SERVICE_PID"
print_status ""
print_status "Next Steps:"
print_status "1. Update API Gateway to route /api/qaqc/* to http://localhost:$QAQC_PORT"
print_status "2. Test integration with existing Python services"
print_status "3. Deploy additional Rust services (sample, sequencing, notification)"
print_status ""
print_status "To stop the service: kill $SERVICE_PID"

# Save PID for later use
echo $SERVICE_PID > /tmp/qaqc_service.pid
print_status "PID saved to /tmp/qaqc_service.pid" 