use axum::{
    routing::{get, post, put},
    Router,
};
use sqlx::PgPool;
use std::sync::Arc;
use tower_http::cors::CorsLayer;
use tower_http::trace::TraceLayer;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

mod config;
mod database;
mod error;
mod models;
mod services;
mod handlers;
mod middleware;

use config::Config;
use database::create_pool;
use handlers::{qc_workflows, quality_metrics, compliance, reports};
use services::{QaqcService, QualityMetricsService, ComplianceService};

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
    
    // Run database migrations
    database::run_migrations(&pool).await?;
    
    // Initialize services
    let qaqc_service = Arc::new(QaqcService::new(pool.clone()));
    let metrics_service = Arc::new(QualityMetricsService::new(pool.clone()));
    let compliance_service = Arc::new(ComplianceService::new(pool.clone()));
    
    // Build application router
    let app = Router::new()
        .route("/health", get(health_check))
        
        // QC Workflow endpoints
        .route("/api/v1/qc/workflows", get(qc_workflows::list_workflows))
        .route("/api/v1/qc/workflows", post(qc_workflows::create_workflow))
        .route("/api/v1/qc/workflows/:id", get(qc_workflows::get_workflow))
        .route("/api/v1/qc/workflows/:id", put(qc_workflows::update_workflow))
        .route("/api/v1/qc/workflows/:id/execute", post(qc_workflows::execute_workflow))
        .route("/api/v1/qc/workflows/:id/status", get(qc_workflows::get_workflow_status))
        
        // Quality Metrics endpoints
        .route("/api/v1/quality/metrics", get(quality_metrics::list_metrics))
        .route("/api/v1/quality/metrics", post(quality_metrics::create_metric))
        .route("/api/v1/quality/metrics/batch", post(quality_metrics::create_batch_metrics))
        .route("/api/v1/quality/metrics/:id", get(quality_metrics::get_metric))
        .route("/api/v1/quality/thresholds", get(quality_metrics::get_thresholds))
        .route("/api/v1/quality/thresholds", put(quality_metrics::update_thresholds))
        .route("/api/v1/quality/analysis", get(quality_metrics::get_quality_analysis))
        
        // Compliance endpoints
        .route("/api/v1/compliance/rules", get(compliance::list_rules))
        .route("/api/v1/compliance/rules", post(compliance::create_rule))
        .route("/api/v1/compliance/validate", post(compliance::validate_compliance))
        .route("/api/v1/compliance/audit", get(compliance::get_audit_trail))
        
        // Reports endpoints
        .route("/api/v1/reports/quality", get(reports::quality_report))
        .route("/api/v1/reports/compliance", get(reports::compliance_report))
        .route("/api/v1/reports/trends", get(reports::trend_analysis))
        .route("/api/v1/reports/export", get(reports::export_data))
        
        // Service state management
        .with_state(qaqc_service.clone())
        .with_state(metrics_service.clone())
        .with_state(compliance_service.clone())
        
        // Middleware
        .layer(axum::middleware::from_fn(crate::middleware::auth_middleware))
        .layer(CorsLayer::permissive())
        .layer(TraceLayer::new_for_http());

    // Start server
    let listener = tokio::net::TcpListener::bind(format!("{}:{}", config.host, config.port)).await?;
    
    tracing::info!(
        "QAQC service listening on {}:{}",
        config.host,
        config.port
    );
    
    axum::serve(listener, app).await?;
    
    Ok(())
}

async fn health_check() -> &'static str {
    "QAQC Service is healthy"
} 
