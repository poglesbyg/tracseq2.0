#!/bin/bash

# TracSeq 2.0 Development Start Script (using uv)
echo "🚀 Starting TracSeq 2.0 Development Environment..."

# Function to cleanup on exit
cleanup() {
    echo "🛑 Stopping services..."
    kill $API_PID $FRONTEND_PID 2>/dev/null
    exit 0
}

trap cleanup INT TERM

# Start API Gateway using uv
echo "🔧 Starting API Gateway on port 8089..."
uv run python api_gateway_simple.py &
API_PID=$!

# Wait for API Gateway to start
sleep 3

# Test API Gateway
if curl -s http://localhost:8089/health > /dev/null; then
    echo "✅ API Gateway started successfully"
else
    echo "❌ API Gateway failed to start"
    exit 1
fi

# Start Frontend
echo "🎨 Starting Frontend on port 5173..."
cd frontend && pnpm dev &
FRONTEND_PID=$!

# Wait for Frontend to start
sleep 5

echo ""
echo "🎉 Development Environment Ready!"
echo "📍 Frontend: http://localhost:5173"
echo "📍 API Gateway: http://localhost:8089"
echo "📊 API Docs: http://localhost:8089/docs"
echo "🔍 Health Check: http://localhost:8089/health"
echo ""
echo "Press Ctrl+C to stop all services"

# Wait for processes
wait 