#!/bin/bash
set -e

echo "Starting TracSeq 2.0 API Gateway..."

# Wait for dependencies
/wait-for-services.sh

# Start the API Gateway
echo "Starting API Gateway service..."
exec python src/api_gateway/simple_main.py 