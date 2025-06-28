// Enhanced Storage Service Library
// Provides all public APIs and functionality for testing and external use

pub mod ai;
pub mod analytics;
pub mod automation;
pub mod blockchain;
pub mod compliance;
pub mod config;
pub mod database;
pub mod digital_twin;
pub mod energy;
pub mod error;
pub mod handlers;
pub mod integrations;
pub mod iot;
pub mod middleware;
pub mod mobile;
pub mod models;
pub mod services;

use std::sync::Arc;

// Re-export main types for easy access
pub use config::Config;
pub use database::{DatabasePool, create_pool, run_migrations};
pub use error::{StorageError, StorageResult};
pub use services::EnhancedStorageService;

/// Application state shared across handlers - Phase 3 Enhanced
#[derive(Clone)]
pub struct AppState {
    pub storage_service: Arc<EnhancedStorageService>,
    pub config: Arc<Config>,
    pub db_pool: DatabasePool,
    pub ai_platform: Arc<ai::AIPlatform>, // Phase 2: AI Platform
    pub integration_hub: Arc<integrations::IntegrationHub>, // Phase 3: Enterprise Integrations
}

impl Default for AppState {
    fn default() -> Self {
        let config = Config::test_config();
        let db_pool = DatabasePool {
            pool: sqlx::PgPool::connect_lazy("postgres://localhost/test").unwrap(),
        };

        // Create a simple storage service for Default implementation
        // Note: For actual testing, use create_test_app_state() instead
        let storage_service = EnhancedStorageService {
            db: db_pool.clone(),
            config: config.clone(),
        };

        let ai_platform = ai::AIPlatform::new(ai::AIConfig::default());
        let integration_hub =
            integrations::IntegrationHub::new(integrations::IntegrationConfig::default());

        Self {
            db_pool,
            config: Arc::new(config),
            storage_service: Arc::new(storage_service),
            ai_platform: Arc::new(ai_platform),
            integration_hub: Arc::new(integration_hub),
        }
    }
}

/// Create the application router with all routes and middleware
pub fn create_app(state: AppState) -> axum::Router {
    use axum::{
        Router,
        routing::{delete, get, post, put},
    };
    use tower::ServiceBuilder;
    use tower_http::{cors::CorsLayer, trace::TraceLayer};

    // Health check routes (no auth required)
    let health_routes = Router::new()
        .route("/health", get(handlers::health::health_check))
        .route("/health/ready", get(handlers::health::readiness_check))
        .route("/health/metrics", get(handlers::health::metrics));

    // Storage management routes
    let storage_routes = Router::new()
        .route(
            "/storage/locations",
            post(handlers::storage::create_location),
        )
        .route("/storage/locations", get(handlers::storage::list_locations))
        .route(
            "/storage/locations/:location_id",
            get(handlers::storage::get_location),
        )
        .route(
            "/storage/locations/:location_id",
            put(handlers::storage::update_location),
        )
        .route(
            "/storage/locations/:location_id",
            delete(handlers::storage::delete_location),
        )
        .route(
            "/storage/locations/:location_id/capacity",
            get(handlers::storage::get_capacity),
        )
        .route("/storage/samples", post(handlers::storage::store_sample))
        .route(
            "/storage/samples/:sample_id/location",
            get(handlers::storage::get_sample_location),
        )
        .route(
            "/storage/samples/:sample_id/move",
            post(handlers::storage::move_sample),
        )
        .route(
            "/storage/samples/:sample_id/retrieve",
            post(handlers::storage::retrieve_sample),
        );

    // IoT integration routes
    let iot_routes = Router::new()
        .route("/iot/sensors", get(handlers::iot::list_sensors))
        .route("/iot/sensors", post(handlers::iot::register_sensor))
        .route("/iot/sensors/:sensor_id", get(handlers::iot::get_sensor))
        .route(
            "/iot/sensors/:sensor_id/data",
            get(handlers::iot::get_sensor_data),
        )
        .route(
            "/iot/sensors/:sensor_id/readings",
            post(handlers::iot::record_sensor_reading),
        )
        .route("/iot/alerts", get(handlers::iot::get_alerts))
        .route(
            "/iot/alerts/:alert_id/resolve",
            post(handlers::iot::resolve_alert),
        )
        .route("/iot/health", get(handlers::iot::get_sensor_health));

    // Combine all routes
    Router::new()
        .merge(health_routes)
        .merge(storage_routes)
        .merge(iot_routes)
        .layer(
            ServiceBuilder::new()
                .layer(TraceLayer::new_for_http())
                .layer(CorsLayer::permissive()),
        )
        .with_state(state)
}

// Test utilities
#[cfg(test)]
pub mod test_utils {
    use super::*;
    use sqlx::PgPool;

    pub async fn get_test_db() -> PgPool {
        let database_url = std::env::var("TEST_DATABASE_URL").unwrap_or_else(|_| {
            "postgres://postgres:postgres@localhost:5432/enhanced_storage_test".to_string()
        });

        create_pool(&database_url)
            .await
            .expect("Failed to create test database pool")
    }

    pub async fn create_test_app_state() -> AppState {
        let pg_pool = get_test_db().await;
        let db_pool = DatabasePool { pool: pg_pool };
        let config = Config::test_config();

        let storage_service = EnhancedStorageService::new(db_pool.clone(), config.clone())
            .await
            .expect("Failed to create storage service");

        let ai_platform = ai::AIPlatform::new(ai::AIConfig::default());
        let integration_hub =
            integrations::IntegrationHub::new(integrations::IntegrationConfig::default());

        AppState {
            db_pool,
            config: Arc::new(config),
            storage_service: Arc::new(storage_service),
            ai_platform: Arc::new(ai_platform),
            integration_hub: Arc::new(integration_hub),
        }
    }

    pub async fn cleanup_test_data(pool: &PgPool) {
        let _ = sqlx::query(
            "TRUNCATE TABLE samples, storage_locations, iot_sensors, sensor_data, alerts CASCADE",
        )
        .execute(pool)
        .await;
    }
}
