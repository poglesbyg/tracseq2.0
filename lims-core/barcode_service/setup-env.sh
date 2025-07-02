#!/bin/bash

# Setup environment variables for barcode service development
# This script sets the DATABASE_URL required for SQLx compile-time query verification

echo "Setting up environment for barcode service..."

# Export DATABASE_URL for SQLx
export DATABASE_URL="postgres://postgres:postgres@localhost:5432/barcode_service"

echo "DATABASE_URL has been set for SQLx compile-time verification"
echo ""
echo "To use this in your current shell, run:"
echo "  source setup-env.sh"
echo ""
echo "Or add this to your shell profile (~/.bashrc, ~/.zshrc, etc.):"
echo "  export DATABASE_URL=\"postgres://postgres:postgres@localhost:5432/barcode_service\"" 