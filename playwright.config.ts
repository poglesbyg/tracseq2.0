import { defineConfig, devices } from '@playwright/test';

/**
 * Playwright Configuration for TracSeq 2.0 Laboratory Management System
 * Tests the complete microservices architecture including:
 * - Frontend (React/Vite)
 * - API Gateway (Python)
 * - Lab Manager Backend (Rust)
 * - Microservices integration
 */

const isCI = !!process.env.CI;
const baseURL = process.env.BASE_URL || 'http://localhost:5176';
const apiGatewayURL = process.env.API_GATEWAY_URL || 'http://localhost:8089';
const backendURL = process.env.BACKEND_URL || 'http://localhost:3000';

/**
 * Read environment variables from file.
 * https://github.com/motdotla/dotenv
 */
// import dotenv from 'dotenv';
// import path from 'path';
// dotenv.config({ path: path.resolve(__dirname, '.env') });

/**
 * See https://playwright.dev/docs/test-configuration.
 */
export default defineConfig({
  testDir: './e2e',
  
  /* Run tests in files in parallel */
  fullyParallel: true,
  
  /* Fail the build on CI if you accidentally left test.only in the source code. */
  forbidOnly: isCI,
  
  /* Retry on CI only */
  retries: isCI ? 2 : 0,
  
  /* Opt out of parallel tests on CI. */
  workers: isCI ? 1 : undefined,
  
  /* Reporter to use */
  reporter: [
    ['html', { outputFolder: 'playwright-report' }],
    ['json', { outputFile: 'test-results/results.json' }],
    isCI ? ['github'] : ['list']
  ],
  
  /* Shared settings for all projects */
  use: {
    /* Base URL for frontend tests */
    baseURL,
    
    /* Collect trace when retrying failed test */
    trace: 'on-first-retry',
    
    /* Take screenshot on failure */
    screenshot: 'only-on-failure',
    
    /* Record video on failure */
    video: 'retain-on-failure',
    
    /* Global test timeout */
    actionTimeout: 15000,
    navigationTimeout: 30000,
  },

  /* Global test timeout */
  timeout: 60000,

  /* Configure projects for different test types */
  projects: [
    // Frontend E2E Tests
    {
      name: 'frontend-e2e',
      testDir: './e2e/frontend',
      use: { 
        ...devices['Desktop Chrome'],
        baseURL,
      },
    },

    // API Integration Tests  
    {
      name: 'api-integration',
      testDir: './e2e/api',
      use: { 
        ...devices['Desktop Chrome'],
        baseURL: apiGatewayURL,
      },
    },

    // Full Stack Integration Tests
    {
      name: 'full-stack',
      testDir: './e2e/integration', 
      use: { 
        ...devices['Desktop Chrome'],
        baseURL,
      },
      dependencies: ['frontend-e2e', 'api-integration'],
    },

    // Mobile Tests
    {
      name: 'mobile-chrome',
      testDir: './e2e/frontend',
      use: { 
        ...devices['Pixel 5'],
        baseURL,
      },
    },

    // Cross-browser Tests
    {
      name: 'firefox',
      testDir: './e2e/frontend',
      use: { 
        ...devices['Desktop Firefox'],
        baseURL,
      },
    },

    {
      name: 'safari',
      testDir: './e2e/frontend',
      use: { 
        ...devices['Desktop Safari'], 
        baseURL,
      },
    },
  ],

  /* Global setup and teardown */
  globalSetup: './e2e/global-setup.ts',
  globalTeardown: './e2e/global-teardown.ts',

  /* Run your local services before starting tests */
  webServer: isCI ? [
    // In CI, start all services
    {
      command: 'pnpm run test:services:start',
      url: backendURL + '/health',
      reuseExistingServer: false,
      timeout: 120000,
    },
    {
      command: 'pnpm run test:frontend:start', 
      url: baseURL,
      reuseExistingServer: false,
      timeout: 60000,
    },
  ] : [
    // In local development, check if services are running
    {
      command: 'echo "Checking if services are running..."',
      url: backendURL + '/health',
      reuseExistingServer: true,
      timeout: 5000,
    },
  ],

  /* Output directories */
  outputDir: 'test-results',
});
