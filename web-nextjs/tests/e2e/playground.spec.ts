import { test, expect } from "@playwright/test";

test.describe("Playground - Fase 3 Features", () => {
  test.beforeEach(async ({ page }) => {
    await page.goto("/playground");
    await page.waitForLoadState("domcontentloaded");
    await page.waitForSelector("text=Playground", { timeout: 10000 });
  });

  test("should display playground with tabs", async ({ page }) => {
    // Check for Single Test and Batch Test tabs (using text() for broader compatibility)
    await expect(page.locator('text="Single Test"')).toBeVisible();
    await expect(page.locator('text="Batch Test"')).toBeVisible();

    // Single Test should be active by default
    const singleTestButton = page.locator('text="Single Test"');
    await expect(singleTestButton).toBeVisible();
  });

  test("should configure and run single test", async ({ page }) => {
    // Fill in scenario configuration using IDs from the Playground component
    await page.fill("#scenario-name", "Test Scenario");
    await page.fill("#policy-store", "ps_test");

    // Configure principal by finding the inputs with labels
    const principalType = page
      .locator("input")
      .filter({ hasText: "User" })
      .first();
    const principalId = page
      .locator("input")
      .filter({ hasText: "alice" })
      .first();

    await principalType.clear();
    await principalType.fill("User");
    await principalId.clear();
    await principalId.fill("alice");

    // Configure action
    const actionType = page
      .locator("input")
      .filter({ hasText: "Action" })
      .first();
    const actionId = page
      .locator("input")
      .filter({ hasText: "viewDocument" })
      .first();

    await actionType.clear();
    await actionType.fill("Action");
    await actionId.clear();
    await actionId.fill("viewDocument");

    // Configure resource
    const resourceType = page
      .locator("input")
      .filter({ hasText: "Document" })
      .first();
    const resourceId = page
      .locator("input")
      .filter({ hasText: "doc123" })
      .first();

    await resourceType.clear();
    await resourceType.fill("Document");
    await resourceId.clear();
    await resourceId.fill("doc123");

    // Run test
    await page.click('text="Run Test"');

    // Wait for results
    await page.waitForTimeout(2000);

    // Verify results are displayed
    const results = page.locator("text=Decision:");
    await expect(results).toBeVisible();
  });

  test("should save scenario", async ({ page }) => {
    // Configure scenario
    await page.fill("#scenario-name", "Save Test Scenario");

    // Click Save Scenario
    await page.click('text="Save Scenario"');

    // Wait for save operation
    await page.waitForTimeout(1000);

    // Verify scenario appears in saved list
    await expect(page.locator("text=Save Test Scenario")).toBeVisible();
  });

  test("should load saved scenario", async ({ page }) => {
    // First, save a scenario
    await page.fill("#scenario-name", "Load Test Scenario");
    await page.click('text="Save Scenario"');
    await page.waitForTimeout(1000);

    // Load the scenario from saved list (using Load icon button)
    const loadButtons = page
      .locator("button")
      .filter({ has: page.locator("svg") });
    await loadButtons.first().click();

    // Verify configuration was loaded
    await expect(page.locator("#scenario-name")).toHaveValue(
      "Load Test Scenario",
    );
  });

  test("should enable and use debug mode", async ({ page }) => {
    // Enable debug mode (button text changes between "Enable" and "Disable")
    await page.click('text="Enable Debug"');

    // Debug panel should appear
    await expect(page.locator("text=Debug Mode")).toBeVisible();

    // Run test with debug enabled
    await page.fill("#scenario-name", "Debug Test Scenario");
    await page.click('text="Run Test"');

    // Wait for debug steps to execute
    await page.waitForTimeout(3000);

    // Verify debug steps are displayed
    const debugSteps = page.locator("text=Step 1");
    await expect(debugSteps).toBeVisible();

    // Should show step descriptions
    await expect(
      page.locator("text=Parse authorization request"),
    ).toBeVisible();
  });

  test("should expand debug step details", async ({ page }) => {
    // Enable debug mode
    await page.click('text="Enable Debug"');

    // Run test
    await page.fill("#scenario-name", "Debug Expand Test");
    await page.click('text="Run Test"');
    await page.waitForTimeout(3000);

    // Click on a debug step to expand
    const firstStep = page
      .locator("text=Step 1")
      .locator('xpath=./ancestor::*[contains(@class, "border")]');
    await firstStep.click();

    // Verify details are shown
    await expect(page.locator("text=Details:")).toBeVisible();
  });

  test("should switch to batch test tab", async ({ page }) => {
    // Click Batch Test tab
    await page.click('text="Batch Test"');

    // Verify batch test UI is visible
    await expect(
      page.locator("text=Batch Authorization Testing"),
    ).toBeVisible();
    await expect(page.locator("text=Predefined Test Suites")).toBeVisible();
  });

  test("should run predefined batch test suites", async ({ page }) => {
    // Switch to batch test tab
    await page.click('text="Batch Test"');

    // Run User Access Tests
    await page.click('text="User Access Tests"');

    // Wait for batch test to complete
    await page.waitForTimeout(5000);

    // Verify results are displayed
    await expect(page.locator("text=Test Summary")).toBeVisible();

    // Check statistics
    const totalTests = page.locator("text=Total Tests");
    await expect(totalTests).toBeVisible();

    // Verify results table exists
    const resultsTable = page.locator("table");
    await expect(resultsTable).toBeVisible();
  });

  test("should run role-based batch test", async ({ page }) => {
    // Switch to batch test tab
    await page.click('text="Batch Test"');

    // Run Role-Based Tests
    await page.click('text="Role-Based Tests"');

    // Wait for batch test to complete
    await page.waitForTimeout(5000);

    // Verify results
    await expect(page.locator("text=Test Summary")).toBeVisible();
    await expect(page.locator("text=ALLOW")).toBeVisible();
    await expect(page.locator("text=DENY")).toBeVisible();
  });

  test("should configure and run custom batch test", async ({ page }) => {
    // Switch to batch test tab
    await page.click('text="Batch Test"');

    // Set number of custom scenarios
    await page.fill('input[type="number"]', "3");

    // Run custom test
    await page.click('text="Run Custom Test"');

    // Wait for batch test to complete
    await page.waitForTimeout(5000);

    // Verify custom scenarios were tested
    await expect(page.locator("text=Custom Scenario")).toBeVisible();
  });

  test("should export batch test results", async ({ page }) => {
    // Switch to batch test tab
    await page.click('text="Batch Test"');

    // Run a test first
    await page.click('text="User Access Tests"');
    await page.waitForTimeout(5000);

    // Export results
    const downloadPromise = page.waitForEvent("download");
    await page.click('text="Export Results"');
    const download = await downloadPromise;

    // Verify CSV file was downloaded
    expect(download.suggestedFilename()).toMatch(/\.csv$/);
  });

  test("should display batch test statistics", async ({ page }) => {
    // Switch to batch test tab
    await page.click('text="Batch Test"');

    // Run test
    await page.click('text="User Access Tests"');
    await page.waitForTimeout(5000);

    // Verify statistics cards
    await expect(page.locator("text=Total Tests")).toBeVisible();
    await expect(page.locator("text=Successful")).toBeVisible();
    await expect(page.locator("text=Failed")).toBeVisible();
    await expect(page.locator("text=Avg Latency")).toBeVisible();

    // Verify decision breakdown (allow/deny counts)
    await expect(page.locator("text=ALLOW")).toBeVisible();
    await expect(page.locator("text=DENY")).toBeVisible();
  });

  test("should show loading state during test execution", async ({ page }) => {
    // Configure and start a test
    await page.fill("#scenario-name", "Loading Test Scenario");
    await page.click('text="Run Test"');

    // Verify loading text appears
    await expect(page.locator("text=Running...")).toBeVisible();

    // Wait for test to complete
    await page.waitForTimeout(2000);

    // Loading should disappear or show results
    const hasResults = (await page.locator("text=Decision:").count()) > 0;
    const stillLoading = (await page.locator("text=Running...").count()) > 0;

    expect(hasResults || !stillLoading).toBeTruthy();
  });

  test("should handle test errors gracefully", async ({ page }) => {
    // Run test (it has default values so should work)
    await page.click('text="Run Test"');
    await page.waitForTimeout(2000);

    // Should either show results or error message
    const hasResults = (await page.locator("text=Decision:").count()) > 0;
    const hasTestResults =
      (await page.locator("text=Test Results").count()) > 0;

    expect(hasResults || hasTestResults).toBeTruthy();
  });
});
