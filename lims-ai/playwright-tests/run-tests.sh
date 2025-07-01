#!/bin/bash

# TracSeq 2.0 Playwright Test Runner
# This script helps run Playwright tests for Python services

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# Default values
TEST_TYPE="all"
HEADED=false
DEBUG=false
UI_MODE=false
SERVICE=""

# Function to display usage
usage() {
    echo "Usage: $0 [OPTIONS]"
    echo ""
    echo "Options:"
    echo "  -t, --type TYPE        Test type: all, smoke, integration, api, websocket (default: all)"
    echo "  -s, --service SERVICE  Specific service: mcp-dashboard, enhanced-rag-service, lab-submission-rag, ml-platform"
    echo "  -h, --headed           Run tests in headed mode (show browser)"
    echo "  -d, --debug            Run tests in debug mode"
    echo "  -u, --ui               Run tests in UI mode"
    echo "  --help                 Display this help message"
    echo ""
    echo "Examples:"
    echo "  $0                     # Run all tests"
    echo "  $0 -t smoke            # Run smoke tests only"
    echo "  $0 -s mcp-dashboard    # Run MCP dashboard tests only"
    echo "  $0 -h -d               # Run all tests in headed debug mode"
}

# Parse arguments
while [[ $# -gt 0 ]]; do
    case $1 in
        -t|--type)
            TEST_TYPE="$2"
            shift 2
            ;;
        -s|--service)
            SERVICE="$2"
            shift 2
            ;;
        -h|--headed)
            HEADED=true
            shift
            ;;
        -d|--debug)
            DEBUG=true
            shift
            ;;
        -u|--ui)
            UI_MODE=true
            shift
            ;;
        --help)
            usage
            exit 0
            ;;
        *)
            echo -e "${RED}Unknown option: $1${NC}"
            usage
            exit 1
            ;;
    esac
done

# Check if npm is installed
if ! command -v npm &> /dev/null; then
    echo -e "${RED}npm is not installed. Please install Node.js and npm first.${NC}"
    exit 1
fi

# Check if we're in the right directory
if [ ! -f "package.json" ]; then
    echo -e "${YELLOW}Not in playwright-tests directory. Changing to correct directory...${NC}"
    cd "$(dirname "$0")"
fi

# Install dependencies if needed
if [ ! -d "node_modules" ]; then
    echo -e "${YELLOW}Installing dependencies...${NC}"
    npm install
fi

# Install browsers if needed
if [ ! -d "$HOME/.cache/ms-playwright" ]; then
    echo -e "${YELLOW}Installing Playwright browsers...${NC}"
    npx playwright install --with-deps
fi

# Start Python services if not running
check_service() {
    local port=$1
    nc -z localhost "$port" 2>/dev/null
}

echo -e "${GREEN}Checking Python services...${NC}"

SERVICES_TO_START=()

if ! check_service 7890; then
    SERVICES_TO_START+=("mcp-dashboard")
fi

if ! check_service 8100; then
    SERVICES_TO_START+=("enhanced-rag")
fi

if ! check_service 8000; then
    SERVICES_TO_START+=("lab-submission-rag")
fi

if ! check_service 9500; then
    SERVICES_TO_START+=("mcp-proxy")
fi

if [ ${#SERVICES_TO_START[@]} -gt 0 ]; then
    echo -e "${YELLOW}The following services are not running: ${SERVICES_TO_START[*]}${NC}"
    echo -e "${YELLOW}Please start them using:${NC}"
    echo "cd ../.. && ./docker/mcp/start-mcp.sh"
    echo ""
    read -p "Do you want to continue anyway? (y/N) " -n 1 -r
    echo
    if [[ ! $REPLY =~ ^[Yy]$ ]]; then
        exit 1
    fi
fi

# Build test command
TEST_CMD="npm run"

if [ "$UI_MODE" = true ]; then
    TEST_CMD="$TEST_CMD test:ui"
elif [ "$DEBUG" = true ]; then
    TEST_CMD="$TEST_CMD test:debug"
elif [ "$HEADED" = true ]; then
    TEST_CMD="$TEST_CMD test:headed"
else
    case $TEST_TYPE in
        smoke)
            TEST_CMD="$TEST_CMD test:smoke"
            ;;
        integration)
            TEST_CMD="$TEST_CMD test:integration"
            ;;
        api)
            TEST_CMD="$TEST_CMD test:api"
            ;;
        websocket)
            TEST_CMD="$TEST_CMD test:websocket"
            ;;
        all)
            TEST_CMD="$TEST_CMD test"
            ;;
        *)
            echo -e "${RED}Invalid test type: $TEST_TYPE${NC}"
            usage
            exit 1
            ;;
    esac
fi

# Add service filter if specified
if [ -n "$SERVICE" ]; then
    if [ "$TEST_TYPE" != "all" ]; then
        echo -e "${YELLOW}Warning: Service filter overrides test type${NC}"
    fi
    TEST_CMD="npm run test -- --project=$SERVICE"
fi

# Run tests
echo -e "${GREEN}Running Playwright tests...${NC}"
echo -e "${GREEN}Command: $TEST_CMD${NC}"
echo ""

$TEST_CMD

# Show report if tests completed
if [ $? -eq 0 ] && [ "$UI_MODE" = false ] && [ "$DEBUG" = false ]; then
    echo ""
    echo -e "${GREEN}Tests completed successfully!${NC}"
    echo -e "${GREEN}View detailed report with: npm run test:report${NC}"
fi 