use axum::{
    routing::{delete, get, post, put},
    Router,
};

use crate::{
    assembly::AppComponents,
    handlers::proxy_handlers::{
        check_microservices_health, initialize_proxy, is_proxy_mode_enabled,
        list_microservices, proxy_auth_request, proxy_create_sample, proxy_list_samples,
        proxy_login, proxy_logout, proxy_sample_request, proxy_sequencing_request,
        proxy_spreadsheet_request, proxy_storage_request, proxy_template_request,
        service_discovery,
    },
};

/// Create routes that proxy to microservices
pub fn create_proxy_routes() -> Router<AppComponents> {
    Router::new()
        // Service discovery and health
        .route("/api/services", get(list_microservices))
        .route("/api/services/health", get(check_microservices_health))
        .route("/api/services/discovery", get(service_discovery))
        
        // Auth service proxy routes
        .route("/api/auth/login", post(proxy_login))
        .route("/api/auth/logout", post(proxy_logout))
        .route("/api/auth/*path", post(proxy_auth_request))
        .route("/api/auth/*path", get(proxy_auth_request))
        .route("/api/auth/*path", put(proxy_auth_request))
        .route("/api/auth/*path", delete(proxy_auth_request))
        
        // Sample service proxy routes
        .route("/api/samples", post(proxy_create_sample))
        .route("/api/samples", get(proxy_list_samples))
        .route("/api/samples/*path", get(proxy_sample_request))
        .route("/api/samples/*path", post(proxy_sample_request))
        .route("/api/samples/*path", put(proxy_sample_request))
        .route("/api/samples/*path", delete(proxy_sample_request))
        
        // Sequencing service proxy routes
        .route("/api/sequencing/*path", get(proxy_sequencing_request))
        .route("/api/sequencing/*path", post(proxy_sequencing_request))
        .route("/api/sequencing/*path", put(proxy_sequencing_request))
        .route("/api/sequencing/*path", delete(proxy_sequencing_request))
        
        // Template service proxy routes
        .route("/api/templates/*path", get(proxy_template_request))
        .route("/api/templates/*path", post(proxy_template_request))
        .route("/api/templates/*path", put(proxy_template_request))
        .route("/api/templates/*path", delete(proxy_template_request))
        
        // Storage service proxy routes
        .route("/api/storage/*path", get(proxy_storage_request))
        .route("/api/storage/*path", post(proxy_storage_request))
        .route("/api/storage/*path", put(proxy_storage_request))
        .route("/api/storage/*path", delete(proxy_storage_request))
        
        // Spreadsheet service proxy routes
        .route("/api/spreadsheets/*path", get(proxy_spreadsheet_request))
        .route("/api/spreadsheets/*path", post(proxy_spreadsheet_request))
        .route("/api/spreadsheets/*path", put(proxy_spreadsheet_request))
        .route("/api/spreadsheets/*path", delete(proxy_spreadsheet_request))
}

/// Initialize proxy system
pub async fn init_proxy_system() {
    if is_proxy_mode_enabled() {
        tracing::info!("🔄 Proxy mode enabled - initializing microservice proxies");
        initialize_proxy().await;
        tracing::info!("✅ Proxy system initialized successfully");
    } else {
        tracing::info!("📦 Monolith mode - using local service implementations");
    }
}