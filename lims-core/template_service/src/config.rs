use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::env;

/// Main configuration structure for the template management service
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub server: ServerConfig,
    pub database_url: String,
    pub template: TemplateConfig,
    pub form: FormConfig,
    pub validation: ValidationConfig,
    pub file: FileConfig,
    pub versioning: VersioningConfig,
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
pub struct TemplateConfig {
    pub max_templates_per_user: usize,
    pub max_fields_per_template: usize,
    pub default_template_type: String,
    pub cache_templates: bool,
    pub cache_ttl_seconds: u64,
    pub auto_backup: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FormConfig {
    pub enable_dynamic_forms: bool,
    pub max_form_size_kb: usize,
    pub enable_form_preview: bool,
    pub enable_conditional_fields: bool,
    pub form_cache_enabled: bool,
    pub render_timeout_seconds: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationConfig {
    pub enable_strict_validation: bool,
    pub max_validation_rules_per_field: usize,
    pub enable_cross_field_validation: bool,
    pub enable_async_validation: bool,
    pub validation_timeout_seconds: u64,
    pub cache_validation_results: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileConfig {
    pub upload_path: String,
    pub max_file_size_mb: usize,
    pub allowed_extensions: Vec<String>,
    pub enable_virus_scanning: bool,
    pub enable_compression: bool,
    pub backup_enabled: bool,
    pub backup_retention_days: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VersioningConfig {
    pub enable_versioning: bool,
    pub max_versions_per_template: usize,
    pub auto_create_versions: bool,
    pub version_naming_strategy: String, // "semantic", "timestamp", "sequential"
    pub compress_old_versions: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceConfig {
    pub auth_service_url: String,
    pub sample_service_url: String,
    pub storage_service_url: Option<String>,
    pub notification_service_url: Option<String>,
    pub request_timeout_seconds: u64,
    pub retry_attempts: u32,
    pub circuit_breaker_enabled: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoggingConfig {
    pub level: String,
    pub format: String,
    pub file_enabled: bool,
    pub file_path: Option<String>,
    pub audit_enabled: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FeatureConfig {
    pub form_builder_enabled: bool,
    pub template_versioning_enabled: bool,
    pub file_upload_enabled: bool,
    pub template_cloning_enabled: bool,
    pub advanced_validation_enabled: bool,
    pub template_sharing_enabled: bool,
    pub template_analytics_enabled: bool,
}

impl Config {
    /// Load configuration from environment variables
    pub fn from_env() -> Result<Self> {
        dotenvy::dotenv().ok(); // Load .env file if present

        let config = Config {
            server: ServerConfig {
                host: env::var("TEMPLATE_HOST").unwrap_or_else(|_| "0.0.0.0".to_string()),
                port: env::var("TEMPLATE_PORT")
                    .unwrap_or_else(|_| "8083".to_string())
                    .parse()
                    .unwrap_or(8083),
                workers: env::var("TEMPLATE_WORKERS")
                    .ok()
                    .and_then(|w| w.parse().ok()),
            },
            database_url: env::var("TEMPLATE_DATABASE_URL")
                .or_else(|_| env::var("DATABASE_URL"))
                .unwrap_or_else(|_| {
                    "postgresql://template_user:template_password@localhost:5432/template_db"
                        .to_string()
                }),
            template: TemplateConfig {
                max_templates_per_user: env::var("TEMPLATE_MAX_PER_USER")
                    .unwrap_or_else(|_| "50".to_string())
                    .parse()
                    .unwrap_or(50),
                max_fields_per_template: env::var("TEMPLATE_MAX_FIELDS")
                    .unwrap_or_else(|_| "100".to_string())
                    .parse()
                    .unwrap_or(100),
                default_template_type: env::var("TEMPLATE_DEFAULT_TYPE")
                    .unwrap_or_else(|_| "sample_collection".to_string()),
                cache_templates: env::var("TEMPLATE_CACHE_ENABLED")
                    .unwrap_or_else(|_| "true".to_string())
                    .parse()
                    .unwrap_or(true),
                cache_ttl_seconds: env::var("TEMPLATE_CACHE_TTL_SECONDS")
                    .unwrap_or_else(|_| "3600".to_string())
                    .parse()
                    .unwrap_or(3600),
                auto_backup: env::var("TEMPLATE_AUTO_BACKUP")
                    .unwrap_or_else(|_| "true".to_string())
                    .parse()
                    .unwrap_or(true),
            },
            form: FormConfig {
                enable_dynamic_forms: env::var("FORM_DYNAMIC_ENABLED")
                    .unwrap_or_else(|_| "true".to_string())
                    .parse()
                    .unwrap_or(true),
                max_form_size_kb: env::var("FORM_MAX_SIZE_KB")
                    .unwrap_or_else(|_| "1024".to_string())
                    .parse()
                    .unwrap_or(1024),
                enable_form_preview: env::var("FORM_PREVIEW_ENABLED")
                    .unwrap_or_else(|_| "true".to_string())
                    .parse()
                    .unwrap_or(true),
                enable_conditional_fields: env::var("FORM_CONDITIONAL_FIELDS")
                    .unwrap_or_else(|_| "true".to_string())
                    .parse()
                    .unwrap_or(true),
                form_cache_enabled: env::var("FORM_CACHE_ENABLED")
                    .unwrap_or_else(|_| "true".to_string())
                    .parse()
                    .unwrap_or(true),
                render_timeout_seconds: env::var("FORM_RENDER_TIMEOUT_SECONDS")
                    .unwrap_or_else(|_| "30".to_string())
                    .parse()
                    .unwrap_or(30),
            },
            validation: ValidationConfig {
                enable_strict_validation: env::var("VALIDATION_STRICT_MODE")
                    .unwrap_or_else(|_| "true".to_string())
                    .parse()
                    .unwrap_or(true),
                max_validation_rules_per_field: env::var("VALIDATION_MAX_RULES_PER_FIELD")
                    .unwrap_or_else(|_| "10".to_string())
                    .parse()
                    .unwrap_or(10),
                enable_cross_field_validation: env::var("VALIDATION_CROSS_FIELD_ENABLED")
                    .unwrap_or_else(|_| "true".to_string())
                    .parse()
                    .unwrap_or(true),
                enable_async_validation: env::var("VALIDATION_ASYNC_ENABLED")
                    .unwrap_or_else(|_| "true".to_string())
                    .parse()
                    .unwrap_or(true),
                validation_timeout_seconds: env::var("VALIDATION_TIMEOUT_SECONDS")
                    .unwrap_or_else(|_| "10".to_string())
                    .parse()
                    .unwrap_or(10),
                cache_validation_results: env::var("VALIDATION_CACHE_RESULTS")
                    .unwrap_or_else(|_| "true".to_string())
                    .parse()
                    .unwrap_or(true),
            },
            file: FileConfig {
                upload_path: env::var("FILE_UPLOAD_PATH")
                    .unwrap_or_else(|_| "./uploads".to_string()),
                max_file_size_mb: env::var("FILE_MAX_SIZE_MB")
                    .unwrap_or_else(|_| "10".to_string())
                    .parse()
                    .unwrap_or(10),
                allowed_extensions: env::var("FILE_ALLOWED_EXTENSIONS")
                    .unwrap_or_else(|_| "xlsx,csv,json,xml".to_string())
                    .split(',')
                    .map(|s| s.trim().to_string())
                    .collect(),
                enable_virus_scanning: env::var("FILE_VIRUS_SCANNING")
                    .unwrap_or_else(|_| "false".to_string())
                    .parse()
                    .unwrap_or(false),
                enable_compression: env::var("FILE_COMPRESSION_ENABLED")
                    .unwrap_or_else(|_| "true".to_string())
                    .parse()
                    .unwrap_or(true),
                backup_enabled: env::var("FILE_BACKUP_ENABLED")
                    .unwrap_or_else(|_| "true".to_string())
                    .parse()
                    .unwrap_or(true),
                backup_retention_days: env::var("FILE_BACKUP_RETENTION_DAYS")
                    .unwrap_or_else(|_| "30".to_string())
                    .parse()
                    .unwrap_or(30),
            },
            versioning: VersioningConfig {
                enable_versioning: env::var("VERSIONING_ENABLED")
                    .unwrap_or_else(|_| "true".to_string())
                    .parse()
                    .unwrap_or(true),
                max_versions_per_template: env::var("VERSIONING_MAX_VERSIONS")
                    .unwrap_or_else(|_| "10".to_string())
                    .parse()
                    .unwrap_or(10),
                auto_create_versions: env::var("VERSIONING_AUTO_CREATE")
                    .unwrap_or_else(|_| "true".to_string())
                    .parse()
                    .unwrap_or(true),
                version_naming_strategy: env::var("VERSIONING_NAMING_STRATEGY")
                    .unwrap_or_else(|_| "semantic".to_string()),
                compress_old_versions: env::var("VERSIONING_COMPRESS_OLD")
                    .unwrap_or_else(|_| "true".to_string())
                    .parse()
                    .unwrap_or(true),
            },
            services: ServiceConfig {
                auth_service_url: env::var("AUTH_SERVICE_URL")
                    .unwrap_or_else(|_| "http://auth-service:8080".to_string()),
                sample_service_url: env::var("SAMPLE_SERVICE_URL")
                    .unwrap_or_else(|_| "http://sample-service:8081".to_string()),
                storage_service_url: env::var("STORAGE_SERVICE_URL").ok(),
                notification_service_url: env::var("NOTIFICATION_SERVICE_URL").ok(),
                request_timeout_seconds: env::var("SERVICE_REQUEST_TIMEOUT_SECONDS")
                    .unwrap_or_else(|_| "30".to_string())
                    .parse()
                    .unwrap_or(30),
                retry_attempts: env::var("SERVICE_RETRY_ATTEMPTS")
                    .unwrap_or_else(|_| "3".to_string())
                    .parse()
                    .unwrap_or(3),
                circuit_breaker_enabled: env::var("SERVICE_CIRCUIT_BREAKER_ENABLED")
                    .unwrap_or_else(|_| "true".to_string())
                    .parse()
                    .unwrap_or(true),
            },
            logging: LoggingConfig {
                level: env::var("LOG_LEVEL").unwrap_or_else(|_| "info".to_string()),
                format: env::var("LOG_FORMAT").unwrap_or_else(|_| "json".to_string()),
                file_enabled: env::var("LOG_FILE_ENABLED")
                    .unwrap_or_else(|_| "false".to_string())
                    .parse()
                    .unwrap_or(false),
                file_path: env::var("LOG_FILE_PATH").ok(),
                audit_enabled: env::var("LOG_AUDIT_ENABLED")
                    .unwrap_or_else(|_| "true".to_string())
                    .parse()
                    .unwrap_or(true),
            },
            features: FeatureConfig {
                form_builder_enabled: env::var("FEATURE_FORM_BUILDER")
                    .unwrap_or_else(|_| "true".to_string())
                    .parse()
                    .unwrap_or(true),
                template_versioning_enabled: env::var("FEATURE_TEMPLATE_VERSIONING")
                    .unwrap_or_else(|_| "true".to_string())
                    .parse()
                    .unwrap_or(true),
                file_upload_enabled: env::var("FEATURE_FILE_UPLOAD")
                    .unwrap_or_else(|_| "true".to_string())
                    .parse()
                    .unwrap_or(true),
                template_cloning_enabled: env::var("FEATURE_TEMPLATE_CLONING")
                    .unwrap_or_else(|_| "true".to_string())
                    .parse()
                    .unwrap_or(true),
                advanced_validation_enabled: env::var("FEATURE_ADVANCED_VALIDATION")
                    .unwrap_or_else(|_| "true".to_string())
                    .parse()
                    .unwrap_or(true),
                template_sharing_enabled: env::var("FEATURE_TEMPLATE_SHARING")
                    .unwrap_or_else(|_| "false".to_string())
                    .parse()
                    .unwrap_or(false),
                template_analytics_enabled: env::var("FEATURE_TEMPLATE_ANALYTICS")
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

        if self.services.sample_service_url.is_empty() {
            return Err(anyhow::anyhow!("Sample service URL cannot be empty"));
        }

        // Validate template configuration
        if self.template.max_templates_per_user == 0 {
            return Err(anyhow::anyhow!(
                "Max templates per user must be greater than 0"
            ));
        }

        if self.template.max_fields_per_template == 0 {
            return Err(anyhow::anyhow!(
                "Max fields per template must be greater than 0"
            ));
        }

        // Validate file configuration
        if self.file.max_file_size_mb == 0 {
            return Err(anyhow::anyhow!("Max file size must be greater than 0"));
        }

        if self.file.allowed_extensions.is_empty() {
            return Err(anyhow::anyhow!(
                "At least one file extension must be allowed"
            ));
        }

        // Validate versioning configuration
        if self.versioning.enable_versioning && self.versioning.max_versions_per_template == 0 {
            return Err(anyhow::anyhow!(
                "Max versions per template must be greater than 0 when versioning is enabled"
            ));
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

    /// Create test configuration for axum-test
    pub fn test_config() -> Self {
        Config {
            server: ServerConfig {
                host: "127.0.0.1".to_string(),
                port: 0, // Let OS assign port for tests
                workers: Some(1),
            },
            database_url: std::env::var("TEST_DATABASE_URL").unwrap_or_else(|_| {
                "postgres://postgres:postgres@localhost:5432/template_service_test".to_string()
            }),
            template: TemplateConfig {
                max_templates_per_user: 10,
                max_fields_per_template: 20,
                default_template_type: "test_template".to_string(),
                cache_templates: false, // Disabled for tests
                cache_ttl_seconds: 60,
                auto_backup: false, // Disabled for tests
            },
            form: FormConfig {
                enable_dynamic_forms: true,
                max_form_size_kb: 512,
                enable_form_preview: true,
                enable_conditional_fields: true,
                form_cache_enabled: false, // Disabled for tests
                render_timeout_seconds: 5,
            },
            validation: ValidationConfig {
                enable_strict_validation: true,
                max_validation_rules_per_field: 5,
                enable_cross_field_validation: true,
                enable_async_validation: false, // Disabled for tests
                validation_timeout_seconds: 2,
                cache_validation_results: false, // Disabled for tests
            },
            file: FileConfig {
                upload_path: "./test_uploads".to_string(),
                max_file_size_mb: 1,
                allowed_extensions: vec!["json".to_string(), "csv".to_string(), "txt".to_string()],
                enable_virus_scanning: false, // Disabled for tests
                enable_compression: false,    // Disabled for tests
                backup_enabled: false,        // Disabled for tests
                backup_retention_days: 1,
            },
            versioning: VersioningConfig {
                enable_versioning: true,
                max_versions_per_template: 3,
                auto_create_versions: false, // Disabled for tests
                version_naming_strategy: "sequential".to_string(),
                compress_old_versions: false, // Disabled for tests
            },
            services: ServiceConfig {
                auth_service_url: "http://localhost:8001".to_string(),
                sample_service_url: "http://localhost:8002".to_string(),
                storage_service_url: Some("http://localhost:8003".to_string()),
                notification_service_url: Some("http://localhost:8004".to_string()),
                request_timeout_seconds: 5,
                retry_attempts: 1,
                circuit_breaker_enabled: false, // Disabled for tests
            },
            logging: LoggingConfig {
                level: "debug".to_string(),
                format: "text".to_string(),
                file_enabled: false,
                file_path: None,
                audit_enabled: false, // Disabled for tests
            },
            features: FeatureConfig {
                form_builder_enabled: true,
                template_versioning_enabled: true,
                file_upload_enabled: true,
                template_cloning_enabled: true,
                advanced_validation_enabled: true,
                template_sharing_enabled: false, // Disabled for tests
                template_analytics_enabled: false, // Disabled for tests
            },
        }
    }

    /// Get the auth service URL
    pub fn auth_service_url(&self) -> &str {
        &self.services.auth_service_url
    }

    /// Get the sample service URL
    pub fn sample_service_url(&self) -> &str {
        &self.services.sample_service_url
    }

    /// Check if a feature is enabled
    pub fn is_feature_enabled(&self, feature: &str) -> bool {
        match feature {
            "form_builder" => self.features.form_builder_enabled,
            "template_versioning" => self.features.template_versioning_enabled,
            "file_upload" => self.features.file_upload_enabled,
            "template_cloning" => self.features.template_cloning_enabled,
            "advanced_validation" => self.features.advanced_validation_enabled,
            "template_sharing" => self.features.template_sharing_enabled,
            "template_analytics" => self.features.template_analytics_enabled,
            _ => false,
        }
    }

    /// Get file upload path
    pub fn upload_path(&self) -> &str {
        &self.file.upload_path
    }

    /// Check if file extension is allowed
    pub fn is_file_extension_allowed(&self, extension: &str) -> bool {
        self.file
            .allowed_extensions
            .contains(&extension.to_lowercase())
    }
}
