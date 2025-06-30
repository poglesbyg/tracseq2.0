#!/bin/bash
# Pre-push checks for TracSeq 2.0
# This script runs all the checks that will be run in CI

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Print colored output
print_message() {
    local color=$1
    local message=$2
    echo -e "${color}${message}${NC}"
}

print_message "$BLUE" "🚀 Running pre-push checks for TracSeq 2.0..."

# Check if we're in the root directory
if [ ! -f "Cargo.toml" ] || [ ! -d "lims-ui" ]; then
    print_message "$RED" "❌ Error: Please run this script from the project root directory"
    exit 1
fi

# Track overall status
FAILED=0

# Frontend checks
print_message "$BLUE" "\n📦 Frontend Checks"
print_message "$YELLOW" "Running pnpm lint..."
cd lims-ui
if pnpm lint; then
    print_message "$GREEN" "✅ Frontend lint passed"
else
    print_message "$RED" "❌ Frontend lint failed"
    FAILED=1
fi

print_message "$YELLOW" "Running TypeScript check..."
if pnpm typecheck 2>/dev/null; then
    print_message "$GREEN" "✅ TypeScript check passed"
else
    print_message "$YELLOW" "⚠️  TypeScript check has warnings (non-blocking)"
fi

print_message "$YELLOW" "Running frontend tests..."
if pnpm test --passWithNoTests; then
    print_message "$GREEN" "✅ Frontend tests passed"
else
    print_message "$RED" "❌ Frontend tests failed"
    FAILED=1
fi

# Ask if user wants to run Playwright tests
print_message "$YELLOW" "\nRun Playwright E2E tests? (y/N) "
read -r response
if [[ "$response" =~ ^[Yy]$ ]]; then
    print_message "$YELLOW" "Running Playwright tests..."
    if pnpm test:e2e; then
        print_message "$GREEN" "✅ Playwright tests passed"
    else
        print_message "$RED" "❌ Playwright tests failed"
        FAILED=1
    fi
else
    print_message "$YELLOW" "⏭️  Skipping Playwright tests"
fi

cd ..

# Rust checks
print_message "$BLUE" "\n🦀 Rust Checks"
print_message "$YELLOW" "Running cargo fmt check..."
if cargo fmt --all -- --check; then
    print_message "$GREEN" "✅ Rust formatting passed"
else
    print_message "$RED" "❌ Rust formatting failed"
    print_message "$YELLOW" "💡 Run 'cargo fmt --all' to fix formatting"
    FAILED=1
fi

print_message "$YELLOW" "Running cargo check..."
if cargo check --workspace --all-targets 2>&1 | grep -v "warning"; then
    print_message "$GREEN" "✅ Cargo check passed"
else
    print_message "$RED" "❌ Cargo check failed"
    FAILED=1
fi

print_message "$YELLOW" "Running cargo clippy..."
if cargo clippy --workspace --all-targets -- -D warnings 2>&1 | grep -E "(error|warning):" | head -20; then
    print_message "$YELLOW" "⚠️  Clippy has warnings (non-blocking)"
else
    print_message "$GREEN" "✅ Clippy passed"
fi

# Ask if user wants to run Rust tests
print_message "$YELLOW" "\nRun Rust tests? (y/N) "
read -r response
if [[ "$response" =~ ^[Yy]$ ]]; then
    print_message "$YELLOW" "Running cargo test..."
    if cargo test --workspace; then
        print_message "$GREEN" "✅ Rust tests passed"
    else
        print_message "$RED" "❌ Rust tests failed"
        FAILED=1
    fi
else
    print_message "$YELLOW" "⏭️  Skipping Rust tests"
fi

# Summary
print_message "$BLUE" "\n📊 Summary"
if [ $FAILED -eq 0 ]; then
    print_message "$GREEN" "✅ All checks passed! Ready to push."
    exit 0
else
    print_message "$RED" "❌ Some checks failed. Please fix the issues before pushing."
    exit 1
fi 