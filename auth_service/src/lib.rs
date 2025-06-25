pub mod config;
pub mod database;
pub mod error;
pub mod handlers;
pub mod middleware;
pub mod models;
pub mod services;

// Re-exports for convenient testing access
pub use config::Config;
pub use database::DatabasePool;
pub use error::{AuthError, AuthResult};
pub use models::*;
pub use services::AuthServiceImpl;

// Application state shared across handlers
#[derive(Clone)]
pub struct AppState {
    pub auth_service: services::AuthServiceImpl,
    pub config: Config,
    pub db_pool: DatabasePool,
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
        let config = Config::from_env().unwrap_or_else(|_| Config {
            server: crate::config::ServerConfig {
                host: "localhost".to_string(),
                port: 8080,
                workers: None,
            },
            database_url: "postgres://test:test@localhost/test".to_string(),
            jwt: crate::config::JwtConfig {
                secret: "test-secret-key-32-characters-long".to_string(),
                access_token_expiry_hours: 1,
                refresh_token_expiry_days: 30,
                issuer: "test".to_string(),
                audience: "test".to_string(),
            },
            security: crate::config::SecurityConfig {
                password_min_length: 8,
                password_require_uppercase: true,
                password_require_lowercase: true,
                password_require_numbers: true,
                password_require_symbols: false,
                max_login_attempts: 5,
                lockout_duration_minutes: 15,
                session_timeout_hours: 8,
                rate_limiting_enabled: false,
                rate_limit_requests_per_minute: 60,
            },
            email: crate::config::EmailConfig {
                smtp_host: "localhost".to_string(),
                smtp_port: 587,
                smtp_username: "".to_string(),
                smtp_password: "".to_string(),
                from_address: "test@test.com".to_string(),
                from_name: "Test".to_string(),
                enabled: false,
            },
            redis: None,
            logging: crate::config::LoggingConfig {
                level: "debug".to_string(),
                format: "json".to_string(),
                file_enabled: false,
                file_path: None,
            },
            features: crate::config::FeatureConfig {
                registration_enabled: true,
                email_verification_required: false,
                password_reset_enabled: true,
                shibboleth_enabled: false,
                audit_logging_enabled: true,
            },
        });

        let auth_service = AuthServiceImpl::new(db_pool.clone(), config.clone())
            .expect("Failed to create auth service");

        AppState {
            db_pool,
            config,
            auth_service,
        }
    }

    pub fn cleanup_test_user(pool: PgPool, user_id: uuid::Uuid) {
        tokio::spawn(async move {
            let _ = sqlx::query("DELETE FROM users WHERE id = $1")
                .bind(user_id)
                .execute(&pool)
                .await;
        });
    }
}
