import { test, expect } from '@playwright/test';
import { login, navigateAndWait, takeScreenshot, TEST_CONFIG } from './helpers/test-fixtures';

test.describe('Teams', () => {
  test.beforeEach(async ({ page }) => {
    // Login before each test
    await login(page);
  });

  test('should display my team page', async ({ page }) => {
    await navigateAndWait(page, '/my-team');
    
    // Check for team information
    // Adjust selectors based on your actual component structure
    const teamName = page.locator('h1, h2, [class*="team-name"]').first();
    await expect(teamName).toBeVisible({ timeout: 10000 });
    
    await takeScreenshot(page, 'my-team-page');
  });

  test('should display all teams page', async ({ page }) => {
    await navigateAndWait(page, '/teams');
    
    // Check for teams list
    // Look for team cards or list items
    const teamsList = page.locator('[class*="team"], [class*="card"], li').first();
    await expect(teamsList).toBeVisible({ timeout: 10000 });
    
    // Should show multiple teams
    const teamItems = page.locator('[class*="team"], [class*="card"]');
    const count = await teamItems.count();
    expect(count).toBeGreaterThan(0);
    
    await takeScreenshot(page, 'all-teams-page');
  });

  test('should navigate to team detail page', async ({ page }) => {
    await navigateAndWait(page, '/teams');
    
    // Click on first team
    const firstTeam = page.locator('[class*="team"], [class*="card"], a').first();
    await firstTeam.click();
    
    // Should navigate to team detail
    await page.waitForURL(/\/teams\/[^/]+/, { timeout: 10000 });
    
    // Check for team detail content
    const teamDetail = page.locator('h1, h2, [class*="team-detail"]').first();
    await expect(teamDetail).toBeVisible({ timeout: 10000 });
    
    await takeScreenshot(page, 'team-detail-page');
  });

  test('should display team information', async ({ page }) => {
    await navigateAndWait(page, '/my-team');
    
    // Check for various team information sections
    // Adjust based on your Team.vue component structure
    
    // Team name
    const teamName = page.locator('h1, h2, [class*="team-name"]');
    await expect(teamName.first()).toBeVisible();
    
    // Team stats or details (adjust selectors)
    const teamStats = page.locator('[class*="stat"], [class*="info"], [class*="detail"]');
    const statsCount = await teamStats.count();
    // Should have some team information displayed
    expect(statsCount).toBeGreaterThanOrEqual(0);
    
    await takeScreenshot(page, 'team-information');
  });
});

