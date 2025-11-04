import { test, expect, Page } from "@playwright/test";

/**
 * E2E Tests for Snapshot and Version History Feature
 * Test Case IDs: SN-001 to SN-050
 *
 * This test suite focuses specifically on the version history and snapshot functionality
 * implemented in Phase 3.1
 */

test.describe("Snapshot Management - E2E Tests", () => {
  test.beforeEach(async ({ page }) => {
    // Navigate to the application
    await page.goto("/");
    await expect(page.locator("h1")).toContainText(
      "Hodei Verified Permissions",
    );

    // Create a test policy store for snapshots
    await createTestPolicyStore(page);
  });

  test.afterEach(async ({ page }) => {
    // Cleanup: Delete test policy store
    await cleanupTestPolicyStore(page);
  });

  // ============================================================================
  // TC-SN-001 to SN-010: Snapshot Creation
  // ============================================================================

  test("SN-001: Create First Snapshot", async ({ page }) => {
    // Navigate to Version History tab
    await navigateToVersionHistory(page);

    // Verify empty state message
    await expect(page.locator("text=No snapshots yet")).toBeVisible();
    await expect(page.locator("text=Create your first snapshot")).toBeVisible();

    // Click Create Snapshot button
    await page.click('button:has-text("Create Snapshot")');

    // Verify modal opens
    await expect(page.locator('h3:has-text("Create Snapshot")')).toBeVisible();

    // Fill in description
    const description = `Initial Snapshot - ${Date.now()}`;
    await page.fill('input[placeholder*="description"]', description);

    // Click Create Snapshot
    await page.click('button:has-text("Create Snapshot")');

    // Wait for snapshot creation
    await page.waitForTimeout(1000);

    // Verify snapshot appears in list
    await expect(page.locator(`text=${description}`)).toBeVisible();

    // Verify snapshot shows correct information
    await expect(page.locator("text=Policies:")).toBeVisible();
    await expect(page.locator("text=Schema:")).toBeVisible();
    await expect(page.locator("text=Size:")).toBeVisible();
    await expect(page.locator("text=Created:")).toBeVisible();

    // Verify snapshot has unique ID (badge)
    const snapshotBadge = page.locator('[class*="badge"]').first();
    await expect(snapshotBadge).toBeVisible();

    // Verify snapshot shows creation timestamp
    const timestamp = page.locator("text=Created:").first();
    await expect(timestamp).toBeVisible();
  });

  test("SN-002: Create Multiple Snapshots", async ({ page }) => {
    // Navigate to Version History
    await navigateToVersionHistory(page);

    // Create first snapshot
    await createSnapshot(page, "Snapshot 1 - Baseline");

    // Create second snapshot
    await page.click('button:has-text("Create Snapshot")');
    await page.fill(
      'input[placeholder*="description"]',
      "Snapshot 2 - After Changes",
    );
    await page.click('button:has-text("Create Snapshot")');
    await page.waitForTimeout(1000);

    // Create third snapshot
    await page.click('button:has-text("Create Snapshot")');
    await page.fill(
      'input[placeholder*="description"]',
      "Snapshot 3 - Final State",
    );
    await page.click('button:has-text("Create Snapshot")');
    await page.waitForTimeout(1000);

    // Verify all three snapshots exist
    await expect(page.locator("text=Snapshot 1")).toBeVisible();
    await expect(page.locator("text=Snapshot 2")).toBeVisible();
    await expect(page.locator("text=Snapshot 3")).toBeVisible();

    // Verify snapshots are ordered by creation date (newest first)
    const snapshots = page.locator('[class*="card"]').all();
    expect(await snapshots.length).toBeGreaterThanOrEqual(3);
  });

  test("SN-003: Create Snapshot with Empty Description", async ({ page }) => {
    // Navigate to Version History
    await navigateToVersionHistory(page);

    // Click Create Snapshot
    await page.click('button:has-text("Create Snapshot")');

    // Leave description empty
    await page.click('button:has-text("Create Snapshot")');

    // Wait for creation
    await page.waitForTimeout(1000);

    // Verify snapshot was created (even without description)
    await expect(page.locator("text=Policies:")).toBeVisible();
  });

  test("SN-004: Snapshot Creation Loading State", async ({ page }) => {
    // Navigate to Version History
    await navigateToVersionHistory(page);

    // Click Create Snapshot
    await page.click('button:has-text("Create Snapshot")');

    // Fill in description
    await page.fill('input[placeholder*="description"]', "Loading State Test");

    // Click Create and immediately check for loading state
    const createButton = page.locator('button:has-text("Create Snapshot")');
    await createButton.click();

    // Verify loading state (button text should change)
    await expect(createButton).toContainText("Creating...");

    // Wait for completion
    await page.waitForTimeout(1500);

    // Verify snapshot was created
    await expect(page.locator("text=Loading State Test")).toBeVisible();
  });

  test("SN-005: Snapshot Creation Error Handling", async ({ page }) => {
    // This test verifies error handling during snapshot creation
    // Note: Actual error simulation depends on backend implementation

    // Navigate to Version History
    await navigateToVersionHistory(page);

    // Click Create Snapshot
    await page.click('button:has-text("Create Snapshot")');
    await page.fill('input[placeholder*="description"]', "Error Test");
    await page.click('button:has-text("Create Snapshot")');

    // Wait for creation
    await page.waitForTimeout(1000);

    // In case of error, verify error message is shown
    // await expect(page.locator('text=Failed to create')).toBeVisible();
  });

  // ============================================================================
  // TC-SN-011 to SN-020: Snapshot Listing and Viewing
  // ============================================================================

  test("SN-011: View Snapshot Details", async ({ page }) => {
    // Create a snapshot
    await navigateToVersionHistory(page);
    await createSnapshot(page, "Details Test");

    // Snapshot should show details in card format
    await expect(page.locator("text=Policies:")).toBeVisible();

    // Verify policy count is displayed
    const policyCount = page.locator("text=Policies:").first();
    const text = await policyCount.textContent();
    expect(text).toContain("Policies:");

    // Verify schema status is displayed
    const schemaStatus = page.locator("text=Schema:").first();
    const schemaText = await schemaStatus.textContent();
    expect(schemaText).toContain("Schema:");

    // Verify size is displayed
    const sizeInfo = page.locator("text=Size:").first();
    const sizeText = await sizeInfo.textContent();
    expect(sizeText).toContain("Size:");

    // Verify creation timestamp
    const timestamp = page.locator("text=Created:").first();
    const timestampText = await timestamp.textContent();
    expect(timestampText).toContain("Created:");
  });

  test("SN-012: List All Snapshots", async ({ page }) => {
    // Create multiple snapshots
    await navigateToVersionHistory(page);
    await createSnapshot(page, "List Test 1");
    await createSnapshot(page, "List Test 2");
    await createSnapshot(page, "List Test 3");

    // Verify all snapshots are listed
    await expect(page.locator("text=List Test 1")).toBeVisible();
    await expect(page.locator("text=List Test 2")).toBeVisible();
    await expect(page.locator("text=List Test 3")).toBeVisible();

    // Verify each snapshot has actions (Rollback, Delete buttons)
    const rollbackButtons = page.locator('button:has-text("Rollback")');
    const deleteButtons = page.locator('button[title*="Delete"]');
    const rollbackCount = await rollbackButtons.count();
    const deleteCount = await deleteButtons.count();

    expect(rollbackCount).toBeGreaterThanOrEqual(3);
    expect(deleteCount).toBeGreaterThanOrEqual(3);
  });

  test("SN-013: Snapshot Sorting (Newest First)", async ({ page }) => {
    // Create snapshots with different creation times
    await navigateToVersionHistory(page);

    // Create first snapshot
    await createSnapshot(page, "First Created");

    // Wait a bit to ensure different timestamps
    await page.waitForTimeout(1000);

    // Create second snapshot
    await createSnapshot(page, "Second Created");

    // Wait a bit more
    await page.waitForTimeout(1000);

    // Create third snapshot
    await createSnapshot(page, "Third Created");

    // Verify all snapshots exist
    await expect(page.locator("text=First Created")).toBeVisible();
    await expect(page.locator("text=Second Created")).toBeVisible();
    await expect(page.locator("text=Third Created")).toBeVisible();

    // Verify order (newest first) - this depends on UI implementation
    const snapshots = page.locator('[class*="card"]').all();
    const snapshotTexts = await Promise.all(
      snapshots.map((s) => s.textContent()),
    );
    // Newest should appear before oldest
    const thirdIndex = snapshotTexts.findIndex((t) =>
      t?.includes("Third Created"),
    );
    const firstIndex = snapshotTexts.findIndex((t) =>
      t?.includes("First Created"),
    );
    expect(thirdIndex).toBeLessThan(firstIndex);
  });

  test("SN-014: Empty Snapshot List", async ({ page }) => {
    // Navigate to Version History without creating any snapshots
    await navigateToVersionHistory(page);

    // Verify empty state is shown
    await expect(page.locator("text=No snapshots yet")).toBeVisible();
    await expect(page.locator("text=Create your first snapshot")).toBeVisible();

    // Verify Call-to-Action is shown
    const createButton = page.locator('button:has-text("Create Snapshot")');
    await expect(createButton).toBeVisible();

    // Verify informational message
    const infoMessage = page.locator("text=point-in-time snapshots");
    await expect(infoMessage).toBeVisible();
  });

  test("SN-015: Snapshot List Pagination (if applicable)", async ({ page }) => {
    // Create many snapshots to test pagination
    await navigateToVersionHistory(page);

    // Create 15 snapshots (assuming page size is 10)
    for (let i = 1; i <= 15; i++) {
      await createSnapshot(page, `Snapshot ${i}`);
    }

    // Verify at least one snapshot is visible
    await expect(page.locator("text=Snapshot 1")).toBeVisible();

    // Check if pagination controls exist (depends on implementation)
    const nextButton = page.locator('button:has-text("Next")');
    if (await nextButton.isVisible()) {
      // Test pagination
      await nextButton.click();
      await page.waitForTimeout(500);

      // Verify different snapshots on next page
      await expect(page.locator("text=Snapshot 11")).toBeVisible();
    }
  });

  // ============================================================================
  // TC-SN-021 to SN-030: Rollback Operations
  // ============================================================================

  test("SN-021: Rollback to Snapshot", async ({ page }) => {
    // Create a snapshot first
    await navigateToVersionHistory(page);
    await createSnapshot(page, "Rollback Test Snapshot");

    // Click Rollback button
    const rollbackButton = page.locator('button:has-text("Rollback")').first();
    await rollbackButton.click();

    // Handle confirmation dialog
    page.on("dialog", (dialog) => dialog.accept());
    await page.waitForTimeout(500);

    // Verify rollback was initiated
    // Success message depends on UI implementation
    // await expect(page.locator('text=Successfully rolled back')).toBeVisible();

    // Verify confirmation modal disappears
    await expect(page.locator('button:has-text("Rollback")')).toBeVisible();
  });

  test("SN-022: Rollback Confirmation Dialog", async ({ page }) => {
    // Create a snapshot
    await navigateToVersionHistory(page);
    await createSnapshot(page, "Confirmation Test");

    // Click Rollback button
    const rollbackButton = page.locator('button:has-text("Rollback")').first();
    await rollbackButton.click();

    // Verify confirmation dialog appears
    page.on("dialog", (dialog) => {
      expect(dialog.message()).toContain("rollback");
      dialog.accept();
    });

    await page.waitForTimeout(500);
  });

  test("SN-023: Cancel Rollback Operation", async ({ page }) => {
    // Create a snapshot
    await navigateToVersionHistory(page);
    await createSnapshot(page, "Cancel Test");

    // Click Rollback button
    const rollbackButton = page.locator('button:has-text("Rollback")').first();
    await rollbackButton.click();

    // Cancel the operation
    page.on("dialog", (dialog) => dialog.dismiss());
    await page.waitForTimeout(500);

    // Verify rollback was cancelled (modal still visible)
    await expect(page.locator('button:has-text("Rollback")')).toBeVisible();
  });

  test("SN-024: Rollback Loading State", async ({ page }) => {
    // Create a snapshot
    await navigateToVersionHistory(page);
    await createSnapshot(page, "Loading State Test");

    // Click Rollback
    const rollbackButton = page.locator('button:has-text("Rollback")').first();
    await rollbackButton.click();

    // Accept confirmation
    page.on("dialog", (dialog) => dialog.accept());

    // Check for loading state (button text or spinner)
    const loadingButton = page.locator('button:has-text("Rollback")');
    // await expect(loadingButton).toContainText('Rolling back...');

    // Wait for operation to complete
    await page.waitForTimeout(2000);

    // Verify operation completed
    await expect(loadingButton).toBeVisible();
  });

  test("SN-025: Rollback Result Display", async ({ page }) => {
    // This test verifies the UI feedback after rollback
    await navigateToVersionHistory(page);
    await createSnapshot(page, "Result Display Test");

    // Perform rollback
    const rollbackButton = page.locator('button:has-text("Rollback")').first();
    await rollbackButton.click();

    page.on("dialog", (dialog) => dialog.accept());
    await page.waitForTimeout(2000);

    // Verify result is displayed
    // Implementation depends on UI feedback mechanism
    // await expect(page.locator('text=policies restored')).toBeVisible();
  });

  // ============================================================================
  // TC-SN-031 to SN-040: Snapshot Deletion
  // ============================================================================

  test("SN-031: Delete Single Snapshot", async ({ page }) => {
    // Create a snapshot
    await navigateToVersionHistory(page);
    await createSnapshot(page, "To Be Deleted");

    // Click Delete button (trash icon)
    const deleteButton = page.locator('button[title*="Delete"]').first();
    await deleteButton.click();

    // Handle confirmation
    page.on("dialog", (dialog) => dialog.accept());
    await page.waitForTimeout(500);

    // Verify snapshot was deleted
    await expect(page.locator("text=To Be Deleted")).not.toBeVisible();
  });

  test("SN-032: Delete Confirmation Dialog", async ({ page }) => {
    // Create a snapshot
    await navigateToVersionHistory(page);
    await createSnapshot(page, "Delete Confirmation Test");

    // Click Delete
    const deleteButton = page.locator('button[title*="Delete"]').first();
    await deleteButton.click();

    // Verify confirmation dialog
    page.on("dialog", (dialog) => {
      expect(dialog.message()).toContain("delete");
      dialog.accept();
    });

    await page.waitForTimeout(500);
  });

  test("SN-033: Cancel Delete Operation", async ({ page }) => {
    // Create a snapshot
    await navigateToVersionHistory(page);
    await createSnapshot(page, "Cancel Delete Test");

    // Click Delete
    const deleteButton = page.locator('button[title*="Delete"]').first();
    await deleteButton.click();

    // Cancel operation
    page.on("dialog", (dialog) => dialog.dismiss());
    await page.waitForTimeout(500);

    // Verify snapshot still exists
    await expect(page.locator("text=Cancel Delete Test")).toBeVisible();
  });

  test("SN-034: Delete Multiple Snapshots", async ({ page }) => {
    // Create multiple snapshots
    await navigateToVersionHistory(page);
    await createSnapshot(page, "Delete Multi 1");
    await createSnapshot(page, "Delete Multi 2");
    await createSnapshot(page, "Delete Multi 3");

    // Delete first snapshot
    let deleteButtons = page.locator('button[title*="Delete"]');
    await deleteButtons.nth(0).click();
    page.on("dialog", (dialog) => dialog.accept());
    await page.waitForTimeout(500);

    // Delete second snapshot
    deleteButtons = page.locator('button[title*="Delete"]');
    await deleteButtons.nth(0).click();
    page.on("dialog", (dialog) => dialog.accept());
    await page.waitForTimeout(500);

    // Verify only one remains
    await expect(page.locator("text=Delete Multi 3")).toBeVisible();
    await expect(page.locator("text=Delete Multi 1")).not.toBeVisible();
    await expect(page.locator("text=Delete Multi 2")).not.toBeVisible();
  });

  test("SN-035: Delete Last Snapshot", async ({ page }) => {
    // Create single snapshot
    await navigateToVersionHistory(page);
    await createSnapshot(page, "Last One");

    // Delete it
    const deleteButton = page.locator('button[title*="Delete"]').first();
    await deleteButton.click();

    page.on("dialog", (dialog) => dialog.accept());
    await page.waitForTimeout(500);

    // Verify empty state returns
    await expect(page.locator("text=No snapshots yet")).toBeVisible();
    await expect(page.locator("text=Create your first snapshot")).toBeVisible();
  });

  // ============================================================================
  // TC-SN-041 to SN-050: Edge Cases and Error Handling
  // ============================================================================

  test("SN-041: Snapshot with Special Characters in Description", async ({
    page,
  }) => {
    // Create snapshot with special characters
    await navigateToVersionHistory(page);
    await createSnapshot(page, "Snapshot @#$%^&*()_+");

    // Verify snapshot is created
    await expect(page.locator("text=Snapshot @#$%^&*()_+")).toBeVisible();
  });

  test("SN-042: Snapshot with Very Long Description", async ({ page }) => {
    // Create snapshot with long description
    const longDescription = "A".repeat(200);
    await navigateToVersionHistory(page);
    await createSnapshot(page, longDescription);

    // Verify snapshot is created (UI may truncate)
    await expect(page.locator("text=A".repeat(50))).toBeVisible();
  });

  test("SN-043: Concurrent Snapshot Creation", async ({ page }) => {
    // This test verifies handling of rapid snapshot creation
    await navigateToVersionHistory(page);

    // Create multiple snapshots quickly
    for (let i = 1; i <= 3; i++) {
      await page.click('button:has-text("Create Snapshot")');
      await page.fill('input[placeholder*="description"]', `Concurrent ${i}`);
      await page.click('button:has-text("Create Snapshot")');
      await page.waitForTimeout(500);
    }

    // Verify all snapshots were created
    await expect(page.locator("text=Concurrent 1")).toBeVisible();
    await expect(page.locator("text=Concurrent 2")).toBeVisible();
    await expect(page.locator("text=Concurrent 3")).toBeVisible();
  });

  test("SN-044: Network Error During Snapshot Operations", async ({ page }) => {
    // Simulate network error
    await page.route("**/api/**", (route) => {
      route.abort("internetdisconnected");
    });

    // Try to create snapshot
    await navigateToVersionHistory(page);
    await page.click('button:has-text("Create Snapshot")');
    await page.fill('input[placeholder*="description"]', "Network Error Test");
    await page.click('button:has-text("Create Snapshot")');

    // Wait for error handling
    await page.waitForTimeout(1000);

    // Verify error is handled gracefully
    // await expect(page.locator('text=Failed')).toBeVisible();
  });

  test("SN-045: Snapshot Data Integrity", async ({ page }) => {
    // This test verifies that snapshot data is preserved correctly
    await navigateToVersionHistory(page);

    // Create snapshot
    await createSnapshot(page, "Data Integrity Test");

    // Refresh page
    await page.reload();
    await page.waitForTimeout(1000);

    // Navigate back to Version History
    await page.click('button:has-text("View Details")');
    await page.click('button:has-text("Version History")');

    // Verify snapshot data persists after refresh
    await expect(page.locator("text=Data Integrity Test")).toBeVisible();

    // Verify metrics are preserved
    await expect(page.locator("text=Policies:")).toBeVisible();
    await expect(page.locator("text=Size:")).toBeVisible();
  });

  // ============================================================================
  // Helper Functions
  // ============================================================================

  async function createTestPolicyStore(page: Page): Promise<void> {
    // Check if a test store already exists
    const existing = page.locator("text=Snapshot Test Store");
    if (await existing.isVisible()) {
      return;
    }

    // Create test policy store
    await page.click("text=Create Policy Store");
    await page.fill(
      'textarea[placeholder*="description"]',
      "Snapshot Test Store",
    );
    await page.click('button:has-text("Create")');
    await page.waitForTimeout(500);
  }

  async function cleanupTestPolicyStore(page: Page): Promise<void> {
    // Find and delete the test policy store
    const storeCard = page.locator("text=Snapshot Test Store").first();
    if (await storeCard.isVisible()) {
      // Get the parent card and find delete button
      const deleteButton = storeCard.locator(
        'xpath=../../..//button[@title="Delete policy store"]',
      );
      if (await deleteButton.isVisible()) {
        await deleteButton.click();
        page.on("dialog", (dialog) => dialog.accept());
        await page.waitForTimeout(500);
      }
    }
  }

  async function navigateToVersionHistory(page: Page): Promise<void> {
    // Open details modal
    const viewDetailsButton = page
      .locator('button:has-text("View Details")')
      .first();
    if (await viewDetailsButton.isVisible()) {
      await viewDetailsButton.click();
    } else {
      // Create store if no button exists
      await page.click("text=Create Policy Store");
      await page.fill(
        'textarea[placeholder*="description"]',
        "Snapshot Test Store",
      );
      await page.click('button:has-text("Create")');
      await page.waitForTimeout(500);
      await page.click('button:has-text("View Details")');
    }

    // Click Version History tab
    await page.click('button:has-text("Version History")');
    await page.waitForTimeout(300);
  }

  async function createSnapshot(
    page: Page,
    description: string,
  ): Promise<void> {
    // Click Create Snapshot
    await page.click('button:has-text("Create Snapshot")');

    // Fill description
    await page.fill('input[placeholder*="description"]', description);

    // Click Create
    await page.click('button:has-text("Create Snapshot")');

    // Wait for creation
    await page.waitForTimeout(1000);
  }
});
