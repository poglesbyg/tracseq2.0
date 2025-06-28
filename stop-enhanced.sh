#!/bin/bash

# TracSeq 2.0 Enhanced Architecture Stop Script
# This script stops the entire enhanced microservices architecture

set -e

echo "🛑 Stopping TracSeq 2.0 Enhanced Architecture..."

# Stop all services
docker-compose -f docker-compose.enhanced.yml down

echo "✅ All services stopped successfully!"
echo ""
echo "💡 To start again, run: ./start-enhanced.sh"
echo "💡 To remove all data (volumes), run: docker-compose -f docker-compose.enhanced.yml down -v" 