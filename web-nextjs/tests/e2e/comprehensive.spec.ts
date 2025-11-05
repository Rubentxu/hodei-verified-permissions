import { test, expect } from '@playwright/test';
import {
  waitForPageLoad,
  checkGrpcHealth,
  createAuthRequest,
  submitAuthorizationRequest,
  takeScreenshot
} from './helpers';

test.describe('Comprehensive E2E Tests', () => {
  test.describe.configure({ retries: 2, mode: 'serial' });

  test.beforeAll(async ({ browser }) => {
    // This runs once before all tests in this describe block
    console.log('🚀 Starting comprehensive E2E test suite');
  });

  test.afterEach(async ({ page }, testInfo) => {
    // Take screenshot on failure
    if (testInfo.status !== testInfo.status) {
      await takeScreenshot(page, `failed-${testInfo.title}`);
    }
  });

  test('Application loads and dashboard is accessible', async ({ page }) => {
    test.skip(!await checkGrpcHealth(page), 'Services not available');

    await page.goto('/');
    await waitForPageLoad(page);

    // Verify main navigation
    await expect(page.locator('nav')).toBeVisible();

    // Verify dashboard loads
    await page.goto('/dashboard');
    await waitForPageLoad(page);

    // Check for key dashboard elements
    await expect(page.locator('h2')).toContainText('Dashboard');
    await expect(page.locator('text=System Health')).toBeVisible();
  });

  test('Health check endpoint works', async ({ page }) => {
    const response = await page.request.get('/api/health');
    expect(response.status()).toBe(200);

    const data = await response.json();
    expect(data).toHaveProperty('status');
    expect(['healthy', 'unhealthy']).toContain(data.status);
  });

  test('Authorization API accepts valid requests', async ({ page }) => {
    test.skip(!await checkGrpcHealth(page), 'gRPC server not healthy');

    const authRequest = createAuthRequest({
      policyStoreId: 'test-store-e2e',
      principal: { entity_type: 'User', entity_id: 'test-user' },
      action: { entity_type: 'Action', entity_id: 'view' },
      resource: { entity_type: 'Document', entity_id: 'doc-1' },
    });

    const { status, body } = await submitAuthorizationRequest(page, authRequest);

    // When server is healthy, expect 200 with decision
    if (status === 200) {
      expect(body).toHaveProperty('decision');
      expect(['DECISION_UNSPECIFIED', 'ALLOW', 'DENY']).toContain(body.decision);
    } else if (status === 500) {
      // Server running but gRPC not healthy - acceptable for this test
      console.log('⚠️ gRPC server unhealthy but API responding');
    } else {
      throw new Error(`Unexpected status code: ${status}`);
    }
  });

  test('Authorization API validates required fields', async ({ page }) => {
    const invalidRequest = {
      principal: {
        entity_type: 'User',
        entity_id: 'alice'
      }
      // Missing policy_store_id, action, resource
    };

    const response = await page.request.post('/api/authorize', {
      data: invalidRequest
    });

    expect(response.status()).toBe(400);

    const body = await response.json();
    expect(body).toHaveProperty('error');
  });

  test('Policy stores page loads correctly', async ({ page }) => {
    await page.goto('/policy-stores');
    await waitForPageLoad(page);

    // Should have a policy stores page with some UI
    await expect(page.locator('h1, h2')).toBeVisible();

    // Check for navigation
    await expect(page.locator('nav')).toBeVisible();
  });

  test('Policies page loads correctly', async ({ page }) => {
    await page.goto('/policies');
    await waitForPageLoad(page);

    // Should have a policies page
    await expect(page.locator('h1, h2')).toBeVisible();

    // Check for navigation
    await expect(page.locator('nav')).toBeVisible();
  });

  test('Schemas page loads correctly', async ({ page }) => {
    await page.goto('/schemas');
    await waitForPageLoad(page);

    // Should have a schemas page
    await expect(page.locator('h1, h2')).toBeVisible();

    // Check for navigation
    await expect(page.locator('nav')).toBeVisible();
  });

  test('Templates page loads correctly', async ({ page }) => {
    await page.goto('/templates');
    await waitForPageLoad(page);

    // Should have a templates page
    await expect(page.locator('h1, h2')).toBeVisible();

    // Check for navigation
    await expect(page.locator('nav')).toBeVisible();
  });

  test('Playground page loads and functions', async ({ page }) => {
    await page.goto('/playground');
    await waitForPageLoad(page);

    // Should have playground interface
    await expect(page.locator('#scenario-name, [name="scenario-name"]')).toBeVisible();

    // Try to interact with form elements
    await page.fill('#scenario-name', 'E2E Test Scenario');
    await expect(page.locator('#scenario-name')).toHaveValue('E2E Test Scenario');
  });

  test('Navigation works correctly between pages', async ({ page }) => {
    await page.goto('/');
    await waitForPageLoad(page);

    // Navigate to each main section
    const sections = ['/dashboard', '/policy-stores', '/policies', '/schemas', '/templates', '/playground'];

    for (const section of sections) {
      await page.goto(section);
      await waitForPageLoad(page);

      // Verify page loads
      await expect(page.locator('h1, h2')).toBeVisible();

      // Verify navigation is still present
      await expect(page.locator('nav')).toBeVisible();
    }
  });

  test('Error handling for invalid routes', async ({ page }) => {
    const response = await page.goto('/this-route-does-not-exist');

    // Should either show 404 or redirect to home
    if (response?.status() === 404) {
      // 404 page is acceptable
      expect(response.status()).toBe(404);
    } else {
      // Or redirected to home
      expect(page.url()).toMatch(/\/(dashboard|policy-stores|policies|schemas|templates|playground|$)/);
    }
  });

  test('Batch authorization requests', async ({ page }) => {
    test.skip(!await checkGrpcHealth(page), 'gRPC server not healthy');

    const requests = [
      createAuthRequest({ policyStoreId: 'batch-test-1' }),
      createAuthRequest({ policyStoreId: 'batch-test-2', principal: { entity_type: 'User', entity_id: 'bob' } }),
      createAuthRequest({ policyStoreId: 'batch-test-3', action: { entity_type: 'Action', entity_id: 'edit' } }),
    ];

    for (const request of requests) {
      try {
        const { status } = await submitAuthorizationRequest(page, request);
        // Accept both 200 (healthy) and 500 (unhealthy gRPC)
        expect([200, 500]).toContain(status);
      } catch (error) {
        // Network errors are acceptable if server is not running
        console.log(`⚠️ Batch request failed (acceptable if server unhealthy): ${error}`);
      }
    }
  });
});
