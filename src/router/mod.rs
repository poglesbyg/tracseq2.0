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
        .route("/api/samples", post(handlers::create_sample))
        .route("/api/samples", get(handlers::list_samples))
        .route("/api/samples/:id", get(handlers::get_sample))
        .route("/api/samples/:id", put(handlers::update_sample))
        .route("/api/samples/batch", post(handlers::create_samples_batch))
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
    Router::new()
        // Public authentication routes
        .route("/api/auth/login", post(handlers::login))
        .route(
            "/api/auth/reset-password",
            post(handlers::request_password_reset),
        )
        .route(
            "/api/auth/confirm-reset",
            post(handlers::confirm_password_reset),
        )
        // Protected user routes (authentication handled in handlers)
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
        .route("/api/users/:user_id", delete(handlers::delete_user))
}

/// Assemble all routes into a complete application router
pub fn create_app_router() -> Router<AppComponents> {
    tracing::info!("🔧 Creating application router");

    let router = Router::new()
        .merge({
            tracing::info!("📋 Merging health routes");
            health_routes()
        })
        .merge({
            tracing::info!("📝 Merging template routes");
            template_routes()
        })
        .merge({
            tracing::info!("🤖 Merging RAG proxy routes");
            rag_proxy_routes()
        })
        .merge({
            tracing::info!("🧪 Merging sample routes");
            sample_routes()
        })
        .merge({
            tracing::info!("🧬 Merging sequencing routes");
            sequencing_routes()
        })
        .merge({
            tracing::info!("📦 Merging storage routes");
            storage_routes()
        })
        .merge({
            tracing::info!("📊 Merging reports routes");
            reports_routes()
        })
        .merge({
            tracing::info!("📈 About to merge spreadsheet routes");
            let spreadsheet_router = spreadsheet_routes();
            tracing::info!("📈 Spreadsheet routes ready for merge");
            spreadsheet_router
        })
        .merge({
            tracing::info!("👤 Merging user routes");
            user_routes()
        })
        .layer(CorsLayer::permissive());

    tracing::info!("✅ Application router created successfully");
    router
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
