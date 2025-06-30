// TracSeq 2.0 Load Test with k6
// Tests the system under normal expected load

import http from 'k6/http';
import { check, sleep, group } from 'k6';
import { Rate, Trend, Counter } from 'k6/metrics';
import { randomItem, randomString, randomIntBetween } from 'https://jslib.k6.io/k6-utils/1.2.0/index.js';

// Custom metrics
const errorRate = new Rate('errors');
const sampleCreationDuration = new Trend('sample_creation_duration');
const authDuration = new Trend('auth_duration');
const searchDuration = new Trend('search_duration');
const successfulSamples = new Counter('successful_samples');

// Configuration
const BASE_URL = __ENV.BASE_URL || 'http://localhost:8089'; // API Gateway

export const options = {
  stages: [
    { duration: '2m', target: 10 },   // Ramp up to 10 users
    { duration: '5m', target: 50 },   // Ramp up to 50 users
    { duration: '10m', target: 100 }, // Stay at 100 users
    { duration: '5m', target: 50 },   // Ramp down to 50 users
    { duration: '2m', target: 0 },    // Ramp down to 0 users
  ],
  thresholds: {
    http_req_duration: ['p(95)<500', 'p(99)<1000'], // 95% of requests under 500ms
    http_req_failed: ['rate<0.05'],                  // Error rate under 5%
    errors: ['rate<0.05'],                            // Custom error rate under 5%
    sample_creation_duration: ['p(95)<1000'],         // Sample creation under 1s
    auth_duration: ['p(95)<200'],                     // Auth under 200ms
    search_duration: ['p(95)<300'],                   // Search under 300ms
  },
};

// Test data
const sampleTypes = ['blood', 'tissue', 'dna', 'rna', 'plasma'];
const storageConditions = ['frozen', 'refrigerated', 'room_temperature'];
const priorities = ['normal', 'urgent', 'stat'];

// Helper function to generate auth token
function authenticate(username, password) {
  const authStart = new Date();
  const response = http.post(`${BASE_URL}/auth/login`, JSON.stringify({
    username: username,
    password: password,
  }), {
    headers: { 'Content-Type': 'application/json' },
  });
  
  authDuration.add(new Date() - authStart);
  
  const success = check(response, {
    'auth successful': (r) => r.status === 200,
    'auth token received': (r) => r.json('access_token') !== undefined,
  });
  
  if (!success) {
    errorRate.add(1);
    return null;
  }
  
  errorRate.add(0);
  return response.json('access_token');
}

// Helper function to create headers with auth
function authHeaders(token) {
  return {
    'Content-Type': 'application/json',
    'Authorization': `Bearer ${token}`,
  };
}

// Main test scenario
export default function() {
  // Authenticate once per VU iteration
  const token = authenticate('test_user', 'test_password');
  if (!token) {
    console.error('Authentication failed');
    return;
  }

  // Sample Creation Workflow
  group('Sample Creation Workflow', () => {
    const sampleStart = new Date();
    
    // Create a new sample
    const sampleData = {
      name: `SAMPLE-${randomString(8)}`,
      sample_type: randomItem(sampleTypes),
      volume_ml: randomIntBetween(1, 10),
      collection_date: new Date().toISOString(),
      patient_id: `PAT-${randomString(6)}`,
      storage_condition: randomItem(storageConditions),
      priority: randomItem(priorities),
      metadata: {
        collected_by: `USER-${randomString(4)}`,
        collection_site: randomItem(['Lab A', 'Lab B', 'Lab C']),
        notes: 'Load test sample',
      },
    };
    
    const createResponse = http.post(
      `${BASE_URL}/samples`,
      JSON.stringify(sampleData),
      { headers: authHeaders(token) }
    );
    
    const createSuccess = check(createResponse, {
      'sample created': (r) => r.status === 201,
      'sample ID returned': (r) => r.json('id') !== undefined,
    });
    
    if (createSuccess) {
      successfulSamples.add(1);
      const sampleId = createResponse.json('id');
      
      // Update sample status
      sleep(1);
      const updateResponse = http.patch(
        `${BASE_URL}/samples/${sampleId}/status`,
        JSON.stringify({ status: 'processing' }),
        { headers: authHeaders(token) }
      );
      
      check(updateResponse, {
        'sample status updated': (r) => r.status === 200,
      });
      
      // Retrieve sample details
      sleep(0.5);
      const getResponse = http.get(
        `${BASE_URL}/samples/${sampleId}`,
        { headers: authHeaders(token) }
      );
      
      check(getResponse, {
        'sample retrieved': (r) => r.status === 200,
        'sample data complete': (r) => r.json('sample_type') !== undefined,
      });
    }
    
    sampleCreationDuration.add(new Date() - sampleStart);
  });
  
  // Search Operations
  group('Search Operations', () => {
    const searchStart = new Date();
    
    // Search for samples
    const searchParams = new URLSearchParams({
      sample_type: randomItem(sampleTypes),
      limit: 20,
      offset: 0,
    });
    
    const searchResponse = http.get(
      `${BASE_URL}/samples/search?${searchParams}`,
      { headers: authHeaders(token) }
    );
    
    check(searchResponse, {
      'search successful': (r) => r.status === 200,
      'search results returned': (r) => Array.isArray(r.json('items')),
    });
    
    searchDuration.add(new Date() - searchStart);
  });
  
  // Template Operations
  group('Template Operations', () => {
    // List templates
    const templatesResponse = http.get(
      `${BASE_URL}/templates`,
      { headers: authHeaders(token) }
    );
    
    check(templatesResponse, {
      'templates listed': (r) => r.status === 200,
      'templates array returned': (r) => Array.isArray(r.json()),
    });
    
    // Get a specific template if any exist
    const templates = templatesResponse.json();
    if (templates && templates.length > 0) {
      const templateId = randomItem(templates).id;
      const templateResponse = http.get(
        `${BASE_URL}/templates/${templateId}`,
        { headers: authHeaders(token) }
      );
      
      check(templateResponse, {
        'template retrieved': (r) => r.status === 200,
      });
    }
  });
  
  // Storage Operations
  group('Storage Operations', () => {
    // Get storage locations
    const storageResponse = http.get(
      `${BASE_URL}/storage/locations`,
      { headers: authHeaders(token) }
    );
    
    check(storageResponse, {
      'storage locations listed': (r) => r.status === 200,
    });
    
    // Check storage capacity
    const capacityResponse = http.get(
      `${BASE_URL}/storage/capacity`,
      { headers: authHeaders(token) }
    );
    
    check(capacityResponse, {
      'storage capacity retrieved': (r) => r.status === 200,
      'capacity data present': (r) => r.json('total_capacity') !== undefined,
    });
  });
  
  // Sequencing Workflow
  group('Sequencing Workflow', () => {
    // Create sequencing run
    const sequencingData = {
      name: `SEQ-RUN-${randomString(6)}`,
      sequencer_id: `SEQ-${randomIntBetween(1, 5)}`,
      run_type: randomItem(['WGS', 'WES', 'RNA-Seq', 'ChIP-Seq']),
      samples: [`SAMPLE-${randomString(8)}`],
      parameters: {
        read_length: randomItem([75, 150, 300]),
        paired_end: true,
        coverage: randomItem([30, 60, 100]),
      },
    };
    
    const seqResponse = http.post(
      `${BASE_URL}/sequencing/runs`,
      JSON.stringify(sequencingData),
      { headers: authHeaders(token) }
    );
    
    check(seqResponse, {
      'sequencing run created': (r) => r.status === 201 || r.status === 200,
    });
  });
  
  // Notification Check
  group('Notification Operations', () => {
    // Get user notifications
    const notificationsResponse = http.get(
      `${BASE_URL}/notifications/unread`,
      { headers: authHeaders(token) }
    );
    
    check(notificationsResponse, {
      'notifications retrieved': (r) => r.status === 200,
      'notifications array': (r) => Array.isArray(r.json()),
    });
  });
  
  // Think time between iterations
  sleep(randomIntBetween(2, 5));
}

// Lifecycle hooks
export function setup() {
  console.log('Starting TracSeq 2.0 load test...');
  
  // Create test user if needed
  const adminToken = authenticate('admin', 'admin_password');
  if (adminToken) {
    http.post(
      `${BASE_URL}/auth/users`,
      JSON.stringify({
        username: 'test_user',
        password: 'test_password',
        email: 'test@tracseq.io',
        role: 'lab_technician',
      }),
      { headers: authHeaders(adminToken) }
    );
  }
  
  return { adminToken };
}

export function teardown(data) {
  console.log('Load test completed');
  
  // Cleanup test data if needed
  if (data.adminToken) {
    // Could delete test user here
  }
}

// Handle test summary
export function handleSummary(data) {
  return {
    'load-test-summary.json': JSON.stringify(data),
    stdout: textSummary(data, { indent: ' ', enableColors: true }),
  };
}

function textSummary(data, options) {
  // Simple text summary for console output
  const { metrics } = data;
  let summary = '\n=== TracSeq 2.0 Load Test Summary ===\n\n';
  
  summary += `Total Requests: ${metrics.http_reqs.values.count}\n`;
  summary += `Request Rate: ${metrics.http_reqs.values.rate.toFixed(2)}/s\n`;
  summary += `Failed Requests: ${(metrics.http_req_failed.values.rate * 100).toFixed(2)}%\n`;
  summary += `Avg Response Time: ${metrics.http_req_duration.values.avg.toFixed(2)}ms\n`;
  summary += `P95 Response Time: ${metrics.http_req_duration.values['p(95)'].toFixed(2)}ms\n`;
  summary += `Successful Samples Created: ${metrics.successful_samples.values.count}\n`;
  
  return summary;
}