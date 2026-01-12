# E2E Testing Setup Summary

This document provides a quick reference for the E2E testing infrastructure that has been set up.

## What Has Been Created

### 1. Test Infrastructure Files

- **`docker-compose.test.yml`** - Isolated PostgreSQL test database (port 5433)
- **`e2e/scripts/setup-test-db.sh`** - Script to start and seed test database
- **`e2e/scripts/start-backend.sh`** - Script to start backend with test config
- **`server/examples/seed_db_test.rs`** - Test-specific seed script for test user

### 2. Test Configuration

- **`playwright.config.ts`** - Updated with:
  - Global setup/teardown for test database
  - Screenshot and video capture on failure
  - HTML and JSON reporting
  - Sequential test execution (1 worker)

### 3. Test Files

- **`e2e/auth.spec.ts`** - Authentication tests (login, register, logout)
- **`e2e/teams.spec.ts`** - Team management tests
- **`e2e/races.spec.ts`** - Race viewing and game view tests
- **`e2e/market.spec.ts`** - Market page tests
- **`e2e/tracks.spec.ts`** - Track browsing tests
- **`e2e/drivers-cars.spec.ts`** - Driver and car detail tests

### 4. Helper Files

- **`e2e/helpers/test-setup.ts`** - Database and server setup utilities
- **`e2e/helpers/test-fixtures.ts`** - Reusable test helpers (login, navigation, screenshots)

### 5. Documentation

- **`e2e/TESTING_PLAN.md`** - Comprehensive testing plan
- **`e2e/README.md`** - User guide for running tests
- **`e2e/SETUP_SUMMARY.md`** - This file

## Quick Start

### 1. Set Up Test Database

```bash
npm run test:setup
```

This will:
- Start the test database container
- Wait for it to be ready
- Run migrations
- Seed with test data
- Create test user

### 2. Run Tests

```bash
# Run all tests (backend server starts automatically)
npm run test:e2e
```

The test infrastructure will automatically:
- Start the backend server with test database configuration
- Start the frontend dev server
- Wait for both servers to be ready
- Run all E2E tests
- Generate reports and screenshots

**Note**: You no longer need to manually start the backend server - it's handled automatically!

# Run in UI mode (interactive)
npm run test:e2e:ui

# Run in headed mode (see browser)
npm run test:e2e:headed
```

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

## Test Data

The test database is seeded with:
- 5 teams (Red Bull, Ferrari, Mercedes, McLaren, Alpine)
- 10 drivers (2 per team + 1 unassigned)
- 11 cars (2 per team + 1 unassigned)
- 6 tracks (Monaco, Bahrain, Bugatti, Le Mans, Circle, Test)
- 1 test user (`testuser`)

## Screenshots and Reports

- **Screenshots**: Automatically captured on test failure in `test-results/`
- **HTML Report**: Generated in `playwright-report/` - view with `npx playwright show-report`
- **Videos**: Recorded on failure in `test-results/`
- **Traces**: Collected on retry for debugging

## Key Features

1. **Isolated Test Database**: Completely separate from production/development
2. **Reproducible Data**: Consistent seed data for reliable tests
3. **Automatic Setup**: Global setup handles database initialization
4. **Comprehensive Coverage**: Tests for all major pages and flows
5. **Visual Regression**: Screenshots for detecting UI changes
6. **Detailed Reporting**: HTML reports with screenshots and traces

## Next Steps

1. **Run the tests** to verify everything works
2. **Adjust selectors** in test files based on your actual component structure
3. **Add more test cases** as you develop new features
4. **Set up CI/CD** integration using the examples in `e2e/README.md`

## Troubleshooting

### Database won't start
```bash
# Check if container exists
docker ps -a | grep tiny_racing_test_db

# Check logs
docker logs tiny_racing_test_db

# Restart
docker-compose -f docker-compose.test.yml restart
```

### Backend server issues
- Ensure ports 3000 and 3030 are available
- Check that `DATABASE_URL` is set correctly
- Verify Rust/Cargo is installed

### Test failures
- Check HTML report: `npx playwright show-report`
- Review screenshots in `test-results/`
- Run in headed mode: `npm run test:e2e:headed`
- Check browser console for errors

## Notes

- The backend server uses fixed ports (3000, 3030) - ensure these are available
- Tests run sequentially (1 worker) to avoid database conflicts
- The test database persists between runs (Docker volume)
- To reset: Stop container and remove volume, then run setup again

