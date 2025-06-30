// Sequencing Service Library
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
pub use clients::{AuthClient, NotificationClient, SampleClient, StorageClient, TemplateClient};
pub use config::Config;
pub use database::{DatabasePool, create_pool, run_migrations};
pub use error::{Result, SequencingError};
pub use services::SequencingServiceImpl;

/// Application state shared across handlers - Arc-wrapped for axum-test
#[derive(Clone)]
pub struct AppState {
    pub sequencing_service: Arc<SequencingServiceImpl>,
    pub config: Arc<Config>,
    pub db_pool: DatabasePool,
    pub auth_client: Arc<AuthClient>,
    pub sample_client: Arc<SampleClient>,
    pub notification_client: Arc<NotificationClient>,
    pub template_client: Arc<TemplateClient>,
    pub storage_client: Arc<StorageClient>,
}

impl Default for AppState {
    fn default() -> Self {
        let config = Config::test_config();
        let db_pool = DatabasePool {
            pool: sqlx::PgPool::connect_lazy("postgres://localhost/test").unwrap(),
        };

        let auth_client = AuthClient::new("http://localhost:8001".to_string());
        let sample_client = SampleClient::new("http://localhost:8002".to_string());
        let notification_client = NotificationClient::new("http://localhost:8004".to_string());
        let template_client = TemplateClient::new("http://localhost:8003".to_string());
        let storage_client = StorageClient::new("http://localhost:8005".to_string());

        let sequencing_service = SequencingServiceImpl::new(
            db_pool.clone(),
            config.clone(),
            auth_client.clone(),
            sample_client.clone(),
            notification_client.clone(),
            template_client.clone(),
        )
        .expect("Failed to create sequencing service");

        Self {
            db_pool,
            config: Arc::new(config),
            sequencing_service: Arc::new(sequencing_service),
            auth_client: Arc::new(auth_client),
            sample_client: Arc::new(sample_client),
            notification_client: Arc::new(notification_client),
            template_client: Arc::new(template_client),
            storage_client: Arc::new(storage_client),
        }
    }
}

/// Create the application router with all routes and middleware
pub fn create_app(state: AppState) -> axum::Router {
    use axum::middleware as axum_middleware;
    use axum::routing::{delete, get, post, put};
    use tower::ServiceBuilder;
    use tower_http::{cors::CorsLayer, trace::TraceLayer};

    // Health check routes (no auth required)
    let health_routes = axum::Router::new()
        .route("/health", get(handlers::health::health_check))
        .route("/health/ready", get(handlers::health::readiness_check))
        .route("/health/metrics", get(handlers::health::metrics));

    // Sequencing job management routes
    let job_routes = axum::Router::new()
        .route("/jobs", post(handlers::jobs::create_job))
        .route("/jobs", get(handlers::jobs::list_jobs))
        .route("/jobs/:job_id", get(handlers::jobs::get_job))
        .route("/jobs/:job_id", put(handlers::jobs::update_job))
        .route("/jobs/:job_id", delete(handlers::jobs::delete_job))
        .route(
            "/jobs/:job_id/status",
            put(handlers::jobs::update_job_status),
        )
        .route("/jobs/:job_id/clone", post(handlers::jobs::clone_job))
        .route("/jobs/:job_id/cancel", post(handlers::jobs::cancel_job))
        .layer(axum_middleware::from_fn_with_state(
            state.clone(),
            middleware::auth_middleware,
        ));

    // Workflow management routes
    let workflow_routes = axum::Router::new()
        .route("/workflows", get(handlers::workflows::list_workflows))
        .route(
            "/workflows/:workflow_id",
            get(handlers::workflows::get_workflow),
        )
        .route(
            "/workflows/:workflow_id/execute",
            post(handlers::workflows::execute_workflow),
        )
        .layer(axum_middleware::from_fn_with_state(
            state.clone(),
            middleware::auth_middleware,
        ));

    // Sample sheet management routes
    let sample_sheet_routes = axum::Router::new()
        .route(
            "/sample-sheets",
            post(handlers::sample_sheets::create_sample_sheet),
        )
        .route(
            "/sample-sheets",
            get(handlers::sample_sheets::list_sample_sheets),
        )
        .route(
            "/sample-sheets/:sheet_id",
            get(handlers::sample_sheets::get_sample_sheet),
        )
        .route(
            "/sample-sheets/:sheet_id",
            put(handlers::sample_sheets::update_sample_sheet),
        )
        .route(
            "/sample-sheets/:sheet_id",
            delete(handlers::sample_sheets::delete_sample_sheet),
        )
        .route(
            "/sample-sheets/:sheet_id/validate",
            post(handlers::sample_sheets::validate_sample_sheet),
        )
        .layer(axum_middleware::from_fn_with_state(
            state.clone(),
            middleware::auth_middleware,
        ));

    // Combine basic routes with middleware
    axum::Router::new()
        .merge(health_routes)
        .merge(job_routes)
        .merge(workflow_routes)
        .merge(sample_sheet_routes)
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
        let notification_client = NotificationClient::new("http://localhost:8004".to_string());
        let template_client = TemplateClient::new("http://localhost:8003".to_string());
        let storage_client = StorageClient::new("http://localhost:8005".to_string());

        let sequencing_service = SequencingServiceImpl::new(
            db_pool.clone(),
            config.clone(),
            auth_client.clone(),
            sample_client.clone(),
            notification_client.clone(),
            template_client.clone(),
        )
        .expect("Failed to create sequencing service");

        AppState {
            db_pool,
            config: Arc::new(config),
            sequencing_service: Arc::new(sequencing_service),
            auth_client: Arc::new(auth_client),
            sample_client: Arc::new(sample_client),
            notification_client: Arc::new(notification_client),
            template_client: Arc::new(template_client),
            storage_client: Arc::new(storage_client),
        }
    }

    pub async fn cleanup_test_job(pool: &PgPool, job_id: uuid::Uuid) {
        let _ = sqlx::query("DELETE FROM sequencing_jobs WHERE id = $1")
            .bind(job_id)
            .execute(pool)
            .await;
    }
}
