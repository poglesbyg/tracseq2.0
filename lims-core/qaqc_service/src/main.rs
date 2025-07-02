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
    // Initialize tracing first so we can see logs
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "qaqc_service=debug,tower_http=debug".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    println!("QAQC SERVICE: Starting up...");
    tracing::info!("🚀 Starting QAQC service");

    // Load configuration
    println!("QAQC SERVICE: Loading configuration...");
    let config = match Config::from_env() {
        Ok(cfg) => {
            println!("QAQC SERVICE: Configuration loaded successfully");
            tracing::info!("Configuration loaded successfully");
            cfg
        }
        Err(e) => {
            println!("QAQC SERVICE: Failed to load configuration: {}", e);
            tracing::error!("Failed to load configuration: {}", e);
            return Err(e.into());
        }
    };

    // Create database connection pool
    println!("QAQC SERVICE: Connecting to database at: {}", config.database_url);
    let pool = match create_pool(&config.database_url).await {
        Ok(p) => {
            println!("QAQC SERVICE: Database connection successful");
            tracing::info!("Database connection established");
            p
        }
        Err(e) => {
            println!("QAQC SERVICE: Failed to connect to database: {}", e);
            tracing::error!("Failed to connect to database: {}", e);
            return Err(e.into());
        }
    };

    // Run database migrations unless SKIP_MIGRATIONS is set
    if std::env::var("SKIP_MIGRATIONS").unwrap_or_default() != "true" {
        println!("QAQC SERVICE: Running database migrations...");
        match database::run_migrations(&pool).await {
            Ok(_) => {
                println!("QAQC SERVICE: Database migrations completed");
                tracing::info!("Database migrations completed");
            }
            Err(e) => {
                println!("QAQC SERVICE: Failed to run migrations: {}", e);
                tracing::error!("Failed to run migrations: {}", e);
                return Err(e.into());
            }
        }
    } else {
        println!("QAQC SERVICE: Skipping database migrations (SKIP_MIGRATIONS=true)");
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
    println!("QAQC SERVICE: Starting server on {}:{}", config.host, config.port);
    let listener = match tokio::net::TcpListener::bind(format!("{}:{}", config.host, config.port)).await {
        Ok(l) => {
            println!("QAQC SERVICE: Successfully bound to {}:{}", config.host, config.port);
            l
        }
        Err(e) => {
            println!("QAQC SERVICE: Failed to bind to {}:{} - {}", config.host, config.port, e);
            tracing::error!("Failed to bind to {}:{} - {}", config.host, config.port, e);
            return Err(e.into());
        }
    };

    println!("QAQC SERVICE: Server is ready, starting to serve requests...");
    tracing::info!("🌐 QAQC service listening on {}:{}", config.host, config.port);

    match axum::serve(listener, app).await {
        Ok(_) => {
            println!("QAQC SERVICE: Server stopped gracefully");
            Ok(())
        }
        Err(e) => {
            println!("QAQC SERVICE: Server error: {}", e);
            tracing::error!("Server error: {}", e);
            Err(e.into())
        }
    }
}

async fn health_check() -> &'static str {
    "QAQC Service is healthy"
}
