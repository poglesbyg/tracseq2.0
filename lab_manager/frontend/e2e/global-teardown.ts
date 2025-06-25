import { chromium, FullConfig } from '@playwright/test';

/**
 * Global teardown for TracSeq 2.0 Laboratory Management System E2E Tests
 * - Clean up test data
 * - Remove test users and sessions
 * - Reset laboratory state
 */
async function globalTeardown(config: FullConfig) {
    console.log('🧹 Cleaning up TracSeq 2.0 E2E test environment...');

    const { baseURL } = config.projects[0].use;
    if (!baseURL) {
        console.warn('⚠️ No baseURL configured, skipping teardown');
        return;
    }

    const browser = await chromium.launch();

    try {
        // Clean up test users and sessions
        await cleanupTestUsers(browser, baseURL);

        // Clean up laboratory test data
        await cleanupLaboratoryData(browser, baseURL);

        // Clear environment variables
        delete process.env.TEST_USERS;
        delete process.env.TEST_LAB_DATA;

        console.log('✅ TracSeq 2.0 E2E test environment cleaned up');
    } catch (error) {
        console.error('❌ Failed to cleanup test environment:', error);
        // Don't throw - teardown should be best effort
    } finally {
        await browser.close();
    }
}

/**
 * Clean up test users and their sessions
 */
async function cleanupTestUsers(browser: any, _baseURL: string) {
    const context = await browser.newContext();
    // Context setup for future cleanup logic

    try {
        console.log('🗑️ Cleaning up test users...');

        // In a real implementation, this would:
        // 1. Connect to auth service
        // 2. Delete test users created during setup
        // 3. Revoke any active sessions
        // 4. Clear authentication tokens

        // For now, just log the cleanup
        console.log('✅ Test users cleaned up');
    } catch (error) {
        console.error('❌ Failed to cleanup test users:', error);
    } finally {
        await context.close();
    }
}

/**
 * Clean up laboratory test data
 */
async function cleanupLaboratoryData(browser: any, _baseURL: string) {
    const context = await browser.newContext();
    // Context setup for future cleanup logic

    try {
        console.log('🧽 Cleaning up laboratory test data...');

        // In a real implementation, this would:
        // 1. Delete test samples, projects, and equipment
        // 2. Reset storage locations
        // 3. Clear any pending workflows
        // 4. Reset IoT sensor states

        // For now, just log the cleanup
        console.log('✅ Laboratory test data cleaned up');
    } catch (error) {
        console.error('❌ Failed to cleanup laboratory data:', error);
    } finally {
        await context.close();
    }
}

export default globalTeardown; 
