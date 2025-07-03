#!/bin/bash

# Start Monitoring Stack for Lab Manager
# This script starts Prometheus, Grafana, and related monitoring services

set -e

echo "🚀 Starting Lab Manager Monitoring Stack..."

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

# Create necessary directories if they don't exist
echo "📁 Creating monitoring directories..."
mkdir -p monitoring/prometheus/rules
mkdir -p monitoring/grafana/provisioning/datasources
mkdir -p monitoring/grafana/provisioning/dashboards
mkdir -p monitoring/grafana/dashboards
mkdir -p monitoring/alertmanager

# Check if configuration files exist
if [ ! -f "monitoring/prometheus/prometheus.yml" ]; then
    echo "❌ Error: Prometheus configuration not found at monitoring/prometheus/prometheus.yml"
    echo "Please ensure monitoring configuration files are in place."
    exit 1
fi

# Load environment variables if .env.monitoring exists
if [ -f ".env.monitoring" ]; then
    echo "📋 Loading environment variables from .env.monitoring..."
    export $(cat .env.monitoring | grep -v '^#' | xargs)
fi

# Start the monitoring stack
echo "🔧 Starting monitoring services..."
$DOCKER_COMPOSE -f docker-compose.monitoring.yml up -d

# Wait for services to be healthy
echo "⏳ Waiting for services to start..."
sleep 10

# Check service status
echo "📊 Checking service status..."
$DOCKER_COMPOSE -f docker-compose.monitoring.yml ps

# Display access URLs
echo ""
echo "✅ Monitoring stack started successfully!"
echo ""
echo "📊 Access URLs:"
echo "   - Prometheus: http://localhost:9090"
echo "   - Grafana: http://localhost:3002 (admin/admin)"
echo "   - Alertmanager: http://localhost:9093"
echo "   - Node Exporter: http://localhost:9100/metrics"
echo "   - cAdvisor: http://localhost:8080"
echo ""
echo "💡 Tips:"
echo "   - Import the Lab Manager dashboard in Grafana"
echo "   - Configure alert notification channels in Alertmanager"
echo "   - Check Prometheus targets at http://localhost:9090/targets"
echo ""