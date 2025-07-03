#!/bin/bash

# Update build scripts to reflect new directory structure

set -e

echo "🔧 Updating build scripts with new directory paths..."

# Update dev.sh
if [ -f "scripts/dev.sh" ]; then
    echo "📝 Updating scripts/dev.sh..."
    sed -i.bak 's|lims-core/enhanced_storage_service|lims-enhanced/enhanced_storage_service|g' scripts/dev.sh
    echo "  ✅ Updated dev.sh"
fi

# Find and update all shell scripts that might reference old paths
echo "🔍 Searching for other scripts with old paths..."

# List of old path to new path mappings
declare -A path_mappings=(
    ["lims-core/enhanced_storage_service"]="lims-enhanced/enhanced_storage_service"
    ["lims-core/cognitive_assistant_service"]="lims-enhanced/cognitive_assistant_service"
    ["lims-core/event_service"]="lims-enhanced/event_service"
    ["lims-core/notification_service"]="lims-enhanced/notification_service"
    ["lims-core/spreadsheet_versioning_service"]="lims-enhanced/spreadsheet_versioning_service"
    ["lims-core/lab_manager"]="lims-laboratory/lab_manager"
    ["lims-core/library_prep_service"]="lims-laboratory/library_prep_service"
    ["lims-core/library_details_service"]="lims-laboratory/library_details_service"
    ["lims-core/sequencing_service"]="lims-laboratory/sequencing_service"
    ["lims-core/qaqc_service"]="lims-laboratory/qaqc_service"
    ["lims-core/flow_cell_service"]="lims-laboratory/flow_cell_service"
    ["lims-core/api_gateway"]="lims-gateway/api_gateway"
)

# Update all shell scripts
for script in scripts/*.sh scripts/*.ps1; do
    if [ -f "$script" ] && [ "$script" != "scripts/reorganize-services.sh" ] && [ "$script" != "scripts/update-cargo-workspace.sh" ] && [ "$script" != "scripts/update-docker-compose.sh" ] && [ "$script" != "scripts/update-build-scripts.sh" ]; then
        modified=false
        for old_path in "${!path_mappings[@]}"; do
            new_path="${path_mappings[$old_path]}"
            if grep -q "$old_path" "$script" 2>/dev/null; then
                if [ "$modified" = false ]; then
                    echo "📝 Updating $script..."
                    cp "$script" "$script.bak"
                    modified=true
                fi
                sed -i "s|$old_path|$new_path|g" "$script"
                echo "  ✅ Replaced $old_path → $new_path"
            fi
        done
    fi
done

echo ""
echo "✅ Build scripts updated!"
echo ""
echo "📋 Backup files created with .bak extension" 