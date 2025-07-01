import { test, expect, APIRequestContext } from '@playwright/test';

test.describe('ML Platform Feature Store Tests', () => {
  let apiContext: APIRequestContext;

  test.beforeAll(async ({ playwright }) => {
    apiContext = await playwright.request.newContext({
      baseURL: 'http://localhost:8090',
      extraHTTPHeaders: {
        'Accept': 'application/json',
        'Content-Type': 'application/json',
      },
    });
  });

  test.afterAll(async () => {
    await apiContext.dispose();
  });

  test('@smoke Feature store health check', async () => {
    const response = await apiContext.get('/health');
    expect(response.status()).toBe(200);
    
    const body = await response.json();
    expect(body).toHaveProperty('status', 'healthy');
    expect(body).toHaveProperty('service', 'feature-store');
  });

  test('Should register a new feature set', async () => {
    const featureSet = {
      name: 'sample_quality_features',
      description: 'Features for sample quality prediction',
      features: [
        {
          name: 'sample_concentration',
          type: 'float',
          description: 'Sample concentration in ng/µL',
        },
        {
          name: 'sample_volume',
          type: 'float',
          description: 'Sample volume in µL',
        },
        {
          name: 'storage_temperature',
          type: 'categorical',
          description: 'Storage temperature category',
          categories: ['-80', '-20', '4', '25'],
        },
        {
          name: 'sample_age_days',
          type: 'integer',
          description: 'Days since sample collection',
        },
      ],
      entity: 'sample_id',
      tags: ['quality', 'prediction', 'laboratory'],
    };

    const response = await apiContext.post('/api/v1/feature-sets', {
      data: featureSet,
    });

    expect(response.status()).toBe(201);
    const result = await response.json();
    
    expect(result).toHaveProperty('feature_set_id');
    expect(result).toHaveProperty('created_at');
    expect(result.name).toBe(featureSet.name);
  });

  test('Should ingest feature data', async () => {
    const featureData = {
      feature_set: 'sample_quality_features',
      entity_id: 'SAMPLE-001',
      features: {
        sample_concentration: 45.6,
        sample_volume: 100,
        storage_temperature: '-80',
        sample_age_days: 7,
      },
      timestamp: new Date().toISOString(),
    };

    const response = await apiContext.post('/api/v1/features/ingest', {
      data: featureData,
    });

    expect(response.status()).toBe(200);
    const result = await response.json();
    
    expect(result).toHaveProperty('success', true);
    expect(result).toHaveProperty('records_ingested', 1);
  });

  test('Should retrieve features for entity', async () => {
    const response = await apiContext.get('/api/v1/features/sample_quality_features/SAMPLE-001');
    
    expect(response.status()).toBe(200);
    const features = await response.json();
    
    expect(features).toHaveProperty('entity_id', 'SAMPLE-001');
    expect(features).toHaveProperty('features');
    expect(features.features).toHaveProperty('sample_concentration');
    expect(features.features).toHaveProperty('sample_volume');
    expect(features.features).toHaveProperty('storage_temperature');
    expect(features.features).toHaveProperty('sample_age_days');
  });

  test('@integration Should compute feature statistics', async () => {
    // First, ingest multiple samples
    const samples = Array.from({ length: 10 }, (_, i) => ({
      feature_set: 'sample_quality_features',
      entity_id: `SAMPLE-${String(i + 1).padStart(3, '0')}`,
      features: {
        sample_concentration: 30 + Math.random() * 40,
        sample_volume: 50 + Math.random() * 150,
        storage_temperature: ['-80', '-20'][Math.floor(Math.random() * 2)],
        sample_age_days: Math.floor(Math.random() * 30),
      },
      timestamp: new Date().toISOString(),
    }));

    // Batch ingest
    const ingestResponse = await apiContext.post('/api/v1/features/batch-ingest', {
      data: { records: samples },
    });
    expect(ingestResponse.status()).toBe(200);

    // Get statistics
    const statsResponse = await apiContext.get('/api/v1/feature-sets/sample_quality_features/statistics');
    expect(statsResponse.status()).toBe(200);
    
    const stats = await statsResponse.json();
    expect(stats).toHaveProperty('feature_statistics');
    
    const concentrationStats = stats.feature_statistics.sample_concentration;
    expect(concentrationStats).toHaveProperty('mean');
    expect(concentrationStats).toHaveProperty('std');
    expect(concentrationStats).toHaveProperty('min');
    expect(concentrationStats).toHaveProperty('max');
    expect(concentrationStats).toHaveProperty('count');
  });

  test('Should support time-travel queries', async () => {
    const yesterday = new Date();
    yesterday.setDate(yesterday.getDate() - 1);
    
    const response = await apiContext.get('/api/v1/features/sample_quality_features/SAMPLE-001', {
      params: {
        timestamp: yesterday.toISOString(),
      },
    });

    expect(response.status()).toBe(200);
    const historicalFeatures = await response.json();
    
    expect(historicalFeatures).toHaveProperty('entity_id', 'SAMPLE-001');
    expect(historicalFeatures).toHaveProperty('timestamp');
    expect(historicalFeatures).toHaveProperty('features');
  });

  test('Should create feature view', async () => {
    const featureView = {
      name: 'sample_quality_prediction_view',
      feature_sets: ['sample_quality_features'],
      features: [
        'sample_concentration',
        'sample_volume',
        'storage_temperature',
        'sample_age_days',
      ],
      filters: {
        sample_age_days: { max: 30 },
        sample_concentration: { min: 10 },
      },
      description: 'Features for ML model training',
    };

    const response = await apiContext.post('/api/v1/feature-views', {
      data: featureView,
    });

    expect(response.status()).toBe(201);
    const result = await response.json();
    
    expect(result).toHaveProperty('view_id');
    expect(result).toHaveProperty('created_at');
    expect(result.name).toBe(featureView.name);
  });

  test('Should materialize features for training', async () => {
    const materializationRequest = {
      feature_view: 'sample_quality_prediction_view',
      entity_ids: ['SAMPLE-001', 'SAMPLE-002', 'SAMPLE-003'],
      start_time: new Date(Date.now() - 7 * 24 * 60 * 60 * 1000).toISOString(), // 7 days ago
      end_time: new Date().toISOString(),
      format: 'parquet',
    };

    const response = await apiContext.post('/api/v1/features/materialize', {
      data: materializationRequest,
    });

    expect(response.status()).toBe(202); // Accepted
    const result = await response.json();
    
    expect(result).toHaveProperty('job_id');
    expect(result).toHaveProperty('status', 'processing');
    expect(result).toHaveProperty('output_path');
  });

  test('Should monitor feature drift', async () => {
    const driftRequest = {
      feature_set: 'sample_quality_features',
      reference_period: {
        start: new Date(Date.now() - 30 * 24 * 60 * 60 * 1000).toISOString(),
        end: new Date(Date.now() - 7 * 24 * 60 * 60 * 1000).toISOString(),
      },
      comparison_period: {
        start: new Date(Date.now() - 7 * 24 * 60 * 60 * 1000).toISOString(),
        end: new Date().toISOString(),
      },
      features: ['sample_concentration', 'sample_volume'],
    };

    const response = await apiContext.post('/api/v1/features/drift-analysis', {
      data: driftRequest,
    });

    expect(response.status()).toBe(200);
    const driftReport = await response.json();
    
    expect(driftReport).toHaveProperty('drift_detected');
    expect(driftReport).toHaveProperty('feature_drift_scores');
    expect(driftReport.feature_drift_scores).toHaveProperty('sample_concentration');
    expect(driftReport.feature_drift_scores).toHaveProperty('sample_volume');
  });

  test('Should handle concurrent feature updates', async () => {
    const entityId = 'SAMPLE-CONCURRENT';
    
    // Send multiple concurrent updates
    const updates = Array.from({ length: 5 }, (_, i) => ({
      feature_set: 'sample_quality_features',
      entity_id: entityId,
      features: {
        sample_concentration: 40 + i,
        sample_volume: 100 + i * 10,
        storage_temperature: '-80',
        sample_age_days: i,
      },
      timestamp: new Date().toISOString(),
    }));

    const promises = updates.map(update => 
      apiContext.post('/api/v1/features/ingest', { data: update })
    );

    const responses = await Promise.all(promises);
    
    // All should succeed
    responses.forEach(response => {
      expect(response.status()).toBe(200);
    });

    // Verify latest value
    const latestResponse = await apiContext.get(`/api/v1/features/sample_quality_features/${entityId}`);
    expect(latestResponse.status()).toBe(200);
  });

  test('Should export feature metadata', async () => {
    const response = await apiContext.get('/api/v1/feature-sets/export');
    
    expect(response.status()).toBe(200);
    const exportData = await response.json();
    
    expect(exportData).toHaveProperty('feature_sets');
    expect(Array.isArray(exportData.feature_sets)).toBeTruthy();
    expect(exportData).toHaveProperty('feature_views');
    expect(exportData).toHaveProperty('export_timestamp');
  });
}); 