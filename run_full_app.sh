#!/bin/bash

# Colors for output
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
RED='\033[0;31m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

print_info() { echo -e "${BLUE}[INFO]${NC} $1"; }
print_success() { echo -e "${GREEN}[SUCCESS]${NC} $1"; }
print_warning() { echo -e "${YELLOW}[WARNING]${NC} $1"; }
print_error() { echo -e "${RED}[ERROR]${NC} $1"; }

print_info "🚀 Starting Lab Manager Application"
echo

# Check if we're in the right directory
if [ ! -f "Cargo.toml" ] || [ ! -d "frontend" ]; then
    print_error "Please run this script from the lab_manager root directory"
    exit 1
fi

# Check Docker installation
if ! command -v docker &> /dev/null; then
    print_error "Docker is not installed. Please install Docker Desktop first."
    exit 1
fi

if ! docker info &> /dev/null; then
    print_error "Docker is not running. Please start Docker Desktop first."
    exit 1
fi

# Stop any existing containers
print_info "Stopping existing containers..."
docker-compose down 2>/dev/null || true

# Function to wait for service
wait_for_service() {
    local url=$1
    local service_name=$2
    local max_attempts=30
    local attempt=1
    
    print_info "Waiting for $service_name to be ready..."
    while [ $attempt -le $max_attempts ]; do
        if curl -s "$url" > /dev/null 2>&1; then
            print_success "$service_name is ready!"
            return 0
        fi
        echo -n "."
        sleep 2
        attempt=$((attempt + 1))
    done
    print_error "$service_name failed to start within $((max_attempts * 2)) seconds"
    return 1
}

# Check if port is available
check_port() {
    local port=$1
    if command -v lsof &> /dev/null; then
        if lsof -Pi :$port -sTCP:LISTEN -t >/dev/null 2>&1; then
            return 0  # Port is in use
        fi
    elif command -v ss &> /dev/null; then
        if ss -ln | grep -q ":$port "; then
            return 0  # Port is in use
        fi
    elif command -v netstat &> /dev/null; then
        if netstat -ln | grep -q ":$port "; then
            return 0  # Port is in use
        fi
    fi
    return 1  # Port is free
}

# Kill any processes that might be using our ports
print_info "Cleaning up any processes using our ports..."
for port in 3000 5173 8000; do
    if check_port $port; then
        print_warning "Port $port is in use, attempting to free it..."
        if command -v lsof &> /dev/null; then
            lsof -ti:$port | xargs kill -9 2>/dev/null || true
        fi
    fi
done

# Start database first
print_info "Starting PostgreSQL database..."
docker-compose up -d db

# Wait for database to be ready
print_info "Waiting for database to be ready..."
sleep 8

# Check if database is accessible
DB_READY=false
for i in {1..15}; do
    if docker-compose exec -T db pg_isready -U postgres >/dev/null 2>&1; then
        DB_READY=true
        break
    fi
    sleep 2
done

if [ "$DB_READY" = true ]; then
    print_success "Database is ready!"
else
    print_error "Database failed to start properly"
    exit 1
fi

# Run migrations if sqlx is available
if command -v sqlx >/dev/null 2>&1; then
    print_info "Running database migrations..."
    export DATABASE_URL="postgres://postgres:postgres@localhost:5433/lab_manager"
    sqlx migrate run || print_warning "Migration failed - this is normal if migrations have already been run"
fi

# Start backend development server
print_info "Starting Rust backend on port 3000..."
docker-compose up -d dev

# Start frontend development server
print_info "Starting React frontend on port 5173..."
docker-compose up -d frontend-dev

# Wait for services to start
print_info "Waiting for services to start..."
sleep 15

# Check service health
echo
print_info "Checking service health..."

# Check Database
if docker-compose exec -T db pg_isready -U postgres >/dev/null 2>&1; then
    print_success "✓ Database: Running on localhost:5433"
else
    print_error "✗ Database: Failed to start"
fi

# Check Backend
if wait_for_service "http://localhost:3000/health" "Backend API"; then
    print_success "✓ Backend API: Running on http://localhost:3000"
else
    print_error "✗ Backend API: Failed to start"
    print_info "Checking backend logs..."
    docker-compose logs dev | tail -20
fi

# Check Frontend
if check_port 5173; then
    print_success "✓ Frontend: Running on http://localhost:5173"
else
    print_error "✗ Frontend: Failed to start"
    print_info "Checking frontend logs..."
    docker-compose logs frontend-dev | tail -20
fi

echo
print_success "🎉 Lab Manager Application is now running!"
echo
print_info "🌐 Access Points:"
echo "  • Main Application: http://localhost:5173"
echo "  • Backend API: http://localhost:3000"
echo "  • API Documentation: http://localhost:3000/docs (if available)"
echo "  • Database: localhost:5433"
echo
print_info "🔐 Default Admin Credentials (if seeded):"
echo "  • Email: admin@lab.local"
echo "  • Password: admin123"
echo
print_warning "⚠️  Note: RAG service is expected to run separately on port 8000"
echo "   The system will work without it, but AI features will be limited."
echo
print_info "🛠️  Useful Commands:"
echo "  • View all logs: docker-compose logs -f"
echo "  • View backend logs: docker-compose logs -f dev"
echo "  • View frontend logs: docker-compose logs -f frontend-dev"
echo "  • Stop all services: docker-compose down"
echo "  • Restart services: docker-compose restart"
echo
print_info "🧪 Testing the system:"
echo "  • Health check: curl http://localhost:3000/health"
echo "  • API test: curl http://localhost:3000/api/samples"
echo

# Optional: Test authentication if available
print_info "🔍 Testing system health..."
if curl -s "http://localhost:3000/health" | grep -q "ok\|healthy\|success" 2>/dev/null; then
    print_success "✓ System health check passed!"
else
    print_warning "⚠ Health check inconclusive - system may still be starting"
fi

echo
print_info "📚 Next Steps:"
echo "  1. Open http://localhost:5173 in your browser"
echo "  2. Try creating a sample or uploading data"
echo "  3. Check the API at http://localhost:3000/api/"
echo "  4. View logs if you encounter any issues"
echo
print_info "🔧 For development:"
echo "  • Backend source code changes will auto-reload"
echo "  • Frontend changes will auto-reload in the browser"
echo "  • Database data persists between restarts"
echo

# Option to show logs
echo
print_warning "Services are running in Docker containers."
print_info "Use 'docker-compose logs -f' to view live logs"
print_info "Use 'docker-compose down' to stop all services"
echo
