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

/// Create the application router with all routes and middleware
pub fn create_router(state: AppState) -> axum::Router {
    use axum::{Router, routing};
    use tower::ServiceBuilder;
    use tower_http::{cors::CorsLayer, trace::TraceLayer};
    
    // Health check routes (no auth required)
    let health_routes = Router::new()
        .route("/health", routing::get(handlers::health::health_check))
        .route("/health/ready", routing::get(handlers::health::readiness_check))
        .route("/health/metrics", routing::get(handlers::health::metrics));

    // Authentication routes (public)
    let auth_routes = Router::new()
        .route("/auth/login", routing::post(handlers::auth::login))
        .route("/auth/register", routing::post(handlers::auth::register))
        .route("/auth/refresh", routing::post(handlers::auth::refresh_token))
        .route("/auth/forgot-password", routing::post(handlers::auth::forgot_password))
        .route("/auth/reset-password", routing::post(handlers::auth::reset_password))
        .route("/auth/verify-email", routing::post(handlers::auth::verify_email));

    // Protected routes (require authentication)
    let protected_routes = Router::new()
        .route("/auth/logout", routing::post(handlers::auth::logout))
        .route("/auth/me", routing::get(handlers::auth::get_current_user))
        .route("/auth/sessions", routing::get(handlers::auth::get_sessions))
        .route("/auth/sessions/:session_id", routing::delete(handlers::auth::revoke_session))
        // TODO: Fix handler type issue with change_password
        // .route("/auth/change-password", routing::put(handlers::auth::change_password))
        .layer(axum::middleware::from_fn_with_state(
            state.clone(),
            middleware::auth_middleware,
        ));

    // Token validation routes (for other services)
    let validation_routes = Router::new()
        .route("/validate/token", routing::post(handlers::validation::validate_token))
        .route("/validate/permissions", routing::post(handlers::validation::validate_permissions))
        .layer(axum::middleware::from_fn_with_state(
            state.clone(),
            middleware::service_auth_middleware,
        ));

    // Admin routes (require admin privileges)
    let admin_routes = Router::new()
        .route("/admin/users", routing::get(handlers::admin::list_users))
        .route("/admin/users/:user_id", routing::get(handlers::admin::get_user))
        .route("/admin/users/:user_id", routing::delete(handlers::admin::delete_user))
        .route("/admin/users/:user_id/disable", routing::post(handlers::admin::disable_user))
        .route("/admin/users/:user_id/enable", routing::post(handlers::admin::enable_user))
        .route("/admin/sessions", routing::get(handlers::admin::list_sessions))
        .route("/admin/audit-log", routing::get(handlers::admin::get_audit_log))
        .layer(axum::middleware::from_fn_with_state(
            state.clone(),
            middleware::admin_middleware,
        ));

    // Combine all routes
    Router::new()
        .merge(health_routes)
        .merge(auth_routes)
        .merge(protected_routes)
        .merge(validation_routes)
        .merge(admin_routes)
        .layer(
            ServiceBuilder::new()
                .layer(TraceLayer::new_for_http())
                .layer(CorsLayer::permissive())
        )
        .with_state(state)
}

// Test utilities - accessible to both unit and integration tests
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
