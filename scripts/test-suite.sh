#!/bin/bash

# TracSeq 2.0 - Comprehensive Test Suite Runner
# Executes all testing types: basic E2E, performance, load, and integration tests

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Test configuration
TIMESTAMP=$(date +%Y%m%d_%H%M%S)
RESULTS_DIR="test_results_${TIMESTAMP}"
LOG_FILE="${RESULTS_DIR}/test_suite.log"

# Helper functions
log_info() {
    echo -e "${GREEN}[INFO]${NC} $1" | tee -a "$LOG_FILE"
}

log_warn() {
    echo -e "${YELLOW}[WARN]${NC} $1" | tee -a "$LOG_FILE"
}

log_error() {
    echo -e "${RED}[ERROR]${NC} $1" | tee -a "$LOG_FILE"
}

log_section() {
    echo -e "${BLUE}[SECTION]${NC} $1" | tee -a "$LOG_FILE"
}

print_header() {
    echo "=============================================================="
    echo "TracSeq 2.0 - Comprehensive Test Suite"
    echo "=============================================================="
    echo "Timestamp: $(date)"
    echo "Results Directory: $RESULTS_DIR"
    echo "=============================================================="
    echo
}

print_summary() {
    echo
    echo "=============================================================="
    echo "TEST SUITE SUMMARY"
    echo "=============================================================="
    echo "Timestamp: $(date)"
    echo "Results Directory: $RESULTS_DIR"
    echo "Log File: $LOG_FILE"
    echo "=============================================================="
    
    # Count test results
    local basic_tests=0
    local performance_tests=0
    local integration_tests=0
    local total_tests=0
    
    if [ -f "${RESULTS_DIR}/basic_e2e_results.txt" ]; then
        basic_tests=$(grep -c "PASS\|FAIL" "${RESULTS_DIR}/basic_e2e_results.txt" 2>/dev/null || echo "0")
    fi
    
    if [ -f "${RESULTS_DIR}/performance_results.json" ]; then
        performance_tests=$(python3 -c "import json; data=json.load(open('${RESULTS_DIR}/performance_results.json')); print(len(data))" 2>/dev/null || echo "0")
    fi
    
    if [ -f "${RESULTS_DIR}/integration_results.json" ]; then
        integration_tests=$(python3 -c "import json; data=json.load(open('${RESULTS_DIR}/integration_results.json')); print(len(data))" 2>/dev/null || echo "0")
    fi
    
    total_tests=$((basic_tests + performance_tests + integration_tests))
    
    echo "Test Results:"
    echo "  Basic E2E Tests: $basic_tests"
    echo "  Performance Tests: $performance_tests"
    echo "  Integration Tests: $integration_tests"
    echo "  Total Tests: $total_tests"
    echo "=============================================================="
    
    log_info "Test suite completed successfully! 🎉"
}

check_prerequisites() {
    log_section "Checking Prerequisites"
    
    # Check if services are running
    if ! curl -s http://localhost:8089/health > /dev/null; then
        log_error "API Gateway not running on port 8089"
        return 1
    fi
    
    if ! curl -s http://localhost:8085/health > /dev/null; then
        log_error "Frontend Proxy not running on port 8085"
        return 1
    fi
    
    # Check individual services
    local services=("8080" "8081" "8082" "8083")
    for port in "${services[@]}"; do
        if ! curl -s "http://localhost:${port}/health" > /dev/null; then
            log_error "Service not running on port ${port}"
            return 1
        fi
    done
    
    # Check Python dependencies
    if ! python3 -c "import aiohttp, psutil" 2>/dev/null; then
        log_error "Python dependencies not installed (aiohttp, psutil)"
        return 1
    fi
    
    # Check jq for JSON processing
    if ! command -v jq &> /dev/null; then
        log_warn "jq not found - some tests may not work properly"
    fi
    
    log_info "All prerequisites met ✅"
    return 0
}

run_basic_e2e_tests() {
    log_section "Running Basic End-to-End Tests"
    
    if [ ! -f "scripts/test-e2e-basic.sh" ]; then
        log_error "Basic E2E test script not found"
        return 1
    fi
    
    log_info "Executing basic E2E tests..."
    
    # Run basic E2E tests and capture output
    if ./scripts/test-e2e-basic.sh > "${RESULTS_DIR}/basic_e2e_results.txt" 2>&1; then
        local passed=$(grep -c "PASS" "${RESULTS_DIR}/basic_e2e_results.txt" 2>/dev/null || echo "0")
        local failed=$(grep -c "FAIL" "${RESULTS_DIR}/basic_e2e_results.txt" 2>/dev/null || echo "0")
        local total=$((passed + failed))
        
        log_info "Basic E2E Tests: ${passed}/${total} passed"
        
        if [ $failed -eq 0 ]; then
            log_info "All basic E2E tests passed ✅"
        else
            log_warn "Some basic E2E tests failed ⚠️"
        fi
    else
        log_error "Basic E2E tests failed to execute"
        return 1
    fi
    
    return 0
}

run_performance_tests() {
    log_section "Running Performance Tests"
    
    if [ ! -f "scripts/test-performance.py" ]; then
        log_error "Performance test script not found"
        return 1
    fi
    
    log_info "Executing performance tests..."
    
    # Run health check performance test
    log_info "Running health check performance test..."
    python3 scripts/test-performance.py --test health > "${RESULTS_DIR}/performance_health.txt" 2>&1
    
    # Run API endpoint performance test
    log_info "Running API endpoint performance test..."
    python3 scripts/test-performance.py --test api > "${RESULTS_DIR}/performance_api.txt" 2>&1
    
    # Run load tests with different user counts
    log_info "Running load tests..."
    for users in 1 5 10; do
        log_info "Load test with ${users} users..."
        python3 scripts/test-performance.py --test load --users $users --duration 15 > "${RESULTS_DIR}/performance_load_${users}.txt" 2>&1
    done
    
    # Run integration performance test
    log_info "Running integration performance test..."
    python3 scripts/test-performance.py --test integration > "${RESULTS_DIR}/performance_integration.txt" 2>&1
    
    # Generate combined performance results
    log_info "Generating combined performance results..."
    {
        echo "Performance Test Results Summary"
        echo "================================"
        echo "Timestamp: $(date)"
        echo
        
        for file in "${RESULTS_DIR}"/performance_*.txt; do
            if [ -f "$file" ]; then
                echo "--- $(basename "$file") ---"
                if grep -q "TEST RESULT:" "$file"; then
                    grep -A 20 "TEST RESULT:" "$file" | head -20
                elif grep -q "LOAD TEST RESULT:" "$file"; then
                    grep -A 15 "LOAD TEST RESULT:" "$file" | head -15
                fi
                echo
            fi
        done
    } > "${RESULTS_DIR}/performance_summary.txt"
    
    log_info "Performance tests completed ✅"
    return 0
}

run_integration_tests() {
    log_section "Running Integration Tests"
    
    if [ ! -f "scripts/test-integration.py" ]; then
        log_error "Integration test script not found"
        return 1
    fi
    
    log_info "Executing integration tests..."
    
    # Run workflow integration test
    log_info "Running workflow integration test..."
    python3 scripts/test-integration.py --test workflow > "${RESULTS_DIR}/integration_workflow.txt" 2>&1
    
    # Run service communication test
    log_info "Running service communication test..."
    python3 scripts/test-integration.py --test communication > "${RESULTS_DIR}/integration_communication.txt" 2>&1
    
    # Run data consistency test
    log_info "Running data consistency test..."
    python3 scripts/test-integration.py --test consistency > "${RESULTS_DIR}/integration_consistency.txt" 2>&1
    
    # Run error handling test
    log_info "Running error handling test..."
    python3 scripts/test-integration.py --test errors > "${RESULTS_DIR}/integration_errors.txt" 2>&1
    
    # Generate combined integration results
    log_info "Generating combined integration results..."
    {
        echo "Integration Test Results Summary"
        echo "================================"
        echo "Timestamp: $(date)"
        echo
        
        for file in "${RESULTS_DIR}"/integration_*.txt; do
            if [ -f "$file" ]; then
                echo "--- $(basename "$file") ---"
                if grep -q "INTEGRATION TEST RESULT:" "$file"; then
                    grep -A 10 "INTEGRATION TEST RESULT:" "$file" | head -10
                fi
                echo
            fi
        done
    } > "${RESULTS_DIR}/integration_summary.txt"
    
    log_info "Integration tests completed ✅"
    return 0
}

run_stress_tests() {
    log_section "Running Stress Tests"
    
    log_info "Running stress tests to find breaking points..."
    
    # Run stress test
    python3 scripts/test-performance.py --test stress > "${RESULTS_DIR}/stress_test.txt" 2>&1
    
    log_info "Stress tests completed ✅"
    return 0
}

generate_html_report() {
    log_section "Generating HTML Report"
    
    local html_file="${RESULTS_DIR}/test_report.html"
    
    cat > "$html_file" << EOF
<!DOCTYPE html>
<html>
<head>
    <title>TracSeq 2.0 - Test Suite Report</title>
    <style>
        body { font-family: Arial, sans-serif; margin: 20px; }
        .header { background: #f0f8ff; padding: 20px; border-radius: 5px; margin-bottom: 20px; }
        .section { margin: 20px 0; padding: 15px; border: 1px solid #ddd; border-radius: 5px; }
        .pass { color: green; }
        .fail { color: red; }
        .warn { color: orange; }
        pre { background: #f5f5f5; padding: 10px; border-radius: 3px; overflow-x: auto; }
        .summary { background: #e8f5e8; padding: 15px; border-radius: 5px; }
    </style>
</head>
<body>
    <div class="header">
        <h1>TracSeq 2.0 - Test Suite Report</h1>
        <p><strong>Generated:</strong> $(date)</p>
        <p><strong>Results Directory:</strong> $RESULTS_DIR</p>
    </div>
    
    <div class="summary">
        <h2>Test Summary</h2>
        <ul>
            <li>Basic E2E Tests: $([ -f "${RESULTS_DIR}/basic_e2e_results.txt" ] && echo "✅ Completed" || echo "❌ Not Run")</li>
            <li>Performance Tests: $([ -f "${RESULTS_DIR}/performance_summary.txt" ] && echo "✅ Completed" || echo "❌ Not Run")</li>
            <li>Integration Tests: $([ -f "${RESULTS_DIR}/integration_summary.txt" ] && echo "✅ Completed" || echo "❌ Not Run")</li>
            <li>Stress Tests: $([ -f "${RESULTS_DIR}/stress_test.txt" ] && echo "✅ Completed" || echo "❌ Not Run")</li>
        </ul>
    </div>
EOF

    # Add Basic E2E Results
    if [ -f "${RESULTS_DIR}/basic_e2e_results.txt" ]; then
        cat >> "$html_file" << EOF
    <div class="section">
        <h2>Basic End-to-End Tests</h2>
        <pre>$(cat "${RESULTS_DIR}/basic_e2e_results.txt")</pre>
    </div>
EOF
    fi
    
    # Add Performance Results
    if [ -f "${RESULTS_DIR}/performance_summary.txt" ]; then
        cat >> "$html_file" << EOF
    <div class="section">
        <h2>Performance Tests</h2>
        <pre>$(cat "${RESULTS_DIR}/performance_summary.txt")</pre>
    </div>
EOF
    fi
    
    # Add Integration Results
    if [ -f "${RESULTS_DIR}/integration_summary.txt" ]; then
        cat >> "$html_file" << EOF
    <div class="section">
        <h2>Integration Tests</h2>
        <pre>$(cat "${RESULTS_DIR}/integration_summary.txt")</pre>
    </div>
EOF
    fi
    
    # Add Stress Test Results
    if [ -f "${RESULTS_DIR}/stress_test.txt" ]; then
        cat >> "$html_file" << EOF
    <div class="section">
        <h2>Stress Tests</h2>
        <pre>$(cat "${RESULTS_DIR}/stress_test.txt")</pre>
    </div>
EOF
    fi
    
    cat >> "$html_file" << EOF
    <div class="section">
        <h2>Log File</h2>
        <pre>$(cat "$LOG_FILE")</pre>
    </div>
</body>
</html>
EOF

    log_info "HTML report generated: $html_file"
}

cleanup() {
    log_info "Cleaning up temporary files..."
    rm -f get-pip.py
}

main() {
    # Parse command line arguments
    local run_basic=true
    local run_performance=true
    local run_integration=true
    local run_stress=true
    local generate_html=true
    
    while [[ $# -gt 0 ]]; do
        case $1 in
            --basic-only)
                run_performance=false
                run_integration=false
                run_stress=false
                shift
                ;;
            --performance-only)
                run_basic=false
                run_integration=false
                run_stress=false
                shift
                ;;
            --integration-only)
                run_basic=false
                run_performance=false
                run_stress=false
                shift
                ;;
            --no-html)
                generate_html=false
                shift
                ;;
            --help)
                echo "Usage: $0 [OPTIONS]"
                echo "Options:"
                echo "  --basic-only      Run only basic E2E tests"
                echo "  --performance-only Run only performance tests"
                echo "  --integration-only Run only integration tests"
                echo "  --no-html         Skip HTML report generation"
                echo "  --help            Show this help message"
                exit 0
                ;;
            *)
                log_error "Unknown option: $1"
                exit 1
                ;;
        esac
    done
    
    # Create results directory
    mkdir -p "$RESULTS_DIR"
    
    # Initialize log file
    echo "TracSeq 2.0 Test Suite Log" > "$LOG_FILE"
    echo "Started: $(date)" >> "$LOG_FILE"
    echo "===========================================" >> "$LOG_FILE"
    
    print_header
    
    # Check prerequisites
    if ! check_prerequisites; then
        log_error "Prerequisites not met. Exiting."
        exit 1
    fi
    
    # Run tests based on options
    local exit_code=0
    
    if [ "$run_basic" = true ]; then
        if ! run_basic_e2e_tests; then
            exit_code=1
        fi
    fi
    
    if [ "$run_performance" = true ]; then
        if ! run_performance_tests; then
            exit_code=1
        fi
    fi
    
    if [ "$run_integration" = true ]; then
        if ! run_integration_tests; then
            exit_code=1
        fi
    fi
    
    if [ "$run_stress" = true ]; then
        if ! run_stress_tests; then
            exit_code=1
        fi
    fi
    
    # Generate HTML report
    if [ "$generate_html" = true ]; then
        generate_html_report
    fi
    
    # Cleanup
    cleanup
    
    # Print summary
    print_summary
    
    if [ $exit_code -eq 0 ]; then
        log_info "All tests completed successfully! 🎉"
    else
        log_warn "Some tests failed. Check the results for details."
    fi
    
    exit $exit_code
}

# Run main function with all arguments
main "$@" 