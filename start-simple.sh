#!/bin/bash

# TracSeq 2.0 Simple Enhanced Architecture Startup
set -e

echo "🚀 Starting TracSeq 2.0 Simple Enhanced Architecture..."

# Check for .env file
if [ ! -f .env ]; then
    echo "📄 Creating default .env file..."
    cat > .env << 'EOF'
# TracSeq 2.0 Enhanced Configuration
COMPOSE_PROJECT_NAME=tracseq-enhanced
DATABASE_URL=postgresql://tracseq_admin:tracseq_secure_password@localhost:5433/tracseq_main
REDIS_URL=redis://:tracseq_redis_password@localhost:6380
API_GATEWAY_URL=http://localhost:8089
FRONTEND_URL=http://localhost:3000
LOG_LEVEL=DEBUG
NODE_ENV=production
REACT_APP_API_BASE_URL=http://localhost:8089
EOF
    echo "✅ Created .env file with default values"
fi

# Load environment variables
source .env

echo "🏗️  Building and starting core services..."
docker-compose -f docker-compose.simple.yml up --build -d

echo "⏳ Waiting for services to be healthy..."
sleep 15

# Check service health
echo "🔍 Checking service health..."
if curl -s http://localhost:3000 > /dev/null; then
    echo "✅ Frontend is running at http://localhost:3000"
else
    echo "❌ Frontend is not responding"
fi

if curl -s http://localhost:8089/health > /dev/null; then
    echo "✅ API Gateway is running at http://localhost:8089"
else
    echo "❌ API Gateway is not responding"
fi

if curl -s http://localhost:8001/health > /dev/null; then
    echo "✅ RAG Service is running at http://localhost:8001"
else
    echo "❌ RAG Service is not responding"
fi

echo ""
echo "🎉 TracSeq 2.0 Simple Enhanced Architecture is ready!"
echo ""
echo "📊 Service URLs:"
echo "   Frontend:     http://localhost:3000"
echo "   API Gateway:  http://localhost:8089"
echo "   RAG Service:  http://localhost:8001"
echo "   PostgreSQL:   localhost:5433"
echo "   Redis:        localhost:6380"
echo "   Ollama:       http://localhost:11435"
echo ""
echo "🔧 Management Commands:"
echo "   View logs:    docker-compose -f docker-compose.simple.yml logs -f"
echo "   Stop all:     docker-compose -f docker-compose.simple.yml down"
echo "   Restart:      docker-compose -f docker-compose.simple.yml restart"
echo "" 