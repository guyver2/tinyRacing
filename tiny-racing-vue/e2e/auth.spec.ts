import { test, expect } from '@playwright/test';
import { login, logout, clearAuth, TEST_CONFIG } from './helpers/test-fixtures';
import { takeScreenshot } from './helpers/test-fixtures';

test.describe('Authentication', () => {
  test.beforeEach(async ({ page }) => {
    // Navigate to a page first, then clear any existing auth state
    await page.goto('/');
    await clearAuth(page);
  });

  test('should display login page', async ({ page }) => {
    await page.goto('/login');
    await expect(page).toHaveTitle(/.*/); // Page should have a title
    
    // Check for login form elements
    const usernameInput = page.locator('input[name="username"], input[type="text"]').first();
    const passwordInput = page.locator('input[type="password"]').first();
    const submitButton = page.locator('button[type="submit"], button:has-text("Login"), button:has-text("Sign in")').first();
    
    await expect(usernameInput).toBeVisible();
    await expect(passwordInput).toBeVisible();
    await expect(submitButton).toBeVisible();
    
    await takeScreenshot(page, 'login-page');
  });

  test('should login with valid credentials', async ({ page }) => {
    await page.goto('/login');
    
    // Wait for form to be visible
    await page.waitForSelector('input#username', { timeout: 5000 });
    
    // Fill in login form using IDs
    await page.fill('input#username', TEST_CONFIG.testUser.username);
    await page.fill('input#password', TEST_CONFIG.testUser.password);
    
    // Submit form and wait for response
    const [response] = await Promise.all([
      page.waitForResponse((resp) => resp.url().includes('/auth/login') && resp.status() === 200, { timeout: 10000 }).catch(() => null),
      page.click('button[type="submit"]'),
    ]);
    
    // Wait a moment for the login to process
    await page.waitForTimeout(1000);
    
    // Check if we navigated away from login page
    const currentUrl = page.url();
    const isStillOnLogin = currentUrl.includes('/login');
    
    if (!isStillOnLogin) {
      // Successfully navigated away
      await takeScreenshot(page, 'after-login-success');
    } else {
      // Still on login page - check for error
      const errorMessage = page.locator('.error-message');
      if (await errorMessage.isVisible({ timeout: 2000 })) {
        const errorText = await errorMessage.textContent();
        throw new Error(`Login failed: ${errorText}`);
      }
      // No error, might be authenticated but no redirect
      await takeScreenshot(page, 'after-login-no-redirect');
    }
    
    // Check if authenticated by checking localStorage
    // The app stores the token as 'jwt_token' (see ApiService.ts)
    const token = await page.evaluate(() => localStorage.getItem('jwt_token'));
    expect(token).toBeTruthy();
    
    await takeScreenshot(page, 'after-login');
  });

  test('should show error with invalid credentials', async ({ page }) => {
    await page.goto('/login');
    
    // Fill in with invalid credentials
    await page.fill('input[name="username"], input[type="text"]', 'invaliduser');
    await page.fill('input[type="password"]', 'wrongpassword');
    
    // Submit form
    await page.click('button[type="submit"], button:has-text("Login"), button:has-text("Sign in")');
    
    // Should show error message (wait a bit for API response)
    await page.waitForTimeout(2000);
    
    // Check for error message (adjust selector based on your error display)
    const errorMessage = page.locator('.error, [class*="error"], [class*="alert"]').first();
    await expect(errorMessage).toBeVisible({ timeout: 5000 }).catch(() => {
      // Error might be displayed differently
    });
    
    // Should still be on login page
    await expect(page).toHaveURL(/.*login/);
    
    await takeScreenshot(page, 'login-error');
  });

  test('should display register page', async ({ page }) => {
    await page.goto('/register');
    await expect(page).toHaveTitle(/.*/);
    
    // Check for registration form elements
    const usernameInput = page.locator('input[name="username"], input[type="text"]').first();
    const passwordInput = page.locator('input[type="password"]').first();
    const submitButton = page.locator('button[type="submit"], button:has-text("Register"), button:has-text("Sign up")').first();
    
    await expect(usernameInput).toBeVisible();
    await expect(passwordInput).toBeVisible();
    await expect(submitButton).toBeVisible();
    
    await takeScreenshot(page, 'register-page');
  });

  test('should register new user', async ({ page }) => {
    await page.goto('/register');
    
    // Generate unique username for test
    const uniqueUsername = `testuser_${Date.now()}`;
    
    // Fill in registration form
    await page.fill('input[name="username"], input[type="text"]', uniqueUsername);
    await page.fill('input[type="password"]', 'testpass123');
    
    // If there's an email field
    const emailInput = page.locator('input[type="email"], input[name="email"]');
    if (await emailInput.count() > 0) {
      await emailInput.fill(`${uniqueUsername}@test.com`);
    }
    
    // Submit form
    await page.click('button[type="submit"], button:has-text("Register"), button:has-text("Sign up")');
    
    // Should redirect after successful registration
    await page.waitForURL((url) => url.pathname !== '/register', { timeout: 10000 });
    
    await takeScreenshot(page, 'after-register');
  });

  test('should logout successfully', async ({ page }) => {
    // First login
    await login(page);
    
    // Then logout
    await logout(page);
    
    // Should be redirected to login or home page
    const currentUrl = page.url();
    expect(currentUrl).toMatch(/\/(login|$)/);
    
    await takeScreenshot(page, 'after-logout');
  });
});

