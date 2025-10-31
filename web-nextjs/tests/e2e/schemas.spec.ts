import { test, expect } from '@playwright/test';

test.describe('Schema Editor', () => {
  test.beforeEach(async ({ page }) => {
    await page.goto('/');
    await page.click('button:has-text("Schemas")');
  });

  test('should display schema editor page', async ({ page }) => {
    await expect(page.locator('h2')).toContainText('Schemas');
    await expect(page.locator('text=Define entity types and their attributes')).toBeVisible();
  });

  test('should create new schema', async ({ page }) => {
    await page.click('button:has-text("New Schema")');
    
    // Wait for the schema to be created and selected
    await page.waitForTimeout(500);
    
    // Check that a schema is now in the library
    const schemaItem = page.locator('text=New Schema');
    await expect(schemaItem).toBeVisible();
  });

  test('should validate schema', async ({ page }) => {
    // Create a new schema first
    await page.click('button:has-text("New Schema")');
    await page.waitForTimeout(500);
    
    // Click validate button
    await page.click('button:has-text("Validate")');
    
    // Wait for validation result
    await page.waitForTimeout(1000);
    
    // Check for validation message
    const validationResult = page.locator('text=Schema is valid').or(page.locator('text=Schema has errors'));
    await expect(validationResult).toBeVisible();
  });

  test('should save schema', async ({ page }) => {
    // Create a new schema
    await page.click('button:has-text("New Schema")');
    await page.waitForTimeout(500);
    
    // Validate first
    await page.click('button:has-text("Validate")');
    await page.waitForTimeout(1000);
    
    // Save the schema
    await page.click('button:has-text("Save")');
    await page.waitForTimeout(500);
    
    // Verify schema is saved (no error message)
    const errorMessage = page.locator('text=Failed to save');
    await expect(errorMessage).not.toBeVisible();
  });

  test('should duplicate schema', async ({ page }) => {
    // Create a schema first
    await page.click('button:has-text("New Schema")');
    await page.waitForTimeout(500);
    
    // Click duplicate button
    const duplicateButton = page.locator('button').filter({ has: page.locator('svg') }).nth(0);
    await duplicateButton.click();
    
    await page.waitForTimeout(500);
    
    // Check that copy exists
    const copySchema = page.locator('text=New Schema (Copy)');
    await expect(copySchema).toBeVisible();
  });

  test('should delete schema', async ({ page }) => {
    // Create a schema first
    await page.click('button:has-text("New Schema")');
    await page.waitForTimeout(500);
    
    // Get the initial count of schemas
    const initialSchemas = await page.locator('[class*="border-gray-200"]').count();
    
    // Click delete button
    const deleteButton = page.locator('button').filter({ has: page.locator('svg') }).nth(1);
    await deleteButton.click();
    
    await page.waitForTimeout(500);
    
    // Check that schema count decreased
    const finalSchemas = await page.locator('[class*="border-gray-200"]').count();
    expect(finalSchemas).toBeLessThan(initialSchemas);
  });
});
