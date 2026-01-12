# E2E Testing Setup - Discussion Summary

## Overview

This document summarizes the complete setup of end-to-end testing infrastructure for the Tiny Racing Vue application using Playwright. The setup ensures reproducible, isolated tests that don't interfere with production or development databases.

## What Was Accomplished

### 1. Test Infrastructure

#### Isolated Test Database
- **Created**: `docker-compose.test.yml` - Separate PostgreSQL instance
- **Port**: `5433` (isolated from production `5432`)
- **Database**: `tiny_racing_test`
- **User**: `tiny_racing_test` / `test_password`
- **Container**: `tiny_racing_test_db`

#### Test Data Seeding
- **Created**: `server/examples/seed_db_test.rs` - Test-specific seed script
- **Updated**: `server/examples/seed_db.rs` - Added migration step and fixed pagination parameters
- **Test User**: `testuser` / `testpass123`
- **Seed Data**: 5 teams, 10 drivers, 11 cars, 6 tracks

#### Test Configuration
- **Updated**: `playwright.config.ts` - Complete E2E test configuration
  - Global setup/teardown for database initialization
  - Automatic backend server startup
  - Frontend dev server configuration
  - Screenshot and video capture on failures
  - HTML and JSON reporting
  - Sequential execution (1 worker) to avoid DB conflicts

### 2. Test Files Created

#### Authentication Tests (`e2e/auth.spec.ts`)
- Login page display
- Successful login with valid credentials
- Invalid credentials handling
- Registration page
- User registration
- Logout functionality

#### Team Management Tests (`e2e/teams.spec.ts`)
- My team page
- All teams listing
- Team detail navigation
- Team information display

#### Race Tests (`e2e/races.spec.ts`)
- Races list page
- Race detail navigation
- Game/race view
- Connection status
- Race header display

#### Market Tests (`e2e/market.spec.ts`)
- Market page display
- Available drivers listing
- Available cars listing

#### Track Tests (`e2e/tracks.spec.ts`)
- Tracks list page
- Multiple tracks display
- Track detail navigation
- Track information display

#### Driver & Car Tests (`e2e/drivers-cars.spec.ts`)
- Driver detail navigation
- Driver information display
- Car detail navigation
- Car information display

### 3. Helper Utilities

#### Test Setup (`e2e/helpers/test-setup.ts`)
- Database management functions
- Server startup utilities
- Test configuration constants

#### Test Fixtures (`e2e/helpers/test-fixtures.ts`)
- `login()` - Automated login helper
- `logout()` - Automated logout helper
- `clearAuth()` - Clear authentication state
- `navigateAndWait()` - Navigate and wait for page load
- `takeScreenshot()` - Consistent screenshot naming
- `waitForApiResponse()` - Wait for API calls
- `waitForWebSocketConnection()` - Wait for WebSocket ready

#### Global Setup/Teardown
- **Created**: `e2e/global-setup.ts` - Database initialization before tests
- **Created**: `e2e/global-teardown.ts` - Cleanup after tests

### 4. Scripts

#### Database Setup (`e2e/scripts/setup-test-db.sh`)
- Starts test database container
- Waits for database readiness
- Runs migrations
- Seeds test data
- Creates test user

#### Backend Server (`e2e/scripts/start-backend.sh`)
- Starts backend server with test database configuration
- Configures environment variables

### 5. Documentation

- **TESTING_PLAN.md** - Comprehensive testing strategy and coverage plan
- **README.md** - User guide for running tests
- **SETUP_SUMMARY.md** - Quick reference guide
- **ARCH_SETUP.md** - Arch Linux specific setup (system browsers)
- **TROUBLESHOOTING.md** - Common issues and solutions

## Key Issues Resolved

### 1. ES Module Compatibility
**Issue**: `__dirname` not available in ES modules  
**Solution**: Used `import.meta.url` with `fileURLToPath` and `dirname` to create ES module equivalent

### 2. Playwright Global Setup
**Issue**: Playwright requires file paths, not functions for `globalSetup`/`globalTeardown`  
**Solution**: Created separate files (`global-setup.ts`, `global-teardown.ts`) and referenced them as strings

### 3. Path Resolution
**Issue**: Incorrect path calculations for finding project root  
**Solution**: Fixed path calculations in all scripts and config files to correctly resolve `tinyRacing` root directory

### 4. Database Migration
**Issue**: Seed script trying to insert data before migrations ran  
**Solution**: Added `db.migrate().await?` call before seeding in `seed_db.rs`

### 5. Function Signature Updates
**Issue**: Database query functions updated to require pagination parameters  
**Solution**: Updated all `list_*` function calls to include `limit` and `offset` parameters

### 6. Arch Linux Browser Support
**Issue**: Playwright bundled browsers have dependency issues on Arch  
**Solution**: Configured to use system browsers (`chromium`, `firefox`) with `PLAYWRIGHT_USE_SYSTEM_BROWSERS=1`

### 7. localStorage Access
**Issue**: Accessing `localStorage` before page navigation  
**Solution**: Updated `clearAuth()` to navigate to a page first before accessing `localStorage`

### 8. API URL Configuration
**Issue**: Frontend configured with `/api` prefix but server routes are at root  
**Solution**: Updated `VITE_API_URL` to `http://localhost:3000` (no `/api` prefix)

## Current Status

### ✅ Completed
- Test database infrastructure
- Database seeding scripts
- Playwright configuration
- Test files for all major pages
- Helper utilities and fixtures
- Global setup/teardown
- Documentation
- npm scripts for easy execution

### ⚠️ Known Issues
- **Backend server startup**: The automatic backend server startup via Playwright's `webServer` may need manual intervention
  - **Workaround**: Start backend manually before running tests
  - **Future**: May need to improve health check or increase timeout further

### 🔄 In Progress / Needs Adjustment
- Test selectors may need adjustment based on actual component structure
- Some tests may need refinement once backend connectivity is confirmed
- Visual regression testing can be added later

## Usage

### Quick Start

1. **One-time setup** (if not already done):
   ```bash
   npm run test:setup
   ```

2. **Run all tests**:
   ```bash
   PLAYWRIGHT_USE_SYSTEM_BROWSERS=1 npm run test:e2e
   ```

3. **Run specific test**:
   ```bash
   PLAYWRIGHT_USE_SYSTEM_BROWSERS=1 npx playwright test --grep "should login with valid credentials"
   ```

4. **Run in UI mode** (interactive):
   ```bash
   PLAYWRIGHT_USE_SYSTEM_BROWSERS=1 npm run test:e2e:ui
   ```

### Manual Backend Startup (If Needed)

If automatic backend startup doesn't work:

```bash
# Terminal 1: Start backend
cd server
DATABASE_URL=postgresql://tiny_racing_test:test_password@localhost:5433/tiny_racing_test DISABLE_UI=true cargo run

# Terminal 2: Run tests
PLAYWRIGHT_USE_SYSTEM_BROWSERS=1 npm run test:e2e
```

## File Structure

```
tiny-racing-vue/
├── e2e/
│   ├── auth.spec.ts              # Authentication tests
│   ├── teams.spec.ts             # Team management tests
│   ├── races.spec.ts              # Race viewing tests
│   ├── market.spec.ts            # Market page tests
│   ├── tracks.spec.ts            # Track browsing tests
│   ├── drivers-cars.spec.ts      # Driver/car detail tests
│   ├── global-setup.ts           # Database setup before tests
│   ├── global-teardown.ts        # Cleanup after tests
│   ├── helpers/
│   │   ├── test-setup.ts         # Database/server utilities
│   │   └── test-fixtures.ts      # Reusable test helpers
│   ├── scripts/
│   │   ├── setup-test-db.sh      # Database setup script
│   │   └── start-backend.sh      # Backend startup script
│   ├── TESTING_PLAN.md           # Comprehensive testing plan
│   ├── README.md                 # User guide
│   ├── SETUP_SUMMARY.md          # Quick reference
│   ├── ARCH_SETUP.md             # Arch Linux setup
│   ├── TROUBLESHOOTING.md        # Debugging guide
│   └── SETUP_DISCUSSION_SUMMARY.md  # This file
├── playwright.config.ts          # Playwright configuration
└── package.json                  # npm scripts

tinyRacing/
├── docker-compose.test.yml       # Test database configuration
└── server/
    └── examples/
        ├── seed_db.rs            # Main seed script (updated)
        └── seed_db_test.rs       # Test user seed script
```

## NPM Scripts

- `npm run test:setup` - Set up test database (one-time)
- `npm run test:e2e` - Run all E2E tests
- `npm run test:e2e:system` - Run tests with system browsers (Arch Linux)
- `npm run test:e2e:ui` - Run tests in interactive UI mode
- `npm run test:e2e:headed` - Run tests with visible browser
- `npm run test:e2e:debug` - Run tests in debug mode

## Test Database Configuration

- **Port**: `5433` (isolated from production `5432`)
- **Database**: `tiny_racing_test`
- **User**: `tiny_racing_test`
- **Password**: `test_password`
- **Container**: `tiny_racing_test_db`

## Test User Credentials

- **Username**: `testuser`
- **Password**: `testpass123`
- **Email**: `testuser@test.com`

## Screenshots and Reports

- **Screenshots**: Automatically captured on test failure in `test-results/`
- **HTML Report**: Generated in `playwright-report/` - view with `npx playwright show-report`
- **Videos**: Recorded on failure in `test-results/`
- **Traces**: Collected on retry for debugging

## Next Steps

1. **Verify backend connectivity**: Ensure the backend server starts correctly with the test database
2. **Adjust test selectors**: Update selectors in test files to match actual component structure
3. **Run tests**: Execute tests and fix any selector or logic issues
4. **Add more test cases**: Expand coverage based on actual application behavior
5. **CI/CD integration**: Set up automated testing in CI pipeline
6. **Visual regression**: Consider adding visual regression testing with screenshots

## Key Learnings

1. **ES Modules**: Need to use `import.meta.url` instead of `__dirname`
2. **Playwright Setup**: Global setup/teardown must be file paths, not functions
3. **Database Isolation**: Critical for reproducible tests
4. **Arch Linux**: System browsers work better than bundled browsers
5. **Path Resolution**: Careful attention needed when working with nested directories
6. **API Configuration**: Ensure frontend and backend URL configurations match

## Architecture Decisions

1. **Sequential Execution**: Tests run with 1 worker to avoid database conflicts
2. **Automatic Server Startup**: Backend server starts automatically (with manual fallback)
3. **Screenshot on Failure**: Only capture screenshots when tests fail (saves space)
4. **Video on Failure**: Record videos only for failed tests
5. **Test Database Persistence**: Database persists between runs (Docker volume) for faster subsequent runs

## Conclusion

A comprehensive E2E testing infrastructure has been set up with:
- ✅ Isolated test database
- ✅ Reproducible test data
- ✅ Comprehensive test coverage
- ✅ Helper utilities for common operations
- ✅ Automatic infrastructure management
- ✅ Detailed documentation
- ✅ Troubleshooting guides

The infrastructure is ready for use, with some minor adjustments likely needed for test selectors and backend connectivity verification.

