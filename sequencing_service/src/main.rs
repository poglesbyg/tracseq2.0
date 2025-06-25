use anyhow::Result;
use std::net::SocketAddr;
use tracing::info;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

// Import everything from lib.rs to avoid conflicts
use sequencing_service::{
    AppState, AuthClient, Config, DatabasePool, NotificationClient, SampleClient,
    SequencingServiceImpl, StorageClient, TemplateClient, create_app,
};

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize tracing
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "sequencing_service=debug,tower_http=debug".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    info!("Starting Sequencing Management Service");

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
    let notification_client = NotificationClient::new(config.notification_service_url.clone());
    let template_client = TemplateClient::new(config.template_service_url.clone());
    let storage_client = StorageClient::new(config.storage_service_url.clone());

    // Initialize sequencing service
    let sequencing_service = SequencingServiceImpl::new(
        db_pool.clone(),
        config.clone(),
        auth_client.clone(),
        sample_client.clone(),
        notification_client.clone(),
        template_client.clone(),
    )?;
    info!("Sequencing service initialized");

    // Setup application state with Arc wrappers
    let app_state = AppState {
        sequencing_service: std::sync::Arc::new(sequencing_service),
        config: std::sync::Arc::new(config.clone()),
        db_pool,
        auth_client: std::sync::Arc::new(auth_client),
        sample_client: std::sync::Arc::new(sample_client),
        notification_client: std::sync::Arc::new(notification_client),
        template_client: std::sync::Arc::new(template_client),
        storage_client: std::sync::Arc::new(storage_client),
    };

    // Build the application router
    let app = create_app(app_state);

    // Start the server
    let addr = SocketAddr::from(([0, 0, 0, 0], config.server.port));
    info!("Sequencing management service listening on {}", addr);

    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}
