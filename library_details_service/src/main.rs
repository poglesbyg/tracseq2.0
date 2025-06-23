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
mod services;
mod handlers;
mod middleware as custom_middleware;

use config::Config;
use database::create_pool;
use handlers::{libraries, protocols, kits, platforms, quality_control};
use services::{LibraryService, ProtocolService, QualityControlService};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize tracing
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "library_details_service=debug,tower_http=debug".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    // Load configuration
    let config = Config::from_env()?;
    
    // Create database connection pool
    let pool = create_pool(&config.database_url).await?;
    
    // Run database migrations
    database::run_migrations(&pool).await?;
    
    // Initialize services
    let library_service = Arc::new(LibraryService::new(pool.clone()));
    let protocol_service = Arc::new(ProtocolService::new(pool.clone()));
    let qc_service = Arc::new(QualityControlService::new(pool.clone()));
    
    // Build application router
    let app = Router::new()
        .route("/health", get(health_check))
        
        // Library endpoints
        .route("/api/v1/libraries", get(libraries::list_libraries))
        .route("/api/v1/libraries", post(libraries::create_library))
        .route("/api/v1/libraries/:id", get(libraries::get_library))
        .route("/api/v1/libraries/:id", put(libraries::update_library))
        .route("/api/v1/libraries/:id", delete(libraries::delete_library))
        .route("/api/v1/libraries/:id/calculate", post(libraries::calculate_library_metrics))
        .route("/api/v1/libraries/:id/normalize", post(libraries::normalize_library))
        .route("/api/v1/libraries/batch", post(libraries::create_batch_libraries))
        
        // Protocol endpoints
        .route("/api/v1/protocols", get(protocols::list_protocols))
        .route("/api/v1/protocols", post(protocols::create_protocol))
        .route("/api/v1/protocols/:id", get(protocols::get_protocol))
        .route("/api/v1/protocols/:id", put(protocols::update_protocol))
        .route("/api/v1/protocols/:id/validate", post(protocols::validate_protocol))
        .route("/api/v1/protocols/:id/steps", get(protocols::get_protocol_steps))
        
        // Kit endpoints
        .route("/api/v1/kits", get(kits::list_kits))
        .route("/api/v1/kits", post(kits::create_kit))
        .route("/api/v1/kits/:id", get(kits::get_kit))
        .route("/api/v1/kits/:id/compatibility", get(kits::get_kit_compatibility))
        .route("/api/v1/kits/search", get(kits::search_kits))
        
        // Platform endpoints
        .route("/api/v1/platforms", get(platforms::list_platforms))
        .route("/api/v1/platforms", post(platforms::create_platform))
        .route("/api/v1/platforms/:id", get(platforms::get_platform))
        .route("/api/v1/platforms/:id/configurations", get(platforms::get_platform_configurations))
        
        // Quality Control endpoints
        .route("/api/v1/qc/metrics", get(quality_control::list_qc_metrics))
        .route("/api/v1/qc/metrics", post(quality_control::create_qc_metric))
        .route("/api/v1/qc/libraries/:id/assess", post(quality_control::assess_library_quality))
        .route("/api/v1/qc/thresholds", get(quality_control::get_quality_thresholds))
        .route("/api/v1/qc/reports", get(quality_control::generate_quality_report))
        
        // Integration endpoints
        .route("/api/v1/integration/sample/:sample_id/libraries", get(libraries::get_libraries_for_sample))
        .route("/api/v1/integration/sequencing/:job_id/libraries", get(libraries::get_libraries_for_sequencing_job))
        .route("/api/v1/integration/protocols/recommend", post(protocols::recommend_protocol))
        
        // Service state management
        .with_state(library_service.clone())
        .with_state(protocol_service.clone())
        .with_state(qc_service.clone())
        
        // Middleware
        .layer(middleware::from_fn(custom_middleware::auth_middleware))
        .layer(CorsLayer::permissive())
        .layer(TraceLayer::new_for_http());

    // Start server
    let listener = tokio::net::TcpListener::bind(format!("{}:{}", config.host, config.port)).await?;
    
    tracing::info!(
        "Library Details service listening on {}:{}",
        config.host,
        config.port
    );
    
    axum::serve(listener, app).await?;
    
    Ok(())
}

async fn health_check() -> &'static str {
    "Library Details Service is healthy"
} 
