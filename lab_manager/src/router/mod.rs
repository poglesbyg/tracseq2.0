use axum::{
    middleware,
    routing::{delete, get, post, put},
    Router,
};
use tower_http::cors::CorsLayer;

use crate::{
    handlers::{
        dashboard, health, rag_proxy, reports, samples, sequencing, spreadsheets, storage,
        templates, users,
    },
    middleware::{
        auth::{auth_middleware, optional_auth_middleware},
        shibboleth_auth::hybrid_auth_middleware,
        validation::validate_input_middleware,
    },
    AppComponents,
};

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

/// RAG proxy routes - forward requests to RAG API Bridge on port 3002
pub fn rag_proxy_routes() -> Router<AppComponents> {
    Router::new()
        .route("/api/rag/submissions", get(handlers::get_rag_submissions))
        .route("/api/rag/process", post(handlers::process_rag_document))
        .route("/api/rag/stats", get(handlers::get_rag_stats))
        .route("/api/rag/health", get(handlers::get_rag_health))
}

/// Sample management routes
pub fn sample_routes() -> Router<AppComponents> {
    Router::new()
        .route("/api/samples", get(handlers::list_samples))
        .route("/api/samples", post(handlers::create_sample))
        .route("/api/samples/batch", post(handlers::create_samples_batch))
        .route("/api/samples/:id", get(handlers::get_sample))
        .route("/api/samples/:id", put(handlers::update_sample))
        .route("/api/samples/:id/validate", post(handlers::validate_sample))
        // RAG-enhanced sample processing routes
        .route(
            "/api/samples/rag/process-document",
            post(handlers::process_document_and_create_samples),
        )
        .route(
            "/api/samples/rag/preview",
            post(handlers::preview_document_extraction),
        )
        .route(
            "/api/samples/rag/create-from-data",
            post(handlers::create_samples_from_rag_data),
        )
        .route(
            "/api/samples/rag/query",
            post(handlers::query_submission_information),
        )
        .route(
            "/api/samples/rag/status",
            get(handlers::get_rag_system_status),
        )
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
            "/api/sequencing/jobs/:id",
            get(handlers::get_sequencing_job),
        )
        .route(
            "/api/sequencing/jobs/:id/status",
            post(handlers::update_job_status),
        )
}

/// Storage management routes
pub fn storage_routes() -> Router<AppComponents> {
    Router::new()
        .route(
            "/api/storage/locations",
            get(handlers::list_storage_locations),
        )
        .route(
            "/api/storage/locations",
            post(handlers::create_storage_location),
        )
        .route("/api/storage/store", post(handlers::store_sample))
        .route("/api/storage/move", post(handlers::move_sample))
        .route("/api/storage/remove", post(handlers::remove_sample))
        .route(
            "/api/storage/scan/:barcode",
            get(handlers::scan_sample_barcode),
        )
        .route(
            "/api/storage/capacity",
            get(handlers::get_capacity_overview),
        )
}

/// Reports and analytics routes
pub fn reports_routes() -> Router<AppComponents> {
    Router::new()
        .route("/api/reports/execute", post(handlers::execute_report))
        .route(
            "/api/reports/templates",
            get(handlers::get_report_templates),
        )
        .route("/api/reports/schema", get(handlers::get_schema))
}

/// Spreadsheet processing routes
pub fn spreadsheet_routes() -> Router<AppComponents> {
    tracing::info!("🔧 Registering spreadsheet routes");

    let router = Router::new()
        .route(
            "/api/spreadsheets/upload",
            post(handlers::upload_spreadsheet),
        )
        .route(
            "/api/spreadsheets/upload-multiple",
            post(handlers::upload_spreadsheet_multiple_sheets),
        )
        .route(
            "/api/spreadsheets/preview-sheets",
            post(handlers::get_sheet_names),
        )
        .route("/api/spreadsheets/search", get(handlers::search_data))
        .route("/api/spreadsheets/datasets", get(handlers::list_datasets))
        .route("/api/spreadsheets/datasets/:id", get(handlers::get_dataset))
        .route(
            "/api/spreadsheets/datasets/:id",
            delete(handlers::delete_dataset),
        )
        .route(
            "/api/spreadsheets/datasets/:id/analyze",
            get(handlers::analyze_dataset),
        )
        .route(
            "/api/spreadsheets/datasets/:id/columns/:column_name/analyze",
            get(handlers::analyze_column),
        )
        .route(
            "/api/spreadsheets/filters",
            get(handlers::get_available_filters),
        )
        .route(
            "/api/spreadsheets/health",
            get(handlers::spreadsheets_health_check),
        )
        .route(
            "/api/spreadsheets/supported-types",
            get(handlers::supported_types),
        );

    tracing::info!("✅ Spreadsheet routes registered successfully");
    router
}

/// User management and authentication routes
pub fn user_routes() -> Router<AppComponents> {
    // Public routes (no authentication required)
    let public_routes = Router::new()
        .route("/api/auth/login", post(handlers::login))
        .route(
            "/api/auth/reset-password",
            post(handlers::request_password_reset),
        )
        .route(
            "/api/auth/confirm-reset",
            post(handlers::confirm_password_reset),
        )
        // Shibboleth-specific routes
        .route(
            "/shibboleth-login",
            get(handlers::shibboleth_login_redirect),
        )
        .route(
            "/shibboleth-logout",
            get(handlers::shibboleth_logout_redirect),
        );

    // Protected routes (authentication required)
    let protected_routes = Router::new()
        .route("/api/auth/logout", post(handlers::logout))
        .route("/api/users/me", get(handlers::get_current_user))
        .route("/api/users/me", put(handlers::update_current_user))
        .route("/api/users/me/password", put(handlers::change_password))
        .route("/api/users/me/sessions", get(handlers::get_user_sessions))
        .route(
            "/api/users/me/sessions",
            delete(handlers::revoke_all_sessions),
        )
        .route(
            "/api/users/me/sessions/:session_id",
            delete(handlers::revoke_session),
        )
        // Admin-only routes (authentication + authorization handled in handlers)
        .route("/api/users", post(handlers::create_user))
        .route("/api/users", get(handlers::list_users))
        .route("/api/users/:user_id", get(handlers::get_user))
        .route("/api/users/:user_id", put(handlers::update_user))
        .route("/api/users/:user_id", delete(handlers::delete_user));

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
        // Apply security and validation middleware to all routes except health checks
        .layer(middleware::from_fn_with_state(
            (), // Placeholder state, you might need to adjust this
            validate_input_middleware,
        ))
        // Apply authentication middleware to protected routes
        .layer(middleware::from_fn_with_state(
            (), // Placeholder state, you might need to adjust this
            hybrid_auth_middleware,
        ))
        // CORS layer
        .layer(
            CorsLayer::new()
                .allow_origin("http://localhost:5173".parse().unwrap())
                .allow_origin("http://localhost:8080".parse().unwrap())
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
