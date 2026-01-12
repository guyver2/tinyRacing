import { execSync } from 'child_process';
import { existsSync } from 'fs';
import { join, dirname } from 'path';
import { fileURLToPath } from 'url';

// ES module equivalent of __dirname
const __filename = fileURLToPath(import.meta.url);
const __dirname = dirname(__filename);

// test-setup.ts is in tiny-racing-vue/e2e/helpers/
// ROOT_DIR is tinyRacing/ (three levels up)
const ROOT_DIR = join(__dirname, '../../../..');
const SERVER_DIR = join(ROOT_DIR, 'server');

export interface TestConfig {
  databaseUrl: string;
  apiUrl: string;
  wsUrl: string;
  testUser: {
    username: string;
    password: string;
  };
}

export const TEST_CONFIG: TestConfig = {
  databaseUrl: 'postgresql://tiny_racing_test:test_password@localhost:5433/tiny_racing_test',
  apiUrl: 'http://localhost:3001',
  wsUrl: 'ws://localhost:3031',
  testUser: {
    username: 'testuser',
    password: 'testpass123',
  },
};

/**
 * Start the test database using docker-compose
 */
export function startTestDatabase(): void {
  console.log('Starting test database...');
  try {
    execSync('docker-compose -f docker-compose.test.yml up -d', {
      cwd: ROOT_DIR,
      stdio: 'inherit',
    });
    
    // Wait for database to be ready
    console.log('Waiting for database to be ready...');
    let retries = 30;
    while (retries > 0) {
      try {
        execSync(
          `docker exec tiny_racing_test_db pg_isready -U tiny_racing_test`,
          { stdio: 'pipe' }
        );
        console.log('Database is ready!');
        break;
      } catch (e) {
        retries--;
        if (retries === 0) {
          throw new Error('Database failed to become ready');
        }
        // Wait 1 second before retrying
        execSync('sleep 1', { stdio: 'pipe' });
      }
    }
  } catch (error) {
    console.error('Failed to start test database:', error);
    throw error;
  }
}

/**
 * Stop the test database
 */
export function stopTestDatabase(): void {
  console.log('Stopping test database...');
  try {
    execSync('docker-compose -f docker-compose.test.yml down', {
      cwd: ROOT_DIR,
      stdio: 'inherit',
    });
  } catch (error) {
    console.error('Failed to stop test database:', error);
    // Don't throw - cleanup should be best effort
  }
}

/**
 * Seed the test database
 */
export function seedTestDatabase(): void {
  console.log('Seeding test database...');
  try {
    // Set environment variable and run seed script
    const env = {
      ...process.env,
      DATABASE_URL: TEST_CONFIG.databaseUrl,
    };
    
    execSync('cargo run --example seed_db', {
      cwd: SERVER_DIR,
      env,
      stdio: 'inherit',
    });
    
    console.log('Database seeded successfully!');
  } catch (error) {
    console.error('Failed to seed test database:', error);
    throw error;
  }
}

/**
 * Reset the test database (drop and recreate)
 */
export function resetTestDatabase(): void {
  console.log('Resetting test database...');
  try {
    // Drop and recreate the database
    execSync(
      `docker exec -i tiny_racing_test_db psql -U tiny_racing_test -c "DROP DATABASE IF EXISTS tiny_racing_test;"`,
      { stdio: 'inherit' }
    );
    execSync(
      `docker exec -i tiny_racing_test_db psql -U tiny_racing_test -c "CREATE DATABASE tiny_racing_test;"`,
      { stdio: 'inherit' }
    );
    
    // Run migrations and seed
    seedTestDatabase();
  } catch (error) {
    console.error('Failed to reset test database:', error);
    throw error;
  }
}

/**
 * Check if test database is running
 */
export function isTestDatabaseRunning(): boolean {
  try {
    execSync(
      `docker exec tiny_racing_test_db pg_isready -U tiny_racing_test`,
      { stdio: 'pipe' }
    );
    return true;
  } catch {
    return false;
  }
}

