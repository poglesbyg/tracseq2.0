#!/bin/bash

echo "🚀 Starting TracSeq 2.0 Development Environment"

# Function to check if port is in use
check_port() {
    lsof -i :$1 > /dev/null 2>&1
}

# Function to wait for service to be ready
wait_for_service() {
    local url=$1
    local name=$2
    echo "⏳ Waiting for $name to be ready..."
    while ! curl -s $url > /dev/null; do
        sleep 1
    done
    echo "✅ $name is ready!"
}

# Clean up any existing processes
echo "🧹 Cleaning up existing processes..."
pkill -f api_gateway_simple.py || true
pkill -f "pnpm dev" || true
sleep 2

# Start API Gateway
echo "🔧 Starting API Gateway on port 8089..."
if check_port 8089; then
    echo "⚠️  Port 8089 is busy, attempting to free it..."
    lsof -ti:8089 | xargs kill -9 || true
    sleep 2
fi

cd "$(dirname "$0")"
uv run python api_gateway_simple.py &
API_PID=$!

# Wait for API Gateway to be ready
wait_for_service "http://localhost:8089/health" "API Gateway"

# Start Frontend
echo "🌐 Starting Frontend on port 5173..."
if check_port 5173; then
    echo "⚠️  Port 5173 is busy, attempting to free it..."
    lsof -ti:5173 | xargs kill -9 || true
    sleep 2
fi

cd frontend
pnpm dev &
FRONTEND_PID=$!

# Wait for Frontend to be ready
wait_for_service "http://localhost:5173" "Frontend"

echo ""
echo "🎉 TracSeq 2.0 Development Environment Started Successfully!"
echo ""
echo "📍 API Gateway:  http://localhost:8089"
echo "📊 Health Check: http://localhost:8089/health"
echo "🔗 API Docs:     http://localhost:8089/docs"
echo "🌐 Frontend:     http://localhost:5173"
echo ""
echo "💡 ChatBot Test:"
echo "   Ask: 'How do I submit a new sample using the AI document processing?'"
echo ""
echo "🛑 To stop: Press Ctrl+C or run: pkill -f 'api_gateway_simple.py|pnpm dev'"
echo ""

# Keep script running and show logs
echo "📋 Service Logs (Ctrl+C to stop):"
echo "---"

# Function to cleanup on exit
cleanup() {
    echo ""
    echo "🛑 Stopping TracSeq 2.0 Development Environment..."
    kill $API_PID 2>/dev/null || true
    kill $FRONTEND_PID 2>/dev/null || true
    pkill -f api_gateway_simple.py || true
    pkill -f "pnpm dev" || true
    echo "✅ All services stopped"
    exit 0
}

# Set up trap for cleanup
trap cleanup SIGINT SIGTERM

# Keep script running
wait 