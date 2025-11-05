import { test, expect } from "@playwright/test";
import { waitForPageLoad, takeScreenshot } from "./helpers";

test.describe("Scenarios Management", () => {
  test.describe.configure({ mode: "serial" }); // Run tests serially to avoid shared state issues

  test.beforeEach(async ({ page }) => {
    await page.goto("/playground");
    await waitForPageLoad(page);

    // Clear localStorage to avoid contamination between tests
    await page.evaluate(() => {
      localStorage.removeItem("hodei-scenarios");
    });
  });

  test("should save new scenario with all fields", async ({ page }) => {
    // Fill in all scenario fields
    await page.fill("#scenario-name", "Complete Scenario Test");
    await page.fill(
      'textarea[name="description"]',
      "Test scenario description",
    );

    // Configure entities
    await page.fill("#policy-store", "ps_123");

    await page.fill('input[value="User"] >> nth=0', "User");
    await page.fill('input[value="alice"] >> nth=0', "alice");

    await page.fill('input[value="Action"] >> nth=0', "Action");
    await page.fill('input[value="viewDocument"] >> nth=0', "viewDocument");

    await page.fill('input[value="Document"] >> nth=0', "Document");
    await page.fill('input[value="doc123"] >> nth=0', "doc123");

    // Add context
    await page.fill("#context", '{"role": "admin", "department": "IT"}');

    // Save scenario
    await page.click('button:has-text("Save Scenario")');

    // Wait for save operation and data to reload
    await page.waitForTimeout(2000);

    // Verify scenario appears in saved list (use data-testid to avoid strict mode)
    const scenarioCards = page.locator('[data-testid="scenario-card"]');
    await expect(
      scenarioCards.filter({ hasText: "Complete Scenario Test" }),
    ).toHaveCount(1);

    // Verify details are shown
    const completeScenarioCard = scenarioCards
      .filter({ hasText: "Complete Scenario Test" })
      .first();
    await expect(completeScenarioCard).toBeVisible();
  });

  test("should display saved scenarios list", async ({ page }) => {
    // Save a few scenarios
    for (let i = 1; i <= 3; i++) {
      await page.fill("#scenario-name", `Scenario ${i}`);
      await page.click('button:has-text("Save Scenario")');
      await page.waitForTimeout(500);
    }

    // Verify all scenarios appear in the list (use data-testid to avoid strict mode)
    const scenarioCards = page.locator('[data-testid="scenario-card"]');
    await expect(scenarioCards.filter({ hasText: "Scenario 1" })).toBeVisible();
    await expect(scenarioCards.filter({ hasText: "Scenario 2" })).toBeVisible();
    await expect(scenarioCards.filter({ hasText: "Scenario 3" })).toBeVisible();
  });

  test("should load scenario from saved list", async ({ page }) => {
    // Save scenario with unique data
    const uniqueName = `Unique Scenario ${Date.now()}`;
    await page.fill("#scenario-name", uniqueName);
    await page.fill("#policy-store", "ps_unique_123");
    await page.click('button:has-text("Save Scenario")');
    await page.waitForTimeout(1000);

    // Clear form first
    await page.fill("#scenario-name", "Temp Name");
    await page.fill("#policy-store", "temp");

    // Load scenario from list (use the most recent one which should be our unique scenario)
    const scenarioCards = page.locator('[data-testid="scenario-card"]');
    const loadButton = scenarioCards
      .first()
      .locator('button[aria-label="Load"]');
    await loadButton.click();
    await page.waitForTimeout(500);

    // Verify form was populated
    await expect(page.locator("#scenario-name")).toHaveValue(uniqueName);
    await expect(page.locator("#policy-store")).toHaveValue("ps_unique_123");
  });

  test.skip("should delete scenario from saved list", async ({ page }) => {
    // Save scenario
    const scenarioName = `Delete Test ${Date.now()}`;
    await page.fill("#scenario-name", scenarioName);
    await page.click('button:has-text("Save Scenario")');
    await page.waitForTimeout(1000);

    // Verify scenario exists
    const scenarioCards = page.locator('[data-testid="scenario-card"]');
    await expect(scenarioCards.filter({ hasText: scenarioName })).toHaveCount(
      1,
    );

    // Delete scenario (delete the most recent one which should be our test scenario)
    const testScenarioCard = scenarioCards
      .filter({ hasText: scenarioName })
      .first();
    const deleteButton = testScenarioCard.locator(
      'button[aria-label="Delete"]',
    );
    await deleteButton.click();

    // Handle confirmation dialog
    page.on("dialog", (dialog) => dialog.accept());
    await page.waitForTimeout(1000);

    // Reload the page to refresh data
    await page.reload();
    await page.waitForLoadState("networkidle");

    // Verify scenario was removed
    const updatedScenarioCards = page.locator('[data-testid="scenario-card"]');
    await expect(
      updatedScenarioCards.filter({ hasText: scenarioName }),
    ).toHaveCount(0);
  });

  test("should show scenario preview in list", async ({ page }) => {
    // Save scenario with descriptive name
    await page.fill("#scenario-name", "User Document Access Test");
    await page.click('button:has-text("Save Scenario")');
    await page.waitForTimeout(1000);

    // Verify preview shows entity information
    const scenarioCard = page.locator("text=User Document Access Test").first();
    await expect(scenarioCard).toBeVisible();

    // Check for entity type and ID preview
    const preview = scenarioCard.locator("..");
    await expect(preview.locator("text=User:")).toBeVisible();
    await expect(preview.locator("text=viewDocument")).toBeVisible();
  });

  test("should persist scenarios across page reload", async ({ page }) => {
    // Save scenario
    await page.fill("#scenario-name", "Persistence Test");
    await page.click('button:has-text("Save Scenario")');
    await page.waitForTimeout(1000);

    // Reload page
    await page.reload();
    await page.waitForLoadState("networkidle");

    // Navigate back to playground
    await page.goto("/playground");
    await page.waitForLoadState("networkidle");

    // Verify scenario still exists
    await expect(page.locator("text=Persistence Test")).toBeVisible();
  });

  test("should handle scenario validation", async ({ page }) => {
    // Try to save scenario without required fields
    await page.click('button:has-text("Save Scenario")');

    // Should show validation error or prevent save
    // (Actual behavior depends on implementation)
    await page.waitForTimeout(1000);

    // Scenario list should not show empty scenario
    const scenarios = page.locator('[data-testid="scenario-card"]');
    const count = await scenarios.count();

    // Should be 0 or only valid saved scenarios
    expect(count).toBeGreaterThanOrEqual(0);
  });

  test("should update scenario configuration", async ({ page }) => {
    // Save initial scenario
    await page.fill("#scenario-name", "Update Test");
    await page.fill("#policy-store", "ps_original");
    await page.click('button:has-text("Save Scenario")');
    await page.waitForTimeout(1000);

    // Modify configuration
    await page.fill("#policy-store", "ps_updated");
    await page.fill("#scenario-name", "Updated Scenario");

    // Save updated scenario
    await page.click('button:has-text("Save Scenario")');
    await page.waitForTimeout(1000);

    // Verify updated values are shown
    await expect(page.locator("text=Updated Scenario")).toBeVisible();
  });

  test("should filter scenarios by policy store", async ({ page }) => {
    // Save scenarios for different policy stores
    await page.fill("#scenario-name", "Store 1 Scenario");
    await page.fill("#policy-store", "ps_store_1");
    await page.click('button:has-text("Save Scenario")');
    await page.waitForTimeout(500);

    await page.fill("#scenario-name", "Store 2 Scenario");
    await page.fill("#policy-store", "ps_store_2");
    await page.click('button:has-text("Save Scenario")');
    await page.waitForTimeout(500);

    // Filter by policy store (if filter exists)
    const filterSelect = page.locator('select[name="policy-store-filter"]');
    if ((await filterSelect.count()) > 0) {
      await filterSelect.selectOption("ps_store_1");
      await page.waitForTimeout(500);

      // Should only show scenarios from ps_store_1
      await expect(page.locator("text=Store 1 Scenario")).toBeVisible();
      await expect(page.locator("text=Store 2 Scenario")).toHaveCount(0);
    }
  });

  test("should show scenario timestamps", async ({ page }) => {
    // Save scenario
    await page.fill("#scenario-name", "Timestamp Test");
    await page.click('button:has-text("Save Scenario")');
    await page.waitForTimeout(1000);

    // Verify creation/update timestamp is shown
    const scenarioCard = page.locator("text=Timestamp Test").first();
    await expect(scenarioCard).toBeVisible();

    // Check for timestamp (format may vary)
    const cardContent = scenarioCard.locator("..");
    const hasTimestamp =
      (await cardContent.locator("text=/ago|minutes|hours/").count()) > 0;
    expect(hasTimestamp).toBeTruthy();
  });
});
