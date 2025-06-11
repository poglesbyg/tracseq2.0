#!/bin/bash

# Setup test database for lab_manager tests

set -e

echo "🔧 Setting up test database for lab_manager..."

# Database configuration
DB_USER="${DB_USER:-lab_manager}"
DB_PASSWORD="${DB_PASSWORD:-lab_manager}"
DB_HOST="${DB_HOST:-localhost}"
DB_PORT="${DB_PORT:-5432}"
DB_NAME="lab_manager_test"

# Check if PostgreSQL is running
if ! pg_isready -h "$DB_HOST" -p "$DB_PORT" -U "$DB_USER" 2>/dev/null; then
    echo "❌ PostgreSQL is not running on $DB_HOST:$DB_PORT"
    echo "Please start PostgreSQL and ensure the lab_manager user exists."
    exit 1
fi

echo "✅ PostgreSQL is running"

# Check if test database exists
if psql -h "$DB_HOST" -p "$DB_PORT" -U "$DB_USER" -lqt | cut -d \| -f 1 | grep -qw "$DB_NAME"; then
    echo "🗃️  Test database '$DB_NAME' already exists"
    echo "ℹ️  Using existing test database"
else
    # Create test database
    echo "📦 Creating test database '$DB_NAME'..."
    createdb -h "$DB_HOST" -p "$DB_PORT" -U "$DB_USER" "$DB_NAME"
fi

# Set environment variable for tests
export TEST_DATABASE_URL="postgres://$DB_USER:$DB_PASSWORD@$DB_HOST:$DB_PORT/$DB_NAME"

echo "✅ Test database setup complete!"
echo "🔗 Test database URL: $TEST_DATABASE_URL"
echo ""
echo "📋 To run tests with database:"
echo "   export TEST_DATABASE_URL=\"$TEST_DATABASE_URL\""
echo "   cargo test --bin lab_manager"
echo ""
echo "🧪 To run only unit tests (no database):"
echo "   cargo test --lib" 
