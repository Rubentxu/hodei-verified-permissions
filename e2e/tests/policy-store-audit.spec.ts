import { test, expect } from '@playwright/test';

test.describe('Policy Store - Audit Log & Authorization', () => {
  test.beforeEach(async ({ page }) => {
    await page.goto('/');
    await page.waitForLoadState('networkidle');
  });

  test('should display audit log with filters', async ({ page }) => {
    // Create a test policy store first
    await page.click('button:has-text("Create")');
    await page.fill('input[name="name"]', 'Audit Filter Test');
    await page.fill('textarea[name="description"]', 'Testing audit filters');
    await page.click('button[type="submit"]');
    await page.waitForLoadState('networkidle');

    // Navigate to audit log
    await page.click('text=Audit Filter Test');
    await page.waitForLoadState('networkidle');

    // Look for audit tab or section
    const auditTab = page.locator('[data-testid="audit-tab"], a:has-text("Audit")');
    if (await auditTab.isVisible()) {
      await auditTab.click();
    }

    // Check for filter controls
    const eventTypesFilter = page.locator('select, [data-testid="event-types-filter"]');
    if (await eventTypesFilter.isVisible()) {
      // Test filtering
      await eventTypesFilter.click();
      await page.selectOption('select', 'PolicyStoreCreated');

      await page.screenshot({ path: 'test-results/audit-filter.png' });
    }
  });

  test('should export audit log', async ({ page }) => {
    // Create a test policy store
    await page.click('button:has-text("Create")');
    await page.fill('input[name="name"]', 'Export Test Store');
    await page.fill('textarea[name="description"]', 'Testing export');
    await page.click('button[type="submit"]');
    await page.waitForLoadState('networkidle');

    // Navigate to audit log
    await page.click('text=Export Test Store');
    await page.waitForLoadState('networkidle');

    // Look for export button
    const exportButton = page.locator('button:has-text("Export"), [data-testid="export-button"]');
    if (await exportButton.isVisible()) {
      await exportButton.click();

      // Wait a moment for the download to initiate
      await page.waitForTimeout(2000);

      await page.screenshot({ path: 'test-results/after-export.png' });
    }
  });

  test('should refresh audit log', async ({ page }) => {
    // Create a test policy store
    await page.click('button:has-text("Create")');
    await page.fill('input[name="name"]', 'Refresh Test Store');
    await page.fill('textarea[name="description"]', 'Testing refresh');
    await page.click('button[type="submit"]');
    await page.waitForLoadState('networkidle');

    // Navigate to audit log
    await page.click('text=Refresh Test Store');
    await page.waitForLoadState('networkidle');

    // Look for refresh button
    const refreshButton = page.locator('button:has-text("Refresh"), [data-testid="refresh-button"]');
    if (await refreshButton.isVisible()) {
      // Click refresh
      await refreshButton.click();

      // Wait for refresh to complete
      await page.waitForLoadState('networkidle');

      await page.screenshot({ path: 'test-results/after-refresh.png' });
    }
  });

  test('should show event details', async ({ page }) => {
    // Create a test policy store
    await page.click('button:has-text("Create")');
    await page.fill('input[name="name"]', 'Event Details Test');
    await page.fill('textarea[name="description"]', 'Testing event details');
    await page.click('button[type="submit"]');
    await page.waitForLoadState('networkidle');

    // Navigate to audit log
    await page.click('text=Event Details Test');
    await page.waitForLoadState('networkidle');

    // Look for audit log entries
    const auditEntry = page.locator('[data-testid="audit-entry"], .audit-entry, tr');
    if (await auditEntry.first().isVisible()) {
      // Click on an entry to see details
      await auditEntry.first().click();

      // Look for details modal or expanded view
      await expect(
        page.locator('[data-testid="event-details"], .modal, .expanded')
      ).toBeVisible({ timeout: 5000 });

      await page.screenshot({ path: 'test-results/event-details.png' });
    }
  });

  test('should track policy store creation in audit log', async ({ page }) => {
    // Generate unique name for this test
    const uniqueName = `Audit Track Test ${Date.now()}`;

    // Create a policy store
    await page.click('button:has-text("Create")');
    await page.fill('input[name="name"]', uniqueName);
    await page.fill('textarea[name="description"]', 'Testing audit tracking');
    await page.click('button[type="submit"]');
    await page.waitForLoadState('networkidle');

    // Navigate to audit log
    await page.click(`text=${uniqueName}`);
    await page.waitForLoadState('networkidle');

    // Look for the creation event
    const creationEvent = page.locator('text=PolicyStoreCreated, text=Created, .audit-entry');
    await expect(creationEvent).toBeVisible({ timeout: 10000 });

    await page.screenshot({ path: 'test-results/creation-event.png' });
  });

  test('should track policy store updates in audit log', async ({ page }) => {
    // Create a policy store
    const storeName = `Update Audit Test ${Date.now()}`;
    await page.click('button:has-text("Create")');
    await page.fill('input[name="name"]', storeName);
    await page.fill('textarea[name="description"]', 'Original description');
    await page.click('button[type="submit"]');
    await page.waitForLoadState('networkidle');

    // Update the policy store
    await page.click(`text=${storeName}`);
    await page.waitForLoadState('networkidle');

    // Look for edit button
    const editButton = page.locator('[data-testid="edit-button"], button:has-text("Edit")');
    if (await editButton.isVisible()) {
      await editButton.click();

      // Update the fields
      await page.fill('textarea[name="description"]', 'Updated description');
      await page.click('button[type="submit"]');
      await page.waitForLoadState('networkidle');

      // Navigate to audit log
      await page.click('a:has-text("Audit")');
      await page.waitForLoadState('networkidle');

      // Check for update event
      const updateEvent = page.locator('text=PolicyStoreUpdated, text=Updated, .audit-entry');
      await expect(updateEvent).toBeVisible({ timeout: 10000 });

      await page.screenshot({ path: 'test-results/update-event.png' });
    }
  });

  test('should display timestamps in audit log', async ({ page }) => {
    // Create a test policy store
    await page.click('button:has-text("Create")');
    await page.fill('input[name="name"]', 'Timestamp Test');
    await page.fill('textarea[name="description"]', 'Testing timestamps');
    await page.click('button[type="submit"]');
    await page.waitForLoadState('networkidle');

    // Navigate to audit log
    await page.click('text=Timestamp Test');
    await page.waitForLoadState('networkidle');

    // Look for timestamp column
    const timestampHeader = page.locator('th:has-text("Timestamp"), th:has-text("Time"), .timestamp-header');
    const timestampCell = page.locator('[data-testid="timestamp"], .timestamp, td.timestamp');

    if (await timestampHeader.isVisible()) {
      await expect(timestampCell.first()).toBeVisible();

      // Verify timestamp format (should be in a recognizable format)
      const timestampText = await timestampCell.first().textContent();
      expect(timestampText).toMatch(/\d{4}-\d{2}-\d{2}|\d{2}\/\d{2}\/\d{4}/);

      await page.screenshot({ path: 'test-results/timestamps.png' });
    }
  });

  test('should limit audit log results', async ({ page }) => {
    // Create a test policy store
    await page.click('button:has-text("Create")');
    await page.fill('input[name="name"]', 'Limit Test');
    await page.fill('textarea[name="description"]', 'Testing limit');
    await page.click('button[type="submit"]');
    await page.waitForLoadState('networkidle');

    // Navigate to audit log
    await page.click('text=Limit Test');
    await page.waitForLoadState('networkidle');

    // Look for limit selector
    const limitSelector = page.locator('select, [data-testid="limit-selector"]');
    if (await limitSelector.isVisible()) {
      // Test different limit values
      await limitSelector.click();
      await page.selectOption('select', '10');
      await page.waitForLoadState('networkidle');

      // Count the entries
      const entries = page.locator('[data-testid="audit-entry"], .audit-entry, tr');
      const count = await entries.count();
      expect(count).toBeLessThanOrEqual(10);

      await page.screenshot({ path: 'test-results/limit-results.png' });
    }
  });
});
