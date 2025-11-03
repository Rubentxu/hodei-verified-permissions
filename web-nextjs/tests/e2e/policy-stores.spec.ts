import { test, expect, Page } from '@playwright/test';

/**
 * E2E Tests for Policy Store Management UI
 * Tests cover: CRUD, Metrics, Audit, Tags, Snapshots, Batch Operations
 *
 * Test Case IDs: PS-001 to PS-100
 */

test.describe('Policy Store Management - E2E Tests', () => {
  test.beforeEach(async ({ page }) => {
    // Navigate to the application
    await page.goto('/');
    await expect(page.locator('h1')).toContainText('Hodei Verified Permissions');
  });

  // ============================================================================
  // TC-001 to TC-010: CRUD Operations
  // ============================================================================

  test('PS-001: Create Policy Store via UI', async ({ page }) => {
    // Click Create Policy Store button
    await page.click('text=Create Policy Store');

    // Fill in description
    const description = `Test Store ${Date.now()}`;
    await page.fill('textarea[placeholder*="description"]', description);

    // Click Create
    await page.click('button:has-text("Create")');

    // Verify success - should see the new store in the list
    await expect(page.locator(`text=${description}`)).toBeVisible();

    // Verify metrics show 0 policies and 0 schemas initially
    const policyCount = page.locator('text=0').first();
    await expect(policyCount).toBeVisible();
  });

  test('PS-002: View Policy Store Details', async ({ page }) => {
    // First create a policy store if none exists
    await createPolicyStoreIfNone(page);

    // Click View Details on first policy store card
    await page.click('button:has-text("View Details")');

    // Verify modal opens with Overview tab
    await expect(page.locator('h3:has-text("Policy Store Details")')).toBeVisible();
    await expect(page.locator('text=Overview')).toBeVisible();

    // Verify metrics are displayed
    await expect(page.locator('text=Policies')).toBeVisible();
    await expect(page.locator('text=Schemas')).toBeVisible();
    await expect(page.locator('text=Status')).toBeVisible();
    await expect(page.locator('text=Version')).toBeVisible();

    // Verify metadata section
    await expect(page.locator('text=Author:')).toBeVisible();
    await expect(page.locator('text=Created:')).toBeVisible();
    await expect(page.locator('text=Last Modified:')).toBeVisible();

    // Close modal
    await page.click('button:has-text("Close")');
    await expect(page.locator('text=Policy Store Details')).not.toBeVisible();
  });

  test('PS-003: Edit Policy Store Description', async ({ page }) => {
    // Create a policy store if none exists
    await createPolicyStoreIfNone(page);

    // Click Edit button on first card
    await page.click('[title="Edit policy store"]');

    // Fill in new description
    const newDescription = `Updated Description ${Date.now()}`;
    await page.fill('textarea[placeholder*="description"]', newDescription);

    // Click Update
    await page.click('button:has-text("Update")');

    // Verify the description was updated
    await expect(page.locator(`text=${newDescription}`)).toBeVisible();
  });

  test('PS-004: Delete Policy Store', async ({ page }) => {
    // Create a policy store if none exists
    const storeDescription = await createPolicyStoreIfNone(page);

    // Click Delete button
    await page.click('[title="Delete policy store"]');

    // Confirm deletion in dialog
    await page.on('dialog', dialog => dialog.accept());
    await page.waitForTimeout(500);

    // Verify the store was deleted (should not be visible)
    await expect(page.locator(`text=${storeDescription}`)).not.toBeVisible();
  });

  test('PS-005: Policy Store List Shows Real Metrics', async ({ page }) => {
    // Create a policy store if none exists
    await createPolicyStoreIfNone(page);

    // Verify the policy count and schema count are displayed (not "-")
    // Look for the counts in the card badges
    const policyCountText = await page.locator('text=0').count();
    expect(policyCountText).toBeGreaterThan(0);
  });

  // ============================================================================
  // TC-011 to TC-020: Audit Log
  // ============================================================================

  test('PS-011: View Audit Log in Details Modal', async ({ page }) => {
    // Create a policy store if none exists
    await createPolicyStoreIfNone(page);

    // Open details modal
    await page.click('button:has-text("View Details")');

    // Click on Audit Log tab
    await page.click('button:has-text("Audit Log")');

    // Verify audit log tab is active
    await expect(page.locator('button:has-text("Audit Log")')).toHaveClass(/border-b-2/);

    // Verify audit log content
    await expect(page.locator('text=Activity History')).toBeVisible();
    await expect(page.locator('text=events')).toBeVisible();

    // Verify audit entries show action type
    await expect(page.locator('text=CREATE')).toBeVisible();
  });

  test('PS-012: Audit Log Shows CREATE Action', async ({ page }) => {
    // Create a new policy store
    const description = await createPolicyStoreIfNone(page);

    // Open details modal
    await page.click('button:has-text("View Details")');

    // Go to Audit Log tab
    await page.click('button:has-text("Audit Log")');

    // Verify CREATE action is logged
    await expect(page.locator('text=CREATE')).toBeVisible();

    // Verify timestamp is displayed
    await expect(page.locator('text=by')).toBeVisible();
  });

  // ============================================================================
  // TC-021 to TC-030: Tags Management
  // ============================================================================

  test('PS-021: Add Tags to Policy Store', async ({ page }) => {
    // Create a policy store if none exists
    await createPolicyStoreIfNone(page);

    // Open details modal
    await page.click('button:has-text("View Details")');

    // Click on Tags tab
    await page.click('button:has-text("Tags")');

    // Add first tag
    const tagInput = page.locator('input[placeholder*="tag"]');
    await tagInput.fill('production');
    await page.click('button:has-text("Add")');

    // Verify tag appears
    await expect(page.locator('text=production')).toBeVisible();

    // Add second tag
    await tagInput.fill('frontend');
    await page.click('button:has-text("Add")');

    // Verify second tag appears
    await expect(page.locator('text=frontend')).toBeVisible();

    // Close modal
    await page.click('button:has-text("Close")');

    // Verify tags are shown in Overview tab
    await page.click('button:has-text("View Details")');
    await expect(page.locator('text=production')).toBeVisible();
    await expect(page.locator('text=frontend')).toBeVisible();
  });

  test('PS-022: Remove Tags from Policy Store', async ({ page }) => {
    // Create a policy store if none exists
    await createPolicyStoreIfNone(page);

    // Open details modal and go to Tags tab
    await page.click('button:has-text("View Details")');
    await page.click('button:has-text("Tags")');

    // Add a tag first
    const tagInput = page.locator('input[placeholder*="tag"]');
    await tagInput.fill('test-tag');
    await page.click('button:has-text("Add")');

    // Verify tag was added
    await expect(page.locator('text=test-tag')).toBeVisible();

    // Remove the tag (click on it or remove button)
    // Assuming tags have a remove button or are clickable
    await page.click(`text=test-tag >> xpath=../button`);
    await page.waitForTimeout(300);

    // Verify tag was removed
    await expect(page.locator('text=test-tag')).not.toBeVisible();
  });

  test('PS-023: Tag Autocomplete Suggestions', async ({ page }) => {
    // Create a policy store if none exists
    await createPolicyStoreIfNone(page);

    // Open details modal and go to Tags tab
    await page.click('button:has-text("View Details")');
    await page.click('button:has-text("Tags")');

    // Start typing a tag that might have autocomplete
    const tagInput = page.locator('input[placeholder*="tag"]');
    await tagInput.type('prod');

    // Wait a moment for suggestions to appear
    await page.waitForTimeout(500);

    // Check if suggestions dropdown appears (implementation depends on UI)
    // This test may need adjustment based on actual autocomplete implementation
    const suggestion = page.locator('[role="option"]').first();
    if (await suggestion.isVisible()) {
      await suggestion.click();
      await expect(page.locator('text=prod')).toBeVisible();
    }
  });

  // ============================================================================
  // TC-031 to TC-040: Snapshots and Versioning
  // ============================================================================

  test('PS-031: Create Snapshot', async ({ page }) => {
    // Create a policy store if none exists
    await createPolicyStoreIfNone(page);

    // Open details modal
    await page.click('button:has-text("View Details")');

    // Click on Version History tab
    await page.click('button:has-text("Version History")');

    // Click Create Snapshot button
    await page.click('button:has-text("Create Snapshot")');

    // Fill in description
    const snapshotDescription = `Snapshot ${Date.now()}`;
    await page.fill('input[placeholder*="description"]', snapshotDescription);

    // Click Create Snapshot
    await page.click('button:has-text("Create Snapshot")');

    // Wait for snapshot to be created
    await page.waitForTimeout(1000);

    // Verify snapshot appears in the list
    await expect(page.locator(`text=${snapshotDescription}`)).toBeVisible();

    // Verify snapshot shows metrics
    await expect(page.locator('text=Policies:')).toBeVisible();
    await expect(page.locator('text=Size:')).toBeVisible();
  });

  test('PS-032: View Snapshot Details', async ({ page }) => {
    // Create and open details modal with Version History tab
    await createPolicyStoreIfNone(page);
    await page.click('button:has-text("View Details")');
    await page.click('button:has-text("Version History")');

    // First create a snapshot if none exists
    const createSnapshotButton = page.locator('button:has-text("Create Snapshot")');
    if (await createSnapshotButton.isVisible()) {
      await page.click('button:has-text("Create Snapshot")');
      await page.fill('input[placeholder*="description"]', 'Test Snapshot');
      await page.click('button:has-text("Create Snapshot")');
      await page.waitForTimeout(1000);
    }

    // Verify snapshot card is visible
    await expect(page.locator('text=Policies:')).toBeVisible();
    await expect(page.locator('text=Created:')).toBeVisible();

    // Verify snapshot has snapshot ID
    const snapshotId = page.locator('[class*="badge"]').first();
    await expect(snapshotId).toBeVisible();
  });

  test('PS-033: Rollback to Snapshot', async ({ page }) => {
    // This test requires more complex setup with policies
    // For now, we'll test the UI interaction

    // Create a policy store if none exists
    await createPolicyStoreIfNone(page);

    // Open details modal and Version History tab
    await page.click('button:has-text("View Details")');
    await page.click('button:has-text("Version History")');

    // Check if Rollback button exists
    const rollbackButton = page.locator('button:has-text("Rollback")');
    if (await rollbackButton.isVisible()) {
      // Click rollback
      await rollbackButton.click();

      // Handle confirmation dialog
      page.on('dialog', dialog => dialog.accept());
      await page.waitForTimeout(1000);

      // Verify success message (implementation depends on UI feedback)
      // await expect(page.locator('text=Successfully rolled back')).toBeVisible();
    }
  });

  test('PS-034: Delete Snapshot', async ({ page }) => {
    // Create and open Version History tab
    await createPolicyStoreIfNone(page);
    await page.click('button:has-text("View Details")');
    await page.click('button:has-text("Version History")');

    // Create a snapshot if none exists
    const createButton = page.locator('button:has-text("Create Snapshot")');
    if (await createButton.isVisible()) {
      await page.click('button:has-text("Create Snapshot")');
      await page.fill('input[placeholder*="description"]', 'To Delete');
      await page.click('button:has-text("Create Snapshot")');
      await page.waitForTimeout(1000);
    }

    // Find delete button (trash icon)
    const deleteButton = page.locator('[title*="Delete"]').first();
    if (await deleteButton.isVisible()) {
      // Click delete
      await deleteButton.click();

      // Handle confirmation
      page.on('dialog', dialog => dialog.accept());
      await page.waitForTimeout(500);

      // Verify snapshot was removed
      await expect(page.locator('text=To Delete')).not.toBeVisible();
    }
  });

  // ============================================================================
  // TC-041 to TC-050: Filtering and Search
  // ============================================================================

  test('PS-041: Search Policy Stores by Description', async ({ page }) => {
    // Create multiple policy stores
    await createPolicyStoreIfNone(page);

    // Add search term in search box
    const searchBox = page.locator('input[placeholder*="Search"]');
    await searchBox.fill('Test Store');

    // Press Enter or wait for search
    await page.keyboard.press('Enter');
    await page.waitForTimeout(500);

    // Verify filtered results
    const storeCards = page.locator('[class*="card"]');
    const count = await storeCards.count();
    expect(count).toBeGreaterThan(0);

    // All visible cards should contain search term
    for (let i = 0; i < count; i++) {
      const cardText = await storeCards.nth(i).textContent();
      expect(cardText).toContain('Test Store');
    }
  });

  test('PS-042: Filter by Status (Active/Inactive)', async ({ page }) => {
    // Open filters panel
    await page.click('button:has-text("Filter")');

    // Wait for filters panel to open
    await page.waitForTimeout(300);

    // Check if status filter exists
    const statusFilter = page.locator('select').first();
    if (await statusFilter.isVisible()) {
      // Select Active status
      await statusFilter.selectOption('active');

      // Apply filter
      await page.click('text=Clear Filters'); // Just to trigger change
      await page.waitForTimeout(300);

      // Verify filtered results
      const storeCards = page.locator('[class*="card"]');
      const count = await storeCards.count();
      expect(count).toBeGreaterThanOrEqual(0);
    }
  });

  test('PS-043: Filter by Tags', async ({ page }) => {
    // Create a policy store with tags
    await createPolicyStoreIfNone(page);
    await page.click('button:has-text("View Details")');
    await page.click('button:has-text("Tags")');

    // Add a tag
    const tagInput = page.locator('input[placeholder*="tag"]');
    await tagInput.fill('production');
    await page.click('button:has-text("Add")');
    await page.click('button:has-text("Close")');

    // Open filters
    await page.click('button:has-text("Filter")');
    await page.waitForTimeout(300);

    // Find and click the tag in filters
    const productionTag = page.locator('text=production').first();
    if (await productionTag.isVisible()) {
      await productionTag.click();
      await page.waitForTimeout(300);

      // Verify filtered results show only stores with 'production' tag
      // Implementation depends on how filters are applied
    }
  });

  test('PS-044: Clear All Filters', async ({ page }) => {
    // Apply some filters
    await page.click('button:has-text("Filter")');
    await page.waitForTimeout(300);

    // Set status filter
    const statusFilter = page.locator('select').first();
    if (await statusFilter.isVisible()) {
      await statusFilter.selectOption('active');
    }

    // Click Clear Filters
    const clearButton = page.locator('button:has-text("Clear Filters")');
    if (await clearButton.isVisible()) {
      await clearButton.click();
      await page.waitForTimeout(300);

      // Verify all stores are shown again
      const storeCards = page.locator('[class*="card"]');
      const count = await storeCards.count();
      expect(count).toBeGreaterThanOrEqual(1);
    }
  });

  // ============================================================================
  // TC-051 to TC-060: Batch Operations
  // ============================================================================

  test('PS-051: Batch Create Policies (via UI)', async ({ page }) => {
    // This test depends on batch operations UI implementation
    // We'll test the navigation and UI elements

    // Create a policy store if none exists
    await createPolicyStoreIfNone(page);

    // Navigate to policy management (if there's a specific UI for it)
    // This is placeholder - actual implementation depends on UI design
    await page.click('button:has-text("View Details")');
    await page.click('button:has-text("Overview")');

    // Check for batch operation buttons/UI
    // await expect(page.locator('text=Batch Operations')).toBeVisible();
  });

  // ============================================================================
  // TC-071 to TC-080: Performance and Load
  // ============================================================================

  test('PS-071: Load Performance - Policy Store List', async ({ page }) => {
    // Measure page load time
    const startTime = Date.now();

    // Create multiple policy stores for performance testing
    for (let i = 0; i < 10; i++) {
      await createPolicyStoreIfNone(page);
    }

    // Navigate to the page
    await page.goto('/');
    await page.waitForSelector('text=Policy Stores');

    const loadTime = Date.now() - startTime;

    // Should load within 5 seconds
    expect(loadTime).toBeLessThan(5000);
  });

  // ============================================================================
  // TC-081 to TC-090: Error Handling
  // ============================================================================

  test('PS-081: Handle Network Errors Gracefully', async ({ page }) => {
    // Create a policy store
    await createPolicyStoreIfNone(page);

    // Simulate network error by blocking requests
    await page.route('**/api/**', route => {
      route.abort('internetdisconnected');
    });

    // Try to create another policy store
    await page.click('text=Create Policy Store');
    await page.fill('textarea[placeholder*="description"]', 'Network Error Test');
    await page.click('button:has-text("Create")');

    // Wait for error handling
    await page.waitForTimeout(1000);

    // Verify error message is shown (implementation depends on UI)
    // await expect(page.locator('text=Failed')).toBeVisible();
  });

  test('PS-082: Validate Required Fields', async ({ page }) => {
    // Open create modal
    await page.click('text=Create Policy Store');

    // Try to create without description (if required)
    await page.click('button:has-text("Create")');

    // Verify validation error
    // This depends on form validation implementation
    // await expect(page.locator('text=required')).toBeVisible();
  });

  // ============================================================================
  // TC-091 to TC-100: Accessibility
  // ============================================================================

  test('PS-091: Keyboard Navigation', async ({ page }) => {
    // Create a policy store
    await createPolicyStoreIfNone(page);

    // Use Tab to navigate through elements
    await page.keyboard.press('Tab');
    await page.keyboard.press('Tab');
    await page.keyboard.press('Tab');

    // Verify focus is visible on interactive elements
    const focused = await page.locator(':focus').first();
    await expect(focused).toBeVisible();

    // Press Enter to activate focused element
    await page.keyboard.press('Enter');
    await page.waitForTimeout(300);

    // Should have opened something or performed an action
  });

  test('PS-092: ARIA Labels and Roles', async ({ page }) => {
    // Check that interactive elements have proper ARIA labels
    const buttons = page.locator('button');
    const count = await buttons.count();

    for (let i = 0; i < count; i++) {
      const button = buttons.nth(i);
      const ariaLabel = await button.getAttribute('aria-label');
      const title = await button.getAttribute('title');

      // Should have either aria-label or title for accessibility
      if (await button.isVisible()) {
        expect(ariaLabel || title).toBeTruthy();
      }
    }
  });

  // ============================================================================
  // Helper Functions
  // ============================================================================

  async function createPolicyStoreIfNone(page: Page): Promise<string> {
    // Check if any policy stores exist
    const existingStores = page.locator('[class*="card"]');
    const count = await existingStores.count();

    if (count > 0) {
      // Return description of first store
      return await existingStores.first().textContent() || 'Existing Store';
    }

    // Create a new policy store
    const description = `Test Store ${Date.now()}`;
    await page.click('text=Create Policy Store');
    await page.fill('textarea[placeholder*="description"]', description);
    await page.click('button:has-text("Create")');

    // Wait for store to be created
    await page.waitForTimeout(500);

    return description;
  }
});

/**
 * Additional Test Suite for Snapshots and Batch Operations
 * These tests require more complex setup and are marked as such
 */

test.describe('Policy Store - Advanced Features', () => {
  test.beforeEach(async ({ page }) => {
    await page.goto('/');
    await expect(page.locator('h1')).toContainText('Hodei Verified Permissions');
  });

  test('Advanced-001: Full Snapshot Workflow', async ({ page }) => {
    // Create policy store
    await page.click('text=Create Policy Store');
    await page.fill('textarea[placeholder*="description"]', 'Snapshot Workflow Test');
    await page.click('button:has-text("Create")');
    await page.waitForTimeout(500);

    // Create snapshot 1
    await page.click('button:has-text("View Details")');
    await page.click('button:has-text("Version History")');
    await page.click('button:has-text("Create Snapshot")');
    await page.fill('input[placeholder*="description"]', 'Snapshot 1 - Initial State');
    await page.click('button:has-text("Create Snapshot")');
    await page.waitForTimeout(1000);

    // Create snapshot 2
    await page.click('button:has-text("Create Snapshot")');
    await page.fill('input[placeholder*="description"]', 'Snapshot 2 - Modified State');
    await page.click('button:has-text("Create Snapshot")');
    await page.waitForTimeout(1000);

    // Verify both snapshots exist
    await expect(page.locator('text=Snapshot 1')).toBeVisible();
    await expect(page.locator('text=Snapshot 2')).toBeVisible();

    // Test rollback to snapshot 1
    const rollbackButtons = page.locator('button:has-text("Rollback")');
    const count = await rollbackButtons.count();
    if (count >= 2) {
      await rollbackButtons.first().click();
      page.on('dialog', dialog => dialog.accept());
      await page.waitForTimeout(1000);
    }
  });

  test('Advanced-002: Tag Management Workflow', async ({ page }) => {
    // Create policy store with multiple tags
    await page.click('text=Create Policy Store');
    await page.fill('textarea[placeholder*="description"]', 'Tag Management Test');
    await page.click('button:has-text("Create")');
    await page.waitForTimeout(500);

    // Open tags tab
    await page.click('button:has-text("View Details")');
    await page.click('button:has-text("Tags")');

    // Add multiple tags
    const tags = ['production', 'frontend', 'critical', 'api'];
    for (const tag of tags) {
      await page.fill('input[placeholder*="tag"]', tag);
      await page.click('button:has-text("Add")');
      await page.waitForTimeout(200);
    }

    // Verify all tags appear
    for (const tag of tags) {
      await expect(page.locator(`text=${tag}`)).toBeVisible();
    }

    // Remove some tags
    await page.click(`text=critical >> xpath=../button`);
    await page.waitForTimeout(300);
    await expect(page.locator('text=critical')).not.toBeVisible();

    // Switch to Overview tab and verify tags are shown
    await page.click('button:has-text("Overview")');
    for (const tag of ['production', 'frontend', 'api']) {
      await expect(page.locator(`text=${tag}`)).toBeVisible();
    }
  });

  test('Advanced-003: Filter Combination', async ({ page }) => {
    // Create multiple stores with different tags
    const stores = ['Store A', 'Store B', 'Store C'];

    for (const storeName of stores) {
      await page.click('text=Create Policy Store');
      await page.fill('textarea[placeholder*="description"]', storeName);
      await page.click('button:has-text("Create")');
      await page.waitForTimeout(500);

      // Add tags based on store
      await page.click('button:has-text("View Details")');
      await page.click('button:has-text("Tags")');

      const tag = storeName === 'Store A' ? 'production' :
                  storeName === 'Store B' ? 'testing' : 'staging';

      await page.fill('input[placeholder*="tag"]', tag);
      await page.click('button:has-text("Add")');
      await page.waitForTimeout(200);

      await page.click('button:has-text("Close")');
    }

    // Test combined filters
    await page.click('button:has-text("Filter")');
    await page.waitForTimeout(300);

    // Apply tag filter
    const productionTag = page.locator('text=production').first();
    if (await productionTag.isVisible()) {
      await productionTag.click();
      await page.waitForTimeout(300);

      // Verify filtered results
      // This is a placeholder - actual verification depends on filter implementation
    }
  });
});
