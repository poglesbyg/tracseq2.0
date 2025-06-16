#!/bin/bash

# Test script for dashboard functionality
# Run this script to verify the dashboard API endpoints are working

echo "Testing Lab Manager Dashboard API..."
echo "======================================"

# Check if backend is running
echo "1. Testing health check endpoint..."
curl -s http://localhost:3000/health | jq . || echo "Health check failed - is the backend running?"

echo -e "\n2. Testing dashboard stats endpoint..."
curl -s http://localhost:3000/api/dashboard/stats | jq . || echo "Dashboard stats failed - check database connection"

echo -e "\n3. Testing templates endpoint..."
curl -s http://localhost:3000/api/templates | jq . || echo "Templates endpoint failed"

echo -e "\n4. Testing samples endpoint..."
curl -s http://localhost:3000/api/samples | jq . || echo "Samples endpoint failed"

echo -e "\n5. Testing sequencing jobs endpoint..."
curl -s http://localhost:3000/api/sequencing/jobs | jq . || echo "Sequencing jobs endpoint failed"

echo -e "\nDashboard API test complete!"
echo "If any endpoints failed, check that:"
echo "  - Backend server is running on port 3000"
echo "  - Database is connected and migrated"
echo "  - All required environment variables are set" 
