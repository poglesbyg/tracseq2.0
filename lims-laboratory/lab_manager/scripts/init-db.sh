#!/bin/bash
set -e

echo "Waiting for PostgreSQL to be ready..."
until pg_isready -h db -p 5432 -U postgres; do
    echo "PostgreSQL is unavailable - sleeping"
    sleep 1
done

echo "PostgreSQL is up - executing database initialization"

# Create database if it doesn't exist
psql -h db -U postgres -tc "SELECT 1 FROM pg_database WHERE datname = 'lab_manager'" | grep -q 1 || \
    psql -h db -U postgres -c "CREATE DATABASE lab_manager"

# Run migrations
echo "Running database migrations..."
cargo sqlx database create
cargo sqlx migrate run

echo "Database initialization completed successfully" 
