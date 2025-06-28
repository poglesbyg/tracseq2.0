import { test, expect } from '@playwright/test';

test.describe('API Gateway Integration', () => {
  const API_BASE = process.env.API_GATEWAY_URL || 'http://localhost:8089';

  test('should return gateway health status', async ({ request }) => {
    const response = await request.get(`${API_BASE}/health`);
    
    expect(response.ok()).toBeTruthy();
    
    const data = await response.json();
    expect(data).toHaveProperty('status', 'healthy');
    expect(data).toHaveProperty('service');
    expect(data).toHaveProperty('version');
    expect(data).toHaveProperty('timestamp');
  });

  test('should route dashboard stats correctly', async ({ request }) => {
    const response = await request.get(`${API_BASE}/api/dashboard/stats`);
    
    expect(response.ok()).toBeTruthy();
    
    const data = await response.json();
    expect(data).toHaveProperty('totalTemplates');
    expect(data).toHaveProperty('totalSamples');
    expect(data).toHaveProperty('pendingSequencing');
    expect(data).toHaveProperty('completedSequencing');
    
    // Verify data types
    expect(typeof data.totalTemplates).toBe('number');
    expect(typeof data.totalSamples).toBe('number');
    expect(typeof data.pendingSequencing).toBe('number');
    expect(typeof data.completedSequencing).toBe('number');
  });

  test('should handle templates API routing', async ({ request }) => {
    const response = await request.get(`${API_BASE}/api/templates`);
    
    expect(response.ok()).toBeTruthy();
    
    const data = await response.json();
    expect(Array.isArray(data)).toBeTruthy();
  });

  test('should handle samples API routing', async ({ request }) => {
    const response = await request.get(`${API_BASE}/api/samples`);
    
    expect(response.ok()).toBeTruthy();
    
    const data = await response.json();
    expect(Array.isArray(data)).toBeTruthy();
  });

  test('should handle sequencing jobs API routing', async ({ request }) => {
    const response = await request.get(`${API_BASE}/api/sequencing/jobs`);
    
    expect(response.ok()).toBeTruthy();
    
    const data = await response.json();
    expect(Array.isArray(data)).toBeTruthy();
  });

  test('should handle CORS headers correctly', async ({ request }) => {
    const response = await request.get(`${API_BASE}/api/dashboard/stats`, {
      headers: {
        'Origin': 'http://localhost:5173'
      }
    });
    
    expect(response.ok()).toBeTruthy();
    
    const headers = response.headers();
    expect(headers['access-control-allow-origin']).toBeDefined();
  });

  test('should handle authentication endpoints', async ({ request }) => {
    // Test login endpoint (should fail without credentials)
    const loginResponse = await request.post(`${API_BASE}/api/auth/login`, {
      data: {}
    });
    
    // Should return 400 or 422 for missing credentials
    expect([400, 422, 401].includes(loginResponse.status())).toBeTruthy();
  });

  test('should return 404 for non-existent endpoints', async ({ request }) => {
    const response = await request.get(`${API_BASE}/api/nonexistent`);
    
    expect(response.status()).toBe(404);
  });

  test('should handle rate limiting gracefully', async ({ request }) => {
    // Make multiple rapid requests
    const promises = Array(10).fill(null).map(() => 
      request.get(`${API_BASE}/api/dashboard/stats`)
    );
    
    const responses = await Promise.all(promises);
    
    // At least some should succeed
    const successCount = responses.filter(r => r.ok()).length;
    expect(successCount).toBeGreaterThan(0);
  });

  test('should proxy requests with proper headers', async ({ request }) => {
    const response = await request.get(`${API_BASE}/api/dashboard/stats`, {
      headers: {
        'X-Test-Header': 'test-value'
      }
    });
    
    expect(response.ok()).toBeTruthy();
  });

  test('should handle large responses efficiently', async ({ request }) => {
    const startTime = Date.now();
    
    const response = await request.get(`${API_BASE}/api/samples`);
    
    const endTime = Date.now();
    const duration = endTime - startTime;
    
    expect(response.ok()).toBeTruthy();
    
    // Response should be reasonably fast (under 5 seconds)
    expect(duration).toBeLessThan(5000);
  });
}); 