use anyhow::Result;
use axum::{
    routing::{get, post, put, delete},
    Router,
};
use std::{net::SocketAddr, sync::Arc};
use tower::ServiceBuilder;
use tower_http::{
    cors::CorsLayer,
    trace::TraceLayer,
};
use tracing::{info, warn};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

mod config;
mod database;
mod error;
mod handlers;
mod models;
mod services;
mod middleware;
mod iot;
mod analytics;
mod digital_twin;
mod blockchain;
mod automation;
mod energy;
mod mobile;
mod compliance;
mod ai; // Phase 2: AI/ML Platform
mod integrations; // Phase 3: Enterprise Integrations

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

    info!("🏪 Starting Enhanced Storage Service - Phase 3");

    // Load configuration
    let config = Config::from_env()?;
    info!("📋 Configuration loaded successfully");

    // Setup database connection
    let db_pool = DatabasePool::new(&config.database_url).await?;
    info!("🗄️ Database connection established");

    // Run database migrations
    db_pool.migrate().await?;
    info!("📊 Database migrations completed");

    // Initialize AI platform
    let ai_platform = ai::initialize_ai_platform().await?;
    info!("🤖 AI/ML Platform initialized");

    // Initialize enterprise integrations
    let integration_config = integrations::IntegrationConfig::default();
    let integration_hub = integrations::initialize_integration_hub(integration_config).await?;
    info!("🏢 Enterprise Integration Hub initialized");

    // Initialize enhanced storage service
    let storage_service = EnhancedStorageService::new(
        db_pool.clone(),
        config.clone(),
    ).await?;
    info!("🏪 Enhanced storage service initialized");

    // Setup application state with Phase 3 capabilities
    let app_state = AppState {
        storage_service,
        config: config.clone(),
        db_pool,
        ai_platform: Arc::new(ai_platform),
        integration_hub: Arc::new(integration_hub),
    };

    // Build the application router
    let app = create_app(app_state);

    // Start the server
    let addr = SocketAddr::from(([0, 0, 0, 0], config.server.port));
    info!("🚀 Enhanced Storage Service Phase 3 listening on {}", addr);

    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}

/// Application state shared across handlers - Phase 3 Enhanced
#[derive(Clone)]
pub struct AppState {
    pub storage_service: EnhancedStorageService,
    pub config: Config,
    pub db_pool: DatabasePool,
    pub ai_platform: Arc<ai::AIPlatform>, // Phase 2: AI Platform
    pub integration_hub: Arc<integrations::IntegrationHub>, // Phase 3: Enterprise Integrations
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

    // Analytics routes
    let analytics_routes = Router::new()
        .route("/analytics/capacity/prediction", get(handlers::analytics::predict_capacity))
        .route("/analytics/maintenance/schedule", get(handlers::analytics::predict_maintenance))
        .route("/analytics/energy/optimization", get(handlers::analytics::optimize_energy))
        .route("/analytics/utilization", get(handlers::analytics::get_utilization_analytics))
        .route("/analytics/sample-flow", get(handlers::analytics::get_sample_flow_analytics))
        .route("/analytics/cost-analysis", get(handlers::analytics::get_cost_analysis))
        .route("/analytics/reports", post(handlers::analytics::generate_analytics_report));

    // Admin routes
    let admin_routes = Router::new()
        .route("/admin/system/status", get(handlers::admin::get_system_status))
        .route("/admin/maintenance/force", post(handlers::admin::force_maintenance))
        .route("/admin/analytics/reset", post(handlers::admin::reset_analytics))
        .route("/admin/blockchain/validate", post(handlers::admin::validate_blockchain))
        .route("/admin/config", put(handlers::admin::update_config))
        .route("/admin/logs", get(handlers::admin::get_system_logs))
        .route("/admin/export", post(handlers::admin::export_system_data))
        .route("/admin/metrics", get(handlers::admin::get_performance_metrics))
        .route("/admin/backup", post(handlers::admin::backup_system))
        .route("/admin/cleanup", post(handlers::admin::cleanup_old_data));

    // Automation routes
    let automation_routes = Router::new()
        .route("/automation/samples/:sample_id/place", post(handlers::automation::automated_placement))
        .route("/automation/samples/:sample_id/retrieve", post(handlers::automation::automated_retrieval))
        .route("/automation/robots", get(handlers::automation::list_robots))
        .route("/automation/robots/:robot_id/status", get(handlers::automation::get_robot_status))
        .route("/automation/robots/:robot_id/commands", post(handlers::automation::send_robot_command))
        .route("/automation/tasks/schedule", post(handlers::automation::schedule_task))
        .route("/automation/jobs", get(handlers::automation::list_jobs))
        .route("/automation/jobs/:job_id", get(handlers::automation::get_job_status))
        .route("/automation/jobs/:job_id/cancel", post(handlers::automation::cancel_job))
        .route("/automation/workflows", get(handlers::automation::list_workflows))
        .route("/automation/workflows", post(handlers::automation::create_workflow))
        .route("/automation/workflows/:workflow_id/execute", post(handlers::automation::execute_workflow))
        .route("/automation/analytics", get(handlers::automation::get_automation_analytics))
        .route("/automation/maintenance", get(handlers::automation::get_maintenance_schedule))
        .route("/automation/maintenance", post(handlers::automation::schedule_maintenance));

    // Blockchain routes
    let blockchain_routes = Router::new()
        .route("/blockchain/integrity", get(handlers::blockchain::verify_integrity))
        .route("/blockchain/transactions", get(handlers::blockchain::list_transactions))
        .route("/blockchain/transactions/:transaction_id", get(handlers::blockchain::get_transaction))
        .route("/blockchain/blocks", get(handlers::blockchain::list_blocks))
        .route("/blockchain/blocks/:block_hash", get(handlers::blockchain::get_block))
        .route("/blockchain/custody/:sample_id/events", post(handlers::blockchain::record_custody_event))
        .route("/blockchain/custody/:sample_id/history", get(handlers::blockchain::get_custody_history))
        .route("/blockchain/custody/:sample_id/validate", get(handlers::blockchain::validate_custody_chain))
        .route("/blockchain/audit", post(handlers::blockchain::create_audit_entry))
        .route("/blockchain/audit/search", get(handlers::blockchain::search_audit_trail))
        .route("/blockchain/stats", get(handlers::blockchain::get_blockchain_stats))
        .route("/blockchain/blocks", post(handlers::blockchain::create_block))
        .route("/blockchain/export", get(handlers::blockchain::export_blockchain))
        .route("/blockchain/verify-signature", post(handlers::blockchain::verify_signature));

    // Digital Twin routes (12 endpoints) - COMPLETE ✅
    let digital_twin_routes = Router::new()
        .route("/digital-twin/overview", get(handlers::digital_twin::get_overview))
        .route("/digital-twin/simulations", post(handlers::digital_twin::run_simulation))
        .route("/digital-twin/scenarios", post(handlers::digital_twin::create_scenario))
        .route("/digital-twin/optimization", get(handlers::digital_twin::get_optimization))
        .route("/digital-twin/models", get(handlers::digital_twin::list_models))
        .route("/digital-twin/models/:model_id", get(handlers::digital_twin::get_model))
        .route("/digital-twin/sync", post(handlers::digital_twin::sync_with_reality))
        .route("/digital-twin/predictions", get(handlers::digital_twin::get_predictions))
        .route("/digital-twin/virtual-experiments", post(handlers::digital_twin::run_virtual_experiment))
        .route("/digital-twin/analytics", get(handlers::digital_twin::get_twin_analytics));

    // Energy management routes (9 endpoints) - COMPLETE ✅
    let energy_routes = Router::new()
        .route("/energy/overview", get(handlers::energy::get_consumption_overview))
        .route("/energy/metrics/realtime", get(handlers::energy::get_realtime_metrics))
        .route("/energy/history", get(handlers::energy::get_consumption_history))
        .route("/energy/optimization/plan", post(handlers::energy::create_optimization_plan))
        .route("/energy/equipment", get(handlers::energy::get_equipment_profiles))
        .route("/energy/equipment/:equipment_id/control", post(handlers::energy::control_equipment_power))
        .route("/energy/analytics", get(handlers::energy::get_energy_analytics))
        .route("/energy/settings", post(handlers::energy::configure_energy_settings))
        .route("/energy/carbon-footprint", get(handlers::energy::get_carbon_footprint));

    // Mobile API routes (10 endpoints) - COMPLETE ✅
    let mobile_routes = Router::new()
        .route("/mobile/dashboard", get(handlers::mobile::get_mobile_dashboard))
        .route("/mobile/samples", get(handlers::mobile::get_mobile_samples))
        .route("/mobile/samples/barcode/:barcode", get(handlers::mobile::get_sample_by_barcode))
        .route("/mobile/alerts", get(handlers::mobile::get_mobile_alerts))
        .route("/mobile/alerts/:alert_id/acknowledge", post(handlers::mobile::acknowledge_alert))
        .route("/mobile/equipment", get(handlers::mobile::get_mobile_equipment))
        .route("/mobile/analytics", get(handlers::mobile::get_mobile_analytics))
        .route("/mobile/preferences", post(handlers::mobile::update_mobile_preferences))
        .route("/mobile/config", get(handlers::mobile::get_mobile_config))
        .route("/mobile/sync", post(handlers::mobile::sync_offline_data));

    // Compliance routes (6 endpoints) - COMPLETE ✅
    let compliance_routes = Router::new()
        .route("/compliance/overview", get(handlers::compliance::get_compliance_overview))
        .route("/compliance/audit-trail", get(handlers::compliance::get_audit_trail))
        .route("/compliance/reports", post(handlers::compliance::generate_compliance_report))
        .route("/compliance/validate", post(handlers::compliance::validate_regulatory_requirements))
        .route("/compliance/data-retention", get(handlers::compliance::manage_data_retention))
        .route("/compliance/access-permissions", get(handlers::compliance::track_access_permissions));

    // AI/ML Platform routes (9 endpoints) - Phase 2 🤖
    let ai_routes = Router::new()
        .route("/ai/overview", get(handlers::ai::get_ai_overview))
        .route("/ai/predict/equipment-failure", post(handlers::ai::predict_equipment_failure))
        .route("/ai/optimize/sample-routing", post(handlers::ai::optimize_sample_routing))
        .route("/ai/detect/anomalies", post(handlers::ai::detect_anomalies))
        .route("/ai/models/:model_name", get(handlers::ai::get_ai_model))
        .route("/ai/models/:model_name/update", post(handlers::ai::update_ai_model))
        .route("/ai/analytics", get(handlers::ai::get_ai_analytics))
        .route("/ai/config", post(handlers::ai::configure_ai_platform))
        .route("/ai/training/:job_id", get(handlers::ai::get_training_job));

    // Enterprise Integration routes (7 endpoints) - Phase 3 🏢
    let integration_routes = Router::new()
        .route("/integrations/overview", get(handlers::integrations::get_integration_overview))
        .route("/integrations/lims/samples/sync", post(handlers::integrations::sync_sample_to_lims))
        .route("/integrations/lims/workflows/:workflow_id", get(handlers::integrations::get_lims_workflow_status))
        .route("/integrations/erp/purchase-requisitions", post(handlers::integrations::create_erp_purchase_requisition))
        .route("/integrations/erp/budget/:department", get(handlers::integrations::get_erp_budget_status))
        .route("/integrations/cloud/upload", post(handlers::integrations::upload_to_cloud_storage))
        .route("/integrations/cloud/analytics", get(handlers::integrations::get_cloud_storage_analytics))
        .route("/integrations/health", get(handlers::integrations::get_integration_health))
        .route("/integrations/config", post(handlers::integrations::configure_integration_settings));

    // Combine all routes
    Router::new()
        .merge(health_routes)
        .merge(storage_routes)
        .merge(iot_routes)
        .merge(analytics_routes)
        .merge(admin_routes)
        .merge(automation_routes)
        .merge(blockchain_routes)
        .merge(digital_twin_routes)
        .merge(energy_routes)
        .merge(mobile_routes)
        .merge(compliance_routes)
        .merge(ai_routes)
        .merge(integration_routes)
        .layer(
            ServiceBuilder::new()
                .layer(TraceLayer::new_for_http())
                .layer(CorsLayer::permissive())
        )
        .with_state(state)
}
