import { test, expect } from '@playwright/test';

test.describe('Lab Submission RAG UI Tests', () => {
  test.beforeEach(async ({ page }) => {
    await page.goto('/');
  });

  test('@smoke Lab submission form should load', async ({ page }) => {
    // Check page title
    await expect(page).toHaveTitle(/Lab Submission|TracSeq/);
    
    // Check main form elements
    await expect(page.locator('h1, h2').filter({ hasText: /Lab.*Submission/i })).toBeVisible();
    await expect(page.locator('form')).toBeVisible();
  });

  test('Should display all required form fields', async ({ page }) => {
    // Check for essential form fields
    const requiredFields = [
      { label: 'Sample ID', type: 'input' },
      { label: 'Sample Type', type: 'select' },
      { label: 'Volume', type: 'input' },
      { label: 'Concentration', type: 'input' },
      { label: 'Storage Temperature', type: 'select' },
    ];

    for (const field of requiredFields) {
      const label = page.locator(`label:has-text("${field.label}")`);
      await expect(label).toBeVisible();
      
      // Check associated input exists
      const fieldId = await label.getAttribute('for');
      if (fieldId) {
        const input = page.locator(`#${fieldId}`);
        await expect(input).toBeVisible();
      }
    }
  });

  test('Should validate required fields', async ({ page }) => {
    // Try to submit empty form
    const submitButton = page.locator('button[type="submit"]');
    await submitButton.click();

    // Check for validation messages
    await expect(page.locator('text=/required|Please fill|cannot be empty/i')).toBeVisible();
  });

  test('Should upload document for processing', async ({ page }) => {
    // Find file upload input
    const fileInput = page.locator('input[type="file"]');
    await expect(fileInput).toBeVisible();

    // Create and upload test file
    const testContent = `
      Laboratory Submission
      Sample ID: LAB-20240115-TEST01
      Sample Type: DNA
      Volume: 100 µL
      Concentration: 50 ng/µL
    `;
    
    await fileInput.setInputFiles({
      name: 'test-submission.txt',
      mimeType: 'text/plain',
      buffer: Buffer.from(testContent),
    });

    // Check file was selected
    await expect(page.locator('text=/test-submission\.txt|File selected/i')).toBeVisible();

    // Click process/upload button
    const processButton = page.locator('button:has-text("Process"), button:has-text("Upload")');
    await processButton.click();

    // Check for processing indicator
    await expect(page.locator('text=/Processing|Uploading|Extracting/i')).toBeVisible();
  });

  test('Should display extracted information', async ({ page }) => {
    // Fill form with test data
    await page.fill('input[name="sampleId"], input[placeholder*="Sample ID"]', 'TEST-001');
    await page.selectOption('select[name="sampleType"]', 'DNA');
    await page.fill('input[name="volume"]', '100');
    await page.fill('input[name="concentration"]', '50');
    await page.selectOption('select[name="storageTemp"]', '-80');

    // Submit form
    await page.click('button[type="submit"]');

    // Wait for results
    await page.waitForSelector('text=/Results|Extracted|Summary/i', { timeout: 10000 });

    // Check extracted data is displayed
    await expect(page.locator('text=TEST-001')).toBeVisible();
    await expect(page.locator('text=DNA')).toBeVisible();
    await expect(page.locator('text=/100.*µL/')).toBeVisible();
  });

  test('Should show confidence scores', async ({ page }) => {
    // Upload a document
    const fileInput = page.locator('input[type="file"]');
    await fileInput.setInputFiles({
      name: 'ambiguous-data.txt',
      mimeType: 'text/plain',
      buffer: Buffer.from('Sample maybe DNA or RNA, about 50-100 units'),
    });

    await page.click('button:has-text("Process")');

    // Wait for results with confidence scores
    await page.waitForSelector('text=/Confidence|Certainty|Score/i');

    // Check confidence indicators
    const confidenceElement = page.locator('text=/\d+%|0\.\d+/');
    await expect(confidenceElement).toBeVisible();
  });

  test('Should handle multiple sample types', async ({ page }) => {
    // Check sample type dropdown options
    const sampleTypeSelect = page.locator('select[name="sampleType"]');
    await sampleTypeSelect.click();

    const expectedTypes = ['DNA', 'RNA', 'Protein', 'Blood', 'Tissue', 'Cell'];
    for (const sampleType of expectedTypes) {
      await expect(page.locator(`option:has-text("${sampleType}")`)).toBeVisible();
    }
  });

  test('Should support batch submission', async ({ page }) => {
    // Look for batch upload option
    const batchButton = page.locator('button:has-text("Batch"), label:has-text("Multiple")');
    
    if (await batchButton.isVisible()) {
      await batchButton.click();
      
      // Check for batch upload interface
      await expect(page.locator('text=/Upload multiple|Batch processing/i')).toBeVisible();
      
      // Should accept CSV or similar
      const batchInput = page.locator('input[type="file"][accept*="csv"]');
      await expect(batchInput).toBeVisible();
    }
  });

  test('@integration Should integrate with storage recommendations', async ({ page }) => {
    // Submit a sample
    await page.fill('input[name="sampleId"]', 'STORAGE-TEST-001');
    await page.selectOption('select[name="sampleType"]', 'RNA');
    await page.fill('input[name="volume"]', '50');
    
    await page.click('button[type="submit"]');

    // Wait for storage recommendations
    await page.waitForSelector('text=/Storage|Temperature|Location/i', { timeout: 10000 });

    // Check for storage suggestions
    await expect(page.locator('text=/-80°C|Ultra-low/i')).toBeVisible();
  });

  test('Should export submission data', async ({ page }) => {
    // Submit a form
    await page.fill('input[name="sampleId"]', 'EXPORT-001');
    await page.selectOption('select[name="sampleType"]', 'DNA');
    await page.click('button[type="submit"]');

    // Wait for results
    await page.waitForSelector('text=/Results|Complete/i');

    // Look for export options
    const exportButton = page.locator('button:has-text("Export"), button:has-text("Download")');
    await expect(exportButton).toBeVisible();

    // Click export and check download
    const downloadPromise = page.waitForEvent('download');
    await exportButton.click();
    
    const download = await downloadPromise;
    expect(download.suggestedFilename()).toMatch(/\.(json|csv|pdf)$/);
  });

  test('Should provide help tooltips', async ({ page }) => {
    // Look for help icons
    const helpIcons = page.locator('[aria-label*="help"], [title*="help"], .help-icon, text=?');
    
    if (await helpIcons.first().isVisible()) {
      // Hover over help icon
      await helpIcons.first().hover();
      
      // Check tooltip appears
      await expect(page.locator('.tooltip, [role="tooltip"]')).toBeVisible();
    }
  });

  test('Should be keyboard accessible', async ({ page }) => {
    // Tab through form fields
    await page.keyboard.press('Tab');
    
    // First focusable element should be highlighted
    const focusedElement = page.locator(':focus');
    await expect(focusedElement).toBeVisible();
    
    // Tab to next field
    await page.keyboard.press('Tab');
    const newFocusedElement = page.locator(':focus');
    await expect(newFocusedElement).toBeVisible();
    
    // Should be able to submit with Enter
    await page.fill('input[name="sampleId"]', 'KEYBOARD-TEST');
    await page.keyboard.press('Enter');
    
    // Check form was submitted or validation triggered
    await expect(page.locator('text=/Submit|Processing|required/i')).toBeVisible();
  });

  test('Should support dark mode', async ({ page }) => {
    // Look for theme toggle
    const themeToggle = page.locator('button[aria-label*="theme"], button:has-text("Dark"), .theme-toggle');
    
    if (await themeToggle.isVisible()) {
      // Get initial background color
      const body = page.locator('body');
      const initialBg = await body.evaluate(el => 
        window.getComputedStyle(el).backgroundColor
      );
      
      // Toggle theme
      await themeToggle.click();
      
      // Check background changed
      const newBg = await body.evaluate(el => 
        window.getComputedStyle(el).backgroundColor
      );
      
      expect(newBg).not.toBe(initialBg);
    }
  });
}); 