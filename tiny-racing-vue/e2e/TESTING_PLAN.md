# End-to-End Testing Plan for Tiny Racing

## Overview

This document outlines the comprehensive E2E testing strategy for the Tiny Racing application using Playwright. The testing infrastructure ensures reproducible, isolated tests that don't interfere with production or development databases.

## Architecture

### Test Infrastructure Components

1. **Isolated Test Database**
   - Separate PostgreSQL instance on port `5433` (vs production `5432`)
   - Database name: `tiny_racing_test`
   - User: `tiny_racing_test`
   - Password: `test_password`
   - Runs in Docker container: `tiny_racing_test_db`

2. **Test Server**
   - Rust backend server configured to use test database
   - Runs on ports `3001` (API) and `3031` (WebSocket) to avoid conflicts
   - Environment: `DISABLE_UI=true`, `DATABASE_URL` pointing to test DB

3. **Test Data Seeding**
   - Reproducible seed script based on `server/examples/seed_db.rs`
   - Creates consistent test data: teams, drivers, cars, tracks, players
   - Test user credentials: `testuser` / `testpass123`

4. **Frontend Test Server**
   - Vue dev server on port `5173` (default)
   - Configured to connect to test backend API/WebSocket

## Test Database Setup

### Docker Compose Configuration

The test database runs in isolation using `docker-compose.test.yml`:
- Port: `5433` (different from production `5432`)
- Database name: `tiny_racing_test`
- Persistent volume for test data (can be reset between test runs)

### Database Lifecycle

1. **Before Tests**: Start test DB, run migrations, seed data
2. **During Tests**: Tests run against seeded database
3. **After Tests**: Optionally reset database for next run

## Test Coverage

### Pages and Routes to Test

#### 1. Authentication Pages
- **Login** (`/login`)
  - [ ] Form validation
  - [ ] Successful login
  - [ ] Invalid credentials
  - [ ] Redirect after login
  - [ ] Logout functionality

- **Register** (`/register`)
  - [ ] Form validation
  - [ ] Successful registration
  - [ ] Duplicate username handling
  - [ ] Password requirements

#### 2. Game/Race View (`/`)
- [ ] Page loads without active race
- [ ] WebSocket connection status
- [ ] Race header displays correctly
- [ ] Track visualization renders
- [ ] Cars table displays player's cars
- [ ] Driver controls are functional
- [ ] Race start/pause controls

#### 3. Team Management
- **My Team** (`/my-team`)
  - [ ] Displays user's team information
  - [ ] Team stats and details
  - [ ] Team drivers list
  - [ ] Team cars list
  - [ ] Cash balance display

- **All Teams** (`/teams`)
  - [ ] Lists all teams
  - [ ] Team cards display correctly
  - [ ] Navigation to team detail

- **Team Detail** (`/teams/:teamId`)
  - [ ] Team information displays
  - [ ] Team drivers
  - [ ] Team cars
  - [ ] Team statistics

#### 4. Races (`/races`)
- [ ] Race list displays
- [ ] Race cards show correct information
- [ ] Navigation to race detail
- [ ] Race creation (if applicable)
- [ ] Race status updates

#### 5. Drivers (`/drivers/:driverId`)
- [ ] Driver profile displays
- [ ] Driver statistics
- [ ] Driver history
- [ ] Driver stats charts

#### 6. Cars (`/cars/:carId`)
- [ ] Car details display
- [ ] Car statistics
- [ ] Car performance charts
- [ ] Car assignment information

#### 7. Market (`/market`)
- [ ] Market page loads
- [ ] Available drivers list
- [ ] Available cars list
- [ ] Purchase functionality (if applicable)
- [ ] Filtering and search

#### 8. Tracks
- **Tracks List** (`/tracks`)
  - [ ] All tracks display
  - [ ] Track cards show information
  - [ ] Navigation to track detail

- **Track Detail** (`/tracks/:trackId`)
  - [ ] Track information
  - [ ] Track statistics
  - [ ] Track visualization
  - [ ] Track configuration

### Functional Testing

#### User Flows
1. **New User Registration Flow**
   - Register → Login → View My Team → Browse Market

2. **Race Participation Flow**
   - Login → Navigate to Races → Start Race → View Race → Control Drivers

3. **Team Management Flow**
   - Login → View My Team → View All Teams → View Team Detail

4. **Market Interaction Flow**
   - Login → Navigate to Market → View Available Drivers/Cars → Purchase (if applicable)

#### API Integration Tests
- [ ] API endpoints return correct data
- [ ] Authentication required endpoints
- [ ] Error handling for invalid requests
- [ ] WebSocket connection and updates

#### WebSocket Testing
- [ ] Connection establishment
- [ ] Race state updates
- [ ] Reconnection handling
- [ ] Connection status display

## Test Execution

### Running Tests

```bash
# Run all E2E tests
npm run test:e2e

# Run tests in UI mode (headed)
npm run test:e2e:ui

# Run specific test file
npx playwright test e2e/auth.spec.ts

# Run tests with debug
npx playwright test --debug
```

### Test Environment Setup

The test infrastructure is automatically managed:
1. Test database starts via docker-compose
2. Database migrations run automatically
3. Test data seeded before tests
4. Backend server starts with test configuration
5. Frontend dev server starts
6. Tests execute
7. Cleanup (optional database reset)

## Screenshots and Reports

### Screenshot Strategy
- **On Failure**: Automatic screenshots saved to `test-results/`
- **Visual Regression**: Screenshots for key pages saved to `e2e/screenshots/`
- **Before/After**: Screenshots for critical user flows

### Report Generation
- **HTML Report**: Generated after each test run in `playwright-report/`
- **Screenshots**: Organized by test name and timestamp
- **Traces**: Saved for failed tests (configurable)

### Visual Regression Testing
- Baseline screenshots stored in `e2e/screenshots/baseline/`
- Comparison screenshots in `e2e/screenshots/actual/`
- Diff images in `e2e/screenshots/diff/`

## Test Data

### Seed Data Structure

The test database is seeded with:
- **5 Teams**: Red Bull, Ferrari, Mercedes, McLaren, Alpine
- **10 Drivers**: 2 per team + 1 unassigned
- **11 Cars**: 2 per team + 1 unassigned
- **6 Tracks**: Monaco, Bahrain, Bugatti, Le Mans, Circle, Test
- **1 Test Player**: `testuser` / `testpass123` (owns Team #1)

### Reproducibility

- All seed data uses fixed values (no randomization)
- UUIDs are deterministic based on seed order
- Timestamps are fixed or relative to test start
- Database is reset before each test run (optional)

## CI/CD Integration

### GitHub Actions / CI Pipeline

```yaml
# Example CI configuration
- Start test database
- Run migrations
- Seed test data
- Start backend server
- Start frontend server
- Run Playwright tests
- Generate and upload reports
- Cleanup
```

### Test Parallelization
- Tests can run in parallel (configured in Playwright)
- Each test worker gets isolated database state
- Or: Sequential execution with shared test database

## Maintenance

### Updating Test Data
- Modify `server/examples/seed_db.rs` or create test-specific seed
- Ensure seed script is idempotent (can run multiple times)

### Adding New Tests
- Follow existing test structure
- Use page object pattern for complex pages
- Add screenshots for visual regression
- Update this plan document

### Debugging Failed Tests
1. Check HTML report for detailed error messages
2. Review screenshots in `test-results/`
3. Check traces for step-by-step execution
4. Verify test database state
5. Check server logs

## Future Enhancements

- [ ] Visual regression testing with Percy or similar
- [ ] Performance testing (Lighthouse CI)
- [ ] Accessibility testing (axe-core)
- [ ] Cross-browser testing (Chrome, Firefox, Safari)
- [ ] Mobile viewport testing
- [ ] API contract testing
- [ ] Load testing for WebSocket connections

