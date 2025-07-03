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

print_info "🧪 Lab Manager Test Suite"
echo

# Check if we're in the right directory
if [ ! -f "Cargo.toml" ]; then
    print_error "Please run this script from the lab_manager directory"
    exit 1
fi

# Set up test environment variables
export TEST_DATABASE_URL="${TEST_DATABASE_URL:-postgres://lab_manager:lab_manager@localhost:5432/lab_manager_test}"
export RUST_LOG="${RUST_LOG:-info}"
export JWT_SECRET="test-jwt-secret-key"

print_info "Setting up test database..."

# Check if PostgreSQL is running
if ! pg_isready -q 2>/dev/null; then
    print_warning "PostgreSQL is not running locally. Please start PostgreSQL first."
    print_info "You can start it with: sudo systemctl start postgresql"
    exit 1
fi

# Create test database if it doesn't exist
if ! psql -U lab_manager -h localhost -lqt | cut -d \| -f 1 | grep -qw lab_manager_test; then
    print_info "Creating test database..."
    createdb -U lab_manager -h localhost lab_manager_test 2>/dev/null || true
fi

# Run migrations on test database
print_info "Running migrations on test database..."
DATABASE_URL="$TEST_DATABASE_URL" sqlx migrate run || print_warning "Migration failed - continuing anyway"

# Run unit tests
print_info "Running unit tests..."
cargo test --lib auth_tests -- --nocapture

# Run integration tests
print_info "Running integration tests..."
cargo test --lib auth_integration_tests -- --nocapture

# Run all authentication-related tests
print_info "Running all authentication tests..."
cargo test auth -- --nocapture

# Run template tests
print_info "Running template tests..."
cargo test template_tests -- --nocapture

# Run validation tests
print_info "Running validation tests..."
cargo test validation_tests -- --nocapture

# Run session security tests
print_info "Running session security tests..."
cargo test session_security_tests -- --nocapture

# Run all tests (optional)
if [ "$1" = "--all" ]; then
    print_info "Running complete test suite..."
    cargo test -- --nocapture
fi

# Test compilation of binaries
print_info "Testing binary compilation..."
cargo check --bin lab_manager
cargo check --bin create_admin

print_success "✅ All tests completed!"
echo
print_info "📊 Test Coverage Summary:"
echo "  • Authentication: Unit + Integration tests (auth_tests.rs)"
echo "  • User Management: Model and service tests"
echo "  • Password Security: Argon2 hashing and validation"
echo "  • JWT Tokens: Generation and validation tests"
echo "  • Input Validation: Comprehensive field validation (validation_tests.rs)"
echo "  • Session Security: Token management and security (session_security_tests.rs)"
echo "  • Role-based Access: Permission system tests"
echo "  • Templates: Basic model tests"
echo "  • RAG Integration: Document processing tests"
echo
print_info "🚀 To run specific test categories:"
echo "  • Auth only: cargo test auth"
echo "  • Validation tests: cargo test validation_tests"
echo "  • Session security: cargo test session_security_tests"
echo "  • Integration tests: cargo test auth_integration_tests"
echo "  • Unit tests: cargo test --lib"
echo "  • All tests: ./scripts/run_tests.sh --all"
echo
print_info "🔧 Environment:"
echo "  • Test Database: $TEST_DATABASE_URL"
echo "  • JWT Secret: Set for testing"
echo "  • Log Level: $RUST_LOG" 
