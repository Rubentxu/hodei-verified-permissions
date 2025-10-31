import { test, expect } from '@playwright/test';

test.describe('Policy Editor', () => {
  test.beforeEach(async ({ page }) => {
    await page.goto('/');
    await page.click('button:has-text("Policies")');
  });

  test('should display policy editor page', async ({ page }) => {
    await expect(page.locator('h2')).toContainText('Policies');
    await expect(page.locator('text=Create and manage authorization policies')).toBeVisible();
  });

  test('should open policy creation wizard', async ({ page }) => {
    await page.click('button:has-text("Create Policy")');
    
    // Wait for modal to appear
    await page.waitForTimeout(500);
    
    // Check wizard is visible
    const wizardTitle = page.locator('text=Create New Policy');
    await expect(wizardTitle).toBeVisible();
  });

  test('should complete policy wizard', async ({ page }) => {
    await page.click('button:has-text("Create Policy")');
    await page.waitForTimeout(500);
    
    // Step 1: Basic Information
    await page.fill('input[placeholder="e.g., Document Access Policy"]', 'Test Policy');
    await page.fill('textarea[placeholder*="Describe"]', 'Test policy description');
    await page.selectOption('select', 'store-1');
    
    // Click Next
    await page.click('button:has-text("Next")');
    await page.waitForTimeout(500);
    
    // Step 2: Choose Template
    await page.click('text=Basic Access Control');
    
    // Click Next
    await page.click('button:has-text("Next")');
    await page.waitForTimeout(500);
    
    // Step 3: Entity Configuration
    await page.check('input[type="checkbox"]', { nth: 0 });
    
    // Click Next
    await page.click('button:has-text("Next")');
    await page.waitForTimeout(500);
    
    // Step 4: Review & Create
    const reviewText = page.locator('text=Policy Summary');
    await expect(reviewText).toBeVisible();
    
    // Click Create Policy
    await page.click('button:has-text("Create Policy")');
    await page.waitForTimeout(500);
    
    // Check policy was created
    const policyName = page.locator('text=Test Policy');
    await expect(policyName).toBeVisible();
  });

  test('should validate policy', async ({ page }) => {
    // Create a policy first
    await page.click('button:has-text("Create Policy")');
    await page.waitForTimeout(500);
    
    // Fill basic info
    await page.fill('input[placeholder="e.g., Document Access Policy"]', 'Validation Test');
    await page.fill('textarea[placeholder*="Describe"]', 'Test description');
    await page.selectOption('select', 'store-1');
    
    // Complete wizard
    await page.click('button:has-text("Next")');
    await page.waitForTimeout(300);
    await page.click('text=Basic Access Control');
    await page.click('button:has-text("Next")');
    await page.waitForTimeout(300);
    await page.check('input[type="checkbox"]', { nth: 0 });
    await page.click('button:has-text("Next")');
    await page.waitForTimeout(300);
    await page.click('button:has-text("Create Policy")');
    await page.waitForTimeout(500);
    
    // Now validate
    await page.click('button:has-text("Validate")');
    await page.waitForTimeout(1000);
    
    // Check validation result
    const validationResult = page.locator('text=valid').or(page.locator('text=errors'));
    await expect(validationResult).toBeVisible();
  });
});
