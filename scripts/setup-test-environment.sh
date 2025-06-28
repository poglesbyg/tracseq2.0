#!/bin/bash

# TracSeq 2.0 Test Environment Setup Script

set -e

echo "🚀 Setting up TracSeq 2.0 test environment..."

# Colors for output
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
RED='\033[0;31m'
NC='\033[0m' # No Color

# Database configuration
DB_HOST="${DB_HOST:-localhost}"
DB_PORT="${DB_PORT:-5432}"
DB_USER="${DB_USER:-tracseq}"
DB_PASSWORD="${DB_PASSWORD:-tracseq}"
DB_NAME="${DB_NAME:-tracseq_test}"

# Export DATABASE_URL for SQLx
export DATABASE_URL="postgresql://${DB_USER}:${DB_PASSWORD}@${DB_HOST}:${DB_PORT}/${DB_NAME}"

echo -e "${YELLOW}📋 Configuration:${NC}"
echo "  Database URL: $DATABASE_URL"
echo "  Rust toolchain: $(rustc --version)"

# Check if PostgreSQL is running
if ! pg_isready -h $DB_HOST -p $DB_PORT > /dev/null 2>&1; then
    echo -e "${RED}❌ PostgreSQL is not running on ${DB_HOST}:${DB_PORT}${NC}"
    echo "Please start PostgreSQL or update the database configuration"
    exit 1
fi

echo -e "${GREEN}✅ PostgreSQL is running${NC}"

# Create test database if it doesn't exist
echo -e "${YELLOW}🗄️ Setting up test database...${NC}"
if ! PGPASSWORD=$DB_PASSWORD psql -h $DB_HOST -p $DB_PORT -U $DB_USER -lqt | cut -d \| -f 1 | grep -qw $DB_NAME; then
    PGPASSWORD=$DB_PASSWORD createdb -h $DB_HOST -p $DB_PORT -U $DB_USER $DB_NAME
    echo -e "${GREEN}✅ Created database: $DB_NAME${NC}"
else
    echo -e "${GREEN}✅ Database already exists: $DB_NAME${NC}"
fi

# Run migrations for each service
echo -e "${YELLOW}🔄 Running database migrations...${NC}"

services=(
    "auth_service"
    "sample_service"
    "sequencing_service"
    "notification_service"
    "qaqc_service"
    "library_details_service"
    "enhanced_storage_service"
    "spreadsheet_versioning_service"
    "transaction_service"
    "template_service"
)

for service in "${services[@]}"; do
    if [ -d "$service/migrations" ]; then
        echo -e "${YELLOW}  Running migrations for $service...${NC}"
        (cd $service && sqlx migrate run) || echo -e "${YELLOW}  ⚠️  No migrations found or failed for $service${NC}"
    fi
done

# Create .env file if it doesn't exist
if [ ! -f .env ]; then
    echo -e "${YELLOW}📝 Creating .env file...${NC}"
    cat > .env << EOF
# TracSeq 2.0 Test Environment Configuration
DATABASE_URL=${DATABASE_URL}
RUST_LOG=debug
TEST_DATABASE_URL=${DATABASE_URL}

# Service URLs
AUTH_SERVICE_URL=http://localhost:8080
SAMPLE_SERVICE_URL=http://localhost:8081
SEQUENCING_SERVICE_URL=http://localhost:8082
NOTIFICATION_SERVICE_URL=http://localhost:8083
QAQC_SERVICE_URL=http://localhost:8084
LIBRARY_DETAILS_SERVICE_URL=http://localhost:8085
RAG_SERVICE_URL=http://localhost:8086
EVENT_SERVICE_URL=http://localhost:8087
TRANSACTION_SERVICE_URL=http://localhost:8088
ENHANCED_STORAGE_SERVICE_URL=http://localhost:8089
SPREADSHEET_SERVICE_URL=http://localhost:8090
TEMPLATE_SERVICE_URL=http://localhost:8091

# Redis configuration
REDIS_URL=redis://localhost:6379

# JWT Secret (for testing only!)
JWT_SECRET=test-secret-key-do-not-use-in-production
EOF
    echo -e "${GREEN}✅ Created .env file${NC}"
fi

# Install SQLx CLI if not present
if ! command -v sqlx &> /dev/null; then
    echo -e "${YELLOW}📦 Installing sqlx-cli...${NC}"
    cargo install sqlx-cli --no-default-features --features postgres
fi

# Prepare SQLx offline mode for each service
echo -e "${YELLOW}🔧 Preparing SQLx offline mode...${NC}"
for service in "${services[@]}"; do
    if [ -f "$service/Cargo.toml" ] && grep -q "sqlx" "$service/Cargo.toml"; then
        echo -e "${YELLOW}  Preparing $service...${NC}"
        (cd $service && cargo sqlx prepare --workspace) || echo -e "${YELLOW}  ⚠️  SQLx prepare failed for $service${NC}"
    fi
done

echo -e "${GREEN}✅ Test environment setup complete!${NC}"
echo ""
echo -e "${YELLOW}📋 Next steps:${NC}"
echo "  1. Run tests: cargo test --workspace"
echo "  2. Run specific service tests: cargo test -p <service_name>"
echo "  3. Run with features: cargo test --workspace --all-features"
echo ""
echo -e "${GREEN}Happy testing! 🚀${NC}"