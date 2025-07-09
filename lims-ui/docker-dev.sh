#!/bin/bash

# TracSeq 2.0 Frontend Docker Development Script
# This script helps you run the frontend in Docker for development

set -e

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

# Check if Docker is running
if ! docker info > /dev/null 2>&1; then
    print_error "Docker is not running. Please start Docker and try again."
    exit 1
fi

# Create network if it doesn't exist
if ! docker network ls | grep -q tracseq-network; then
    print_status "Creating Docker network 'tracseq-network'..."
    docker network create tracseq-network
    print_success "Network created successfully"
else
    print_status "Docker network 'tracseq-network' already exists"
fi

# Function to show help
show_help() {
    echo "TracSeq 2.0 Frontend Docker Development Script"
    echo ""
    echo "Usage: $0 [COMMAND]"
    echo ""
    echo "Commands:"
    echo "  start         Start the frontend development container"
    echo "  stop          Stop the frontend development container"
    echo "  restart       Restart the frontend development container"
    echo "  logs          Show container logs"
    echo "  shell         Open a shell in the container"
    echo "  build         Build the development image"
    echo "  clean         Clean up containers and images"
    echo "  full          Start frontend + API gateway"
    echo "  help          Show this help message"
    echo ""
    echo "Examples:"
    echo "  $0 start                    # Start frontend only"
    echo "  $0 full                     # Start frontend + API gateway"
    echo "  $0 logs                     # View logs"
    echo "  $0 shell                    # Open shell in container"
}

# Function to start frontend only
start_frontend() {
    print_status "Starting TracSeq 2.0 Frontend Development Container..."
    
    # Check if API gateway is running in Docker
    if docker ps | grep -q "lims-gateway"; then
        print_success "API Gateway detected running in Docker (lims-gateway)"
        print_status "Starting frontend container..."
        docker-compose -f docker-compose.dev.yml up -d --build frontend-dev
    elif curl -s http://localhost:8089/health > /dev/null 2>&1; then
        print_success "API Gateway detected running on localhost:8089"
        print_status "Starting frontend container..."
        docker-compose -f docker-compose.dev.yml up -d --build frontend-dev
    else
        print_warning "No API Gateway detected"
        print_warning "Please ensure the LIMS ecosystem is running with: docker-compose up -d"
        print_status "Starting frontend container anyway..."
        docker-compose -f docker-compose.dev.yml up -d --build frontend-dev
    fi
    
    print_success "Frontend container started!"
    print_status "Frontend available at: http://localhost:5173"
    print_status "API proxy working: http://localhost:5173/api/health"
    print_status "Connected to existing LIMS network: docker_lims-network"
    print_status "Use '$0 logs' to view container logs"
}

# Function to start frontend + API gateway
start_full() {
    print_status "Starting TracSeq 2.0 Frontend + API Gateway..."
    docker-compose -f docker-compose.dev.yml --profile api-gateway up -d
    
    print_success "Frontend and API Gateway started!"
    print_status "Frontend available at: http://localhost:5173"
    print_status "API Gateway available at: http://localhost:8000"
    print_status "Use '$0 logs' to view container logs"
}

# Function to stop containers
stop_containers() {
    print_status "Stopping TracSeq 2.0 containers..."
    docker-compose -f docker-compose.dev.yml --profile api-gateway down
    print_success "Containers stopped"
}

# Function to restart containers
restart_containers() {
    print_status "Restarting TracSeq 2.0 containers..."
    stop_containers
    sleep 2
    start_frontend
}

# Function to show logs
show_logs() {
    print_status "Showing container logs (press Ctrl+C to exit)..."
    docker-compose -f docker-compose.dev.yml logs -f
}

# Function to open shell
open_shell() {
    print_status "Opening shell in frontend container..."
    docker-compose -f docker-compose.dev.yml exec frontend-dev sh
}

# Function to build image
build_image() {
    print_status "Building frontend development image..."
    docker-compose -f docker-compose.dev.yml build frontend-dev
    print_success "Image built successfully"
}

# Function to clean up
clean_up() {
    print_status "Cleaning up Docker resources..."
    docker-compose -f docker-compose.dev.yml --profile api-gateway down --rmi all --volumes --remove-orphans
    print_success "Cleanup completed"
}

# Main script logic
case "${1:-start}" in
    "start")
        start_frontend
        ;;
    "full")
        start_full
        ;;
    "stop")
        stop_containers
        ;;
    "restart")
        restart_containers
        ;;
    "logs")
        show_logs
        ;;
    "shell")
        open_shell
        ;;
    "build")
        build_image
        ;;
    "clean")
        clean_up
        ;;
    "help"|"-h"|"--help")
        show_help
        ;;
    *)
        print_error "Unknown command: $1"
        echo ""
        show_help
        exit 1
        ;;
esac 