use axum::{
    routing::get,
    Router,
    Json,
};
use serde_json::json;
use std::net::SocketAddr;
use tokio::net::TcpListener;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🏪 Starting Enhanced Storage Service (Minimal)");

    let port = std::env::var("PORT")
        .unwrap_or_else(|_| "8082".to_string())
        .parse::<u16>()
        .unwrap_or(8082);

    println!("📋 Port: {}", port);

    let app = Router::new()
        .route("/health", get(health_check))
        .route("/health/ready", get(health_check))
        .route("/api/storage/locations", get(list_locations));

    let addr = SocketAddr::from(([0, 0, 0, 0], port));
    println!("🚀 Enhanced Storage Service listening on {}", addr);

    let listener = TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}

async fn health_check() -> Json<serde_json::Value> {
    Json(json!({
        "status": "healthy",
        "service": "enhanced-storage-service",
        "version": "1.0.0",
        "timestamp": chrono::Utc::now()
    }))
}

async fn list_locations() -> Json<serde_json::Value> {
    Json(json!({
        "locations": [
            {
                "id": "loc-001",
                "name": "Freezer A1",
                "temperature": -80,
                "capacity": 100,
                "used": 25
            }
        ],
        "total": 1
    }))
} 