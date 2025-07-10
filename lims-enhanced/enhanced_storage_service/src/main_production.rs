use anyhow::Result;
use std::net::SocketAddr;
use tracing::info;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

use enhanced_storage_service::{Config, AppState, create_app};

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize tracing
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "enhanced_storage_service=info,tower_http=debug,sqlx=warn".into()),
        )
        .with(tracing_subscriber::fmt::layer().with_target(false))
        .init();

    info!("🏪 Starting Enhanced Storage Service - Production Version with Hierarchical Storage");

    // Load configuration from environment
    let config = Config::from_env();
    info!("📋 Configuration loaded successfully");

    // Create application state
    let app_state = AppState::new(config.clone()).await?;
    info!("✅ Application state initialized");

    // Create the application using the lib.rs create_app function
    let app = create_app(app_state);

    // Start the server
    let port = config.port;
    let addr = SocketAddr::from(([0, 0, 0, 0], port));
    info!("🚀 Enhanced Storage Service (Production) listening on {}", addr);

    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
} 