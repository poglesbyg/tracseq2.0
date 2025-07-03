use anyhow::Result;
use axum::{
    routing::{get, post},
    Router,
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
                .unwrap_or_else(|_| "reports_service=info,tower_http=debug".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    info!("📊 Starting Reports Service");

    // Get configuration from environment
    let port = std::env::var("PORT")
        .unwrap_or_else(|_| "8000".to_string())
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

    // Run migrations
    // Commented out for standalone build - migrations should be run separately
    // sqlx::migrate!("./migrations").run(&pool).await?;
    info!("📊 Skipping database migrations in standalone mode");

    // Build the application router with basic routes
    let app = Router::new()
        .route("/health", get(health_check))
        .route("/api/reports", get(list_reports))
        .route("/api/reports/generate", post(generate_report))
        .with_state(pool);

    // Start the server
    let addr = SocketAddr::from(([0, 0, 0, 0], port));
    info!("🚀 Reports Service listening on {}", addr);

    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}

async fn health_check() -> Json<serde_json::Value> {
    Json(json!({
        "status": "healthy",
        "service": "reports-service",
        "timestamp": chrono::Utc::now()
    }))
}

async fn list_reports() -> Json<serde_json::Value> {
    Json(json!({
        "reports": []
    }))
}

async fn generate_report() -> Json<serde_json::Value> {
    Json(json!({
        "id": uuid::Uuid::new_v4(),
        "status": "generating"
    }))
} 