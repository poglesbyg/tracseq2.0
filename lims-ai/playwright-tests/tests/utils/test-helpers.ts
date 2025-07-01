import { Page, expect } from '@playwright/test';
import { v4 as uuidv4 } from 'uuid';
import jwt from 'jsonwebtoken';

/**
 * Generate a test authentication token
 */
export function generateTestToken(userId: string = 'test-user', role: string = 'lab_technician'): string {
  const payload = {
    sub: userId,
    role: role,
    exp: Math.floor(Date.now() / 1000) + (60 * 60), // 1 hour
    iat: Math.floor(Date.now() / 1000),
  };
  
  // Use a test secret - in real tests, this should match the service's test secret
  return jwt.sign(payload, 'test-secret-key');
}

/**
 * Login helper for authenticated endpoints
 */
export async function loginUser(page: Page, username: string = 'testuser', password: string = 'testpass123'): Promise<string> {
  await page.goto('/login');
  await page.fill('input[name="username"]', username);
  await page.fill('input[name="password"]', password);
  await page.click('button[type="submit"]');
  
  // Wait for redirect after login
  await page.waitForURL(/dashboard|home/, { timeout: 5000 });
  
  // Get auth token from localStorage or cookie
  const token = await page.evaluate(() => {
    return localStorage.getItem('auth_token') || '';
  });
  
  return token;
}

/**
 * Generate test sample data
 */
export function generateTestSample() {
  return {
    sampleId: `TEST-${uuidv4().substring(0, 8).toUpperCase()}`,
    sampleType: ['DNA', 'RNA', 'Protein', 'Blood'][Math.floor(Math.random() * 4)],
    volume: Math.floor(Math.random() * 200) + 50,
    concentration: Math.floor(Math.random() * 100) + 10,
    storageTemperature: ['-80', '-20', '4', '25'][Math.floor(Math.random() * 4)],
    submittedBy: 'Test User',
    submissionDate: new Date().toISOString(),
  };
}

/**
 * Generate test document content
 */
export function generateTestDocument(type: 'lab_submission' | 'protocol' | 'report' = 'lab_submission'): string {
  const sample = generateTestSample();
  
  switch (type) {
    case 'lab_submission':
      return `
Laboratory Submission Form
==========================
Sample ID: ${sample.sampleId}
Sample Type: ${sample.sampleType}
Volume: ${sample.volume} µL
Concentration: ${sample.concentration} ng/µL
Storage Temperature: ${sample.storageTemperature}°C
Submitted By: ${sample.submittedBy}
Date: ${sample.submissionDate}

Additional Notes:
Sample collected under sterile conditions.
Requires genomic sequencing analysis.
      `.trim();
      
    case 'protocol':
      return `
DNA Extraction Protocol
======================
Protocol Version: 2.1
Last Updated: ${new Date().toISOString()}

Materials:
- DNA extraction kit
- Ethanol (70%)
- Centrifuge tubes
- Pipettes

Procedure:
1. Add 200µL of sample to extraction tube
2. Add lysis buffer and mix
3. Incubate at 56°C for 10 minutes
4. Add binding buffer
5. Centrifuge at 12,000g for 1 minute
      `.trim();
      
    case 'report':
      return `
Sequencing Report
=================
Report ID: RPT-${uuidv4().substring(0, 8)}
Sample ID: ${sample.sampleId}
Analysis Date: ${new Date().toISOString()}

Results Summary:
- Total Reads: 10,234,567
- Quality Score: Q30 > 92%
- Coverage: 30X
- Variants Detected: 1,234

Conclusion:
Sample quality meets requirements for analysis.
      `.trim();
  }
}

/**
 * Wait for API response with retry
 */
export async function waitForApiResponse(
  page: Page,
  url: string | RegExp,
  options: { timeout?: number; retries?: number } = {}
): Promise<any> {
  const { timeout = 10000, retries = 3 } = options;
  
  for (let i = 0; i < retries; i++) {
    try {
      const response = await page.waitForResponse(
        response => {
          if (typeof url === 'string') {
            return response.url().includes(url) && response.status() === 200;
          } else {
            return url.test(response.url()) && response.status() === 200;
          }
        },
        { timeout }
      );
      
      return await response.json();
    } catch (error) {
      if (i === retries - 1) throw error;
      await page.waitForTimeout(1000); // Wait 1s before retry
    }
  }
}

/**
 * Upload file helper
 */
export async function uploadFile(
  page: Page,
  selector: string,
  fileName: string,
  content: string,
  mimeType: string = 'text/plain'
): Promise<void> {
  const fileInput = page.locator(selector);
  await fileInput.setInputFiles({
    name: fileName,
    mimeType: mimeType,
    buffer: Buffer.from(content),
  });
}

/**
 * Check notification appears
 */
export async function expectNotification(
  page: Page,
  type: 'success' | 'error' | 'warning' | 'info',
  messagePattern?: string | RegExp
): Promise<void> {
  const notificationSelector = `.notification.${type}, .alert.alert-${type}, [role="alert"].${type}`;
  const notification = page.locator(notificationSelector);
  
  await expect(notification).toBeVisible({ timeout: 5000 });
  
  if (messagePattern) {
    if (typeof messagePattern === 'string') {
      await expect(notification).toContainText(messagePattern);
    } else {
      await expect(notification).toHaveText(messagePattern);
    }
  }
}

/**
 * Mock API response
 */
export async function mockApiResponse(
  page: Page,
  url: string | RegExp,
  response: any,
  status: number = 200
): Promise<void> {
  await page.route(url, route => {
    route.fulfill({
      status: status,
      contentType: 'application/json',
      body: JSON.stringify(response),
    });
  });
}

/**
 * Take screenshot with timestamp
 */
export async function takeScreenshot(
  page: Page,
  name: string,
  fullPage: boolean = false
): Promise<void> {
  const timestamp = new Date().toISOString().replace(/[:.]/g, '-');
  await page.screenshot({
    path: `test-results/screenshots/${name}-${timestamp}.png`,
    fullPage: fullPage,
  });
}

/**
 * Check accessibility
 */
export async function checkAccessibility(page: Page, selector?: string): Promise<void> {
  const target = selector ? page.locator(selector) : page;
  
  // Check for basic accessibility attributes
  const elements = await target.locator('[role], [aria-label], [aria-describedby]').all();
  expect(elements.length).toBeGreaterThan(0);
  
  // Check for form labels
  const inputs = await target.locator('input:not([type="hidden"]), select, textarea').all();
  for (const input of inputs) {
    const id = await input.getAttribute('id');
    if (id) {
      const label = page.locator(`label[for="${id}"]`);
      await expect(label).toBeVisible();
    }
  }
  
  // Check for alt text on images
  const images = await target.locator('img').all();
  for (const img of images) {
    const alt = await img.getAttribute('alt');
    expect(alt).toBeTruthy();
  }
}

/**
 * Performance timing helper
 */
export async function measurePerformance(
  page: Page,
  action: () => Promise<void>,
  metricName: string
): Promise<number> {
  const startTime = Date.now();
  await action();
  const endTime = Date.now();
  
  const duration = endTime - startTime;
  console.log(`Performance: ${metricName} took ${duration}ms`);
  
  return duration;
} 