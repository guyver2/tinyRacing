# E2E Testing Troubleshooting Guide

## Backend Server Not Responding

If you see "Network error" or the API server isn't responding:

### Check 1: Is the server starting?

The backend server should start automatically when you run tests. Check the Playwright output for:
- `[WebServer]` logs showing cargo compilation
- `API server listening on http://localhost:3000` message

### Check 2: Manual server start

Try starting the server manually to see if there are errors:

```bash
cd ../server
DATABASE_URL=postgresql://tiny_racing_test:test_password@localhost:5433/tiny_racing_test DISABLE_UI=true cargo run
```

### Check 3: Test database is running

Ensure the test database is running:

```bash
docker ps | grep tiny_racing_test_db
```

If not running:
```bash
npm run test:setup
```

### Check 4: Port conflicts

Check if port 3000 is already in use:

```bash
lsof -i :3000
# or
netstat -tlnp | grep :3000
```

If something else is using port 3000, stop it or change the backend port.

### Check 5: API URL mismatch

The frontend should connect to `http://localhost:3000` (no `/api` prefix). Verify in:
- `playwright.config.ts` - `VITE_API_URL` should be `http://localhost:3000`
- The server routes are at root level (e.g., `/auth/login`, not `/api/auth/login`)

### Check 6: Cargo build issues

If cargo is taking too long or failing:
- First build can take 2-3 minutes
- Check for compilation errors in Playwright output
- Try building manually: `cd server && cargo build`

### Check 7: Database connection

The server needs to connect to the test database. Verify:
- Test database is running on port 5433
- `DATABASE_URL` environment variable is set correctly
- Database migrations have run successfully

## Common Solutions

### Solution 1: Increase timeout

If the server takes longer to start, increase the timeout in `playwright.config.ts`:

```typescript
timeout: 300 * 1000, // 5 minutes
```

### Solution 2: Start server manually

Instead of relying on Playwright's webServer, start the backend manually:

```bash
# Terminal 1: Start backend
cd server
DATABASE_URL=postgresql://tiny_racing_test:test_password@localhost:5433/tiny_racing_test DISABLE_UI=true cargo run

# Terminal 2: Run tests
npm run test:e2e
```

Then set `reuseExistingServer: true` in the backend webServer config.

### Solution 3: Check server logs

Enable stdout/stderr output to see server logs:

```typescript
stdout: 'inherit', // Instead of 'pipe'
stderr: 'inherit',
```

### Solution 4: Verify API endpoint

Test the API manually:

```bash
curl http://localhost:3000/tracks?limit=1
```

Should return JSON with tracks data.

## Debug Mode

Run tests in debug mode to see what's happening:

```bash
PLAYWRIGHT_USE_SYSTEM_BROWSERS=1 npx playwright test --debug --grep "should login"
```

This opens Playwright Inspector where you can step through the test and see network requests.

