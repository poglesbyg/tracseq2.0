#!/bin/bash
set -e

# Wait for Redis
echo "Waiting for Redis..."
until nc -z redis 6379; do
  echo "Redis is unavailable - sleeping"
  sleep 1
done
echo "Redis is up!"

# Wait for PostgreSQL
echo "Waiting for PostgreSQL..."
until nc -z postgres 5432; do
  echo "PostgreSQL is unavailable - sleeping"
  sleep 1
done
echo "PostgreSQL is up!"

# Wait for Auth Service
echo "Waiting for Auth Service..."
until curl -f http://auth-service:8080/health >/dev/null 2>&1; do
  echo "Auth Service is unavailable - sleeping"
  sleep 2
done
echo "Auth Service is up!"

echo "All services are ready!" 