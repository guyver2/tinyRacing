import process from 'node:process'
import { defineConfig, devices } from '@playwright/test'
import { join, dirname } from 'path'
import { fileURLToPath } from 'url'

/**
 * Read environment variables from file.
 * https://github.com/motdotla/dotenv
 */
// require('dotenv').config();

// ES module equivalent of __dirname
const __filename = fileURLToPath(import.meta.url);
const __dirname = dirname(__filename);

// playwright.config.ts is in tiny-racing-vue/
// PROJECT_ROOT is tiny-racing-vue/
// ROOT_DIR is the parent (tinyRacing/)
const PROJECT_ROOT = __dirname; // tiny-racing-vue/
const ROOT_DIR = join(PROJECT_ROOT, '..'); // tinyRacing root directory
const SERVER_DIR = join(ROOT_DIR, 'server');

/**
 * See https://playwright.dev/docs/test-configuration.
 */
export default defineConfig({
  testDir: './e2e',
  /* Maximum time one test can run for. */
  timeout: 60 * 1000, // Increased for E2E tests
  expect: {
    /**
     * Maximum time expect() should wait for the condition to be met.
     * For example in `await expect(locator).toHaveText();`
     */
    timeout: 10000, // Increased for slower operations
  },
  /* Fail the build on CI if you accidentally left test.only in the source code. */
  forbidOnly: !!process.env.CI,
  /* Retry on CI only */
  retries: process.env.CI ? 2 : 0,
  /* Opt out of parallel tests on CI. */
  workers: process.env.CI ? 1 : 1, // Run sequentially to avoid DB conflicts
  /* Reporter to use. See https://playwright.dev/docs/test-reporters */
  reporter: [
    ['html', { outputFolder: 'playwright-report', open: 'never' }], // Don't auto-open, use npm run test:e2e:report instead
    ['list'],
    ['json', { outputFile: 'test-results/results.json' }],
  ],
  /* Global setup and teardown */
  globalSetup: './e2e/global-setup.ts',
  globalTeardown: './e2e/global-teardown.ts',
  /* Shared settings for all the projects below. See https://playwright.dev/docs/api/class-testoptions. */
  use: {
    /* Maximum time each action such as `click()` can take. Defaults to 0 (no limit). */
    actionTimeout: 10000,
    /* Base URL to use in actions like `await page.goto('/')`. */
    baseURL: process.env.CI ? 'http://localhost:4173' : 'http://localhost:5173',

    /* Collect trace when retrying the failed test. See https://playwright.dev/docs/trace-viewer */
    trace: 'on-first-retry',
    
    /* Take screenshot on failure */
    screenshot: 'only-on-failure',
    
    /* Record video on failure */
    video: 'retain-on-failure',

    /* Run headless by default, unless explicitly overridden with --headed flag */
    headless: true,
  },

  /* Configure projects for major browsers */
  projects: [
    {
      name: 'chromium',
      use: {
        ...devices['Desktop Chrome'],
        // Use system Chromium on Arch Linux
        executablePath: process.env.PLAYWRIGHT_USE_SYSTEM_BROWSERS ? '/usr/bin/chromium' : undefined,
      },
    },
    {
      name: 'firefox',
      use: {
        ...devices['Desktop Firefox'],
        // Use system Firefox on Arch Linux
        executablePath: process.env.PLAYWRIGHT_USE_SYSTEM_BROWSERS ? '/usr/bin/firefox' : undefined,
      },
    },
    // WebKit is not available as a system package on Arch, so we'll skip it or use bundled
    // {
    //   name: 'webkit',
    //   use: {
    //     ...devices['Desktop Safari'],
    //   },
    // },

    /* Test against mobile viewports. */
    // {
    //   name: 'Mobile Chrome',
    //   use: {
    //     ...devices['Pixel 5'],
    //   },
    // },
    // {
    //   name: 'Mobile Safari',
    //   use: {
    //     ...devices['iPhone 12'],
    //   },
    // },

    /* Test against branded browsers. */
    // {
    //   name: 'Microsoft Edge',
    //   use: {
    //     channel: 'msedge',
    //   },
    // },
    // {
    //   name: 'Google Chrome',
    //   use: {
    //     channel: 'chrome',
    //   },
    // },
  ],

  /* Folder for test artifacts such as screenshots, videos, traces, etc. */
  outputDir: 'test-results/',

  /* Run your local dev server before starting the tests */
  webServer: [
    /**
     * Backend API server with test database
     * Starts the Rust backend server configured to use the test database
     */
    {
      command: `cd ${SERVER_DIR} && DATABASE_URL=postgresql://tiny_racing_test:test_password@localhost:5433/tiny_racing_test DISABLE_UI=true cargo run`,
      url: 'http://localhost:3000/tracks?limit=1',
      reuseExistingServer: false, // Always start fresh and kill after tests
      timeout: 300 * 1000, // 5 minutes for cargo build + server start (first build can be slow)
      stdout: 'pipe',
      stderr: 'pipe',
      // Wait for the server to be ready by checking if the API responds
      // Using /tracks endpoint as it's public and doesn't require auth
      // The server logs "API server listening on http://localhost:3000" when ready
      // Note: First cargo build can take 2-3 minutes, so we allow 5 minutes total
    },
    /**
     * Frontend dev server
     * Use the dev server by default for faster feedback loop.
     * Use the preview server on CI for more realistic testing.
     * Playwright will re-use the local server if there is already a dev-server running.
     */
    {
      command: process.env.CI ? 'npm run preview' : 'npm run dev',
      port: process.env.CI ? 4173 : 5173,
      reuseExistingServer: !process.env.CI,
      timeout: 120 * 1000,
      env: {
        // Frontend will connect to backend on default ports 3000/3030
        // Note: Server routes are at root level (no /api prefix)
        VITE_API_URL: process.env.VITE_API_URL || 'http://localhost:3000',
        VITE_WS_URL: process.env.VITE_WS_URL || 'ws://localhost:3030',
      },
    },
  ],
})
