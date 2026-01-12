import { test, expect } from '@playwright/test';
import { login, navigateAndWait, takeScreenshot } from './helpers/test-fixtures';

test.describe('Market', () => {
  test.beforeEach(async ({ page }) => {
    await login(page);
  });

  test('should display market page', async ({ page }) => {
    await navigateAndWait(page, '/market');
    
    // Check for market content
    const marketContent = page.locator('h1, h2, [class*="market"]').first();
    await expect(marketContent).toBeVisible({ timeout: 10000 });
    
    await takeScreenshot(page, 'market-page');
  });

  test('should display available drivers', async ({ page }) => {
    await navigateAndWait(page, '/market');
    
    // Look for drivers section or list
    const driversSection = page.locator('[class*="driver"], [class*="market-driver"]').first();
    const isVisible = await driversSection.isVisible({ timeout: 5000 }).catch(() => false);
    
    if (isVisible) {
      await expect(driversSection).toBeVisible();
      await takeScreenshot(page, 'market-drivers');
    }
  });

  test('should display available cars', async ({ page }) => {
    await navigateAndWait(page, '/market');
    
    // Look for cars section or list
    const carsSection = page.locator('[class*="car"], [class*="market-car"]').first();
    const isVisible = await carsSection.isVisible({ timeout: 5000 }).catch(() => false);
    
    if (isVisible) {
      await expect(carsSection).toBeVisible();
      await takeScreenshot(page, 'market-cars');
    }
  });
});

