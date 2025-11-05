import { Page, Request, Response } from '@playwright/test';

/**
 * Wait for API response with retry mechanism
 */
export async function waitForApiResponse(
  page: Page,
  url: string,
  options: {
    method?: string;
    timeout?: number;
    retries?: number;
  } = {}
): Promise<Response | null> {
  const { method = 'GET', timeout = 5000, retries = 3 } = options;

  for (let i = 0; i < retries; i++) {
    try {
      const response = await page.request.get(url, { timeout });
      if (response.ok()) {
        return response;
      }
    } catch (error) {
      if (i === retries - 1) {
        throw error;
      }
      await page.waitForTimeout(1000);
    }
  }
  return null;
}

/**
 * Check if gRPC server is healthy
 */
export async function checkGrpcHealth(page: Page): Promise<boolean> {
  try {
    const response = await page.request.get('/api/health', { timeout: 5000 });
    if (response.status() === 200) {
      const body = await response.json();
      return body.status === 'healthy';
    }
    return false;
  } catch (error) {
    return false;
  }
}

/**
 * Create a test authorization request
 */
export function createAuthRequest(overrides: Partial<{
  policyStoreId: string;
  principal: { entity_type: string; entity_id: string };
  action: { entity_type: string; entity_id: string };
  resource: { entity_type: string; entity_id: string };
  context: string;
  entities: any[];
}> = {}) {
  return {
    policy_store_id: overrides.policyStoreId || 'test-store',
    principal: overrides.principal || { entity_type: 'User', entity_id: 'alice' },
    action: overrides.action || { entity_type: 'Action', entity_id: 'view' },
    resource: overrides.resource || { entity_type: 'Document', entity_id: 'doc-1' },
    context: overrides.context || '{}',
    entities: overrides.entities || [],
  };
}

/**
 * Submit authorization request and return response
 */
export async function submitAuthorizationRequest(
  page: Page,
  request: any
): Promise<{ status: number; body: any }> {
  const response = await page.request.post('/api/authorize', {
    data: request,
    timeout: 10000,
  });

  const body = response.status() !== 204 ? await response.json() : null;
  return { status: response.status(), body };
}

/**
 * Wait for page to be fully loaded with network idle
 */
export async function waitForPageLoad(page: Page): Promise<void> {
  await page.waitForLoadState('networkidle');
  await page.waitForTimeout(500); // Extra buffer for state stabilization
}

/**
 * Check if element is visible with retry
 */
export async function waitForElementVisible(
  page: Page,
  selector: string,
  timeout = 10000
): Promise<void> {
  await page.waitForSelector(selector, { state: 'visible', timeout });
}

/**
 * Take a screenshot with timestamp
 */
export async function takeScreenshot(
  page: Page,
  name: string,
  fullPage = false
): Promise<void> {
  await page.screenshot({
    path: `test-results/screenshots/${name}-${Date.now()}.png`,
    fullPage,
  });
}
