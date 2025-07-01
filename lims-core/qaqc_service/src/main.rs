use axum::{
    Router,
    routing::{get, post, put},
};
use tower_http::cors::CorsLayer;
use tower_http::trace::TraceLayer;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

mod config;
mod database;
mod error;
mod models;

mod handlers;
mod middleware;
mod services;

use config::Config;
use database::create_pool;
// use handlers::{compliance, qc_workflows, quality_metrics, reports};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize tracing
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "qaqc_service=debug,tower_http=debug".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    // Load configuration
    let config = Config::from_env()?;

    // Create database connection pool
    let pool = create_pool(&config.database_url).await?;

    // Run database migrations unless SKIP_MIGRATIONS is set
    if std::env::var("SKIP_MIGRATIONS").unwrap_or_default() != "true" {
        database::run_migrations(&pool).await?;
    }

    // Build application router
    let app = Router::new()
        .route("/health", get(health_check))
        // Old routes commented out - using new handlers
        // .route("/api/v1/qc/workflows", get(qc_workflows::list_workflows))
        // ... (old routes commented out)
        // New QC endpoints
        .route("/api/v1/qc/dashboard", get(handlers::get_qc_dashboard))
        .route("/api/v1/qc/reviews", get(handlers::list_qc_reviews))
        .route("/api/v1/qc/reviews", post(handlers::create_qc_review))
        .route("/api/v1/qc/reviews/:id", get(handlers::get_qc_review))
        .route("/api/v1/qc/reviews/:id/complete", post(handlers::complete_qc_review))
        .route("/api/v1/qc/library-prep", post(handlers::create_library_prep_qc))
        .route("/api/v1/qc/library-prep/:id", get(handlers::get_library_prep_qc))
        .route("/api/v1/qc/metrics/trends", get(handlers::get_qc_metric_trends))
        .route("/api/v1/qc/metrics/recent", get(handlers::get_recent_qc_metrics))
        .route("/api/v1/qc/metrics/definitions", get(handlers::list_qc_metrics))
        .route("/api/v1/qc/metrics/definitions", post(handlers::upsert_qc_metric))
        .route("/api/v1/qc/control-samples", get(handlers::list_control_samples))
        .route("/api/v1/qc/control-samples", post(handlers::create_control_sample))
        .route("/api/v1/qc/control-samples/results", post(handlers::record_control_result))
        .route("/api/v1/qc/control-samples/:id/results", get(handlers::get_control_results))
        .with_state(pool)
        // Middleware
        .layer(axum::middleware::from_fn(
            crate::middleware::auth_middleware,
        ))
        .layer(CorsLayer::permissive())
        .layer(TraceLayer::new_for_http());

    // Start server
    let listener =
        tokio::net::TcpListener::bind(format!("{}:{}", config.host, config.port)).await?;

    tracing::info!("QAQC service listening on {}:{}", config.host, config.port);

    axum::serve(listener, app).await?;

    Ok(())
}

async fn health_check() -> &'static str {
    "QAQC Service is healthy"
}
