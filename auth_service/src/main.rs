use anyhow::Result;
use axum::{
    routing::get,
    Router,
};
use std::net::SocketAddr;
use tracing::info;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

mod config;
mod database;
mod error;
mod handlers;
mod models;
mod services;
mod middleware;

use config::Config;
use database::DatabasePool;
use services::AuthServiceImpl;

#[tokio::main]
async fn main() -> Result<()> {
    println!("AUTH SERVICE: Starting main function");
    
    // Initialize tracing
    println!("AUTH SERVICE: Initializing tracing");
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "auth_service=debug,tower_http=debug".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    println!("AUTH SERVICE: Tracing initialized");
    info!("Starting Authentication Service");

    // Load configuration
    println!("AUTH SERVICE: Loading configuration");
    let config = match Config::from_env() {
        Ok(config) => {
            println!("AUTH SERVICE: Configuration loaded successfully");
            config
        }
        Err(e) => {
            println!("AUTH SERVICE: Failed to load configuration: {e}");
            return Err(e);
        }
    };
    info!("Configuration loaded successfully");

    // Setup database connection
    println!("AUTH SERVICE: Setting up database connection to: {}", config.database_url);
    let db_pool = match DatabasePool::new(&config.database_url).await {
        Ok(pool) => {
            println!("AUTH SERVICE: Database connection established");
            pool
        }
        Err(e) => {
            println!("AUTH SERVICE: Failed to establish database connection: {e}");
            return Err(e);
        }
    };
    info!("Database connection established");

    // Run database migrations
    println!("AUTH SERVICE: Running database migrations");
    if let Err(e) = db_pool.migrate().await {
        println!("AUTH SERVICE: Database migration failed: {e}");
        return Err(e);
    }
    println!("AUTH SERVICE: Database migrations completed");
    info!("Database migrations completed");

    // Initialize authentication service
    println!("AUTH SERVICE: Initializing authentication service");
    let auth_service = match AuthServiceImpl::new(db_pool.clone(), config.clone()) {
        Ok(service) => {
            println!("AUTH SERVICE: Authentication service initialized");
            service
        }
        Err(e) => {
            println!("AUTH SERVICE: Failed to initialize authentication service: {e}");
            return Err(e);
        }
    };
    info!("Authentication service initialized");

    // Setup application state
    println!("AUTH SERVICE: Setting up application state");
    let app_state = AppState {
        auth_service,
        config: config.clone(),
        db_pool,
    };

    // Build the application router
    println!("AUTH SERVICE: Building application router");
    let app = create_app(app_state);

    // Start the server
    let addr = SocketAddr::from(([0, 0, 0, 0], config.server.port));
    println!("AUTH SERVICE: Starting server on {addr}");
    info!("Authentication service listening on {addr}");

    let listener = match tokio::net::TcpListener::bind(addr).await {
        Ok(listener) => {
            println!("AUTH SERVICE: Successfully bound to address {addr}");
            listener
        }
        Err(e) => {
            println!("AUTH SERVICE: Failed to bind to address {addr}: {e}");
            return Err(e.into());
        }
    };

    println!("AUTH SERVICE: About to start serving requests");
    if let Err(e) = axum::serve(listener, app).await {
        println!("AUTH SERVICE: Server failed with error: {e}");
        return Err(e.into());
    }

    println!("AUTH SERVICE: Server exited normally");
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
fn create_app(_state: AppState) -> Router {
    // Simplified router for testing
    Router::new()
        .route("/health", get(|| async { "OK" }))
}

/*
// Full router implementation - commented out for debugging
fn create_app_full(state: AppState) -> Router {
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
        .layer(axum::middleware::from_fn_with_state(
            state.clone(),
            crate::middleware::auth_middleware,
        ));

    // Token validation routes (for other services)
    let validation_routes = Router::new()
        .route("/validate/token", post(handlers::validation::validate_token))
        .route("/validate/permissions", post(handlers::validation::validate_permissions))
        .layer(axum::middleware::from_fn_with_state(
            state.clone(),
            crate::middleware::service_auth_middleware,
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
        .layer(axum::middleware::from_fn_with_state(
            state.clone(),
            crate::middleware::admin_middleware,
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
                .layer(CorsLayer::permissive()) // Configure CORS as needed
        )
        .with_state(state)
}
*/ 
