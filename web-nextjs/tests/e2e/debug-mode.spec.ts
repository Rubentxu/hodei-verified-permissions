import { test, expect } from '@playwright/test';

test.describe('Debug Mode', () => {
  test.beforeEach(async ({ page }) => {
    await page.goto('/playground');
    await page.waitForLoadState('networkidle');
  });

  test('should enable debug mode and show debug panel', async ({ page }) => {
    // Click Enable Debug button
    await page.click('button:has-text("Enable Debug")');

    // Verify debug panel is visible
    await expect(page.locator('text=Debug Mode')).toBeVisible();
    await expect(page.locator('text=Execution Steps')).toBeVisible();

    // Verify debug button changed to "Disable Debug"
    await expect(page.locator('button:has-text("Disable Debug")')).toBeVisible();
  });

  test('should run debug steps when test is executed', async ({ page }) => {
    // Enable debug mode
    await page.click('button:has-text("Enable Debug")');

    // Fill in a test scenario
    await page.fill('#scenario-name', 'Debug Test Scenario');
    await page.fill('#policy-store', 'ps_debug_test');

    // Run the test
    await page.click('button:has-text("Run Test")');

    // Wait for debug steps to complete
    await page.waitForTimeout(3000);

    // Verify debug steps are shown
    await expect(page.locator('text=Step 1')).toBeVisible();
    await expect(page.locator('text=Step 2')).toBeVisible();
    await expect(page.locator('text=Step 3')).toBeVisible();
    await expect(page.locator('text=Step 4')).toBeVisible();
    await expect(page.locator('text=Step 5')).toBeVisible();

    // Verify some steps are completed
    const completedSteps = page.locator('text=Completed');
    const count = await completedSteps.count();
    expect(count).toBeGreaterThan(0);
  });

  test('should show step details when expanded', async ({ page }) => {
    // Enable debug mode and run a test
    await page.click('button:has-text("Enable Debug")');
    await page.fill('#scenario-name', 'Debug Test Scenario');
    await page.click('button:has-text("Run Test")');
    await page.waitForTimeout(3000);

    // Click on a step to expand details
    const stepHeader = page.locator('.border.rounded-lg').first();
    await stepHeader.click();

    // Verify expanded details are shown
    await expect(page.locator('text=Details:')).toBeVisible();
  });

  test('should disable debug mode and hide panel', async ({ page }) => {
    // First enable debug mode
    await page.click('button:has-text("Enable Debug")');
    await expect(page.locator('text=Debug Mode')).toBeVisible();

    // Then disable it
    await page.click('button:has-text("Disable Debug")');

    // Verify debug panel is hidden
    await expect(page.locator('text=Debug Mode')).toHaveCount(0);
  });
});
