#!/bin/bash
# Quick Start Script for LIMS Microservice System

set -e

echo "🧪 LIMS Microservice System - Quick Start"
echo "========================================"

# Check Docker is running
if ! docker info > /dev/null 2>&1; then
    echo "❌ Docker is not running. Please start Docker Desktop."
    exit 1
fi

echo "✅ Docker is running"

# Navigate to docker directory
cd docker

# Start core services first (database and cache)
echo ""
echo "🚀 Starting core infrastructure..."
docker-compose up -d postgres redis

# Wait for postgres to be ready
echo "⏳ Waiting for PostgreSQL to be ready..."
sleep 5

# Check if postgres is healthy
until docker-compose exec -T postgres pg_isready -U postgres -d lims_db; do
    echo "   Waiting for database..."
    sleep 2
done

echo "✅ Database is ready"

# Start remaining services
echo ""
echo "🚀 Starting all services..."
docker-compose up -d

# Show service status
echo ""
echo "📊 Service Status:"
docker-compose ps

echo ""
echo "✨ LIMS System is starting up!"
echo ""
echo "🌐 Access Points:"
echo "   - Frontend:      http://localhost:3000"
echo "   - API Gateway:   http://localhost:8089"
echo "   - Auth Service:  http://localhost:8011"
echo "   - Sample Service: http://localhost:8012"
echo "   - Storage Service: http://localhost:8013"
echo ""
echo "📝 Useful Commands:"
echo "   - View logs:        cd docker && docker-compose logs -f"
echo "   - Stop services:    cd docker && docker-compose down"
echo "   - Clean restart:    cd docker && docker-compose down -v && docker-compose up -d"
echo ""
echo "💡 For development, use: ./scripts/dev.sh" 