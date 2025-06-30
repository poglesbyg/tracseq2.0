// Template Service Library
// Provides all public APIs and functionality for testing and external use

pub mod clients;
pub mod config;
pub mod database;
pub mod error;
pub mod handlers;
pub mod middleware;
pub mod models;
pub mod services;

use std::sync::Arc;

// Re-export main types for easy access
pub use clients::{AuthClient, SampleClient};
pub use config::Config;
pub use database::{DatabasePool, create_pool, run_migrations};
pub use error::{TemplateResult, TemplateServiceError};
pub use services::TemplateServiceImpl;

/// Application state shared across handlers - Arc-wrapped for axum-test
#[derive(Clone)]
pub struct AppState {
    pub template_service: Arc<TemplateServiceImpl>,
    pub config: Arc<Config>,
    pub db_pool: DatabasePool,
    pub auth_client: Arc<AuthClient>,
    pub sample_client: Arc<SampleClient>,
}

impl Default for AppState {
    fn default() -> Self {
        let config = Config::test_config();
        let db_pool = DatabasePool {
            pool: sqlx::PgPool::connect_lazy("postgres://localhost/test").unwrap(),
        };

        let auth_client = AuthClient::new("http://localhost:8001".to_string());
        let sample_client = SampleClient::new("http://localhost:8002".to_string());

        let template_service = TemplateServiceImpl::new(
            db_pool.clone(),
            config.clone(),
            auth_client.clone(),
            sample_client.clone(),
        )
        .expect("Failed to create template service");

        Self {
            db_pool,
            config: Arc::new(config),
            template_service: Arc::new(template_service),
            auth_client: Arc::new(auth_client),
            sample_client: Arc::new(sample_client),
        }
    }
}

/// Create the application router with all routes and middleware
pub fn create_app(state: AppState) -> axum::Router {
    use axum::routing::{delete, get, post, put};
    use tower::ServiceBuilder;
    use tower_http::{cors::CorsLayer, trace::TraceLayer};

    // Health check routes (no auth required)
    let health_routes = axum::Router::new()
        .route("/health", get(handlers::health::health_check))
        .route("/health/ready", get(handlers::health::readiness_check))
        .route("/health/metrics", get(handlers::health::metrics));

    // Template CRUD routes (require authentication)
    let template_routes = axum::Router::new()
        .route("/templates", post(handlers::templates::create_template))
        .route("/templates", get(handlers::templates::list_templates))
        .route(
            "/templates/:template_id",
            get(handlers::templates::get_template),
        )
        .route(
            "/templates/:template_id",
            put(handlers::templates::update_template),
        )
        .route(
            "/templates/:template_id",
            delete(handlers::templates::delete_template),
        )
        .route(
            "/templates/:template_id/clone",
            post(handlers::templates::clone_template),
        )
        .layer(axum::middleware::from_fn_with_state(
            state.clone(),
            middleware::auth_middleware,
        ));

    // Template Field CRUD routes (require authentication)
    let field_routes = axum::Router::new()
        .route(
            "/templates/:template_id/fields",
            post(handlers::template_fields::create_field),
        )
        .route(
            "/templates/:template_id/fields",
            get(handlers::template_fields::list_fields),
        )
        .route(
            "/templates/:template_id/fields/:field_id",
            get(handlers::template_fields::get_field),
        )
        .route(
            "/templates/:template_id/fields/:field_id",
            put(handlers::template_fields::update_field),
        )
        .route(
            "/templates/:template_id/fields/:field_id",
            delete(handlers::template_fields::delete_field),
        )
        .layer(axum::middleware::from_fn_with_state(
            state.clone(),
            middleware::auth_middleware,
        ));

    // File upload/download routes
    let file_routes = axum::Router::new()
        .route("/templates/upload", post(handlers::files::upload_template))
        .route(
            "/templates/:template_id/download",
            get(handlers::files::download_template),
        )
        .route(
            "/templates/:template_id/export",
            get(handlers::files::export_template),
        )
        .route("/templates/import", post(handlers::files::import_templates))
        .layer(axum::middleware::from_fn_with_state(
            state.clone(),
            middleware::auth_middleware,
        ));

    // Combine all routes with basic middleware
    axum::Router::new()
        .merge(health_routes)
        .merge(template_routes)
        .merge(field_routes)
        .merge(file_routes)
        .layer(
            ServiceBuilder::new()
                .layer(TraceLayer::new_for_http())
                .layer(CorsLayer::permissive()),
        )
        .with_state(state)
}

// Test utilities
#[cfg(test)]
pub mod test_utils {
    use super::*;
    use sqlx::PgPool;

    pub async fn create_test_app_state() -> AppState {
        let config = Config::test_config();
        let db_pool = DatabasePool {
            pool: sqlx::PgPool::connect_lazy("postgres://localhost/test").unwrap(),
        };

        let auth_client = AuthClient::new("http://localhost:8001".to_string());
        let sample_client = SampleClient::new("http://localhost:8002".to_string());

        let template_service = TemplateServiceImpl::new(
            db_pool.clone(),
            config.clone(),
            auth_client.clone(),
            sample_client.clone(),
        )
        .expect("Failed to create template service");

        AppState {
            db_pool,
            config: Arc::new(config),
            template_service: Arc::new(template_service),
            auth_client: Arc::new(auth_client),
            sample_client: Arc::new(sample_client),
        }
    }

    pub async fn cleanup_test_template(pool: &PgPool, template_id: uuid::Uuid) {
        let _ = sqlx::query("DELETE FROM templates WHERE id = $1")
            .bind(template_id)
            .execute(pool)
            .await;
    }
}
