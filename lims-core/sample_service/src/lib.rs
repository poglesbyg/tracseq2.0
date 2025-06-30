pub mod clients;
pub mod config;
pub mod database;
pub mod error;
pub mod handlers;
pub mod middleware;
pub mod models;
pub mod services;

use std::sync::Arc;

// Re-exports for convenient testing access
pub use config::Config;
pub use database::{DatabasePool, create_pool, run_migrations};
pub use error::{SampleResult, SampleServiceError};
pub use models::*;
pub use services::SampleServiceImpl;

// Type alias for service trait
pub type SampleService = SampleServiceImpl;

/// Application state containing shared services and configuration
#[derive(Debug, Clone)]
pub struct AppState {
    pub db_pool: DatabasePool,
    pub config: Arc<Config>,
    pub sample_service: Arc<SampleServiceImpl>,
    pub auth_client: Arc<clients::AuthClient>,
    pub storage_client: Arc<clients::StorageClient>,
}

impl Default for AppState {
    fn default() -> Self {
        let config = Config::test_config();
        let db_pool = DatabasePool {
            pool: sqlx::PgPool::connect_lazy("postgres://localhost/test").unwrap(),
        };
        let auth_client = clients::AuthClient::new("http://localhost:8001".to_string());
        let storage_client = clients::StorageClient::new("http://localhost:8002".to_string());

        let sample_service = SampleServiceImpl::new(
            db_pool.clone(),
            config.clone(),
            auth_client.clone(),
            storage_client.clone(),
        )
        .expect("Failed to create sample service");

        Self {
            db_pool,
            config: Arc::new(config),
            sample_service: Arc::new(sample_service),
            auth_client: Arc::new(auth_client),
            storage_client: Arc::new(storage_client),
        }
    }
}

// Test utilities
#[cfg(test)]
pub mod test_utils {
    use super::*;
    use sqlx::PgPool;
    use std::sync::Arc;
    use tokio::sync::OnceCell;

    static TEST_DATABASE: OnceCell<PgPool> = OnceCell::const_new();

    pub async fn get_test_db() -> &'static PgPool {
        TEST_DATABASE
            .get_or_init(|| async {
                let database_url = std::env::var("TEST_DATABASE_URL").unwrap_or_else(|_| {
                    "postgres://postgres:postgres@localhost:5432/sample_service_test".to_string()
                });

                let pool = create_pool(&database_url)
                    .await
                    .expect("Failed to create test database pool");

                run_migrations(&pool)
                    .await
                    .expect("Failed to run test migrations");

                pool
            })
            .await
    }

    pub async fn create_test_app_state() -> AppState {
        let pool = get_test_db().await.clone();
        let config = Config::test_config();
        let db_pool = DatabasePool { pool: pool.clone() };

        // Create mock clients
        let auth_client = crate::clients::AuthClient::new("http://localhost:8001".to_string());
        let storage_client =
            crate::clients::StorageClient::new("http://localhost:8002".to_string());

        let sample_service = SampleServiceImpl::new(
            db_pool.clone(),
            config.clone(),
            auth_client.clone(),
            storage_client.clone(),
        )
        .expect("Failed to create sample service");

        AppState {
            db_pool,
            config: Arc::new(config),
            sample_service: Arc::new(sample_service),
            auth_client: Arc::new(auth_client),
            storage_client: Arc::new(storage_client),
        }
    }

    pub async fn cleanup_test_sample(pool: &PgPool, sample_id: uuid::Uuid) {
        let _ = sqlx::query("DELETE FROM samples WHERE id = $1")
            .bind(sample_id)
            .execute(pool)
            .await;
    }
}
