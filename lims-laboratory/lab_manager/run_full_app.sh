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

# Show help if requested
if [[ "$1" == "--help" ]] || [[ "$1" == "-h" ]]; then
    echo
    echo "Usage: $0 [OPTIONS]"
    echo
    echo "Options:"
    echo "  --reset-db    Reset the database and remove all data"
    echo "  --help, -h    Show this help message"
    echo
    echo "Examples:"
    echo "  $0              # Normal startup"
    echo "  $0 --reset-db   # Reset database and start fresh"
    echo
    exit 0
fi

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

# Check for reset flag
if [[ "$1" == "--reset-db" ]]; then
    print_warning "Resetting database and removing all data..."
    docker-compose down -v 2>/dev/null || true
    print_info "Database volumes removed. Starting fresh..."
else
    # Stop any existing containers
    print_info "Stopping existing containers..."
    docker-compose down 2>/dev/null || true
fi

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

# Check if database is accessible and ready
DB_READY=false
for i in {1..15}; do
    if docker-compose exec -T db pg_isready -U postgres >/dev/null 2>&1; then
        # Also verify we can actually connect and query
        if docker-compose exec -T db psql -U postgres -d lab_manager -c "SELECT 1;" >/dev/null 2>&1; then
            DB_READY=true
            break
        fi
    fi
    sleep 2
done

if [ "$DB_READY" = true ]; then
    print_success "Database is ready!"
else
    print_error "Database failed to start properly"
    print_info "Checking database logs..."
    docker-compose logs db | tail -10
    exit 1
fi

# Check for migration issues and clean up if necessary
print_info "Checking database migration state..."
if docker-compose exec -T db psql -U postgres -d lab_manager -c "SELECT version FROM _sqlx_migrations ORDER BY version;" >/dev/null 2>&1; then
    # Migration table exists, check for consistency
    MIGRATION_COUNT=$(docker-compose exec -T db psql -U postgres -d lab_manager -t -c "SELECT COUNT(*) FROM _sqlx_migrations;" 2>/dev/null | tr -d ' \n\r' || echo "0")
    if [ "$MIGRATION_COUNT" != "6" ]; then
        print_warning "Migration table exists but appears incomplete (found $MIGRATION_COUNT migrations, expected 6)"
        print_warning "Cleaning up migration table to allow fresh migration..."
        docker-compose exec -T db psql -U postgres -d lab_manager -c "DROP TABLE IF EXISTS _sqlx_migrations CASCADE;" >/dev/null 2>&1 || true
        print_info "Migration table cleaned. Application will recreate it on startup."
    else
        print_info "Database migrations appear consistent. Application will verify on startup..."
    fi
else
    print_info "No existing migration table found. Application will create it on startup..."
fi

# Start backend development server
print_info "Starting Rust backend on port 3000..."
docker-compose up -d dev

# Wait for backend to compile and start (this can take a while)
print_info "Waiting for backend to compile and start (this may take 30-60 seconds)..."
BACKEND_READY=false
for i in {1..40}; do
    if curl -s "http://localhost:3000/health" >/dev/null 2>&1; then
        BACKEND_READY=true
        break
    fi
    # Check if backend crashed due to migration issues
    if docker-compose ps dev | grep -q "Exit"; then
        print_warning "Backend container exited. Checking for migration issues..."
        if docker-compose logs dev 2>/dev/null | grep -q "VersionMissing\|Migration.*Error"; then
            print_error "🔥 Migration error detected during startup!"
            print_info "The database migration system is in an inconsistent state."
            print_info "Stopping services and run with --reset-db to fix:"
            echo
            echo "    docker-compose down"
            echo "    ./run_full_app.sh --reset-db"
            echo
            exit 1
        fi
        docker-compose up -d dev  # Restart if it crashed for other reasons
    fi
    echo -n "."
    sleep 3
done

if [ "$BACKEND_READY" = true ]; then
    print_success "Backend started successfully!"
else
    print_warning "Backend taking longer than expected to start..."
fi

# Start frontend development server
print_info "Starting React frontend on port 5173..."
docker-compose up -d frontend-dev

# Wait for frontend to start
print_info "Waiting for frontend to start..."
sleep 10

# Final service health check
echo
print_info "Final service health check..."

# Check Database
if docker-compose exec -T db pg_isready -U postgres >/dev/null 2>&1; then
    print_success "✓ Database: Running on localhost:5433"
else
    print_error "✗ Database: Not responding"
fi

# Check Backend
if [ "$BACKEND_READY" = true ]; then
    print_success "✓ Backend API: Running on http://localhost:3000"
elif curl -s "http://localhost:3000/health" >/dev/null 2>&1; then
    print_success "✓ Backend API: Running on http://localhost:3000"
else
    print_error "✗ Backend API: Not responding"
    
    # Check for migration errors specifically
    if docker-compose logs dev 2>/dev/null | grep -q "VersionMissing\|Migration.*Error\|migrate"; then
        print_error "🔥 Migration error detected!"
        print_info "The database migration system is in an inconsistent state."
        print_info "Run the following command to reset and try again:"
        echo
        echo "    ./run_full_app.sh --reset-db"
        echo
        print_info "This will completely reset the database and start fresh."
    else
        print_info "Recent backend logs:"
        docker-compose logs dev | tail -10
    fi
fi

# Check Frontend
if curl -s "http://localhost:5173" >/dev/null 2>&1; then
    print_success "✓ Frontend: Running on http://localhost:5173"
elif check_port 5173; then
    print_success "✓ Frontend: Running on http://localhost:5173"
else
    print_error "✗ Frontend: Not responding"
    print_info "Checking frontend logs..."
    docker-compose logs frontend-dev | tail -10
fi

# Determine overall status
OVERALL_STATUS="SUCCESS"
if ! docker-compose exec -T db pg_isready -U postgres >/dev/null 2>&1; then
    OVERALL_STATUS="FAILED"
fi

if [ "$BACKEND_READY" != true ] && ! curl -s "http://localhost:3000/health" >/dev/null 2>&1; then
    OVERALL_STATUS="FAILED"
fi

if ! curl -s "http://localhost:5173" >/dev/null 2>&1 && ! check_port 5173; then
    OVERALL_STATUS="PARTIAL"
fi

echo

if [ "$OVERALL_STATUS" = "SUCCESS" ]; then
    print_success "🎉 Lab Manager Application is now running!"
elif [ "$OVERALL_STATUS" = "PARTIAL" ]; then
    print_warning "⚠️  Lab Manager Application is partially running"
    print_info "Some services may need more time to start or have issues."
else
    print_error "❌ Lab Manager Application failed to start properly"
    print_info "Please check the error messages above and try the suggested fixes."
fi
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
echo "  • Reset database (if migration issues): ./run_full_app.sh --reset-db"
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

if [ "$OVERALL_STATUS" != "SUCCESS" ]; then
    print_info "🔧 Troubleshooting:"
    echo
    if [ "$OVERALL_STATUS" = "FAILED" ]; then
        echo "  Common issues and solutions:"
        echo "  • Migration errors: ./run_full_app.sh --reset-db"
        echo "  • Port conflicts: Check what's using ports 3000, 5173, 5433"
        echo "  • Docker issues: Restart Docker Desktop"
        echo "  • Permission issues: Check Docker permissions"
        echo
    fi
    echo "  View detailed logs:"
    echo "  • All services: docker-compose logs -f"
    echo "  • Backend only: docker-compose logs -f dev"
    echo "  • Database only: docker-compose logs -f db"
    echo "  • Frontend only: docker-compose logs -f frontend-dev"
    echo
fi

# Option to show logs
echo
print_warning "Services are running in Docker containers."
print_info "Use 'docker-compose logs -f' to view live logs"
print_info "Use 'docker-compose down' to stop all services"
echo
