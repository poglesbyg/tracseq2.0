#!/bin/bash

# Deploy New Microservices for TracSeq 2.0
# This script builds and deploys the new feature microservices

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

echo -e "${GREEN}🚀 Deploying New TracSeq 2.0 Microservices${NC}"

# Check if the main services are running
if ! docker ps | grep -q "tracseq-postgres-primary"; then
    echo -e "${RED}❌ Error: PostgreSQL is not running. Please start the main services first.${NC}"
    exit 1
fi

if ! docker ps | grep -q "tracseq-api-gateway"; then
    echo -e "${YELLOW}⚠️  Warning: API Gateway is not running. Services will be accessible directly only.${NC}"
fi

# Build the services
echo -e "${YELLOW}📦 Building microservices...${NC}"

cd lims-core

# Build each service
for service in project_service library_prep_service qaqc_service flow_cell_service; do
    echo -e "${YELLOW}Building $service...${NC}"
    cd $service
    if cargo build --release; then
        echo -e "${GREEN}✅ $service built successfully${NC}"
    else
        echo -e "${RED}❌ Failed to build $service${NC}"
        exit 1
    fi
    cd ..
done

cd ..

# Start the services
echo -e "${YELLOW}🏃 Starting services with Docker Compose...${NC}"
docker-compose -f docker/docker-compose.new-features.yml up -d --build

# Wait for services to be healthy
echo -e "${YELLOW}⏳ Waiting for services to be healthy...${NC}"

services=("project-service:8084" "library-prep-service:8085" "qaqc-service:8089" "flow-cell-service:8086")

for service_port in "${services[@]}"; do
    service="${service_port%:*}"
    port="${service_port#*:}"
    
    echo -n "Checking $service on port $port..."
    
    for i in {1..30}; do
        if curl -f "http://localhost:$port/health" >/dev/null 2>&1; then
            echo -e " ${GREEN}✅${NC}"
            break
        else
            echo -n "."
            sleep 2
        fi
        
        if [ $i -eq 30 ]; then
            echo -e " ${RED}❌ Failed${NC}"
            echo -e "${RED}Service $service did not become healthy in time${NC}"
            exit 1
        fi
    done
done

echo -e "${GREEN}✅ All services are healthy!${NC}"

# Display service URLs
echo -e "\n${GREEN}📡 Service Endpoints:${NC}"
echo "Project Service:      http://localhost:8084/health"
echo "Library Prep Service: http://localhost:8085/health"
echo "QA/QC Service:        http://localhost:8089/health"
echo "Flow Cell Service:    http://localhost:8086/health"

if docker ps | grep -q "tracseq-api-gateway"; then
    echo -e "\n${GREEN}🌐 API Gateway Routes:${NC}"
    echo "Projects:     http://localhost:8089/projects/api/v1/*"
    echo "Library Prep: http://localhost:8089/library-prep/api/v1/*"
    echo "QA/QC:        http://localhost:8089/qc/api/v1/*"
    echo "Flow Cells:   http://localhost:8089/flow-cells/api/v1/*"
fi

echo -e "\n${GREEN}✨ Deployment complete!${NC}" 