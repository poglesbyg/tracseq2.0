use axum::{Router, routing::get, Json};
use serde_json::json;
use std::net::SocketAddr;
use tracing::info;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

#[tokio::main]
async fn main() {
    // Initialize tracing
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "reports_service=debug,tower_http=debug".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    info!("📊 Starting Reports Service - Minimal Version");

    // Get port from environment or use default
    let port = std::env::var("REPORTS_PORT")
        .unwrap_or_else(|_| "8000".to_string())
        .parse::<u16>()
        .unwrap_or(8000);

    // Create minimal HTTP server
    let app = Router::new()
        .route("/health", get(|| async { 
            Json(json!({
                "status": "healthy",
                "service": "reports-service",
                "version": "0.1.0-minimal"
            }))
        }))
        .route("/", get(|| async { "Reports Service - Running (Minimal Mode)" }))
        .route("/api/reports/health", get(|| async { 
            Json(json!({
                "status": "healthy",
                "service": "reports",
                "version": "0.1.0-minimal",
                "mode": "minimal",
                "note": "Running without full reporting capabilities"
            }))
        }))
        .route("/api/reports/templates", get(|| async {
            Json(json!({
                "templates": [
                    {
                        "id": "RPT-001",
                        "name": "Sample Summary Report",
                        "description": "Summary of all samples",
                        "category": "samples"
                    },
                    {
                        "id": "RPT-002", 
                        "name": "Storage Report",
                        "description": "Storage utilization report",
                        "category": "storage"
                    }
                ],
                "totalCount": 2
            }))
        }))
        .route("/api/reports/status", get(|| async {
            Json(json!({
                "operational": true,
                "features": {
                    "pdf_generation": false,
                    "excel_export": false,
                    "scheduling": false
                }
            }))
        }));

    // Start the server
    let addr = SocketAddr::from(([0, 0, 0, 0], port));
    info!("🚀 Reports Service (Minimal) listening on {}", addr);

    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
} 