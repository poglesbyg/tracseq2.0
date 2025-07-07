#!/bin/bash

echo "Stopping TracSeq 2.0 Development Services..."

# Kill by PID files
if [ -f .api_gateway.pid ]; then
    kill $(cat .api_gateway.pid) 2>/dev/null || true
    rm .api_gateway.pid
fi

if [ -f .frontend.pid ]; then
    kill $(cat .frontend.pid) 2>/dev/null || true
    rm .frontend.pid
fi

# Kill by process name as backup
pkill -f "api_gateway.py" 2>/dev/null || true
pkill -f "vite" 2>/dev/null || true

echo "Services stopped."