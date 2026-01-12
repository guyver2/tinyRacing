import { Page, expect } from '@playwright/test';
import { TEST_CONFIG } from './test-setup';

// Re-export TEST_CONFIG for convenience
export { TEST_CONFIG };

/**
 * Login helper function
 */
export async function login(page: Page, username = TEST_CONFIG.testUser.username, password = TEST_CONFIG.testUser.password) {
  await page.goto('/login');
  
  // Wait for form to be visible
  await page.waitForSelector('input#username', { timeout: 5000 });
  
  // Fill in login form using IDs
  await page.fill('input#username', username);
  await page.fill('input#password', password);
  
  // Wait for the login API response to complete
  const [response] = await Promise.all([
    page.waitForResponse((resp) => resp.url().includes('/auth/login') && resp.status() === 200, { timeout: 15000 }),
    page.click('button[type="submit"]'),
  ]);
  
  // Verify the response contains a token
  const responseData = await response.json();
  if (responseData.status !== 'success' || !responseData.data?.token) {
    // Check for error message on the page
    const errorMessage = page.locator('.error-message');
    if (await errorMessage.isVisible({ timeout: 2000 })) {
      const errorText = await errorMessage.textContent();
      throw new Error(`Login failed: ${errorText}`);
    }
    throw new Error(`Login failed: ${responseData.message || 'Invalid response'}`);
  }
  
  // Wait for navigation (if it happens) and for token to be stored in localStorage
  // The ApiService.login() function stores the token synchronously, but we'll wait a bit
  // to ensure Vue has processed the response
  await page.waitForTimeout(500);
  
  // Wait for navigation away from login page (if it happens)
  try {
    await page.waitForURL((url) => url.pathname !== '/login', { timeout: 5000 });
  } catch {
    // Navigation might not happen, that's OK
  }
  
  // Verify we're authenticated by checking localStorage
  // The app stores the token as 'jwt_token' (see ApiService.ts)
  const token = await page.evaluate(() => localStorage.getItem('jwt_token'));
  if (!token) {
    // Debug: log what's actually in localStorage
    const allStorage = await page.evaluate(() => {
      const items: Record<string, string> = {};
      for (let i = 0; i < localStorage.length; i++) {
        const key = localStorage.key(i);
        if (key) items[key] = localStorage.getItem(key) || '';
      }
      return items;
    });
    throw new Error(`Login succeeded but no token found in localStorage. Available keys: ${Object.keys(allStorage).join(', ')}`);
  }
}

/**
 * Logout helper function
 */
export async function logout(page: Page) {
  // Look for logout button/link in header or menu
  const logoutButton = page.locator('button:has-text("Logout"), a:has-text("Logout"), button:has-text("Sign out")');
  if (await logoutButton.isVisible({ timeout: 1000 })) {
    await logoutButton.click();
    await page.waitForURL((url) => url.pathname === '/login' || url.pathname === '/', { timeout: 5000 });
  }
}

/**
 * Wait for API response helper
 */
export async function waitForApiResponse(page: Page, urlPattern: string | RegExp, timeout = 10000) {
  return page.waitForResponse(
    (response) => {
      const url = response.url();
      if (typeof urlPattern === 'string') {
        return url.includes(urlPattern);
      }
      return urlPattern.test(url);
    },
    { timeout }
  );
}

/**
 * Wait for WebSocket connection
 */
export async function waitForWebSocketConnection(page: Page, timeout = 10000) {
  // Check for connection status indicator
  const connectionStatus = page.locator('[data-testid="connection-status"], .connection-status, [class*="connected"]');
  await connectionStatus.waitFor({ state: 'visible', timeout });
  
  // Alternative: wait for WebSocket message
  await page.evaluate(() => {
    return new Promise<void>((resolve) => {
      const checkConnection = () => {
        // Check if WebSocket is connected (this depends on your implementation)
        const ws = (window as any).__ws;
        if (ws && ws.readyState === WebSocket.OPEN) {
          resolve();
        } else {
          setTimeout(checkConnection, 100);
        }
      };
      checkConnection();
    });
  }).catch(() => {
    // If WebSocket check fails, assume connection is OK if status indicator is visible
  });
}

/**
 * Take screenshot helper with consistent naming
 */
export async function takeScreenshot(page: Page, name: string) {
  await page.screenshot({
    path: `e2e/screenshots/${name}-${Date.now()}.png`,
    fullPage: true,
  });
}

/**
 * Navigate and wait for page load
 */
export async function navigateAndWait(page: Page, path: string) {
  await page.goto(path);
  // Wait for main content to load
  await page.waitForLoadState('networkidle');
  // Wait a bit more for Vue to render
  await page.waitForTimeout(500);
}

/**
 * Get authentication token from localStorage
 */
export async function getAuthToken(page: Page): Promise<string | null> {
  return page.evaluate(() => {
    // The app stores the token as 'jwt_token' (see ApiService.ts)
    return localStorage.getItem('jwt_token') || localStorage.getItem('token') || localStorage.getItem('authToken') || null;
  });
}

/**
 * Set authentication token in localStorage
 */
export async function setAuthToken(page: Page, token: string) {
  await page.evaluate((t) => {
    // The app stores the token as 'jwt_token' (see ApiService.ts)
    localStorage.setItem('jwt_token', t);
    // Also set a future expiry date (24 hours from now)
    const expiresAt = new Date(Date.now() + 24 * 60 * 60 * 1000).toISOString();
    localStorage.setItem('jwt_token_expires_at', expiresAt);
  }, token);
}

/**
 * Clear all authentication data
 */
export async function clearAuth(page: Page) {
  // Navigate to a page first to ensure we have a valid origin
  await page.goto('/');
  await page.evaluate(() => {
    // The app stores the token as 'jwt_token' (see ApiService.ts)
    localStorage.removeItem('jwt_token');
    localStorage.removeItem('jwt_token_expires_at');
    localStorage.removeItem('token');
    localStorage.removeItem('authToken');
    localStorage.removeItem('user');
  });
}

