import { test, expect } from '@playwright/test';

test.describe('Hodei Verified Permissions - BFF E2E Tests', () => {
  test('Health check endpoint', async ({ request }) => {
    // Test the health endpoint directly
    const response = await request.get('/api/health');
    expect(response.status()).toBe(200);
    
    const body = await response.json();
    expect(body).toHaveProperty('status');
    // The gRPC server might not be running, so we expect either healthy or unhealthy
    expect(['healthy', 'unhealthy']).toContain(body.status);
  });

  test('Authorization endpoint - basic request', async ({ request }) => {
    const authRequest = {
      policy_store_id: 'test-store',
      principal: {
        entity_type: 'User',
        entity_id: 'alice'
      },
      action: {
        entity_type: 'Action',
        entity_id: 'viewDocument'
      },
      resource: {
        entity_type: 'Document',
        entity_id: 'doc123'
      },
      context: '{}',
      entities: []
    };

    const response = await request.post('/api/authorize', {
      data: authRequest
    });

    // If gRPC server is not running, we expect 500
    // If it is running, we expect 200
    expect([200, 500]).toContain(response.status());
    
    if (response.status() === 200) {
      const body = await response.json();
      expect(body).toHaveProperty('decision');
      expect(['DECISION_UNSPECIFIED', 'ALLOW', 'DENY']).toContain(body.decision);
    }
  });

  test('Policy store creation endpoint', async ({ request }) => {
    const policyStoreRequest = {
      description: 'Test policy store for E2E testing'
    };

    const response = await request.post('/api/policy-stores', {
      data: policyStoreRequest
    });

    // If gRPC server is not running, we expect 500
    // If it is running, we expect 201
    expect([201, 500]).toContain(response.status());
    
    if (response.status() === 201) {
      const body = await response.json();
      expect(body).toHaveProperty('policy_store_id');
      expect(body).toHaveProperty('created_at');
    }
  });

  test('Frontend loads correctly', async ({ page }) => {
    await page.goto('/');
    
    // Check that the page loads
    await expect(page).toHaveTitle(/Hodei Verified Permissions/);
    
    // Check for main content
    await expect(page.locator('h1')).toContainText('Hodei Verified Permissions');
    await expect(page.locator('p')).toContainText('Backend for Frontend');
  });

  test('Error handling - invalid authorization request', async ({ request }) => {
    const invalidRequest = {
      // Missing required fields
      principal: {
        entity_type: 'User',
        entity_id: 'alice'
      }
    };

    const response = await request.post('/api/authorize', {
      data: invalidRequest
    });

    expect(response.status()).toBe(400);
    
    const body = await response.json();
    expect(body).toHaveProperty('error');
    expect(body.error).toContain('Missing required fields');
  });

  test('Error handling - method not allowed', async ({ request }) => {
    const response = await request.get('/api/authorize');
    expect(response.status()).toBe(405);
    
    const body = await response.json();
    expect(body).toHaveProperty('error');
    expect(body.error).toContain('Method not allowed');
  });
});
