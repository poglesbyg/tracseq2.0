// Enhanced Storage Service Library
// Provides hierarchical storage functionality

pub mod config;
pub mod database;
pub mod error;
pub mod handlers;
pub mod models;
pub mod services;

use std::sync::Arc;

// Re-export main types for easy access
pub use config::Config;
pub use database::DatabasePool;
pub use error::{StorageError, StorageResult};
pub use services::EnhancedStorageService;

// Re-export handlers for easier access
pub use handlers::*;

// Re-export all models
pub use models::*;

/// Application state shared across handlers
#[derive(Clone)]
pub struct AppState {
    pub storage_service: Arc<EnhancedStorageService>,
    pub config: Arc<Config>,
    pub db_pool: DatabasePool,
}

impl AppState {
    pub async fn new(config: Config) -> Result<Self, anyhow::Error> {
        let db_pool = DatabasePool::new(&config.database_url).await?;
        let storage_service = EnhancedStorageService::new(db_pool.clone(), config.clone()).await?;

        Ok(Self {
            db_pool,
            config: Arc::new(config),
            storage_service: Arc::new(storage_service),
        })
    }

    pub fn test_config() -> Self {
        let config = Config::test_config();
        let db_pool = DatabasePool {
            pool: sqlx::PgPool::connect_lazy("postgres://postgres:postgres@localhost:5432/test_db").unwrap(),
        };

        // Create a simple storage service for test implementation
        let storage_service = EnhancedStorageService {
            db: db_pool.clone(),
            config: Arc::new(config.clone()),
        };

        Self {
            db_pool,
            config: Arc::new(config),
            storage_service: Arc::new(storage_service),
        }
    }
}

/// Create the application router with hierarchical storage routes
pub fn create_app(state: AppState) -> axum::Router {
    use axum::{
        Router,
        routing::{delete, get, post, put},
    };
    use tower::ServiceBuilder;
    use tower_http::{cors::CorsLayer, trace::TraceLayer};

    // Health check routes
    let health_routes = Router::new()
        .route("/health", get(handlers::health_check))
        .route("/health/ready", get(handlers::readiness_check))
        .route("/health/metrics", get(handlers::metrics_check));

    // Storage location routes
    let storage_routes = Router::new()
        .route("/storage/locations", post(handlers::create_storage_location))
        .route("/storage/locations", get(handlers::list_storage_locations))
        .route("/storage/locations/:location_id", get(handlers::get_storage_location))
        .route("/storage/locations/:location_id", put(handlers::update_storage_location))
        .route("/storage/locations/:location_id", delete(handlers::delete_storage_location))
        .route("/storage/locations/:location_id/capacity", get(handlers::get_location_capacity));

    // Hierarchical storage container routes
    let container_routes = Router::new()
        .route("/storage/containers", post(handlers::hierarchical_storage::create_container))
        .route("/storage/containers", get(handlers::hierarchical_storage::list_containers))
        .route("/storage/containers/:container_id", get(handlers::hierarchical_storage::get_container_with_details))
        .route("/storage/containers/:container_id/hierarchy", get(handlers::hierarchical_storage::get_storage_hierarchy))
        .route("/storage/containers/:container_id/grid", get(handlers::hierarchical_storage::get_container_grid));

    // Sample position management routes
    let sample_routes = Router::new()
        .route("/storage/samples/assign", post(handlers::hierarchical_storage::assign_sample_to_position))
        .route("/storage/samples/:sample_id/move", put(handlers::hierarchical_storage::move_sample_to_position))
        .route("/storage/samples/:sample_id/location-detailed", get(handlers::hierarchical_storage::get_sample_location_detailed))
        .route("/storage/samples/:sample_id/position", delete(handlers::hierarchical_storage::remove_sample_from_position));

    // Sample management routes
    let basic_sample_routes = Router::new()
        .route("/storage/samples", post(handlers::store_sample))
        .route("/storage/samples/:sample_id", get(handlers::get_sample))
        .route("/storage/samples/:sample_id/location", get(handlers::get_sample_location))
        .route("/storage/samples/:sample_id/retrieve", post(handlers::retrieve_sample));

    // Analytics and reporting routes
    let analytics_routes = Router::new()
        .route("/storage/capacity/summary", get(handlers::get_capacity_summary))
        .route("/storage/utilization", get(handlers::get_utilization_report))
        .route("/storage/containers/available", get(handlers::get_available_positions))
        .route("/storage/containers/by-temperature/:temperature_zone", get(handlers::get_containers_by_temperature));

    // Combine all routes
    Router::new()
        .merge(health_routes)
        .merge(storage_routes)
        .merge(container_routes)
        .merge(sample_routes)
        .merge(basic_sample_routes)
        .merge(analytics_routes)
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
            "postgres://postgres:postgres@localhost:5432/test_db".to_string()
        });

        DatabasePool::new(&database_url)
            .await
            .expect("Failed to create test database pool")
            .pool
    }

    pub async fn create_test_app_state() -> AppState {
        let config = Config::test_config();
        AppState::new(config).await.expect("Failed to create test app state")
    }

    pub async fn cleanup_test_data(pool: &PgPool) {
        let _ = sqlx::query(
            "TRUNCATE TABLE samples, storage_locations, storage_containers, sample_positions CASCADE",
        )
        .execute(pool)
        .await;
    }
}
