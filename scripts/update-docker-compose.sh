#!/bin/bash

# Update Docker compose files to reflect new directory structure

set -e

echo "🐳 Updating Docker compose files..."

# Find all docker-compose files
COMPOSE_FILES=$(find docker -name "docker-compose*.yml" -type f)

for file in $COMPOSE_FILES; do
    echo "📝 Updating $file..."
    
    # Create backup
    cp "$file" "$file.bak"
    
    # Update paths for enhanced services
    sed -i.tmp 's|lims-core/enhanced_storage_service|lims-enhanced/enhanced_storage_service|g' "$file"
    sed -i.tmp 's|lims-core/cognitive_assistant_service|lims-enhanced/cognitive_assistant_service|g' "$file"
    sed -i.tmp 's|lims-core/event_service|lims-enhanced/event_service|g' "$file"
    sed -i.tmp 's|lims-core/notification_service|lims-enhanced/notification_service|g' "$file"
    sed -i.tmp 's|lims-core/spreadsheet_versioning_service|lims-enhanced/spreadsheet_versioning_service|g' "$file"
    
    # Update paths for laboratory services
    sed -i.tmp 's|lims-core/lab_manager|lims-laboratory/lab_manager|g' "$file"
    sed -i.tmp 's|lims-core/library_prep_service|lims-laboratory/library_prep_service|g' "$file"
    sed -i.tmp 's|lims-core/library_details_service|lims-laboratory/library_details_service|g' "$file"
    sed -i.tmp 's|lims-core/sequencing_service|lims-laboratory/sequencing_service|g' "$file"
    sed -i.tmp 's|lims-core/qaqc_service|lims-laboratory/qaqc_service|g' "$file"
    sed -i.tmp 's|lims-core/flow_cell_service|lims-laboratory/flow_cell_service|g' "$file"
    
    # Update API gateway path
    sed -i.tmp 's|lims-core/api_gateway|lims-gateway/api_gateway|g' "$file"
    
    # Remove temporary files
    rm -f "$file.tmp"
done

echo ""
echo "✅ Docker compose files updated!"
echo ""
echo "📋 Updated files:"
for file in $COMPOSE_FILES; do
    echo "  - $file"
done 