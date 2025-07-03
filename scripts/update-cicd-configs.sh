#!/bin/bash

# Update CI/CD configurations to reflect new directory structure

set -e

echo "🚀 Updating CI/CD configurations..."

# Update GitHub Actions workflows
if [ -d ".github/workflows" ]; then
    echo "📝 Updating GitHub Actions workflows..."
    
    for workflow in .github/workflows/*.yml; do
        if [ -f "$workflow" ]; then
            modified=false
            
            # Check if file contains old paths
            if grep -q "lims-core/api_gateway\|lims-core/enhanced_storage_service\|lims-core/event_service\|lims-core/notification_service\|lims-core/cognitive_assistant_service\|lims-core/spreadsheet_versioning_service\|lims-core/lab_manager\|lims-core/library_prep_service\|lims-core/library_details_service\|lims-core/sequencing_service\|lims-core/qaqc_service\|lims-core/flow_cell_service" "$workflow"; then
                echo "  📝 Updating $workflow..."
                cp "$workflow" "$workflow.bak"
                
                # Update paths
                sed -i 's|lims-core/enhanced_storage_service|lims-enhanced/enhanced_storage_service|g' "$workflow"
                sed -i 's|lims-core/cognitive_assistant_service|lims-enhanced/cognitive_assistant_service|g' "$workflow"
                sed -i 's|lims-core/event_service|lims-enhanced/event_service|g' "$workflow"
                sed -i 's|lims-core/notification_service|lims-enhanced/notification_service|g' "$workflow"
                sed -i 's|lims-core/spreadsheet_versioning_service|lims-enhanced/spreadsheet_versioning_service|g' "$workflow"
                sed -i 's|lims-core/lab_manager|lims-laboratory/lab_manager|g' "$workflow"
                sed -i 's|lims-core/library_prep_service|lims-laboratory/library_prep_service|g' "$workflow"
                sed -i 's|lims-core/library_details_service|lims-laboratory/library_details_service|g' "$workflow"
                sed -i 's|lims-core/sequencing_service|lims-laboratory/sequencing_service|g' "$workflow"
                sed -i 's|lims-core/qaqc_service|lims-laboratory/qaqc_service|g' "$workflow"
                sed -i 's|lims-core/flow_cell_service|lims-laboratory/flow_cell_service|g' "$workflow"
                sed -i 's|lims-core/api_gateway|lims-gateway/api_gateway|g' "$workflow"
                
                echo "  ✅ Updated $(basename $workflow)"
            fi
        fi
    done
fi

# Update GitLab CI if it exists
if [ -f ".gitlab-ci.yml" ]; then
    echo "📝 Updating GitLab CI configuration..."
    cp .gitlab-ci.yml .gitlab-ci.yml.bak
    
    sed -i 's|lims-core/enhanced_storage_service|lims-enhanced/enhanced_storage_service|g' .gitlab-ci.yml
    sed -i 's|lims-core/cognitive_assistant_service|lims-enhanced/cognitive_assistant_service|g' .gitlab-ci.yml
    sed -i 's|lims-core/event_service|lims-enhanced/event_service|g' .gitlab-ci.yml
    sed -i 's|lims-core/notification_service|lims-enhanced/notification_service|g' .gitlab-ci.yml
    sed -i 's|lims-core/spreadsheet_versioning_service|lims-enhanced/spreadsheet_versioning_service|g' .gitlab-ci.yml
    sed -i 's|lims-core/lab_manager|lims-laboratory/lab_manager|g' .gitlab-ci.yml
    sed -i 's|lims-core/library_prep_service|lims-laboratory/library_prep_service|g' .gitlab-ci.yml
    sed -i 's|lims-core/library_details_service|lims-laboratory/library_details_service|g' .gitlab-ci.yml
    sed -i 's|lims-core/sequencing_service|lims-laboratory/sequencing_service|g' .gitlab-ci.yml
    sed -i 's|lims-core/qaqc_service|lims-laboratory/qaqc_service|g' .gitlab-ci.yml
    sed -i 's|lims-core/flow_cell_service|lims-laboratory/flow_cell_service|g' .gitlab-ci.yml
    sed -i 's|lims-core/api_gateway|lims-gateway/api_gateway|g' .gitlab-ci.yml
    
    echo "  ✅ Updated GitLab CI"
fi

# Update Jenkins files if they exist
if [ -f "Jenkinsfile" ]; then
    echo "📝 Updating Jenkinsfile..."
    cp Jenkinsfile Jenkinsfile.bak
    
    sed -i 's|lims-core/enhanced_storage_service|lims-enhanced/enhanced_storage_service|g' Jenkinsfile
    sed -i 's|lims-core/cognitive_assistant_service|lims-enhanced/cognitive_assistant_service|g' Jenkinsfile
    sed -i 's|lims-core/event_service|lims-enhanced/event_service|g' Jenkinsfile
    sed -i 's|lims-core/notification_service|lims-enhanced/notification_service|g' Jenkinsfile
    sed -i 's|lims-core/spreadsheet_versioning_service|lims-enhanced/spreadsheet_versioning_service|g' Jenkinsfile
    sed -i 's|lims-core/lab_manager|lims-laboratory/lab_manager|g' Jenkinsfile
    sed -i 's|lims-core/library_prep_service|lims-laboratory/library_prep_service|g' Jenkinsfile
    sed -i 's|lims-core/library_details_service|lims-laboratory/library_details_service|g' Jenkinsfile
    sed -i 's|lims-core/sequencing_service|lims-laboratory/sequencing_service|g' Jenkinsfile
    sed -i 's|lims-core/qaqc_service|lims-laboratory/qaqc_service|g' Jenkinsfile
    sed -i 's|lims-core/flow_cell_service|lims-laboratory/flow_cell_service|g' Jenkinsfile
    sed -i 's|lims-core/api_gateway|lims-gateway/api_gateway|g' Jenkinsfile
    
    echo "  ✅ Updated Jenkinsfile"
fi

echo ""
echo "✅ CI/CD configurations updated!"
echo ""
echo "📋 Files updated:"
find .github/workflows -name "*.yml.bak" -exec basename {} .bak \; 2>/dev/null | sed 's/^/  - /'
[ -f ".gitlab-ci.yml.bak" ] && echo "  - .gitlab-ci.yml"
[ -f "Jenkinsfile.bak" ] && echo "  - Jenkinsfile" 