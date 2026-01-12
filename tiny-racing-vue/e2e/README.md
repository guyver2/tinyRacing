# End-to-End Testing with Playwright

This directory contains end-to-end tests for the Tiny Racing Vue application using Playwright.

## Overview

The E2E test suite provides comprehensive testing of the application's user-facing functionality, including:

- Authentication (login, register, logout)
- Team management
- Race viewing and management
- Market interactions
- Track browsing
- Driver and car details

## Test Infrastructure

### Test Database

Tests run against an isolated PostgreSQL database:
- **Port**: `5433` (different from production `5432`)
- **Database**: `tiny_racing_test`
- **User**: `tiny_racing_test`
- **Password**: `test_password`

The test database is managed via Docker Compose (`docker-compose.test.yml`).

### Test Server

The backend server runs with:
- Test database connection via `DATABASE_URL` environment variable
- UI disabled (`DISABLE_UI=true`)
- Standard ports (3000 for API, 3030 for WebSocket)

### Test Data

The test database is seeded with reproducible data:
- 5 teams (Red Bull, Ferrari, Mercedes, McLaren, Alpine)
- 10 drivers (2 per team + 1 unassigned)
- 11 cars (2 per team + 1 unassigned)
- 6 tracks (Monaco, Bahrain, Bugatti, Le Mans, Circle, Test)
- 1 test user: `testuser` / `testpass123`

## Setup

### Prerequisites

1. **Docker** - For running the test database
2. **Rust/Cargo** - For building and running the backend server
3. **Node.js** - For running Playwright tests

### Initial Setup

1. **Set up the test database** (one-time setup):
   ```bash
   npm run test:setup
   ```

2. **Run the tests** (backend server starts automatically):
   ```bash
   npm run test:e2e
   ```

The test infrastructure will automatically:
- Start the backend server with test database configuration
- Start the frontend dev server
- Run all E2E tests
- Generate reports and screenshots

## Running Tests

### Basic Commands

```bash
# Run all E2E tests
npm run test:e2e

# Run tests in UI mode (interactive)
npm run test:e2e:ui

# Run tests in headed mode (see browser)
npm run test:e2e:headed

# Run tests in debug mode
npm run test:e2e:debug

# Run specific test file
npx playwright test e2e/auth.spec.ts

# Run tests matching a pattern
npx playwright test --grep "login"
```

### Test Setup Scripts

```bash
# Set up test database (start, migrate, seed)
npm run test:setup

# Start backend server with test configuration
npm run test:setup:backend
```

## Test Structure

### Test Files

- `auth.spec.ts` - Authentication tests (login, register, logout)
- `teams.spec.ts` - Team management tests
- `races.spec.ts` - Race viewing and game view tests
- `market.spec.ts` - Market page tests
- `tracks.spec.ts` - Track browsing tests
- `drivers-cars.spec.ts` - Driver and car detail tests

### Helper Files

- `helpers/test-setup.ts` - Database and server setup utilities
- `helpers/test-fixtures.ts` - Reusable test helpers (login, navigation, etc.)

## Screenshots and Reports

### Screenshots

Screenshots are automatically captured:
- **On failure**: Saved to `test-results/`
- **Manual**: Use `takeScreenshot()` helper in tests
- **Visual regression**: Screenshots saved to `e2e/screenshots/`

### HTML Report

After running tests, view the HTML report:
```bash
npx playwright show-report
```

The report includes:
- Test results and timing
- Screenshots of failures
- Video recordings (on failure)
- Traces for debugging

## Configuration

Test configuration is in `playwright.config.ts`:

- **Timeout**: 60 seconds per test
- **Workers**: Sequential execution (1 worker) to avoid database conflicts
- **Screenshots**: On failure only
- **Videos**: Retained on failure
- **Traces**: Collected on retry

## Writing New Tests

### Example Test

```typescript
import { test, expect } from '@playwright/test';
import { login, navigateAndWait, takeScreenshot } from './helpers/test-fixtures';

test.describe('My Feature', () => {
  test.beforeEach(async ({ page }) => {
    await login(page);
  });

  test('should do something', async ({ page }) => {
    await navigateAndWait(page, '/my-page');
    
    // Your test assertions
    await expect(page.locator('h1')).toHaveText('Expected Text');
    
    await takeScreenshot(page, 'my-feature');
  });
});
```

### Best Practices

1. **Use helpers**: Leverage `login()`, `navigateAndWait()`, etc.
2. **Take screenshots**: Use `takeScreenshot()` for visual regression
3. **Wait for elements**: Use `waitFor()` or `toBeVisible()` with timeouts
4. **Clean up**: Tests should be independent and clean up after themselves
5. **Use data-testid**: Add `data-testid` attributes to key elements for reliable selectors

## Troubleshooting

### Database Issues

If the test database fails to start:
```bash
# Check if container is running
docker ps | grep tiny_racing_test_db

# Check logs
docker logs tiny_racing_test_db

# Restart database
docker-compose -f docker-compose.test.yml restart
```

### Server Issues

If the backend server fails to start:
- Check that port 3000 and 3030 are available
- Verify `DATABASE_URL` environment variable
- Check server logs for errors

### Test Failures

1. **Check HTML report**: `npx playwright show-report`
2. **Review screenshots**: Check `test-results/` directory
3. **Check traces**: Use `npx playwright show-trace <trace-file>`
4. **Run in headed mode**: `npm run test:e2e:headed` to see what's happening

## CI/CD Integration

For CI/CD pipelines:

1. Start test database
2. Run migrations and seed
3. Start backend server
4. Run Playwright tests
5. Upload test results and screenshots

Example GitHub Actions workflow:
```yaml
- name: Start test database
  run: docker-compose -f docker-compose.test.yml up -d

- name: Setup test database
  run: npm run test:setup

- name: Start backend
  run: |
    cd server
    DATABASE_URL=postgresql://tiny_racing_test:test_password@localhost:5433/tiny_racing_test DISABLE_UI=true cargo run &
  env:
    DATABASE_URL: postgresql://tiny_racing_test:test_password@localhost:5433/tiny_racing_test

- name: Run E2E tests
  run: npm run test:e2e
```

## Maintenance

### Updating Test Data

To update the seed data, modify `server/examples/seed_db.rs` or create a test-specific seed script.

### Adding New Test Pages

1. Create a new test file in `e2e/`
2. Add test cases following the existing pattern
3. Update `TESTING_PLAN.md` with new coverage
4. Add screenshots for visual regression

### Debugging

- Use `test.debug()` to pause execution
- Use `page.pause()` to interact with the page
- Check browser console: `page.on('console', msg => console.log(msg.text()))`

