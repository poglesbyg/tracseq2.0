mod assembly;
mod config;
mod handlers;
mod models;
mod router;
mod sample_submission;
mod sequencing;
mod storage;
pub mod tests;

use sqlx::PgPool;
use std::{net::SocketAddr, sync::Arc};

use assembly::assemble_production_components;
use router::create_app_router;

/// Core application components that can be assembled independently
#[derive(Clone)]
pub struct AppComponents {
    pub database: DatabaseComponent,
    pub storage: StorageComponent,
    pub sample_processing: SampleProcessingComponent,
    pub sequencing: SequencingComponent,
}

/// Database component with its own configuration and lifecycle
#[derive(Clone)]
pub struct DatabaseComponent {
    pub pool: PgPool,
}

/// Storage component for managing sample storage
#[derive(Clone)]
pub struct StorageComponent {
    pub storage: Arc<storage::Storage>,
}

/// Sample processing component for handling sample submissions
#[derive(Clone)]
pub struct SampleProcessingComponent {
    pub manager: Arc<sample_submission::SampleSubmissionManager>,
}

/// Sequencing component for managing sequencing jobs
#[derive(Clone)]
pub struct SequencingComponent {
    pub manager: Arc<sequencing::SequencingManager>,
}

#[tokio::main]
async fn main() {
    // Initialize tracing
    tracing_subscriber::fmt::init();

    // Load environment variables
    dotenv::dotenv().ok();

    // Assemble all components using the new modular system
    let components = assemble_production_components()
        .await
        .expect("Failed to assemble application components");

    // Create the application router
    let app = create_app_router().with_state(components);

    // Get server configuration
    let config = config::AppConfig::from_env().expect("Failed to load configuration");

    // Run the application
    let addr = SocketAddr::from(([0, 0, 0, 0], config.server.port));
    tracing::info!("Starting server on {}", addr);

    axum::serve(
        tokio::net::TcpListener::bind(addr).await.unwrap(),
        app.into_make_service(),
    )
    .await
    .unwrap();
}
