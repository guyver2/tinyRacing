import { execSync } from 'child_process';
import { join, dirname } from 'path';
import { fileURLToPath } from 'url';

// ES module equivalent of __dirname
const __filename = fileURLToPath(import.meta.url);
const __dirname = dirname(__filename);

// global-teardown.ts is in tiny-racing-vue/e2e/
// ROOT_DIR is tinyRacing/ (two levels up)
const ROOT_DIR = join(__dirname, '../..');

/**
 * Teardown function to clean up after tests
 * Playwright's webServer automatically kills servers it starts when tests complete.
 * This teardown only runs minimal cleanup and doesn't interfere with report generation.
 */
async function globalTeardown() {
  console.log('Running global teardown...');
  
  // Playwright handles killing webServer processes automatically.
  // We only do minimal cleanup here and don't interfere with report generation.
  
  // Optionally stop the database (comment out if you want to keep it running)
  // execSync('docker-compose -f docker-compose.test.yml down', {
  //   cwd: ROOT_DIR,
  //   stdio: 'inherit',
  // });
  
  console.log('Global teardown complete!');
}

export default globalTeardown;

