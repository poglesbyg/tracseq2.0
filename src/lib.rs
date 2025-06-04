// Library re-exports for examples and external use

pub mod assembly;
pub mod config;
pub mod errors;
pub mod events;
pub mod handlers;
pub mod models;
pub mod router;
pub mod sample_submission;
pub mod sequencing;
pub mod services;
pub mod storage;
pub mod validation;

// Re-export main component types for convenience
pub use assembly::{AssemblyError, ComponentBuilder};
pub use config::{AppConfig, DatabaseConfig, ServerConfig, StorageConfig};
