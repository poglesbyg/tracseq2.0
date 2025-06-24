pub mod config;
pub mod database;
pub mod error;
pub mod handlers;
pub mod middleware;
pub mod models;
pub mod services;
pub mod clients;

// Re-exports for convenient testing access
pub use config::Config;
pub use database::{create_pool, run_migrations, DatabasePool};
pub use error::{SampleServiceError, SampleResult};
pub use models::*;
pub use services::SampleService;

// Application state for testing
pub use crate::AppState;

// Test utilities
#[cfg(test)]
pub mod test_utils {
    use super::*;
    use sqlx::PgPool;
    use std::sync::Arc;
    use tokio::sync::OnceCell;

    static TEST_DATABASE: OnceCell<PgPool> = OnceCell::const_new();

    pub async fn get_test_db() -> &'static PgPool {
        TEST_DATABASE.get_or_init(|| async {
            let database_url = std::env::var("TEST_DATABASE_URL")
                .unwrap_or_else(|_| "postgres://postgres:postgres@localhost:5432/sample_service_test".to_string());
            
            let pool = create_pool(&database_url).await
                .expect("Failed to create test database pool");
            
            run_migrations(&pool).await
                .expect("Failed to run test migrations");
            
            pool
        }).await
    }

    pub async fn create_test_app_state() -> AppState {
        let pool = get_test_db().await.clone();
        let config = Config::test_config();
        let sample_service = SampleService::new(pool.clone());
        
        AppState {
            db_pool: DatabasePool { pool },
            config: Arc::new(config),
            sample_service: Arc::new(sample_service),
        }
    }

    pub fn cleanup_test_sample(pool: &PgPool, sample_id: uuid::Uuid) {
        tokio::spawn(async move {
            let _ = sqlx::query("DELETE FROM samples WHERE id = $1")
                .bind(sample_id)
                .execute(pool)
                .await;
        });
    }
} 
