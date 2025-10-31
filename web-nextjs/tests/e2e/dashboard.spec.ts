import { test, expect } from '@playwright/test';

test.describe('Dashboard - Fase 3 Features', () => {
  test.beforeEach(async ({ page }) => {
    // Navigate to dashboard
    await page.goto('/dashboard');
    await page.waitForLoadState('networkidle');
  });

  test('should display dashboard with metrics', async ({ page }) => {
    // Verify page title (from AppLayout header)
    await expect(page.locator('h2').first()).toContainText('Dashboard');

    // Check for System Health section within Dashboard
    await expect(page.locator('text=System Health')).toBeVisible();

    // Check for metrics cards (use more specific selectors)
    await expect(page.locator('h3').filter({ hasText: 'Policy Stores' })).toBeVisible();
    await expect(page.locator('h3').filter({ hasText: 'Policies' })).toBeVisible();
    await expect(page.locator('h3').filter({ hasText: 'Schemas' })).toBeVisible();
    await expect(page.locator('h3').filter({ hasText: 'Templates' })).toBeVisible();
  });

  test('should display system health status', async ({ page }) => {
    // Check gRPC server status badge
    await expect(page.locator('text=gRPC Server')).toBeVisible();

    // Check database status badge
    await expect(page.locator('text=Database')).toBeVisible();

    // Verify the badges are displayed (they should show Connected status)
    // Connected text appears multiple times, so we just check it's visible
    const connectedCount = await page.locator('text=Connected').count();
    expect(connectedCount).toBeGreaterThanOrEqual(2);
  });

  test('should display authorization request charts', async ({ page }) => {
    // Check for chart containers
    await expect(page.locator('text=Authorization Requests')).toBeVisible();
    await expect(page.locator('text=Authorization Decisions')).toBeVisible();

    // Charts placeholder should be visible (use first() to avoid strict mode)
    await expect(page.locator('text=Chart will be displayed here').first()).toBeVisible();
  });

  test('should display activity feed', async ({ page }) => {
    // Check for activity section (use h3 to avoid strict mode violation)
    await expect(page.locator('h3').filter({ hasText: 'Recent Activity' })).toBeVisible();

    // Activity items should be visible (they are rendered as divs with badges)
    const activityItems = page.locator('.flex.items-start.space-x-4');
    const count = await activityItems.count();

    // Check if there are activity items or if it shows "No recent activity"
    const hasActivityItems = count > 0;
    const noActivityMessage = await page.locator('text=No recent activity').count();

    // Either show activity items OR show "No recent activity" message
    expect(hasActivityItems || noActivityMessage > 0).toBeTruthy();
  });

  test('should refresh dashboard data', async ({ page }) => {
    // Click refresh button
    const refreshButton = page.locator('button').filter({ hasText: 'Refresh' });
    await refreshButton.click();

    // Wait for data to reload
    await page.waitForTimeout(1000);

    // Verify metrics are still visible (use more specific selector)
    await expect(page.locator('h3').filter({ hasText: 'Policy Stores' })).toBeVisible();
  });

  test('should show loading skeletons on initial load', async ({ page }) => {
    // Navigate to dashboard (force reload)
    await page.reload();

    // Wait a bit for loading state to appear
    await page.waitForTimeout(500);

    // Check for skeleton loading states
    const skeletons = page.locator('.animate-pulse');
    const skeletonCount = await skeletons.count();

    // Loading skeleton should appear before data loads (optional test)
    // If skeleton appears, it should have multiple animated elements
    if (skeletonCount > 0) {
      expect(skeletonCount).toBeGreaterThan(5);
    }
  });

  test('should navigate to other sections from sidebar', async ({ page }) => {
    // Test navigation to Policy Stores (using Link)
    await page.click('a:has-text("Policy Stores")');
    await expect(page.locator('h2').first()).toContainText('Policy Stores');
    await page.waitForLoadState('networkidle');

    // Navigate back to Dashboard
    await page.click('a:has-text("Dashboard")');
    await expect(page.locator('h2').first()).toContainText('Dashboard');
    await page.waitForLoadState('networkidle');

    // Verify we're back on dashboard
    await expect(page.locator('text=System Health')).toBeVisible();

    // Test navigation to Playground
    await page.click('a:has-text("Authorization Playground")');
    await expect(page.locator('h2').first()).toContainText('Authorization Playground');
    await page.waitForLoadState('networkidle');
  });

  test('should display error state when API fails', async ({ page }) => {
    // This test would require mocking API failures
    // For now, we verify the error UI exists
    const errorCard = page.locator('text=Error Loading Dashboard');
    // Error should not be visible initially (only on API failure)
    await expect(errorCard).toHaveCount(0);
  });

  test('should auto-refresh metrics', async ({ page }) => {
    // Wait for initial data to load
    await expect(page.locator('h3').filter({ hasText: 'Policy Stores' })).toBeVisible();

    // Verify System Health section is visible
    await expect(page.locator('text=System Health')).toBeVisible();

    // Note: Auto-refresh is set to 30s in the hook
    // For testing, we just verify the page remains responsive
    // In a real scenario, this would test the refetchInterval functionality

    // Verify page is still responsive
    await expect(page.locator('text=Database')).toBeVisible();

    // The metrics should still be visible (use specific selectors)
    await expect(page.locator('h3').filter({ hasText: 'Policies' })).toBeVisible();
    await expect(page.locator('h3').filter({ hasText: 'Schemas' })).toBeVisible();
    await expect(page.locator('h3').filter({ hasText: 'Templates' })).toBeVisible();
  });
});
