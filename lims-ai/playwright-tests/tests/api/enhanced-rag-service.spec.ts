import { test, expect, APIRequestContext } from '@playwright/test';

test.describe('Enhanced RAG Service API Tests', () => {
  let apiContext: APIRequestContext;

  test.beforeAll(async ({ playwright }) => {
    apiContext = await playwright.request.newContext({
      baseURL: 'http://localhost:8100',
      extraHTTPHeaders: {
        'Accept': 'application/json',
      },
    });
  });

  test.afterAll(async () => {
    await apiContext.dispose();
  });

  test('@smoke API health check should return 200', async () => {
    const response = await apiContext.get('/health');
    expect(response.status()).toBe(200);
    
    const body = await response.json();
    expect(body).toHaveProperty('status', 'healthy');
    expect(body).toHaveProperty('service', 'enhanced-rag-service');
  });

  test('Should get API documentation', async () => {
    const response = await apiContext.get('/docs');
    expect(response.status()).toBe(200);
    expect(response.headers()['content-type']).toContain('text/html');
  });

  test('Should upload and process a document', async ({ page }) => {
    // Create a test document
    const testContent = `
      Laboratory Submission Form
      Sample ID: TEST-001
      Sample Type: DNA
      Volume: 100 µL
      Concentration: 50 ng/µL
      Storage Temperature: -80°C
    `;
    
    const testFile = Buffer.from(testContent);
    
    // Upload document
    const uploadResponse = await apiContext.post('/api/v1/documents/upload', {
      multipart: {
        file: {
          name: 'test-submission.txt',
          mimeType: 'text/plain',
          buffer: testFile,
        },
        metadata: JSON.stringify({
          document_type: 'lab_submission',
          submitter: 'test_user',
        }),
      },
    });

    expect(uploadResponse.status()).toBe(200);
    const uploadResult = await uploadResponse.json();
    expect(uploadResult).toHaveProperty('document_id');
    expect(uploadResult).toHaveProperty('status', 'processing');

    const documentId = uploadResult.document_id;

    // Check processing status
    const statusResponse = await apiContext.get(`/api/v1/documents/${documentId}/status`);
    expect(statusResponse.status()).toBe(200);
    
    const status = await statusResponse.json();
    expect(status).toHaveProperty('status');
    expect(['processing', 'completed', 'failed']).toContain(status.status);
  });

  test('Should extract information from document using RAG', async () => {
    const extractRequest = {
      document_content: `
        Patient: John Doe
        Sample Type: Blood
        Collection Date: 2024-01-15
        Volume: 5 mL
        Analysis Required: Genomic Sequencing
      `,
      extraction_template: 'lab_submission',
    };

    const response = await apiContext.post('/api/v1/extract', {
      data: extractRequest,
    });

    expect(response.status()).toBe(200);
    const result = await response.json();
    
    expect(result).toHaveProperty('extracted_data');
    expect(result.extracted_data).toHaveProperty('sample_type');
    expect(result.extracted_data).toHaveProperty('volume');
    expect(result).toHaveProperty('confidence_score');
    expect(result.confidence_score).toBeGreaterThan(0);
    expect(result.confidence_score).toBeLessThanOrEqual(1);
  });

  test('Should perform similarity search', async () => {
    const searchRequest = {
      query: 'DNA extraction protocol for blood samples',
      limit: 5,
      threshold: 0.7,
    };

    const response = await apiContext.post('/api/v1/search', {
      data: searchRequest,
    });

    expect(response.status()).toBe(200);
    const results = await response.json();
    
    expect(results).toHaveProperty('results');
    expect(Array.isArray(results.results)).toBeTruthy();
    expect(results.results.length).toBeLessThanOrEqual(5);
    
    if (results.results.length > 0) {
      const firstResult = results.results[0];
      expect(firstResult).toHaveProperty('content');
      expect(firstResult).toHaveProperty('similarity_score');
      expect(firstResult).toHaveProperty('metadata');
      expect(firstResult.similarity_score).toBeGreaterThanOrEqual(0.7);
    }
  });

  test('Should handle Q&A queries', async () => {
    const qaRequest = {
      question: 'What is the recommended storage temperature for DNA samples?',
      context_type: 'laboratory_guidelines',
    };

    const response = await apiContext.post('/api/v1/qa', {
      data: qaRequest,
    });

    expect(response.status()).toBe(200);
    const answer = await response.json();
    
    expect(answer).toHaveProperty('answer');
    expect(answer).toHaveProperty('sources');
    expect(answer).toHaveProperty('confidence');
    expect(answer.answer).toBeTruthy();
    expect(Array.isArray(answer.sources)).toBeTruthy();
  });

  test('@integration Should process batch documents', async () => {
    const batchRequest = {
      documents: [
        {
          id: 'doc1',
          content: 'Sample 1: DNA, 100µL, -80°C storage',
        },
        {
          id: 'doc2',
          content: 'Sample 2: RNA, 50µL, -80°C storage',
        },
        {
          id: 'doc3',
          content: 'Sample 3: Protein, 200µL, -20°C storage',
        },
      ],
      processing_options: {
        parallel: true,
        extract_entities: true,
      },
    };

    const response = await apiContext.post('/api/v1/batch/process', {
      data: batchRequest,
    });

    expect(response.status()).toBe(202); // Accepted for processing
    const batchResult = await response.json();
    
    expect(batchResult).toHaveProperty('batch_id');
    expect(batchResult).toHaveProperty('status', 'queued');
    expect(batchResult).toHaveProperty('total_documents', 3);
  });

  test('Should retrieve vector embeddings', async () => {
    const embeddingRequest = {
      texts: [
        'DNA extraction from blood samples',
        'RNA sequencing protocol',
        'Protein purification methods',
      ],
      model: 'default',
    };

    const response = await apiContext.post('/api/v1/embeddings', {
      data: embeddingRequest,
    });

    expect(response.status()).toBe(200);
    const embeddings = await response.json();
    
    expect(embeddings).toHaveProperty('embeddings');
    expect(Array.isArray(embeddings.embeddings)).toBeTruthy();
    expect(embeddings.embeddings.length).toBe(3);
    
    // Check embedding dimensions
    embeddings.embeddings.forEach((embedding: number[]) => {
      expect(Array.isArray(embedding)).toBeTruthy();
      expect(embedding.length).toBeGreaterThan(0); // Should have dimensions
      embedding.forEach((value: number) => {
        expect(typeof value).toBe('number');
      });
    });
  });

  test('Should handle API errors gracefully', async () => {
    // Test with invalid document ID
    const response = await apiContext.get('/api/v1/documents/invalid-id-12345/status');
    expect(response.status()).toBe(404);
    
    const error = await response.json();
    expect(error).toHaveProperty('error');
    expect(error).toHaveProperty('message');
  });

  test('Should validate request data', async () => {
    // Test with invalid extraction request
    const invalidRequest = {
      // Missing required fields
      extraction_template: 'lab_submission',
    };

    const response = await apiContext.post('/api/v1/extract', {
      data: invalidRequest,
    });

    expect(response.status()).toBe(422); // Unprocessable Entity
    const error = await response.json();
    expect(error).toHaveProperty('detail');
  });

  test('Should support pagination for search results', async () => {
    const searchRequest = {
      query: 'sample storage protocols',
      page: 1,
      page_size: 10,
    };

    const response = await apiContext.post('/api/v1/search', {
      data: searchRequest,
    });

    expect(response.status()).toBe(200);
    const results = await response.json();
    
    expect(results).toHaveProperty('results');
    expect(results).toHaveProperty('total');
    expect(results).toHaveProperty('page');
    expect(results).toHaveProperty('page_size');
    expect(results).toHaveProperty('has_next');
    expect(results.results.length).toBeLessThanOrEqual(10);
  });
}); 