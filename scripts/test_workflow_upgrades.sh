#!/bin/bash

# Test script for validating upgraded GitHub Actions workflows
# Tests workflow syntax, configuration, and functionality

# set -e  # Commented out to see all test results

echo "🚀 Testing GitHub Actions Workflow Upgrades"
echo "=============================================="

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Test counters
TESTS_PASSED=0
TESTS_FAILED=0

# Function to print test results
print_result() {
    if [ $1 -eq 0 ]; then
        echo -e "${GREEN}✅ $2${NC}"
        ((TESTS_PASSED++))
    else
        echo -e "${RED}❌ $2${NC}"
        ((TESTS_FAILED++))
    fi
}

echo -e "${BLUE}📋 Step 1: Validating Workflow Files Exist${NC}"

# Check if workflow files exist
test -f .github/workflows/ci.yml
print_result $? "CI workflow file exists"

test -f .github/workflows/deploy.yml
print_result $? "Deployment workflow file exists"

test -f .github/workflows/security.yml
print_result $? "Security workflow file exists"

test -f .github/workflows/performance.yml
print_result $? "Performance workflow file exists"

echo -e "${BLUE}📋 Step 2: Validating YAML Syntax${NC}"

# Install yq if not present (for YAML validation)
if ! command -v yq &> /dev/null; then
    echo -e "${YELLOW}Installing yq for YAML validation...${NC}"
    if [[ "$OSTYPE" == "linux-gnu"* ]]; then
        sudo wget -qO /usr/local/bin/yq https://github.com/mikefarah/yq/releases/latest/download/yq_linux_amd64
        sudo chmod +x /usr/local/bin/yq
    elif [[ "$OSTYPE" == "darwin"* ]]; then
        brew install yq
    fi
fi

# Validate YAML syntax
yq eval '.name' .github/workflows/ci.yml > /dev/null 2>&1
print_result $? "CI workflow YAML syntax is valid"

yq eval '.name' .github/workflows/deploy.yml > /dev/null 2>&1
print_result $? "Deploy workflow YAML syntax is valid"

yq eval '.name' .github/workflows/security.yml > /dev/null 2>&1
print_result $? "Security workflow YAML syntax is valid"

yq eval '.name' .github/workflows/performance.yml > /dev/null 2>&1
print_result $? "Performance workflow YAML syntax is valid"

echo -e "${BLUE}📋 Step 3: Checking Enhanced Features${NC}"

# Check for enhanced CI features
grep -q "reports" .github/workflows/ci.yml
print_result $? "CI workflow includes reports module testing"

grep -q "multi-platform" .github/workflows/ci.yml
print_result $? "CI workflow has multi-platform Docker builds"

grep -q "cargo-tarpaulin" .github/workflows/ci.yml
print_result $? "CI workflow includes code coverage"

grep -q "criterion" .github/workflows/ci.yml
print_result $? "CI workflow includes performance benchmarking"

# Check for enhanced deployment features
grep -q "reports-only" .github/workflows/deploy.yml
print_result $? "Deploy workflow supports reports-only variant"

grep -q "blue-green" .github/workflows/deploy.yml
print_result $? "Deploy workflow includes blue-green deployment"

grep -q "rollback" .github/workflows/deploy.yml
print_result $? "Deploy workflow includes rollback capability"

grep -q "microservices" .github/workflows/deploy.yml
print_result $? "Deploy workflow supports microservices deployment"

# Check for enhanced security features
grep -q "sql-security-analysis" .github/workflows/security.yml
print_result $? "Security workflow includes SQL security analysis"

grep -q "penetration-testing" .github/workflows/security.yml
print_result $? "Security workflow includes penetration testing"

grep -q "cargo-geiger" .github/workflows/security.yml
print_result $? "Security workflow includes unsafe code analysis"

grep -q "trufflehog" .github/workflows/security.yml
print_result $? "Security workflow includes advanced secret scanning"

echo -e "${BLUE}📋 Step 4: Validating SQL Reports Security${NC}"

# Check if reports module exists
if [ -f "src/handlers/reports/mod.rs" ]; then
    print_result 0 "Reports module exists for security testing"
    
    # Check for security features in reports module
    if grep -q "is_safe_query\|validate\|sanitize" src/handlers/reports/mod.rs; then
        print_result 0 "Reports module has security validation functions"
    else
        print_result 1 "Reports module missing security validation functions"
    fi
else
    print_result 1 "Reports module not found"
fi

echo -e "${BLUE}📋 Step 5: Checking Workflow Triggers${NC}"

# Check for proper workflow triggers
grep -q "workflow_dispatch" .github/workflows/ci.yml
print_result $? "CI workflow has manual trigger capability"

grep -q "schedule" .github/workflows/security.yml
print_result $? "Security workflow has scheduled scanning"

grep -q "pull_request" .github/workflows/deploy.yml
print_result $? "Deploy workflow can be triggered by PR labels"

echo -e "${BLUE}📋 Step 6: Testing Component Matrix Configuration${NC}"

# Check for component matrix in CI
if grep -A 5 "matrix:" .github/workflows/ci.yml | grep -q "reports"; then
    print_result 0 "CI workflow includes reports in component matrix"
else
    print_result 1 "CI workflow missing reports in component matrix"
fi

# Check for service matrix in deployment
if grep -A 5 "matrix:" .github/workflows/deploy.yml | grep -q "reports"; then
    print_result 0 "Deploy workflow includes reports in service matrix"
else
    print_result 1 "Deploy workflow missing reports in service matrix"
fi

echo -e "${BLUE}📋 Step 7: Validating Documentation${NC}"

# Check if documentation exists
test -f docs/WORKFLOW_UPGRADES_SUMMARY.md
print_result $? "Workflow upgrades documentation exists"

# Check if documentation mentions key features
if [ -f docs/WORKFLOW_UPGRADES_SUMMARY.md ]; then
    grep -qi "sql reports\|reports.*security" docs/WORKFLOW_UPGRADES_SUMMARY.md
    print_result $? "Documentation mentions SQL Reports security"
    
    grep -qi "multi-platform\|platforms.*amd64.*arm64" docs/WORKFLOW_UPGRADES_SUMMARY.md
    print_result $? "Documentation mentions multi-platform builds"
    
    grep -qi "penetration.*testing\|security.*testing" docs/WORKFLOW_UPGRADES_SUMMARY.md
    print_result $? "Documentation mentions penetration testing"
fi

echo -e "${BLUE}📋 Step 8: Checking Cache Configuration${NC}"

# Check for proper cache configuration
grep -A 10 "Cache cargo dependencies" .github/workflows/ci.yml | grep -q "registry/index/"
print_result $? "CI workflow has optimized cargo cache"

grep -A 10 "Cache cargo dependencies" .github/workflows/security.yml | grep -q "registry/index/"
print_result $? "Security workflow has optimized cargo cache"

echo -e "${BLUE}📋 Step 9: Testing Action Versions${NC}"

# Check for latest action versions
grep -q "actions/checkout@v4" .github/workflows/ci.yml
print_result $? "CI workflow uses latest checkout action"

grep -q "docker/build-push-action@v5" .github/workflows/deploy.yml
print_result $? "Deploy workflow uses latest Docker build action"

grep -q "github/codeql-action/upload-sarif@v3" .github/workflows/security.yml
print_result $? "Security workflow uses latest CodeQL action"

echo -e "${BLUE}📋 Step 10: Simulating Workflow Execution${NC}"

# Create a simple test to simulate workflow steps
echo "Testing workflow step simulation..."

# Simulate component testing
for component in "handlers" "storage" "reports"; do
    if [ -d "src/${component}" ] || [ -f "src/handlers/${component}/mod.rs" ]; then
        print_result 0 "Component ${component} exists for testing"
    else
        print_result 1 "Component ${component} missing"
    fi
done

# Simulate security checks
if command -v cargo &> /dev/null; then
    # Test if we can run basic cargo commands
    cargo check --quiet > /dev/null 2>&1
    print_result $? "Cargo check passes (simulating CI)"
else
    print_result 1 "Cargo not available for testing"
fi

echo ""
echo "🎯 Test Summary"
echo "==============="
echo -e "${GREEN}Tests Passed: ${TESTS_PASSED}${NC}"
echo -e "${RED}Tests Failed: ${TESTS_FAILED}${NC}"

if [ $TESTS_FAILED -eq 0 ]; then
    echo -e "${GREEN}🎉 All workflow upgrade tests passed!${NC}"
    echo -e "${GREEN}✅ Your GitHub Actions workflows are ready for production${NC}"
    exit 0
else
    echo -e "${RED}❌ Some tests failed. Please review the workflow configurations.${NC}"
    exit 1
fi 
