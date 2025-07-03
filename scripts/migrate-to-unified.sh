#!/bin/bash
# Script to migrate from multiple docker-compose files to unified configuration

set -e

echo "==================================="
echo " TracSeq 2.0 Migration Script"
echo " Migrating to Unified Docker Setup"
echo "==================================="

# Change to the docker directory
cd "$(dirname "$0")/../docker"

echo "Step 1: Stopping all current services..."
docker-compose -f docker-compose.yml -f docker-compose.basic.yml -f docker-compose.enhanced-services.yml down || true
docker stop $(docker ps -aq) 2>/dev/null || true

echo "Step 2: Removing old containers..."
docker rm $(docker ps -aq) 2>/dev/null || true

echo "Step 3: Removing old networks..."
docker network rm docker_lims-network 2>/dev/null || true
docker network rm docker_tracseq-network 2>/dev/null || true

echo "Step 4: Starting infrastructure services with unified configuration..."
docker-compose -f docker-compose.unified.yml up -d postgres redis

echo "Waiting for database to be ready..."
sleep 10

echo "Step 5: Applying database migrations..."
docker-compose -f docker-compose.unified.yml exec -T postgres psql -U postgres -d lims_db < ../db/migrations/auth_service/002_fix_rate_limits_table.sql

echo "Step 6: Starting all services..."
docker-compose -f docker-compose.unified.yml up -d

echo "Step 7: Waiting for services to start..."
sleep 20

echo "Step 8: Checking service health..."
echo ""
echo "Service Status:"
echo "==============="

# Check each service
services=("postgres" "redis" "api-gateway" "auth-service" "template-service" "notification-service" "transaction-service")
for service in "${services[@]}"; do
    if docker-compose -f docker-compose.unified.yml ps | grep -q "$service.*Up"; then
        echo "✅ $service"
    else
        echo "❌ $service"
    fi
done

echo ""
echo "Migration complete!"
echo ""
echo "To use the unified setup from now on, use:"
echo "  docker-compose -f docker/docker-compose.unified.yml [command]"
echo ""
echo "Or create an alias:"
echo "  alias dc='docker-compose -f docker/docker-compose.unified.yml'" 