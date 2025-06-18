use anyhow::Result;
use axum::{
    middleware,
    routing::{get, post, put, delete},
    Router,
};
use std::net::SocketAddr;
use tower::ServiceBuilder;
use tower_http::{
    cors::CorsLayer,
    trace::TraceLayer,
    compression::CompressionLayer,
};
use tracing::{info, warn};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

mod config;
mod database;
mod error;
mod handlers;
mod models;
mod services;
mod middleware as storage_middleware;
mod iot;
mod analytics;
mod digital_twin;
mod blockchain;
mod automation;
mod energy;
mod mobile;
mod compliance;

use config::Config;
use database::DatabasePool;
use services::EnhancedStorageService;

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize tracing
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "enhanced_storage_service=debug,tower_http=debug".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    info!("🏪 Starting Enhanced Storage Service");

    // Load configuration
    let config = Config::from_env()?;
    info!("📋 Configuration loaded successfully");

    // Setup database connection
    let db_pool = DatabasePool::new(&config.database_url).await?;
    info!("🗄️ Database connection established");

    // Run database migrations
    db_pool.migrate().await?;
    info!("📊 Database migrations completed");

    // Initialize enhanced storage service
    let storage_service = EnhancedStorageService::new(
        db_pool.clone(),
        config.clone(),
    ).await?;
    info!("🏪 Enhanced storage service initialized");

    // Setup application state
    let app_state = AppState {
        storage_service,
        config: config.clone(),
        db_pool,
    };

    // Build the application router
    let app = create_app(app_state);

    // Start the server
    let addr = SocketAddr::from(([0, 0, 0, 0], config.server.port));
    info!("🚀 Enhanced Storage Service listening on {}", addr);

    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}

/// Application state shared across handlers
#[derive(Clone)]
pub struct AppState {
    pub storage_service: EnhancedStorageService,
    pub config: Config,
    pub db_pool: DatabasePool,
}

/// Create the application router with all routes and middleware
fn create_app(state: AppState) -> Router {
    // Health check routes (no auth required)
    let health_routes = Router::new()
        .route("/health", get(handlers::health::health_check))
        .route("/health/ready", get(handlers::health::readiness_check))
        .route("/health/metrics", get(handlers::health::metrics));

    // Storage management routes
    let storage_routes = Router::new()
        .route("/storage/locations", post(handlers::storage::create_location))
        .route("/storage/locations", get(handlers::storage::list_locations))
        .route("/storage/locations/:location_id", get(handlers::storage::get_location))
        .route("/storage/samples", post(handlers::storage::store_sample))
        .route("/storage/samples/:sample_id/location", get(handlers::storage::get_sample_location));

    // IoT integration routes
    let iot_routes = Router::new()
        .route("/iot/sensors", get(handlers::iot::list_sensors))
        .route("/iot/sensors/:sensor_id/data", get(handlers::iot::get_sensor_data))
        .route("/iot/alerts", get(handlers::iot::get_alerts));

    // Analytics routes
    let analytics_routes = Router::new()
        .route("/analytics/capacity/prediction", get(handlers::analytics::predict_capacity))
        .route("/analytics/maintenance/schedule", get(handlers::analytics::predict_maintenance))
        .route("/analytics/energy/optimization", get(handlers::analytics::optimize_energy));

    // Combine all routes
    Router::new()
        .merge(health_routes)
        .merge(storage_routes)
        .merge(iot_routes)
        .merge(analytics_routes)
        .layer(
            ServiceBuilder::new()
                .layer(TraceLayer::new_for_http())
                .layer(CompressionLayer::new())
                .layer(CorsLayer::permissive())
        )
        .with_state(state)
}
