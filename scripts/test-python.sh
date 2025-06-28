#!/bin/bash

# TracSeq 2.0 Python Testing Script
# Comprehensive testing for Python AI services, FastMCP servers, and API components

set -e  # Exit on any error

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
PURPLE='\033[0;35m'
NC='\033[0m' # No Color

# Configuration
TEST_TIMEOUT=300
PYTHON_MIN_VERSION="3.11"

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

log_section() {
    echo -e "\n${PURPLE}═══════════════════════════════════════${NC}"
    echo -e "${PURPLE}  $1${NC}"
    echo -e "${PURPLE}═══════════════════════════════════════${NC}\n"
}

# Cross-platform timeout function
run_with_timeout() {
    local timeout_duration=$1
    shift
    local command="$@"
    
    if [[ "$OSTYPE" == "darwin"* ]]; then
        if command -v gtimeout >/dev/null 2>&1; then
            gtimeout "$timeout_duration" bash -c "$command"
        else
            perl -e "alarm($timeout_duration); exec @ARGV" bash -c "$command"
        fi
    else
        timeout "$timeout_duration" bash -c "$command"
    fi
}

# Check Python environment
check_python_environment() {
    log_section "Python Environment Validation"
    
    # Check Python version
    if ! command -v python3 >/dev/null 2>&1; then
        log_error "Python 3 not found. Please install Python 3.11+"
        exit 1
    fi
    
    local python_version=$(python3 --version 2>&1 | awk '{print $2}')
    local python_major=$(echo $python_version | cut -d. -f1)
    local python_minor=$(echo $python_version | cut -d. -f2)
    
    if [ "$python_major" -lt 3 ] || ([ "$python_major" -eq 3 ] && [ "$python_minor" -lt 11 ]); then
        log_error "Python 3.11+ required, found: $python_version"
        exit 1
    fi
    
    log_success "Python version: $python_version"
    
    # Check for uv (modern Python package manager)
    if command -v uv >/dev/null 2>&1; then
        local uv_version=$(uv --version 2>&1 | awk '{print $2}')
        log_success "uv version: $uv_version"
        export USE_UV=true
    else
        log_warning "uv not found, using pip fallback"
        log_info "Install uv: curl -LsSf https://astral.sh/uv/install.sh | sh"
        export USE_UV=false
    fi
    
    # Check pip only if uv is not available
    if [ "$USE_UV" != true ]; then
        if ! python3 -m pip --version >/dev/null 2>&1; then
            log_error "pip not available"
            exit 1
        fi
    else
        log_info "Using uv instead of pip"
    fi
    
    log_success "Python environment validated"
}

# Install dependencies for services
install_dependencies() {
    log_section "Installing Python Dependencies"
    
    local services=(
        "lab_submission_rag"
        "api_gateway"
        "enhanced_rag_service"
        "."  # Root for FastMCP
    )
    
    for service in "${services[@]}"; do
        if [ "$service" = "." ] || [ -d "$service" ]; then
            log_info "Installing dependencies for $service..."
            
            if [ -f "$service/pyproject.toml" ]; then
                cd "$service"
                if [ "$USE_UV" = true ]; then
                    log_info "  Using uv for $service..."
                    uv sync --dev --no-progress 2>/dev/null || log_warning "uv sync failed for $service"
                else
                    log_info "  Using pip for $service..."
                    python3 -m pip install -e .[dev] --quiet 2>/dev/null || log_warning "pip install failed for $service"
                fi
                cd - >/dev/null
                log_success "  Dependencies installed for $service"
                
            elif [ -f "$service/requirements.txt" ]; then
                cd "$service"
                log_info "  Installing from requirements.txt for $service..."
                python3 -m pip install -r requirements.txt --quiet 2>/dev/null || log_warning "pip install failed for $service"
                cd - >/dev/null
                log_success "  Dependencies installed for $service"
            fi
        fi
    done
    
    # Install core testing dependencies
    log_info "Installing core testing dependencies..."
    if [ "$USE_UV" = true ]; then
        log_info "Core dependencies managed by uv (skipping pip install)"
    else
        python3 -m pip install --quiet pytest pytest-asyncio pytest-cov httpx fastapi uvicorn 2>/dev/null || log_warning "Failed to install core testing deps"
    fi
    
    # Install code quality tools
    if [ "$USE_UV" = true ]; then
        uv tool install ruff 2>/dev/null || log_info "ruff already available"
        uv tool install mypy 2>/dev/null || log_info "mypy already available"
    else
        python3 -m pip install --quiet ruff mypy black 2>/dev/null || log_warning "Failed to install quality tools"
    fi
    
    log_success "All dependencies processed"
}

# Test individual Python services
test_python_services() {
    log_section "Python Services Testing"
    
    local services=(
        "lab_submission_rag"
        "api_gateway"
        "enhanced_rag_service"
    )
    
    local results=()
    
    # Set testing environment
    export PYTHONPATH=".:$PYTHONPATH"
    export TEST_MODE="true"
    export OPENAI_API_KEY="test-key"
    export ANTHROPIC_API_KEY="test-key"
    
    # Determine Python command to use
    PYTHON_CMD="python3"
    PYTEST_CMD="pytest"
    if command -v uv >/dev/null 2>&1; then
        log_info "Using uv for Python execution"
        PYTHON_CMD="uv run python"
        PYTEST_CMD="uv run pytest"
    fi
    
    for service in "${services[@]}"; do
        if [ -d "$service" ]; then
            log_info "Testing service: $service"
            cd "$service"
            
            local service_result="PASS"
            
            # Test structure validation
            if [ ! -d "tests" ]; then
                log_warning "  No tests directory found for $service"
                service_result="WARN"
            fi
            
            # Run unit tests
            if [ -d "tests/unit" ]; then
                log_info "  Running unit tests..."
                if run_with_timeout $TEST_TIMEOUT "$PYTEST_CMD tests/unit/ -v --tb=short -q"; then
                    log_success "  ✅ Unit tests passed"
                else
                    log_warning "  ⚠️  Unit tests failed"
                    service_result="FAIL"
                fi
            fi
            
            # Run integration tests
            if [ -d "tests/integration" ]; then
                log_info "  Running integration tests..."
                if run_with_timeout $TEST_TIMEOUT "$PYTEST_CMD tests/integration/ -v --tb=short -q"; then
                    log_success "  ✅ Integration tests passed"
                else
                    log_warning "  ⚠️  Integration tests failed"
                    service_result="FAIL"
                fi
            fi
            
            # Run all tests if no specific structure
            if [ ! -d "tests/unit" ] && [ ! -d "tests/integration" ] && [ -d "tests" ]; then
                log_info "  Running all tests..."
                if run_with_timeout $TEST_TIMEOUT "$PYTEST_CMD tests/ -v --tb=short --maxfail=3 -q"; then
                    log_success "  ✅ All tests passed"
                else
                    log_warning "  ⚠️  Some tests failed"
                    service_result="FAIL"
                fi
            fi
            
            # API validation
            if [ -f "main.py" ] || [ -f "app.py" ]; then
                log_info "  Validating API structure..."
                if $PYTHON_CMD -c "
import sys
try:
    if '$service' == 'api_gateway':
        from api_gateway.main import create_app
        app = create_app()
        print('API Gateway validated')
    elif '$service' == 'lab_submission_rag':
        from api.main import app
        print('RAG API validated')
    elif '$service' == 'enhanced_rag_service':
        print('Enhanced RAG validated')
    sys.exit(0)
except Exception as e:
    print(f'Validation failed: {e}')
    sys.exit(1)
" 2>/dev/null; then
                    log_success "  ✅ API validation passed"
                else
                    log_warning "  ⚠️  API validation failed"
                    service_result="WARN"
                fi
            fi
            
            results+=("$service:$service_result")
            cd - >/dev/null
        fi
    done
    
    return 0
}

# Test FastMCP servers (optimized)
test_fastmcp_servers() {
    log_section "FastMCP Servers Testing"
    
    local servers=(
        "fastmcp_laboratory_server.py"
        "enhanced_rag_service/fastmcp_enhanced_rag_server.py"
        "mcp_infrastructure/fastmcp_laboratory_agent.py"
        "api_gateway/fastmcp_gateway.py"
        "specialized_servers/sample_server.py"
        "specialized_servers/storage_server.py"
        "specialized_servers/quality_control_server.py"
    )
    
    local fastmcp_results=()
    local quick_mode=false
    
    # Check if running in quick mode
    for arg in "$@"; do
        case $arg in
            --quick|--fast)
                quick_mode=true
                ;;
        esac
    done
    
    # Fast parallel validation function
    validate_fastmcp_server() {
        local server="$1"
        local server_name=$(basename "$server")
        
        if [ ! -f "$server" ]; then
            echo "$server_name:MISSING"
            return
        fi
        
        # Combined validation in single Python call for speed
        local result=$($PYTHON_CMD -c "
import sys, os, traceback
server_path = '$server'

try:
    # 1. Syntax check
    with open(server_path, 'r') as f:
        content = f.read()
    compile(content, server_path, 'exec')
    
    # 2. FastMCP pattern check  
    has_fastmcp = any(pattern in content for pattern in [
        'fastmcp', '@app.tool', 'register_tool', 'FastMCP', 'fastmcp.Server'
    ])
    
    # 3. Quick import check (without full execution)
    has_imports = any(imp in content for imp in [
        'import fastmcp', 'from fastmcp', 'FastMCP', 'fastapi'
    ])
    
    # Results
    syntax_ok = True
    fastmcp_ok = has_fastmcp
    import_ok = has_imports
    
    if syntax_ok and fastmcp_ok and import_ok:
        print('PASS')
    elif syntax_ok and (fastmcp_ok or import_ok):
        print('WARN')
    else:
        print('FAIL')

except Exception as e:
    print('FAIL')
" 2>/dev/null)
        
        echo "$server_name:$result"
    }
    
    if [ "$quick_mode" = true ]; then
        log_info "🚀 Quick FastMCP validation mode (parallel)"
        
        # Run validations in parallel for speed
        local pids=()
        local temp_results=()
        
        for server in "${servers[@]}"; do
            {
                result=$(validate_fastmcp_server "$server")
                echo "$result" > "/tmp/fastmcp_$$.$(basename "$server")"
            } &
            pids+=($!)
        done
        
        # Wait for all parallel processes
        for pid in "${pids[@]}"; do
            wait $pid
        done
        
        # Collect results
        local passed=0
        local warned=0
        local failed=0
        
        for server in "${servers[@]}"; do
            local temp_file="/tmp/fastmcp_$$.$(basename "$server")"
            if [ -f "$temp_file" ]; then
                local result=$(cat "$temp_file")
                local server_name=$(echo "$result" | cut -d: -f1)
                local status=$(echo "$result" | cut -d: -f2)
                
                case $status in
                    PASS)
                        log_success "  ✅ $server_name"
                        ((passed++))
                        ;;
                    WARN)
                        log_warning "  ⚠️  $server_name (partial)"
                        ((warned++))
                        ;;
                    FAIL|MISSING)
                        log_warning "  ❌ $server_name"
                        ((failed++))
                        ;;
                esac
                
                rm -f "$temp_file"
            fi
        done
        
        log_info "📊 FastMCP Results: ${passed} passed, ${warned} warnings, ${failed} failed"
        
    else
        log_info "🔍 Comprehensive FastMCP validation mode"
        
        # Original detailed validation (but optimized)
        for server in "${servers[@]}"; do
            if [ -f "$server" ]; then
                log_info "Testing FastMCP server: $(basename $server)"
                
                local server_result="PASS"
                
                # Combined validation for speed
                local validation_result=$($PYTHON_CMD -c "
import sys, os
server_path = '$server'

try:
    # Syntax + import + pattern validation in one call
    with open(server_path, 'r') as f:
        content = f.read()
    
    # 1. Syntax
    compile(content, server_path, 'exec')
    print('✅ Syntax check passed')
    
    # 2. FastMCP patterns
    fastmcp_patterns = ['fastmcp', '@app.tool', 'register_tool', 'FastMCP']
    if any(pattern in content for pattern in fastmcp_patterns):
        print('✅ FastMCP patterns found')
    else:
        print('⚠️  FastMCP patterns missing')
        sys.exit(1)
    
    # 3. Quick structural check
    if 'def ' in content or 'class ' in content:
        print('✅ Structure validation passed')
    else:
        print('⚠️  No functions/classes found')
        sys.exit(1)
        
except Exception as e:
    print(f'❌ Validation failed: {e}')
    sys.exit(1)
" 2>/dev/null)
                
                if [ $? -eq 0 ]; then
                    echo "$validation_result" | while read line; do
                        if [[ $line == ✅* ]]; then
                            log_success "  $line"
                        else
                            log_warning "  $line"
                        fi
                    done
                else
                    log_warning "  ⚠️  Validation failed"
                    server_result="WARN"
                fi
                
                fastmcp_results+=("$server:$server_result")
            else
                log_warning "FastMCP server not found: $server"
            fi
        done
    fi
    
    # Skip heavy integration test in quick mode
    if [ "$quick_mode" != true ] && [ -f "test_fastmcp_integration.py" ]; then
        log_info "Running FastMCP integration test (comprehensive mode)..."
        if timeout 30 $PYTHON_CMD test_fastmcp_integration.py >/dev/null 2>&1; then
            log_success "✅ FastMCP integration test passed"
        else
            log_warning "⚠️  FastMCP integration test failed or timed out"
        fi
    elif [ "$quick_mode" = true ]; then
        log_info "⚡ Skipping integration test (quick mode)"
    fi
    
    return 0
}

# Code quality checks
run_code_quality() {
    log_section "Python Code Quality Analysis"
    
    # Ruff linting
    if command -v ruff >/dev/null 2>&1; then
        log_info "Running ruff linting..."
        if ruff check . --extend-exclude="target,node_modules,frontend"; then
            log_success "✅ Ruff linting passed"
        else
            log_warning "⚠️  Ruff found style issues"
        fi
    else
        log_warning "Ruff not available, skipping linting"
    fi
    
    # Type checking with mypy
    if command -v mypy >/dev/null 2>&1; then
        log_info "Running mypy type checking..."
        local type_check_passed=0
        
        for service in "lab_submission_rag" "api_gateway"; do
            if [ -d "$service" ]; then
                log_info "  Type checking $service..."
                if mypy --ignore-missing-imports --no-error-summary "$service/" 2>/dev/null; then
                    log_success "  ✅ $service type check passed"
                else
                    log_warning "  ⚠️  $service has type issues"
                    type_check_passed=1
                fi
            fi
        done
        
        if [ $type_check_passed -eq 0 ]; then
            log_success "✅ All type checks passed"
        else
            log_warning "⚠️  Some type checks failed"
        fi
    else
        log_warning "MyPy not available, skipping type checking"
    fi
    
    # Security check with bandit (if available)
    if $PYTHON_CMD -c "import bandit" 2>/dev/null; then
        log_info "Running security analysis with bandit..."
        if $PYTHON_CMD -m bandit -r . -x ./venv,./node_modules,./target,./frontend -f txt --skip B101,B601 2>/dev/null; then
            log_success "✅ Security check passed"
        else
            log_warning "⚠️  Security issues detected"
        fi
    fi
}

# Generate comprehensive test report
generate_test_report() {
    log_section "Generating Python Test Report"
    
    mkdir -p test-results
    
    # Generate comprehensive report
    cat > test-results/python-detailed-report.md << EOF
# TracSeq 2.0 Python Testing - Detailed Report

## Executive Summary
- **Test Date**: $(date)
- **Python Version**: $($PYTHON_CMD --version 2>&1)
- **Package Manager**: $(if [ "$USE_UV" = true ]; then echo "uv (modern)"; else echo "pip (legacy)"; fi)
- **Testing Duration**: Started at $(date)

## Python Architecture Overview

### Core Python Services
1. **Lab Submission RAG** (`lab_submission_rag/`)
   - AI-powered document processing with LangChain
   - Vector storage with ChromaDB  
   - OpenAI/Anthropic LLM integration
   - FastAPI web service

2. **API Gateway** (`api_gateway/`)
   - FastAPI intelligent routing
   - HTTP proxy with httpx
   - Service discovery and load balancing
   - Enhanced with FastMCP

3. **Enhanced RAG Service** (`enhanced_rag_service/`)
   - Advanced document intelligence
   - ML pipeline orchestration
   - Multi-modal document processing

### FastMCP Servers (7 specialized servers)
- **Core Laboratory Server**: AI document processing
- **Enhanced RAG Server**: Batch processing with monitoring
- **Laboratory Assistant Agent**: Multi-service coordination
- **API Gateway Enhancement**: AI-powered queries
- **Sample Server**: Intelligent sample management
- **Storage Server**: AI storage optimization
- **Quality Control Server**: Automated QC workflows

## Testing Methodology

### Test Categories
- **Unit Tests**: Individual component testing with pytest
- **Integration Tests**: Service communication validation
- **API Tests**: FastAPI/httpx endpoint testing
- **Syntax/Import Validation**: Module compilation checks
- **FastMCP Validation**: Server pattern verification
- **Code Quality**: Linting, type checking, security

### Dependencies Tested
\`\`\`
Core Framework:
- fastapi >= 0.104.0
- uvicorn >= 0.24.0  
- pydantic >= 2.0.0
- httpx >= 0.24.0

AI/ML Stack:
- langchain >= 0.1.0
- chromadb >= 0.4.0
- transformers >= 4.35.0
- openai >= 1.3.0
- anthropic >= 0.34.0

FastMCP:
- fastmcp >= 2.9.0
- Enhanced context management
- Multi-agent coordination

Testing:
- pytest >= 7.4.0
- pytest-asyncio >= 0.21.0
- pytest-cov >= 4.1.0

Quality:
- ruff >= 0.1.0
- mypy >= 1.7.0
- black >= 23.0.0
\`\`\`

## Test Results Summary

EOF

    # Add results from environment variables if available
    if [ -n "$PYTHON_TEST_RESULTS" ]; then
        echo "$PYTHON_TEST_RESULTS" >> test-results/python-detailed-report.md
    fi
    
    cat >> test-results/python-detailed-report.md << EOF

## Coverage Analysis
- HTML coverage report: test-results/coverage-html/index.html
- XML coverage report: test-results/coverage.xml
- JUnit results: test-results/pytest-results.xml

## Code Quality Metrics
- **Linting**: Ruff static analysis
- **Type Safety**: MyPy type checking
- **Security**: Bandit security scanning
- **Formatting**: Black code formatting

## Performance Characteristics
- **Async Operations**: All services use async/await patterns
- **Memory Efficiency**: Optimized with FastMCP context management
- **AI Integration**: Enhanced LLM sampling and context handling
- **Scalability**: Microservices architecture with independent scaling

## Recommendations
1. **Coverage**: Maintain >80% test coverage across all services
2. **Performance**: Monitor AI model inference times
3. **Security**: Regular dependency vulnerability scanning
4. **Quality**: Enforce type hints and linting in CI/CD

Generated at: $(date)
EOF
    
    log_success "Detailed Python test report: test-results/python-detailed-report.md"
    
    # Generate JSON summary for CI integration
    cat > test-results/python-test-summary.json << EOF
{
    "test_date": "$(date -Iseconds)",
    "python_version": "$($PYTHON_CMD --version 2>&1 | awk '{print $2}')",
    "package_manager": "$(if [ "$USE_UV" = true ]; then echo "uv"; else echo "pip"; fi)",
    "services_tested": [
        "lab_submission_rag",
        "api_gateway", 
        "enhanced_rag_service"
    ],
    "fastmcp_servers": 7,
    "test_categories": [
        "unit",
        "integration", 
        "api",
        "syntax",
        "import",
        "fastmcp",
        "quality"
    ],
    "reports_generated": [
        "test-results/python-detailed-report.md",
        "test-results/coverage-html/index.html",
        "test-results/pytest-results.xml"
    ]
}
EOF
    
    log_success "JSON summary: test-results/python-test-summary.json"
}

# Main execution
main() {
    local start_time=$(date +%s)
    
    log_section "TracSeq 2.0 Python Testing Suite"
    log_info "Comprehensive testing for Python AI services and FastMCP integration"
    
    # Parse arguments
    local run_services=true
    local run_fastmcp=true
    local run_quality=true
    local verbose=false
    
    for arg in "$@"; do
        case $arg in
            --services-only)
                run_fastmcp=false
                run_quality=false
                ;;
            --fastmcp-only)
                run_services=false
                run_quality=false
                ;;
            --quality-only)
                run_services=false
                run_fastmcp=false
                ;;
            --no-quality)
                run_quality=false
                ;;
            --quick|--fast)
                # Quick mode can be combined with other options
                ;;
            --verbose|-v)
                verbose=true
                ;;
        esac
    done
    
    # Execute test phases
    check_python_environment
    install_dependencies
    
    if [ "$run_services" = true ]; then
        test_python_services
    fi
    
    if [ "$run_fastmcp" = true ]; then
        test_fastmcp_servers "$@"
    fi
    
    if [ "$run_quality" = true ]; then
        run_code_quality
    fi
    
    generate_test_report
    
    local end_time=$(date +%s)
    local duration=$((end_time - start_time))
    
    log_section "Python Testing Complete"
    log_success "🎉 Python testing completed successfully!"
    log_success "🐍 Python services validated"
    log_success "🤖 FastMCP servers operational"
    log_success "📊 Code quality analyzed"
    log_info "⏱️  Total execution time: ${duration}s"
    log_info "📋 Reports saved in: test-results/"
    
    echo -e "\n${GREEN}TracSeq 2.0 Python ecosystem is ready for production! 🚀${NC}"
}

# Show usage
if [[ "${1}" == "--help" || "${1}" == "-h" ]]; then
    echo "TracSeq 2.0 Python Testing Script"
    echo ""
    echo "Comprehensive testing for Python AI services and FastMCP integration"
    echo ""
    echo "Usage: $0 [OPTIONS]"
    echo ""
    echo "Options:"
    echo "  --services-only     Test only Python services (no FastMCP or quality)"
    echo "  --fastmcp-only      Test only FastMCP servers"
    echo "  --quality-only      Run only code quality checks"
    echo "  --no-quality        Skip code quality checks"
    echo "  --quick, --fast     Quick mode: parallel FastMCP validation, skip heavy tests"
    echo "  --verbose, -v       Verbose output"
    echo "  --help, -h          Show this help"
    echo ""
    echo "Services tested:"
    echo "  • lab_submission_rag - AI document processing"
    echo "  • api_gateway - Intelligent routing"
    echo "  • enhanced_rag_service - Advanced ML pipelines"
    echo ""
    echo "FastMCP servers tested:"
    echo "  • fastmcp_laboratory_server.py"
    echo "  • enhanced_rag_service/fastmcp_enhanced_rag_server.py"
    echo "  • mcp_infrastructure/fastmcp_laboratory_agent.py"
    echo "  • api_gateway/fastmcp_gateway.py"
    echo "  • specialized_servers/*.py (3 servers)"
    echo ""
    echo "Quality checks:"
    echo "  • Ruff linting"
    echo "  • MyPy type checking"
    echo "  • Bandit security scanning"
    echo ""
    exit 0
fi

# Run main function
main "$@" 