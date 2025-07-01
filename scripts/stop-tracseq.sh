#!/bin/bash

# TracSeq 2.0 Laboratory Management System Stop Script
# This script cleanly shuts down all running services

echo "🛑 TRACSEQ 2.0 - STOPPING LABORATORY MANAGEMENT SYSTEM"
echo "======================================================"
echo ""

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

print_status() {
    echo -e "${BLUE}[INFO]${NC} $1"
}

print_success() {
    echo -e "${GREEN}[SUCCESS]${NC} $1"
}

print_warning() {
    echo -e "${YELLOW}[WARNING]${NC} $1"
}

# Function to stop a service
stop_service() {
    local service_name=$1
    local pid_file="logs/${service_name}.pid"
    
    if [ -f "$pid_file" ]; then
        local pid=$(cat "$pid_file")
        if kill -0 "$pid" 2>/dev/null; then
            print_status "Stopping $service_name (PID: $pid)..."
            kill "$pid"
            sleep 2
            
            # Force kill if still running
            if kill -0 "$pid" 2>/dev/null; then
                print_warning "Force killing $service_name..."
                kill -9 "$pid" 2>/dev/null || true
            fi
            
            print_success "$service_name stopped"
        else
            print_warning "$service_name was not running"
        fi
        rm -f "$pid_file"
    else
        # Try to kill by process name
        if pgrep -f "$service_name" > /dev/null; then
            print_status "Stopping $service_name processes..."
            pkill -f "$service_name" || true
            sleep 2
            print_success "$service_name stopped"
        else
            print_warning "$service_name not found running"
        fi
    fi
}

# Stop all services
print_status "Stopping all TracSeq 2.0 services..."

# Stop Rust services
services=(
    "lab_manager"
    "auth_service"
    "sample_service"
    "sequencing_service"
    "enhanced_storage_service"
    "event_service"
    "template_service"
    "library_details_service"
    "qaqc_service"
    "spreadsheet_versioning_service"
)

for service in "${services[@]}"; do
    stop_service "$service"
done

# Stop Python services
print_status "Stopping Python AI services..."
stop_service "rag_service"

# Kill any remaining Python processes related to TracSeq
pkill -f "rag_orchestrator.py" 2>/dev/null || true
pkill -f "fastmcp.*laboratory" 2>/dev/null || true

# Clean up any stray processes
print_status "Cleaning up any remaining processes..."
pkill -f "target/release" 2>/dev/null || true

echo ""
print_success "🎉 All TracSeq 2.0 services have been stopped"
print_status "📊 Database containers are still running (use 'docker stop' to stop them)"
print_status "📜 Log files preserved in ./logs/ directory"
echo "" 