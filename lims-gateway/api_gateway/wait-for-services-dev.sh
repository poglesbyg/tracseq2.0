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

# Wait for Dashboard Service (acting as auth service)
echo "Waiting for Dashboard Service..."
until curl -f http://dashboard-service:8080/health >/dev/null 2>&1; do
  echo "Dashboard Service is unavailable - sleeping"
  sleep 2
done
echo "Dashboard Service is up!"

# Wait for Samples Service
echo "Waiting for Samples Service..."
until curl -f http://samples-service:8080/health >/dev/null 2>&1; do
  echo "Samples Service is unavailable - sleeping"
  sleep 2
done
echo "Samples Service is up!"

# Wait for Sequencing Service
echo "Waiting for Sequencing Service..."
until curl -f http://sequencing-service:8080/health >/dev/null 2>&1; do
  echo "Sequencing Service is unavailable - sleeping"
  sleep 2
done
echo "Sequencing Service is up!"

# Wait for Spreadsheet Service
echo "Waiting for Spreadsheet Service..."
until curl -f http://spreadsheet-service:8080/health >/dev/null 2>&1; do
  echo "Spreadsheet Service is unavailable - sleeping"
  sleep 2
done
echo "Spreadsheet Service is up!"

echo "All services are ready!" 