import { test, expect } from '@playwright/test';
import { AuthHelpers, LabHelpers, DataGenerators, WaitHelpers, AssertionHelpers } from '../utils/test-helpers';

test.describe('TracSeq 2.0 Laboratory - Sample Management', () => {
    let auth: AuthHelpers;
    let lab: LabHelpers;
    let wait: WaitHelpers;
    let assertions: AssertionHelpers;

    test.beforeEach(async ({ page }) => {
        auth = new AuthHelpers(page);
        lab = new LabHelpers(page);
        wait = new WaitHelpers(page);
        assertions = new AssertionHelpers(page);

        // Login as researcher for most tests
        await auth.loginAs('researcher');
    });

    test.afterEach(async () => {
        await auth.logout();
    });

    test('should display samples dashboard correctly', async ({ page }) => {
        await lab.goToSamples();

        await expect(page.locator('[data-testid="samples-dashboard"]')).toBeVisible();
        await expect(page.locator('[data-testid="samples-table"]')).toBeVisible();
        await expect(page.locator('[data-testid="create-sample-button"]')).toBeVisible();
        await expect(page.locator('[data-testid="search-input"]')).toBeVisible();
        await expect(page.locator('[data-testid="filter-controls"]')).toBeVisible();
    });

    test('should create a new DNA sample successfully', async ({ page }) => {
        const sampleName = DataGenerators.generateSampleName();

        await lab.createSample({
            name: sampleName,
            type: 'DNA',
            projectId: 'PROJ-001'
        });

        // Verify sample appears in list
        await lab.searchSamples(sampleName);
        await expect(page.locator(`[data-testid="sample-${sampleName}"]`)).toBeVisible();

        // Verify sample details
        await expect(page.locator(`[data-testid="sample-${sampleName}-type"]`)).toContainText('DNA');
        await expect(page.locator(`[data-testid="sample-${sampleName}-status"]`)).toContainText('Pending');
    });

    test('should create a new RNA sample successfully', async ({ page }) => {
        const sampleName = DataGenerators.generateSampleName();

        await lab.createSample({
            name: sampleName,
            type: 'RNA',
            projectId: 'PROJ-002'
        });

        // Verify sample creation
        await lab.searchSamples(sampleName);
        await expect(page.locator(`[data-testid="sample-${sampleName}-type"]`)).toContainText('RNA');
    });

    test('should validate required fields when creating sample', async ({ page }) => {
        await lab.goToSamples();
        await page.click('[data-testid="create-sample-button"]');

        // Try to submit without filling required fields
        await page.click('[data-testid="submit-sample-button"]');

        // Should show validation errors
        await expect(page.locator('[data-testid="sample-name-error"]')).toBeVisible();
        await expect(page.locator('[data-testid="sample-type-error"]')).toBeVisible();
        await expect(page.locator('[data-testid="project-error"]')).toBeVisible();
    });

    test('should update sample status through workflow', async ({ page }) => {
        const sampleName = DataGenerators.generateSampleName();

        // Create sample
        await lab.createSample({
            name: sampleName,
            type: 'DNA',
            projectId: 'PROJ-001'
        });

        // Navigate to sample details
        await lab.searchSamples(sampleName);
        await page.click(`[data-testid="sample-${sampleName}-details"]`);

        // Update status to Processing
        await page.click('[data-testid="update-status-button"]');
        await page.selectOption('[data-testid="status-select"]', 'processing');
        await page.click('[data-testid="confirm-status-change"]');

        await wait.waitForNotification('success');

        // Verify status updated
        await expect(page.locator('[data-testid="sample-status"]')).toContainText('Processing');

        // Verify audit trail
        await expect(page.locator('[data-testid="status-history"]')).toContainText('Processing');
        await expect(page.locator('[data-testid="status-history"]')).toContainText('Research Scientist');
    });

    test('should search samples by name', async ({ page }) => {
        const sampleName = DataGenerators.generateSampleName();

        // Create test sample
        await lab.createSample({
            name: sampleName,
            type: 'DNA',
            projectId: 'PROJ-001'
        });

        // Search for the sample
        await lab.searchSamples(sampleName);

        // Should find the sample
        await expect(page.locator(`[data-testid="sample-${sampleName}"]`)).toBeVisible();

        // Search for non-existent sample
        await lab.searchSamples('NonExistentSample');
        await expect(page.locator('[data-testid="no-samples-message"]')).toBeVisible();
    });

    test('should filter samples by status', async ({ page }) => {
        await lab.goToSamples();

        // Apply status filter
        await page.click('[data-testid="status-filter"]');
        await page.click('[data-testid="status-filter-pending"]');

        await wait.waitForLoading();

        // All visible samples should have 'Pending' status
        const statusElements = await page.locator('[data-testid$="-status"]').all();
        for (const element of statusElements) {
            await expect(element).toContainText('Pending');
        }
    });

    test('should filter samples by type', async ({ page }) => {
        await lab.goToSamples();

        // Apply type filter
        await page.click('[data-testid="type-filter"]');
        await page.click('[data-testid="type-filter-dna"]');

        await wait.waitForLoading();

        // All visible samples should be DNA type
        const typeElements = await page.locator('[data-testid$="-type"]').all();
        for (const element of typeElements) {
            await expect(element).toContainText('DNA');
        }
    });

    test('should assign samples to storage location', async ({ page }) => {
        const sampleName = DataGenerators.generateSampleName();

        // Create sample
        await lab.createSample({
            name: sampleName,
            type: 'DNA',
            projectId: 'PROJ-001'
        });

        // Navigate to sample details
        await lab.searchSamples(sampleName);
        await page.click(`[data-testid="sample-${sampleName}-details"]`);

        // Assign to storage
        await page.click('[data-testid="assign-storage-button"]');
        await page.selectOption('[data-testid="storage-location-select"]', 'STOR-001');
        await page.fill('[data-testid="storage-position-input"]', 'A1');
        await page.click('[data-testid="confirm-storage-assignment"]');

        await wait.waitForNotification('success');

        // Verify storage assignment
        await expect(page.locator('[data-testid="storage-location"]')).toContainText('Freezer -80°C Unit 1');
        await expect(page.locator('[data-testid="storage-position"]')).toContainText('A1');
    });

    test('should generate and assign barcodes', async ({ page }) => {
        const sampleName = DataGenerators.generateSampleName();

        // Create sample
        await lab.createSample({
            name: sampleName,
            type: 'DNA',
            projectId: 'PROJ-001'
        });

        // Navigate to sample details
        await lab.searchSamples(sampleName);
        await page.click(`[data-testid="sample-${sampleName}-details"]`);

        // Generate barcode
        await page.click('[data-testid="generate-barcode-button"]');

        await wait.waitForNotification('success');

        // Verify barcode generated
        const barcodeElement = page.locator('[data-testid="sample-barcode"]');
        await expect(barcodeElement).toBeVisible();

        const barcodeText = await barcodeElement.textContent();
        expect(barcodeText).toMatch(/^TRC-[A-Z0-9]{9}$/);

        // Verify barcode QR code
        await expect(page.locator('[data-testid="barcode-qr"]')).toBeVisible();
    });

    test('should handle sample batch operations', async ({ page }) => {
        const sampleNames = [
            DataGenerators.generateSampleName(),
            DataGenerators.generateSampleName(),
            DataGenerators.generateSampleName()
        ];

        // Create multiple samples
        for (const name of sampleNames) {
            await lab.createSample({
                name,
                type: 'DNA',
                projectId: 'PROJ-001'
            });
        }

        await lab.goToSamples();

        // Select multiple samples
        for (const name of sampleNames) {
            await page.check(`[data-testid="sample-${name}-checkbox"]`);
        }

        // Perform batch status update
        await page.click('[data-testid="batch-actions-button"]');
        await page.click('[data-testid="batch-update-status"]');
        await page.selectOption('[data-testid="batch-status-select"]', 'processing');
        await page.click('[data-testid="confirm-batch-update"]');

        await wait.waitForNotification('success');

        // Verify all samples updated
        for (const name of sampleNames) {
            await expect(page.locator(`[data-testid="sample-${name}-status"]`)).toContainText('Processing');
        }
    });

    test('should export sample data', async ({ page }) => {
        await lab.goToSamples();

        // Set up download handler
        const downloadPromise = page.waitForEvent('download');

        // Trigger export
        await page.click('[data-testid="export-samples-button"]');
        await page.click('[data-testid="export-csv"]');

        // Wait for download
        const download = await downloadPromise;
        expect(download.suggestedFilename()).toMatch(/samples.*\.csv$/);
    });

    test('should handle sample chain of custody', async ({ page }) => {
        const sampleName = DataGenerators.generateSampleName();

        // Create sample
        await lab.createSample({
            name: sampleName,
            type: 'DNA',
            projectId: 'PROJ-001'
        });

        // Navigate to sample details
        await lab.searchSamples(sampleName);
        await page.click(`[data-testid="sample-${sampleName}-details"]`);

        // View chain of custody
        await page.click('[data-testid="chain-of-custody-tab"]');

        // Verify initial entry
        await expect(page.locator('[data-testid="custody-log"]')).toBeVisible();
        await expect(page.locator('[data-testid="custody-log"]')).toContainText('Sample Created');
        await expect(page.locator('[data-testid="custody-log"]')).toContainText('Research Scientist');

        // Add custody entry
        await page.click('[data-testid="add-custody-entry"]');
        await page.fill('[data-testid="custody-action-input"]', 'Sample transferred to storage');
        await page.fill('[data-testid="custody-notes-input"]', 'Moved to -80°C freezer for preservation');
        await page.click('[data-testid="submit-custody-entry"]');

        await wait.waitForNotification('success');

        // Verify new entry
        await expect(page.locator('[data-testid="custody-log"]')).toContainText('Sample transferred to storage');
        await expect(page.locator('[data-testid="custody-log"]')).toContainText('-80°C freezer');
    });

    test('should handle sample quality control', async ({ page }) => {
        const sampleName = DataGenerators.generateSampleName();

        // Create sample
        await lab.createSample({
            name: sampleName,
            type: 'DNA',
            projectId: 'PROJ-001'
        });

        // Navigate to sample details
        await lab.searchSamples(sampleName);
        await page.click(`[data-testid="sample-${sampleName}-details"]`);

        // Add QC measurements
        await page.click('[data-testid="qc-tab"]');
        await page.click('[data-testid="add-qc-measurement"]');

        await page.fill('[data-testid="qc-concentration-input"]', '50.5');
        await page.fill('[data-testid="qc-purity-input"]', '1.8');
        await page.fill('[data-testid="qc-volume-input"]', '100');
        await page.selectOption('[data-testid="qc-instrument-select"]', 'Nanodrop');
        await page.click('[data-testid="submit-qc-measurement"]');

        await wait.waitForNotification('success');

        // Verify QC data
        await expect(page.locator('[data-testid="qc-concentration"]')).toContainText('50.5 ng/μL');
        await expect(page.locator('[data-testid="qc-purity"]')).toContainText('1.8');
        await expect(page.locator('[data-testid="qc-volume"]')).toContainText('100 μL');
    });

    test('should handle admin-only operations', async ({ page }) => {
        // Logout researcher and login as admin
        await auth.logout();
        await auth.loginAs('admin');

        await lab.goToSamples();

        // Admin should see additional controls
        await expect(page.locator('[data-testid="admin-controls"]')).toBeVisible();
        await expect(page.locator('[data-testid="bulk-import-button"]')).toBeVisible();
        await expect(page.locator('[data-testid="system-settings-button"]')).toBeVisible();

        // Test bulk import
        await page.click('[data-testid="bulk-import-button"]');
        await expect(page.locator('[data-testid="bulk-import-modal"]')).toBeVisible();

        // Admin can delete samples
        const sampleName = DataGenerators.generateSampleName();
        await lab.createSample({
            name: sampleName,
            type: 'DNA',
            projectId: 'PROJ-001'
        });

        await lab.searchSamples(sampleName);
        await page.click(`[data-testid="sample-${sampleName}-details"]`);

        await expect(page.locator('[data-testid="delete-sample-button"]')).toBeVisible();
    });

    test('should handle technician workflow', async ({ page }) => {
        // Logout researcher and login as technician
        await auth.logout();
        await auth.loginAs('technician');

        await lab.goToSamples();

        // Technician should see processing controls
        await expect(page.locator('[data-testid="processing-queue"]')).toBeVisible();
        await expect(page.locator('[data-testid="equipment-status"]')).toBeVisible();

        // Technician can update processing status
        const sampleName = DataGenerators.generateSampleName();
        await lab.createSample({
            name: sampleName,
            type: 'DNA',
            projectId: 'PROJ-001'
        });

        await lab.searchSamples(sampleName);
        await page.click(`[data-testid="sample-${sampleName}-details"]`);

        // Start processing
        await page.click('[data-testid="start-processing-button"]');
        await page.selectOption('[data-testid="equipment-select"]', 'SEQ-001');
        await page.click('[data-testid="confirm-processing"]');

        await wait.waitForNotification('success');

        // Verify processing started
        await expect(page.locator('[data-testid="sample-status"]')).toContainText('Processing');
        await expect(page.locator('[data-testid="assigned-equipment"]')).toContainText('Illumina NovaSeq 6000');
    });

    test('should validate sample data integrity', async ({ page }) => {
        const sampleName = DataGenerators.generateSampleName();

        // Create sample
        await lab.createSample({
            name: sampleName,
            type: 'DNA',
            projectId: 'PROJ-001'
        });

        // Navigate to sample details
        await lab.searchSamples(sampleName);
        await page.click(`[data-testid="sample-${sampleName}-details"]`);

        // Verify all timestamps are present and valid
        await expect(page.locator('[data-testid="created-at"]')).toBeVisible();
        await expect(page.locator('[data-testid="updated-at"]')).toBeVisible();

        // Verify sample ID is UUID format
        const sampleId = await page.locator('[data-testid="sample-id"]').textContent();
        expect(sampleId).toMatch(/^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$/);

        // Verify no console errors
        await assertions.assertNoConsoleErrors();
    });
}); 
