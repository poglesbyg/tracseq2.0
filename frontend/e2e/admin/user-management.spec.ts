import { test, expect } from '@playwright/test';
import { AuthHelpers, DataGenerators, WaitHelpers, AssertionHelpers } from '../utils/test-helpers';

test.describe('TracSeq 2.0 Admin - User Management', () => {
    let auth: AuthHelpers;
    let wait: WaitHelpers;
    let assertions: AssertionHelpers;

    test.beforeEach(async ({ page }) => {
        auth = new AuthHelpers(page);
        wait = new WaitHelpers(page);
        assertions = new AssertionHelpers(page);

        // Login as admin for all tests
        await auth.loginAs('admin');
    });

    test.afterEach(async () => {
        await auth.logout();
    });

    test('should display admin dashboard with user management section', async ({ page }) => {
        await page.goto('/admin/users');

        await expect(page.locator('[data-testid="admin-dashboard"]')).toBeVisible();
        await expect(page.locator('[data-testid="users-table"]')).toBeVisible();
        await expect(page.locator('[data-testid="create-user-button"]')).toBeVisible();
        await expect(page.locator('[data-testid="user-filters"]')).toBeVisible();
        await expect(page.locator('[data-testid="bulk-actions"]')).toBeVisible();
    });

    test('should create a new researcher user', async ({ page }) => {
        await page.goto('/admin/users');

        const testEmail = DataGenerators.generateTestEmail();

        await page.click('[data-testid="create-user-button"]');

        // Fill user creation form
        await page.fill('[data-testid="first-name-input"]', 'Test');
        await page.fill('[data-testid="last-name-input"]', 'Researcher');
        await page.fill('[data-testid="email-input"]', testEmail);
        await page.selectOption('[data-testid="role-select"]', 'ResearchScientist');
        await page.fill('[data-testid="department-input"]', 'Genomics');
        await page.fill('[data-testid="position-input"]', 'Senior Researcher');

        await page.click('[data-testid="submit-user-button"]');

        await wait.waitForNotification('success');

        // Verify user created
        await page.fill('[data-testid="search-users-input"]', testEmail);
        await page.press('[data-testid="search-users-input"]', 'Enter');

        await expect(page.locator(`[data-testid="user-${testEmail}"]`)).toBeVisible();
        await expect(page.locator(`[data-testid="user-${testEmail}-role"]`)).toContainText('Research Scientist');
        await expect(page.locator(`[data-testid="user-${testEmail}-status"]`)).toContainText('Active');
    });

    test('should create a new technician user', async ({ page }) => {
        await page.goto('/admin/users');

        const testEmail = DataGenerators.generateTestEmail();

        await page.click('[data-testid="create-user-button"]');

        await page.fill('[data-testid="first-name-input"]', 'Test');
        await page.fill('[data-testid="last-name-input"]', 'Technician');
        await page.fill('[data-testid="email-input"]', testEmail);
        await page.selectOption('[data-testid="role-select"]', 'LabTechnician');
        await page.fill('[data-testid="department-input"]', 'Laboratory Operations');

        await page.click('[data-testid="submit-user-button"]');

        await wait.waitForNotification('success');

        // Verify user created with correct permissions
        await page.fill('[data-testid="search-users-input"]', testEmail);
        await page.press('[data-testid="search-users-input"]', 'Enter');

        await expect(page.locator(`[data-testid="user-${testEmail}-role"]`)).toContainText('Lab Technician');
    });

    test('should validate required fields when creating user', async ({ page }) => {
        await page.goto('/admin/users');

        await page.click('[data-testid="create-user-button"]');

        // Try to submit without required fields
        await page.click('[data-testid="submit-user-button"]');

        // Should show validation errors
        await expect(page.locator('[data-testid="first-name-error"]')).toBeVisible();
        await expect(page.locator('[data-testid="last-name-error"]')).toBeVisible();
        await expect(page.locator('[data-testid="email-error"]')).toBeVisible();
        await expect(page.locator('[data-testid="role-error"]')).toBeVisible();
    });

    test('should prevent duplicate email addresses', async ({ page }) => {
        await page.goto('/admin/users');

        // Try to create user with existing email
        await page.click('[data-testid="create-user-button"]');

        await page.fill('[data-testid="first-name-input"]', 'Duplicate');
        await page.fill('[data-testid="last-name-input"]', 'User');
        await page.fill('[data-testid="email-input"]', 'admin.test@tracseq.com'); // Existing admin email
        await page.selectOption('[data-testid="role-select"]', 'ResearchScientist');

        await page.click('[data-testid="submit-user-button"]');

        // Should show error
        await expect(page.locator('[data-testid="email-error"]')).toBeVisible();
        await expect(page.locator('[data-testid="email-error"]')).toContainText(/already exists/i);
    });

    test('should view user details and edit profile', async ({ page }) => {
        await page.goto('/admin/users');

        // Find and click on a user
        const userRow = page.locator('[data-testid^="user-"]:first-child');
        await userRow.click();

        // Should navigate to user details
        await expect(page.locator('[data-testid="user-details-page"]')).toBeVisible();
        await expect(page.locator('[data-testid="user-profile"]')).toBeVisible();
        await expect(page.locator('[data-testid="user-sessions"]')).toBeVisible();
        await expect(page.locator('[data-testid="user-activity-log"]')).toBeVisible();

        // Edit user profile
        await page.click('[data-testid="edit-user-button"]');

        await page.fill('[data-testid="department-input"]', 'Updated Department');
        await page.fill('[data-testid="position-input"]', 'Updated Position');

        await page.click('[data-testid="save-user-button"]');

        await wait.waitForNotification('success');

        // Verify changes saved
        await expect(page.locator('[data-testid="user-department"]')).toContainText('Updated Department');
        await expect(page.locator('[data-testid="user-position"]')).toContainText('Updated Position');
    });

    test('should disable and enable users', async ({ page }) => {
        await page.goto('/admin/users');

        const testEmail = DataGenerators.generateTestEmail();

        // Create test user first
        await page.click('[data-testid="create-user-button"]');
        await page.fill('[data-testid="first-name-input"]', 'Test');
        await page.fill('[data-testid="last-name-input"]', 'User');
        await page.fill('[data-testid="email-input"]', testEmail);
        await page.selectOption('[data-testid="role-select"]', 'ResearchScientist');
        await page.click('[data-testid="submit-user-button"]');

        await wait.waitForNotification('success');

        // Search for the user
        await page.fill('[data-testid="search-users-input"]', testEmail);
        await page.press('[data-testid="search-users-input"]', 'Enter');

        // Disable user
        await page.click(`[data-testid="user-${testEmail}-actions"]`);
        await page.click(`[data-testid="disable-user-${testEmail}"]`);

        // Confirm disable
        await page.click('[data-testid="confirm-disable-user"]');

        await wait.waitForNotification('success');

        // Verify user disabled
        await expect(page.locator(`[data-testid="user-${testEmail}-status"]`)).toContainText('Inactive');

        // Enable user
        await page.click(`[data-testid="user-${testEmail}-actions"]`);
        await page.click(`[data-testid="enable-user-${testEmail}"]`);

        await wait.waitForNotification('success');

        // Verify user enabled
        await expect(page.locator(`[data-testid="user-${testEmail}-status"]`)).toContainText('Active');
    });

    test('should reset user password', async ({ page }) => {
        await page.goto('/admin/users');

        // Find a user and open actions menu
        const userEmail = 'researcher.test@tracseq.com';
        await page.fill('[data-testid="search-users-input"]', userEmail);
        await page.press('[data-testid="search-users-input"]', 'Enter');

        await page.click(`[data-testid="user-${userEmail}-actions"]`);
        await page.click(`[data-testid="reset-password-${userEmail}"]`);

        // Confirm password reset
        await page.click('[data-testid="confirm-password-reset"]');

        await wait.waitForNotification('success');

        // Should show temporary password or email sent message
        await expect(page.locator('[data-testid="success-message"]'))
            .toContainText(/password reset/i);
    });

    test('should view and manage user sessions', async ({ page }) => {
        await page.goto('/admin/sessions');

        await expect(page.locator('[data-testid="sessions-table"]')).toBeVisible();
        await expect(page.locator('[data-testid="active-sessions-count"]')).toBeVisible();

        // Should show session details
        const sessionRows = page.locator('[data-testid^="session-"]');
        const firstSession = sessionRows.first();

        await expect(firstSession.locator('[data-testid$="-user-email"]')).toBeVisible();
        await expect(firstSession.locator('[data-testid$="-last-activity"]')).toBeVisible();
        await expect(firstSession.locator('[data-testid$="-ip-address"]')).toBeVisible();

        // Revoke a session
        await firstSession.locator('[data-testid$="-revoke-button"]').click();
        await page.click('[data-testid="confirm-revoke-session"]');

        await wait.waitForNotification('success');
    });

    test('should filter users by role', async ({ page }) => {
        await page.goto('/admin/users');

        // Filter by role
        await page.click('[data-testid="role-filter"]');
        await page.click('[data-testid="role-filter-admin"]');

        await wait.waitForLoading();

        // All visible users should be admins
        const roleElements = await page.locator('[data-testid$="-role"]').all();
        for (const element of roleElements) {
            await expect(element).toContainText(/Admin/i);
        }
    });

    test('should filter users by status', async ({ page }) => {
        await page.goto('/admin/users');

        // Filter by status
        await page.click('[data-testid="status-filter"]');
        await page.click('[data-testid="status-filter-active"]');

        await wait.waitForLoading();

        // All visible users should be active
        const statusElements = await page.locator('[data-testid$="-status"]').all();
        for (const element of statusElements) {
            await expect(element).toContainText('Active');
        }
    });

    test('should perform bulk user operations', async ({ page }) => {
        await page.goto('/admin/users');

        // Select multiple users
        const userCheckboxes = page.locator('[data-testid$="-checkbox"]');
        const firstThree = await userCheckboxes.first(3).all();

        for (const checkbox of firstThree) {
            await checkbox.check();
        }

        // Perform bulk operation
        await page.click('[data-testid="bulk-actions-dropdown"]');
        await page.click('[data-testid="bulk-export-users"]');

        // Should trigger download
        const downloadPromise = page.waitForEvent('download');
        await page.click('[data-testid="confirm-bulk-export"]');

        const download = await downloadPromise;
        expect(download.suggestedFilename()).toMatch(/users.*\.csv$/);
    });

    test('should view audit log', async ({ page }) => {
        await page.goto('/admin/audit');

        await expect(page.locator('[data-testid="audit-log-table"]')).toBeVisible();
        await expect(page.locator('[data-testid="audit-filters"]')).toBeVisible();

        // Filter by event type
        await page.selectOption('[data-testid="event-type-filter"]', 'user_login');
        await page.click('[data-testid="apply-filters-button"]');

        await wait.waitForLoading();

        // All entries should be login events
        const eventElements = await page.locator('[data-testid$="-event-type"]').all();
        for (const element of eventElements) {
            await expect(element).toContainText(/login/i);
        }
    });

    test('should manage system settings', async ({ page }) => {
        await page.goto('/admin/settings');

        await expect(page.locator('[data-testid="system-settings"]')).toBeVisible();
        await expect(page.locator('[data-testid="security-settings"]')).toBeVisible();
        await expect(page.locator('[data-testid="email-settings"]')).toBeVisible();

        // Update a setting
        await page.fill('[data-testid="session-timeout-input"]', '120');
        await page.click('[data-testid="save-settings-button"]');

        await wait.waitForNotification('success');

        // Verify setting saved
        await expect(page.locator('[data-testid="session-timeout-input"]')).toHaveValue('120');
    });

    test('should display system statistics', async ({ page }) => {
        await page.goto('/admin/dashboard');

        await expect(page.locator('[data-testid="total-users-stat"]')).toBeVisible();
        await expect(page.locator('[data-testid="active-sessions-stat"]')).toBeVisible();
        await expect(page.locator('[data-testid="total-samples-stat"]')).toBeVisible();
        await expect(page.locator('[data-testid="system-health-stat"]')).toBeVisible();

        // Verify stats have numeric values
        const totalUsers = await page.locator('[data-testid="total-users-value"]').textContent();
        expect(parseInt(totalUsers || '0')).toBeGreaterThan(0);
    });

    test('should handle non-admin access restrictions', async ({ page }) => {
        // Logout admin and login as researcher
        await auth.logout();
        await auth.loginAs('researcher');

        // Try to access admin pages
        await page.goto('/admin/users');

        // Should be redirected or show access denied
        await expect(page).not.toHaveURL('/admin/users');
        await expect(page.locator('[data-testid="access-denied"]')).toBeVisible();
    });

    test('should validate data integrity across admin operations', async ({ page }) => {
        await page.goto('/admin/users');

        // Verify no console errors during admin operations
        await assertions.assertNoConsoleErrors();

        // Verify API responses are successful
        await assertions.assertAPISuccess('/api/admin/users');

        // Verify table data loads correctly
        await assertions.assertTableRowCount('users-table', 3); // Expected number of test users
    });
}); 
