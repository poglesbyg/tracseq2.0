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

    // Storage management routes (existing)
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
            "/storage/samples/:sample_id",
            get(handlers::storage::get_sample),
        )
        .route(
            "/storage/samples/:sample_id/location",
            get(handlers::storage::get_sample_location),
        )
        .route(
            "/storage/samples/:sample_id/move",
            post(handlers::storage::move_sample),
        )
        .route(
            "/storage/samples/:sample_id/move",
            put(handlers::storage::move_sample),
        )
        .route(
            "/storage/samples/:sample_id/retrieve",
            post(handlers::storage::retrieve_sample),
        );

    // Hierarchical storage routes (new)
    let hierarchical_storage_routes = Router::new()
        // Container management
        .route(
            "/storage/containers",
            post(handlers::hierarchical_storage::create_container),
        )
        .route(
            "/storage/containers",
            get(handlers::hierarchical_storage::list_containers),
        )
        .route(
            "/storage/containers/:container_id",
            get(handlers::hierarchical_storage::get_container_with_details),
        )
        .route(
            "/storage/containers/:container_id/hierarchy",
            get(handlers::hierarchical_storage::get_storage_hierarchy),
        )
        .route(
            "/storage/containers/:container_id/grid",
            get(handlers::hierarchical_storage::get_container_grid),
        )
        // Sample position management
        .route(
            "/storage/samples/assign",
            post(handlers::hierarchical_storage::assign_sample_to_position),
        )
        .route(
            "/storage/samples/:sample_id/move",
            put(handlers::hierarchical_storage::move_sample_to_position),
        )
        .route(
            "/storage/samples/:sample_id/location-detailed",
            get(handlers::hierarchical_storage::get_sample_location_detailed),
        )
        .route(
            "/storage/samples/:sample_id/position",
            delete(handlers::hierarchical_storage::remove_sample_from_position),
        );

    // IoT and sensor routes
    let iot_routes = Router::new()
        .route("/iot/sensors", post(handlers::iot::create_sensor))
        .route("/iot/sensors", get(handlers::iot::list_sensors))
        .route("/iot/sensors/:sensor_id", get(handlers::iot::get_sensor))
        .route("/iot/sensors/:sensor_id", put(handlers::iot::update_sensor))
        .route(
            "/iot/sensors/:sensor_id/readings",
            post(handlers::iot::record_reading),
        )
        .route(
            "/iot/sensors/:sensor_id/readings",
            get(handlers::iot::get_readings),
        )
        .route("/iot/readings/batch", post(handlers::iot::batch_readings))
        .route("/iot/alerts", get(handlers::iot::get_alerts))
        .route("/iot/alerts/:alert_id/acknowledge", post(handlers::iot::acknowledge_alert));

    // Analytics routes
    let analytics_routes = Router::new()
        .route("/analytics/capacity", get(handlers::analytics::capacity_analytics))
        .route("/analytics/utilization", get(handlers::analytics::utilization_trends))
        .route("/analytics/predictions", get(handlers::analytics::get_predictions))
        .route("/analytics/models", get(handlers::analytics::list_models))
        .route("/analytics/models", post(handlers::analytics::create_model))
        .route("/analytics/models/:model_id", get(handlers::analytics::get_model))
        .route("/analytics/models/:model_id/train", post(handlers::analytics::train_model))
        .route("/analytics/models/:model_id/predict", post(handlers::analytics::make_prediction));

    // Admin routes
    let admin_routes = Router::new()
        .route("/admin/system/status", get(handlers::admin::system_status))
        .route("/admin/system/config", get(handlers::admin::get_config))
        .route("/admin/system/config", put(handlers::admin::update_config))
        .route("/admin/maintenance", post(handlers::admin::schedule_maintenance))
        .route("/admin/maintenance", get(handlers::admin::list_maintenance))
        .route("/admin/users", get(handlers::admin::list_users))
        .route("/admin/users/:user_id/permissions", put(handlers::admin::update_permissions))
        .route("/admin/audit/logs", get(handlers::admin::get_audit_logs))
        .route("/admin/backup", post(handlers::admin::create_backup))
        .route("/admin/backup", get(handlers::admin::list_backups))
        .route("/admin/backup/:backup_id/restore", post(handlers::admin::restore_backup));

    // Automation routes
    let automation_routes = Router::new()
        .route("/automation/tasks", post(handlers::automation::create_task))
        .route("/automation/tasks", get(handlers::automation::list_tasks))
        .route("/automation/tasks/:task_id", get(handlers::automation::get_task))
        .route("/automation/tasks/:task_id/status", put(handlers::automation::update_task_status))
        .route("/automation/robots", get(handlers::automation::list_robots))
        .route("/automation/robots/:robot_id", get(handlers::automation::get_robot))
        .route("/automation/robots/:robot_id/status", put(handlers::automation::update_robot_status))
        .route("/automation/schedule", get(handlers::automation::get_schedule))
        .route("/automation/schedule", post(handlers::automation::schedule_task));

    // Blockchain routes
    let blockchain_routes = Router::new()
        .route("/blockchain/transactions", post(handlers::blockchain::create_transaction))
        .route("/blockchain/transactions", get(handlers::blockchain::list_transactions))
        .route("/blockchain/transactions/:tx_hash", get(handlers::blockchain::get_transaction))
        .route("/blockchain/verify/:tx_hash", get(handlers::blockchain::verify_transaction))
        .route("/blockchain/chain", get(handlers::blockchain::get_chain))
        .route("/blockchain/integrity", get(handlers::blockchain::verify_integrity));

    // Digital twin routes
    let digital_twin_routes = Router::new()
        .route("/digital-twin/models", post(handlers::digital_twin::create_model))
        .route("/digital-twin/models", get(handlers::digital_twin::list_models))
        .route("/digital-twin/models/:model_id", get(handlers::digital_twin::get_model))
        .route("/digital-twin/models/:model_id/simulate", post(handlers::digital_twin::run_simulation))
        .route("/digital-twin/simulations", get(handlers::digital_twin::list_simulations))
        .route("/digital-twin/simulations/:sim_id", get(handlers::digital_twin::get_simulation))
        .route("/digital-twin/sync/:entity_id", post(handlers::digital_twin::sync_entity));

    // Energy management routes
    let energy_routes = Router::new()
        .route("/energy/consumption", get(handlers::energy::get_consumption))
        .route("/energy/consumption", post(handlers::energy::record_consumption))
        .route("/energy/optimization", get(handlers::energy::get_optimization_suggestions))
        .route("/energy/reports", get(handlers::energy::generate_reports))
        .route("/energy/efficiency", get(handlers::energy::efficiency_metrics))
        .route("/energy/costs", get(handlers::energy::cost_analysis));

    // Mobile integration routes
    let mobile_routes = Router::new()
        .route("/mobile/tasks", get(handlers::mobile::get_tasks))
        .route("/mobile/tasks/:task_id/complete", post(handlers::mobile::complete_task))
        .route("/mobile/scan", post(handlers::mobile::process_barcode_scan))
        .route("/mobile/samples/nearby", get(handlers::mobile::get_nearby_samples))
        .route("/mobile/locations/navigate", post(handlers::mobile::navigate_to_location))
        .route("/mobile/sync", post(handlers::mobile::sync_data));

    // Compliance routes
    let compliance_routes = Router::new()
        .route("/compliance/events", post(handlers::compliance::create_event))
        .route("/compliance/events", get(handlers::compliance::list_events))
        .route("/compliance/status", get(handlers::compliance::get_status))
        .route("/compliance/audit", get(handlers::compliance::generate_audit_report))
        .route("/compliance/violations", get(handlers::compliance::list_violations))
        .route("/compliance/violations/:violation_id/remediate", post(handlers::compliance::remediate_violation));

    // Combine all routes
    Router::new()
        .merge(health_routes)
        .merge(storage_routes)
        .merge(hierarchical_storage_routes)
        .merge(iot_routes)
        .merge(analytics_routes)
        .merge(admin_routes)
        .merge(automation_routes)
        .merge(blockchain_routes)
        .merge(digital_twin_routes)
        .merge(energy_routes)
        .merge(mobile_routes)
        .merge(compliance_routes)
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
