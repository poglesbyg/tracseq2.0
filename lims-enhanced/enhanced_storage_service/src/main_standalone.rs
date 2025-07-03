use anyhow::Result;
use axum::{
    routing::{get, post},
    Router,
    http::StatusCode,
    Json,
};
use serde_json::json;
use std::net::SocketAddr;
use tracing::info;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize tracing
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "enhanced_storage_service=info,tower_http=debug".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    info!("🏪 Starting Enhanced Storage Service");

    // Get configuration from environment
    let port = std::env::var("PORT")
        .unwrap_or_else(|_| "8082".to_string())
        .parse::<u16>()?;
    let database_url = std::env::var("DATABASE_URL")
        .or_else(|_| std::env::var("STORAGE_DATABASE_URL"))
        .unwrap_or_else(|_| "postgres://postgres:postgres@localhost:5432/lab_manager".to_string());

    info!("📋 Configuration loaded - Port: {}, Database: {}", port, database_url);

    // Setup database connection
    let pool = sqlx::postgres::PgPoolOptions::new()
        .max_connections(5)
        .connect(&database_url)
        .await?;
    info!("🗄️ Database connection established");

    // Run migrations
    // Commented out for standalone build - migrations should be run separately
    // sqlx::migrate!("./migrations").run(&pool).await?;
    info!("📊 Skipping database migrations in standalone mode");

    // Build the application router with basic routes
    let app = Router::new()
        .route("/health", get(health_check))
        .route("/health/ready", get(readiness_check))
        .route("/api/storage/locations", get(list_locations))
        .route("/api/storage/locations", post(create_location))
        .with_state(pool);

    // Start the server
    let addr = SocketAddr::from(([0, 0, 0, 0], port));
    info!("🚀 Enhanced Storage Service listening on {}", addr);

    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}

async fn health_check() -> Json<serde_json::Value> {
    Json(json!({
        "status": "healthy",
        "service": "enhanced-storage-service",
        "timestamp": chrono::Utc::now()
    }))
}

async fn readiness_check() -> Result<Json<serde_json::Value>, StatusCode> {
    Ok(Json(json!({
        "status": "ready",
        "service": "enhanced-storage-service",
        "timestamp": chrono::Utc::now()
    })))
}

async fn list_locations() -> Json<serde_json::Value> {
    Json(json!({
        "locations": []
    }))
}

async fn create_location() -> Json<serde_json::Value> {
    Json(json!({
        "id": uuid::Uuid::new_v4(),
        "created": true
    }))
} 