use axum::{
    routing::{get, post, put, delete},
    Router,
    middleware,
};
use sqlx::PgPool;
use std::sync::Arc;
use tower_http::cors::CorsLayer;
use tower_http::trace::TraceLayer;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

mod config;
mod database;
mod error;
mod models;

use config::Config;
use database::create_pool;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize tracing
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "qaqc_service=debug,tower_http=debug".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    // Load configuration
    let config = Config::from_env()?;
    
    // Create database connection pool
    let pool = create_pool(&config.database_url).await?;
    
    // Run database migrations
    database::run_migrations(&pool).await?;
    
    // Build application router
    let app = Router::new()
        .route("/health", get(health_check))
        .layer(CorsLayer::permissive())
        .layer(TraceLayer::new_for_http());

    // Start server
    let listener = tokio::net::TcpListener::bind(format!("{}:{}", config.host, config.port)).await?;
    
    tracing::info!(
        "QAQC service listening on {}:{}",
        config.host,
        config.port
    );
    
    axum::serve(listener, app).await?;
    
    Ok(())
}

async fn health_check() -> &'static str {
    "QAQC Service is healthy"
} 
