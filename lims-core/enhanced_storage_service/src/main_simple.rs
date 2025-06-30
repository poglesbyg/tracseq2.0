use anyhow::Result;
use std::net::SocketAddr;
use tracing::info;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};
use axum::{Router, routing::get, Json};
use serde_json::json;

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize tracing
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "enhanced_storage_service=debug,tower_http=debug".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    info!("🏪 Starting Enhanced Storage Service - Minimal Version");

    // Get port from environment or use default
    let port = std::env::var("STORAGE_PORT")
        .unwrap_or_else(|_| "8080".to_string())
        .parse::<u16>()
        .unwrap_or(8080);

    // Create a minimal HTTP server without database initialization
    let app = Router::new()
        .route("/health", get(|| async { "OK" }))
        .route("/", get(|| async { "Enhanced Storage Service - Running (Minimal Mode)" }))
        .route("/api/storage/health", get(|| async { 
            Json(json!({
                "status": "healthy",
                "service": "storage",
                "version": "0.1.0-minimal",
                "mode": "minimal",
                "note": "Running without database initialization"
            }))
        }))
        .route("/api/storage/status", get(|| async {
            Json(json!({
                "operational": true,
                "database": "not_connected",
                "features": {
                    "ai": false,
                    "iot": false,
                    "integrations": false,
                    "blockchain": false
                }
            }))
        }));

    // Start the server
    let addr = SocketAddr::from(([0, 0, 0, 0], port));
    info!("🚀 Enhanced Storage Service (Minimal) listening on {}", addr);

    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
} 