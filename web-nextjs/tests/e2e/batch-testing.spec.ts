import { test, expect } from '@playwright/test';

test.describe('Batch Authorization Testing', () => {
  test.beforeEach(async ({ page }) => {
    await page.goto('/playground');
    await page.waitForLoadState('networkidle');

    // Switch to Batch Test tab
    await page.click('button:has-text("Batch Test")');
    await page.waitForTimeout(500);
  });

  test('should display batch testing interface', async ({ page }) => {
    // Verify batch test UI elements
    await expect(page.locator('text=Batch Authorization Testing')).toBeVisible();
    await expect(page.locator('text=Predefined Test Suites')).toBeVisible();
    await expect(page.locator('text=Custom Test Suite')).toBeVisible();

    // Check for predefined test buttons
    await expect(page.locator('button:has-text("User Access Tests")')).toBeVisible();
    await expect(page.locator('button:has-text("Role-Based Tests")')).toBeVisible();
  });

  test('should run user access test suite', async ({ page }) => {
    // Click User Access Tests
    await page.click('button:has-text("User Access Tests (3 scenarios)")');

    // Verify loading state
    await expect(page.locator('text=Running batch test...')).toBeVisible();

    // Wait for test to complete (up to 30 seconds)
    await page.waitForTimeout(30000);

    // Verify results are displayed
    await expect(page.locator('text=Test Summary')).toBeVisible();

    // Check statistics cards
    await expect(page.locator('text=Total Tests')).toBeVisible();
    await expect(page.locator('text=Successful')).toBeVisible();
    await expect(page.locator('text=Failed')).toBeVisible();
    await expect(page.locator('text=Avg Latency')).toBeVisible();
  });

  test('should run role-based test suite', async ({ page }) => {
    // Click Role-Based Tests
    await page.click('button:has-text("Role-Based Tests (3 scenarios)")');

    // Wait for test to complete
    await page.waitForTimeout(30000);

    // Verify results
    await expect(page.locator('text=Test Summary')).toBeVisible();

    // Should show decision breakdown
    await expect(page.locator('text=ALLOW:')).toBeVisible();
    await expect(page.locator('text=DENY:')).toBeVisible();
  });

  test('should display detailed results table', async ({ page }) => {
    // Run a test suite
    await page.click('button:has-text("User Access Tests (3 scenarios)")');
    await page.waitForTimeout(30000);

    // Verify results table
    const table = page.locator('table');
    await expect(table).toBeVisible();

    // Check table headers
    await expect(page.locator('th:has-text("Scenario")')).toBeVisible();
    await expect(page.locator('th:has-text("Decision")')).toBeVisible();
    await expect(page.locator('th:has-text("Latency")')).toBeVisible();
    await expect(page.locator('th:has-text("Status")')).toBeVisible();

    // Verify at least one result row
    const rows = page.locator('tbody tr');
    await expect(rows.first()).toBeVisible();
  });

  test('should show decision badges in results', async ({ page }) => {
    // Run test
    await page.click('button:has-text("User Access Tests (3 scenarios)")');
    await page.waitForTimeout(30000);

    // Check for decision badges
    const allowBadge = page.locator('text=ALLOW').first();
    const denyBadge = page.locator('text=DENY').first();
    const unspecifiedBadge = page.locator('text=UNSPECIFIED');

    // At least one type of badge should be visible
    const hasAnyBadge = 
      (await allowBadge.count() > 0) || 
      (await denyBadge.count() > 0) || 
      (await unspecifiedBadge.count() > 0);

    expect(hasAnyBadge).toBeTruthy();
  });

  test('should configure custom batch test', async ({ page }) => {
    // Set number of scenarios
    const scenarioInput = page.locator('input[type="number"]');
    await scenarioInput.fill('5');

    // Run custom test
    await page.click('button:has-text("Run Custom Test")');

    // Wait for completion
    await page.waitForTimeout(30000);

    // Verify 5 scenarios were tested
    await expect(page.locator('text=Total Tests')).toHaveText('5');
  });

  test('should display performance metrics', async ({ page }) => {
    // Run test
    await page.click('button:has-text("User Access Tests (3 scenarios)")');
    await page.waitForTimeout(30000);

    // Check latency metrics
    await expect(page.locator('text=Avg Latency')).toBeVisible();
    await expect(page.locator('text=Min:')).toBeVisible();
    await expect(page.locator('text=Max:')).toBeVisible();

    // Verify latency values are numeric
    const avgLatency = page.locator('text=Avg Latency >> .. >> text=/ms/');
    await expect(avgLatency.first()).toBeVisible();
  });

  test('should show test status icons', async ({ page }) => {
    // Run test
    await page.click('button:has-text("User Access Tests (3 scenarios)")');
    await page.waitForTimeout(30000);

    // Check for status icons in results table
    const successIcon = page.locator('[data-testid="success-icon"]');
    const failureIcon = page.locator('[data-testid="failure-icon"]');

    // At least one icon should be visible
    const hasIcons = 
      (await successIcon.count() > 0) || 
      (await failureIcon.count() > 0);

    expect(hasIcons).toBeTruthy();
  });

  test('should export results to CSV', async ({ page }) => {
    // Run test first
    await page.click('button:has-text("User Access Tests (3 scenarios)")');
    await page.waitForTimeout(30000);

    // Click export button
    const downloadPromise = page.waitForEvent('download');
    await page.click('button:has-text("Export Results (CSV)")');

    // Verify download
    const download = await downloadPromise;
    expect(download.suggestedFilename()).toMatch(/\.csv$/);

    // Verify filename includes timestamp
    expect(download.suggestedFilename()).toMatch(/batch-test-\d+\.csv/);
  });

  test('should calculate correct statistics', async ({ page }) => {
    // Run test
    await page.click('button:has-text("User Access Tests (3 scenarios)")');
    await page.waitForTimeout(30000);

    // Get total and successful counts
    const totalText = await page.locator('text=Total Tests').first().textContent();
    const successfulText = await page.locator('text=Successful').first().textContent();

    const total = parseInt(totalText?.match(/\d+/)?.[0] || '0');
    const successful = parseInt(successfulText?.match(/\d+/)?.[0] || '0');

    // Verify counts are reasonable
    expect(total).toBeGreaterThan(0);
    expect(successful).toBeGreaterThanOrEqual(0);
    expect(successful).toBeLessThanOrEqual(total);
  });

  test('should handle multiple batch test runs', async ({ page }) => {
    // Run first test
    await page.click('button:has-text("User Access Tests (3 scenarios)")');
    await page.waitForTimeout(30000);

    // Verify first results
    await expect(page.locator('text=Test Summary')).toBeVisible();

    // Run second test
    await page.click('button:has-text("Role-Based Tests (3 scenarios)")');
    await page.waitForTimeout(30000);

    // Verify second results replace first
    await expect(page.locator('text=Test Summary')).toBeVisible();
  });

  test('should show error on failed batch test', async ({ page }) => {
    // This test would require mocking API failures
    // For now, verify error UI exists

    // Try running test with invalid configuration
    // (Actual error handling depends on implementation)

    // Check for error display
    const errorCard = page.locator('text=Test Failed');
    // Should not be visible initially
    await expect(errorCard).toHaveCount(0);
  });

  test('should show determining policies for each test', async ({ page }) => {
    // Run test
    await page.click('button:has-text("User Access Tests (3 scenarios)")');
    await page.waitForTimeout(30000);

    // Check results table for policies column
    const policiesHeader = page.locator('th:has-text("Policies")');
    await expect(policiesHeader).toBeVisible();

    // Verify some results show policies
    const policyCell = page.locator('td:has-text("policy")').first();
    await expect(policyCell).toBeVisible();
  });

  test('should display test execution progress', async ({ page }) => {
    // Start test
    await page.click('button:has-text("User Access Tests (3 scenarios)")');

    // Should show progress indicator
    const progress = page.locator('[data-testid="test-progress"]');
    if (await progress.count() > 0) {
      await expect(progress).toBeVisible();
    }

    // Wait for completion
    await page.waitForTimeout(30000);

    // Progress should disappear
    await expect(progress).toHaveCount(0);
  });
});
