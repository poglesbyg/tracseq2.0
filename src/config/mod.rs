pub mod database;

use std::path::PathBuf;

/// Configuration for the entire application
#[derive(Debug, Clone)]
pub struct AppConfig {
    pub database: DatabaseConfig,
    pub storage: StorageConfig,
    pub server: ServerConfig,
    pub rag: RagIntegrationConfig,
}

/// Database configuration
#[derive(Debug, Clone)]
pub struct DatabaseConfig {
    pub url: String,
    pub max_connections: u32,
    pub min_connections: u32,
    pub acquire_timeout: std::time::Duration,
    pub idle_timeout: Option<std::time::Duration>,
    pub max_lifetime: Option<std::time::Duration>,
}

impl DatabaseConfig {
    /// Create configuration from environment variables
    pub fn from_env() -> Result<Self, ConfigError> {
        let database_url = std::env::var("DATABASE_URL")
            .map_err(|_| ConfigError::MissingEnvVar("DATABASE_URL"))?;

        Ok(Self {
            url: database_url,
            max_connections: 10,
            min_connections: 1,
            acquire_timeout: std::time::Duration::from_secs(30),
            idle_timeout: Some(std::time::Duration::from_secs(600)),
            max_lifetime: Some(std::time::Duration::from_secs(1800)),
        })
    }

    /// Create a default configuration for testing
    pub fn for_testing() -> Self {
        Self {
            url: "sqlite::memory:".to_string(),
            max_connections: 5,
            min_connections: 1,
            acquire_timeout: std::time::Duration::from_secs(30),
            idle_timeout: Some(std::time::Duration::from_secs(300)),
            max_lifetime: Some(std::time::Duration::from_secs(1800)),
        }
    }
}

impl Default for DatabaseConfig {
    fn default() -> Self {
        Self::for_testing()
    }
}

/// Storage configuration
#[derive(Debug, Clone)]
pub struct StorageConfig {
    pub base_path: PathBuf,
    pub max_file_size: u64,
    pub allowed_extensions: Vec<String>,
    pub temp_dir: PathBuf,
}

impl StorageConfig {
    /// Create configuration from environment variables
    pub fn from_env() -> Result<Self, ConfigError> {
        let storage_path = std::env::var("STORAGE_PATH")
            .map_err(|_| ConfigError::MissingEnvVar("STORAGE_PATH"))?;

        Ok(Self {
            base_path: PathBuf::from(&storage_path),
            max_file_size: 100 * 1024 * 1024, // 100MB
            allowed_extensions: vec!["xlsx".to_string(), "xls".to_string(), "csv".to_string()],
            temp_dir: PathBuf::from(&storage_path).join("temp"),
        })
    }

    /// Create a default configuration for testing
    pub fn for_testing() -> Self {
        Self {
            base_path: PathBuf::from("/tmp/lab_manager_test"),
            max_file_size: 10 * 1024 * 1024, // 10MB for tests
            allowed_extensions: vec!["xlsx".to_string(), "csv".to_string()],
            temp_dir: PathBuf::from("/tmp/lab_manager_test/temp"),
        }
    }
}

impl Default for StorageConfig {
    fn default() -> Self {
        Self::for_testing()
    }
}

/// Server configuration
#[derive(Debug, Clone)]
pub struct ServerConfig {
    pub host: String,
    pub port: u16,
    pub cors_enabled: bool,
}

/// RAG integration configuration
#[derive(Debug, Clone)]
pub struct RagIntegrationConfig {
    pub enabled: bool,
    pub base_url: String,
    pub timeout_seconds: u64,
    pub max_file_size_mb: u64,
    pub supported_formats: Vec<String>,
    pub default_confidence_threshold: f64,
    pub auto_create_samples: bool,
}

impl Default for RagIntegrationConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            base_url: "http://127.0.0.1:8000".to_string(),
            timeout_seconds: 300,
            max_file_size_mb: 50,
            supported_formats: vec!["pdf".to_string(), "docx".to_string(), "txt".to_string()],
            default_confidence_threshold: 0.7,
            auto_create_samples: false,
        }
    }
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

        let rag_base_url = std::env::var("RAG_SERVICE_URL")
            .unwrap_or_else(|_| "http://host.docker.internal:8000".to_string());

        Ok(Self {
            database: DatabaseConfig {
                url: database_url,
                max_connections: 10,
                min_connections: 1,
                acquire_timeout: std::time::Duration::from_secs(30),
                idle_timeout: Some(std::time::Duration::from_secs(600)),
                max_lifetime: Some(std::time::Duration::from_secs(1800)),
            },
            storage: StorageConfig {
                base_path: PathBuf::from(&storage_path),
                max_file_size: 100 * 1024 * 1024, // 100MB
                allowed_extensions: vec!["xlsx".to_string(), "xls".to_string(), "csv".to_string()],
                temp_dir: PathBuf::from(&storage_path).join("temp"),
            },
            server: ServerConfig {
                host,
                port,
                cors_enabled: true,
            },
            rag: RagIntegrationConfig {
                base_url: rag_base_url,
                ..RagIntegrationConfig::default()
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
                acquire_timeout: std::time::Duration::from_secs(30),
                idle_timeout: Some(std::time::Duration::from_secs(300)),
                max_lifetime: Some(std::time::Duration::from_secs(1800)),
            },
            storage: StorageConfig {
                base_path: PathBuf::from("/tmp/lab_manager_test"),
                max_file_size: 10 * 1024 * 1024, // 10MB for tests
                allowed_extensions: vec!["xlsx".to_string(), "csv".to_string()],
                temp_dir: PathBuf::from("/tmp/lab_manager_test/temp"),
            },
            server: ServerConfig {
                host: "127.0.0.1".to_string(),
                port: 0, // Random port for tests
                cors_enabled: false,
            },
            rag: RagIntegrationConfig::default(),
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
