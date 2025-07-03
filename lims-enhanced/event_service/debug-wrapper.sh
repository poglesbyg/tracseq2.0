#!/bin/sh
echo "=== DEBUG WRAPPER START ==="
echo "Current user: $(whoami)"
echo "Current directory: $(pwd)"
echo "Environment variables:"
env | grep -E "REDIS|DATABASE|HOST|PORT|RUST" | sort
echo "Binary info:"
ls -la /app/event_service
echo "Attempting to run event_service..."
exec /app/event_service 