pub mod config;
pub mod models;
pub mod channels;
pub mod services;
pub mod templates;
pub mod handlers;
pub mod error;

pub use config::Config;
pub use error::{NotificationError, Result};
pub use models::*;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_library_imports() {
        // Basic smoke test to ensure all modules can be imported
        assert!(true);
    }
} 