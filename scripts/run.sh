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
    if lsof -Pi :$1 -sTCP:LISTEN -t >/dev/null ; then
        echo "Port $1 is already in use. Please free up this port and try again."
        exit 1
    fi
}

# Check if required ports are available
check_port 80
check_port 3000
check_port 3001
check_port 5173
check_port 5432

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
    echo -e "${GREEN}Frontend:${NC} http://localhost"
    echo -e "${GREEN}API:${NC} http://localhost/api"
    echo -e "${GREEN}Development Frontend:${NC} http://localhost:5173"
    echo -e "${GREEN}Development API:${NC} http://localhost:3000"
else
    echo "Some services failed to start. Check the logs with: docker-compose logs"
fi

# Show logs
echo -e "\n${YELLOW}Showing logs (press Ctrl+C to exit)...${NC}"
docker-compose logs -f 
