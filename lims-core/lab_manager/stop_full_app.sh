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

print_info "🛑 Stopping Complete Lab Manager Application"
echo

# Stop processes by PID files
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

# Stop tracked processes
stop_process "/tmp/lab_manager_backend.pid" "Rust Backend"
stop_process "/tmp/lab_manager_frontend.pid" "React Frontend"
stop_process "/tmp/rag_api.pid" "RAG API"

# Kill any remaining processes
print_info "Cleaning up remaining processes..."
pkill -f "lab_manager" 2>/dev/null || true
pkill -f "uvicorn.*main:app" 2>/dev/null || true
pkill -f "vite.*frontend" 2>/dev/null || true
pkill -f "cargo run" 2>/dev/null || true

# Stop Docker containers
print_info "Stopping Docker containers..."
cd lab_manager 2>/dev/null || true
docker-compose down 2>/dev/null || true
cd .. 2>/dev/null || true

# Clean up log files
print_info "Cleaning up log files..."
rm -f /tmp/lab_manager_backend.log
rm -f /tmp/lab_manager_frontend.log
rm -f /tmp/rag_api.log

print_success "✅ All services stopped successfully!"
print_info "Application is now completely shut down." 
