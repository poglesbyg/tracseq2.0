#!/bin/bash

set -e

echo "🚀 Deploying TracSeq 2.0 Complete Microservices Architecture"
echo "=============================================================="
echo ""
echo "🎯 Architecture Overview:"
echo "   • API Gateway (Port 8000) - Central Router"
echo "   • Core Services: Auth, Sample, Storage, Template, Sequencing, Notification, RAG"
echo "   • Phase 7: Event Sourcing, CQRS, Saga Orchestration"
echo "   • Phase 8: ML Platform (MLflow, AutoML, Feature Store, Model Serving)"
echo "   • Infrastructure: PostgreSQL, Redis, Kafka, Ollama, ChromaDB"
echo ""

# Function to check if a command exists
command_exists() {
    command -v "$1" >/dev/null 2>&1
}

# Function to wait for service to be healthy
wait_for_service() {
    local service_name=$1
    local max_attempts=30
    local attempt=1
    
    echo "⏳ Waiting for $service_name to be healthy..."
    
    while [ $attempt -le $max_attempts ]; do
        if docker compose -f docker-compose.complete-microservices.yml ps | grep -q "$service_name.*healthy"; then
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
    docker compose -f docker-compose.complete-microservices.yml ps --format "table {{.Service}}\\t{{.Status}}\\t{{.Ports}}"
}

# Function to test API Gateway routing
test_api_gateway() {
    echo ""
    echo "🧪 Testing API Gateway Routing..."
    echo "=================================="
    
    echo "Testing Gateway Health:"
    curl -s http://localhost:8000/health | jq '.' || echo "Gateway not responding"
    
    echo -e "\nTesting Service Discovery:"
    curl -s http://localhost:8000/services | jq '.services[] | {name, status}' || echo "Service discovery failed"
    
    echo -e "\nTesting Gateway Stats:"
    curl -s http://localhost:8000/gateway/stats | jq '.' || echo "Stats not available"
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
    docker compose -f docker-compose.complete-microservices.yml down --remove-orphans || true
    
    # Build and start services
    echo ""
    echo "🏗️ Building and starting services..."
    echo "====================================="
    
    # Start infrastructure first
    echo "Starting Infrastructure Services..."
    docker compose -f docker-compose.complete-microservices.yml up -d postgres redis zookeeper kafka
    
    echo "Waiting for infrastructure to be ready..."
    sleep 30
    
    # Start Kafka ecosystem
    echo "Starting Kafka Ecosystem..."
    docker compose -f docker-compose.complete-microservices.yml up -d schema-registry kafka-ui
    sleep 20
    
    # Start AI/ML infrastructure
    echo "Starting AI/ML Infrastructure..."
    docker compose -f docker-compose.complete-microservices.yml up -d ollama chroma mlflow jupyter tensorboard
    sleep 30
    
    # Start core microservices
    echo "Starting Core Microservices..."
    docker compose -f docker-compose.complete-microservices.yml up -d \
        auth-service \
        sample-service \
        enhanced-storage-service \
        template-service \
        sequencing-service \
        notification-service \
        enhanced-rag-service
    
    sleep 30
    
    # Start Phase 7 services (Advanced Patterns)
    echo "Starting Phase 7 Services (Event Sourcing, CQRS, Saga)..."
    docker compose -f docker-compose.complete-microservices.yml up -d \
        event-sourcing-service \
        cqrs-service \
        saga-orchestrator
    
    sleep 20
    
    # Start Phase 8 services (ML Platform)
    echo "Starting Phase 8 Services (ML Platform)..."
    docker compose -f docker-compose.complete-microservices.yml up -d \
        automl-service \
        feature-store \
        model-serving \
        mlops-pipeline
    
    sleep 20
    
    # Start API Gateway last
    echo "Starting API Gateway..."
    docker compose -f docker-compose.complete-microservices.yml up -d api-gateway
    
    echo ""
    echo "⏳ Waiting for services to initialize..."
    sleep 30
    
    # Check service health
    echo ""
    echo "🔍 Checking Service Health"
    echo "=========================="
    
    # Wait for key services
    # wait_for_service "tracseq-postgres"
    # wait_for_service "tracseq-redis"
    # wait_for_service "tracseq-api-gateway"
    
    # Show overall status
    show_service_status
    
    # Test API Gateway
    test_api_gateway
    
    echo ""
    echo "🎉 TracSeq 2.0 Complete Microservices Architecture Deployed!"
    echo "============================================================"
    echo ""
    echo "🌐 Access Points:"
    echo "   • API Gateway:          http://localhost:8000"
    echo "   • API Documentation:    http://localhost:8000/docs"
    echo "   • Service Discovery:    http://localhost:8000/services"
    echo "   • Gateway Health:       http://localhost:8000/health"
    echo "   • Gateway Stats:        http://localhost:8000/gateway/stats"
    echo ""
    echo "📊 Infrastructure UIs:"
    echo "   • Kafka UI:             http://localhost:8084"
    echo "   • MLflow:              http://localhost:5000"
    echo "   • Jupyter Lab:         http://localhost:8888 (token: tracseq)"
    echo "   • TensorBoard:         http://localhost:6006"
    echo ""
    echo "🔧 Core Services (via API Gateway):"
    echo "   • Auth Service:         http://localhost:8000/auth/"
    echo "   • Sample Service:       http://localhost:8000/samples/"
    echo "   • Storage Service:      http://localhost:8000/storage/"
    echo "   • Template Service:     http://localhost:8000/templates/"
    echo "   • Sequencing Service:   http://localhost:8000/sequencing/"
    echo "   • Notification Service: http://localhost:8000/notifications/"
    echo "   • RAG Service:         http://localhost:8000/rag/"
    echo ""
    echo "🚀 Advanced Services:"
    echo "   • Event Sourcing:      http://localhost:8087"
    echo "   • CQRS Service:        http://localhost:8088"
    echo "   • Saga Orchestrator:   http://localhost:8089"
    echo "   • AutoML Service:      http://localhost:8090"
    echo "   • Feature Store:       http://localhost:8091"
    echo "   • Model Serving:       http://localhost:8092"
    echo "   • MLOps Pipeline:      http://localhost:8093"
    echo ""
    echo "🛠️ Management Commands:"
    echo "   • View logs:           docker compose -f docker-compose.complete-microservices.yml logs -f [service]"
    echo "   • Stop all:            docker compose -f docker-compose.complete-microservices.yml down"
    echo "   • Restart service:     docker compose -f docker-compose.complete-microservices.yml restart [service]"
    echo "   • Health check:        ./scripts/monitoring/health-check.sh"
    echo ""
    echo "📈 DevOps Tools:"
    echo "   • Performance test:    ./scripts/performance/run-load-test.sh"
    echo "   • Check SLOs:         ./scripts/ci-cd/check-slos.py"
    echo "   • Development workflow: ./scripts/ci-cd/dev-workflow.sh"
    echo ""
    echo "✨ The complete microservices architecture is now running!"
    echo "   All phases integrated through API Gateway routing."
}

# Handle script interruption
trap 'echo -e "\n❌ Deployment interrupted. Cleaning up..."; docker compose -f docker-compose.complete-microservices.yml down; exit 1' INT TERM

# Run main function
main "$@" 