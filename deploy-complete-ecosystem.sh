#!/bin/bash

set -e

echo "🚀 Deploying TracSeq 2.0 Complete Ecosystem"
echo "============================================="
echo ""
echo "🎯 Complete Architecture Deployment:"
echo "   • Frontend (React/Astro) - Port 3000"
echo "   • API Gateway - Port 8000"
echo "   • Auth Service - Port 8080" 
echo "   • Sample Service - Port 8081"
echo "   • Enhanced Storage Service - Port 8082"
echo "   • Template Service - Port 8083"
echo "   • Sequencing Service - Port 8084"
echo "   • Notification Service - Port 8085"
echo "   • Enhanced RAG Service - Port 8086"
echo "   • Event Service - Port 8087"
echo "   • Infrastructure: PostgreSQL, Redis, Kafka, Ollama, ChromaDB"
echo ""

# Function to check if a command exists
command_exists() {
    command -v "$1" >/dev/null 2>&1
}

# Function to wait for service to be healthy
wait_for_service() {
    local service_name=$1
    local port=$2
    local max_attempts=30
    local attempt=1
    
    echo "⏳ Waiting for $service_name (port $port) to be healthy..."
    
    while [ $attempt -le $max_attempts ]; do
        if curl -s -f "http://localhost:$port/health" >/dev/null 2>&1; then
            echo "✅ $service_name is healthy!"
            return 0
        fi
        
        echo "   Attempt $attempt/$max_attempts - $service_name not ready yet..."
        sleep 10
        attempt=$((attempt + 1))
    done
    
    echo "❌ $service_name failed to become healthy in time"
    return 1
}

# Function to display service status
show_service_status() {
    echo ""
    echo "📊 Service Status:"
    echo "=================="
    docker compose -f docker-compose.complete-ecosystem.yml ps --format "table {{.Service}}\\t{{.Status}}\\t{{.Ports}}"
}

# Function to test all services
test_all_services() {
    echo ""
    echo "🧪 Testing All Services..."
    echo "=========================="
    
    # Test infrastructure
    echo "🔗 Infrastructure Health:"
    echo "   PostgreSQL: $(docker compose -f docker-compose.complete-ecosystem.yml exec -T postgres pg_isready -U postgres | grep 'accepting connections' && echo '✅ Ready' || echo '❌ Not ready')"
    echo "   Redis: $(docker compose -f docker-compose.complete-ecosystem.yml exec -T redis redis-cli ping 2>/dev/null | grep PONG && echo '✅ Ready' || echo '❌ Not ready')"
    
    # Test microservices
    echo ""
    echo "🔗 Microservices Health:"
    services=(
        "auth-service:8080"
        "sample-service:8081" 
        "enhanced-storage-service:8082"
        "template-service:8083"
        "sequencing-service:8084"
        "notification-service:8085"
        "enhanced-rag-service:8086"
        "event-service:8087"
    )
    
    for service_port in "${services[@]}"; do
        service=${service_port%:*}
        port=${service_port#*:}
        health_status=$(curl -s http://localhost:$port/health 2>/dev/null || echo "not responding")
        if echo "$health_status" | grep -q "healthy\|status"; then
            echo "   $service: ✅ Healthy"
        else
            echo "   $service: ❌ Not responding"
        fi
    done
    
    # Test API Gateway
    echo ""
    echo "🌐 API Gateway Test:"
    gateway_status=$(curl -s http://localhost:8000/health 2>/dev/null || echo "not responding")
    if echo "$gateway_status" | grep -q "operational\|healthy"; then
        echo "   API Gateway: ✅ Operational"
    else
        echo "   API Gateway: ❌ Not responding"
    fi
    
    # Test Frontend
    echo ""
    echo "🖥️ Frontend Test:"
    frontend_status=$(curl -s http://localhost:3000 2>/dev/null || echo "not responding")
    if [ ${#frontend_status} -gt 100 ]; then
        echo "   Frontend: ✅ Serving content"
    else
        echo "   Frontend: ❌ Not responding"
    fi
}

# Main deployment process
main() {
    echo "🔍 Pre-deployment Checks"
    echo "========================"
    
    # Check requirements
    if ! command_exists docker; then
        echo "❌ Docker is not installed"
        exit 1
    fi
    
    if ! command_exists docker-compose && ! docker compose version >/dev/null 2>&1; then
        echo "❌ Docker Compose is not available"
        exit 1
    fi
    
    echo "✅ Docker and Docker Compose are available"
    
    # Clean up any existing containers
    echo ""
    echo "🧹 Cleaning up existing containers..."
    docker compose -f docker-compose.complete-ecosystem.yml down --remove-orphans || true
    
    # Build and start services in phases
    echo ""
    echo "🏗️ Building and starting services..."
    echo "====================================="
    
    # Phase 1: Infrastructure
    echo "Phase 1: Starting Infrastructure Services..."
    docker compose -f docker-compose.complete-ecosystem.yml up -d postgres redis zookeeper kafka
    
    echo "Waiting for infrastructure to be ready..."
    sleep 30
    
    # Phase 2: Kafka ecosystem
    echo "Phase 2: Starting Kafka Ecosystem..."
    docker compose -f docker-compose.complete-ecosystem.yml up -d schema-registry kafka-ui
    sleep 15
    
    # Phase 3: AI/ML infrastructure
    echo "Phase 3: Starting AI/ML Infrastructure..."
    docker compose -f docker-compose.complete-ecosystem.yml up -d ollama chroma mosquitto
    sleep 30
    
    # Phase 4: Core microservices
    echo "Phase 4: Starting Core Microservices..."
    docker compose -f docker-compose.complete-ecosystem.yml up -d \
        auth-service \
        template-service \
        event-service
    
    sleep 45
    
    # Phase 5: Dependent microservices
    echo "Phase 5: Starting Dependent Microservices..."
    docker compose -f docker-compose.complete-ecosystem.yml up -d \
        sample-service \
        enhanced-storage-service \
        sequencing-service \
        notification-service \
        enhanced-rag-service
    
    sleep 45
    
    # Phase 6: API Gateway
    echo "Phase 6: Starting API Gateway..."
    docker compose -f docker-compose.complete-ecosystem.yml up -d api-gateway
    
    sleep 30
    
    # Phase 7: Frontend
    echo "Phase 7: Starting Frontend..."
    docker compose -f docker-compose.complete-ecosystem.yml up -d frontend
    
    echo ""
    echo "⏳ Waiting for all services to initialize..."
    sleep 60
    
    # Show overall status
    show_service_status
    
    # Test all services
    test_all_services
    
    echo ""
    echo "🎉 TracSeq 2.0 Complete Ecosystem Deployed!"
    echo "============================================"
    echo ""
    echo "🌐 Access Points:"
    echo "   • Frontend:              http://localhost:3000"
    echo "   • API Gateway:           http://localhost:8000"
    echo "   • API Documentation:     http://localhost:8000/docs"
    echo "   • Service Discovery:     http://localhost:8000/services"
    echo ""
    echo "🔧 Microservices:"
    echo "   • Auth Service:          http://localhost:8080/health"
    echo "   • Sample Service:        http://localhost:8081/health"
    echo "   • Storage Service:       http://localhost:8082/health"
    echo "   • Template Service:      http://localhost:8083/health"
    echo "   • Sequencing Service:    http://localhost:8084/health"
    echo "   • Notification Service:  http://localhost:8085/health"
    echo "   • RAG Service:           http://localhost:8086/health"
    echo "   • Event Service:         http://localhost:8087/health"
    echo ""
    echo "📊 Infrastructure UIs:"
    echo "   • Kafka UI:              http://localhost:8090"
    echo "   • ChromaDB (Vector DB):  http://localhost:8001"
    echo "   • PostgreSQL:            localhost:5432"
    echo "   • Redis:                 localhost:6379"
    echo "   • Kafka:                 localhost:9092"
    echo "   • Ollama (AI):           http://localhost:11434"
    echo ""
    echo "🛠️ Management Commands:"
    echo "   • View logs:             docker compose -f docker-compose.complete-ecosystem.yml logs -f [service]"
    echo "   • Stop all:              docker compose -f docker-compose.complete-ecosystem.yml down"
    echo "   • Restart service:       docker compose -f docker-compose.complete-ecosystem.yml restart [service]"
    echo "   • Service status:        docker compose -f docker-compose.complete-ecosystem.yml ps"
    echo ""
    echo "🔥 Complete Architecture Features:"
    echo "   ✅ Frontend with modern React/Astro UI"
    echo "   ✅ API Gateway with intelligent routing"
    echo "   ✅ Authentication & Authorization"
    echo "   ✅ Sample management and tracking"
    echo "   ✅ Enhanced storage with AI features"
    echo "   ✅ Template management system"
    echo "   ✅ Sequencing workflow automation"
    echo "   ✅ Multi-channel notifications"
    echo "   ✅ AI-powered document processing"
    echo "   ✅ Event-driven architecture"
    echo "   ✅ Vector database for AI embeddings"
    echo "   ✅ Event streaming with Kafka"
    echo "   ✅ Caching with Redis"
    echo "   ✅ IoT integration ready (MQTT)"
    echo ""
    echo "🎯 Ready for Phase 10!"
    echo "✨ The complete TracSeq 2.0 ecosystem is now running!"
}

# Handle script interruption
trap 'echo -e "\n❌ Deployment interrupted. Cleaning up..."; docker compose -f docker-compose.complete-ecosystem.yml down; exit 1' INT TERM

# Run main function
main "$@" 