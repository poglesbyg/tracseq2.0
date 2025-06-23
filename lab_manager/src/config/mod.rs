pub mod database;

use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// Configuration for the entire application
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppConfig {
    pub database: DatabaseConfig,
    pub storage: StorageConfig,
    pub server: ServerConfig,
    pub rag: RagIntegrationConfig,
    pub shibboleth: ShibbolethConfig,
}

/// Database configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DatabaseConfig {
    pub url: String,
    pub max_connections: u32,
    pub min_connections: u32,
}

/// Storage configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StorageConfig {
    pub base_path: PathBuf,
    pub max_file_size: u64,
    pub allowed_extensions: Vec<String>,
}

/// Server configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerConfig {
    pub host: String,
    pub port: u16,
    pub cors_enabled: bool,
}

/// RAG integration configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RagIntegrationConfig {
    pub enabled: bool,
    pub base_url: String,
    pub timeout_seconds: u64,
    pub max_file_size_mb: u64,
    pub supported_formats: Vec<String>,
    pub default_confidence_threshold: f64,
    pub auto_create_samples: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ShibbolethConfig {
    /// Whether Shibboleth authentication is enabled
    pub enabled: bool,

    /// Whether to allow hybrid authentication (both Shibboleth and JWT)
    pub hybrid_mode: bool,

    /// URL path that requires Shibboleth authentication (e.g., "/shibboleth-login")
    pub login_path: String,

    /// URL to redirect to after successful Shibboleth authentication
    pub success_redirect: String,

    /// URL to redirect to after logout
    pub logout_redirect: String,

    /// Whether to auto-create users from Shibboleth attributes
    pub auto_create_users: bool,

    /// Whether to auto-update user attributes from Shibboleth on each login
    pub auto_update_attributes: bool,

    /// Default role for new users created via Shibboleth
    pub default_role: String,

    /// Required Shibboleth attributes (user creation will fail if missing)
    pub required_attributes: Vec<String>,

    /// Custom attribute mappings from Shibboleth headers to internal names
    pub attribute_mappings: std::collections::HashMap<String, String>,
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

impl Default for ShibbolethConfig {
    fn default() -> Self {
        let mut default_mappings = std::collections::HashMap::new();
        default_mappings.insert("eppn".to_string(), "HTTP_EPPN".to_string());
        default_mappings.insert("mail".to_string(), "HTTP_MAIL".to_string());
        default_mappings.insert("displayName".to_string(), "HTTP_DISPLAYNAME".to_string());
        default_mappings.insert("givenName".to_string(), "HTTP_GIVENNAME".to_string());
        default_mappings.insert("surname".to_string(), "HTTP_SN".to_string());
        default_mappings.insert("affiliation".to_string(), "HTTP_AFFILIATION".to_string());
        default_mappings.insert("entitlement".to_string(), "HTTP_ENTITLEMENT".to_string());
        default_mappings.insert("isMemberOf".to_string(), "HTTP_ISMEMBEROF".to_string());

        Self {
            enabled: false,
            hybrid_mode: true,
            login_path: "/shibboleth-login".to_string(),
            success_redirect: "/dashboard".to_string(),
            logout_redirect: "/".to_string(),
            auto_create_users: true,
            auto_update_attributes: true,
            default_role: "Guest".to_string(),
            required_attributes: vec!["mail".to_string(), "eppn".to_string()],
            attribute_mappings: default_mappings,
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

        // Shibboleth configuration from environment
        let shibboleth_enabled = std::env::var("SHIBBOLETH_ENABLED")
            .unwrap_or_else(|_| "false".to_string())
            .parse()
            .unwrap_or(false);

        let shibboleth_hybrid_mode = std::env::var("SHIBBOLETH_HYBRID_MODE")
            .unwrap_or_else(|_| "true".to_string())
            .parse()
            .unwrap_or(true);

        let shibboleth_auto_create = std::env::var("SHIBBOLETH_AUTO_CREATE_USERS")
            .unwrap_or_else(|_| "true".to_string())
            .parse()
            .unwrap_or(true);

        let shibboleth_auto_update = std::env::var("SHIBBOLETH_AUTO_UPDATE_ATTRIBUTES")
            .unwrap_or_else(|_| "true".to_string())
            .parse()
            .unwrap_or(true);

        let shibboleth_default_role =
            std::env::var("SHIBBOLETH_DEFAULT_ROLE").unwrap_or_else(|_| "Guest".to_string());

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
            rag: RagIntegrationConfig {
                base_url: rag_base_url,
                ..RagIntegrationConfig::default()
            },
            shibboleth: ShibbolethConfig {
                enabled: shibboleth_enabled,
                hybrid_mode: shibboleth_hybrid_mode,
                auto_create_users: shibboleth_auto_create,
                auto_update_attributes: shibboleth_auto_update,
                default_role: shibboleth_default_role,
                ..ShibbolethConfig::default()
            },
        })
    }

    /// Create a default configuration for testing
    pub fn for_testing() -> Self {
        Self {
            database: DatabaseConfig {
                url: "postgres://postgres:postgres@localhost:5432/lab_manager_test".to_string(),
                max_connections: 5,
                min_connections: 1,
            },
            storage: StorageConfig {
                base_path: PathBuf::from("./test_storage"),
                max_file_size: 10 * 1024 * 1024, // 10MB for tests
                allowed_extensions: vec!["xlsx".to_string(), "csv".to_string()],
            },
            server: ServerConfig {
                host: "127.0.0.1".to_string(),
                port: 0, // Let the system choose
                cors_enabled: false,
            },
            rag: RagIntegrationConfig::default(),
            shibboleth: ShibbolethConfig {
                enabled: false, // Disabled for tests by default
                ..ShibbolethConfig::default()
            },
        }
    }
}

impl DatabaseConfig {
    /// Create a configuration for testing
    pub fn for_testing() -> Self {
        Self {
            url: "postgres://postgres:postgres@localhost:5432/lab_manager_test".to_string(),
            max_connections: 5,
            min_connections: 1,
        }
    }

    /// Create configuration from environment variables
    pub fn from_env() -> Result<Self, ConfigError> {
        let database_url = std::env::var("DATABASE_URL")
            .map_err(|_| ConfigError::MissingEnvVar("DATABASE_URL"))?;

        Ok(Self {
            url: database_url,
            max_connections: 10,
            min_connections: 1,
        })
    }
}

impl StorageConfig {
    /// Create configuration from environment variables
    pub fn from_env() -> Result<Self, ConfigError> {
        let storage_path = std::env::var("STORAGE_PATH")
            .map_err(|_| ConfigError::MissingEnvVar("STORAGE_PATH"))?;

        Ok(Self {
            base_path: PathBuf::from(storage_path),
            max_file_size: 100 * 1024 * 1024, // 100MB
            allowed_extensions: vec!["xlsx".to_string(), "xls".to_string(), "csv".to_string()],
        })
    }
}

impl Default for StorageConfig {
    fn default() -> Self {
        Self {
            base_path: PathBuf::from("./storage"),
            max_file_size: 100 * 1024 * 1024, // 100MB
            allowed_extensions: vec!["xlsx".to_string(), "xls".to_string(), "csv".to_string()],
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
