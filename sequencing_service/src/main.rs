use anyhow::Result;
use axum::{
    middleware as axum_middleware,
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
mod middleware;
mod clients;
// TODO: Re-enable when modules exist
// mod workflow;
// mod analysis; 
// mod scheduling;

use config::Config;
use database::DatabasePool;
use services::SequencingServiceImpl;
use clients::{AuthClient, SampleClient, NotificationClient, TemplateClient};

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize tracing
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "sequencing_service=debug,tower_http=debug".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    info!("Starting Sequencing Management Service");

    // Load configuration
    let config = Config::from_env()?;
    info!("Configuration loaded successfully");

    // Setup database connection
    let db_pool = DatabasePool::new(&config.database_url).await?;
    info!("Database connection established");

    // Run database migrations
    db_pool.migrate().await?;
    info!("Database migrations completed");

    // Initialize external service clients
    let auth_client = AuthClient::new(config.auth_service_url.clone());
    let sample_client = SampleClient::new(config.sample_service_url.clone());
    let notification_client = NotificationClient::new(config.notification_service_url.clone());
    let template_client = TemplateClient::new(config.template_service_url.clone());

    // Initialize sequencing service
    let sequencing_service = SequencingServiceImpl::new(
        db_pool.clone(),
        config.clone(),
        auth_client.clone(),
        sample_client.clone(),
        notification_client.clone(),
        template_client.clone(),
    )?;
    info!("Sequencing service initialized");

    // Setup application state
    let app_state = AppState {
        sequencing_service,
        config: config.clone(),
        db_pool,
        auth_client,
        sample_client,
        notification_client,
        template_client,
    };

    // Build the application router
    let app = create_app(app_state);

    // Start the server
    let addr = SocketAddr::from(([0, 0, 0, 0], config.server.port));
    info!("Sequencing management service listening on {}", addr);

    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}

/// Application state shared across handlers
#[derive(Clone)]
pub struct AppState {
    pub sequencing_service: SequencingServiceImpl,
    pub config: Config,
    pub db_pool: DatabasePool,
    pub auth_client: AuthClient,
    pub sample_client: SampleClient,
    pub notification_client: NotificationClient,
    pub template_client: TemplateClient,
}

/// Create the application router with all routes and middleware
fn create_app(state: AppState) -> Router {
    // Health check routes (no auth required)
    let health_routes = Router::new()
        .route("/health", get(handlers::health::health_check))
        .route("/health/ready", get(handlers::health::readiness_check))
        .route("/health/metrics", get(handlers::health::metrics));

    // Sequencing job management routes
    let job_routes = Router::new()
        .route("/jobs", post(handlers::jobs::create_job))
        .route("/jobs", get(handlers::jobs::list_jobs))
        .route("/jobs/:job_id", get(handlers::jobs::get_job))
        .route("/jobs/:job_id", put(handlers::jobs::update_job))
        .route("/jobs/:job_id", delete(handlers::jobs::delete_job))
        .route("/jobs/:job_id/status", put(handlers::jobs::update_job_status))
        .route("/jobs/:job_id/clone", post(handlers::jobs::clone_job))
        .route("/jobs/:job_id/cancel", post(handlers::jobs::cancel_job))
        .layer(axum_middleware::from_fn_with_state(
            state.clone(),
            crate::middleware::auth_middleware,
        ));

    // Workflow management routes
    let workflow_routes = Router::new()
        .route("/workflows", get(handlers::workflows::list_workflows))
        .route("/workflows/:workflow_id", get(handlers::workflows::get_workflow))
        .route("/workflows/:workflow_id/execute", post(handlers::workflows::execute_workflow))
        .route("/workflows/:workflow_id/pause", post(handlers::workflows::pause_workflow))
        .route("/workflows/:workflow_id/resume", post(handlers::workflows::resume_workflow))
        .route("/workflows/:workflow_id/abort", post(handlers::workflows::abort_workflow))
        .layer(axum_middleware::from_fn_with_state(
            state.clone(),
            crate::middleware::auth_middleware,
        ));

    // Sample sheet management routes
    let sample_sheet_routes = Router::new()
        .route("/sample-sheets", post(handlers::sample_sheets::create_sample_sheet))
        .route("/sample-sheets", get(handlers::sample_sheets::list_sample_sheets))
        .route("/sample-sheets/:sheet_id", get(handlers::sample_sheets::get_sample_sheet))
        .route("/sample-sheets/:sheet_id", put(handlers::sample_sheets::update_sample_sheet))
        .route("/sample-sheets/:sheet_id", delete(handlers::sample_sheets::delete_sample_sheet))
        .route("/sample-sheets/:sheet_id/download", get(handlers::sample_sheets::download_sample_sheet))
        .route("/sample-sheets/:sheet_id/validate", post(handlers::sample_sheets::validate_sample_sheet))
        .layer(axum_middleware::from_fn_with_state(
            state.clone(),
            crate::middleware::auth_middleware,
        ));

    // Sequencing run management routes
    let run_routes = Router::new()
        .route("/runs", post(handlers::runs::create_run))
        .route("/runs", get(handlers::runs::list_runs))
        .route("/runs/:run_id", get(handlers::runs::get_run))
        .route("/runs/:run_id", put(handlers::runs::update_run))
        .route("/runs/:run_id", delete(handlers::runs::delete_run))
        .route("/runs/:run_id/start", post(handlers::runs::start_run))
        .route("/runs/:run_id/stop", post(handlers::runs::stop_run))
        .route("/runs/:run_id/metrics", get(handlers::runs::get_run_metrics))
        .layer(axum_middleware::from_fn_with_state(
            state.clone(),
            crate::middleware::auth_middleware,
        ));

    // Analysis pipeline routes
    let analysis_routes = Router::new()
        .route("/analysis/pipelines", get(handlers::analysis::list_pipelines))
        .route("/analysis/pipelines/:pipeline_id", get(handlers::analysis::get_pipeline))
        .route("/analysis/pipelines/:pipeline_id/execute", post(handlers::analysis::execute_pipeline))
        .route("/analysis/jobs", get(handlers::analysis::list_analysis_jobs))
        .route("/analysis/jobs/:job_id", get(handlers::analysis::get_analysis_job))
        .route("/analysis/jobs/:job_id/results", get(handlers::analysis::get_analysis_results))
        .layer(axum_middleware::from_fn_with_state(
            state.clone(),
            crate::middleware::auth_middleware,
        ));

    // Quality control routes
    let qc_routes = Router::new()
        .route("/qc/metrics", get(handlers::quality::get_qc_metrics))
        .route("/qc/reports", get(handlers::quality::list_qc_reports))
        .route("/qc/reports/:report_id", get(handlers::quality::get_qc_report))
        .route("/qc/thresholds", get(handlers::quality::get_qc_thresholds))
        .route("/qc/thresholds", put(handlers::quality::update_qc_thresholds))
        .layer(axum_middleware::from_fn_with_state(
            state.clone(),
            crate::middleware::auth_middleware,
        ));

    // Scheduling routes
    let scheduling_routes = Router::new()
        .route("/schedule/jobs", get(handlers::scheduling::list_scheduled_jobs))
        .route("/schedule/jobs", post(handlers::scheduling::schedule_job))
        .route("/schedule/jobs/:job_id", get(handlers::scheduling::get_scheduled_job))
        .route("/schedule/jobs/:job_id", put(handlers::scheduling::update_scheduled_job))
        .route("/schedule/jobs/:job_id", delete(handlers::scheduling::cancel_scheduled_job))
        .route("/schedule/calendar", get(handlers::scheduling::get_schedule_calendar))
        .layer(axum_middleware::from_fn_with_state(
            state.clone(),
            crate::middleware::auth_middleware,
        ));

    // Integration routes
    let integration_routes = Router::new()
        .route("/integration/samples/validate", post(handlers::integration::validate_samples_for_sequencing))
        .route("/integration/templates/sequencing", get(handlers::integration::get_sequencing_templates))
        .route("/integration/notifications/subscribe", post(handlers::integration::subscribe_to_notifications))
        .route("/integration/lims/sync", post(handlers::integration::sync_with_lims))
        .layer(axum_middleware::from_fn_with_state(
            state.clone(),
            crate::middleware::auth_middleware,
        ));

    // Data export routes
    let export_routes = Router::new()
        .route("/export/jobs", get(handlers::export::export_jobs))
        .route("/export/runs", get(handlers::export::export_runs))
        .route("/export/metrics", get(handlers::export::export_metrics))
        .route("/export/results", get(handlers::export::export_results))
        .layer(axum_middleware::from_fn_with_state(
            state.clone(),
            crate::middleware::auth_middleware,
        ));

    // Admin routes (require admin privileges)
    let admin_routes = Router::new()
        .route("/admin/statistics", get(handlers::admin::get_sequencing_statistics))
        .route("/admin/maintenance", post(handlers::admin::run_maintenance))
        .route("/admin/config", get(handlers::admin::get_configuration))
        .route("/admin/config", put(handlers::admin::update_configuration))
        .route("/admin/cleanup", post(handlers::admin::cleanup_old_data))
        .route("/admin/backup", post(handlers::admin::backup_data))
        .layer(axum_middleware::from_fn_with_state(
            state.clone(),
            crate::middleware::admin_middleware,
        ));

    // Combine all routes
    Router::new()
        .merge(health_routes)
        .merge(job_routes)
        .merge(workflow_routes)
        .merge(sample_sheet_routes)
        .merge(run_routes)
        .merge(analysis_routes)
        .merge(qc_routes)
        .merge(scheduling_routes)
        .merge(integration_routes)
        .merge(export_routes)
        .merge(admin_routes)
        .layer(
            ServiceBuilder::new()
                .layer(TraceLayer::new_for_http())
                .layer(CompressionLayer::new())
                .layer(CorsLayer::permissive()) // Configure CORS as needed
        )
        .with_state(state)
} 
