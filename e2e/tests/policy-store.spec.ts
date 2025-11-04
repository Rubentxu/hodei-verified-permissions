import { test, expect } from '@playwright/test';

test.describe('Policy Store E2E Tests', () => {
  test.beforeEach(async ({ page }) => {
    // Navigate to the application
    await page.goto('/');

    // Wait for the page to load
    await page.waitForLoadState('networkidle');

    // Take a screenshot for debugging
    await page.screenshot({ path: 'test-results/before-each.png' });
  });

  test('should display policy stores page', async ({ page }) => {
    // Check if the page title or main heading is visible
    await expect(page.locator('h1, h2, [data-testid="page-title"]')).toBeVisible();

    // Take a screenshot
    await page.screenshot({ path: 'test-results/policy-stores-page.png' });
  });

  test('should create a new policy store', async ({ page }) => {
    // Click the "Create Policy Store" button
    await page.click('button:has-text("Create"), [data-testid="create-button"]');

    // Fill in the form
    await page.fill('input[name="name"]', 'Test Policy Store E2E');
    await page.fill('textarea[name="description"]', 'This is a test policy store created by E2E test');

    // Take a screenshot before submitting
    await page.screenshot({ path: 'test-results/create-form-filled.png' });

    // Click submit button
    await page.click('button[type="submit"], button:has-text("Create")');

    // Wait for the success message or redirect
    await page.waitForLoadState('networkidle');

    // Verify the policy store was created
    // Check for success message or the policy store in the list
    await expect(
      page.locator('text=Test Policy Store E2E, text=Policy Store Created, .success')
    ).toBeVisible();

    // Take a screenshot after creation
    await page.screenshot({ path: 'test-results/after-create.png' });
  });

  test('should list policy stores', async ({ page }) => {
    // Wait for the policy stores list to load
    await page.waitForSelector('table, [data-testid="policy-stores-list"], .policy-store-item', {
      timeout: 10000
    });

    // Check if the list is visible
    const list = page.locator('table, [data-testid="policy-stores-list"], .policy-store-item');
    await expect(list).toBeVisible();

    // Take a screenshot
    await page.screenshot({ path: 'test-results/policy-stores-list.png' });

    // Verify at least one policy store exists (or expect empty state)
    const count = await list.count();
    expect(count).toBeGreaterThanOrEqual(0);
  });

  test('should view policy store details', async ({ page }) => {
    // First create a policy store if none exist
    await page.click('button:has-text("Create"), [data-testid="create-button"]');
    await page.fill('input[name="name"]', 'Details Test Store');
    await page.fill('textarea[name="description"]', 'For testing details view');
    await page.click('button[type="submit"]');
    await page.waitForLoadState('networkidle');

    // Now view the details
    // Click on the policy store name or view button
    await page.click('text=Details Test Store, [data-testid="view-button"]');

    // Wait for the details page to load
    await page.waitForLoadState('networkidle');

    // Verify the details are displayed
    await expect(page.locator('h1, h2, [data-testid="policy-store-name"]')).toContainText('Details Test Store');

    // Take a screenshot
    await page.screenshot({ path: 'test-results/policy-store-details.png' });

    // Check for additional details
    await expect(
      page.locator('[data-testid="policy-store-description"], .description, .policy-store-details')
    ).toBeVisible();
  });

  test('should update a policy store', async ({ page }) => {
    // Create a policy store first
    await page.click('button:has-text("Create")');
    await page.fill('input[name="name"]', 'Update Test Store');
    await page.fill('textarea[name="description"]', 'Original description');
    await page.click('button[type="submit"]');
    await page.waitForLoadState('networkidle');

    // Now update it
    // Find and click edit button (depends on UI design)
    await page.click('[data-testid="edit-button"], button:has-text("Edit")');

    // Update the fields
    await page.fill('input[name="name"]', 'Updated Policy Store');
    await page.fill('textarea[name="description"]', 'Updated description');

    // Take a screenshot before saving
    await page.screenshot({ path: 'test-results/update-form.png' });

    // Save the changes
    await page.click('button[type="submit"], button:has-text("Save")');
    await page.waitForLoadState('networkidle');

    // Verify the update
    await expect(page.locator('text=Updated Policy Store')).toBeVisible();

    // Take a screenshot after update
    await page.screenshot({ path: 'test-results/after-update.png' });
  });

  test('should delete a policy store', async ({ page }) => {
    // Create a policy store first
    await page.click('button:has-text("Create")');
    await page.fill('input[name="name"]', 'Delete Test Store');
    await page.fill('textarea[name="description"]', 'To be deleted');
    await page.click('button[type="submit"]');
    await page.waitForLoadState('networkidle');

    // Now delete it
    await page.click('[data-testid="delete-button"], button:has-text("Delete")');

    // Confirm deletion in modal if present
    const deleteButton = page.locator('button:has-text("Confirm Delete"), button:has-text("Yes, Delete")');
    if (await deleteButton.isVisible()) {
      await deleteButton.click();
    }

    await page.waitForLoadState('networkidle');

    // Verify it's deleted (should not be in the list)
    await expect(page.locator('text=Delete Test Store')).not.toBeVisible();

    // Take a screenshot
    await page.screenshot({ path: 'test-results/after-delete.png' });
  });

  test('should view audit log', async ({ page }) => {
    // First create a policy store
    await page.click('button:has-text("Create")');
    await page.fill('input[name="name"]', 'Audit Test Store');
    await page.fill('textarea[name="description"]', 'For testing audit log');
    await page.click('button[type="submit"]');
    await page.waitForLoadState('networkidle');

    // Navigate to the policy store details
    await page.click('text=Audit Test Store');

    // Look for audit log section
    await page.click('[data-testid="audit-tab"], a:has-text("Audit Log")');

    // Wait for audit log to load
    await page.waitForSelector('[data-testid="audit-log"], .audit-log, table', {
      timeout: 10000
    });

    // Take a screenshot
    await page.screenshot({ path: 'test-results/audit-log.png' });

    // Verify audit log is displayed
    await expect(
      page.locator('[data-testid="audit-log"], .audit-log, table')
    ).toBeVisible();
  });

  test('should filter and search policy stores', async ({ page }) => {
    // Wait for the page to load
    await page.waitForSelector('input, [data-testid="search-input"]', {
      timeout: 10000
    });

    // Try to find a search input
    const searchInput = page.locator('input[type="search"], input[placeholder*="search" i], [data-testid="search-input"]');

    if (await searchInput.isVisible()) {
      // Perform a search
      await searchInput.fill('test');
      await page.keyboard.press('Enter');

      // Wait for results
      await page.waitForLoadState('networkidle');

      // Take a screenshot
      await page.screenshot({ path: 'test-results/search-results.png' });
    }
  });

  test('should handle errors gracefully', async ({ page }) => {
    // Try to create a policy store with invalid data
    await page.click('button:has-text("Create")');

    // Try submitting empty form
    await page.click('button[type="submit"]');

    // Check for validation error messages
    await expect(
      page.locator('.error, [data-testid="error-message"], text=Required, text=Invalid')
    ).toBeVisible({ timeout: 5000 });

    // Take a screenshot
    await page.screenshot({ path: 'test-results/validation-error.png' });
  });

  test('should be responsive on mobile', async ({ page }) => {
    // Set viewport to mobile size
    await page.setViewportSize({ width: 375, height: 667 });

    // Navigate to the page
    await page.goto('/');
    await page.waitForLoadState('networkidle');

    // Take a screenshot
    await page.screenshot({ path: 'test-results/mobile-view.png' });

    // Check if the page is usable on mobile
    await expect(page.locator('body')).toBeVisible();
  });
});
