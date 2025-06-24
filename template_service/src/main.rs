use anyhow::Result;
use axum::{
    middleware,
    routing::{get, post, put, delete},
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
mod middleware;
mod clients;

use config::Config;
use database::DatabasePool;
use services::TemplateServiceImpl;
use clients::{AuthClient, SampleClient};

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize tracing
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "template_service=debug,tower_http=debug".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    info!("Starting Template Management Service");

    // Load configuration
    let config = Config::from_env()?;
    info!("Configuration loaded successfully");

    // Setup database connection
    let db_pool = DatabasePool::new(&config.database_url).await?;
    info!("Database connection established");

    // Run database migrations
    db_pool.migrate().await?;
    info!("Database migrations completed");

    // Initialize external service clients
    let auth_client = AuthClient::new(config.auth_service_url.clone());
    let sample_client = SampleClient::new(config.sample_service_url.clone());

    // Initialize template service
    let template_service = TemplateServiceImpl::new(
        db_pool.clone(),
        config.clone(),
        auth_client.clone(),
        sample_client.clone(),
    )?;
    info!("Template service initialized");

    // Setup application state
    let app_state = AppState {
        template_service,
        config: config.clone(),
        db_pool,
        auth_client,
        sample_client,
    };

    // Build the application router
    let app = create_app(app_state);

    // Start the server
    let addr = SocketAddr::from(([0, 0, 0, 0], config.server.port));
    info!("Template management service listening on {}", addr);

    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}

/// Application state shared across handlers
#[derive(Clone)]
pub struct AppState {
    pub template_service: TemplateServiceImpl,
    pub config: Config,
    pub db_pool: DatabasePool,
    pub auth_client: AuthClient,
    pub sample_client: SampleClient,
}

/// Create the application router with all routes and middleware
fn create_app(state: AppState) -> Router {
    // Health check routes (no auth required)
    let health_routes = Router::new()
        .route("/health", get(handlers::health::health_check))
        .route("/health/ready", get(handlers::health::readiness_check))
        .route("/health/metrics", get(handlers::health::metrics));

    // Template CRUD routes (require authentication)
    let template_routes = Router::new()
        .route("/templates", post(handlers::templates::create_template))
        .route("/templates", get(handlers::templates::list_templates))
        .route("/templates/:template_id", get(handlers::templates::get_template))
        .route("/templates/:template_id", put(handlers::templates::update_template))
        .route("/templates/:template_id", delete(handlers::templates::delete_template))
        .route("/templates/:template_id/clone", post(handlers::templates::clone_template))
        .layer(middleware::from_fn_with_state(
            state.clone(),
            crate::middleware::auth_middleware,
        ));

    // File upload/download routes
    let file_routes = Router::new()
        .route("/templates/upload", post(handlers::files::upload_template))
        .route("/templates/:template_id/download", get(handlers::files::download_template))
        .route("/templates/:template_id/export", get(handlers::files::export_template))
        .route("/templates/import", post(handlers::files::import_templates))
        .layer(middleware::from_fn_with_state(
            state.clone(),
            crate::middleware::auth_middleware,
        ));

    // Template versioning routes
    let version_routes = Router::new()
        .route("/templates/:template_id/versions", get(handlers::versions::list_versions))
        .route("/templates/:template_id/versions", post(handlers::versions::create_version))
        .route("/templates/:template_id/versions/:version", get(handlers::versions::get_version))
        .route("/templates/:template_id/versions/:version", delete(handlers::versions::delete_version))
        .route("/templates/:template_id/versions/:version/restore", post(handlers::versions::restore_version))
        .layer(middleware::from_fn_with_state(
            state.clone(),
            crate::middleware::auth_middleware,
        ));

    // Form builder routes
    let form_routes = Router::new()
        .route("/forms/:template_id/generate", get(handlers::forms::generate_form))
        .route("/forms/:template_id/validate", post(handlers::forms::validate_form_data))
        .route("/forms/:template_id/preview", get(handlers::forms::preview_form))
        .route("/forms/:template_id/render", post(handlers::forms::render_form))
        .layer(middleware::from_fn_with_state(
            state.clone(),
            crate::middleware::auth_middleware,
        ));

    // Field management routes
    let field_routes = Router::new()
        .route("/templates/:template_id/fields", get(handlers::fields::list_fields))
        .route("/templates/:template_id/fields", post(handlers::fields::create_field))
        .route("/templates/:template_id/fields/:field_id", get(handlers::fields::get_field))
        .route("/templates/:template_id/fields/:field_id", put(handlers::fields::update_field))
        .route("/templates/:template_id/fields/:field_id", delete(handlers::fields::delete_field))
        .route("/templates/:template_id/fields/reorder", post(handlers::fields::reorder_fields))
        .layer(middleware::from_fn_with_state(
            state.clone(),
            crate::middleware::auth_middleware,
        ));

    // Validation routes
    let validation_routes = Router::new()
        .route("/templates/:template_id/validation", get(handlers::validation::get_validation_rules))
        .route("/templates/:template_id/validation", post(handlers::validation::create_validation_rule))
        .route("/templates/:template_id/validation/:rule_id", put(handlers::validation::update_validation_rule))
        .route("/templates/:template_id/validation/:rule_id", delete(handlers::validation::delete_validation_rule))
        .route("/templates/:template_id/validate-data", post(handlers::validation::validate_template_data))
        .layer(middleware::from_fn_with_state(
            state.clone(),
            crate::middleware::auth_middleware,
        ));

    // Template integration routes (with sample service)
    let integration_routes = Router::new()
        .route("/integration/samples/create", post(handlers::integration::create_sample_from_template))
        .route("/integration/samples/validate", post(handlers::integration::validate_sample_data))
        .route("/integration/templates/for-samples", get(handlers::integration::get_templates_for_samples))
        .layer(middleware::from_fn_with_state(
            state.clone(),
            crate::middleware::auth_middleware,
        ));

    // Schema routes for template structure management
    let schema_routes = Router::new()
        .route("/schemas", get(handlers::schemas::list_schemas))
        .route("/schemas/:schema_name", get(handlers::schemas::get_schema))
        .route("/templates/:template_id/schema", get(handlers::schemas::get_template_schema))
        .route("/templates/:template_id/schema/validate", post(handlers::schemas::validate_template_schema))
        .layer(middleware::from_fn_with_state(
            state.clone(),
            crate::middleware::auth_middleware,
        ));

    // Admin routes (require admin privileges)
    let admin_routes = Router::new()
        .route("/admin/templates/stats", get(handlers::admin::get_template_statistics))
        .route("/admin/templates/cleanup", post(handlers::admin::cleanup_templates))
        .route("/admin/templates/migrate", post(handlers::admin::migrate_templates))
        .route("/admin/usage", get(handlers::admin::get_usage_statistics))
        .route("/admin/validation/test", post(handlers::admin::test_validation_rules))
        .layer(middleware::from_fn_with_state(
            state.clone(),
            crate::middleware::admin_middleware,
        ));

    // Combine all routes
    Router::new()
        .merge(health_routes)
        .merge(template_routes)
        .merge(file_routes)
        .merge(version_routes)
        .merge(form_routes)
        .merge(field_routes)
        .merge(validation_routes)
        .merge(integration_routes)
        .merge(schema_routes)
        .merge(admin_routes)
        .layer(
            ServiceBuilder::new()
                .layer(TraceLayer::new_for_http())
                .layer(CompressionLayer::new())
                .layer(CorsLayer::permissive()) // Configure CORS as needed
        )
        .with_state(state)
} 
