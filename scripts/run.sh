#!/bin/bash

# Colors for output
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
RED='\033[0;31m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Configuration
BACKEND_PORT=3000
FRONTEND_PORT=5173
DB_PORT=5433
BACKEND_HOST="127.0.0.1"

# PID files for process tracking
BACKEND_PID_FILE="/tmp/lab_manager_backend.pid"
FRONTEND_PID_FILE="/tmp/lab_manager_frontend.pid"

# Function to print colored messages
print_info() { echo -e "${BLUE}[INFO]${NC} $1"; }
print_success() { echo -e "${GREEN}[SUCCESS]${NC} $1"; }
print_warning() { echo -e "${YELLOW}[WARNING]${NC} $1"; }
print_error() { echo -e "${RED}[ERROR]${NC} $1"; }

# Function to check if a command exists
command_exists() {
    command -v "$1" >/dev/null 2>&1
}

# Function to check if a port is in use
check_port() {
    local port=$1
    
    # Try multiple methods to detect if port is in use
    # Method 1: lsof (works for most local processes)
    if lsof -Pi :$port -sTCP:LISTEN -t >/dev/null 2>&1; then
        return 0  # Port is in use
    fi
    
    # Method 2: netstat (works better for Docker containers)
    if netstat -tln 2>/dev/null | grep -q ":$port "; then
        return 0  # Port is in use
    fi
    
    # Method 3: Direct connection test (most reliable)
    if timeout 2 bash -c "</dev/tcp/localhost/$port" >/dev/null 2>&1; then
        return 0  # Port is in use
    fi
    
    return 1  # Port is free
}

# Function to stop processes by PID file
stop_process() {
    local pid_file="$1"
    local service_name="$2"
    
    if [ -f "$pid_file" ]; then
        local pid=$(cat "$pid_file")
        if kill -0 "$pid" 2>/dev/null; then
            print_info "Stopping $service_name (PID: $pid)..."
            kill "$pid" 2>/dev/null
            sleep 2
            if kill -0 "$pid" 2>/dev/null; then
                print_warning "Force killing $service_name..."
                kill -9 "$pid" 2>/dev/null
            fi
        fi
        rm -f "$pid_file"
    fi
}

# Function to stop all lab_manager processes
stop_all_processes() {
    print_info "Stopping all Lab Manager processes..."
    
    # Stop tracked processes
    stop_process "$BACKEND_PID_FILE" "Backend"
    stop_process "$FRONTEND_PID_FILE" "Frontend"
    
    # Kill any remaining lab_manager processes
    pkill -f "lab_manager" 2>/dev/null || true
    pkill -f "vite.*frontend" 2>/dev/null || true
    
    # Stop database container
    if command_exists docker-compose; then
        print_info "Stopping database container..."
        docker-compose down 2>/dev/null || true
    fi
    
    # Wait a moment for cleanup
    sleep 2
}

# Function to check service health
check_service_health() {
    local url="$1"
    local service_name="$2"
    local max_attempts=10
    local attempt=1
    
    print_info "Checking $service_name health..."
    
    while [ $attempt -le $max_attempts ]; do
        if curl -s "$url" >/dev/null 2>&1; then
            print_success "$service_name is healthy!"
            return 0
        fi
        
        if [ $attempt -lt $max_attempts ]; then
            print_info "Attempt $attempt/$max_attempts - waiting for $service_name..."
            sleep 3
        fi
        attempt=$((attempt + 1))
    done
    
    print_error "$service_name failed to start or is not responding"
    return 1
}

# Function to create .env file
create_env_file() {
    if [ ! -f .env ]; then
        print_info "Creating .env file..."
        cat > .env << EOL
DATABASE_URL=postgres://postgres:postgres@localhost:5433/lab_manager
STORAGE_PATH=./storage
SERVER_HOST=127.0.0.1
SERVER_PORT=3000
CORS_ENABLED=true
RUST_LOG=info
RAG_API_URL=http://127.0.0.1:8000
EOL
        print_success ".env file created"
    else
        print_info ".env file already exists"
    fi
}

# Function to setup database with Docker
setup_database() {
    print_info "Setting up PostgreSQL database..."
    
    # Create necessary directories
    mkdir -p storage
    
    # Check if Docker is available for database
    if command_exists docker && command_exists docker-compose; then
        # Stop any existing containers
        docker-compose down 2>/dev/null || true
        
        # Start just the database
        print_info "Starting PostgreSQL with Docker..."
        docker-compose up -d db
        
        # Wait for database to be ready
        print_info "Waiting for database to be ready..."
        sleep 8
        
        # Run migrations if sqlx is available
        if command_exists sqlx; then
            print_info "Running database migrations..."
            sqlx migrate run || print_warning "Migration failed - continuing anyway"
        else
            print_warning "SQLx CLI not found - skipping migrations"
        fi
        
        print_success "Database is ready!"
        return 0
    else
        print_error "Docker not available. Please ensure PostgreSQL is running on localhost:5432"
        return 1
    fi
}

# Function to start backend
start_backend() {
    print_info "Starting backend server..."
    
    # Check if Rust/Cargo is available
    if ! command_exists cargo; then
        print_error "Cargo not found. Please install Rust."
        return 1
    fi
    
    # Check if port is available
    if check_port $BACKEND_PORT; then
        print_warning "Port $BACKEND_PORT is already in use"
        return 1
    fi
    
    # Start backend in background
    nohup cargo run > /tmp/lab_manager_backend.log 2>&1 &
    local backend_pid=$!
    echo $backend_pid > "$BACKEND_PID_FILE"
    
    print_info "Backend starting (PID: $backend_pid)..."
    
    # Check health
    if check_service_health "http://$BACKEND_HOST:$BACKEND_PORT/api/dashboard/stats" "Backend"; then
        print_success "Backend is running on http://$BACKEND_HOST:$BACKEND_PORT"
        return 0
    else
        print_error "Backend failed to start. Check logs: tail -f /tmp/lab_manager_backend.log"
        return 1
    fi
}

# Function to start frontend
start_frontend() {
    print_info "Starting frontend development server..."
    
    # Check if Node.js/npm is available
    if ! command_exists npm; then
        print_error "npm not found. Please install Node.js."
        return 1
    fi
    
    # Check if port is available
    if check_port $FRONTEND_PORT; then
        print_warning "Port $FRONTEND_PORT is already in use"
        return 1
    fi
    
    # Navigate to frontend directory and start
    cd frontend || { print_error "Frontend directory not found"; return 1; }
    
    # Install dependencies if needed
    if [ ! -d "node_modules" ]; then
        print_info "Installing frontend dependencies..."
        npm install
    fi
    
    # Start frontend in background
    nohup npm run dev > /tmp/lab_manager_frontend.log 2>&1 &
    local frontend_pid=$!
    echo $frontend_pid > "$FRONTEND_PID_FILE"
    
    cd ..
    print_info "Frontend starting (PID: $frontend_pid)..."
    
    # Check health (wait a bit longer for Vite to start)
    sleep 5
    if check_service_health "http://localhost:$FRONTEND_PORT" "Frontend"; then
        print_success "Frontend is running on http://localhost:$FRONTEND_PORT"
        return 0
    else
        print_error "Frontend failed to start. Check logs: tail -f /tmp/lab_manager_frontend.log"
        return 1
    fi
}

# Function to check database health
check_database_health() {
    # Check if Docker container is running
    if docker ps --format "table {{.Names}}" 2>/dev/null | grep -q "lab_manager_db"; then
        # Check if port is accessible
        if check_port $DB_PORT; then
            return 0  # Database is healthy
        fi
    fi
    return 1  # Database is not healthy
}

# Function to show status
show_status() {
    print_info "Lab Manager Status:"
    echo
    
    # Check database
    if check_database_health; then
        print_success "✓ Database: Running on localhost:$DB_PORT"
    else
        print_error "✗ Database: Not running"
    fi
    
    # Check backend
    if check_port $BACKEND_PORT; then
        print_success "✓ Backend: Running on http://$BACKEND_HOST:$BACKEND_PORT"
    else
        print_error "✗ Backend: Not running"
    fi
    
    # Check frontend
    if check_port $FRONTEND_PORT; then
        print_success "✓ Frontend: Running on http://localhost:$FRONTEND_PORT"
    else
        print_error "✗ Frontend: Not running"
    fi
    
    echo
    if check_database_health && check_port $BACKEND_PORT && check_port $FRONTEND_PORT; then
        print_success "🎉 All services are running! Access the app at: http://localhost:$FRONTEND_PORT"
    fi
}

# Function to show logs
show_logs() {
    print_info "Showing logs (press Ctrl+C to exit)..."
    echo
    
    if [ -f /tmp/lab_manager_backend.log ] && [ -f /tmp/lab_manager_frontend.log ]; then
        tail -f /tmp/lab_manager_backend.log /tmp/lab_manager_frontend.log
    elif [ -f /tmp/lab_manager_backend.log ]; then
        tail -f /tmp/lab_manager_backend.log
    elif [ -f /tmp/lab_manager_frontend.log ]; then
        tail -f /tmp/lab_manager_frontend.log
    else
        print_warning "No log files found"
    fi
}

# Main execution
case "$1" in
    "stop")
        stop_all_processes
        print_success "All processes stopped"
        ;;
    "status")
        show_status
        ;;
    "logs")
        show_logs
        ;;
    "restart")
        stop_all_processes
        sleep 2
        exec "$0" "start"
        ;;
    "start"|"")
        print_info "Starting Lab Manager..."
        echo
        
        # Create .env file
        create_env_file
        
        # Stop any existing processes
        stop_all_processes
        
        # Setup database
        if ! setup_database; then
            print_error "Database setup failed"
            exit 1
        fi
        
        # Start backend
        if ! start_backend; then
            print_error "Backend startup failed"
            exit 1
        fi
        
        # Start frontend
        if ! start_frontend; then
            print_error "Frontend startup failed"
            stop_all_processes
            exit 1
        fi
        
        echo
        print_success "🚀 Lab Manager is now running!"
        echo
        print_info "Access points:"
        echo "  • Frontend: http://localhost:$FRONTEND_PORT"
        echo "  • Backend API: http://$BACKEND_HOST:$BACKEND_PORT"
        echo "  • Database: localhost:$DB_PORT"
        echo
        print_info "Useful commands:"
        echo "  • View status: ./scripts/run.sh status"
        echo "  • View logs: ./scripts/run.sh logs"
        echo "  • Stop all: ./scripts/run.sh stop"
        echo "  • Restart: ./scripts/run.sh restart"
        ;;
    *)
        echo "Usage: $0 {start|stop|restart|status|logs}"
        echo
        echo "Commands:"
        echo "  start    - Start all services (default)"
        echo "  stop     - Stop all services"
        echo "  restart  - Restart all services"
        echo "  status   - Show service status"
        echo "  logs     - Show live logs"
        exit 1
        ;;
esac 
