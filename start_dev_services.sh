#!/bin/bash

# Start TracSeq 2.0 Development Services

# Colors for output
GREEN='\033[0;32m'
BLUE='\033[0;34m'
NC='\033[0m'

log_info() {
    echo -e "${BLUE}[INFO]${NC} $1"
}

log_success() {
    echo -e "${GREEN}[SUCCESS]${NC} $1"
}

# Kill any existing services
pkill -f "api_gateway.py" 2>/dev/null || true
pkill -f "vite" 2>/dev/null || true

log_info "Starting TracSeq 2.0 Development Environment..."

# Start API Gateway
log_info "Starting API Gateway on port 8089..."
cd dev-services
python3 api_gateway.py &
API_GATEWAY_PID=$!
cd ..

# Wait for API Gateway to start
sleep 3

# Start Frontend
log_info "Starting Frontend on port 5173..."
cd lims-ui
npm run dev &
FRONTEND_PID=$!
cd ..

log_success "Services started successfully!"
echo ""
echo "🌐 Frontend: http://localhost:5173"
echo "🔧 API Gateway: http://localhost:8089"
echo "📊 Health Check: http://localhost:8089/health"
echo ""
echo "To stop services, run: ./stop_dev_services.sh"

# Save PIDs for cleanup
echo "$API_GATEWAY_PID" > .api_gateway.pid
echo "$FRONTEND_PID" > .frontend.pid

# Wait for user input to keep script running
echo "Press Ctrl+C to stop all services..."
trap 'kill $API_GATEWAY_PID $FRONTEND_PID 2>/dev/null; exit' INT
wait