import { test, expect } from '@playwright/test';

test.describe('Templates System', () => {
  test.beforeEach(async ({ page }) => {
    await page.goto('/');
    await page.click('button:has-text("Templates")');
  });

  test('should display templates page', async ({ page }) => {
    await expect(page.locator('h2')).toContainText('Policy Templates');
    await expect(page.locator('text=Create and share reusable policy templates')).toBeVisible();
  });

  test('should open template creation modal', async ({ page }) => {
    await page.click('button:has-text("Create Template")');
    
    await page.waitForTimeout(500);
    
    const modalTitle = page.locator('text=Create New Template');
    await expect(modalTitle).toBeVisible();
  });

  test('should create new template', async ({ page }) => {
    await page.click('button:has-text("Create Template")');
    await page.waitForTimeout(500);
    
    // Fill template info
    await page.fill('input[placeholder="e.g., Document Access Template"]', 'Test Template');
    await page.fill('textarea[placeholder*="Describe"]', 'Test template description');
    
    // Click Create Template
    await page.click('button:has-text("Create Template")');
    await page.waitForTimeout(500);
    
    // Check template was created
    const templateName = page.locator('text=Test Template');
    await expect(templateName).toBeVisible();
  });

  test('should add template parameters', async ({ page }) => {
    await page.click('button:has-text("Create Template")');
    await page.waitForTimeout(500);
    
    // Fill basic info
    await page.fill('input[placeholder="e.g., Document Access Template"]', 'Param Test');
    await page.fill('textarea[placeholder*="Describe"]', 'Test with parameters');
    
    // Add parameter
    await page.click('button:has-text("Add Parameter")');
    await page.waitForTimeout(300);
    
    // Fill parameter info
    const paramInputs = page.locator('input[placeholder="Parameter name"]');
    await paramInputs.first().fill('resourceType');
    
    // Click Create Template
    await page.click('button:has-text("Create Template")');
    await page.waitForTimeout(500);
    
    // Verify template created
    const templateName = page.locator('text=Param Test');
    await expect(templateName).toBeVisible();
  });

  test('should search templates', async ({ page }) => {
    // Create a template first
    await page.click('button:has-text("Create Template")');
    await page.waitForTimeout(500);
    await page.fill('input[placeholder="e.g., Document Access Template"]', 'Search Test');
    await page.fill('textarea[placeholder*="Describe"]', 'Template for search');
    await page.click('button:has-text("Create Template")');
    await page.waitForTimeout(500);
    
    // Search for template
    const searchInput = page.locator('input[placeholder="Search templates..."]');
    await searchInput.fill('Search Test');
    
    await page.waitForTimeout(500);
    
    // Check search result
    const result = page.locator('text=Search Test');
    await expect(result).toBeVisible();
  });

  test('should filter templates by category', async ({ page }) => {
    // Create a template first
    await page.click('button:has-text("Create Template")');
    await page.waitForTimeout(500);
    await page.fill('input[placeholder="e.g., Document Access Template"]', 'Category Test');
    await page.fill('textarea[placeholder*="Describe"]', 'Template for category');
    await page.selectOption('select', 'rbac');
    await page.click('button:has-text("Create Template")');
    await page.waitForTimeout(500);
    
    // Filter by category
    const categorySelect = page.locator('select').last();
    await categorySelect.selectOption('rbac');
    
    await page.waitForTimeout(500);
    
    // Check filtered result
    const result = page.locator('text=Category Test');
    await expect(result).toBeVisible();
  });

  test('should validate template', async ({ page }) => {
    // Create a template
    await page.click('button:has-text("Create Template")');
    await page.waitForTimeout(500);
    await page.fill('input[placeholder="e.g., Document Access Template"]', 'Validation Test');
    await page.fill('textarea[placeholder*="Describe"]', 'Template validation test');
    await page.click('button:has-text("Create Template")');
    await page.waitForTimeout(500);
    
    // Click validate
    await page.click('button:has-text("Validate")');
    await page.waitForTimeout(1000);
    
    // Check validation result
    const validationResult = page.locator('text=valid').or(page.locator('text=errors'));
    await expect(validationResult).toBeVisible();
  });
});
