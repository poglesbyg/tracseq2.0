use anyhow::Result;
use std::net::SocketAddr;
use tracing::info;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

// Import everything from lib.rs to avoid conflicts
use template_service::{
    AppState, AuthClient, Config, DatabasePool, SampleClient, TemplateServiceImpl, create_app,
};

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
    let auth_client = AuthClient::new(config.services.auth_service_url.clone());
    let sample_client = SampleClient::new(config.services.sample_service_url.clone());

    // Initialize template service
    let template_service = TemplateServiceImpl::new(
        db_pool.clone(),
        config.clone(),
        auth_client.clone(),
        sample_client.clone(),
    )?;
    info!("Template service initialized");

    // Setup application state with Arc wrappers
    let app_state = AppState {
        template_service: std::sync::Arc::new(template_service),
        config: std::sync::Arc::new(config.clone()),
        db_pool,
        auth_client: std::sync::Arc::new(auth_client),
        sample_client: std::sync::Arc::new(sample_client),
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
