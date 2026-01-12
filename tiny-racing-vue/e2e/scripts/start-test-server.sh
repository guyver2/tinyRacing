#!/bin/bash
# Script to start the backend server with test database configuration

set -e

PROJECT_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
SERVER_DIR="$PROJECT_ROOT/server"

# Test database configuration
export DATABASE_URL="postgresql://tiny_racing_test:test_password@localhost:5433/tiny_racing_test"
export DISABLE_UI="true"

# Change to server directory and start the server
cd "$SERVER_DIR"

echo "Starting test backend server..."
echo "DATABASE_URL: $DATABASE_URL"
echo "DISABLE_UI: $DISABLE_UI"

# Build and run the server
# The server will run on ports 3000 and 3030 by default
# We'll need to modify the server to accept port configuration or use a proxy
exec cargo run

