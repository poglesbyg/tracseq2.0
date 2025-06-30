#!/bin/sh
set -e

host="$1"
port="$2"

echo "Waiting for API Gateway at $host:$port..."

until curl -f "http://$host:$port/health" >/dev/null 2>&1; do
  echo "Still waiting for API Gateway..."
  sleep 2
done

echo "API Gateway is ready!" 