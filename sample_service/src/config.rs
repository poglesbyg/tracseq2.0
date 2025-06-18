use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::env;

/// Main configuration structure for the sample management service
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub server: ServerConfig,
    pub database_url: String,
    pub sample: SampleConfig,
    pub barcode: BarcodeConfig,
    pub workflow: WorkflowConfig,
    pub services: ServiceConfig,
    pub logging: LoggingConfig,
    pub features: FeatureConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerConfig {
    pub host: String,
    pub port: u16,
    pub workers: Option<usize>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SampleConfig {
    pub max_batch_size: usize,
    pub default_status: String,
    pub auto_generate_barcode: bool,
    pub validation_timeout_seconds: u64,
    pub metadata_max_size_kb: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BarcodeConfig {
    pub prefix: String,
    pub length: usize,
    pub include_timestamp: bool,
    pub include_sequence: bool,
    pub separator: String,
    pub checksum: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkflowConfig {
    pub auto_transitions: bool,
    pub validation_required: bool,
    pub notification_enabled: bool,
    pub status_timeout_hours: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceConfig {
    pub auth_service_url: String,
    pub storage_service_url: String,
    pub template_service_url: Option<String>,
    pub sequencing_service_url: Option<String>,
    pub request_timeout_seconds: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoggingConfig {
    pub level: String,
    pub format: String,
    pub file_enabled: bool,
    pub file_path: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FeatureConfig {
    pub batch_processing_enabled: bool,
    pub template_integration_enabled: bool,
    pub workflow_automation_enabled: bool,
    pub barcode_scanning_enabled: bool,
    pub audit_logging_enabled: bool,
}

impl Config {
    /// Load configuration from environment variables
    pub fn from_env() -> Result<Self> {
        dotenvy::dotenv().ok(); // Load .env file if present

        let config = Config {
            server: ServerConfig {
                host: env::var("SAMPLE_HOST").unwrap_or_else(|_| "0.0.0.0".to_string()),
                port: env::var("SAMPLE_PORT")
                    .unwrap_or_else(|_| "8081".to_string())
                    .parse()
                    .unwrap_or(8081),
                workers: env::var("SAMPLE_WORKERS").ok().and_then(|w| w.parse().ok()),
            },
            database_url: env::var("SAMPLE_DATABASE_URL")
                .or_else(|_| env::var("DATABASE_URL"))
                .unwrap_or_else(|_| {
                    "postgresql://sample_user:sample_password@localhost:5432/sample_db".to_string()
                }),
            sample: SampleConfig {
                max_batch_size: env::var("SAMPLE_MAX_BATCH_SIZE")
                    .unwrap_or_else(|_| "100".to_string())
                    .parse()
                    .unwrap_or(100),
                default_status: env::var("SAMPLE_DEFAULT_STATUS")
                    .unwrap_or_else(|_| "pending".to_string()),
                auto_generate_barcode: env::var("SAMPLE_AUTO_GENERATE_BARCODE")
                    .unwrap_or_else(|_| "true".to_string())
                    .parse()
                    .unwrap_or(true),
                validation_timeout_seconds: env::var("SAMPLE_VALIDATION_TIMEOUT_SECONDS")
                    .unwrap_or_else(|_| "30".to_string())
                    .parse()
                    .unwrap_or(30),
                metadata_max_size_kb: env::var("SAMPLE_METADATA_MAX_SIZE_KB")
                    .unwrap_or_else(|_| "64".to_string())
                    .parse()
                    .unwrap_or(64),
            },
            barcode: BarcodeConfig {
                prefix: env::var("BARCODE_PREFIX").unwrap_or_else(|_| "LAB".to_string()),
                length: env::var("BARCODE_LENGTH")
                    .unwrap_or_else(|_| "12".to_string())
                    .parse()
                    .unwrap_or(12),
                include_timestamp: env::var("BARCODE_INCLUDE_TIMESTAMP")
                    .unwrap_or_else(|_| "true".to_string())
                    .parse()
                    .unwrap_or(true),
                include_sequence: env::var("BARCODE_INCLUDE_SEQUENCE")
                    .unwrap_or_else(|_| "true".to_string())
                    .parse()
                    .unwrap_or(true),
                separator: env::var("BARCODE_SEPARATOR").unwrap_or_else(|_| "-".to_string()),
                checksum: env::var("BARCODE_CHECKSUM")
                    .unwrap_or_else(|_| "false".to_string())
                    .parse()
                    .unwrap_or(false),
            },
            workflow: WorkflowConfig {
                auto_transitions: env::var("WORKFLOW_AUTO_TRANSITIONS")
                    .unwrap_or_else(|_| "false".to_string())
                    .parse()
                    .unwrap_or(false),
                validation_required: env::var("WORKFLOW_VALIDATION_REQUIRED")
                    .unwrap_or_else(|_| "true".to_string())
                    .parse()
                    .unwrap_or(true),
                notification_enabled: env::var("WORKFLOW_NOTIFICATION_ENABLED")
                    .unwrap_or_else(|_| "false".to_string())
                    .parse()
                    .unwrap_or(false),
                status_timeout_hours: env::var("WORKFLOW_STATUS_TIMEOUT_HOURS")
                    .unwrap_or_else(|_| "24".to_string())
                    .parse()
                    .unwrap_or(24),
            },
            services: ServiceConfig {
                auth_service_url: env::var("AUTH_SERVICE_URL")
                    .unwrap_or_else(|_| "http://auth-service:8080".to_string()),
                storage_service_url: env::var("STORAGE_SERVICE_URL")
                    .unwrap_or_else(|_| "http://storage-service:8082".to_string()),
                template_service_url: env::var("TEMPLATE_SERVICE_URL").ok(),
                sequencing_service_url: env::var("SEQUENCING_SERVICE_URL").ok(),
                request_timeout_seconds: env::var("SERVICE_REQUEST_TIMEOUT_SECONDS")
                    .unwrap_or_else(|_| "30".to_string())
                    .parse()
                    .unwrap_or(30),
            },
            logging: LoggingConfig {
                level: env::var("LOG_LEVEL").unwrap_or_else(|_| "info".to_string()),
                format: env::var("LOG_FORMAT").unwrap_or_else(|_| "json".to_string()),
                file_enabled: env::var("LOG_FILE_ENABLED")
                    .unwrap_or_else(|_| "false".to_string())
                    .parse()
                    .unwrap_or(false),
                file_path: env::var("LOG_FILE_PATH").ok(),
            },
            features: FeatureConfig {
                batch_processing_enabled: env::var("FEATURE_BATCH_PROCESSING")
                    .unwrap_or_else(|_| "true".to_string())
                    .parse()
                    .unwrap_or(true),
                template_integration_enabled: env::var("FEATURE_TEMPLATE_INTEGRATION")
                    .unwrap_or_else(|_| "true".to_string())
                    .parse()
                    .unwrap_or(true),
                workflow_automation_enabled: env::var("FEATURE_WORKFLOW_AUTOMATION")
                    .unwrap_or_else(|_| "false".to_string())
                    .parse()
                    .unwrap_or(false),
                barcode_scanning_enabled: env::var("FEATURE_BARCODE_SCANNING")
                    .unwrap_or_else(|_| "true".to_string())
                    .parse()
                    .unwrap_or(true),
                audit_logging_enabled: env::var("FEATURE_AUDIT_LOGGING")
                    .unwrap_or_else(|_| "true".to_string())
                    .parse()
                    .unwrap_or(true),
            },
        };

        config.validate()?;
        Ok(config)
    }

    /// Validate the configuration
    pub fn validate(&self) -> Result<()> {
        // Validate server port
        if self.server.port == 0 {
            return Err(anyhow::anyhow!("Server port must be greater than 0"));
        }

        // Validate database URL
        if self.database_url.is_empty() {
            return Err(anyhow::anyhow!("Database URL cannot be empty"));
        }

        // Validate service URLs
        if self.services.auth_service_url.is_empty() {
            return Err(anyhow::anyhow!("Auth service URL cannot be empty"));
        }

        if self.services.storage_service_url.is_empty() {
            return Err(anyhow::anyhow!("Storage service URL cannot be empty"));
        }

        // Validate sample configuration
        if self.sample.max_batch_size == 0 {
            return Err(anyhow::anyhow!("Max batch size must be greater than 0"));
        }

        if self.sample.max_batch_size > 1000 {
            return Err(anyhow::anyhow!("Max batch size cannot exceed 1000"));
        }

        // Validate barcode configuration
        if self.barcode.prefix.is_empty() {
            return Err(anyhow::anyhow!("Barcode prefix cannot be empty"));
        }

        if self.barcode.length < 6 {
            return Err(anyhow::anyhow!("Barcode length must be at least 6"));
        }

        Ok(())
    }

    /// Get environment-specific configuration
    pub fn environment(&self) -> String {
        env::var("ENVIRONMENT").unwrap_or_else(|_| "development".to_string())
    }

    /// Check if running in production
    pub fn is_production(&self) -> bool {
        self.environment().to_lowercase() == "production"
    }

    /// Check if running in development
    pub fn is_development(&self) -> bool {
        self.environment().to_lowercase() == "development"
    }

    /// Get the auth service URL
    pub fn auth_service_url(&self) -> &str {
        &self.services.auth_service_url
    }

    /// Get the storage service URL
    pub fn storage_service_url(&self) -> &str {
        &self.services.storage_service_url
    }
}
