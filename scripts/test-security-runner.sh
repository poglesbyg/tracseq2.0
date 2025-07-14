#!/bin/bash

# TracSeq 2.0 - Security Test Runner
# Comprehensive security testing with detailed reporting

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
PURPLE='\033[0;35m'
NC='\033[0m' # No Color

# Test configuration
TIMESTAMP=$(date +%Y%m%d_%H%M%S)
RESULTS_DIR="security_test_results_${TIMESTAMP}"
LOG_FILE="${RESULTS_DIR}/security_test.log"

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

log_security() {
    echo -e "${PURPLE}[SECURITY]${NC} $1" | tee -a "$LOG_FILE"
}

log_section() {
    echo -e "${BLUE}[SECTION]${NC} $1" | tee -a "$LOG_FILE"
}

print_header() {
    echo "=============================================================="
    echo "TracSeq 2.0 - Security Testing Suite"
    echo "=============================================================="
    echo "Timestamp: $(date)"
    echo "Results Directory: $RESULTS_DIR"
    echo "=============================================================="
    echo
}

print_security_summary() {
    echo
    echo "=============================================================="
    echo "SECURITY TEST SUMMARY"
    echo "=============================================================="
    echo "Timestamp: $(date)"
    echo "Results Directory: $RESULTS_DIR"
    echo "Log File: $LOG_FILE"
    echo "=============================================================="
    
    # Count vulnerabilities and issues
    local total_vulnerabilities=0
    local total_issues=0
    local total_tests=0
    local passed_tests=0
    
    if [ -f "${RESULTS_DIR}/security_summary.txt" ]; then
        total_vulnerabilities=$(grep -c "VULNERABILITIES FOUND:" "${RESULTS_DIR}/security_summary.txt" 2>/dev/null || echo "0")
        total_issues=$(grep -c "SECURITY ISSUES:" "${RESULTS_DIR}/security_summary.txt" 2>/dev/null || echo "0")
        
        # Extract test statistics
        if grep -q "Total Tests:" "${RESULTS_DIR}/security_summary.txt"; then
            total_tests=$(grep "Total Tests:" "${RESULTS_DIR}/security_summary.txt" | awk '{sum += $3} END {print sum}')
            passed_tests=$(grep "Passed Tests:" "${RESULTS_DIR}/security_summary.txt" | awk '{sum += $3} END {print sum}')
        fi
    fi
    
    echo "Security Test Results:"
    echo "  Total Tests: ${total_tests:-0}"
    echo "  Passed Tests: ${passed_tests:-0}"
    echo "  Failed Tests: $((${total_tests:-0} - ${passed_tests:-0}))"
    echo "  Vulnerabilities Found: ${total_vulnerabilities:-0}"
    echo "  Security Issues Found: ${total_issues:-0}"
    
    if [ "${total_vulnerabilities:-0}" -gt 0 ] || [ "${total_issues:-0}" -gt 0 ]; then
        echo "  Security Status: ⚠️  ATTENTION REQUIRED"
    else
        echo "  Security Status: ✅ SECURE"
    fi
    
    echo "=============================================================="
    
    if [ "${total_vulnerabilities:-0}" -gt 0 ] || [ "${total_issues:-0}" -gt 0 ]; then
        log_security "Security vulnerabilities or issues found - review required!"
    else
        log_info "Security testing completed successfully! 🔒"
    fi
}

check_security_prerequisites() {
    log_section "Checking Security Testing Prerequisites"
    
    # Check if services are running
    if ! curl -s http://localhost:8089/health > /dev/null; then
        log_error "API Gateway not running on port 8089"
        return 1
    fi
    
    # Check Python dependencies
    if ! python3 -c "import aiohttp, psutil, jwt" 2>/dev/null; then
        log_error "Python dependencies not installed (aiohttp, psutil, PyJWT)"
        return 1
    fi
    
    # Check security test script
    if [ ! -f "scripts/test-security.py" ]; then
        log_error "Security test script not found"
        return 1
    fi
    
    log_info "All security testing prerequisites met ✅"
    return 0
}

run_authentication_tests() {
    log_section "Running Authentication Security Tests"
    
    log_info "Testing authentication flow, login security, and token validation..."
    
    if python3 scripts/test-security.py --test auth > "${RESULTS_DIR}/auth_security.txt" 2>&1; then
        log_info "Authentication tests completed"
        
        # Check for vulnerabilities
        if grep -q "VULNERABILITIES FOUND:" "${RESULTS_DIR}/auth_security.txt"; then
            local vuln_count=$(grep -c "  - " "${RESULTS_DIR}/auth_security.txt" | head -1)
            log_security "Authentication vulnerabilities found: ${vuln_count}"
        else
            log_info "No authentication vulnerabilities found ✅"
        fi
    else
        log_error "Authentication tests failed to execute"
        return 1
    fi
    
    return 0
}

run_authorization_tests() {
    log_section "Running Authorization and RBAC Tests"
    
    log_info "Testing role-based access control and authorization..."
    
    if python3 scripts/test-security.py --test rbac > "${RESULTS_DIR}/rbac_security.txt" 2>&1; then
        log_info "Authorization tests completed"
        
        # Check for vulnerabilities
        if grep -q "VULNERABILITIES FOUND:" "${RESULTS_DIR}/rbac_security.txt"; then
            local vuln_count=$(grep -c "  - " "${RESULTS_DIR}/rbac_security.txt" | head -1)
            log_security "Authorization vulnerabilities found: ${vuln_count}"
        else
            log_info "No authorization vulnerabilities found ✅"
        fi
    else
        log_error "Authorization tests failed to execute"
        return 1
    fi
    
    return 0
}

run_input_validation_tests() {
    log_section "Running Input Validation Security Tests"
    
    log_info "Testing XSS, SQL injection, and command injection prevention..."
    
    if python3 scripts/test-security.py --test input > "${RESULTS_DIR}/input_security.txt" 2>&1; then
        log_info "Input validation tests completed"
        
        # Check for vulnerabilities
        if grep -q "VULNERABILITIES FOUND:" "${RESULTS_DIR}/input_security.txt"; then
            local vuln_count=$(grep -c "  - " "${RESULTS_DIR}/input_security.txt" | head -1)
            log_security "Input validation vulnerabilities found: ${vuln_count}"
        else
            log_info "No input validation vulnerabilities found ✅"
        fi
    else
        log_error "Input validation tests failed to execute"
        return 1
    fi
    
    return 0
}

run_session_security_tests() {
    log_section "Running Session Security Tests"
    
    log_info "Testing session management, token expiration, and logout security..."
    
    if python3 scripts/test-security.py --test session > "${RESULTS_DIR}/session_security.txt" 2>&1; then
        log_info "Session security tests completed"
        
        # Check for vulnerabilities
        if grep -q "VULNERABILITIES FOUND:" "${RESULTS_DIR}/session_security.txt"; then
            local vuln_count=$(grep -c "  - " "${RESULTS_DIR}/session_security.txt" | head -1)
            log_security "Session security vulnerabilities found: ${vuln_count}"
        else
            log_info "No session security vulnerabilities found ✅"
        fi
    else
        log_error "Session security tests failed to execute"
        return 1
    fi
    
    return 0
}

run_comprehensive_security_tests() {
    log_section "Running Comprehensive Security Test Suite"
    
    log_info "Executing all security tests..."
    
    if python3 scripts/test-security.py --test all > "${RESULTS_DIR}/comprehensive_security.txt" 2>&1; then
        log_info "Comprehensive security tests completed"
        
        # Extract summary information
        if grep -q "Security Testing Complete!" "${RESULTS_DIR}/comprehensive_security.txt"; then
            local total_tests=$(grep "Total Tests:" "${RESULTS_DIR}/comprehensive_security.txt" | awk '{print $3}')
            local passed_tests=$(grep "Passed Tests:" "${RESULTS_DIR}/comprehensive_security.txt" | awk '{print $3}')
            local vulnerabilities=$(grep "Vulnerabilities Found:" "${RESULTS_DIR}/comprehensive_security.txt" | awk '{print $3}')
            local issues=$(grep "Security Issues Found:" "${RESULTS_DIR}/comprehensive_security.txt" | awk '{print $4}')
            
            log_info "Security Test Results: ${passed_tests}/${total_tests} tests passed"
            
            if [ "${vulnerabilities:-0}" -gt 0 ] || [ "${issues:-0}" -gt 0 ]; then
                log_security "Security attention required: ${vulnerabilities} vulnerabilities, ${issues} issues"
            else
                log_info "No critical security vulnerabilities found! ✅"
            fi
        fi
    else
        log_error "Comprehensive security tests failed to execute"
        return 1
    fi
    
    return 0
}

generate_security_summary() {
    log_section "Generating Security Summary Report"
    
    local summary_file="${RESULTS_DIR}/security_summary.txt"
    
    {
        echo "TracSeq 2.0 - Security Test Summary"
        echo "=================================="
        echo "Generated: $(date)"
        echo "Test Suite: Security Testing Framework"
        echo
        
        echo "Test Categories Executed:"
        echo "========================"
        
        if [ -f "${RESULTS_DIR}/auth_security.txt" ]; then
            echo "✅ Authentication Security Tests"
            grep -A 5 "SECURITY TEST RESULT: Authentication Flow" "${RESULTS_DIR}/auth_security.txt" | head -6
            echo
        fi
        
        if [ -f "${RESULTS_DIR}/rbac_security.txt" ]; then
            echo "✅ Authorization and RBAC Tests"
            grep -A 5 "SECURITY TEST RESULT: Authorization and RBAC" "${RESULTS_DIR}/rbac_security.txt" | head -6
            echo
        fi
        
        if [ -f "${RESULTS_DIR}/input_security.txt" ]; then
            echo "✅ Input Validation Security Tests"
            grep -A 5 "SECURITY TEST RESULT: Input Validation Security" "${RESULTS_DIR}/input_security.txt" | head -6
            echo
        fi
        
        if [ -f "${RESULTS_DIR}/session_security.txt" ]; then
            echo "✅ Session Security Tests"
            grep -A 5 "SECURITY TEST RESULT: Session Security" "${RESULTS_DIR}/session_security.txt" | head -6
            echo
        fi
        
        if [ -f "${RESULTS_DIR}/comprehensive_security.txt" ]; then
            echo "Comprehensive Security Test Results:"
            echo "===================================="
            grep -A 10 "Security Testing Complete!" "${RESULTS_DIR}/comprehensive_security.txt" | head -10
            echo
        fi
        
        echo "Security Vulnerabilities Found:"
        echo "=============================="
        for file in "${RESULTS_DIR}"/*_security.txt; do
            if [ -f "$file" ]; then
                if grep -q "VULNERABILITIES FOUND:" "$file"; then
                    echo "From $(basename "$file"):"
                    grep -A 20 "VULNERABILITIES FOUND:" "$file" | grep "  - " | head -10
                    echo
                fi
            fi
        done
        
        echo "Security Issues Found:"
        echo "===================="
        for file in "${RESULTS_DIR}"/*_security.txt; do
            if [ -f "$file" ]; then
                if grep -q "SECURITY ISSUES:" "$file"; then
                    echo "From $(basename "$file"):"
                    grep -A 20 "SECURITY ISSUES:" "$file" | grep "  - " | head -10
                    echo
                fi
            fi
        done
        
        echo "Security Recommendations:"
        echo "======================="
        for file in "${RESULTS_DIR}"/*_security.txt; do
            if [ -f "$file" ]; then
                if grep -q "RECOMMENDATIONS:" "$file"; then
                    echo "From $(basename "$file"):"
                    grep -A 20 "RECOMMENDATIONS:" "$file" | grep "  - " | head -10
                    echo
                fi
            fi
        done
        
    } > "$summary_file"
    
    log_info "Security summary report generated: $summary_file"
}

generate_security_html_report() {
    log_section "Generating Security HTML Report"
    
    local html_file="${RESULTS_DIR}/security_report.html"
    
    cat > "$html_file" << EOF
<!DOCTYPE html>
<html>
<head>
    <title>TracSeq 2.0 - Security Test Report</title>
    <style>
        body { font-family: Arial, sans-serif; margin: 20px; }
        .header { background: #f0f0f0; padding: 20px; border-radius: 5px; margin-bottom: 20px; }
        .section { margin: 20px 0; padding: 15px; border: 1px solid #ddd; border-radius: 5px; }
        .vulnerability { background: #ffe6e6; padding: 10px; margin: 10px 0; border-left: 4px solid #ff0000; }
        .issue { background: #fff3cd; padding: 10px; margin: 10px 0; border-left: 4px solid #ffc107; }
        .recommendation { background: #e6f3ff; padding: 10px; margin: 10px 0; border-left: 4px solid #007bff; }
        .pass { color: green; font-weight: bold; }
        .fail { color: red; font-weight: bold; }
        pre { background: #f5f5f5; padding: 10px; border-radius: 3px; overflow-x: auto; }
        .summary { background: #f8f9fa; padding: 15px; border-radius: 5px; margin: 20px 0; }
        .security-status { font-size: 1.2em; font-weight: bold; }
        .secure { color: green; }
        .insecure { color: red; }
    </style>
</head>
<body>
    <div class="header">
        <h1>TracSeq 2.0 - Security Test Report</h1>
        <p><strong>Generated:</strong> $(date)</p>
        <p><strong>Results Directory:</strong> $RESULTS_DIR</p>
    </div>
    
    <div class="summary">
        <h2>Security Test Summary</h2>
EOF

    # Add test results summary
    if [ -f "${RESULTS_DIR}/security_summary.txt" ]; then
        local total_vulnerabilities=$(grep -c "VULNERABILITIES FOUND:" "${RESULTS_DIR}/security_summary.txt" 2>/dev/null || echo "0")
        local total_issues=$(grep -c "SECURITY ISSUES:" "${RESULTS_DIR}/security_summary.txt" 2>/dev/null || echo "0")
        
        if [ "${total_vulnerabilities:-0}" -gt 0 ] || [ "${total_issues:-0}" -gt 0 ]; then
            cat >> "$html_file" << EOF
        <div class="security-status insecure">⚠️ SECURITY ATTENTION REQUIRED</div>
        <ul>
            <li>Vulnerabilities Found: ${total_vulnerabilities}</li>
            <li>Security Issues Found: ${total_issues}</li>
        </ul>
EOF
        else
            cat >> "$html_file" << EOF
        <div class="security-status secure">✅ SECURE</div>
        <p>No critical security vulnerabilities found.</p>
EOF
        fi
    fi
    
    cat >> "$html_file" << EOF
    </div>
EOF

    # Add individual test results
    for test_type in auth rbac input session; do
        local test_file="${RESULTS_DIR}/${test_type}_security.txt"
        if [ -f "$test_file" ]; then
            local test_name=""
            case $test_type in
                auth) test_name="Authentication Security" ;;
                rbac) test_name="Authorization and RBAC" ;;
                input) test_name="Input Validation Security" ;;
                session) test_name="Session Security" ;;
            esac
            
            cat >> "$html_file" << EOF
    <div class="section">
        <h2>$test_name</h2>
        <pre>$(cat "$test_file")</pre>
    </div>
EOF
        fi
    done
    
    # Add comprehensive results if available
    if [ -f "${RESULTS_DIR}/comprehensive_security.txt" ]; then
        cat >> "$html_file" << EOF
    <div class="section">
        <h2>Comprehensive Security Test Results</h2>
        <pre>$(cat "${RESULTS_DIR}/comprehensive_security.txt")</pre>
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

    log_info "Security HTML report generated: $html_file"
}

cleanup() {
    log_info "Cleaning up temporary files..."
    # No specific cleanup needed for security tests
}

main() {
    # Parse command line arguments
    local run_auth=true
    local run_rbac=true
    local run_input=true
    local run_session=true
    local run_comprehensive=true
    local generate_html=true
    
    while [[ $# -gt 0 ]]; do
        case $1 in
            --auth-only)
                run_rbac=false
                run_input=false
                run_session=false
                run_comprehensive=false
                shift
                ;;
            --rbac-only)
                run_auth=false
                run_input=false
                run_session=false
                run_comprehensive=false
                shift
                ;;
            --input-only)
                run_auth=false
                run_rbac=false
                run_session=false
                run_comprehensive=false
                shift
                ;;
            --session-only)
                run_auth=false
                run_rbac=false
                run_input=false
                run_comprehensive=false
                shift
                ;;
            --comprehensive-only)
                run_auth=false
                run_rbac=false
                run_input=false
                run_session=false
                shift
                ;;
            --no-html)
                generate_html=false
                shift
                ;;
            --help)
                echo "Usage: $0 [OPTIONS]"
                echo "Options:"
                echo "  --auth-only           Run only authentication tests"
                echo "  --rbac-only           Run only authorization/RBAC tests"
                echo "  --input-only          Run only input validation tests"
                echo "  --session-only        Run only session security tests"
                echo "  --comprehensive-only  Run only comprehensive security tests"
                echo "  --no-html             Skip HTML report generation"
                echo "  --help                Show this help message"
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
    echo "TracSeq 2.0 Security Test Log" > "$LOG_FILE"
    echo "Started: $(date)" >> "$LOG_FILE"
    echo "===========================================" >> "$LOG_FILE"
    
    print_header
    
    # Check prerequisites
    if ! check_security_prerequisites; then
        log_error "Security testing prerequisites not met. Exiting."
        exit 1
    fi
    
    # Run security tests based on options
    local exit_code=0
    
    if [ "$run_auth" = true ]; then
        if ! run_authentication_tests; then
            exit_code=1
        fi
    fi
    
    if [ "$run_rbac" = true ]; then
        if ! run_authorization_tests; then
            exit_code=1
        fi
    fi
    
    if [ "$run_input" = true ]; then
        if ! run_input_validation_tests; then
            exit_code=1
        fi
    fi
    
    if [ "$run_session" = true ]; then
        if ! run_session_security_tests; then
            exit_code=1
        fi
    fi
    
    if [ "$run_comprehensive" = true ]; then
        if ! run_comprehensive_security_tests; then
            exit_code=1
        fi
    fi
    
    # Generate reports
    generate_security_summary
    
    if [ "$generate_html" = true ]; then
        generate_security_html_report
    fi
    
    # Cleanup
    cleanup
    
    # Print summary
    print_security_summary
    
    if [ $exit_code -eq 0 ]; then
        log_info "Security testing completed successfully! 🔒"
    else
        log_warn "Some security tests failed. Check the results for details."
    fi
    
    exit $exit_code
}

# Run main function with all arguments
main "$@" 