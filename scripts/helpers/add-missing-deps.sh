#!/bin/bash

echo "🔧 Adding missing dependencies to all services..."
echo "========================================"

# List of services that need dependencies
services=(
    "template_service"
    "sample_service"
    "enhanced_storage_service"
    "notification_service"
    "sequencing_service"
    "qaqc_service"
    "library_details_service"
    "cognitive_assistant_service"
    "dashboard_service"
    "reports_service"
    "event_service"
    "transaction_service"
    "spreadsheet_versioning_service"
)

# Check if dependencies already exist in Cargo.toml before adding
add_dependency() {
    local service=$1
    local dep_line=$2
    local dep_name=$(echo "$dep_line" | cut -d'=' -f1 | tr -d ' ')
    
    if [ -f "$service/Cargo.toml" ]; then
        if ! grep -q "^$dep_name" "$service/Cargo.toml"; then
            echo "$dep_line" >> "$service/Cargo.toml"
            echo "  ✅ Added $dep_name to $service"
        else
            echo "  ⏭️  $dep_name already exists in $service"
        fi
    fi
}

for service in "${services[@]}"; do
    if [ -f "$service/Cargo.toml" ]; then
        echo ""
        echo "📦 Processing $service..."
        
        # Add commonly missing dependencies
        add_dependency "$service" 'dotenvy = "0.15"'
        add_dependency "$service" 'validator = { version = "0.18", features = ["derive"] }'
        add_dependency "$service" 'futures = "0.3"'
        add_dependency "$service" 'base64 = "0.21"'
        add_dependency "$service" 'regex = "1.10"'
        add_dependency "$service" 'once_cell = "1.19"'
        add_dependency "$service" 'redis = { version = "0.24", features = ["tokio-comp", "connection-manager"] }'
        
        # Service-specific dependencies
        if [[ "$service" == *"storage"* ]]; then
            add_dependency "$service" 'rumqttc = "0.23"'
        fi
        
        if [[ "$service" == "event_service" ]] || [[ "$service" == "transaction_service" ]]; then
            add_dependency "$service" 'rdkafka = { version = "0.36", features = ["cmake-build"] }'
        fi
    else
        echo "⚠️  $service/Cargo.toml not found, skipping..."
    fi
done

echo ""
echo "🎉 Dependencies added successfully!"
