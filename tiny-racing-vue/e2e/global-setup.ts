import { execSync } from 'child_process';
import { join, dirname } from 'path';
import { fileURLToPath } from 'url';

// ES module equivalent of __dirname
const __filename = fileURLToPath(import.meta.url);
const __dirname = dirname(__filename);

// global-setup.ts is in tiny-racing-vue/e2e/
// ROOT_DIR is tinyRacing/ (two levels up)
const ROOT_DIR = join(__dirname, '../..'); // tinyRacing root directory
const SERVER_DIR = join(ROOT_DIR, 'server');

/**
 * Setup function to start test database and seed it
 */
async function globalSetup() {
  console.log('Running global setup...');
  
  // Start test database
  console.log('Starting test database...');
  try {
    execSync('docker-compose -f docker-compose.test.yml up -d', {
      cwd: ROOT_DIR,
      stdio: 'inherit',
    });
    
    // Wait for database to be ready
    let retries = 30;
    while (retries > 0) {
      try {
        execSync('docker exec tiny_racing_test_db pg_isready -U tiny_racing_test', {
          stdio: 'pipe',
        });
        console.log('Database is ready!');
        break;
      } catch {
        retries--;
        if (retries === 0) {
          throw new Error('Database failed to become ready');
        }
        await new Promise(resolve => setTimeout(resolve, 1000));
      }
    }
    
    // Seed database with regular seed data
    console.log('Seeding test database...');
    const env = {
      ...process.env,
      DATABASE_URL: 'postgresql://tiny_racing_test:test_password@localhost:5433/tiny_racing_test',
    };
    
    execSync('cargo run --example seed_db', {
      cwd: SERVER_DIR,
      env,
      stdio: 'inherit',
    });
    
    // Ensure test user exists
    console.log('Ensuring test user exists...');
    try {
      execSync('cargo run --example seed_db_test', {
        cwd: SERVER_DIR,
        env,
        stdio: 'inherit',
      });
    } catch (error) {
      console.warn('Test user setup failed (may already exist):', error);
    }
    
    console.log('Test database setup complete!');
  } catch (error) {
    console.error('Global setup failed:', error);
    throw error;
  }
}

export default globalSetup;

