#!/bin/bash

# TracSeq 2.0 Laboratory Management System Startup Script
# This script launches all core services in the correct order

set -e

echo "🚀 TRACSEQ 2.0 - STARTING LABORATORY MANAGEMENT SYSTEM"
echo "======================================================"
echo ""

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Function to print colored output
print_status() {
    echo -e "${BLUE}[INFO]${NC} $1"
}

print_success() {
    echo -e "${GREEN}[SUCCESS]${NC} $1"
}

print_warning() {
    echo -e "${YELLOW}[WARNING]${NC} $1"
}

print_error() {
    echo -e "${RED}[ERROR]${NC} $1"
}

# Check if we're in the right directory
if [ ! -f "Cargo.toml" ] || [ ! -d "lab_submission_rag" ]; then
    print_error "Please run this script from the TracSeq 2.0 root directory"
    exit 1
fi

print_status "Checking prerequisites..."

# Check if services are built
if [ ! -f "target/release/lab_manager" ]; then
    print_error "Rust services not built. Please run 'cargo build --workspace --release' first"
    exit 1
fi

# Check database containers
if ! docker ps | grep -q "tracseq-test-postgres"; then
    print_warning "PostgreSQL container not running. Starting databases..."
    ./scripts/test-all.sh db-only &
    sleep 5
fi

# Verify database connectivity
print_status "Testing database connectivity..."
if docker exec tracseq-test-postgres pg_isready -U tracseq_admin > /dev/null 2>&1; then
    print_success "Database connectivity verified"
else
    print_error "Database not ready. Please ensure PostgreSQL container is running"
    exit 1
fi

# Create logs directory
mkdir -p logs

# Environment variables
export DATABASE_URL="postgresql://tracseq_admin:tracseq_secure_password@localhost:5433/tracseq_main"
export REDIS_URL="redis://localhost:6380"
export RUST_LOG="info"
export LOG_LEVEL="info"

# TracSeq-specific environment variables
export STORAGE_PATH="./storage"
export HOST="0.0.0.0"
export PORT="3000"
export RAG_SERVICE_URL="http://localhost:8000"

# Create storage directory if it doesn't exist
mkdir -p storage

# Shibboleth configuration (disabled by default)
export SHIBBOLETH_ENABLED="false"
export SHIBBOLETH_HYBRID_MODE="true"
export SHIBBOLETH_AUTO_CREATE_USERS="true"
export SHIBBOLETH_DEFAULT_ROLE="Guest"

print_status "Starting core services..."

# Function to start a service in background
start_service() {
    local service_name=$1
    local binary_path=$2
    local port=$3
    
    print_status "Starting $service_name on port $port..."
    
    # Kill existing process if running
    pkill -f "$binary_path" 2>/dev/null || true
    
    # Start the service
    nohup $binary_path > logs/${service_name}.log 2>&1 &
    local pid=$!
    
    # Wait a moment and check if it's still running
    sleep 2
    if kill -0 $pid 2>/dev/null; then
        print_success "$service_name started (PID: $pid)"
        echo $pid > logs/${service_name}.pid
    else
        print_error "Failed to start $service_name"
        cat logs/${service_name}.log | tail -5
        return 1
    fi
}

# Start services in order
echo ""
print_status "🔧 Phase 1: Core Infrastructure Services"
start_service "auth_service" "./target/release/auth_service" "8001"
sleep 2

start_service "sample_service" "./target/release/sample_service" "8002"
sleep 2

start_service "sequencing_service" "./target/release/sequencing_service" "8003"
sleep 2

echo ""
print_status "🔧 Phase 2: Laboratory Management Services"
start_service "lab_manager" "./target/release/lab_manager" "3000"
sleep 2

start_service "enhanced_storage_service" "./target/release/enhanced_storage_service" "8005"
sleep 2

echo ""
print_status "🔧 Phase 3: Support Services"
start_service "event_service" "./target/release/event_service" "8006"
sleep 2

start_service "template_service" "./target/release/template_service" "8007"
sleep 2

# Start Python AI services
echo ""
print_status "🐍 Phase 4: AI Services"
print_status "Starting lab_submission_rag service..."
cd lab_submission_rag
nohup uv run python rag_orchestrator.py > ../logs/rag_service.log 2>&1 &
rag_pid=$!
echo $rag_pid > ../logs/rag_service.pid
cd ..

if kill -0 $rag_pid 2>/dev/null; then
    print_success "RAG service started (PID: $rag_pid)"
else
    print_warning "RAG service may have failed to start (check logs/rag_service.log)"
fi

sleep 3

# Health check
echo ""
print_status "🔍 Phase 5: Health Check"
print_status "Performing health checks..."

# Check if processes are still running
services=(
    "auth_service:8001"
    "sample_service:8002" 
    "sequencing_service:8003"
    "lab_manager:3000"
    "enhanced_storage_service:8005"
    "event_service:8006"
    "template_service:8007"
)

healthy_services=0
total_services=${#services[@]}

for service_port in "${services[@]}"; do
    service=$(echo $service_port | cut -d: -f1)
    port=$(echo $service_port | cut -d: -f2)
    
    if pgrep -f "$service" > /dev/null; then
        if curl -s -o /dev/null -w "%{http_code}" "http://localhost:$port/health" | grep -q "200\|404"; then
            print_success "$service: ✅ Running (port $port)"
            ((healthy_services++))
        else
            print_warning "$service: ⚠️  Running but health check failed"
        fi
    else
        print_error "$service: ❌ Not running"
    fi
done

echo ""
print_status "📊 SYSTEM STATUS SUMMARY"
echo "=========================="
print_status "Healthy services: $healthy_services/$total_services"
print_status "Database: PostgreSQL (port 5433)"
print_status "Cache: Redis (port 6380)"
print_status "Main API: http://localhost:3000"
print_status "Auth Service: http://localhost:8001" 
print_status "Logs directory: ./logs/"

if [ $healthy_services -eq $total_services ]; then
    echo ""
    print_success "🎉 TracSeq 2.0 Laboratory Management System is fully operational!"
    print_success "🔗 Main application: http://localhost:3000"
    print_success "📚 API documentation: http://localhost:3000/docs"
else
    echo ""
    print_warning "⚠️  Some services may need attention. Check individual service logs in ./logs/"
fi

echo ""
print_status "📜 To view logs:"
print_status "  tail -f logs/lab_manager.log"
print_status "  tail -f logs/auth_service.log"
print_status "  tail -f logs/rag_service.log"
echo ""
print_status "🛑 To stop all services: ./scripts/stop-tracseq.sh"
echo "" 