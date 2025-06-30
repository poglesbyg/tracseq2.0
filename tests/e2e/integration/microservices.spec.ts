import { test, expect } from '@playwright/test';

test.describe('Full Stack Microservices Integration', () => {
  test.beforeEach(async ({ page }) => {
    // Navigate to the application
    await page.goto('/');
    await page.waitForLoadState('networkidle');
  });

  test('should complete full user workflow: view dashboard → navigate → view data', async ({ page }) => {
    // Step 1: Verify dashboard loads
    await expect(page.locator('h1')).toContainText(/Dashboard|TracSeq/);
    
    // Step 2: Check statistics are loaded via API Gateway
    await page.waitForSelector('[data-testid="dashboard-stats"]', { timeout: 10000 });
    
    const totalSamples = await page.locator('[data-testid="total-samples"]').textContent();
    expect(totalSamples).toMatch(/\d+/); // Should be a number
    
    // Step 3: Navigate to samples page
    await page.click('[data-testid="nav-samples"]');
    await page.waitForURL('**/samples');
    
    // Step 4: Verify samples data loads
    await expect(page.locator('h1')).toContainText('Samples');
    
    // Step 5: Return to dashboard and verify consistency
    await page.click('[data-testid="nav-dashboard"]');
    await page.waitForURL('**/');
    
    // Verify stats are still visible
    await expect(page.locator('[data-testid="dashboard-stats"]')).toBeVisible();
  });

  test('should handle end-to-end API data flow', async ({ page, request }) => {
    // Test direct API access
    const apiResponse = await request.get('http://localhost:8089/api/dashboard/stats');
    expect(apiResponse.ok()).toBeTruthy();
    
    const apiData = await apiResponse.json();
    
    // Navigate to frontend and verify same data appears
    await page.goto('/');
    await page.waitForSelector('[data-testid="dashboard-stats"]');
    
    // Compare API data with frontend display
    const frontendSamples = await page.locator('[data-testid="total-samples"]').textContent();
    expect(frontendSamples).toContain(apiData.totalSamples.toString());
  });

  test('should handle real-time data updates', async ({ page }) => {
    // Load dashboard
    await page.goto('/');
    await page.waitForSelector('[data-testid="dashboard-stats"]');
    
    // Get initial stats
    const initialStats = await page.locator('[data-testid="total-samples"]').textContent();
    
    // Simulate refresh/reload
    await page.click('[data-testid="refresh-button"]');
    
    // Wait for update
    await page.waitForTimeout(2000);
    
    // Verify stats are still displayed (may be same values)
    const updatedStats = await page.locator('[data-testid="total-samples"]').textContent();
    expect(updatedStats).toBeDefined();
  });

  test('should maintain session across microservices', async ({ page, context }) => {
    // Navigate through different sections
    await page.goto('/');
    
    await page.click('[data-testid="nav-templates"]');
    await page.waitForURL('**/templates');
    
    await page.click('[data-testid="nav-samples"]');
    await page.waitForURL('**/samples');
    
    await page.click('[data-testid="nav-dashboard"]');
    await page.waitForURL('**/');
    
    // Verify session/state is maintained throughout navigation
    await expect(page.locator('[data-testid="dashboard-stats"]')).toBeVisible();
  });

  test('should handle error scenarios gracefully across stack', async ({ page }) => {
    // Simulate network failure
    await page.route('**/api/**', route => {
      route.abort('failed');
    });
    
    // Navigate to page
    await page.goto('/');
    
    // Should show appropriate error handling
    await expect(page.locator('[data-testid="error-message"], .error, [data-error]')).toBeVisible({ timeout: 10000 });
    
    // Clear route and test recovery
    await page.unroute('**/api/**');
    
    // Reload page
    await page.reload();
    
    // Should recover successfully
    await expect(page.locator('[data-testid="dashboard-stats"]')).toBeVisible({ timeout: 10000 });
  });

  test('should handle concurrent users simulation', async ({ browser }) => {
    // Create multiple browser contexts to simulate concurrent users
    const contexts = await Promise.all([
      browser.newContext(),
      browser.newContext(),
      browser.newContext()
    ]);
    
    const pages = await Promise.all(
      contexts.map(context => context.newPage())
    );
    
    // Navigate all users to dashboard simultaneously
    await Promise.all(
      pages.map(page => page.goto('/'))
    );
    
    // Wait for all to load
    await Promise.all(
      pages.map(page => page.waitForSelector('[data-testid="dashboard-stats"]'))
    );
    
    // Verify all users can access data
    for (const page of pages) {
      await expect(page.locator('[data-testid="total-samples"]')).toBeVisible();
    }
    
    // Cleanup
    await Promise.all(contexts.map(context => context.close()));
  });

  test('should validate microservices health during operation', async ({ page, request }) => {
    // Check all services are healthy before test
    const services = [
      'http://localhost:3000/health',
      'http://localhost:8089/health',
      'http://localhost:8000/health'
    ];
    
    for (const service of services) {
      const response = await request.get(service);
      expect(response.ok()).toBeTruthy();
    }
    
    // Use the application
    await page.goto('/');
    await page.waitForSelector('[data-testid="dashboard-stats"]');
    
    // Navigate through different sections
    await page.click('[data-testid="nav-samples"]');
    await page.waitForURL('**/samples');
    
    await page.click('[data-testid="nav-templates"]');
    await page.waitForURL('**/templates');
    
    // Check services are still healthy after usage
    for (const service of services) {
      const response = await request.get(service);
      expect(response.ok()).toBeTruthy();
    }
  });

  test('should handle data consistency across API calls', async ({ request }) => {
    // Make multiple API calls and verify consistency
    const responses = await Promise.all([
      request.get('http://localhost:8089/api/dashboard/stats'),
      request.get('http://localhost:8089/api/dashboard/stats'),
      request.get('http://localhost:8089/api/dashboard/stats')
    ]);
    
    // All should succeed
    for (const response of responses) {
      expect(response.ok()).toBeTruthy();
    }
    
    // Parse data
    const dataArray = await Promise.all(
      responses.map(r => r.json())
    );
    
    // Verify consistency (all should return same values)
    const firstData = dataArray[0];
    for (const data of dataArray) {
      expect(data.totalSamples).toBe(firstData.totalSamples);
      expect(data.totalTemplates).toBe(firstData.totalTemplates);
    }
  });
}); 