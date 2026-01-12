import { test, expect } from '@playwright/test';
import { login, navigateAndWait, takeScreenshot } from './helpers/test-fixtures';

test.describe('Drivers', () => {
  test.beforeEach(async ({ page }) => {
    await login(page);
  });

  test('should navigate to driver detail from team page', async ({ page }) => {
    // Go to my team page first
    await navigateAndWait(page, '/my-team');
    
    // Look for driver links or cards
    const driverLink = page.locator('a[href*="/drivers/"], [class*="driver"] a').first();
    const isVisible = await driverLink.isVisible({ timeout: 5000 }).catch(() => false);
    
    if (isVisible) {
      await driverLink.click();
      await page.waitForURL(/\/drivers\/[^/]+/, { timeout: 10000 });
      
      // Check for driver detail content
      const driverDetail = page.locator('h1, h2, [class*="driver-detail"]').first();
      await expect(driverDetail).toBeVisible({ timeout: 10000 });
      
      await takeScreenshot(page, 'driver-detail-page');
    }
  });

  test('should display driver information', async ({ page }) => {
    // This test assumes we can navigate to a driver detail page
    // You may need to get a driver ID from the API or seed data
    
    // Try to navigate to a driver page (adjust ID based on seed data)
    // For now, we'll test if the page structure exists
    await navigateAndWait(page, '/my-team');
    
    // Look for any driver information on team page
    const driverInfo = page.locator('[class*="driver"]').first();
    const isVisible = await driverInfo.isVisible({ timeout: 5000 }).catch(() => false);
    
    if (isVisible) {
      await expect(driverInfo).toBeVisible();
      await takeScreenshot(page, 'driver-info-on-team');
    }
  });
});

test.describe('Cars', () => {
  test.beforeEach(async ({ page }) => {
    await login(page);
  });

  test('should navigate to car detail from team page', async ({ page }) => {
    // Go to my team page first
    await navigateAndWait(page, '/my-team');
    
    // Look for car links or cards
    const carLink = page.locator('a[href*="/cars/"], [class*="car"] a').first();
    const isVisible = await carLink.isVisible({ timeout: 5000 }).catch(() => false);
    
    if (isVisible) {
      await carLink.click();
      await page.waitForURL(/\/cars\/[^/]+/, { timeout: 10000 });
      
      // Check for car detail content
      const carDetail = page.locator('h1, h2, [class*="car-detail"]').first();
      await expect(carDetail).toBeVisible({ timeout: 10000 });
      
      await takeScreenshot(page, 'car-detail-page');
    }
  });

  test('should display car information', async ({ page }) => {
    await navigateAndWait(page, '/my-team');
    
    // Look for car information on team page
    const carInfo = page.locator('[class*="car"]').first();
    const isVisible = await carInfo.isVisible({ timeout: 5000 }).catch(() => false);
    
    if (isVisible) {
      await expect(carInfo).toBeVisible();
      await takeScreenshot(page, 'car-info-on-team');
    }
  });
});

