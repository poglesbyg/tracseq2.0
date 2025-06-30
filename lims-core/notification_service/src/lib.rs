pub mod config;
pub mod models;
pub mod channels;
pub mod services;
pub mod templates;
pub mod handlers;
pub mod error;
pub mod database;
pub mod clients;

pub use config::Config;
pub use error::{NotificationError, Result};
pub use models::*;

// AppState struct for application state management
use sqlx::PgPool;
use std::sync::Arc;

#[derive(Clone)]
pub struct AppState {
    pub notification_service: Arc<services::NotificationServiceImpl>,
    pub database: PgPool,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_library_imports() {
        // Basic smoke test to ensure all modules can be imported
        assert!(true);
    }
} 