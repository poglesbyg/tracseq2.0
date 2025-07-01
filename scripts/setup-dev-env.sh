#!/bin/bash
# TracSeq 2.0 Development Environment Setup Script (Bash)
# This script sets up all necessary environment variables for development

echo "Setting up TracSeq 2.0 Development Environment..."

# Get the directory of this script
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
ENV_FILE="$SCRIPT_DIR/../deploy/services.env"

if [[ -f "$ENV_FILE" ]]; then
    echo "Loading environment variables from $ENV_FILE"
    
    # Export variables from the env file
    while IFS='=' read -r key value; do
        # Skip empty lines and comments
        if [[ $key && $value && ! $key =~ ^[[:space:]]*# ]]; then
            # Remove any quotes and whitespace
            key=$(echo "$key" | xargs)
            value=$(echo "$value" | xargs)
            export "$key=$value"
            echo "Set $key"
        fi
    done < "$ENV_FILE"
else
    echo "Environment file not found: $ENV_FILE"
    echo "Creating default environment variables..."
    
    # Core database configuration
    export DATABASE_URL="postgresql://postgres:postgres@localhost:5433/lab_manager"
    export SQLX_OFFLINE="false"
    export RUST_LOG="info"
    
    # Service ports
    export LAB_MANAGER_PORT="3000"
    export AUTH_SERVICE_PORT="8001"
    export SAMPLE_SERVICE_PORT="8002"
    export SEQUENCING_SERVICE_PORT="8003"
    export TRANSACTION_SERVICE_PORT="8006"
    export EVENT_SERVICE_PORT="8008"
    
    # Security
    export JWT_SECRET="tracseq_jwt_secret_2024_change_in_production"
    
    echo "Default environment variables set"
fi

# Display current environment
echo ""
echo "Current Environment Configuration:"
echo "DATABASE_URL: $DATABASE_URL"
echo "RUST_LOG: $RUST_LOG"
echo "LAB_MANAGER_PORT: $LAB_MANAGER_PORT"
echo "SEQUENCING_SERVICE_PORT: $SEQUENCING_SERVICE_PORT"

echo ""
echo "Environment setup complete!"
echo "You can now run: cargo build --workspace" 
