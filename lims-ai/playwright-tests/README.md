# TracSeq 2.0 Python Services - Playwright E2E Tests

This directory contains end-to-end tests for all Python services in the TracSeq 2.0 laboratory management system using Playwright.

## Overview

The test suite covers:
- **MCP Dashboard** - Real-time monitoring interface
- **Enhanced RAG Service** - Document processing and information extraction
- **Lab Submission RAG** - Laboratory submission form processing
- **MCP Proxy** - WebSocket-based service orchestration
- **ML Platform Services** - Feature store, model serving, and AutoML

## Setup

### Prerequisites
- Node.js 18+ and npm/pnpm
- Python 3.10+ with all services installed
- Docker (if running services in containers)
- Ollama with required models

### Installation

```bash
# Navigate to test directory
cd lims-ai/playwright-tests

# Install dependencies
npm install

# Install Playwright browsers
npm run install:browsers
```

### Environment Configuration

Create a `.env` file:

```env
# Service URLs
MCP_DASHBOARD_URL=http://localhost:7890
ENHANCED_RAG_URL=http://localhost:8100
LAB_SUBMISSION_URL=http://localhost:8000
MCP_PROXY_URL=ws://localhost:9500
ML_PLATFORM_URL=http://localhost:8090

# Test Configuration
TEST_TIMEOUT=30000
HEADLESS=true
SLOW_MO=0

# Authentication
TEST_USER=testuser
TEST_PASSWORD=testpass123
JWT_SECRET=test-secret-key
```

## Running Tests

### All Tests
```bash
npm test
```

### Specific Service Tests
```bash
# MCP Dashboard
npm run test:mcp

# RAG Services
npm run test:rag

# ML Platform
npm run test:ml

# API Tests only
npm run test:api

# WebSocket Tests
npm run test:websocket
```

### Test Modes
```bash
# Run with UI mode
npm run test:ui

# Debug mode
npm run test:debug

# Headed mode (see browser)
npm run test:headed

# Smoke tests only
npm run test:smoke

# Integration tests
npm run test:integration
```

### Generate Tests
```bash
# Use Playwright codegen
npm run codegen
```

## Test Structure

```
tests/
├── mcp-dashboard/           # MCP Dashboard UI tests
│   └── dashboard.spec.ts
├── api/                     # API endpoint tests
│   ├── enhanced-rag-service.spec.ts
│   └── lab-submission-api.spec.ts
├── lab-submission/          # Lab submission UI tests
│   └── submission-ui.spec.ts
├── websocket/              # WebSocket tests
│   └── mcp-proxy-ws.spec.ts
├── ml-platform/            # ML platform tests
│   ├── feature-store.spec.ts
│   └── model-serving.spec.ts
└── utils/                  # Test utilities
    └── test-helpers.ts
```

## Test Categories

### Smoke Tests (@smoke)
Quick tests to verify basic functionality:
- Service health checks
- Main page loads
- Core features accessible

### Integration Tests (@integration)
Tests that verify service interactions:
- Document processing workflows
- Service-to-service communication
- End-to-end user flows

### API Tests
Direct API endpoint testing:
- REST endpoints
- Response validation
- Error handling
- Performance metrics

### WebSocket Tests
Real-time communication testing:
- Connection establishment
- Message routing
- Service orchestration
- Transaction support

## Writing Tests

### Basic Test Structure
```typescript
import { test, expect } from '@playwright/test';

test.describe('Service Name Tests', () => {
  test.beforeEach(async ({ page }) => {
    await page.goto('/');
  });

  test('@smoke should load homepage', async ({ page }) => {
    await expect(page).toHaveTitle(/Service Name/);
  });
});
```

### Using Test Helpers
```typescript
import { 
  generateTestSample, 
  uploadFile, 
  expectNotification 
} from '../utils/test-helpers';

test('should process document', async ({ page }) => {
  const sample = generateTestSample();
  await uploadFile(page, 'input[type="file"]', 'test.txt', sample);
  await expectNotification(page, 'success', 'Document processed');
});
```

## CI/CD Integration

### GitHub Actions
```yaml
name: E2E Tests
on: [push, pull_request]
jobs:
  test:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v3
      - uses: actions/setup-node@v3
      - uses: actions/setup-python@v4
      - name: Install dependencies
        run: |
          npm ci
          pip install -r requirements.txt
      - name: Run Playwright tests
        run: npm test
      - uses: actions/upload-artifact@v3
        if: always()
        with:
          name: playwright-report
          path: playwright-report/
```

## Debugging

### VSCode Configuration
```json
{
  "version": "0.2.0",
  "configurations": [
    {
      "name": "Debug Playwright Tests",
      "type": "node",
      "request": "launch",
      "program": "${workspaceFolder}/node_modules/.bin/playwright",
      "args": ["test", "--debug"],
      "console": "integratedTerminal",
      "internalConsoleOptions": "neverOpen"
    }
  ]
}
```

### Common Issues

1. **Services not running**
   ```bash
   # Start all services
   cd ../..
   ./docker/mcp/start-mcp.sh
   ```

2. **Port conflicts**
   - Check `.env` file for correct ports
   - Use `lsof -i :PORT` to find conflicts

3. **Timeout issues**
   - Increase `TEST_TIMEOUT` in `.env`
   - Check service health endpoints
   - Verify network connectivity

## Reports

### View HTML Report
```bash
npm run test:report
```

### Report Locations
- HTML Report: `playwright-report/index.html`
- JSON Results: `test-results/results.json`
- JUnit XML: `test-results/junit.xml`
- Screenshots: `test-results/screenshots/`
- Videos: `test-results/videos/`

## Best Practices

1. **Test Isolation**
   - Each test should be independent
   - Clean up test data after tests
   - Don't rely on test execution order

2. **Selectors**
   - Use data-testid attributes when possible
   - Prefer role-based selectors for accessibility
   - Avoid brittle CSS selectors

3. **Assertions**
   - Use specific assertions
   - Wait for elements before asserting
   - Check both positive and negative cases

4. **Performance**
   - Keep tests focused and fast
   - Use test.parallel() when possible
   - Mock external dependencies when appropriate

## Contributing

1. Follow the existing test patterns
2. Add appropriate test tags (@smoke, @integration)
3. Update this README for new test categories
4. Ensure all tests pass before submitting PR

## License

See main project LICENSE file.

---

*For more information about the TracSeq 2.0 project, see the main project documentation.* 