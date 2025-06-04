pub mod database;

use std::path::PathBuf;

/// Configuration for the entire application
#[derive(Debug, Clone)]
pub struct AppConfig {
    pub database: DatabaseConfig,
    pub storage: StorageConfig,
    pub server: ServerConfig,
}

/// Database configuration
#[derive(Debug, Clone)]
pub struct DatabaseConfig {
    pub url: String,
    pub max_connections: u32,
    pub min_connections: u32,
}

/// Storage configuration
#[derive(Debug, Clone)]
pub struct StorageConfig {
    pub base_path: PathBuf,
    pub max_file_size: u64,
    pub allowed_extensions: Vec<String>,
}

/// Server configuration
#[derive(Debug, Clone)]
pub struct ServerConfig {
    pub host: String,
    pub port: u16,
    pub cors_enabled: bool,
}

impl AppConfig {
    /// Create configuration from environment variables
    pub fn from_env() -> Result<Self, ConfigError> {
        let database_url = std::env::var("DATABASE_URL")
            .map_err(|_| ConfigError::MissingEnvVar("DATABASE_URL"))?;

        let storage_path = std::env::var("STORAGE_PATH")
            .map_err(|_| ConfigError::MissingEnvVar("STORAGE_PATH"))?;

        let host = std::env::var("HOST").unwrap_or_else(|_| "0.0.0.0".to_string());
        let port = std::env::var("PORT")
            .unwrap_or_else(|_| "3000".to_string())
            .parse()
            .map_err(ConfigError::InvalidPort)?;

        Ok(Self {
            database: DatabaseConfig {
                url: database_url,
                max_connections: 10,
                min_connections: 1,
            },
            storage: StorageConfig {
                base_path: PathBuf::from(storage_path),
                max_file_size: 100 * 1024 * 1024, // 100MB
                allowed_extensions: vec!["xlsx".to_string(), "xls".to_string(), "csv".to_string()],
            },
            server: ServerConfig {
                host,
                port,
                cors_enabled: true,
            },
        })
    }

    /// Create a default configuration for testing
    pub fn for_testing() -> Self {
        Self {
            database: DatabaseConfig {
                url: "postgres://test:test@localhost:5432/test_lab_manager".to_string(),
                max_connections: 5,
                min_connections: 1,
            },
            storage: StorageConfig {
                base_path: PathBuf::from("/tmp/lab_manager_test"),
                max_file_size: 10 * 1024 * 1024, // 10MB for tests
                allowed_extensions: vec!["xlsx".to_string(), "csv".to_string()],
            },
            server: ServerConfig {
                host: "127.0.0.1".to_string(),
                port: 0, // Random port for tests
                cors_enabled: false,
            },
        }
    }
}

#[derive(Debug, thiserror::Error)]
pub enum ConfigError {
    #[error("Missing environment variable: {0}")]
    MissingEnvVar(&'static str),
    #[error("Invalid port number")]
    InvalidPort(#[from] std::num::ParseIntError),
}
