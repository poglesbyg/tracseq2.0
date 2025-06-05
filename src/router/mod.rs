use axum::{
    routing::{delete, get, post, put},
    Router,
};
use tower_http::cors::CorsLayer;

use crate::{handlers, AppComponents};

/// Health and system routes
pub fn health_routes() -> Router<AppComponents> {
    Router::new()
        .route("/health", get(handlers::health_check))
        .route("/api/dashboard/stats", get(handlers::get_dashboard_stats))
}

/// Template management routes
pub fn template_routes() -> Router<AppComponents> {
    Router::new()
        .route("/api/templates/upload", post(handlers::upload_template))
        .route("/api/templates", get(handlers::list_templates))
        .route("/api/templates/:id", get(handlers::get_template))
        .route("/api/templates/:id", put(handlers::update_template))
        .route("/api/templates/:id/data", get(handlers::get_template_data))
        .route("/api/templates/:id", delete(handlers::delete_template))
}

/// Sample management routes
pub fn sample_routes() -> Router<AppComponents> {
    Router::new()
        .route("/api/samples", post(handlers::create_sample))
        .route("/api/samples", get(handlers::list_samples))
        .route("/api/samples/:id", get(handlers::get_sample))
        .route("/api/samples/:id", put(handlers::update_sample))
        .route("/api/samples/batch", post(handlers::create_samples_batch))
        .route("/api/samples/:id/validate", post(handlers::validate_sample))
}

/// Sequencing management routes
pub fn sequencing_routes() -> Router<AppComponents> {
    Router::new()
        .route(
            "/api/sequencing/jobs",
            post(handlers::create_sequencing_job),
        )
        .route("/api/sequencing/jobs", get(handlers::list_sequencing_jobs))
        .route(
            "/api/sequencing/jobs/:id/status",
            post(handlers::update_job_status),
        )
}

/// Storage management routes
pub fn storage_routes() -> Router<AppComponents> {
    Router::new().route(
        "/api/storage/locations",
        get(handlers::list_storage_locations),
    )
}

/// Assemble all routes into a complete application router
pub fn create_app_router() -> Router<AppComponents> {
    Router::new()
        .merge(health_routes())
        .merge(template_routes())
        .merge(sample_routes())
        .merge(sequencing_routes())
        .merge(storage_routes())
        .layer(CorsLayer::permissive())
}

/// Create a minimal router for testing
pub fn create_test_router() -> Router<AppComponents> {
    Router::new()
        .route("/health", get(handlers::health_check))
        .route("/api/templates", get(handlers::list_templates))
}

/// Create API-only router (no file uploads)
pub fn create_api_only_router() -> Router<AppComponents> {
    Router::new()
        .merge(health_routes())
        .route("/api/templates", get(handlers::list_templates))
        .merge(sample_routes())
        .merge(sequencing_routes())
        .merge(storage_routes())
        .layer(CorsLayer::permissive())
}
