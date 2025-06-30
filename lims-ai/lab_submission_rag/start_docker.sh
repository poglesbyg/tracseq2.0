#!/bin/bash

# 🐳 Laboratory Submission RAG System - Docker Quick Start
# Ultra-lightweight setup with Ollama

set -e

echo "🧬 Laboratory Submission RAG System"
echo "🐳 Docker Quick Start"
echo "="*50

# Check if Docker is installed
if ! command -v docker &> /dev/null; then
    echo "❌ Docker is not installed. Please install Docker first:"
    echo "   https://docs.docker.com/get-docker/"
    exit 1
fi

# Check if Docker Compose is available
if ! command -v docker-compose &> /dev/null && ! docker --help | grep -q compose; then
    echo "❌ Docker Compose is not available. Please install Docker Compose:"
    echo "   https://docs.docker.com/compose/install/"
    exit 1
fi

# Use docker compose or docker-compose based on availability
COMPOSE_CMD="docker-compose"
if ! command -v docker-compose &> /dev/null; then
    COMPOSE_CMD="docker compose"
fi

echo "✅ Docker is ready"

# Create necessary directories
echo "📁 Creating directories..."
mkdir -p data uploads exports demo

# Set permissions (Linux/Mac)
if [[ "$OSTYPE" != "msys" && "$OSTYPE" != "win32" ]]; then
    chmod 755 data uploads exports demo
fi

# Check available resources
echo "🔍 Checking system resources..."
if command -v free &> /dev/null; then
    MEMORY_GB=$(free -g | awk 'NR==2{printf "%.1f", $7}')
    echo "   Available RAM: ${MEMORY_GB}GB"
    
    if (( $(echo "$MEMORY_GB < 3" | bc -l) )); then
        echo "⚠️  Warning: Less than 3GB RAM available. Consider using llama3.2:1b model"
    fi
elif command -v sysctl &> /dev/null; then
    MEMORY_BYTES=$(sysctl -n hw.memsize 2>/dev/null || echo "0")
    MEMORY_GB=$(echo "scale=1; $MEMORY_BYTES / 1024 / 1024 / 1024" | bc)
    echo "   Available RAM: ${MEMORY_GB}GB"
fi

# Check disk space
if command -v df &> /dev/null; then
    DISK_GB=$(df -BG . | awk 'NR==2 {print $4}' | sed 's/G//')
    echo "   Available Disk: ${DISK_GB}GB"
    
    if (( DISK_GB < 5 )); then
        echo "⚠️  Warning: Less than 5GB disk space available"
    fi
fi

echo ""
echo "🚀 Starting Docker containers..."

# Stop any existing containers
$COMPOSE_CMD -f docker-compose-simple.yml down 2>/dev/null || true

# Start the services
$COMPOSE_CMD -f docker-compose-simple.yml up -d

echo "✅ Containers started!"
echo ""
echo "📥 Downloading Ollama model (first time only)..."
echo "   This may take 2-5 minutes depending on your internet connection..."

# Wait for model download
for i in {1..60}; do
    if docker logs simple-rag-model-downloader 2>/dev/null | grep -q "Model downloaded successfully"; then
        echo "✅ Model download completed!"
        break
    elif docker logs simple-rag-model-downloader 2>/dev/null | grep -q "Error\|error\|failed"; then
        echo "❌ Model download failed. Check logs:"
        docker logs simple-rag-model-downloader
        exit 1
    else
        echo -n "."
        sleep 5
    fi
done

echo ""
echo "🔍 Checking system health..."

# Wait for services to be ready
for i in {1..12}; do
    if curl -s http://localhost:8000/health > /dev/null 2>&1; then
        echo "✅ System is ready!"
        break
    else
        echo -n "."
        sleep 5
    fi
done

echo ""
echo "🎉 Laboratory Submission RAG System is ready!"
echo ""
echo "🌐 Web Interface: http://localhost:8000"
echo "🏥 Health Check:  http://localhost:8000/health"
echo ""
echo "📊 Quick Status Check:"

# Show container status
$COMPOSE_CMD -f docker-compose-simple.yml ps

echo ""
echo "🔧 Useful Commands:"
echo "   View logs:        $COMPOSE_CMD -f docker-compose-simple.yml logs"
echo "   Stop system:      $COMPOSE_CMD -f docker-compose-simple.yml down"
echo "   Restart system:   $COMPOSE_CMD -f docker-compose-simple.yml restart"
echo "   System stats:     docker stats"
echo ""
echo "📚 Documentation: README_DOCKER.md"
echo ""
echo "🚀 Ready to process laboratory submissions!"
echo "   Open http://localhost:8000 in your browser to get started." 
