use anyhow::Result;
use axum::{
    routing::{get, post},
    Router,
};
use tower_http::{cors::CorsLayer, trace::TraceLayer};
use tracing::{info, warn};

mod config;
mod handlers;
mod models;
mod service;
mod database;
mod error;

use config::Config;
use database::DatabasePool;

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize tracing
    tracing_subscriber::fmt::init();
    
    // Load configuration
    dotenvy::dotenv().ok();
    let config = Config::from_env()?;
    
    info!("🏷️  Starting TracSeq Barcode Service v{}", env!("CARGO_PKG_VERSION"));
    info!("🔧 Environment: {}", config.environment);
    info!("🌐 Server: {}:{}", config.server.host, config.server.port);
    
    // Initialize database
    let db_pool = database::new(&config.database_url).await?;
    
    // Run database migrations
    database::run_migrations(&db_pool).await?;
    
    // Create barcode service
    let barcode_service = service::BarcodeService::new(db_pool.clone(), config.barcode.clone()).await?;
    
    // Build router
    let app = Router::new()
        .route("/health", get(handlers::health_check))
        .route("/api/v1/barcodes/generate", post(handlers::generate_barcode))
        .route("/api/v1/barcodes/validate", post(handlers::validate_barcode))
        .route("/api/v1/barcodes/parse", post(handlers::parse_barcode))
        .route("/api/v1/barcodes/reserve", post(handlers::reserve_barcode))
        .route("/api/v1/barcodes/release", post(handlers::release_barcode))
        .route("/api/v1/barcodes/check", post(handlers::check_barcode_unique))
        .route("/api/v1/barcodes/stats", get(handlers::get_stats))
        .with_state(barcode_service)
        .layer(TraceLayer::new_for_http())
        .layer(CorsLayer::permissive());
    
    // Start server
    let listener = tokio::net::TcpListener::bind(format!("{}:{}", config.server.host, config.server.port))
        .await?;
    
    info!("✅ Barcode Service ready at http://{}:{}", config.server.host, config.server.port);
    
    axum::serve(listener, app).await?;
    
    Ok(())
} 