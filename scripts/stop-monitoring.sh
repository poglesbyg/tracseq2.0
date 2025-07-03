#!/bin/bash

# Stop Monitoring Stack for Lab Manager
# This script stops Prometheus, Grafana, and related monitoring services

set -e

echo "🛑 Stopping Lab Manager Monitoring Stack..."

# Check if docker-compose is installed
if ! command -v docker-compose &> /dev/null && ! command -v docker compose &> /dev/null; then
    echo "❌ Error: docker-compose is not installed"
    exit 1
fi

# Determine docker-compose command
if command -v docker compose &> /dev/null; then
    DOCKER_COMPOSE="docker compose"
else
    DOCKER_COMPOSE="docker-compose"
fi

# Stop the monitoring stack
echo "🔧 Stopping monitoring services..."
$DOCKER_COMPOSE -f docker-compose.monitoring.yml down

# Optional: Remove volumes (uncomment if you want to clean up data)
# echo "🗑️  Removing monitoring data volumes..."
# $DOCKER_COMPOSE -f docker-compose.monitoring.yml down -v

echo ""
echo "✅ Monitoring stack stopped successfully!"
echo ""
echo "💡 To restart monitoring, run: ./scripts/start-monitoring.sh"