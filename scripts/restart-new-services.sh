#!/bin/bash

# Restart new microservices for TracSeq 2.0

echo "Stopping existing services..."
docker-compose -f docker/docker-compose.new-features.yml down

echo "Removing old images..."
docker rmi docker:project-service docker:library-prep-service docker:qaqc-service docker:flow-cell-service 2>/dev/null || true

echo "Building services..."
docker-compose -f docker/docker-compose.new-features.yml build --no-cache

echo "Starting services..."
docker-compose -f docker/docker-compose.new-features.yml up -d

echo "Waiting for services to start..."
sleep 10

echo "Checking service health..."
echo -n "Project Service: "
curl -s http://localhost:8101/health || echo "Not responding"

echo -n "Library Prep Service: "
curl -s http://localhost:8102/health || echo "Not responding"

echo -n "QA/QC Service: "
curl -s http://localhost:8103/health || echo "Not responding"

echo -n "Flow Cell Service: "
curl -s http://localhost:8104/health || echo "Not responding"

echo ""
echo "Service logs:"
docker-compose -f docker/docker-compose.new-features.yml logs --tail=10 