#!/bin/bash

# Colors for output
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
RED='\033[0;31m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

print_info() { echo -e "${BLUE}[INFO]${NC} $1"; }
print_success() { echo -e "${GREEN}[SUCCESS]${NC} $1"; }
print_warning() { echo -e "${YELLOW}[WARNING]${NC} $1"; }
print_error() { echo -e "${RED}[ERROR]${NC} $1"; }

print_info "🚀 Starting Complete Lab Manager Application with User System"
echo

# Check if we're in the right directory
if [ ! -d "lab_manager" ] || [ ! -d "lab_submission_rag" ]; then
    print_error "Please run this script from the tracseq2.0 directory containing both lab_manager and lab_submission_rag folders"
    exit 1
fi

# Kill any existing processes
print_info "Cleaning up existing processes..."
pkill -f "lab_manager" 2>/dev/null || true
pkill -f "uvicorn.*main:app" 2>/dev/null || true
pkill -f "vite.*frontend" 2>/dev/null || true

# Function to check if port is available
check_port() {
    local port=$1
    if lsof -Pi :$port -sTCP:LISTEN -t >/dev/null 2>&1; then
        return 0  # Port is in use
    fi
    return 1  # Port is free
}

# Function to wait for service
wait_for_service() {
    local url=$1
    local service_name=$2
    local max_attempts=30
    local attempt=1
    
    print_info "Waiting for $service_name to be ready..."
    while [ $attempt -le $max_attempts ]; do
        if curl -s "$url" > /dev/null 2>&1; then
            print_success "$service_name is ready!"
            return 0
        fi
        echo -n "."
        sleep 2
        attempt=$((attempt + 1))
    done
    print_error "$service_name failed to start within $((max_attempts * 2)) seconds"
    return 1
}

# Check if PostgreSQL is running locally
print_info "Checking PostgreSQL database..."
if ! pg_isready -q 2>/dev/null; then
    print_warning "PostgreSQL not running locally. Starting with Docker..."
    cd lab_manager
    docker-compose down 2>/dev/null || true
    docker-compose up -d db
    DB_PORT=5433
    DB_HOST=localhost
    cd ..
    # Wait for database
    print_info "Waiting for Docker database to be ready..."
    sleep 8
else
    print_success "Using local PostgreSQL database"
    DB_PORT=5432
    DB_HOST=localhost
fi

# Setup database and user system
print_info "Setting up database and user system..."
cd lab_manager

# Create .env file with correct configuration
cat > .env << EOF
DATABASE_URL=postgres://lab_manager:lab_manager@${DB_HOST}:${DB_PORT}/lab_manager
STORAGE_PATH=./storage
HOST=0.0.0.0
PORT=8080
CORS_ENABLED=true
RUST_LOG=info
RAG_API_URL=http://127.0.0.1:8000
JWT_SECRET=your-super-secret-jwt-key-change-this-in-production
EOF

# Run database setup if needed
if [ "$DB_PORT" = "5432" ]; then
    print_info "Setting up local database..."
    ./scripts/setup.sh 2>/dev/null || true
fi

# Run migrations
if command -v sqlx >/dev/null 2>&1; then
    print_info "Running database migrations..."
    sqlx migrate run || print_warning "Migration failed - continuing anyway"
fi

# Create storage directory
mkdir -p storage

# Ensure admin user has correct password
print_info "Setting up admin user with correct password..."
export DATABASE_URL="postgres://lab_manager:lab_manager@${DB_HOST}:${DB_PORT}/lab_manager"
export PGPASSWORD="lab_manager"

# Update admin password hash using SQL directly
psql -h $DB_HOST -p $DB_PORT -U lab_manager -d lab_manager -c "
UPDATE users 
SET password_hash = '\$argon2id\$v=19\$m=19456,t=2,p=1\$PvuEaXN3YH/FJTijYPcxHQ\$ou3lWvq/bw3z4Y2ax7unRqeG26+C7shIX0VW9UGSVa0' 
WHERE email = 'admin@lab.local';
" 2>/dev/null || print_warning "Could not update admin password - continuing anyway"

cd ..

# Start Python RAG API
print_info "Starting Python RAG API on port 8000..."
cd lab_submission_rag

if [ ! -d ".venv" ]; then
    print_info "Creating Python virtual environment..."
    python3 -m venv .venv
fi

source .venv/bin/activate
pip install -r requirements.txt > /dev/null 2>&1

# Set environment variables for RAG
export PYTHONPATH="${PYTHONPATH}:$(pwd)/app"
export RAG_DATA_PATH="$(pwd)/app/data"

cd app
nohup python -m uvicorn api.main:app --host 0.0.0.0 --port 8000 > /tmp/rag_api.log 2>&1 &
RAG_PID=$!
echo $RAG_PID > /tmp/rag_api.pid
cd ../..

# Start Rust Backend on port 8080
print_info "Starting Rust backend on port 8080..."
cd lab_manager

# Source the environment file and start the backend with correct binary
source .env
nohup cargo run --bin lab_manager > /tmp/lab_manager_backend.log 2>&1 &
BACKEND_PID=$!
echo $BACKEND_PID > /tmp/lab_manager_backend.pid
cd ..

# Start React Frontend
print_info "Starting React frontend on port 5173..."
cd lab_manager/frontend

# Ensure Vite config is correct for proxy
cat > vite.config.ts << 'EOF'
import { defineConfig } from 'vite'
import react from '@vitejs/plugin-react'

// https://vitejs.dev/config/
export default defineConfig({
  plugins: [react()],
  server: {
    host: true,
    port: 5173,
    proxy: {
      '/api': {
        target: 'http://localhost:8080',
        changeOrigin: true,
        secure: false,
        configure: (proxy, _options) => {
          proxy.on('error', (err, _req, _res) => {
            console.log('proxy error', err);
          });
          proxy.on('proxyReq', (proxyReq, req, _res) => {
            console.log('Sending Request to the Target:', req.method, req.url);
          });
          proxy.on('proxyRes', (proxyRes, req, _res) => {
            console.log('Received Response from the Target:', proxyRes.statusCode, req.url);
          });
        },
      },
      '/health': {
        target: 'http://localhost:8080',
        changeOrigin: true,
        secure: false,
      },
    },
  },
})
EOF

# Create frontend environment file
cat > .env << EOF
VITE_API_URL=
EOF

if [ ! -d "node_modules" ]; then
    print_info "Installing frontend dependencies..."
    npm install
fi

nohup npm run dev > /tmp/lab_manager_frontend.log 2>&1 &
FRONTEND_PID=$!
echo $FRONTEND_PID > /tmp/lab_manager_frontend.pid
cd ../..

# Wait for services to start
print_info "Waiting for services to start..."
sleep 20

# Check service health with better validation
echo
print_info "Checking service health..."

# Check RAG API
if wait_for_service "http://localhost:8000/health" "RAG API"; then
    print_success "✓ RAG API: Running on http://localhost:8000"
else
    print_error "✗ RAG API: Failed to start"
fi

# Check Backend
if wait_for_service "http://localhost:8080/health" "Backend API"; then
    print_success "✓ Backend API: Running on http://localhost:8080"
else
    print_error "✗ Backend API: Failed to start"
fi

# Check Frontend
if check_port 5173; then
    print_success "✓ Frontend: Running on http://localhost:5173"
else
    print_error "✗ Frontend: Failed to start"
fi

# Check Database
if check_port $DB_PORT; then
    print_success "✓ Database: Running on localhost:${DB_PORT}"
else
    print_error "✗ Database: Failed to start"
fi

echo
print_success "🎉 Lab Manager Application with User System is now running!"
echo
print_info "🔐 Default Admin Credentials:"
echo "  • Email: admin@lab.local"
echo "  • Password: admin123"
echo "  • Role: Lab Administrator"
echo
print_warning "⚠️  Please change the default admin password after first login!"
echo
print_info "🌐 Access Points:"
echo "  • Main Application: http://localhost:5173"
echo "  • Backend API: http://localhost:8080"
echo "  • RAG API: http://localhost:8000"
echo "  • Database: localhost:${DB_PORT}"
echo
print_info "🔧 User Management Features:"
echo "  • Create/Edit/Delete users (Admin only)"
echo "  • Role-based access control"
echo "  • Profile management"
echo "  • Session management"
echo
print_info "📁 Log Files:"
echo "  • Backend: /tmp/lab_manager_backend.log"
echo "  • Frontend: /tmp/lab_manager_frontend.log"
echo "  • RAG API: /tmp/rag_api.log"
echo
print_info "🛠️  Useful Commands:"
echo "  • View logs: tail -f /tmp/*.log"
echo "  • Stop all: ./stop_full_app.sh"
echo "  • Check auth endpoints: curl http://localhost:8080/api/auth/login"
echo
print_info "📚 User Roles Available:"
echo "  • Lab Administrator: Full system access"
echo "  • Principal Investigator: Lab oversight"
echo "  • Lab Technician: Sample processing"
echo "  • Research Scientist: Research activities"
echo "  • Data Analyst: Data analysis and reporting"
echo "  • Guest: Limited read-only access"
echo

# Check if authentication works with correct endpoint
print_info "🔍 Testing authentication system..."
sleep 5

if curl -s "http://localhost:8080/api/auth/login" -X POST \
   -H "Content-Type: application/json" \
   -d '{"email":"admin@lab.local","password":"admin123"}' | grep -q "success"; then
    print_success "✓ User authentication system is working!"
else
    print_warning "⚠ User authentication test failed - check logs for details"
    print_info "You can test manually: curl -X POST http://localhost:8080/api/auth/login -H 'Content-Type: application/json' -d '{\"email\":\"admin@lab.local\",\"password\":\"admin123\"}'"
fi

echo
print_warning "Keep this terminal open or services will stop!"
print_info "Press Ctrl+C to view live logs, or close terminal to keep running in background"

# Option to show logs
echo
read -p "Press Enter to view live logs, or Ctrl+C to exit: " -r
echo
print_info "Showing live logs (Ctrl+C to stop)..."
tail -f /tmp/lab_manager_backend.log /tmp/lab_manager_frontend.log /tmp/rag_api.log 
