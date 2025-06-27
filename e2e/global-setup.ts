import { chromium, FullConfig } from '@playwright/test';

/**
 * Global setup for TracSeq 2.0 E2E Tests
 * Ensures all microservices are running and healthy before tests start
 */

const SERVICES = [
  { name: 'Database', url: 'http://localhost:5432', skipHealthCheck: true },
  { name: 'Redis', url: 'http://localhost:6380', skipHealthCheck: true },
  { name: 'Lab Manager Backend', url: 'http://localhost:3000/health' },
  { name: 'API Gateway', url: 'http://localhost:8089/health' },
  { name: 'RAG Service', url: 'http://localhost:8000/health' },
  { name: 'Frontend', url: 'http://localhost:5176', skipHealthCheck: true },
];

async function checkServiceHealth(service: { name: string; url: string; skipHealthCheck?: boolean }) {
  if (service.skipHealthCheck) {
    console.log(`⏭️  Skipping health check for ${service.name}`);
    return true;
  }

  try {
    const response = await fetch(service.url, { 
      method: 'GET',
      signal: AbortSignal.timeout(5000)
    });
    
    if (response.ok) {
      const data = await response.json();
      console.log(`✅ ${service.name} is healthy:`, data.status || 'OK');
      return true;
    } else {
      console.log(`❌ ${service.name} returned status: ${response.status}`);
      return false;
    }
  } catch (error) {
    console.log(`❌ ${service.name} is not responding:`, error.message);
    return false;
  }
}

async function waitForServices(maxAttempts = 30, delayMs = 2000) {
  console.log('🔍 Checking microservices health...');
  
  for (let attempt = 1; attempt <= maxAttempts; attempt++) {
    console.log(`\nAttempt ${attempt}/${maxAttempts}:`);
    
    const healthChecks = await Promise.all(
      SERVICES.map(service => checkServiceHealth(service))
    );
    
    const allHealthy = healthChecks.every(Boolean);
    
    if (allHealthy) {
      console.log('\n🎉 All services are healthy! Starting tests...\n');
      return true;
    }
    
    if (attempt < maxAttempts) {
      console.log(`⏳ Waiting ${delayMs}ms before next check...`);
      await new Promise(resolve => setTimeout(resolve, delayMs));
    }
  }
  
  console.log('\n❌ Some services are not responding. Tests may fail.\n');
  return false;
}

async function setupTestData() {
  console.log('🗄️  Setting up test data...');
  
  try {
    // Check if we can connect to the API Gateway
    const response = await fetch('http://localhost:8089/api/dashboard/stats');
    if (response.ok) {
      console.log('✅ API Gateway connection verified');
    }
    
    // Could add test user creation, sample data setup, etc.
    // For now, we'll rely on the existing data
    
  } catch (error) {
    console.log('⚠️  Could not set up test data:', error.message);
  }
}

async function globalSetup(config: FullConfig) {
  console.log('🚀 TracSeq 2.0 E2E Test Setup Starting...\n');
  
  // Wait for services to be healthy
  const servicesReady = await waitForServices();
  
  if (!servicesReady && process.env.CI) {
    throw new Error('Services are not ready in CI environment');
  }
  
  // Set up test data
  await setupTestData();
  
  // Create a browser instance for shared state if needed
  const browser = await chromium.launch();
  const context = await browser.newContext();
  const page = await context.newPage();
  
  // Pre-authenticate or set up global state if needed
  try {
    await page.goto('http://localhost:5176');
    console.log('✅ Frontend is accessible');
  } catch (error) {
    console.log('⚠️  Frontend may not be accessible:', error.message);
  }
  
  await browser.close();
  
  console.log('✅ Global setup completed!\n');
}

export default globalSetup; 