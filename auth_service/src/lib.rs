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
pub use error::{AuthError, AuthResult};
pub use models::*;
pub use services::AuthServiceImpl;

// Application state shared across handlers
#[derive(Debug, Clone)]
pub struct AppState {
    pub auth_service: Arc<services::AuthServiceImpl>,
    pub config: Arc<Config>,
    pub db_pool: DatabasePool,
}

impl Default for AppState {
    fn default() -> Self {
        let config = Config::test_config();
        let db_pool = DatabasePool {
            pool: sqlx::PgPool::connect_lazy("postgres://localhost/test").unwrap(),
        };

        let auth_service = AuthServiceImpl::new(db_pool.clone(), config.clone())
            .expect("Failed to create auth service");

        Self {
            db_pool,
            config: Arc::new(config),
            auth_service: Arc::new(auth_service),
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
                    "postgres://postgres:postgres@localhost:5432/auth_service_test".to_string()
                });

                let pool = DatabasePool::new(&database_url)
                    .await
                    .expect("Failed to create test database pool");

                pool.migrate().await.expect("Failed to run test migrations");

                pool.pool
            })
            .await
    }

    pub async fn create_test_app_state() -> AppState {
        let pg_pool = get_test_db().await.clone();
        let db_pool = DatabasePool { pool: pg_pool };
        let config = Config::test_config();

        let auth_service = AuthServiceImpl::new(db_pool.clone(), config.clone())
            .expect("Failed to create auth service");

        AppState {
            db_pool,
            config: Arc::new(config),
            auth_service: Arc::new(auth_service),
        }
    }

    pub async fn cleanup_test_user(pool: &PgPool, user_id: uuid::Uuid) {
        let _ = sqlx::query("DELETE FROM users WHERE id = $1")
            .bind(user_id)
            .execute(pool)
            .await;
    }
}
