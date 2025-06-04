#!/bin/bash

# Colors for output
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

echo -e "${YELLOW}Starting Lab Manager Project...${NC}"

# Check if Docker is running
if ! docker info > /dev/null 2>&1; then
    echo "Docker is not running. Please start Docker first."
    exit 1
fi

# Create necessary directories
mkdir -p storage

# Create .env file if it doesn't exist
if [ ! -f .env ]; then
    echo -e "${YELLOW}Creating .env file...${NC}"
    cat > .env << EOL
DATABASE_URL=postgres://postgres:postgres@db:5432/lab_manager
STORAGE_PATH=/usr/local/bin/storage
RUST_LOG=info
EOL
fi

# Function to check if a port is in use
check_port() {
    if lsof -Pi :$1 -sTCP:LISTEN -t >/dev/null 2>&1; then
        echo -e "${YELLOW}Warning: Port $1 is already in use. You may need to stop other services.${NC}"
        return 1
    fi
    return 0
}

# Check if required ports are available
echo -e "${YELLOW}Checking port availability...${NC}"
ports_in_use=0
check_port 8080 || ((ports_in_use++))
check_port 3000 || ((ports_in_use++))
check_port 3001 || ((ports_in_use++))
check_port 5173 || ((ports_in_use++))
check_port 5432 || ((ports_in_use++))

if [ $ports_in_use -gt 0 ]; then
    echo -e "${YELLOW}$ports_in_use ports are in use. Continuing anyway...${NC}"
fi

# Stop any existing containers
echo -e "${YELLOW}Stopping any existing containers...${NC}"
docker-compose down

# Build and start the services
echo -e "${YELLOW}Building and starting services...${NC}"
docker-compose up --build -d

# Wait for services to be ready
echo -e "${YELLOW}Waiting for services to be ready...${NC}"
sleep 10

# Check if services are running
echo -e "${YELLOW}Checking service status...${NC}"
if docker-compose ps | grep -q "Up"; then
    echo -e "${GREEN}All services are running!${NC}"
    echo -e "\nAccess points:"
    echo -e "${GREEN}Frontend:${NC} http://localhost:8080"
    echo -e "${GREEN}API:${NC} http://localhost:3001"
    echo -e "${GREEN}Development Frontend:${NC} http://localhost:5173"
    echo -e "${GREEN}Development API:${NC} http://localhost:3000"
else
    echo "Some services failed to start. Check the logs with: docker-compose logs"
fi

# Show logs
echo -e "\n${YELLOW}Showing logs (press Ctrl+C to exit)...${NC}"
docker-compose logs -f 
