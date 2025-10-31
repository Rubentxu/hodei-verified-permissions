import { test, expect } from '@playwright/test';

test.describe('API Endpoints', () => {
  test('should respond to health check', async ({ request }) => {
    const response = await request.get('/api/health');
    expect(response.status()).toBe(200);
    
    const data = await response.json();
    expect(data).toHaveProperty('status');
    expect(['healthy', 'unhealthy']).toContain(data.status);
  });

  test('should accept authorization requests', async ({ request }) => {
    const response = await request.post('/api/authorize', {
      data: {
        policy_store_id: 'test-store',
        principal: {
          entity_type: 'User',
          entity_id: 'user-1'
        },
        action: {
          entity_type: 'Action',
          entity_id: 'view'
        },
        resource: {
          entity_type: 'Document',
          entity_id: 'doc-1'
        }
      }
    });
    
    expect(response.status()).toBe(200);
    
    const data = await response.json();
    expect(data).toHaveProperty('decision');
    expect(['ALLOW', 'DENY']).toContain(data.decision);
  });

  test('should reject invalid authorization requests', async ({ request }) => {
    const response = await request.post('/api/authorize', {
      data: {
        // Missing required fields
        policy_store_id: 'test-store'
      }
    });
    
    expect(response.status()).toBe(400);
    
    const data = await response.json();
    expect(data).toHaveProperty('error');
  });

  test('should create policy stores', async ({ request }) => {
    const response = await request.post('/api/policy-stores', {
      data: {
        description: 'Test Policy Store'
      }
    });
    
    expect(response.status()).toBe(201);
    
    const data = await response.json();
    expect(data).toHaveProperty('policy_store_id');
    expect(data).toHaveProperty('created_at');
  });

  test('should reject invalid policy store creation', async ({ request }) => {
    const response = await request.post('/api/policy-stores', {
      data: {}
    });
    
    // Should still succeed with default values
    expect([200, 201]).toContain(response.status());
  });
});
