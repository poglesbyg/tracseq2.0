import { test, expect } from '@playwright/test';

test.describe('MCP Dashboard Tests', () => {
  test.beforeEach(async ({ page }) => {
    await page.goto('/');
  });

  test('@smoke Dashboard should load successfully', async ({ page }) => {
    // Check that the dashboard loads
    await expect(page).toHaveTitle(/TracSeq MCP Monitor/);
    
    // Check for main dashboard elements
    await expect(page.locator('h1')).toContainText('TracSeq MCP Monitoring Dashboard');
    await expect(page.locator('text=System Overview')).toBeVisible();
  });

  test('Should display system health status', async ({ page }) => {
    // Check health score is displayed
    const healthScore = page.locator('text=/Health Score: \\d+\\/100/');
    await expect(healthScore).toBeVisible();
    
    // Check status indicators
    await expect(page.locator('text=/Status: (healthy|degraded|impaired|critical)/')).toBeVisible();
  });

  test('Should show service status for all MCP services', async ({ page }) => {
    // Check for service status section
    await expect(page.locator('h2:text("Service Status")')).toBeVisible();
    
    // Check for key services
    const services = ['cognitive_assistant', 'rag_service', 'storage_optimizer'];
    for (const service of services) {
      const serviceElement = page.locator(`h3:has-text("${service}")`);
      await expect(serviceElement).toBeVisible();
      
      // Check for status emoji (🟢 or 🔴)
      await expect(serviceElement.locator('text=/[🟢🔴]/')).toBeVisible();
    }
  });

  test('Should display performance metrics', async ({ page }) => {
    await expect(page.locator('h2:text("Performance Metrics")')).toBeVisible();
    
    // Check key metrics are displayed
    await expect(page.locator('text=/Total Requests: [\\d,]+/')).toBeVisible();
    await expect(page.locator('text=/Success Rate: \\d+\\.\\d+%/')).toBeVisible();
    await expect(page.locator('text=/Average Response Time: \\d+ms/')).toBeVisible();
    await expect(page.locator('text=/Active Connections: \\d+/')).toBeVisible();
  });

  test('Should show active alerts section', async ({ page }) => {
    await expect(page.locator('h2:text("Active Alerts")')).toBeVisible();
    
    // Check for either no alerts or alert list
    const noAlerts = page.locator('text=✅ No active alerts');
    const alertWarning = page.locator('text=/⚠️ (WARNING|CRITICAL):/');
    
    const hasNoAlerts = await noAlerts.isVisible().catch(() => false);
    const hasAlerts = await alertWarning.isVisible().catch(() => false);
    
    expect(hasNoAlerts || hasAlerts).toBeTruthy();
  });

  test('@integration Should refresh dashboard data', async ({ page }) => {
    // Get initial timestamp
    const initialTimestamp = await page.locator('text=/Last Update: .+/').textContent();
    
    // Wait for refresh interval (mock faster refresh for testing)
    await page.waitForTimeout(5000);
    
    // Check if timestamp has changed
    const newTimestamp = await page.locator('text=/Last Update: .+/').textContent();
    expect(newTimestamp).not.toBe(initialTimestamp);
  });

  test('Should navigate to service details', async ({ page }) => {
    // Click on a service
    await page.click('h3:has-text("cognitive_assistant")');
    
    // Should show service details
    await expect(page.locator('text=Service Details')).toBeVisible();
    await expect(page.locator('text=/Uptime: \\d+d \\d+h \\d+m/')).toBeVisible();
    await expect(page.locator('text=/Availability: \\d+\\.\\d+%/')).toBeVisible();
  });

  test('Should display quick actions', async ({ page }) => {
    await expect(page.locator('h2:text("Quick Actions")')).toBeVisible();
    
    // Check for action links
    await expect(page.locator('text=Check specific service')).toBeVisible();
    await expect(page.locator('text=View performance trends')).toBeVisible();
    await expect(page.locator('text=Configure alerts')).toBeVisible();
  });

  test('Should handle service errors gracefully', async ({ page }) => {
    // Navigate to a service that's offline
    const offlineService = page.locator('h3:has-text("storage_optimizer")');
    if (await offlineService.isVisible()) {
      await offlineService.click();
      
      // Should show error state
      await expect(page.locator('text=/Status: offline/')).toBeVisible();
      await expect(page.locator('text=/Error|Unavailable/')).toBeVisible();
    }
  });

  test('Should be responsive on mobile devices', async ({ page, viewport }) => {
    // Set mobile viewport
    await page.setViewportSize({ width: 375, height: 667 });
    
    // Check dashboard is still functional
    await expect(page.locator('h1')).toBeVisible();
    await expect(page.locator('text=System Overview')).toBeVisible();
    
    // Check layout adapts for mobile
    const mainContent = page.locator('main, .container, #root');
    const box = await mainContent.boundingBox();
    expect(box?.width).toBeLessThanOrEqual(375);
  });
}); 