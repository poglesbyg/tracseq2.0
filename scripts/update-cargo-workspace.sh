#!/bin/bash

# Update Cargo.toml workspace members after reorganization

set -e

echo "📝 Updating Cargo.toml workspace members..."

# Create a backup
cp Cargo.toml Cargo.toml.bak

# Create the new Cargo.toml with updated paths
cat > Cargo.toml << 'EOF'
[workspace]
resolver = "2"
members = [
    # Core services
    "lims-core/auth_service",
    "lims-core/barcode_service", 
    "lims-core/project_service",
    "lims-core/reports_service",
    "lims-core/sample_service",
    "lims-core/template_service",
    "lims-core/transaction_service",
    "lims-core/dashboard_service",
    "lims-core/config-service",
    "lims-core/circuit-breaker-lib",
    "lims-core/mcp-bridge",
    
    # Enhanced services
    "lims-enhanced/enhanced_storage_service",
    "lims-enhanced/cognitive_assistant_service",
    "lims-enhanced/event_service",
    "lims-enhanced/notification_service",
    "lims-enhanced/spreadsheet_versioning_service",
    "lims-enhanced/saga_orchestrator",
    "lims-enhanced/saga_orchestrator/orchestrator",
    
    # Laboratory services
    "lims-laboratory/lab_manager",
    "lims-laboratory/library_prep_service",
    "lims-laboratory/library_details_service",
    "lims-laboratory/sequencing_service",
    "lims-laboratory/qaqc_service",
    "lims-laboratory/flow_cell_service",
    
    # Test helpers
    "test-helpers",
    "temp_auth_test",
    "kafka"
]

[workspace.package]
version = "0.1.0"
authors = ["TracSeq Team"]
edition = "2021"
rust-version = "1.90"

[workspace.dependencies]
# Async runtime
tokio = { version = "1.42", features = ["full"] }
async-trait = "0.1"

# Web frameworks
axum = { version = "0.7", features = ["multipart", "ws", "macros"] }
actix-web = "4.4"
actix-cors = "0.6"

# Database
sqlx = { version = "0.8", features = ["runtime-tokio-rustls", "postgres", "uuid", "time", "json", "macros"] }
sea-orm = { version = "1.1", features = ["sqlx-postgres", "runtime-tokio-rustls", "macros"] }

# Serialization
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"

# Other common dependencies
uuid = { version = "1.11", features = ["v4", "serde"] }
chrono = { version = "0.4", features = ["serde"] }
tracing = "0.1"
tracing-subscriber = { version = "0.3", features = ["env-filter", "json"] }
tower = { version = "0.5", features = ["full"] }
tower-http = { version = "0.6", features = ["cors", "trace"] }
thiserror = "2.0"
anyhow = "1.0"
dotenv = "0.15"
reqwest = { version = "0.12", features = ["json", "rustls-tls"] }
jsonwebtoken = "9"
argon2 = "0.5"
redis = { version = "0.27", features = ["aio", "tokio-comp"] }
lapin = "2.3"
deadpool-lapin = "0.12"
kafka = { version = "0.10", features = [] }

[profile.dev]
opt-level = 1

[profile.release]
lto = true
codegen-units = 1
EOF

echo "✅ Cargo.toml updated successfully!"
echo ""
echo "🔍 Verifying workspace configuration..."
cargo metadata --no-deps > /dev/null 2>&1 && echo "✅ Workspace configuration is valid!" || echo "❌ Workspace configuration has errors!" 