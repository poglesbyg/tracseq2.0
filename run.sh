#!/bin/bash

# TracSeq 2.0 Runner Script for Windows
# This script manages both lab_manager and lab_submission_rag services

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Function to print colored output
print_info() {
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

print_header() {
    echo -e "${BLUE}================================${NC}"
    echo -e "${BLUE}  TracSeq 2.0 - $1${NC}"
    echo -e "${BLUE}================================${NC}"
}

# Function to check if Docker is running
check_docker() {
    if ! docker info >/dev/null 2>&1; then
        print_error "Docker is not running. Please start Docker Desktop and try again."
        exit 1
    fi
    print_success "Docker is running"
}

# Function to check if required files exist
check_files() {
    if [ ! -f "lab_manager/docker-compose.yml" ]; then
        print_error "lab_manager/docker-compose.yml not found"
        exit 1
    fi
    
    if [ ! -f "lab_submission_rag/docker-compose.yml" ]; then
        print_error "lab_submission_rag/docker-compose.yml not found"
        exit 1
    fi
    
    print_success "All required files found"
}

# Function to setup environment variables for RAG service
setup_rag_env() {
    print_info "Setting up RAG service environment..."
    
    # Create .env file if it doesn't exist
    if [ ! -f "lab_submission_rag/.env" ]; then
        print_warning "Creating default .env file for RAG service"
        cat > lab_submission_rag/.env << EOF
# LLM Configuration
OPENAI_API_KEY=your_openai_key_here
ANTHROPIC_API_KEY=your_anthropic_key_here

# Ollama Configuration (for local LLM)
USE_OLLAMA=true
OLLAMA_MODEL=llama2
OLLAMA_BASE_URL=http://localhost:11434

# LLM Parameters
LLM_TEMPERATURE=0.7
MAX_TOKENS=2048
EOF
        print_warning "Please edit lab_submission_rag/.env with your API keys"
    fi
    
    # Setup Ollama
    setup_ollama
}

# Function to create required directories
create_directories() {
    print_info "Creating required directories..."
    
    # RAG service directories
    mkdir -p lab_submission_rag/uploads
    mkdir -p lab_submission_rag/exports
    mkdir -p lab_submission_rag/logs
    mkdir -p lab_submission_rag/data
    
    # Lab manager directories
    mkdir -p lab_manager/storage
    mkdir -p lab_manager/uploads
    
    print_success "Directories created"
}

# Function to start lab manager in production mode
start_lab_manager_prod() {
    print_info "Starting Lab Manager (Production)..."
    cd lab_manager
    docker-compose up -d frontend app db
    cd ..
    print_success "Lab Manager started on http://localhost:8080"
}

# Function to start lab manager in development mode
start_lab_manager_dev() {
    print_info "Starting Lab Manager (Development)..."
    cd lab_manager
    docker-compose up -d frontend-dev dev db
    cd ..
    print_success "Lab Manager (Dev) started on http://localhost:5173"
}

# Function to start RAG service
start_rag_service() {
    print_info "Starting RAG Service..."
    cd lab_submission_rag
    docker-compose up -d
    cd ..
    print_success "RAG Service started on http://localhost:8000"
}

# Function to stop all services
stop_services() {
    print_info "Stopping all services..."
    
    cd lab_manager
    docker-compose down
    cd ..
    
    cd lab_submission_rag
    docker-compose down
    cd ..
    
    # Stop Ollama if it's running
    if command -v ollama &> /dev/null; then
        print_info "Stopping Ollama service..."
        pkill -f "ollama serve" 2>/dev/null || true
    fi
    
    print_success "All services stopped"
}

# Function to show service status
show_status() {
    print_header "Service Status"
    
    echo -e "\n${YELLOW}Lab Manager Services:${NC}"
    cd lab_manager
    docker-compose ps
    cd ..
    
    echo -e "\n${YELLOW}RAG Service:${NC}"
    cd lab_submission_rag
    docker-compose ps
    cd ..
}

# Function to show logs
show_logs() {
    local service=$1
    if [ "$service" == "lab-manager" ]; then
        print_info "Showing Lab Manager logs..."
        cd lab_manager
        docker-compose logs -f
        cd ..
    elif [ "$service" == "rag" ]; then
        print_info "Showing RAG Service logs..."
        cd lab_submission_rag
        docker-compose logs -f
        cd ..
    else
        print_error "Invalid service. Use 'lab-manager' or 'rag'"
        exit 1
    fi
}

# Function to rebuild services
rebuild_services() {
    print_info "Rebuilding all services..."
    
    cd lab_manager
    docker-compose build --no-cache
    cd ..
    
    cd lab_submission_rag
    docker-compose build --no-cache
    cd ..
    
    print_success "All services rebuilt"
}

# Function to show help
show_help() {
    echo "TracSeq 2.0 Runner Script"
    echo ""
    echo "Usage: $0 [COMMAND]"
    echo ""
    echo "Commands:"
    echo "  start-prod     Start all services in production mode"
    echo "  start-dev      Start all services in development mode"
    echo "  stop           Stop all services"
    echo "  restart-prod   Restart all services in production mode"
    echo "  restart-dev    Restart all services in development mode"
    echo "  status         Show status of all services"
    echo "  logs <service> Show logs (service: lab-manager or rag)"
    echo "  rebuild        Rebuild all Docker images"
    echo "  clean          Clean up Docker resources"
    echo "  help           Show this help message"
    echo ""
    echo "Ollama Commands:"
    echo "  install-ollama Install Ollama for local LLM"
    echo "  start-ollama   Start Ollama service"
    echo "  stop-ollama    Stop Ollama service"
    echo "  pull-model     Download default model or: pull-model [model-name]"
    echo ""
    echo "Services:"
    echo "  - Lab Manager (Frontend): http://localhost:8080 (prod) or http://localhost:5173 (dev)"
    echo "  - Lab Manager (Backend): http://localhost:3001 (prod) or http://localhost:3000 (dev)"
    echo "  - RAG Service: http://localhost:8000"
    echo "  - PostgreSQL: localhost:5433"
}

# Function to clean up Docker resources
clean_docker() {
    print_info "Cleaning up Docker resources..."
    
    # Stop all services first
    stop_services
    
    # Remove unused containers, networks, images
    docker system prune -f
    
    # Remove volumes (optional - uncomment if you want to remove data)
    # docker volume prune -f
    
    print_success "Docker cleanup completed"
}

# Function to setup Ollama
setup_ollama() {
    print_info "Checking Ollama installation..."
    
    if ! command -v ollama &> /dev/null; then
        print_warning "Ollama not found. Run './run.sh install-ollama' to install it."
        print_info "Or download from: https://ollama.ai/download"
        return
    fi
    
    print_success "Ollama is installed"
    
    # Check if Ollama service is running
    if ! curl -s http://localhost:11434/api/version >/dev/null 2>&1; then
        print_info "Starting Ollama service..."
        ollama serve > /dev/null 2>&1 &
        sleep 3
    else
        print_success "Ollama service is already running"
    fi
    
    # Get model name from .env file
    local model_name="llama2"
    if [ -f "lab_submission_rag/.env" ]; then
        model_name=$(grep "OLLAMA_MODEL=" lab_submission_rag/.env | cut -d'=' -f2 | tr -d ' ')
        if [ -z "$model_name" ]; then
            model_name="llama2"
        fi
    fi
    
    # Check if model exists
    if ! ollama list | grep -q "$model_name"; then
        print_warning "Model $model_name not found. Run './run.sh pull-model' to download it."
    else
        print_success "Model $model_name is available"
    fi
}

# Function to install Ollama
install_ollama() {
    print_info "Installing Ollama..."
    
    # Detect OS
    if [[ "$OSTYPE" == "linux-gnu"* ]]; then
        print_info "Installing Ollama on Linux..."
        curl -fsSL https://ollama.ai/install.sh | sh
    elif [[ "$OSTYPE" == "darwin"* ]]; then
        print_info "Installing Ollama on macOS..."
        if command -v brew &> /dev/null; then
            brew install ollama
        else
            print_error "Homebrew not found. Please install from: https://ollama.ai/download"
            return 1
        fi
    elif [[ "$OSTYPE" == "msys" ]] || [[ "$OSTYPE" == "cygwin" ]]; then
        print_info "For Windows, please download from: https://ollama.ai/download/windows"
        print_info "Or use the batch script: run.bat install-ollama"
        return 1
    else
        print_error "Unsupported OS. Please install manually from: https://ollama.ai/download"
        return 1
    fi
    
    print_success "Ollama installation completed"
    print_info "Run './run.sh pull-model' to download a model"
}

# Function to start Ollama service
start_ollama() {
    print_info "Starting Ollama service..."
    
    if ! command -v ollama &> /dev/null; then
        print_error "Ollama not installed. Run './run.sh install-ollama' first."
        return 1
    fi
    
    ollama serve > /dev/null 2>&1 &
    print_success "Ollama service started"
}

# Function to stop Ollama service
stop_ollama() {
    print_info "Stopping Ollama service..."
    pkill -f "ollama serve" || print_info "Ollama service was not running"
    print_success "Ollama service stopped"
}

# Function to pull Ollama model
pull_model() {
    local model_name="${1:-llama2}"
    
    # Get model name from .env file if not provided
    if [ -z "$1" ] && [ -f "lab_submission_rag/.env" ]; then
        model_name=$(grep "OLLAMA_MODEL=" lab_submission_rag/.env | cut -d'=' -f2 | tr -d ' ')
        if [ -z "$model_name" ]; then
            model_name="llama2"
        fi
    fi
    
    print_info "Pulling Ollama model: $model_name"
    print_info "This may take several minutes depending on model size..."
    
    if ollama pull "$model_name"; then
        print_success "Model $model_name downloaded successfully"
    else
        print_error "Failed to download model $model_name"
        print_info "Available models: llama2, llama3, codellama, mistral, neural-chat"
        return 1
    fi
}

# Main script logic
case "$1" in
    "start-prod")
        print_header "Starting Production Mode"
        check_docker
        check_files
        setup_rag_env
        create_directories
        start_lab_manager_prod
        start_rag_service
        echo ""
        print_success "All services started successfully!"
        echo -e "${GREEN}Lab Manager Frontend:${NC} http://localhost:8080"
        echo -e "${GREEN}Lab Manager Backend:${NC} http://localhost:3001"
        echo -e "${GREEN}RAG Service:${NC} http://localhost:8000"
        ;;
    
    "start-dev")
        print_header "Starting Development Mode"
        check_docker
        check_files
        setup_rag_env
        create_directories
        start_lab_manager_dev
        start_rag_service
        echo ""
        print_success "All services started successfully!"
        echo -e "${GREEN}Lab Manager Frontend (Dev):${NC} http://localhost:5173"
        echo -e "${GREEN}Lab Manager Backend (Dev):${NC} http://localhost:3000"
        echo -e "${GREEN}RAG Service:${NC} http://localhost:8000"
        ;;
    
    "stop")
        print_header "Stopping Services"
        stop_services
        ;;
    
    "restart-prod")
        print_header "Restarting (Production)"
        stop_services
        sleep 2
        check_docker
        check_files
        setup_rag_env
        create_directories
        start_lab_manager_prod
        start_rag_service
        print_success "All services restarted successfully!"
        ;;
    
    "restart-dev")
        print_header "Restarting (Development)"
        stop_services
        sleep 2
        check_docker
        check_files
        setup_rag_env
        create_directories
        start_lab_manager_dev
        start_rag_service
        print_success "All services restarted successfully!"
        ;;
    
    "status")
        show_status
        ;;
    
    "logs")
        if [ -z "$2" ]; then
            print_error "Please specify a service: lab-manager or rag"
            exit 1
        fi
        show_logs "$2"
        ;;
    
    "rebuild")
        print_header "Rebuilding Services"
        rebuild_services
        ;;
    
    "clean")
        print_header "Cleaning Docker Resources"
        clean_docker
        ;;
    
    "install-ollama")
        print_header "Installing Ollama"
        install_ollama
        ;;
    
    "start-ollama")
        print_header "Starting Ollama"
        start_ollama
        ;;
    
    "stop-ollama")
        print_header "Stopping Ollama"
        stop_ollama
        ;;
    
    "pull-model")
        print_header "Pulling Ollama Model"
        pull_model "$2"
        ;;
    
    "help"|"--help"|"-h"|"")
        show_help
        ;;
    
    *)
        print_error "Unknown command: $1"
        show_help
        exit 1
        ;;
esac 
