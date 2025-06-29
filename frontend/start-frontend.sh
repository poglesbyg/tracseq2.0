#!/bin/sh
set -e

echo "Starting TracSeq 2.0 Frontend..."

# Wait for API Gateway if specified
if [ -n "$API_GATEWAY_HOST" ] && [ -n "$API_GATEWAY_PORT" ]; then
  echo "Waiting for API Gateway..."
  /wait-for-api-gateway.sh "$API_GATEWAY_HOST" "$API_GATEWAY_PORT"
fi

# Test nginx configuration
echo "Testing nginx configuration..."
nginx -t

# Start nginx
echo "Starting nginx..."
exec nginx -g "daemon off;" 