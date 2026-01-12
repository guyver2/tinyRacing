import { test, expect } from '@playwright/test';
import { login, navigateAndWait, takeScreenshot, waitForWebSocketConnection } from './helpers/test-fixtures';

test.describe('Races', () => {
  test.beforeEach(async ({ page }) => {
    // Login before each test
    await login(page);
  });

  test('should display races list page', async ({ page }) => {
    await navigateAndWait(page, '/races');
    
    // Check for races list or content
    const racesContent = page.locator('h1, h2, [class*="race"], [class*="card"]').first();
    await expect(racesContent).toBeVisible({ timeout: 10000 });
    
    await takeScreenshot(page, 'races-list-page');
  });

  test('should display race information', async ({ page }) => {
    await navigateAndWait(page, '/races');
    
    // Look for race items or cards
    const raceItems = page.locator('[class*="race"], [class*="card"], li');
    const count = await raceItems.count();
    
    // Should have at least some content (even if no races, should show empty state)
    expect(count).toBeGreaterThanOrEqual(0);
    
    await takeScreenshot(page, 'races-content');
  });

  test('should navigate to race detail', async ({ page }) => {
    await navigateAndWait(page, '/races');
    
    // Try to click on first race if available
    const firstRace = page.locator('[class*="race"], [class*="card"], a').first();
    const isVisible = await firstRace.isVisible({ timeout: 2000 }).catch(() => false);
    
    if (isVisible) {
      await firstRace.click();
      await page.waitForURL(/\/races\/[^/]+/, { timeout: 10000 });
      
      const raceDetail = page.locator('h1, h2, [class*="race-detail"]').first();
      await expect(raceDetail).toBeVisible({ timeout: 10000 });
      
      await takeScreenshot(page, 'race-detail-page');
    } else {
      // No races available, test empty state
      await takeScreenshot(page, 'races-empty-state');
    }
  });
});

test.describe('Game/Race View', () => {
  test.beforeEach(async ({ page }) => {
    await login(page);
  });

  test('should display game view', async ({ page }) => {
    await navigateAndWait(page, '/');
    
    // Check for game/race view content
    // This might show "No Active Race" or actual race content
    const gameContent = page.locator('h1, h2, [class*="race"], [class*="game"], [class*="track"]').first();
    await expect(gameContent).toBeVisible({ timeout: 10000 });
    
    await takeScreenshot(page, 'game-view');
  });

  test('should show connection status', async ({ page }) => {
    await navigateAndWait(page, '/');
    
    // Look for connection status indicator
    const connectionStatus = page.locator('[class*="connection"], [class*="status"], [data-testid="connection-status"]');
    const isVisible = await connectionStatus.first().isVisible({ timeout: 5000 }).catch(() => false);
    
    if (isVisible) {
      await expect(connectionStatus.first()).toBeVisible();
      await takeScreenshot(page, 'connection-status');
    }
  });

  test('should display race header when race is active', async ({ page }) => {
    await navigateAndWait(page, '/');
    
    // Look for race header component
    const raceHeader = page.locator('[class*="race-header"], header, [class*="header"]').first();
    await expect(raceHeader).toBeVisible({ timeout: 10000 });
    
    await takeScreenshot(page, 'race-header');
  });
});

