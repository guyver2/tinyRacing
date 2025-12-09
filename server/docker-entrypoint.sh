#!/bin/bash
set -e

# Wait for database to be ready
echo "Waiting for database to be ready..."
until pg_isready -h postgres -U "${DB_USER:-tiny_racing}" -d "${DB_NAME:-tiny_racing}" > /dev/null 2>&1; do
  echo "Database is unavailable - sleeping"
  sleep 1
done

echo "Database is ready!"

# Start the application (migrations will run automatically in the Rust code)
exec /app/tiny_racing /app/assets/race.json

