// Library re-exports for examples and external use

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
pub mod validation;

// Re-export main component types for convenience
pub use assembly::{AssemblyError, ComponentBuilder};
pub use config::{AppConfig, ServerConfig};

// Main application component types
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
