#!/bin/bash

# TracSeq 2.0 Workspace Test Script
# Tests that all components are properly configured and can start

set -e  # Exit on any error

echo "🧬 TracSeq 2.0 Workspace Structure Test"
echo "========================================"

# Color codes for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Helper functions
log_info() {
    echo -e "${BLUE}ℹ️  $1${NC}"
}

log_success() {
    echo -e "${GREEN}✅ $1${NC}"
}

log_warning() {
    echo -e "${YELLOW}⚠️  $1${NC}"
}

log_error() {
    echo -e "${RED}❌ $1${NC}"
}

# Check if we're in the workspace root
if [[ ! -f "Cargo.toml" ]] || [[ ! -d "lab_manager" ]] || [[ ! -d "lab_submission_rag" ]]; then
    log_error "Not in TracSeq 2.0 workspace root directory!"
    log_info "Please run this script from the workspace root."
    exit 1
fi

log_info "Testing workspace structure..."

# Test 1: Verify directory structure
log_info "1. Checking directory structure..."

required_dirs=(
    "lab_manager"
    "lab_submission_rag" 
    "scripts"
    "deploy"
    "docs"
    "deploy/production"
    "deploy/development"
)

for dir in "${required_dirs[@]}"; do
    if [[ -d "$dir" ]]; then
        log_success "$dir/ exists"
    else
        log_error "$dir/ is missing!"
        exit 1
    fi
done

# Test 2: Verify key files
log_info "2. Checking key configuration files..."

required_files=(
    "Cargo.toml"
    "docker-compose.yml"
    "lab_manager/Cargo.toml"
    "lab_manager/frontend/package.json"
    "lab_submission_rag/pyproject.toml"
    "deploy/tracseq.env"
)

for file in "${required_files[@]}"; do
    if [[ -f "$file" ]]; then
        log_success "$file exists"
    else
        log_error "$file is missing!"
        exit 1
    fi
done

# Test 3: Check Docker configuration
log_info "3. Validating Docker configuration..."

if command -v docker &> /dev/null; then
    log_success "Docker is installed"
    
    if command -v docker-compose &> /dev/null; then
        log_success "Docker Compose is installed"
        
        # Test docker-compose syntax
        if docker-compose config &> /dev/null; then
            log_success "docker-compose.yml syntax is valid"
        else
            log_error "docker-compose.yml has syntax errors!"
            exit 1
        fi
    else
        log_warning "Docker Compose not found - you may need to install it"
    fi
else
    log_warning "Docker not found - required for running the application"
fi

# Test 4: Check Rust workspace
log_info "4. Validating Rust workspace..."

if command -v cargo &> /dev/null; then
    log_success "Cargo is installed"
    
    # Check workspace configuration
    if cargo check --workspace --quiet &> /dev/null; then
        log_success "Rust workspace configuration is valid"
    else
        log_warning "Rust workspace has issues - run 'cargo check' for details"
    fi
else
    log_info "Cargo not found - install Rust for development"
fi

# Test 5: Check Python RAG service
log_info "5. Validating Python RAG service..."

if command -v python3 &> /dev/null; then
    log_success "Python 3 is installed"
    
    cd lab_submission_rag
    if python3 -c "import tomllib; tomllib.load(open('pyproject.toml', 'rb'))" &> /dev/null; then
        log_success "Python project configuration is valid"
    else
        log_warning "Python project configuration may have issues"
    fi
    cd ..
else
    log_info "Python 3 not found - install Python for RAG development"
fi

# Test 6: Check Node.js frontend
log_info "6. Validating React frontend..."

if command -v node &> /dev/null; then
    log_success "Node.js is installed"
    
    cd lab_manager/frontend
    if [[ -f "package.json" ]]; then
        log_success "Frontend package.json exists"
        
        if command -v npm &> /dev/null && npm list &> /dev/null; then
            log_success "Frontend dependencies are installed"
        else
            log_info "Frontend dependencies not installed - run 'npm install'"
        fi
    fi
    cd ../..
else
    log_info "Node.js not found - install Node.js for frontend development"
fi

# Test 7: Test quick startup (optional)
log_info "7. Testing quick startup (optional)..."

if [[ "${1:-}" == "--test-startup" ]]; then
    log_info "Starting services for 30 seconds..."
    
    # Start services in background
    docker-compose up -d --quiet-pull &> /dev/null
    
    sleep 5
    
    # Check if services are running
    if docker-compose ps | grep -q "Up"; then
        log_success "Services started successfully"
        
        # Test health endpoints if possible
        if curl -s http://localhost:3000/health &> /dev/null; then
            log_success "Lab Manager health check passed"
        else
            log_info "Lab Manager not yet ready (normal for initial startup)"
        fi
        
        if curl -s http://localhost:8000/health &> /dev/null; then
            log_success "RAG Service health check passed"
        else
            log_info "RAG Service not yet ready (normal for initial startup)"
        fi
        
        sleep 25
        
        log_info "Stopping test services..."
        docker-compose down --quiet &> /dev/null
        log_success "Services stopped cleanly"
    else
        log_error "Services failed to start properly"
        docker-compose logs --tail=20
        docker-compose down &> /dev/null
        exit 1
    fi
else
    log_info "Use --test-startup to test actual service startup"
fi

echo ""
echo "🎉 Workspace structure test completed successfully!"
echo ""
echo "📋 Next steps:"
echo "   • Run 'docker-compose up -d' to start all services"
echo "   • Visit http://localhost:5173 for the frontend"
echo "   • Visit http://localhost:3000 for the API"
echo "   • Visit http://localhost:8000 for the RAG service"
echo ""
echo "📖 For more information:"
echo "   • Read the README.md for detailed setup instructions"
echo "   • Check docs/ for component-specific documentation"
echo "   • Use scripts/ for development and deployment helpers"
echo ""

log_success "TracSeq 2.0 workspace is ready! 🚀" 
