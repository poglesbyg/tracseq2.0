use anyhow::Result;
use axum::{
    middleware,
    routing::{get, post, delete},
    Router,
};
use std::net::SocketAddr;
use tower::ServiceBuilder;
use tower_http::{
    cors::CorsLayer,
    trace::TraceLayer,
    compression::CompressionLayer,
};
use tracing::{info, warn};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

mod config;
mod database;
mod error;
mod handlers;
mod models;
mod services;
mod middleware as auth_middleware;

use config::Config;
use database::DatabasePool;
use services::AuthServiceImpl;

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize tracing
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "auth_service=debug,tower_http=debug".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    info!("Starting Authentication Service");

    // Load configuration
    let config = Config::from_env()?;
    info!("Configuration loaded successfully");

    // Setup database connection
    let db_pool = DatabasePool::new(&config.database_url).await?;
    info!("Database connection established");

    // Run database migrations
    db_pool.migrate().await?;
    info!("Database migrations completed");

    // Initialize authentication service
    let auth_service = AuthServiceImpl::new(db_pool.clone(), config.clone())?;
    info!("Authentication service initialized");

    // Setup application state
    let app_state = AppState {
        auth_service,
        config: config.clone(),
        db_pool,
    };

    // Build the application router
    let app = create_app(app_state);

    // Start the server
    let addr = SocketAddr::from(([0, 0, 0, 0], config.server.port));
    info!("Authentication service listening on {}", addr);

    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}

/// Application state shared across handlers
#[derive(Clone)]
pub struct AppState {
    pub auth_service: AuthServiceImpl,
    pub config: Config,
    pub db_pool: DatabasePool,
}

/// Create the application router with all routes and middleware
fn create_app(state: AppState) -> Router {
    // Health check routes (no auth required)
    let health_routes = Router::new()
        .route("/health", get(handlers::health::health_check))
        .route("/health/ready", get(handlers::health::readiness_check))
        .route("/health/metrics", get(handlers::health::metrics));

    // Authentication routes (public)
    let auth_routes = Router::new()
        .route("/auth/login", post(handlers::auth::login))
        .route("/auth/register", post(handlers::auth::register))
        .route("/auth/refresh", post(handlers::auth::refresh_token))
        .route("/auth/forgot-password", post(handlers::auth::forgot_password))
        .route("/auth/reset-password", post(handlers::auth::reset_password))
        .route("/auth/verify-email", post(handlers::auth::verify_email));

    // Protected routes (require authentication)
    let protected_routes = Router::new()
        .route("/auth/logout", post(handlers::auth::logout))
        .route("/auth/me", get(handlers::auth::get_current_user))
        .route("/auth/sessions", get(handlers::auth::get_sessions))
        .route("/auth/sessions/:session_id", delete(handlers::auth::revoke_session))
        .route("/auth/change-password", post(handlers::auth::change_password))
        .layer(middleware::from_fn_with_state(
            state.clone(),
            auth_middleware::auth_middleware,
        ));

    // Token validation routes (for other services)
    let validation_routes = Router::new()
        .route("/validate/token", post(handlers::validation::validate_token))
        .route("/validate/permissions", post(handlers::validation::validate_permissions))
        .layer(middleware::from_fn_with_state(
            state.clone(),
            auth_middleware::service_auth_middleware,
        ));

    // Admin routes (require admin privileges)
    let admin_routes = Router::new()
        .route("/admin/users", get(handlers::admin::list_users))
        .route("/admin/users/:user_id", get(handlers::admin::get_user))
        .route("/admin/users/:user_id", delete(handlers::admin::delete_user))
        .route("/admin/users/:user_id/disable", post(handlers::admin::disable_user))
        .route("/admin/users/:user_id/enable", post(handlers::admin::enable_user))
        .route("/admin/sessions", get(handlers::admin::list_sessions))
        .route("/admin/audit-log", get(handlers::admin::get_audit_log))
        .layer(middleware::from_fn_with_state(
            state.clone(),
            auth_middleware::admin_middleware,
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
                .layer(CompressionLayer::new())
                .layer(CorsLayer::permissive()) // Configure CORS as needed
        )
        .with_state(state)
} 
