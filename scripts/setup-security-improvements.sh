#!/bin/bash

# TracSeq 2.0 Security Improvements Setup Script
# This script implements the critical security and performance improvements

set -e  # Exit on any error

echo "🔐 TracSeq 2.0 Security Improvements Setup"
echo "=========================================="

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Function to print colored output
print_status() {
    echo -e "${BLUE}[INFO]${NC} $1"
}

print_warning() {
    echo -e "${YELLOW}[WARNING]${NC} $1"
}

print_error() {
    echo -e "${RED}[ERROR]${NC} $1"
}

print_success() {
    echo -e "${GREEN}[SUCCESS]${NC} $1"
}

# Check if we're in the right directory
if [ ! -f "Cargo.toml" ] || [ ! -d "lab_manager" ]; then
    print_error "Please run this script from the TracSeq 2.0 root directory"
    exit 1
fi

print_status "Starting security improvements setup..."

# 1. Generate secure secrets
print_status "Step 1: Generating secure secrets..."

# Function to generate secure secret
generate_secret() {
    if command -v openssl >/dev/null 2>&1; then
        openssl rand -base64 32 | tr -d '\n'
    else
        print_warning "OpenSSL not found, generating fallback secret"
        # Fallback using /dev/urandom
        head -c 32 /dev/urandom | base64 | tr -d '\n'
    fi
}

JWT_SECRET=$(generate_secret)
DB_PASSWORD=$(generate_secret)

print_success "Generated secure JWT secret (${#JWT_SECRET} characters)"
print_success "Generated secure database password (${#DB_PASSWORD} characters)"

# 2. Update environment files with generated secrets
print_status "Step 2: Updating environment files with secure secrets..."

# Create backup of existing environment files
if [ -f "lab_manager/env.development" ]; then
    cp "lab_manager/env.development" "lab_manager/env.development.backup.$(date +%Y%m%d_%H%M%S)"
    print_status "Backed up existing env.development file"
fi

if [ -f "lab_manager/env.production" ]; then
    cp "lab_manager/env.production" "lab_manager/env.production.backup.$(date +%Y%m%d_%H%M%S)"
    print_status "Backed up existing env.production file"
fi

# Update development environment
sed -i.bak "s/JWT_SECRET=.*/JWT_SECRET=${JWT_SECRET}/" lab_manager/env.development
print_success "Updated development JWT secret"

# Update production environment  
sed -i.bak "s/JWT_SECRET=.*/JWT_SECRET=${JWT_SECRET}/" lab_manager/env.production
sed -i.bak "s/POSTGRES_PASSWORD=.*/POSTGRES_PASSWORD=${DB_PASSWORD}/" lab_manager/env.production
sed -i.bak "s/:REPLACE_WITH_SECURE_DATABASE_PASSWORD_32_CHARS_MIN/:${DB_PASSWORD}/g" lab_manager/env.production
print_success "Updated production secrets"

# Update unified environment file
if [ -f "deploy/tracseq.env" ]; then
    sed -i.bak "s/JWT_SECRET=.*/JWT_SECRET=${JWT_SECRET}/" deploy/tracseq.env
    sed -i.bak "s/POSTGRES_PASSWORD=.*/POSTGRES_PASSWORD=${DB_PASSWORD}/" deploy/tracseq.env
    sed -i.bak "s/:REPLACE_WITH_SECURE_DB_PASSWORD_FOR_PRODUCTION/:${DB_PASSWORD}/g" deploy/tracseq.env
    print_success "Updated unified environment file"
fi

# 3. Build with new dependencies
print_status "Step 3: Building with new security dependencies..."

cd lab_manager

# Check if Rust is installed
if ! command -v cargo >/dev/null 2>&1; then
    print_error "Rust/Cargo not found. Please install Rust first: https://rustup.rs/"
    exit 1
fi

print_status "Running cargo check to verify dependencies..."
if cargo check --features security; then
    print_success "Cargo check passed with security features"
else
    print_warning "Cargo check had issues, continuing anyway..."
fi

# 4. Run database migrations
print_status "Step 4: Preparing to run database migrations..."

print_warning "Database migrations need to be run when your database is available."
print_status "To run migrations:"
echo "  1. Ensure PostgreSQL is running"
echo "  2. Set DATABASE_URL environment variable"
echo "  3. Run: sqlx migrate run --source migrations"

# 5. Test health endpoints
print_status "Step 5: Testing setup..."

print_status "Building the application..."
if cargo build --release --features production; then
    print_success "Application built successfully with production features"
else
    print_error "Build failed. Please check the error messages above."
    exit 1
fi

# 6. Create a quick verification script
print_status "Step 6: Creating verification script..."

cat > ../verify-security-setup.sh << 'EOF'
#!/bin/bash

echo "🔍 TracSeq 2.0 Security Setup Verification"
echo "========================================="

# Test health endpoints when server is running
test_endpoint() {
    local url=$1
    local name=$2
    
    if curl -s -f "$url" > /dev/null; then
        echo "✅ $name endpoint is responding"
    else
        echo "❌ $name endpoint is not responding"
    fi
}

echo "Testing health endpoints (server must be running)..."
test_endpoint "http://localhost:3000/health" "Basic health"
test_endpoint "http://localhost:3000/health/system" "System health"
test_endpoint "http://localhost:3000/health/database" "Database health"
test_endpoint "http://localhost:3000/health/ready" "Readiness"
test_endpoint "http://localhost:3000/health/live" "Liveness"

echo ""
echo "Check environment variables:"
echo "JWT_SECRET length: $(echo $JWT_SECRET | wc -c) characters (should be >30)"
echo "DATABASE_URL is set: $([ -n "$DATABASE_URL" ] && echo "✅ Yes" || echo "❌ No")"

echo ""
echo "Next steps:"
echo "1. Start your services: docker-compose up -d"
echo "2. Run database migrations: sqlx migrate run --source lab_manager/migrations"
echo "3. Test the endpoints above"
echo "4. Check logs for any security warnings"
EOF

chmod +x ../verify-security-setup.sh
print_success "Created verification script: verify-security-setup.sh"

cd ..

# 7. Summary and next steps
print_success "Security improvements setup completed!"
echo ""
echo "📋 Summary of changes:"
echo "  ✅ Generated secure JWT secret and database password"
echo "  ✅ Updated environment files with secure secrets"
echo "  ✅ Added comprehensive security dependencies"
echo "  ✅ Implemented input validation middleware"
echo "  ✅ Added comprehensive health check endpoints"
echo "  ✅ Created database migrations for indexes and audit logging"
echo ""
echo "🚀 Next steps:"
echo "  1. Review the updated environment files"
echo "  2. Start your services: docker-compose up -d"
echo "  3. Run database migrations:"
echo "     cd lab_manager && sqlx migrate run --source migrations"
echo "  4. Test the setup: ./verify-security-setup.sh"
echo "  5. Monitor logs for any issues"
echo ""
echo "⚠️  Important security notes:"
echo "  • Keep your generated secrets secure and never commit them to version control"
echo "  • Regularly rotate secrets in production"
echo "  • Monitor the new health endpoints for system status"
echo "  • Review security audit logs regularly"
echo ""
echo "📍 New health endpoints available:"
echo "  • GET /health - Basic health check"
echo "  • GET /health/system - Comprehensive system health"
echo "  • GET /health/database - Database-specific health"
echo "  • GET /health/metrics - Application metrics"
echo "  • GET /health/ready - Kubernetes readiness probe"
echo "  • GET /health/live - Kubernetes liveness probe"
echo ""

print_success "Setup complete! 🎉" 
