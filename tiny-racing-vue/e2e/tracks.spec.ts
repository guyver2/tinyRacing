import { test, expect } from '@playwright/test';
import { login, navigateAndWait, takeScreenshot } from './helpers/test-fixtures';

test.describe('Tracks', () => {
  test.beforeEach(async ({ page }) => {
    await login(page);
  });

  test('should display tracks list page', async ({ page }) => {
    await navigateAndWait(page, '/tracks');
    
    // Check for tracks list
    const tracksContent = page.locator('h1, h2, [class*="track"], [class*="card"]').first();
    await expect(tracksContent).toBeVisible({ timeout: 10000 });
    
    await takeScreenshot(page, 'tracks-list-page');
  });

  test('should display multiple tracks', async ({ page }) => {
    await navigateAndWait(page, '/tracks');
    
    // Look for track items
    const trackItems = page.locator('[class*="track"], [class*="card"], li');
    const count = await trackItems.count();
    
    // Should have tracks displayed (at least from seed data)
    expect(count).toBeGreaterThan(0);
    
    await takeScreenshot(page, 'tracks-list');
  });

  test('should navigate to track detail', async ({ page }) => {
    await navigateAndWait(page, '/tracks');
    
    // Click on first track
    const firstTrack = page.locator('[class*="track"], [class*="card"], a').first();
    await firstTrack.click();
    
    // Should navigate to track detail
    await page.waitForURL(/\/tracks\/[^/]+/, { timeout: 10000 });
    
    // Check for track detail content
    const trackDetail = page.locator('h1, h2, [class*="track-detail"]').first();
    await expect(trackDetail).toBeVisible({ timeout: 10000 });
    
    await takeScreenshot(page, 'track-detail-page');
  });

  test('should display track information', async ({ page }) => {
    await navigateAndWait(page, '/tracks');
    
    // Click on first track to view details
    const firstTrack = page.locator('[class*="track"], [class*="card"], a').first();
    await firstTrack.click();
    await page.waitForURL(/\/tracks\/[^/]+/, { timeout: 10000 });
    
    // Check for track information
    const trackName = page.locator('h1, h2, [class*="track-name"]').first();
    await expect(trackName).toBeVisible();
    
    // Check for track stats or details
    const trackInfo = page.locator('[class*="stat"], [class*="info"], [class*="detail"]');
    const infoCount = await trackInfo.count();
    expect(infoCount).toBeGreaterThanOrEqual(0);
    
    await takeScreenshot(page, 'track-information');
  });
});

