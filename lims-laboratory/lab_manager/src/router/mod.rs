pub mod proxy_routes;

use axum::{
    routing::{delete, get, post, put},
    Router,
};
use tower_http::cors::CorsLayer;
use std::sync::Arc;

use crate::{
    assembly::AppComponents,
    handlers::{
        dashboard, health, proxy_handlers, rag_proxy, reports, samples, sequencing, spreadsheets,
        templates, users, projects, library_prep, qc, flow_cell,
    },
};

use self::proxy_routes::create_proxy_routes;

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

/// Storage management routes (disabled - storage module removed)
pub fn storage_routes() -> Router<AppComponents> {
    Router::new()
        // Storage routes temporarily disabled
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

/// Project management routes
pub fn project_routes() -> Router<AppComponents> {
    Router::new()
        .route("/api/projects", get(projects::list_projects))
        .route("/api/projects", post(projects::create_project))
        .route("/api/projects/:id", get(projects::get_project))
        .route("/api/projects/:id", put(projects::update_project))
        .route("/api/projects/:id", delete(projects::delete_project))
        .route("/api/projects/:id/files", get(projects::get_project_files))
        .route("/api/projects/:id/files", post(projects::upload_project_file))
        .route("/api/projects/:id/signoffs", get(projects::list_project_signoffs))
        .route("/api/batches", get(projects::list_batches))
        .route("/api/batches", post(projects::create_batch))
        .route("/api/batches/:id", get(projects::get_batch))
        .route("/api/signoffs", post(projects::create_signoff))
        .route("/api/signoffs/:id", put(projects::update_signoff))
        .route("/api/templates-repository", get(projects::list_templates_repository))
        .route("/api/templates-repository/:id/download", post(projects::download_template_repository))
        .route("/api/permission-queue", get(projects::list_permission_queue))
        .route("/api/permission-queue", post(projects::create_permission_request))
        .route("/api/permission-queue/:id", put(projects::update_permission_request))
}

/// Library preparation routes
pub fn library_prep_routes() -> Router<AppComponents> {
    Router::new()
        .route("/api/library-prep/protocols", get(library_prep::list_protocols))
        .route("/api/library-prep/protocols", post(library_prep::create_protocol))
        .route("/api/library-prep/protocols/:id", get(library_prep::get_protocol))
        .route("/api/library-prep/protocols/:id", put(library_prep::update_protocol))
        .route("/api/library-prep/preparations", get(library_prep::list_library_preps))
        .route("/api/library-prep/preparations", post(library_prep::create_library_prep))
        .route("/api/library-prep/preparations/:id", get(library_prep::get_library_prep))
        .route("/api/library-prep/preparations/:id", put(library_prep::update_library_prep))
        .route("/api/library-prep/preparations/:id/complete", post(library_prep::complete_library_prep))
        .route("/api/library-prep/stats", get(library_prep::get_library_prep_stats))
        .route("/api/library-prep/search", get(library_prep::search_library_preps))
}

/// Quality control routes
pub fn qc_routes() -> Router<AppComponents> {
    Router::new()
        .route("/api/qc/dashboard", get(qc::get_qc_dashboard))
        .route("/api/qc/reviews", get(qc::list_qc_reviews))
        .route("/api/qc/reviews", post(qc::create_qc_review))
        .route("/api/qc/reviews/:id", get(qc::get_qc_review))
        .route("/api/qc/reviews/:id/complete", post(qc::complete_qc_review))
        .route("/api/qc/library-prep", post(qc::create_library_prep_qc))
        .route("/api/qc/library-prep/:id", get(qc::get_library_prep_qc))
        .route("/api/qc/metrics/trends", get(qc::get_qc_metric_trends))
        .route("/api/qc/metrics/recent", get(qc::get_recent_qc_metrics))
        .route("/api/qc/metrics/definitions", get(qc::list_qc_metrics))
        .route("/api/qc/metrics/definitions", post(qc::upsert_qc_metric))
        .route("/api/qc/control-samples", get(qc::list_control_samples))
        .route("/api/qc/control-samples", post(qc::create_control_sample))
        .route("/api/qc/control-samples/results", post(qc::record_control_result))
        .route("/api/qc/control-samples/:id/results", get(qc::get_control_results))
}

/// Flow cell design routes
pub fn flow_cell_routes() -> Router<AppComponents> {
    Router::new()
        .route("/api/flow-cells/types", get(flow_cell::list_flow_cell_types))
        .route("/api/flow-cells/types/:id/stats", get(flow_cell::get_flow_cell_type_stats))
        .route("/api/flow-cells/designs", get(flow_cell::list_flow_cell_designs))
        .route("/api/flow-cells/designs", post(flow_cell::create_flow_cell_design))
        .route("/api/flow-cells/designs/:id", get(flow_cell::get_flow_cell_design))
        .route("/api/flow-cells/designs/:id", put(flow_cell::update_flow_cell_design))
        .route("/api/flow-cells/designs/:id", delete(flow_cell::delete_flow_cell_design))
        .route("/api/flow-cells/designs/:id/approve", post(flow_cell::approve_flow_cell_design))
        .route("/api/flow-cells/designs/:id/lanes", get(flow_cell::get_flow_cell_lanes))
        .route("/api/flow-cells/designs/:design_id/lanes/:lane_number", put(flow_cell::update_flow_cell_lane))
        .route("/api/flow-cells/optimize", post(flow_cell::optimize_flow_cell_design))
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
        .merge(project_routes())
        .merge(library_prep_routes())
        .merge(qc_routes())
        .merge(flow_cell_routes())
}

/// Assemble all routes into a complete application router
pub fn create_app_router() -> Router<AppComponents> {
    // Check if proxy mode is enabled
    if proxy_handlers::is_proxy_mode_enabled() {
        tracing::info!("🔄 Creating router in PROXY MODE - routing to microservices");
        return create_proxy_app_router();
    }
    
    tracing::info!("📦 Creating router in MONOLITH MODE - using local services");
    create_monolith_app_router()
}

/// Create router for proxy mode (routes to microservices)
fn create_proxy_app_router() -> Router<AppComponents> {
    Router::new()
        // Health check routes (always local)
        .route("/health", get(health::health_check))
        .route("/health/system", get(health::system_health_check))
        .route("/health/database", get(health::database_health_check))
        .route("/health/metrics", get(health::application_metrics))
        .route("/health/ready", get(health::readiness_check))
        .route("/health/live", get(health::liveness_check))
        // Proxy routes to microservices
        .merge(create_proxy_routes())
        // Dashboard routes (still local)
        .route("/dashboard/stats", get(dashboard::get_dashboard_stats))
        .route("/api/dashboard/stats", get(dashboard::get_dashboard_stats))
        // CORS layer
        .layer(create_cors_layer())
}

/// Create router for monolith mode (uses local services)
fn create_monolith_app_router() -> Router<AppComponents> {
    Router::new()
        // Health check routes (no authentication required for monitoring)
        .route("/health", get(health::health_check))
        .route("/health/system", get(health::system_health_check))
        .route("/health/database", get(health::database_health_check))
        .route("/health/metrics", get(health::application_metrics))
        .route("/health/ready", get(health::readiness_check))
        .route("/health/live", get(health::liveness_check))
        // Public authentication routes (both /auth and /api/auth for compatibility)
        .route("/auth/login", post(users::login))
        .route("/auth/logout", post(users::logout))
        .route("/auth/reset-password", post(users::reset_password))
        .route("/api/auth/login", post(users::login))
        .route("/api/auth/logout", post(users::logout))
        .route("/api/auth/reset-password", post(users::reset_password))
        // Dashboard routes (require authentication)
        .route("/dashboard/stats", get(dashboard::get_dashboard_stats))
        .route("/api/dashboard/stats", get(dashboard::get_dashboard_stats))
        // API routes for frontend compatibility
        .route("/api/samples", get(samples::list_samples))
        .route("/api/samples", post(samples::create_sample))
        .route("/api/samples/:id", get(samples::get_sample))
        .route("/api/samples/:id", put(samples::update_sample))
        .route("/api/templates", get(templates::list_templates))
        .route("/api/templates", post(templates::create_template))
        .route("/api/templates/:id", get(templates::get_template))
        .route("/api/templates/:id", put(templates::update_template))
        .route(
            "/api/sequencing/jobs",
            get(sequencing::list_sequencing_jobs),
        )
        .route(
            "/api/sequencing/jobs",
            post(sequencing::create_sequencing_job),
        )
        .route(
            "/api/sequencing/jobs/:id",
            get(sequencing::get_sequencing_job),
        )
        // User management API routes
        .route("/api/users/me", get(users::get_current_user))
        .route("/api/users/me", put(users::update_current_user))
        .route("/api/users", get(users::list_users))
        .route("/api/users", post(users::create_user))
        .route("/api/users/:user_id", get(users::get_user))
        .route("/api/users/:user_id", put(users::update_user))
        .route("/api/users/:user_id", delete(users::delete_user))
        // RAG API routes
        .route("/api/rag/submissions", get(rag_proxy::get_rag_submissions))
        .route("/api/rag/process", post(rag_proxy::process_rag_document))
        .route("/api/rag/stats", get(rag_proxy::get_rag_stats))
        .route("/api/rag/health", get(rag_proxy::get_rag_health))
        // Spreadsheet API routes
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
            "/api/spreadsheets/upload",
            post(spreadsheets::upload_spreadsheet),
        )
        // Storage API routes - temporarily disabled
        // Reports API routes
        .route("/api/reports/templates", get(reports::get_report_templates))
        .route("/api/reports/schema", get(reports::get_schema))
        .route("/api/reports/execute", post(reports::execute_report))
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
        // Storage management routes - temporarily disabled
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
        // Merge new feature routes
        .merge(project_routes())
        .merge(library_prep_routes())
        .merge(qc_routes())
        .merge(flow_cell_routes())
        // CORS layer
        .layer(create_cors_layer())
}

/// Create CORS layer for the application
fn create_cors_layer() -> CorsLayer {
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
        .allow_credentials(true)
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
