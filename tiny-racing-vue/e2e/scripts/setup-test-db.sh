#!/bin/bash
# Script to set up the test database: start, migrate, and seed

set -e

# Get the script directory and navigate to project root (tinyRacing)
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/../../.." && pwd)"
SERVER_DIR="$PROJECT_ROOT/server"

echo "Setting up test database..."
echo "Project root: $PROJECT_ROOT"

# Start the test database
echo "Starting test database container..."
cd "$PROJECT_ROOT"
docker-compose -f docker-compose.test.yml up -d

# Wait for database to be ready
echo "Waiting for database to be ready..."
sleep 2  # Give container a moment to start

for i in {1..60}; do
  # Check if container is running
  if ! docker ps | grep -q tiny_racing_test_db; then
    echo "Container is not running, waiting... ($i/60)"
    sleep 1
    continue
  fi
  
  # Check if database is accepting connections (use default postgres db for connection check)
  if docker exec tiny_racing_test_db pg_isready -U tiny_racing_test > /dev/null 2>&1; then
    echo "Database is ready!"
    # Verify the actual database exists
    sleep 1
    if docker exec tiny_racing_test_db psql -U tiny_racing_test -d tiny_racing_test -c "SELECT 1;" > /dev/null 2>&1; then
      echo "Database 'tiny_racing_test' is accessible!"
      break
    else
      echo "Database exists but not accessible yet, waiting... ($i/60)"
    fi
  else
    echo "Database not ready yet, waiting... ($i/60)"
  fi
  
  if [ $i -eq 60 ]; then
    echo "Database failed to become ready after 60 seconds"
    echo "Container status:"
    docker ps -a | grep tiny_racing_test_db || echo "Container not found"
    echo "Container logs (last 30 lines):"
    docker logs tiny_racing_test_db 2>&1 | tail -30
    exit 1
  fi
  
  sleep 1
done

# Run migrations and seed
echo "Running migrations and seeding database..."
cd "$SERVER_DIR"
export DATABASE_URL="postgresql://tiny_racing_test:test_password@localhost:5433/tiny_racing_test"

# First, ensure the database exists (it should from docker-compose, but just in case)
docker exec -i tiny_racing_test_db psql -U tiny_racing_test -c "SELECT 1 FROM pg_database WHERE datname='tiny_racing_test'" | grep -q 1 || \
  docker exec -i tiny_racing_test_db psql -U tiny_racing_test -c "CREATE DATABASE tiny_racing_test;"

# Seed the database with regular data
echo "Seeding database with teams, drivers, cars, tracks..."
cargo run --example seed_db

# Ensure test user exists
echo "Ensuring test user exists..."
cargo run --example seed_db_test || echo "Test user setup skipped (may already exist)"

echo "Test database setup complete!"
echo ""
echo "Test user credentials:"
echo "  Username: testuser"
echo "  Password: testpass123"
