import { defineConfig, devices } from '@playwright/test';

/**
 * Playwright configuration for TracSeq 2.0 Python services testing
 */
export default defineConfig({
  testDir: './tests',
  /* Run tests in files in parallel */
  fullyParallel: true,
  /* Fail the build on CI if you accidentally left test.only in the source code. */
  forbidOnly: !!process.env.CI,
  /* Retry on CI only */
  retries: process.env.CI ? 2 : 0,
  /* Opt out of parallel tests on CI. */
  workers: process.env.CI ? 1 : undefined,
  /* Reporter to use. See https://playwright.dev/docs/test-reporters */
  reporter: [
    ['html', { outputFolder: 'playwright-report' }],
    ['json', { outputFile: 'test-results/results.json' }],
    ['junit', { outputFile: 'test-results/junit.xml' }],
  ],
  /* Shared settings for all the projects below. See https://playwright.dev/docs/api/class-testoptions. */
  use: {
    /* Base URL to use in actions like `await page.goto('/')`. */
    baseURL: process.env.BASE_URL || 'http://localhost:8000',

    /* Collect trace when retrying the failed test. See https://playwright.dev/docs/trace-viewer */
    trace: 'on-first-retry',

    /* Screenshot on failure */
    screenshot: 'only-on-failure',

    /* Video on failure */
    video: 'retain-on-failure',
  },

  /* Configure projects for major browsers and services */
  projects: [
    {
      name: 'mcp-dashboard',
      use: { 
        ...devices['Desktop Chrome'],
        baseURL: 'http://localhost:7890',
      },
    },
    {
      name: 'enhanced-rag-service',
      use: { 
        ...devices['Desktop Chrome'],
        baseURL: 'http://localhost:8100',
      },
    },
    {
      name: 'lab-submission-rag',
      use: { 
        ...devices['Desktop Chrome'],
        baseURL: 'http://localhost:8000',
      },
    },
    {
      name: 'mcp-proxy-websocket',
      use: { 
        ...devices['Desktop Chrome'],
        baseURL: 'ws://localhost:9500',
      },
    },
    {
      name: 'ml-platform',
      use: { 
        ...devices['Desktop Chrome'],
        baseURL: 'http://localhost:8090',
      },
    },

    /* Test against mobile viewports. */
    {
      name: 'Mobile Chrome',
      use: { ...devices['Pixel 5'] },
    },
    {
      name: 'Mobile Safari',
      use: { ...devices['iPhone 12'] },
    },

    /* Test against branded browsers. */
    {
      name: 'Microsoft Edge',
      use: { ...devices['Desktop Edge'], channel: 'msedge' },
    },
    {
      name: 'Google Chrome',
      use: { ...devices['Desktop Chrome'], channel: 'chrome' },
    },
  ],

  /* Run your local dev server before starting the tests */
  webServer: [
    {
      command: 'cd ../mcp-dashboard && python mcp_monitoring_dashboard.py',
      port: 7890,
      timeout: 120 * 1000,
      reuseExistingServer: !process.env.CI,
    },
    {
      command: 'cd ../enhanced_rag_service && python -m uvicorn src.enhanced_rag_service.main:app --port 8100',
      port: 8100,
      timeout: 120 * 1000,
      reuseExistingServer: !process.env.CI,
    },
    {
      command: 'cd ../lab_submission_rag && python -m uvicorn api.main:app --port 8000',
      port: 8000,
      timeout: 120 * 1000,
      reuseExistingServer: !process.env.CI,
    },
    {
      command: 'cd ../mcp-proxy && python mcp_proxy_server.py',
      port: 9500,
      timeout: 120 * 1000,
      reuseExistingServer: !process.env.CI,
    },
  ],
}); 