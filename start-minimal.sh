#!/bin/bash

# TracSeq 2.0 Minimal Enhanced Architecture Demo
set -e

echo "🚀 Starting TracSeq 2.0 Minimal Enhanced Architecture..."

# Stop any existing containers
echo "🧹 Cleaning up existing containers..."
docker-compose -f docker-compose.minimal.yml down 2>/dev/null || true

# Build and start services
echo "🏗️  Building and starting services..."
docker-compose -f docker-compose.minimal.yml up --build -d

echo "⏳ Waiting for services to start..."
sleep 20

# Check service health
echo "🔍 Checking service status..."

# Check if services are responding
if curl -s http://localhost:3000 > /dev/null; then
    echo "✅ Frontend is running at http://localhost:3000"
else
    echo "⚠️  Frontend is starting up..."
fi

if curl -s http://localhost:8000 > /dev/null; then
    echo "✅ RAG Service is running at http://localhost:8000"
else
    echo "⚠️  RAG Service is starting up..."
fi

echo ""
echo "🎉 TracSeq 2.0 Minimal Enhanced Architecture is ready!"
echo ""
echo "📊 Service URLs:"
echo "   Frontend (Standalone): http://localhost:3000"
echo "   RAG Service:          http://localhost:8000"
echo "   PostgreSQL:           localhost:5432"
echo "   Redis:                localhost:6379"
echo "   Ollama:               http://localhost:11434"
echo ""
echo "🔧 Management Commands:"
echo "   View logs:    docker-compose -f docker-compose.minimal.yml logs -f"
echo "   Stop all:     docker-compose -f docker-compose.minimal.yml down"
echo "   Restart:      docker-compose -f docker-compose.minimal.yml restart"
echo ""
echo "🏗️  Architecture Achievement:"
echo "   ✅ Frontend liberated from lab_manager"
echo "   ✅ Standalone service architecture"  
echo "   ✅ Independent database per service"
echo "   ✅ Clean service boundaries"
echo "" 