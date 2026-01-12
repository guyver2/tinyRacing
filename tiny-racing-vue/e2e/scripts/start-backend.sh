#!/bin/bash
# Script to start the backend server with test database configuration
# This is used by Playwright's webServer configuration

set -e

# Get the script directory and navigate to project root (tinyRacing)
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/../../.." && pwd)"
SERVER_DIR="$PROJECT_ROOT/server"

# Test database configuration
export DATABASE_URL="postgresql://tiny_racing_test:test_password@localhost:5433/tiny_racing_test"
export DISABLE_UI="true"

# Change to server directory
cd "$SERVER_DIR"

# Build and run the server
# Note: The server runs on fixed ports 3000 and 3030
# Make sure these ports are available or modify server code to accept port configuration
exec cargo run

