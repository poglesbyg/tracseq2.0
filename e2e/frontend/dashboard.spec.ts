import { test, expect } from '@playwright/test';

test.describe('TracSeq Dashboard', () => {
  test.beforeEach(async ({ page }) => {
    // Navigate to the dashboard
    await page.goto('/');
    
    // Wait for the page to load
    await page.waitForLoadState('networkidle');
  });

  test('should load dashboard successfully', async ({ page }) => {
    // Check for page title
    await expect(page).toHaveTitle(/TracSeq/);
    
    // Check for main navigation elements
    await expect(page.locator('nav')).toBeVisible();
    
    // Check for dashboard content
    await expect(page.locator('main')).toBeVisible();
  });

  test('should display dashboard statistics', async ({ page }) => {
    // Wait for statistics to load
    await page.waitForSelector('[data-testid="dashboard-stats"]', { timeout: 10000 });
    
    // Check for key statistics
    const statsContainer = page.locator('[data-testid="dashboard-stats"]');
    await expect(statsContainer).toBeVisible();
    
    // Check for specific stat cards
    await expect(page.locator('[data-testid="total-templates"]')).toBeVisible();
    await expect(page.locator('[data-testid="total-samples"]')).toBeVisible();
    await expect(page.locator('[data-testid="pending-sequencing"]')).toBeVisible();
    await expect(page.locator('[data-testid="completed-sequencing"]')).toBeVisible();
  });

  test('should navigate to samples page', async ({ page }) => {
    // Click on samples navigation link
    await page.click('[data-testid="nav-samples"]');
    
    // Wait for navigation
    await page.waitForURL('**/samples');
    
    // Check that we're on the samples page
    await expect(page.locator('h1')).toContainText('Samples');
  });

  test('should navigate to templates page', async ({ page }) => {
    // Click on templates navigation link
    await page.click('[data-testid="nav-templates"]');
    
    // Wait for navigation
    await page.waitForURL('**/templates');
    
    // Check that we're on the templates page
    await expect(page.locator('h1')).toContainText('Templates');
  });

  test('should handle API errors gracefully', async ({ page }) => {
    // Intercept API calls and simulate error
    await page.route('**/api/dashboard/stats', route => {
      route.fulfill({
        status: 500,
        contentType: 'application/json',
        body: JSON.stringify({ error: 'Internal Server Error' })
      });
    });
    
    // Reload the page
    await page.reload();
    
    // Check for error handling
    await expect(page.locator('[data-testid="error-message"]')).toBeVisible();
  });

  test('should be responsive on mobile', async ({ page }) => {
    // Set mobile viewport
    await page.setViewportSize({ width: 375, height: 667 });
    
    // Check that navigation is responsive
    await expect(page.locator('nav')).toBeVisible();
    
    // Check that dashboard content is visible
    await expect(page.locator('main')).toBeVisible();
  });

  test('should refresh data when refresh button is clicked', async ({ page }) => {
    // Wait for initial load
    await page.waitForSelector('[data-testid="dashboard-stats"]');
    
    // Get initial stats
    const initialStats = await page.locator('[data-testid="total-samples"]').textContent();
    
    // Click refresh button
    await page.click('[data-testid="refresh-button"]');
    
    // Wait for refresh to complete
    await page.waitForTimeout(1000);
    
    // Verify stats are still displayed (could be same values)
    await expect(page.locator('[data-testid="total-samples"]')).toBeVisible();
  });
}); 