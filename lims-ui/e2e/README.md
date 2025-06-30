# TracSeq 2.0 End-to-End Testing with Playwright

## 🎭 **Overview**

This directory contains comprehensive end-to-end (E2E) tests for the TracSeq 2.0 Laboratory Management System using Playwright. The test suite covers critical laboratory workflows, authentication, and administrative functions across multiple browsers and devices.

## 📊 **Test Coverage**

### **Test Statistics**
- **Total Test Scenarios**: 47 unique tests
- **Total Test Executions**: 288 (47 tests × 6 browsers)
- **Test Categories**: 3 main areas
- **Browser Coverage**: 6 browsers including mobile

### **Test Categories**

#### 🔐 **Authentication Tests** (`auth/`)
- **File**: `login.spec.ts`
- **Tests**: 15 test scenarios
- **Coverage**: Login/logout, session management, password reset, rate limiting, error handling

#### 🧬 **Laboratory Tests** (`laboratory/`)
- **File**: `sample-management.spec.ts`
- **Tests**: 16 test scenarios
- **Coverage**: Sample creation, workflow management, QC processes, storage assignment, chain of custody

#### 👨‍💼 **Admin Tests** (`admin/`)
- **File**: `user-management.spec.ts`
- **Tests**: 16 test scenarios
- **Coverage**: User management, role-based access, audit logging, system settings

## 🛠️ **Test Infrastructure**

### **Directory Structure**
```
e2e/
├── auth/                    # Authentication tests
├── laboratory/              # Laboratory workflow tests
├── admin/                   # Administrative tests
├── utils/                   # Test utility functions
├── fixtures/                # Test data fixtures
├── pages/                   # Page object models
├── global-setup.ts          # Global test setup
├── global-teardown.ts       # Global test cleanup
└── README.md               # This documentation
```

### **Test Utilities** (`utils/test-helpers.ts`)
- **AuthHelpers**: Authentication workflows
- **LabHelpers**: Laboratory operations
- **FormHelpers**: Form interactions
- **WaitHelpers**: Async operations
- **AssertionHelpers**: Custom assertions
- **DataGenerators**: Test data creation

## 🚀 **Running Tests**

### **Available Scripts**
```bash
# Run all E2E tests
pnpm test:e2e

# Run tests with UI mode
pnpm test:e2e:ui

# Run tests in debug mode
pnpm test:e2e:debug

# Run tests in headed mode (see browser)
pnpm test:e2e:headed

# Run specific test categories
pnpm test:e2e:auth      # Authentication tests only
pnpm test:e2e:lab       # Laboratory tests only

# List all available tests
npx playwright test --list

# Run tests on specific browser
npx playwright test --project=chromium
npx playwright test --project=firefox
npx playwright test --project=webkit
```

### **Test Execution Examples**
```bash
# Run single test file
npx playwright test auth/login.spec.ts

# Run specific test
npx playwright test -g "should login successfully"

# Run tests in parallel (default)
npx playwright test --workers=4

# Generate test report
npx playwright show-report
```

## 🔧 **Configuration**

### **Browser Coverage**
- **Desktop**: Chromium, Firefox, Safari (WebKit), Microsoft Edge
- **Mobile**: Mobile Chrome (Pixel 5), Mobile Safari (iPhone 12)

### **Test Environment**
- **Base URL**: `http://localhost:5173` (Vite dev server)
- **Timeouts**: 30s default, 2m for webServer startup
- **Retries**: 2 retries in CI, 0 locally
- **Screenshots**: Only on failure
- **Videos**: Retained on failure
- **Traces**: On first retry

### **Test Data**
Test users and laboratory data are configured in global setup:

```typescript
// Test Users
{
  admin: 'admin.test@tracseq.com',
  researcher: 'researcher.test@tracseq.com', 
  technician: 'tech.test@tracseq.com'
}

// Laboratory Data
{
  projects: ['PROJ-001', 'PROJ-002'],
  samples: ['SAM-001', 'SAM-002'],
  equipment: ['SEQ-001', 'STOR-001']
}
```

## 📋 **Test Scenarios**

### **Authentication Tests**
- ✅ Login form validation and display
- ✅ Successful login for all user roles
- ✅ Invalid credentials handling
- ✅ Password reset workflow
- ✅ Session management and persistence
- ✅ Rate limiting protection
- ✅ Expired session handling
- ✅ Logout functionality
- ✅ Redirect preservation

### **Laboratory Management Tests**
- ✅ Sample dashboard display
- ✅ DNA/RNA sample creation
- ✅ Form validation
- ✅ Sample status workflows
- ✅ Search and filtering
- ✅ Storage location assignment
- ✅ Barcode generation
- ✅ Batch operations
- ✅ Data export functionality
- ✅ Chain of custody tracking
- ✅ Quality control measurements
- ✅ Role-based access control
- ✅ Data integrity validation

### **Admin Management Tests**
- ✅ User dashboard and controls
- ✅ User creation (researchers, technicians)
- ✅ Form validation and duplicate prevention
- ✅ User profile editing
- ✅ User enable/disable operations
- ✅ Password reset administration
- ✅ Session management
- ✅ User filtering and search
- ✅ Bulk operations
- ✅ Audit log viewing
- ✅ System settings management
- ✅ Statistics dashboard
- ✅ Access control enforcement

## 🎯 **Test Data Management**

### **Dynamic Test Data**
- Unique sample names: `TEST-SAMPLE-{timestamp}`
- Unique project names: `TEST-PROJECT-{timestamp}`
- Unique emails: `test.user.{timestamp}@tracseq.test`
- Random barcodes: `TRC-{9-char-alphanumeric}`

### **Test Isolation**
- Each test runs with clean state
- Database cleanup after each test
- Session isolation between tests
- No cross-test dependencies

## 🚦 **Continuous Integration**

### **CI Configuration**
Tests are configured for CI environments:
- **Retries**: 2 on failure
- **Workers**: 1 (serial execution for stability)
- **Browsers**: All platforms supported
- **Reports**: HTML, JSON, JUnit formats

### **Test Reports**
- **HTML Report**: `playwright-report/index.html`
- **JSON Results**: `playwright-report/results.json`
- **JUnit XML**: `playwright-report/results.xml`

## 🔍 **Debugging Tests**

### **Debug Mode**
```bash
# Step through tests interactively
pnpm test:e2e:debug

# Debug specific test
npx playwright test --debug auth/login.spec.ts

# Debug with browser developer tools
npx playwright test --debug --headed
```

### **Visual Debugging**
```bash
# Run with UI mode for visual debugging
pnpm test:e2e:ui

# Generate trace for failed tests
npx playwright show-trace trace.zip
```

### **Common Debug Techniques**
- Use `page.pause()` to stop execution
- Add `console.log()` for debugging
- Check `page.screenshot()` at failure points
- Examine network requests in traces
- Validate element selectors in browser

## 📈 **Best Practices**

### **Test Design**
- **Atomic Tests**: Each test is independent
- **Data-Driven**: Use test data generators
- **Page Objects**: Reusable page interactions
- **Assertions**: Clear, specific expectations
- **Error Handling**: Graceful failure management

### **Element Selection**
- **Preferred**: `data-testid` attributes
- **Fallback**: Semantic selectors
- **Avoid**: CSS classes and IDs
- **Pattern**: `[data-testid="component-action"]`

### **Async Handling**
- **Waits**: Explicit waits for elements
- **Loading**: Wait for loading states
- **APIs**: Wait for network responses
- **Animations**: Account for UI transitions

## 🚨 **Troubleshooting**

### **Common Issues**

#### **Test Timeouts**
```bash
# Increase timeout for slow operations
await page.waitForSelector('[data-testid="element"]', { timeout: 60000 });
```

#### **Element Not Found**
```bash
# Wait for element to be visible
await expect(page.locator('[data-testid="element"]')).toBeVisible();
```

#### **Flaky Tests**
```bash
# Add retry logic for unstable operations
await expect(async () => {
  await page.click('[data-testid="button"]');
  await expect(page.locator('[data-testid="result"]')).toBeVisible();
}).toPass({ timeout: 30000 });
```

### **Performance Issues**
- Run tests in parallel: `--workers=4`
- Skip unnecessary browser downloads
- Use headless mode for faster execution
- Optimize test data setup/teardown

## 📞 **Support**

### **Documentation**
- [Playwright Documentation](https://playwright.dev/)
- [Best Practices Guide](https://playwright.dev/docs/best-practices)
- [API Reference](https://playwright.dev/docs/api/class-playwright)

### **Team Resources**
- **Test Framework**: Playwright v1.53+
- **Language**: TypeScript
- **Runner**: Built-in Playwright Test Runner
- **CI/CD**: Configured for GitHub Actions/Azure DevOps

---

*Context improved by Giga AI - TracSeq 2.0 Laboratory Management System E2E Testing Suite* 
