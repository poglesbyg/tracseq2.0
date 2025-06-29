#!/bin/bash

# Fix workspace dependencies in all microservices

set -e

echo "🔄 Fixing workspace dependencies in microservices..."

services=(
    "sequencing_service"
    "notification_service" 
    "event_service"
    "qaqc_service"
    "spreadsheet_versioning_service"
    "library_details_service"
)

for service in "${services[@]}"; do
    echo "🔧 Fixing $service..."
    
    # Replace main sqlx dependency
    if [ -f "$service/Cargo.toml" ]; then
        sed -i '' 's/sqlx = { workspace = true }/sqlx = { version = "0.8", features = ["runtime-tokio-rustls", "postgres", "uuid", "chrono", "json", "macros", "migrate"] }/g' "$service/Cargo.toml"
        echo "✅ Fixed $service"
    else
        echo "⚠️  $service/Cargo.toml not found"
    fi
done

echo "✅ All workspace dependencies fixed!" 