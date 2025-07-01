import { test, expect } from '@playwright/test';
import WebSocket from 'ws';

test.describe('MCP Proxy WebSocket Tests', () => {
  let ws: WebSocket;
  const WS_URL = 'ws://localhost:9500';

  test.afterEach(async () => {
    if (ws && ws.readyState === WebSocket.OPEN) {
      ws.close();
    }
  });

  test('@smoke Should connect to WebSocket server', async () => {
    await new Promise<void>((resolve, reject) => {
      ws = new WebSocket(WS_URL);
      
      ws.on('open', () => {
        expect(ws.readyState).toBe(WebSocket.OPEN);
        resolve();
      });
      
      ws.on('error', (err) => {
        reject(err);
      });
    });
  });

  test('Should receive welcome message on connection', async () => {
    await new Promise<void>((resolve, reject) => {
      ws = new WebSocket(WS_URL);
      
      ws.on('open', () => {
        // Server should send a welcome message
      });
      
      ws.on('message', (data) => {
        const message = JSON.parse(data.toString());
        expect(message).toHaveProperty('type');
        expect(['welcome', 'connected', 'ready']).toContain(message.type);
        resolve();
      });
      
      ws.on('error', reject);
    });
  });

  test('Should handle service discovery', async () => {
    await new Promise<void>((resolve, reject) => {
      ws = new WebSocket(WS_URL);
      
      ws.on('open', () => {
        // Request available services
        ws.send(JSON.stringify({
          id: 'test-001',
          method: 'discover_services',
          params: {}
        }));
      });
      
      ws.on('message', (data) => {
        const response = JSON.parse(data.toString());
        
        if (response.id === 'test-001') {
          expect(response).toHaveProperty('result');
          expect(response.result).toHaveProperty('services');
          expect(Array.isArray(response.result.services)).toBeTruthy();
          
          // Should include known services
          const serviceNames = response.result.services.map(s => s.name);
          expect(serviceNames).toContain('cognitive_assistant');
          expect(serviceNames).toContain('rag_service');
          
          resolve();
        }
      });
      
      ws.on('error', reject);
    });
  });

  test('Should route messages to services', async () => {
    await new Promise<void>((resolve, reject) => {
      ws = new WebSocket(WS_URL);
      
      ws.on('open', () => {
        // Send a message to cognitive assistant
        ws.send(JSON.stringify({
          id: 'test-002',
          method: 'route_to_service',
          params: {
            service: 'cognitive_assistant',
            method: 'process_query',
            data: {
              query: 'What is the storage temperature for DNA samples?'
            }
          }
        }));
      });
      
      ws.on('message', (data) => {
        const response = JSON.parse(data.toString());
        
        if (response.id === 'test-002') {
          expect(response).toHaveProperty('result');
          expect(response.error).toBeUndefined();
          resolve();
        }
      });
      
      ws.on('error', reject);
    });
  });

  test('Should handle workflow orchestration', async () => {
    await new Promise<void>((resolve, reject) => {
      ws = new WebSocket(WS_URL);
      
      ws.on('open', () => {
        // Start a workflow
        ws.send(JSON.stringify({
          id: 'test-003',
          method: 'start_workflow',
          params: {
            workflow_id: 'sample_processing',
            steps: [
              {
                service: 'rag_service',
                method: 'extract_info',
                data: { document: 'Sample ID: TEST-WF-001' }
              },
              {
                service: 'storage_optimizer',
                method: 'recommend_storage',
                data: { sample_type: 'DNA' }
              }
            ]
          }
        }));
      });
      
      ws.on('message', (data) => {
        const response = JSON.parse(data.toString());
        
        if (response.id === 'test-003') {
          expect(response).toHaveProperty('result');
          expect(response.result).toHaveProperty('workflow_id');
          expect(response.result).toHaveProperty('status');
          expect(['started', 'running', 'completed']).toContain(response.result.status);
          resolve();
        }
      });
      
      ws.on('error', reject);
    });
  });

  test('@integration Should support transactions', async () => {
    let transactionId: string;
    
    await new Promise<void>((resolve, reject) => {
      ws = new WebSocket(WS_URL);
      
      ws.on('open', () => {
        // Begin transaction
        ws.send(JSON.stringify({
          id: 'test-004',
          method: 'begin_transaction',
          params: {
            services: ['rag_service', 'storage_optimizer']
          }
        }));
      });
      
      ws.on('message', async (data) => {
        const response = JSON.parse(data.toString());
        
        if (response.id === 'test-004') {
          expect(response.result).toHaveProperty('transaction_id');
          transactionId = response.result.transaction_id;
          
          // Execute operations in transaction
          ws.send(JSON.stringify({
            id: 'test-005',
            method: 'execute_in_transaction',
            params: {
              transaction_id: transactionId,
              operations: [
                {
                  service: 'rag_service',
                  method: 'store_document',
                  data: { doc_id: 'TX-001', content: 'Test document' }
                }
              ]
            }
          }));
        } else if (response.id === 'test-005') {
          // Commit transaction
          ws.send(JSON.stringify({
            id: 'test-006',
            method: 'commit_transaction',
            params: { transaction_id: transactionId }
          }));
        } else if (response.id === 'test-006') {
          expect(response.result).toHaveProperty('status', 'committed');
          resolve();
        }
      });
      
      ws.on('error', reject);
    });
  });

  test('Should handle service health monitoring', async () => {
    await new Promise<void>((resolve, reject) => {
      ws = new WebSocket(WS_URL);
      
      ws.on('open', () => {
        // Subscribe to health updates
        ws.send(JSON.stringify({
          id: 'test-007',
          method: 'subscribe_health',
          params: {
            services: ['cognitive_assistant', 'rag_service']
          }
        }));
      });
      
      let subscriptionConfirmed = false;
      
      ws.on('message', (data) => {
        const response = JSON.parse(data.toString());
        
        if (response.id === 'test-007') {
          expect(response.result).toHaveProperty('subscribed', true);
          subscriptionConfirmed = true;
        } else if (response.method === 'health_update' && subscriptionConfirmed) {
          expect(response.params).toHaveProperty('service');
          expect(response.params).toHaveProperty('status');
          expect(response.params).toHaveProperty('timestamp');
          resolve();
        }
      });
      
      ws.on('error', reject);
    });
  });

  test('Should handle errors gracefully', async () => {
    await new Promise<void>((resolve, reject) => {
      ws = new WebSocket(WS_URL);
      
      ws.on('open', () => {
        // Send invalid request
        ws.send(JSON.stringify({
          id: 'test-008',
          method: 'invalid_method',
          params: {}
        }));
      });
      
      ws.on('message', (data) => {
        const response = JSON.parse(data.toString());
        
        if (response.id === 'test-008') {
          expect(response).toHaveProperty('error');
          expect(response.error).toHaveProperty('code');
          expect(response.error).toHaveProperty('message');
          resolve();
        }
      });
      
      ws.on('error', reject);
    });
  });

  test('Should support message compression', async () => {
    const largeData = 'x'.repeat(10000); // 10KB of data
    
    await new Promise<void>((resolve, reject) => {
      ws = new WebSocket(WS_URL, {
        perMessageDeflate: true
      });
      
      ws.on('open', () => {
        ws.send(JSON.stringify({
          id: 'test-009',
          method: 'echo',
          params: { data: largeData }
        }));
      });
      
      ws.on('message', (data) => {
        const response = JSON.parse(data.toString());
        
        if (response.id === 'test-009') {
          expect(response.result).toHaveProperty('data', largeData);
          resolve();
        }
      });
      
      ws.on('error', reject);
    });
  });

  test('Should handle concurrent connections', async () => {
    const connections = 5;
    const websockets: WebSocket[] = [];
    
    const promises = Array.from({ length: connections }, (_, i) => {
      return new Promise<void>((resolve, reject) => {
        const ws = new WebSocket(WS_URL);
        websockets.push(ws);
        
        ws.on('open', () => {
          ws.send(JSON.stringify({
            id: `concurrent-${i}`,
            method: 'ping',
            params: {}
          }));
        });
        
        ws.on('message', (data) => {
          const response = JSON.parse(data.toString());
          if (response.id === `concurrent-${i}`) {
            expect(response.result).toHaveProperty('pong', true);
            resolve();
          }
        });
        
        ws.on('error', reject);
      });
    });
    
    await Promise.all(promises);
    
    // Clean up
    websockets.forEach(ws => ws.close());
  });

  test('Should reconnect after disconnection', async () => {
    let reconnectCount = 0;
    
    await new Promise<void>((resolve, reject) => {
      const connect = () => {
        ws = new WebSocket(WS_URL);
        
        ws.on('open', () => {
          if (reconnectCount === 0) {
            // First connection - close it
            setTimeout(() => ws.close(), 100);
          } else {
            // Reconnected successfully
            expect(reconnectCount).toBe(1);
            resolve();
          }
        });
        
        ws.on('close', () => {
          if (reconnectCount === 0) {
            reconnectCount++;
            setTimeout(connect, 100); // Reconnect after 100ms
          }
        });
        
        ws.on('error', (err) => {
          if (reconnectCount === 0) {
            // Expected during reconnection
          } else {
            reject(err);
          }
        });
      };
      
      connect();
    });
  });
}); 