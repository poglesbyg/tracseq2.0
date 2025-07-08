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
use tracing::{info, error};

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
    // Initialize comprehensive tracing with debug info
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::from_default_env()
                .add_directive("spreadsheet_versioning_service=debug".parse()?)
        )
        .with_file(true)
        .with_line_number(true)
        .with_target(true)
        .init();

    info!("🚀 Starting Spreadsheet Versioning Service");
    info!("Environment variables:");
    info!("  DATABASE_URL: {}", std::env::var("DATABASE_URL").unwrap_or_else(|_| "NOT SET".to_string()));
    info!("  PORT: {}", std::env::var("PORT").unwrap_or_else(|_| "NOT SET".to_string()));
    info!("  RUST_LOG: {}", std::env::var("RUST_LOG").unwrap_or_else(|_| "NOT SET".to_string()));

    // Load configuration with detailed error handling
    info!("📋 Loading configuration...");
    let config = match Config::load() {
        Ok(config) => {
            info!("✅ Configuration loaded successfully");
            info!("  Port: {}", config.port);
            info!("  Database URL: {}", config.database_url);
            info!("  Max file size: {} MB", config.max_file_size_mb);
            Arc::new(config)
        }
        Err(e) => {
            error!("❌ Failed to load configuration: {}", e);
            return Err(e.into());
        }
    };
    
    // Initialize database with detailed error handling
    info!("🗄️ Initializing database connection...");
    let database = match Database::new(&config.database_url).await {
        Ok(db) => {
            info!("✅ Database connection established");
            Arc::new(db)
        }
        Err(e) => {
            error!("❌ Failed to connect to database: {}", e);
            error!("Database URL: {}", config.database_url);
            return Err(e.into());
        }
    };
    
    // Run migrations with detailed error handling
    info!("🔄 Running database migrations...");
    if let Err(e) = database.migrate().await {
        error!("❌ Failed to run migrations: {}", e);
        return Err(e.into());
    }
    info!("✅ Database migrations completed");
    
    // Test database connection
    info!("🔍 Testing database connection...");
    if let Err(e) = database.health_check().await {
        error!("❌ Database health check failed: {}", e);
        return Err(e.into());
    }
    info!("✅ Database health check passed");
    
    // Initialize services
    info!("⚙️ Initializing versioning service...");
    let versioning_service = Arc::new(VersioningService::new(database.clone()));
    info!("✅ Versioning service initialized");
    
    let app_state = AppState {
        config: config.clone(),
        database,
        versioning_service,
    };

    // Create router
    info!("🛣️ Creating application router...");
    let app = create_router().with_state(app_state);
    info!("✅ Router created");

    // Start server
    let addr = SocketAddr::from(([0, 0, 0, 0], config.port));
    info!("🌐 Binding to address: {}", addr);
    
    let listener = match TcpListener::bind(addr).await {
        Ok(listener) => {
            info!("✅ Successfully bound to {}", addr);
            listener
        }
        Err(e) => {
            error!("❌ Failed to bind to {}: {}", addr, e);
            return Err(e.into());
        }
    };
    
    info!("🎉 Spreadsheet Versioning Service is ready and listening on {}", addr);
    
    // Start serving with error handling
    if let Err(e) = axum::serve(listener, app).await {
        error!("❌ Server error: {}", e);
        return Err(e.into());
    }

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

async fn health_check(State(state): State<AppState>) -> Result<Json<serde_json::Value>, ServiceError> {
    info!("Health check requested");
    
    // Test database connection
    match state.database.health_check().await {
        Ok(_) => {
            info!("Health check passed");
            Ok(Json(json!({
                "service": "Spreadsheet Versioning Service",
                "status": "healthy",
                "timestamp": chrono::Utc::now(),
                "version": "1.0.0",
                "database": "connected",
                "port": state.config.port
            })))
        }
        Err(e) => {
            error!("Health check failed: {}", e);
            Err(ServiceError::DatabaseConnection(format!("Health check failed: {}", e)))
        }
    }
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
