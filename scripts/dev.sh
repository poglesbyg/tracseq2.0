#!/bin/bash
# Development helper script for LIMS Microservices

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# Function to print colored output
print_status() {
    echo -e "${GREEN}[INFO]${NC} $1"
}

print_error() {
    echo -e "${RED}[ERROR]${NC} $1"
}

print_warning() {
    echo -e "${YELLOW}[WARN]${NC} $1"
}

# Main menu
show_menu() {
    echo "================================"
    echo "   LIMS Development Helper"
    echo "================================"
    echo "1. Start all services (Docker)"
    echo "2. Start specific service"
    echo "3. Run tests"
    echo "4. Build all services"
    echo "5. Clean and reset"
    echo "6. View logs"
    echo "7. Database operations"
    echo "8. Exit"
    echo "================================"
}

# Start all services
start_all() {
    print_status "Starting all services with Docker Compose..."
    cd docker
    docker-compose up -d
    cd ..
    print_status "All services started!"
    echo ""
    echo "Services available at:"
    echo "  - Frontend: http://localhost:3000"
    echo "  - API Gateway: http://localhost:8080"
    echo "  - Auth Service: http://localhost:8001"
    echo "  - Sample Service: http://localhost:8002"
}

# Start specific service
start_service() {
    echo "Select service to start:"
    echo "1. Frontend (lims-ui)"
    echo "2. Auth Service"
    echo "3. Sample Service"
    echo "4. Storage Service"
    echo "5. RAG Service"
    read -p "Enter choice: " service_choice
    
    case $service_choice in
        1)
            print_status "Starting frontend..."
            cd lims-ui
            pnpm dev
            ;;
        2)
            print_status "Starting auth service..."
            cd lims-core/auth_service
            cargo run
            ;;
        3)
            print_status "Starting sample service..."
            cd lims-core/sample_service
            cargo run
            ;;
        4)
            print_status "Starting storage service..."
            cd lims-core/enhanced_storage_service
            cargo run
            ;;
        5)
            print_status "Starting RAG service..."
            cd lims-ai/enhanced_rag_service
            python -m src.main
            ;;
        *)
            print_error "Invalid choice"
            ;;
    esac
}

# Run tests
run_tests() {
    echo "Select test suite:"
    echo "1. All tests"
    echo "2. Rust tests (lims-core)"
    echo "3. Frontend tests (lims-ui)"
    echo "4. Python tests (lims-ai)"
    read -p "Enter choice: " test_choice
    
    case $test_choice in
        1)
            print_status "Running all tests..."
            cd lims-core && cargo test --workspace
            cd ../lims-ui && pnpm test
            cd ../lims-ai && pytest
            ;;
        2)
            print_status "Running Rust tests..."
            cd lims-core
            cargo test --workspace
            ;;
        3)
            print_status "Running frontend tests..."
            cd lims-ui
            pnpm test
            ;;
        4)
            print_status "Running Python tests..."
            cd lims-ai
            pytest
            ;;
        *)
            print_error "Invalid choice"
            ;;
    esac
}

# Build all services
build_all() {
    print_status "Building all services..."
    
    # Build Rust services
    print_status "Building Rust services..."
    cd lims-core
    cargo build --release --workspace
    cd ..
    
    # Build frontend
    print_status "Building frontend..."
    cd lims-ui
    pnpm build
    cd ..
    
    # Build Docker images
    print_status "Building Docker images..."
    cd docker
    docker-compose build
    cd ..
    
    print_status "Build complete!"
}

# Clean and reset
clean_reset() {
    print_warning "This will stop all services and clean build artifacts."
    read -p "Continue? (y/n): " confirm
    
    if [ "$confirm" = "y" ]; then
        print_status "Stopping services..."
        cd docker
        docker-compose down -v
        cd ..
        
        print_status "Cleaning build artifacts..."
        cd lims-core
        cargo clean
        cd ../lims-ui
        rm -rf dist node_modules
        cd ..
        
        print_status "Clean complete!"
    fi
}

# View logs
view_logs() {
    echo "Select service logs:"
    echo "1. All services"
    echo "2. Frontend"
    echo "3. Auth Service"
    echo "4. Sample Service"
    echo "5. Database"
    read -p "Enter choice: " log_choice
    
    cd docker
    case $log_choice in
        1)
            docker-compose logs -f
            ;;
        2)
            docker-compose logs -f frontend
            ;;
        3)
            docker-compose logs -f auth-service
            ;;
        4)
            docker-compose logs -f sample-service
            ;;
        5)
            docker-compose logs -f postgres
            ;;
        *)
            print_error "Invalid choice"
            ;;
    esac
    cd ..
}

# Database operations
db_operations() {
    echo "Database operations:"
    echo "1. Run migrations"
    echo "2. Seed database"
    echo "3. Reset database"
    echo "4. Connect to psql"
    read -p "Enter choice: " db_choice
    
    case $db_choice in
        1)
            print_status "Running migrations..."
            # Add migration logic here
            ;;
        2)
            print_status "Seeding database..."
            # Add seed logic here
            ;;
        3)
            print_warning "This will delete all data!"
            read -p "Continue? (y/n): " confirm
            if [ "$confirm" = "y" ]; then
                cd docker
                docker-compose exec postgres psql -U postgres -c "DROP DATABASE IF EXISTS lims_db; CREATE DATABASE lims_db;"
                cd ..
            fi
            ;;
        4)
            cd docker
            docker-compose exec postgres psql -U postgres lims_db
            cd ..
            ;;
        *)
            print_error "Invalid choice"
            ;;
    esac
}

# Main loop
while true; do
    show_menu
    read -p "Enter your choice: " choice
    
    case $choice in
        1) start_all ;;
        2) start_service ;;
        3) run_tests ;;
        4) build_all ;;
        5) clean_reset ;;
        6) view_logs ;;
        7) db_operations ;;
        8) 
            print_status "Goodbye!"
            exit 0 
            ;;
        *)
            print_error "Invalid choice. Please try again."
            ;;
    esac
    
    echo ""
    read -p "Press Enter to continue..."
    clear
done 