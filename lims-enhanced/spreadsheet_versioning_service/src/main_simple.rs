use anyhow::Result;
use axum::{
    routing::{get, post, put, delete},
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
                .unwrap_or_else(|_| "spreadsheet_versioning_service=info,tower_http=debug".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    info!("📊 Starting Spreadsheet Versioning Service");

    // Get configuration from environment
    let port = std::env::var("PORT")
        .unwrap_or_else(|_| "8088".to_string())
        .parse::<u16>()?;
    let database_url = std::env::var("DATABASE_URL")
        .unwrap_or_else(|_| "postgres://postgres:postgres@localhost:5432/lab_manager".to_string());

    info!("📋 Configuration loaded - Port: {}, Database: {}", port, database_url);

    // Setup database connection
    let pool = sqlx::postgres::PgPoolOptions::new()
        .max_connections(5)
        .connect(&database_url)
        .await?;
    info!("🗄️ Database connection established");

    // Build the application router with basic routes
    let app = Router::new()
        .route("/health", get(health_check))
        .route("/health/ready", get(readiness_check))
        .route("/api/v1/spreadsheets", get(list_spreadsheets))
        .route("/api/v1/spreadsheets", post(create_spreadsheet))
        .route("/api/v1/spreadsheets/:id", get(get_spreadsheet))
        .route("/api/v1/spreadsheets/:id", put(update_spreadsheet))
        .route("/api/v1/spreadsheets/:id", delete(delete_spreadsheet))
        .route("/api/v1/spreadsheets/:id/versions", get(list_versions))
        .route("/api/v1/spreadsheets/:id/versions", post(create_version))
        .route("/api/v1/spreadsheets/:id/preview", get(preview_spreadsheet))
        .with_state(pool);

    // Start the server
    let addr = SocketAddr::from(([0, 0, 0, 0], port));
    info!("🚀 Spreadsheet Versioning Service listening on {}", addr);

    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}

async fn health_check() -> Json<serde_json::Value> {
    Json(json!({
        "status": "healthy",
        "service": "spreadsheet-versioning-service",
        "version": "1.0.0",
        "timestamp": chrono::Utc::now()
    }))
}

async fn readiness_check() -> Result<Json<serde_json::Value>, StatusCode> {
    Ok(Json(json!({
        "status": "ready",
        "service": "spreadsheet-versioning-service",
        "timestamp": chrono::Utc::now()
    })))
}

async fn list_spreadsheets() -> Json<serde_json::Value> {
    Json(json!({
        "spreadsheets": [],
        "total": 0
    }))
}

async fn create_spreadsheet() -> Json<serde_json::Value> {
    Json(json!({
        "id": uuid::Uuid::new_v4(),
        "created": true,
        "message": "Spreadsheet created successfully"
    }))
}

async fn get_spreadsheet() -> Json<serde_json::Value> {
    Json(json!({
        "id": uuid::Uuid::new_v4(),
        "name": "Sample Spreadsheet",
        "status": "active"
    }))
}

async fn update_spreadsheet() -> Json<serde_json::Value> {
    Json(json!({
        "updated": true,
        "message": "Spreadsheet updated successfully"
    }))
}

async fn delete_spreadsheet() -> Json<serde_json::Value> {
    Json(json!({
        "deleted": true,
        "message": "Spreadsheet deleted successfully"
    }))
}

async fn list_versions() -> Json<serde_json::Value> {
    Json(json!({
        "versions": [],
        "total": 0
    }))
}

async fn create_version() -> Json<serde_json::Value> {
    Json(json!({
        "version_id": uuid::Uuid::new_v4(),
        "created": true,
        "message": "Version created successfully"
    }))
}

async fn preview_spreadsheet() -> Json<serde_json::Value> {
    Json(json!({
        "preview": {
            "rows": 0,
            "columns": 0,
            "data": []
        },
        "message": "Preview generated successfully"
    }))
} 