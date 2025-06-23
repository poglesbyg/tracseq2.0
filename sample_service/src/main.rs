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
mod middleware;
mod clients;

use config::Config;
use database::DatabasePool;
use services::SampleServiceImpl;
use clients::{AuthClient, StorageClient};

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize tracing
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "sample_service=debug,tower_http=debug".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    info!("Starting Sample Management Service");

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
    let storage_client = StorageClient::new(config.storage_service_url.clone());

    // Initialize sample service
    let sample_service = SampleServiceImpl::new(
        db_pool.clone(),
        config.clone(),
        auth_client.clone(),
        storage_client.clone(),
    )?;
    info!("Sample service initialized");

    // Setup application state
    let app_state = AppState {
        sample_service,
        config: config.clone(),
        db_pool,
        auth_client,
        storage_client,
    };

    // Build the application router
    let app = create_app(app_state);

    // Start the server
    let addr = SocketAddr::from(([0, 0, 0, 0], config.server.port));
    info!("Sample management service listening on {}", addr);

    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}

/// Application state shared across handlers
#[derive(Clone)]
pub struct AppState {
    pub sample_service: SampleServiceImpl,
    pub config: Config,
    pub db_pool: DatabasePool,
    pub auth_client: AuthClient,
    pub storage_client: StorageClient,
}

/// Create the application router with all routes and middleware
fn create_app(state: AppState) -> Router {
    // Health check routes (no auth required)
    let health_routes = Router::new()
        .route("/health", get(handlers::health::health_check))
        .route("/health/ready", get(handlers::health::readiness_check))
        .route("/health/metrics", get(handlers::health::metrics));

    // Public sample routes (require authentication)
    let sample_routes = Router::new()
        .route("/samples", post(handlers::samples::create_sample))
        .route("/samples", get(handlers::samples::list_samples))
        .route("/samples/:sample_id", get(handlers::samples::get_sample))
        .route("/samples/:sample_id", put(handlers::samples::update_sample))
        .route("/samples/:sample_id", delete(handlers::samples::delete_sample))
        .route("/samples/:sample_id/validate", post(handlers::samples::validate_sample))
        .route("/samples/:sample_id/status", put(handlers::samples::update_status))
        .route("/samples/barcode/:barcode", get(handlers::samples::get_sample_by_barcode))
        .layer(middleware::from_fn_with_state(
            state.clone(),
            sample_middleware::auth_middleware,
        ));

    // Batch operations
    let batch_routes = Router::new()
        .route("/samples/batch", post(handlers::samples::create_batch_samples))
        .route("/samples/batch/validate", post(handlers::samples::validate_batch))
        .layer(middleware::from_fn_with_state(
            state.clone(),
            sample_middleware::auth_middleware,
        ));

    // Barcode operations
    let barcode_routes = Router::new()
        .route("/barcodes/generate", post(handlers::barcodes::generate_barcode))
        .route("/barcodes/validate", post(handlers::barcodes::validate_barcode))
        .route("/barcodes/scan", post(handlers::barcodes::scan_barcode))
        .layer(middleware::from_fn_with_state(
            state.clone(),
            sample_middleware::auth_middleware,
        ));

    // Workflow routes
    let workflow_routes = Router::new()
        .route("/workflow/transitions", get(handlers::workflow::get_valid_transitions))
        .route("/workflow/history/:sample_id", get(handlers::workflow::get_sample_history))
        .layer(middleware::from_fn_with_state(
            state.clone(),
            sample_middleware::auth_middleware,
        ));

    // Template integration routes
    let template_routes = Router::new()
        .route("/templates", get(handlers::templates::list_templates))
        .route("/templates/:template_id/samples", post(handlers::templates::create_sample_from_template))
        .route("/templates/:template_id/validate", post(handlers::templates::validate_template_data))
        .layer(middleware::from_fn_with_state(
            state.clone(),
            sample_middleware::auth_middleware,
        ));

    // Admin routes (require admin privileges)
    let admin_routes = Router::new()
        .route("/admin/samples/stats", get(handlers::admin::get_sample_statistics))
        .route("/admin/samples/cleanup", post(handlers::admin::cleanup_samples))
        .route("/admin/workflow/status", get(handlers::admin::get_workflow_status))
        .layer(middleware::from_fn_with_state(
            state.clone(),
            sample_middleware::admin_middleware,
        ));

    // Combine all routes
    Router::new()
        .merge(health_routes)
        .merge(sample_routes)
        .merge(batch_routes)
        .merge(barcode_routes)
        .merge(workflow_routes)
        .merge(template_routes)
        .merge(admin_routes)
        .layer(
            ServiceBuilder::new()
                .layer(TraceLayer::new_for_http())
                .layer(CompressionLayer::new())
                .layer(CorsLayer::permissive()) // Configure CORS as needed
        )
        .with_state(state)
} 
