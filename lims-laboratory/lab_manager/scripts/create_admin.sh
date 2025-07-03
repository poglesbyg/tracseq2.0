#!/bin/bash

echo "🧪 Lab Manager - Quick Admin User Creation"
echo "=========================================="

# Check if database is running
if ! pg_isready -q; then
    echo "❌ PostgreSQL is not running. Please start it first."
    exit 1
fi

# Build and run the admin creation tool
echo "Building admin creation tool..."
cd "$(dirname "$0")/.."

if cargo build --bin create_admin; then
    echo "✅ Build successful. Starting admin creation..."
    cargo run --bin create_admin
else
    echo "❌ Build failed. Please check the error messages above."
    exit 1
fi 
