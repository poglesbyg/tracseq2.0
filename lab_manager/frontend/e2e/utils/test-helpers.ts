import { Page, expect } from '@playwright/test';

/**
 * Test utility functions for TracSeq 2.0 Laboratory Management System
 */

export interface TestUser {
    email: string;
    password: string;
    firstName: string;
    lastName: string;
    role: string;
}

export interface TestSample {
    id: string;
    name: string;
    type: 'DNA' | 'RNA' | 'Protein' | 'Tissue';
    projectId: string;
    status: 'pending' | 'processing' | 'completed' | 'failed';
}

/**
 * Authentication helpers
 */
export class AuthHelpers {
    constructor(private page: Page) { }

    /**
     * Log in with test user credentials
     */
    async loginAs(userType: 'admin' | 'researcher' | 'technician') {
        const testUsers = JSON.parse(process.env.TEST_USERS || '{}');
        const user = testUsers[userType];

        if (!user) {
            throw new Error(`Test user '${userType}' not found`);
        }

        await this.page.goto('/login');
        await this.page.fill('[data-testid="email-input"]', user.email);
        await this.page.fill('[data-testid="password-input"]', user.password);
        await this.page.click('[data-testid="login-button"]');

        // Wait for successful login redirect
        await this.page.waitForURL('/dashboard');
        await expect(this.page.locator('[data-testid="user-menu"]')).toBeVisible();
    }

    /**
     * Log out current user
     */
    async logout() {
        await this.page.click('[data-testid="user-menu"]');
        await this.page.click('[data-testid="logout-button"]');
        await this.page.waitForURL('/login');
    }

    /**
     * Check if user is logged in
     */
    async isLoggedIn(): Promise<boolean> {
        try {
            await this.page.locator('[data-testid="user-menu"]').waitFor({ timeout: 5000 });
            return true;
        } catch {
            return false;
        }
    }
}

/**
 * Laboratory data helpers
 */
export class LabHelpers {
    constructor(private page: Page) { }

    /**
     * Navigate to samples page
     */
    async goToSamples() {
        await this.page.click('[data-testid="nav-samples"]');
        await this.page.waitForURL('/samples');
    }

    /**
     * Create a new sample
     */
    async createSample(sample: Partial<TestSample>) {
        await this.goToSamples();
        await this.page.click('[data-testid="create-sample-button"]');

        // Fill sample form
        if (sample.name) {
            await this.page.fill('[data-testid="sample-name-input"]', sample.name);
        }
        if (sample.type) {
            await this.page.selectOption('[data-testid="sample-type-select"]', sample.type);
        }
        if (sample.projectId) {
            await this.page.selectOption('[data-testid="project-select"]', sample.projectId);
        }

        await this.page.click('[data-testid="submit-sample-button"]');

        // Wait for success message
        await expect(this.page.locator('[data-testid="success-message"]')).toBeVisible();
    }

    /**
     * Search for samples
     */
    async searchSamples(query: string) {
        await this.goToSamples();
        await this.page.fill('[data-testid="search-input"]', query);
        await this.page.press('[data-testid="search-input"]', 'Enter');

        // Wait for search results
        await this.page.waitForSelector('[data-testid="sample-list"]');
    }

    /**
     * Get sample status
     */
    async getSampleStatus(sampleId: string): Promise<string> {
        await this.goToSamples();
        const statusLocator = this.page.locator(`[data-testid="sample-${sampleId}-status"]`);
        return await statusLocator.textContent() || '';
    }
}

/**
 * Form helpers
 */
export class FormHelpers {
    constructor(private page: Page) { }

    /**
     * Fill form fields by test IDs
     */
    async fillForm(fields: Record<string, string>) {
        for (const [testId, value] of Object.entries(fields)) {
            await this.page.fill(`[data-testid="${testId}"]`, value);
        }
    }

    /**
     * Select dropdown options by test IDs
     */
    async selectOptions(options: Record<string, string>) {
        for (const [testId, value] of Object.entries(options)) {
            await this.page.selectOption(`[data-testid="${testId}"]`, value);
        }
    }

    /**
     * Upload file to input
     */
    async uploadFile(testId: string, filePath: string) {
        await this.page.setInputFiles(`[data-testid="${testId}"]`, filePath);
    }
}

/**
 * Wait helpers
 */
export class WaitHelpers {
    constructor(private page: Page) { }

    /**
     * Wait for loading to complete
     */
    async waitForLoading() {
        await this.page.waitForSelector('[data-testid="loading-spinner"]', { state: 'hidden' });
    }

    /**
     * Wait for API request to complete
     */
    async waitForAPI(urlPattern: string) {
        await this.page.waitForResponse(response =>
            response.url().includes(urlPattern) && response.status() === 200
        );
    }

    /**
     * Wait for notification to appear
     */
    async waitForNotification(type: 'success' | 'error' | 'info' = 'success') {
        await this.page.waitForSelector(`[data-testid="${type}-notification"]`);
    }
}

/**
 * Data generation helpers
 */
export class DataGenerators {
    /**
     * Generate unique sample name
     */
    static generateSampleName(): string {
        const timestamp = Date.now();
        return `TEST-SAMPLE-${timestamp}`;
    }

    /**
     * Generate unique project name
     */
    static generateProjectName(): string {
        const timestamp = Date.now();
        return `TEST-PROJECT-${timestamp}`;
    }

    /**
     * Generate test email
     */
    static generateTestEmail(): string {
        const timestamp = Date.now();
        return `test.user.${timestamp}@tracseq.test`;
    }

    /**
     * Generate random barcode
     */
    static generateBarcode(): string {
        return `TRC-${Math.random().toString(36).substr(2, 9).toUpperCase()}`;
    }
}

/**
 * Assertion helpers
 */
export class AssertionHelpers {
    constructor(private page: Page) { }

    /**
     * Assert page has no console errors
     */
    async assertNoConsoleErrors() {
        const errors: string[] = [];
        this.page.on('console', msg => {
            if (msg.type() === 'error') {
                errors.push(msg.text());
            }
        });

        // Wait a bit for any errors to appear
        await this.page.waitForTimeout(1000);

        if (errors.length > 0) {
            throw new Error(`Console errors found: ${errors.join(', ')}`);
        }
    }

    /**
     * Assert API response is successful
     */
    async assertAPISuccess(urlPattern: string) {
        const response = await this.page.waitForResponse(response =>
            response.url().includes(urlPattern)
        );
        expect(response.status()).toBe(200);
    }

    /**
     * Assert table has expected number of rows
     */
    async assertTableRowCount(tableTestId: string, expectedCount: number) {
        const rows = this.page.locator(`[data-testid="${tableTestId}"] tbody tr`);
        await expect(rows).toHaveCount(expectedCount);
    }
} 
