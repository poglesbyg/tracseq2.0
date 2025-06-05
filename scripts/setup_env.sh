#!/bin/bash

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# Function to generate a secure random string
generate_secret() {
    openssl rand -base64 32 | tr -d '\n'
}

# Function to prompt for input with default value
prompt_with_default() {
    local prompt="$1"
    local default="$2"
    local var_name="$3"
    
    read -p "$prompt [$default]: " input
    eval "$var_name=\${input:-$default}"
}

echo -e "${YELLOW}Setting up environment variables for Lab Manager${NC}\n"

# Generate a secure JWT secret
JWT_SECRET=$(generate_secret)
echo -e "${GREEN}Generated secure JWT secret${NC}"

# Prompt for domain
prompt_with_default "Enter your application domain" "localhost:3000" DOMAIN

# Set up SAML configuration
SAML_ENTITY_ID="https://$DOMAIN/saml"
SAML_ACS_URL="https://$DOMAIN/api/auth/sso/callback"
SAML_IDP_METADATA_URL="https://sso.unc.edu/idp/shibboleth"

# Create .env file
cat > .env << EOL
# Database Configuration
DATABASE_URL=postgres://postgres:postgres@localhost:5432/lab_manager
POSTGRES_USER=postgres
POSTGRES_PASSWORD=postgres
POSTGRES_DB=lab_manager

# Authentication Configuration
JWT_SECRET=$JWT_SECRET
SAML_ENTITY_ID=$SAML_ENTITY_ID
SAML_ACS_URL=$SAML_ACS_URL
SAML_IDP_METADATA_URL=$SAML_IDP_METADATA_URL

# Application Configuration
APP_ENV=development
APP_PORT=3000
APP_HOST=0.0.0.0
APP_URL=http://$DOMAIN

# Storage Configuration
STORAGE_PATH=./storage
MAX_UPLOAD_SIZE=10485760  # 10MB in bytes

# Logging Configuration
LOG_LEVEL=info
RUST_LOG=info,lab_manager=debug

# Security Configuration
CORS_ORIGINS=http://localhost:3000,http://localhost:5173
RATE_LIMIT_REQUESTS=100
RATE_LIMIT_DURATION=60  # in seconds
EOL

echo -e "\n${GREEN}Environment file created successfully!${NC}"
echo -e "\n${YELLOW}Important:${NC}"
echo "1. Review the generated .env file"
echo "2. Update the SAML configuration with your actual UNC SSO registration details"
echo "3. Make sure to keep your JWT_SECRET secure and never commit it to version control"
echo -e "\n${YELLOW}Next steps:${NC}"
echo "1. Register your application with UNC SSO using these values:"
echo "   - Entity ID: $SAML_ENTITY_ID"
echo "   - ACS URL: $SAML_ACS_URL"
echo "2. Update the SAML_IDP_METADATA_URL if needed"
echo "3. Start your application with: cargo run" 
