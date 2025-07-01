//! Reports Service Library
//! 
//! This module exposes the internal types and functions for testing purposes.

use axum::{
    routing::{get, post, put, delete},
    Router,
};
use sqlx::postgres::PgPoolOptions;
use std::sync::Arc;
use tower_http::cors::CorsLayer;

// Include the modules from main.rs
pub mod config;
pub mod handlers;
pub mod models;
pub mod services;

// Re-export types
pub use crate::config::{Settings, ServiceUrls, StorageConfig};

// Define AppState
#[derive(Clone)]
pub struct AppState {
    pub pool: sqlx::PgPool,
    pub http_client: reqwest::Client,
    pub settings: Settings,
    pub template_engine: tera::Tera,
    pub scheduler: Arc<tokio_cron_scheduler::JobScheduler>,
}

// Health check handler
pub async fn health_check() -> axum::Json<serde_json::Value> {
    axum::Json(serde_json::json!({
        "status": "healthy",
        "service": "reports-service",
        "timestamp": chrono::Utc::now()
    }))
}

// Create router for testing
pub fn create_router(state: Arc<AppState>) -> Router {
    Router::new()
        // Health check
        .route("/health", get(health_check))
        // Report generation endpoints
        .route("/api/reports", get(handlers::reports::list_reports))
        .route("/api/reports/:id", get(handlers::reports::get_report))
        .route("/api/reports/generate", post(handlers::reports::generate_report))
        .route("/api/reports/:id/download", get(handlers::reports::download_report))
        // Report templates
        .route("/api/reports/templates", get(handlers::templates::list_templates))
        .route("/api/reports/templates/:id", get(handlers::templates::get_template))
        .route("/api/reports/templates", post(handlers::templates::create_template))
        // Scheduled reports
        .route("/api/reports/schedules", get(handlers::schedules::list_schedules))
        .route("/api/reports/schedules", post(handlers::schedules::create_schedule))
        .route("/api/reports/schedules/:id", get(handlers::schedules::get_schedule))
        .route("/api/reports/schedules/:id", put(handlers::schedules::update_schedule))
        .route("/api/reports/schedules/:id", delete(handlers::schedules::delete_schedule))
        // Analytics reports
        .route("/api/reports/analytics/samples", get(handlers::analytics::sample_analytics))
        .route("/api/reports/analytics/sequencing", get(handlers::analytics::sequencing_analytics))
        .route("/api/reports/analytics/storage", get(handlers::analytics::storage_analytics))
        .route("/api/reports/analytics/financial", get(handlers::analytics::financial_analytics))
        .route("/api/reports/analytics/performance", get(handlers::analytics::performance_analytics))
        // Export endpoints
        .route("/api/reports/export/pdf", post(handlers::export::export_pdf))
        .route("/api/reports/export/excel", post(handlers::export::export_excel))
        .route("/api/reports/export/csv", post(handlers::export::export_csv))
        // Custom queries
        .route("/api/reports/query", post(handlers::query::execute_query))
        .route("/api/reports/query/saved", get(handlers::query::list_saved_queries))
        .route("/api/reports/query/saved", post(handlers::query::save_query))
        .layer(CorsLayer::permissive())
        .with_state(state)
}