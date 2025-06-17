#!/bin/bash

# 🧠 TracSeq 2.0 RAG Service Build Test Script
# Tests RAG service builds with different dependency configurations

set -e

echo "🧠 Testing TracSeq 2.0 RAG Service Builds"
echo "=========================================="
echo ""

# Colors for output
GREEN='\033[0;32m'
RED='\033[0;31m'
BLUE='\033[0;34m'
YELLOW='\033[1;33m'
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

print_warning() {
    echo -e "${YELLOW}[WARNING]${NC} $1"
}

# Function to test standard RAG build
test_standard_rag_build() {
    print_info "Testing standard RAG service build..."
    
    cd lab_submission_rag
    
    if docker build -f Dockerfile -t tracseq-rag-standard-test .; then
        print_success "Standard RAG build successful!"
        cd ..
        return 0
    else
        print_error "Standard RAG build failed!"
        cd ..
        return 1
    fi
}

# Function to test lightweight RAG build
test_lightweight_rag_build() {
    print_info "Testing lightweight RAG service build..."
    
    cd lab_submission_rag
    
    if docker build -f Dockerfile.lite -t tracseq-rag-lite-test .; then
        print_success "Lightweight RAG build successful!"
        cd ..
        return 0
    else
        print_error "Lightweight RAG build failed!"
        cd ..
        return 1
    fi
}

# Function to clean up test images
cleanup_test_images() {
    print_info "Cleaning up RAG test images..."
    
    docker rmi tracseq-rag-standard-test 2>/dev/null || true
    docker rmi tracseq-rag-lite-test 2>/dev/null || true
    
    print_success "RAG test images cleaned up!"
}

# Function to show troubleshooting tips
show_troubleshooting() {
    echo ""
    print_warning "Troubleshooting RAG Service Build Issues:"
    echo ""
    echo "If you encounter dependency issues:"
    echo ""
    echo "1. 📦 Use Lightweight Build:"
    echo "   docker-compose -f docker-compose.yml up --build rag-service"
    echo "   # Change 'dockerfile: Dockerfile' to 'dockerfile: Dockerfile.lite'"
    echo ""
    echo "2. 🔧 Manual Dependency Fix:"
    echo "   cd lab_submission_rag"
    echo "   pip install --upgrade pip"
    echo "   pip install -r requirements-lite.txt"
    echo ""
    echo "3. 🏗️ Complete Clean Build:"
    echo "   docker system prune -f"
    echo "   docker-compose build --no-cache rag-service"
    echo ""
    echo "4. 🐛 Hash Verification Issues:"
    echo "   # Use requirements-lite.txt (already fixed)"
    echo "   # or disable hash checking with --trusted-host flags"
    echo ""
}

# Main test function
main() {
    print_info "Starting RAG service build tests..."
    echo ""
    
    # Try standard build first
    if test_standard_rag_build; then
        echo ""
        print_success "✅ Standard RAG build works - no changes needed!"
        echo ""
        echo "Your RAG service is ready to use with full dependencies."
        echo "To start: docker-compose up -d rag-service"
    else
        echo ""
        print_warning "⚠️ Standard RAG build failed - trying lightweight build..."
        echo ""
        
        # Try lightweight build as fallback
        if test_lightweight_rag_build; then
            echo ""
            print_success "✅ Lightweight RAG build successful!"
            echo ""
            echo "Recommendation: Use the lightweight build for better compatibility."
            echo "Edit docker-compose.yml:"
            echo "  Change: dockerfile: Dockerfile"
            echo "  To:     dockerfile: Dockerfile.lite"
            echo ""
        else
            print_error "❌ Both RAG builds failed!"
            show_troubleshooting
        fi
    fi
    
    # Clean up
    cleanup_test_images
    echo ""
    
    print_info "RAG build test completed!"
}

# Handle script arguments
case "${1:-}" in
    --help|-h)
        echo "TracSeq 2.0 RAG Service Build Test Script"
        echo ""
        echo "This script tests RAG service Docker builds and provides alternatives for dependency issues."
        echo ""
        echo "Usage:"
        echo "  ./scripts/test-rag-build.sh           # Run RAG build tests"
        echo "  ./scripts/test-rag-build.sh --help    # Show this help"
        echo ""
        exit 0
        ;;
    --lite)
        print_info "Testing lightweight RAG build only..."
        test_lightweight_rag_build
        cleanup_test_images
        ;;
    --standard)
        print_info "Testing standard RAG build only..."
        test_standard_rag_build
        cleanup_test_images
        ;;
    *)
        main
        ;;
esac 
