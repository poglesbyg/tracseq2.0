#!/bin/bash
# TracSeq 2.0 Cleanup Helper Script

echo "🧹 TracSeq 2.0 Cleanup Helper"
echo "============================="

# Color codes
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# Function to print colored output
print_status() {
    case $1 in
        "error") echo -e "${RED}❌ $2${NC}" ;;
        "success") echo -e "${GREEN}✅ $2${NC}" ;;
        "warning") echo -e "${YELLOW}⚠️  $2${NC}" ;;
        "info") echo -e "ℹ️  $2" ;;
    esac
}

# Check if running from workspace root
if [ ! -f "package.json" ] || [ ! -f "Cargo.toml" ]; then
    print_status "error" "Please run this script from the workspace root directory"
    exit 1
fi

# Menu options
show_menu() {
    echo ""
    echo "Select an option:"
    echo "1) Fix Rust warnings (automatic)"
    echo "2) Check Python syntax errors"
    echo "3) Run frontend checks"
    echo "4) Check database migrations"
    echo "5) Count remaining issues"
    echo "6) Run all checks"
    echo "0) Exit"
    echo ""
}

# Fix Rust warnings
fix_rust_warnings() {
    print_status "info" "Fixing Rust warnings..."
    
    # Run cargo fix for workspace
    cargo fix --workspace --allow-dirty 2>&1 | tail -5
    
    # Count remaining warnings
    WARNING_COUNT=$(cargo check --workspace 2>&1 | grep -c "warning:")
    
    if [ $WARNING_COUNT -gt 0 ]; then
        print_status "warning" "Still have $WARNING_COUNT warnings remaining"
    else
        print_status "success" "All Rust warnings fixed!"
    fi
}

# Check Python syntax
check_python() {
    print_status "info" "Checking Python syntax..."
    
    # Check lab_submission_rag
    if [ -d "lab_submission_rag" ]; then
        cd lab_submission_rag
        ERRORS=$(python3 -m py_compile *.py 2>&1 | grep -c "Error")
        if [ $ERRORS -gt 0 ]; then
            print_status "error" "Found $ERRORS Python syntax errors in lab_submission_rag"
            python3 -m py_compile *.py 2>&1 | grep "Error" | head -5
        else
            print_status "success" "No Python syntax errors in lab_submission_rag"
        fi
        cd ..
    fi
    
    # Check enhanced_rag_service
    if [ -d "enhanced_rag_service" ]; then
        cd enhanced_rag_service
        if [ -f "pyproject.toml" ]; then
            print_status "info" "enhanced_rag_service uses pyproject.toml"
        fi
        cd ..
    fi
}

# Run frontend checks
check_frontend() {
    print_status "info" "Running frontend checks..."
    
    # TypeScript check
    print_status "info" "Running TypeScript check..."
    pnpm typecheck 2>&1 | tail -5
    
    # ESLint check
    print_status "info" "Running ESLint..."
    pnpm lint 2>&1 | tail -5
}

# Check database migrations
check_migrations() {
    print_status "info" "Checking database migrations..."
    
    SERVICES=("auth_service" "lab_manager" "sample_service" "transaction_service")
    
    for service in "${SERVICES[@]}"; do
        if [ -d "$service/migrations" ]; then
            MIGRATION_COUNT=$(ls -1 $service/migrations/*.sql 2>/dev/null | wc -l)
            print_status "info" "$service has $MIGRATION_COUNT migration files"
        else
            print_status "warning" "$service has no migrations directory"
        fi
    done
}

# Count all remaining issues
count_issues() {
    print_status "info" "Counting remaining issues..."
    echo ""
    
    # Rust warnings
    RUST_WARNINGS=$(cargo check --workspace 2>&1 | grep -c "warning:")
    echo "Rust warnings: $RUST_WARNINGS"
    
    # TypeScript errors
    TS_ERRORS=$(cd frontend && pnpm typecheck 2>&1 | grep -c "error TS" || echo "0")
    echo "TypeScript errors: $TS_ERRORS"
    
    # ESLint errors
    ESLINT_ERRORS=$(cd frontend && pnpm lint 2>&1 | grep -c "error" || echo "0")
    echo "ESLint errors: $ESLINT_ERRORS"
    
    # Python errors
    PYTHON_ERRORS=$(cd lab_submission_rag && python3 -m py_compile *.py 2>&1 | grep -c "Error" || echo "0")
    echo "Python syntax errors: $PYTHON_ERRORS"
    
    TOTAL=$((RUST_WARNINGS + TS_ERRORS + ESLINT_ERRORS + PYTHON_ERRORS))
    echo ""
    print_status "info" "Total issues: $TOTAL"
}

# Run all checks
run_all() {
    print_status "info" "Running all checks..."
    echo ""
    fix_rust_warnings
    echo ""
    check_python
    echo ""
    check_frontend
    echo ""
    check_migrations
    echo ""
    count_issues
}

# Main loop
while true; do
    show_menu
    read -p "Enter your choice: " choice
    
    case $choice in
        1) fix_rust_warnings ;;
        2) check_python ;;
        3) check_frontend ;;
        4) check_migrations ;;
        5) count_issues ;;
        6) run_all ;;
        0) 
            print_status "info" "Exiting cleanup helper"
            exit 0 
            ;;
        *)
            print_status "error" "Invalid option"
            ;;
    esac
done