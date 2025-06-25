use anyhow::Result;
use std::{net::SocketAddr, sync::Arc};
use tracing::{info, warn};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

// Import everything from lib.rs to avoid conflicts
use enhanced_storage_service::{
    AppState, Config, DatabasePool, ai, create_app, integrations, services::EnhancedStorageService,
};

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize tracing
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "enhanced_storage_service=debug,tower_http=debug".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    info!("🏪 Starting Enhanced Storage Service - Phase 3");

    // Load configuration
    let config = Config::from_env()?;
    info!("📋 Configuration loaded successfully");

    // Setup database connection
    let db_pool = DatabasePool::new(&config.database_url).await?;
    info!("🗄️ Database connection established");

    // Run database migrations
    db_pool.migrate().await?;
    info!("📊 Database migrations completed");

    // Initialize AI platform
    let ai_platform = ai::initialize_ai_platform().await?;
    info!("🤖 AI/ML Platform initialized");

    // Initialize enterprise integrations
    let integration_config = integrations::IntegrationConfig::default();
    let integration_hub = integrations::initialize_integration_hub(integration_config).await?;
    info!("🏢 Enterprise Integration Hub initialized");

    // Initialize enhanced storage service
    let storage_service = EnhancedStorageService::new(db_pool.clone(), config.clone()).await?;
    info!("🏪 Enhanced storage service initialized");

    // Setup application state with Phase 3 capabilities using Arc wrappers
    let app_state = AppState {
        storage_service: Arc::new(storage_service),
        config: Arc::new(config.clone()),
        db_pool,
        ai_platform: Arc::new(ai_platform),
        integration_hub: Arc::new(integration_hub),
    };

    // Build the application router
    let app = create_app(app_state);

    // Start the server
    let addr = SocketAddr::from(([0, 0, 0, 0], config.server.port));
    info!("🚀 Enhanced Storage Service Phase 3 listening on {}", addr);

    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}
