// Enhanced Storage Service Library
// Provides all public APIs and functionality for testing and external use

pub mod config;
pub mod database;
pub mod error;
pub mod handlers;
pub mod models;
pub mod services;
pub mod middleware;
pub mod iot;
pub mod analytics;
pub mod digital_twin;
pub mod blockchain;
pub mod automation;
pub mod energy;
pub mod mobile;
pub mod compliance;
pub mod ai;
pub mod integrations;

// Re-export main types for easy access
pub use config::Config;
pub use database::DatabasePool;
pub use error::{StorageError, StorageResult};
pub use services::EnhancedStorageService;

/// Application state shared across handlers - Phase 3 Enhanced
#[derive(Clone)]
pub struct AppState {
    pub storage_service: EnhancedStorageService,
    pub config: Config,
    pub db_pool: DatabasePool,
    pub ai_platform: std::sync::Arc<ai::AIPlatform>, // Phase 2: AI Platform
    pub integration_hub: std::sync::Arc<integrations::IntegrationHub>, // Phase 3: Enterprise Integrations
}

/// Create the application router with all routes and middleware
pub fn create_app(state: AppState) -> axum::Router {
    use axum::{
        routing::{get, post, put, delete},
        Router,
    };
    use tower::ServiceBuilder;
    use tower_http::{
        cors::CorsLayer,
        trace::TraceLayer,
    };

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
        .route("/storage/locations/:location_id", put(handlers::storage::update_location))
        .route("/storage/locations/:location_id", delete(handlers::storage::delete_location))
        .route("/storage/locations/:location_id/capacity", get(handlers::storage::get_capacity))
        .route("/storage/samples", post(handlers::storage::store_sample))
        .route("/storage/samples/:sample_id/location", get(handlers::storage::get_sample_location))
        .route("/storage/samples/:sample_id/move", post(handlers::storage::move_sample))
        .route("/storage/samples/:sample_id/retrieve", post(handlers::storage::retrieve_sample));

    // IoT integration routes
    let iot_routes = Router::new()
        .route("/iot/sensors", get(handlers::iot::list_sensors))
        .route("/iot/sensors", post(handlers::iot::register_sensor))
        .route("/iot/sensors/:sensor_id", get(handlers::iot::get_sensor))
        .route("/iot/sensors/:sensor_id/data", get(handlers::iot::get_sensor_data))
        .route("/iot/sensors/:sensor_id/readings", post(handlers::iot::record_sensor_reading))
        .route("/iot/alerts", get(handlers::iot::get_alerts))
        .route("/iot/alerts/:alert_id/resolve", post(handlers::iot::resolve_alert))
        .route("/iot/health", get(handlers::iot::get_sensor_health));

    // Combine all routes
    Router::new()
        .merge(health_routes)
        .merge(storage_routes)
        .merge(iot_routes)
        .layer(
            ServiceBuilder::new()
                .layer(TraceLayer::new_for_http())
                .layer(CorsLayer::permissive())
        )
        .with_state(state)
} 
