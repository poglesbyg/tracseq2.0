use axum::{
    extract::State,
    response::Json,
    routing::{get, post, put, delete},
    Router,
};
use serde_json::json;
use std::{net::SocketAddr, sync::Arc};
use tokio::net::TcpListener;
use tower_http::{cors::CorsLayer, trace::TraceLayer};
use tracing::info;

mod config;
mod database;
mod error;
mod handlers;
mod models;
mod services;

use config::Config;
use database::Database;
use error::ServiceError;
use services::versioning::VersioningService;

#[derive(Clone)]
pub struct AppState {
    pub config: Arc<Config>,
    pub database: Arc<Database>,
    pub versioning_service: Arc<VersioningService>,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize tracing
    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .init();

    info!("🚀 Starting Spreadsheet Versioning Service");

    // Load configuration
    let config = Arc::new(Config::load()?);
    
    // Initialize database
    let database = Arc::new(Database::new(&config.database_url).await?);
    
    // Run migrations
    database.migrate().await?;
    
    // Initialize services
    let versioning_service = Arc::new(VersioningService::new(database.clone()));
    
    let app_state = AppState {
        config: config.clone(),
        database,
        versioning_service,
    };

    // Create router
    let app = create_router().with_state(app_state);

    // Start server
    let addr = SocketAddr::from(([0, 0, 0, 0], config.port));
    info!("✅ Spreadsheet Versioning Service listening on {}", addr);
    
    let listener = TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}

fn create_router() -> Router<AppState> {
    Router::new()
        // Health check
        .route("/health", get(health_check))
        .route("/health/ready", get(readiness_check))
        
        // Version management
        .route("/api/v1/versions", post(handlers::versions::create_version))
        .route("/api/v1/versions/:version_id", get(handlers::versions::get_version))
        .route("/api/v1/versions/:version_id", put(handlers::versions::update_version))
        .route("/api/v1/versions/:version_id", delete(handlers::versions::delete_version))
        
        // Spreadsheet versioning
        .route("/api/v1/spreadsheets/:spreadsheet_id/versions", get(handlers::spreadsheets::list_versions))
        .route("/api/v1/spreadsheets/:spreadsheet_id/versions", post(handlers::spreadsheets::create_version))
        .route("/api/v1/spreadsheets/:spreadsheet_id/versions/:version_id", get(handlers::spreadsheets::get_version))
        
        // Difference engine
        .route("/api/v1/diff/compare", post(handlers::diff::compare_versions))
        .route("/api/v1/diff/merge", post(handlers::diff::merge_versions))
        .route("/api/v1/diff/conflicts", post(handlers::diff::detect_conflicts))
        
        // Conflict resolution
        .route("/api/v1/conflicts", get(handlers::conflicts::list_conflicts))
        .route("/api/v1/conflicts/:conflict_id", get(handlers::conflicts::get_conflict))
        .route("/api/v1/conflicts/:conflict_id/resolve", post(handlers::conflicts::resolve_conflict))
        
        // Add middleware
        .layer(CorsLayer::permissive())
        .layer(TraceLayer::new_for_http())
}

async fn health_check() -> Json<serde_json::Value> {
    Json(json!({
        "status": "healthy",
        "service": "Spreadsheet Versioning Service",
        "version": "1.0.0",
        "timestamp": chrono::Utc::now()
    }))
}

async fn readiness_check(State(state): State<AppState>) -> Result<Json<serde_json::Value>, ServiceError> {
    // Check database connection
    let db_health = state.database.health_check().await;
    
    if db_health.is_ok() {
        Ok(Json(json!({
            "status": "ready",
            "service": "Spreadsheet Versioning Service",
            "database": "connected",
            "timestamp": chrono::Utc::now()
        })))
    } else {
        Err(ServiceError::DatabaseConnection("Health check failed".to_string()))
    }
} 
