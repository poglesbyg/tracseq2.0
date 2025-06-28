import { defineConfig, devices } from '@playwright/test';

/**
 * Playwright configuration for TracSeq 2.0 Laboratory Management System
 * @see https://playwright.dev/docs/test-configuration
 */
export default defineConfig({
    testDir: './e2e',
    fullyParallel: true,
    forbidOnly: !!process.env.CI,
    retries: process.env.CI ? 2 : 0,
    workers: process.env.CI ? 1 : undefined,
    reporter: [
        ['html'],
        ['json', { outputFile: 'playwright-report/results.json' }],
        ['junit', { outputFile: 'playwright-report/results.xml' }]
    ],
    use: {
        baseURL: 'http://localhost:5173', // Vite dev server
        trace: 'on-first-retry',
        screenshot: 'only-on-failure',
        video: 'retain-on-failure',
    },

    projects: [
        {
            name: 'chromium',
            use: { ...devices['Desktop Chrome'] },
        },

        {
            name: 'firefox',
            use: { ...devices['Desktop Firefox'] },
        },

        {
            name: 'webkit',
            use: { ...devices['Desktop Safari'] },
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
    ],

    /* Configure API testing for microservices */
    webServer: [
        {
            command: 'pnpm dev',
            url: 'http://localhost:5173',
            reuseExistingServer: !process.env.CI,
            timeout: 120 * 1000,
        },
        // Uncomment when backend services are ready
        // {
        //   command: 'cargo run --bin auth_service',
        //   url: 'http://localhost:8001/health',
        //   cwd: '../../auth_service',
        //   reuseExistingServer: !process.env.CI,
        // },
    ],

    /* Global setup and teardown for laboratory testing */
    globalSetup: './e2e/global-setup.ts',
    globalTeardown: './e2e/global-teardown.ts',
}); 
