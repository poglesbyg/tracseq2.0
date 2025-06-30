#!/bin/bash

# Create standalone Cargo.toml for services without workspace dependencies
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
)

for service in "${services[@]}"; do
    if [ -f "$service/Cargo.toml" ]; then
        echo "Creating standalone Cargo.toml for $service..."
        
        # Extract package info from existing Cargo.toml
        name=$(grep "^name" "$service/Cargo.toml" | head -1 | cut -d'"' -f2)
        
        # Create standalone version
        cat > "$service/Cargo.standalone.toml" << 'CARGOEOF'
[package]
name = "SERVICE_NAME"
version = "0.1.0"
edition = "2021"

[dependencies]
tokio = { version = "1.36", features = ["full"] }
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
uuid = { version = "1.6.1", features = ["v4", "serde"] }
chrono = { version = "0.4", features = ["serde"] }
anyhow = "1.0"
thiserror = "1.0"
axum = { version = "0.7", features = ["macros"] }
tower = "0.4"
tower-http = { version = "0.5", features = ["cors", "trace"] }
sqlx = { version = "0.8", features = ["runtime-tokio-rustls", "postgres", "uuid", "chrono", "json", "macros", "migrate"] }
reqwest = { version = "0.11", features = ["json"] }
tracing = "0.1"
tracing-subscriber = { version = "0.3", features = ["env-filter"] }
config = "0.14"
async-trait = "0.1"
CARGOEOF
        
        # Replace SERVICE_NAME
        sed -i.bak "s/SERVICE_NAME/$name/g" "$service/Cargo.standalone.toml"
        rm -f "$service/Cargo.standalone.toml.bak"
        
        # Backup original and use standalone
        mv "$service/Cargo.toml" "$service/Cargo.toml.workspace"
        mv "$service/Cargo.standalone.toml" "$service/Cargo.toml"
        
        echo "✅ Created standalone Cargo.toml for $service"
    fi
done

echo "Done!"
