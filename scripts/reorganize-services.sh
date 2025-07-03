#!/bin/bash

# TracSeq 2.0 Service Reorganization Script
# This script reorganizes the service structure for better maintainability

set -e  # Exit on error

echo "🚀 Starting TracSeq 2.0 service reorganization..."

# Check if we're in the correct directory
if [ ! -f "Cargo.toml" ] || [ ! -d "lims-core" ]; then
    echo "❌ Error: Please run this script from the project root directory"
    exit 1
fi

# Create timestamp for backup
TIMESTAMP=$(date +%Y%m%d_%H%M%S)
BACKUP_DIR="backup_reorganization_$TIMESTAMP"

echo "📦 Creating backup directory: $BACKUP_DIR"
mkdir -p "$BACKUP_DIR"

# Backup current Cargo.toml and docker-compose files
echo "💾 Backing up configuration files..."
cp Cargo.toml "$BACKUP_DIR/Cargo.toml.bak"
cp -r docker/*.yml "$BACKUP_DIR/" 2>/dev/null || true

# Create new directory structure
echo "📁 Creating new directory structure..."
mkdir -p lims-enhanced
mkdir -p lims-laboratory  
mkdir -p lims-gateway

# Remove duplicate flow_cell_service at root if it exists
if [ -d "flow_cell_service" ]; then
    echo "🗑️  Removing duplicate flow_cell_service at root..."
    rm -rf flow_cell_service
fi

# Move API Gateway
echo "🔄 Moving API Gateway..."
if [ -d "lims-core/api_gateway" ]; then
    mv lims-core/api_gateway lims-gateway/
    echo "  ✅ Moved api_gateway to lims-gateway/"
fi

# Move enhanced services
echo "🔄 Moving enhanced services..."
ENHANCED_SERVICES=(
    "enhanced_storage_service"
    "cognitive_assistant_service"
    "event_service"
    "notification_service"
    "spreadsheet_versioning_service"
)

for service in "${ENHANCED_SERVICES[@]}"; do
    if [ -d "lims-core/$service" ]; then
        mv "lims-core/$service" "lims-enhanced/"
        echo "  ✅ Moved $service to lims-enhanced/"
    fi
done

# Move saga-enhanced to enhanced services
if [ -d "saga-enhanced" ]; then
    mv saga-enhanced lims-enhanced/saga_orchestrator
    echo "  ✅ Moved saga-enhanced to lims-enhanced/saga_orchestrator"
fi

# Move laboratory-specific services
echo "🔄 Moving laboratory services..."
LAB_SERVICES=(
    "lab_manager"
    "library_prep_service"
    "library_details_service"
    "sequencing_service"
    "qaqc_service"
    "flow_cell_service"
)

for service in "${LAB_SERVICES[@]}"; do
    if [ -d "lims-core/$service" ]; then
        mv "lims-core/$service" "lims-laboratory/"
        echo "  ✅ Moved $service to lims-laboratory/"
    fi
done

# Remove qaqc_service_new if it exists (keeping the original)
if [ -d "lims-core/qaqc_service_new" ]; then
    echo "🗑️  Removing qaqc_service_new (keeping original qaqc_service)..."
    rm -rf lims-core/qaqc_service_new
fi

# List remaining services in lims-core
echo ""
echo "📋 Services remaining in lims-core:"
ls -1 lims-core/ | grep -v "target" | grep -v "Cargo" | sed 's/^/  - /'

echo ""
echo "✅ Directory reorganization complete!"
echo ""
echo "⚠️  Next steps:"
echo "1. Update Cargo.toml workspace members"
echo "2. Update Docker compose files"
echo "3. Update build scripts"
echo ""
echo "Run './scripts/update-cargo-workspace.sh' to update Cargo.toml" 