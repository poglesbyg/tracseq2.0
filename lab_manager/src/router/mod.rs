use axum::{
    routing::{delete, get, post, put},
    Router,
};
use tower_http::cors::CorsLayer;

use crate::{
    assembly::AppComponents,
    handlers::{
        dashboard, health, rag_proxy, reports, samples, sequencing, spreadsheets, storage,
        templates, users,
    },
};

/// Health and system routes
pub fn health_routes() -> Router<AppComponents> {
    Router::new()
        .route("/health", get(health::health_check))
        .route("/api/dashboard/stats", get(dashboard::get_dashboard_stats))
}

/// Template management routes
pub fn template_routes() -> Router<AppComponents> {
    Router::new()
        .route("/api/templates/upload", post(templates::upload_template))
        .route("/api/templates", get(templates::list_templates))
        .route("/api/templates/:id", get(templates::get_template))
        .route("/api/templates/:id", put(templates::update_template))
        .route("/api/templates/:id/data", get(templates::get_template_data))
        .route("/api/templates/:id", delete(templates::delete_template))
}

/// RAG proxy routes - forward requests to RAG API Bridge on port 3002
pub fn rag_proxy_routes() -> Router<AppComponents> {
    Router::new()
        .route("/api/rag/submissions", get(rag_proxy::get_rag_submissions))
        .route("/api/rag/process", post(rag_proxy::process_rag_document))
        .route("/api/rag/stats", get(rag_proxy::get_rag_stats))
        .route("/api/rag/health", get(rag_proxy::get_rag_health))
}

/// Sample management routes
pub fn sample_routes() -> Router<AppComponents> {
    Router::new()
        .route("/api/samples", get(samples::list_samples))
        .route("/api/samples", post(samples::create_sample))
        .route("/api/samples/batch", post(samples::create_samples_batch))
        .route("/api/samples/:id", get(samples::get_sample))
        .route("/api/samples/:id", put(samples::update_sample))
        .route("/api/samples/:id/validate", post(samples::validate_sample))
        // RAG-enhanced sample processing routes
        .route(
            "/api/samples/rag/process-document",
            post(samples::process_document_and_create_samples),
        )
        .route(
            "/api/samples/rag/preview",
            post(samples::preview_document_extraction),
        )
        .route(
            "/api/samples/rag/create-from-data",
            post(samples::create_samples_from_rag_data),
        )
        .route(
            "/api/samples/rag/query",
            post(samples::query_submission_information),
        )
        .route(
            "/api/samples/rag/status",
            get(samples::get_rag_system_status),
        )
}

/// Sequencing management routes
pub fn sequencing_routes() -> Router<AppComponents> {
    Router::new()
        .route(
            "/api/sequencing/jobs",
            post(sequencing::create_sequencing_job),
        )
        .route(
            "/api/sequencing/jobs",
            get(sequencing::list_sequencing_jobs),
        )
        .route(
            "/api/sequencing/jobs/:id",
            get(sequencing::get_sequencing_job),
        )
        .route(
            "/api/sequencing/jobs/:id/status",
            post(sequencing::update_job_status),
        )
}

/// Storage management routes
pub fn storage_routes() -> Router<AppComponents> {
    Router::new()
        .route(
            "/api/storage/locations",
            get(storage::get_storage_locations),
        )
        .route(
            "/api/storage/locations",
            post(storage::create_storage_location),
        )
        .route("/api/storage/store", post(storage::store_sample))
        .route("/api/storage/move", post(storage::move_sample))
        .route("/api/storage/remove", post(storage::remove_sample))
        .route(
            "/api/storage/scan/:barcode",
            get(storage::scan_sample_barcode),
        )
        .route("/api/storage/capacity", get(storage::get_capacity_overview))
}

/// Reports and analytics routes
pub fn reports_routes() -> Router<AppComponents> {
    Router::new()
        .route("/api/reports/execute", post(reports::execute_report))
        .route("/api/reports/templates", get(reports::get_report_templates))
        .route("/api/reports/schema", get(reports::get_schema))
}

/// Spreadsheet processing routes
pub fn spreadsheet_routes() -> Router<AppComponents> {
    tracing::info!("🔧 Registering spreadsheet routes");

    let router = Router::new()
        .route(
            "/api/spreadsheets/upload",
            post(spreadsheets::upload_spreadsheet),
        )
        .route(
            "/api/spreadsheets/upload-multiple",
            post(spreadsheets::upload_spreadsheet_multiple_sheets),
        )
        .route(
            "/api/spreadsheets/preview-sheets",
            post(spreadsheets::get_sheet_names),
        )
        .route("/api/spreadsheets/search", get(spreadsheets::search_data))
        .route(
            "/api/spreadsheets/datasets",
            get(spreadsheets::list_datasets),
        )
        .route(
            "/api/spreadsheets/datasets/:id",
            get(spreadsheets::get_dataset),
        )
        .route(
            "/api/spreadsheets/datasets/:id",
            delete(spreadsheets::delete_dataset),
        )
        .route(
            "/api/spreadsheets/datasets/:id/analyze",
            get(spreadsheets::analyze_dataset),
        )
        .route(
            "/api/spreadsheets/datasets/:id/columns/:column_name/analyze",
            get(spreadsheets::analyze_column),
        )
        .route(
            "/api/spreadsheets/filters",
            get(spreadsheets::get_available_filters),
        )
        .route("/api/spreadsheets/health", get(spreadsheets::health_check))
        .route(
            "/api/spreadsheets/supported-types",
            get(spreadsheets::supported_types),
        );

    tracing::info!("✅ Spreadsheet routes registered successfully");
    router
}

/// User management and authentication routes
pub fn user_routes() -> Router<AppComponents> {
    // Public routes (no authentication required)
    let public_routes = Router::new()
        .route("/api/auth/login", post(users::login))
        .route(
            "/api/auth/reset-password",
            post(users::request_password_reset),
        )
        .route(
            "/api/auth/confirm-reset",
            post(users::confirm_password_reset),
        )
        // Shibboleth-specific routes
        .route("/shibboleth-login", get(users::shibboleth_login_redirect))
        .route("/shibboleth-logout", get(users::shibboleth_logout_redirect));

    // Protected routes (authentication required)
    let protected_routes = Router::new()
        .route("/api/auth/logout", post(users::logout))
        .route("/api/users/me", get(users::get_current_user))
        .route("/api/users/me", put(users::update_current_user))
        .route("/api/users/me/password", put(users::change_password))
        .route("/api/users/me/sessions", get(users::get_user_sessions))
        .route("/api/users/me/sessions", delete(users::revoke_all_sessions))
        .route(
            "/api/users/me/sessions/:session_id",
            delete(users::revoke_session),
        )
        // Admin-only routes (authentication + authorization handled in handlers)
        .route("/api/users", post(users::create_user))
        .route("/api/users", get(users::list_users))
        .route("/api/users/:user_id", get(users::get_user))
        .route("/api/users/:user_id", put(users::update_user))
        .route("/api/users/:user_id", delete(users::delete_user));

    // Combine public and protected routes
    Router::new().merge(public_routes).merge(protected_routes)
}

/// Create authenticated routes (middleware applied at app level)
pub fn create_authenticated_routes() -> Router<AppComponents> {
    Router::new()
        .merge(template_routes())
        .merge(rag_proxy_routes())
        .merge(sample_routes())
        .merge(sequencing_routes())
        .merge(storage_routes())
        .merge(reports_routes())
        .merge(spreadsheet_routes())
        .merge(user_routes())
}

/// Assemble all routes into a complete application router
pub fn create_app_router() -> Router<AppComponents> {
    Router::new()
        // Health check routes (no authentication required for monitoring)
        .route("/health", get(health::health_check))
        .route("/health/system", get(health::system_health_check))
        .route("/health/database", get(health::database_health_check))
        .route("/health/metrics", get(health::application_metrics))
        .route("/health/ready", get(health::readiness_check))
        .route("/health/live", get(health::liveness_check))
        // Public authentication routes
        .route("/auth/login", post(users::login))
        .route("/auth/logout", post(users::logout))
        .route("/auth/reset-password", post(users::reset_password))
        // Dashboard routes (require authentication)
        .route("/dashboard/stats", get(dashboard::get_dashboard_stats))
        // Sample management routes
        .route("/samples", get(samples::list_samples))
        .route("/samples", post(samples::create_sample))
        .route("/samples/:id", get(samples::get_sample))
        .route("/samples/:id", put(samples::update_sample))
        .route("/samples/:id", delete(samples::delete_sample))
        // RAG proxy routes
        .route("/rag/process-document", post(rag_proxy::process_document))
        .route("/rag/query", post(rag_proxy::query_submissions))
        // Sequencing job routes
        .route("/sequencing", get(sequencing::list_sequencing_jobs))
        .route("/sequencing", post(sequencing::create_sequencing_job))
        .route("/sequencing/:id", get(sequencing::get_sequencing_job))
        .route("/sequencing/:id", put(sequencing::update_sequencing_job))
        .route("/sequencing/:id", delete(sequencing::delete_sequencing_job))
        // Storage management routes
        .route("/storage", get(storage::get_storage_locations))
        .route("/storage/:id", put(storage::update_storage_location))
        // Template management routes
        .route("/templates", get(templates::list_templates))
        .route("/templates", post(templates::create_template))
        .route("/templates/:id", get(templates::get_template))
        .route("/templates/:id", put(templates::update_template))
        .route("/templates/:id", delete(templates::delete_template))
        // Spreadsheet routes
        .route("/spreadsheets", get(spreadsheets::list_datasets))
        .route("/spreadsheets", post(spreadsheets::create_dataset))
        .route("/spreadsheets/:id", get(spreadsheets::get_dataset))
        .route("/spreadsheets/:id", put(spreadsheets::update_dataset))
        .route("/spreadsheets/:id", delete(spreadsheets::delete_dataset))
        .route(
            "/spreadsheets/search",
            post(spreadsheets::search_spreadsheet_data),
        )
        // Reports routes
        .route("/reports", get(reports::list_reports))
        .route("/reports", post(reports::create_custom_report))
        .route("/reports/:id", get(reports::get_report))
        .route("/reports/:id", put(reports::update_report))
        .route("/reports/:id", delete(reports::delete_report))
        .route("/reports/templates", get(reports::get_available_templates))
        .route("/reports/templates", post(reports::save_report_template))
        // User management routes
        .route("/users", get(users::list_users))
        .route("/users", post(users::create_user))
        .route("/users/me", get(users::get_current_user))
        .route("/users/:id", get(users::get_user))
        .route("/users/:id", put(users::update_user))
        .route("/users/:id", delete(users::delete_user))
        // CORS layer
        .layer(
            CorsLayer::new()
                .allow_origin(
                    "http://localhost:5173"
                        .parse::<axum::http::HeaderValue>()
                        .map_err(|e| {
                            tracing::error!("Failed to parse CORS origin: {}", e);
                            e
                        })
                        .unwrap_or_else(|_| {
                            axum::http::HeaderValue::from_static("http://localhost:5173")
                        }),
                )
                .allow_origin(
                    "http://localhost:8080"
                        .parse::<axum::http::HeaderValue>()
                        .map_err(|e| {
                            tracing::error!("Failed to parse CORS origin: {}", e);
                            e
                        })
                        .unwrap_or_else(|_| {
                            axum::http::HeaderValue::from_static("http://localhost:8080")
                        }),
                )
                .allow_methods([
                    axum::http::Method::GET,
                    axum::http::Method::POST,
                    axum::http::Method::PUT,
                    axum::http::Method::DELETE,
                    axum::http::Method::OPTIONS,
                ])
                .allow_headers([
                    axum::http::header::CONTENT_TYPE,
                    axum::http::header::AUTHORIZATION,
                    axum::http::header::USER_AGENT,
                ])
                .allow_credentials(true),
        )
}

/// Create a minimal router for testing
pub fn create_test_router() -> Router<AppComponents> {
    Router::new()
        .route("/health", get(health::health_check))
        .route("/api/templates", get(templates::list_templates))
}

/// Create API-only router (no file uploads)
pub fn create_api_only_router() -> Router<AppComponents> {
    Router::new()
        .merge(health_routes())
        .route("/api/templates", get(templates::list_templates))
        .merge(sample_routes())
        .merge(sequencing_routes())
        .merge(storage_routes())
        .layer(CorsLayer::permissive())
}
