pub mod assembly;
pub mod config;
pub mod errors;
pub mod events;
pub mod handlers;
pub mod middleware;
pub mod models;
pub mod plugins;
pub mod repositories;
pub mod router;
pub mod sample_submission;
pub mod sequencing;
pub mod services;
pub mod storage;
pub mod tests;
pub mod validation;

use std::net::SocketAddr;

use assembly::assemble_production_components;
use router::create_app_router;

// Re-export the component types from the library for binary usage
use sqlx::PgPool;
use std::sync::Arc;

#[derive(Clone)]
pub struct AppComponents {
    pub config: config::AppConfig,
    pub database: DatabaseComponent,
    pub storage: StorageComponent,
    pub sample_processing: SampleProcessingComponent,
    pub sequencing: SequencingComponent,
    pub repositories: assembly::RepositoriesComponent,
    pub user_manager: models::user::UserManager,
    pub auth_service: services::auth_service::AuthService,
    pub spreadsheet_service: services::spreadsheet_service::SpreadsheetService,
}

#[derive(Clone)]
pub struct DatabaseComponent {
    pub pool: PgPool,
}

#[derive(Clone)]
pub struct StorageComponent {
    pub storage: Arc<storage::Storage>,
}

#[derive(Clone)]
pub struct SampleProcessingComponent {
    pub manager: Arc<sample_submission::SampleSubmissionManager>,
}

#[derive(Clone)]
pub struct SequencingComponent {
    pub manager: Arc<sequencing::SequencingManager>,
}

#[tokio::main]
async fn main() {
    // Initialize tracing
    tracing_subscriber::fmt::init();

    // Load environment variables
    dotenvy::dotenv().ok();

    // Assemble all components using the new modular system
    let components = assemble_production_components()
        .await
        .expect("Failed to assemble application components");

    // Create the application router
    let app = create_app_router().with_state(components);

    // Get server configuration
    let config = config::AppConfig::from_env().expect("Failed to load configuration");

    // Run the application
    let addr = format!("{}:{}", config.server.host, config.server.port)
        .parse::<SocketAddr>()
        .expect("Invalid host:port combination");
    tracing::info!("Starting server on {}", addr);

    axum::serve(
        tokio::net::TcpListener::bind(addr).await.unwrap(),
        app.into_make_service_with_connect_info::<SocketAddr>(),
    )
    .await
    .unwrap();
}
