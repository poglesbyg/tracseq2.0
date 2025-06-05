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
DATABASE_URL=postgres://postgres:postgres@localhost:5432/lab_manager
STORAGE_PATH=./storage
SERVER_HOST=0.0.0.0
SERVER_PORT=3000
CORS_ENABLED=true
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
check_port 3000 || ((ports_in_use++))
check_port 5173 || ((ports_in_use++))
check_port 5432 || ((ports_in_use++))

if [ $ports_in_use -gt 0 ]; then
    echo -e "${YELLOW}$ports_in_use ports are in use. Continuing anyway...${NC}"
fi

# Stop any existing containers
echo -e "${YELLOW}Stopping any existing containers...${NC}"
docker-compose down

# Start just the database
echo -e "${YELLOW}Starting PostgreSQL database...${NC}"
docker-compose up -d db

# Wait for database to be ready
echo -e "${YELLOW}Waiting for database to be ready...${NC}"
sleep 5

# Run migrations
echo -e "${YELLOW}Running database migrations...${NC}"
if command -v sqlx &> /dev/null; then
    sqlx migrate run || echo "Migration failed - continuing anyway"
else
    echo "SQLx CLI not found - skipping migrations"
fi

# Check if we should start services automatically
if [ "$1" = "--docker" ]; then
    echo -e "${YELLOW}Starting all services with Docker...${NC}"
    docker-compose up --build -d
    sleep 10
else
    echo -e "${YELLOW}Database ready! You can now start the services manually:${NC}"
    echo "Backend: cargo run"
    echo "Frontend: cd frontend && npm run dev"
    echo ""
    echo "Or run with --docker flag to start all services automatically"
fi

# Check if services are running and show appropriate information
if [ "$1" = "--docker" ]; then
    echo -e "${YELLOW}Checking service status...${NC}"
    if docker-compose ps | grep -q "Up"; then
        echo -e "${GREEN}All services are running!${NC}"
        echo -e "\nAccess points:"
        echo -e "${GREEN}Frontend (Dev):${NC} http://localhost:5173"
        echo -e "${GREEN}Backend API:${NC} http://localhost:3000"
        echo -e "${GREEN}Database:${NC} localhost:5432"
        
        # Show logs
        echo -e "\n${YELLOW}Showing logs (press Ctrl+C to exit)...${NC}"
        docker-compose logs -f
    else
        echo "Some services failed to start. Check the logs with: docker-compose logs"
    fi
else
    echo -e "${GREEN}Setup complete!${NC}"
    echo -e "\nTo start the application:"
    echo -e "1. ${YELLOW}Backend:${NC} cargo run"
    echo -e "2. ${YELLOW}Frontend:${NC} cd frontend && npm run dev"
    echo -e "\nThen access: ${GREEN}http://localhost:5173${NC}"
    echo -e "\nDatabase is already running on: ${GREEN}localhost:5432${NC}"
fi 
