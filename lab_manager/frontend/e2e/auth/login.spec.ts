import { test, expect } from '@playwright/test';
import { AuthHelpers, DataGenerators } from '../utils/test-helpers';

test.describe('TracSeq 2.0 Authentication - Login', () => {
    test.beforeEach(async ({ page }) => {
        // Ensure we start each test logged out
        await page.goto('/login');
    });

    test('should display login page correctly', async ({ page }) => {
        await expect(page).toHaveTitle(/TracSeq.*Login/);
        await expect(page.locator('[data-testid="login-form"]')).toBeVisible();
        await expect(page.locator('[data-testid="email-input"]')).toBeVisible();
        await expect(page.locator('[data-testid="password-input"]')).toBeVisible();
        await expect(page.locator('[data-testid="login-button"]')).toBeVisible();
        await expect(page.locator('[data-testid="forgot-password-link"]')).toBeVisible();
    });

    test('should login successfully with valid admin credentials', async ({ page }) => {
        const auth = new AuthHelpers(page);

        await auth.loginAs('admin');

        // Verify redirect to dashboard
        await expect(page).toHaveURL('/dashboard');
        await expect(page.locator('[data-testid="dashboard-title"]')).toBeVisible();
        await expect(page.locator('[data-testid="user-menu"]')).toContainText('Admin Test');
    });

    test('should login successfully with valid researcher credentials', async ({ page }) => {
        const auth = new AuthHelpers(page);

        await auth.loginAs('researcher');

        // Verify redirect to dashboard
        await expect(page).toHaveURL('/dashboard');
        await expect(page.locator('[data-testid="user-menu"]')).toContainText('Research Scientist');
    });

    test('should login successfully with valid technician credentials', async ({ page }) => {
        const auth = new AuthHelpers(page);

        await auth.loginAs('technician');

        // Verify redirect to dashboard
        await expect(page).toHaveURL('/dashboard');
        await expect(page.locator('[data-testid="user-menu"]')).toContainText('Lab Technician');
    });

    test('should show error for invalid credentials', async ({ page }) => {
        await page.fill('[data-testid="email-input"]', 'invalid@example.com');
        await page.fill('[data-testid="password-input"]', 'wrongpassword');
        await page.click('[data-testid="login-button"]');

        // Should stay on login page
        await expect(page).toHaveURL('/login');
        await expect(page.locator('[data-testid="error-message"]')).toBeVisible();
        await expect(page.locator('[data-testid="error-message"]')).toContainText(/Invalid credentials/i);
    });

    test('should show validation errors for empty fields', async ({ page }) => {
        await page.click('[data-testid="login-button"]');

        await expect(page.locator('[data-testid="email-error"]')).toBeVisible();
        await expect(page.locator('[data-testid="password-error"]')).toBeVisible();
    });

    test('should show validation error for invalid email format', async ({ page }) => {
        await page.fill('[data-testid="email-input"]', 'invalid-email');
        await page.fill('[data-testid="password-input"]', 'somepassword');
        await page.click('[data-testid="login-button"]');

        await expect(page.locator('[data-testid="email-error"]')).toBeVisible();
        await expect(page.locator('[data-testid="email-error"]')).toContainText(/valid email/i);
    });

    test('should handle forgotten password flow', async ({ page }) => {
        await page.click('[data-testid="forgot-password-link"]');

        await expect(page).toHaveURL('/forgot-password');
        await expect(page.locator('[data-testid="forgot-password-form"]')).toBeVisible();

        // Test email submission
        const testEmail = DataGenerators.generateTestEmail();
        await page.fill('[data-testid="email-input"]', testEmail);
        await page.click('[data-testid="submit-button"]');

        await expect(page.locator('[data-testid="success-message"]')).toBeVisible();
        await expect(page.locator('[data-testid="success-message"]'))
            .toContainText(/password reset instructions/i);
    });

    test('should redirect authenticated users away from login page', async ({ page }) => {
        const auth = new AuthHelpers(page);

        // Login first
        await auth.loginAs('admin');

        // Try to access login page
        await page.goto('/login');

        // Should be redirected to dashboard
        await expect(page).toHaveURL('/dashboard');
    });

    test('should maintain session across page reloads', async ({ page }) => {
        const auth = new AuthHelpers(page);

        await auth.loginAs('admin');

        // Reload the page
        await page.reload();

        // Should still be logged in
        await expect(page.locator('[data-testid="user-menu"]')).toBeVisible();
        expect(await auth.isLoggedIn()).toBe(true);
    });

    test('should handle concurrent login attempts (rate limiting)', async ({ page }) => {
        const invalidEmail = 'test@example.com';
        const invalidPassword = 'wrongpassword';

        // Attempt multiple rapid logins
        for (let i = 0; i < 6; i++) {
            await page.fill('[data-testid="email-input"]', invalidEmail);
            await page.fill('[data-testid="password-input"]', invalidPassword);
            await page.click('[data-testid="login-button"]');

            if (i < 5) {
                // First 5 attempts should show invalid credentials
                await expect(page.locator('[data-testid="error-message"]')).toBeVisible();
            }
        }

        // 6th attempt should show rate limiting error
        await expect(page.locator('[data-testid="error-message"]'))
            .toContainText(/too many attempts/i);
    });

    test('should logout successfully', async ({ page }) => {
        const auth = new AuthHelpers(page);

        // Login first
        await auth.loginAs('admin');

        // Logout
        await auth.logout();

        // Should be back on login page
        await expect(page).toHaveURL('/login');
        expect(await auth.isLoggedIn()).toBe(false);
    });

    test('should handle expired sessions gracefully', async ({ page }) => {
        const auth = new AuthHelpers(page);

        await auth.loginAs('admin');

        // Simulate expired session by manipulating localStorage/cookies
        await page.evaluate(() => {
            localStorage.removeItem('auth_token');
            document.cookie = 'session_token=; expires=Thu, 01 Jan 1970 00:00:00 UTC; path=/;';
        });

        // Try to access protected route
        await page.goto('/samples');

        // Should be redirected to login
        await expect(page).toHaveURL('/login');
        await expect(page.locator('[data-testid="info-message"]'))
            .toContainText(/session expired/i);
    });

    test('should preserve redirect URL after login', async ({ page }) => {
        // Try to access protected route while logged out
        await page.goto('/samples');

        // Should be redirected to login with return URL
        await expect(page).toHaveURL(/\/login\?redirect=.*samples/);

        const auth = new AuthHelpers(page);
        await auth.loginAs('researcher');

        // Should redirect back to original URL
        await expect(page).toHaveURL('/samples');
    });
}); 
