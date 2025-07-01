#!/bin/bash

# 🔧 TracSeq 2.0 Build Test Script
# Tests Docker builds to ensure they work correctly

set -e

echo "🔧 Testing TracSeq 2.0 Docker Builds"
echo "====================================="
echo ""

# Colors for output
GREEN='\033[0;32m'
RED='\033[0;31m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

print_info() {
    echo -e "${BLUE}[INFO]${NC} $1"
}

print_success() {
    echo -e "${GREEN}[SUCCESS]${NC} $1"
}

print_error() {
    echo -e "${RED}[ERROR]${NC} $1"
}

# Function to test backend build
test_backend_build() {
    print_info "Testing lab_manager backend build..."
    
    cd lab_manager
    
    if docker build -f Dockerfile -t tracseq-backend-test .; then
        print_success "Backend production build successful!"
    else
        print_error "Backend production build failed!"
        return 1
    fi
    
    if docker build -f Dockerfile.dev -t tracseq-backend-dev-test .; then
        print_success "Backend development build successful!"
    else
        print_error "Backend development build failed!"
        return 1
    fi
    
    cd ..
}

# Function to test frontend build
test_frontend_build() {
    print_info "Testing frontend build..."
    
    cd lab_manager/frontend
    
    if docker build -f Dockerfile -t tracseq-frontend-test .; then
        print_success "Frontend production build successful!"
    else
        print_error "Frontend production build failed!"
        return 1
    fi
    
    if docker build -f Dockerfile.dev -t tracseq-frontend-dev-test .; then
        print_success "Frontend development build successful!"
    else
        print_error "Frontend development build failed!"
        return 1
    fi
    
    cd ../..
}

# Function to test RAG service build
test_rag_build() {
    print_info "Testing RAG service build..."
    
    cd lab_submission_rag
    
    if docker build -f Dockerfile -t tracseq-rag-test .; then
        print_success "RAG service build successful!"
    else
        print_error "RAG service build failed!"
        return 1
    fi
    
    cd ..
}

# Function to clean up test images
cleanup_test_images() {
    print_info "Cleaning up test images..."
    
    docker rmi tracseq-backend-test 2>/dev/null || true
    docker rmi tracseq-backend-dev-test 2>/dev/null || true
    docker rmi tracseq-frontend-test 2>/dev/null || true
    docker rmi tracseq-frontend-dev-test 2>/dev/null || true
    docker rmi tracseq-rag-test 2>/dev/null || true
    
    print_success "Test images cleaned up!"
}

# Function to test docker-compose syntax
test_compose_syntax() {
    print_info "Testing docker-compose file syntax..."
    
    if docker-compose config --quiet; then
        print_success "Docker Compose syntax is valid!"
    else
        print_error "Docker Compose syntax error!"
        return 1
    fi
}

# Main test function
main() {
    print_info "Starting build tests..."
    echo ""
    
    # Test docker-compose syntax first
    test_compose_syntax
    echo ""
    
    # Test individual service builds
    test_backend_build
    echo ""
    
    test_frontend_build
    echo ""
    
    test_rag_build
    echo ""
    
    # Clean up
    cleanup_test_images
    echo ""
    
    print_success "🎉 All build tests passed!"
    echo ""
    echo "Next steps:"
    echo "  1. Start services: docker-compose up -d"
    echo "  2. Initialize Ollama: ./deploy/azure/ollama-init.sh"
    echo "  3. Test application: http://localhost:5173"
}

# Handle script arguments
case "${1:-}" in
    --help|-h)
        echo "TracSeq 2.0 Build Test Script"
        echo ""
        echo "This script tests all Docker builds to ensure they work correctly."
        echo ""
        echo "Usage:"
        echo "  ./scripts/test-build.sh           # Run all build tests"
        echo "  ./scripts/test-build.sh --help    # Show this help"
        echo ""
        exit 0
        ;;
    *)
        main
        ;;
esac 
