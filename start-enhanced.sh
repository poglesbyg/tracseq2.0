#!/bin/bash

# TracSeq 2.0 Enhanced Architecture Start Script
# This script starts the entire enhanced microservices architecture

set -e

echo "🚀 Starting TracSeq 2.0 Enhanced Architecture..."

# Check if Docker is running
if ! docker info > /dev/null 2>&1; then
    echo "❌ Docker is not running. Please start Docker first."
    exit 1
fi

# Check if Docker Compose is available
if ! command -v docker-compose > /dev/null 2>&1; then
    echo "❌ Docker Compose is not installed. Please install Docker Compose first."
    exit 1
fi

# Create environment file if it doesn't exist
if [ ! -f .env ]; then
    echo "📄 Creating default .env file..."
    cat > .env << 'EOF'
# TracSeq 2.0 Enhanced Architecture Environment Variables

# Frontend Configuration
FRONTEND_PORT=3000
VITE_API_GATEWAY_URL=http://localhost:8089
VITE_API_BASE_URL=http://localhost:8089/api

# API Gateway
API_GATEWAY_PORT=8089

# Microservices Ports
AUTH_SERVICE_PORT=8080
SAMPLE_SERVICE_PORT=8081
STORAGE_SERVICE_PORT=8082
TEMPLATE_SERVICE_PORT=8083
SEQUENCING_SERVICE_PORT=8084
NOTIFICATION_SERVICE_PORT=8085
RAG_SERVICE_PORT=8086
EVENT_SERVICE_PORT=8087
TRANSACTION_SERVICE_PORT=8088

# Infrastructure Ports
POSTGRES_PORT=5433
REDIS_PORT=6379
OLLAMA_PORT=11434

# Security
JWT_SECRET=your-super-secret-jwt-key-change-in-production

# Ollama Configuration
OLLAMA_MODEL=llama3.2:3b
EOF
    echo "✅ Created .env file with default values"
fi

# Start the enhanced architecture
echo "🏗️  Building and starting services..."
docker-compose -f docker-compose.enhanced.yml up -d --build

echo "⏳ Waiting for services to be ready..."
sleep 30

# Check service health
echo "🔍 Checking service health..."

services=(
    "Frontend:http://localhost:3000/health"
    "API Gateway:http://localhost:8089/health" 
    "Auth Service:http://localhost:8080/health"
    "Sample Service:http://localhost:8081/health"
    "Storage Service:http://localhost:8082/health"
    "Template Service:http://localhost:8083/health"
    "Sequencing Service:http://localhost:8084/health"
    "Notification Service:http://localhost:8085/health"
    "RAG Service:http://localhost:8086/health"
    "Event Service:http://localhost:8087/health"
    "Transaction Service:http://localhost:8088/health"
)

for service in "${services[@]}"; do
    name=$(echo $service | cut -d: -f1)
    url=$(echo $service | cut -d: -f2-)
    
    if curl -s -f "$url" > /dev/null 2>&1; then
        echo "✅ $name is healthy"
    else
        echo "⚠️  $name is not responding (this is normal during startup)"
    fi
done

echo ""
echo "🎉 TracSeq 2.0 Enhanced Architecture is starting!"
echo ""
echo "📊 Service URLs:"
echo "   Frontend:          http://localhost:3000"
echo "   API Gateway:       http://localhost:8089"
echo "   Auth Service:      http://localhost:8080"
echo "   Sample Service:    http://localhost:8081"
echo "   Storage Service:   http://localhost:8082"
echo "   Template Service:  http://localhost:8083"
echo "   Sequencing Service: http://localhost:8084"
echo "   Notification Service: http://localhost:8085"
echo "   RAG Service:       http://localhost:8086"
echo "   Event Service:     http://localhost:8087"
echo "   Transaction Service: http://localhost:8088"
echo ""
echo "🗄️  Infrastructure:"
echo "   PostgreSQL:        localhost:5433"
echo "   Redis:             localhost:6379"
echo "   Ollama:            localhost:11434"
echo ""
echo "🛠️  Development Commands:"
echo "   View logs:         docker-compose -f docker-compose.enhanced.yml logs -f"
echo "   Stop services:     docker-compose -f docker-compose.enhanced.yml down"
echo "   Restart service:   docker-compose -f docker-compose.enhanced.yml restart <service-name>"
echo ""
echo "Happy coding! 🚀" 