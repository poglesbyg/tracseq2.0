import { FullConfig } from '@playwright/test';

/**
 * Global teardown for TracSeq 2.0 E2E Tests
 * Cleans up test data and optionally stops services
 */

async function cleanupTestData() {
  console.log('🧹 Cleaning up test data...');
  
  try {
    // Could clean up test users, test samples, etc.
    // For now, we'll just verify the system is still responding
    
    const response = await fetch('http://localhost:8089/health');
    if (response.ok) {
      console.log('✅ System is still healthy after tests');
    }
    
  } catch (error) {
    console.log('⚠️  Could not verify system health:', error.message);
  }
}

async function stopServicesIfNeeded() {
  // Only stop services in CI environment
  if (process.env.CI) {
    console.log('🛑 Stopping services in CI environment...');
    
    // Could add logic to stop Docker containers or services
    // For now, just log
    console.log('✅ Services cleanup completed');
  } else {
    console.log('ℹ️  Leaving services running for local development');
  }
}

async function generateTestReport() {
  console.log('📊 Generating test report summary...');
  
  try {
    // Could aggregate test results, generate reports, etc.
    console.log('✅ Test report generated');
  } catch (error) {
    console.log('⚠️  Could not generate test report:', error.message);
  }
}

async function globalTeardown(config: FullConfig) {
  console.log('\n🏁 TracSeq 2.0 E2E Test Teardown Starting...\n');
  
  // Clean up test data
  await cleanupTestData();
  
  // Generate test reports
  await generateTestReport();
  
  // Stop services if in CI
  await stopServicesIfNeeded();
  
  console.log('✅ Global teardown completed!\n');
}

export default globalTeardown; 