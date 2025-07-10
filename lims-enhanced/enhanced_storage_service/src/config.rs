use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::env;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub database_url: String,
    pub server: ServerConfig,
    pub storage: StorageConfig,
    pub security: SecurityConfig,
    pub features: FeatureConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerConfig {
    pub host: String,
    pub port: u16,
    pub workers: Option<usize>,
    pub keep_alive: Option<u64>,
    pub timeout: Option<u64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StorageConfig {
    pub max_hierarchy_depth: i32,
    pub default_temperature_zone: String,
    pub auto_generate_barcodes: bool,
    pub enable_capacity_alerts: bool,
    pub capacity_warning_threshold: f64,
    pub capacity_critical_threshold: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityConfig {
    pub enable_authentication: bool,
    pub jwt_secret: Option<String>,
    pub cors_origins: Vec<String>,
    pub rate_limiting: bool,
    pub max_requests_per_minute: Option<u32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FeatureConfig {
    pub enable_hierarchical_storage: bool,
    pub enable_sample_tracking: bool,
    pub enable_capacity_management: bool,
    pub enable_analytics: bool,
    pub enable_bulk_operations: bool,
    pub enable_audit_logging: bool,
}

impl Config {
    pub fn from_env() -> Result<Self, env::VarError> {
        Ok(Self {
            database_url: env::var("DATABASE_URL")
                .or_else(|_| env::var("STORAGE_DATABASE_URL"))
                .unwrap_or_else(|_| "postgres://postgres:postgres@localhost:5432/lims_db".to_string()),
            server: ServerConfig {
                host: env::var("HOST").unwrap_or_else(|_| "0.0.0.0".to_string()),
                port: env::var("PORT")
                    .unwrap_or_else(|_| "8082".to_string())
                    .parse()
                    .unwrap_or(8082),
                workers: env::var("WORKERS")
                    .ok()
                    .and_then(|w| w.parse().ok()),
                keep_alive: env::var("KEEP_ALIVE")
                    .ok()
                    .and_then(|k| k.parse().ok()),
                timeout: env::var("TIMEOUT")
                    .ok()
                    .and_then(|t| t.parse().ok()),
            },
            storage: StorageConfig {
                max_hierarchy_depth: env::var("MAX_HIERARCHY_DEPTH")
                    .unwrap_or_else(|_| "10".to_string())
                    .parse()
                    .unwrap_or(10),
                default_temperature_zone: env::var("DEFAULT_TEMPERATURE_ZONE")
                    .unwrap_or_else(|_| "room_temperature".to_string()),
                auto_generate_barcodes: env::var("AUTO_GENERATE_BARCODES")
                    .unwrap_or_else(|_| "true".to_string())
                    .parse()
                    .unwrap_or(true),
                enable_capacity_alerts: env::var("ENABLE_CAPACITY_ALERTS")
                    .unwrap_or_else(|_| "true".to_string())
                    .parse()
                    .unwrap_or(true),
                capacity_warning_threshold: env::var("CAPACITY_WARNING_THRESHOLD")
                    .unwrap_or_else(|_| "0.8".to_string())
                    .parse()
                    .unwrap_or(0.8),
                capacity_critical_threshold: env::var("CAPACITY_CRITICAL_THRESHOLD")
                    .unwrap_or_else(|_| "0.95".to_string())
                    .parse()
                    .unwrap_or(0.95),
            },
            security: SecurityConfig {
                enable_authentication: env::var("ENABLE_AUTHENTICATION")
                    .unwrap_or_else(|_| "false".to_string())
                    .parse()
                    .unwrap_or(false),
                jwt_secret: env::var("JWT_SECRET").ok(),
                cors_origins: env::var("CORS_ORIGINS")
                    .unwrap_or_else(|_| "*".to_string())
                    .split(',')
                    .map(|s| s.trim().to_string())
                    .collect(),
                rate_limiting: env::var("RATE_LIMITING")
                    .unwrap_or_else(|_| "false".to_string())
                    .parse()
                    .unwrap_or(false),
                max_requests_per_minute: env::var("MAX_REQUESTS_PER_MINUTE")
                    .ok()
                    .and_then(|r| r.parse().ok()),
            },
            features: FeatureConfig {
                enable_hierarchical_storage: env::var("ENABLE_HIERARCHICAL_STORAGE")
                    .unwrap_or_else(|_| "true".to_string())
                    .parse()
                    .unwrap_or(true),
                enable_sample_tracking: env::var("ENABLE_SAMPLE_TRACKING")
                    .unwrap_or_else(|_| "true".to_string())
                    .parse()
                    .unwrap_or(true),
                enable_capacity_management: env::var("ENABLE_CAPACITY_MANAGEMENT")
                    .unwrap_or_else(|_| "true".to_string())
                    .parse()
                    .unwrap_or(true),
                enable_analytics: env::var("ENABLE_ANALYTICS")
                    .unwrap_or_else(|_| "true".to_string())
                    .parse()
                    .unwrap_or(true),
                enable_bulk_operations: env::var("ENABLE_BULK_OPERATIONS")
                    .unwrap_or_else(|_| "true".to_string())
                    .parse()
                    .unwrap_or(true),
                enable_audit_logging: env::var("ENABLE_AUDIT_LOGGING")
                    .unwrap_or_else(|_| "true".to_string())
                    .parse()
                    .unwrap_or(true),
            },
        })
    }

    pub fn default() -> Self {
        Self {
            database_url: "postgres://postgres:postgres@localhost:5432/lims_db".to_string(),
            server: ServerConfig {
                host: "0.0.0.0".to_string(),
                port: 8082,
                workers: None,
                keep_alive: None,
                timeout: None,
            },
            storage: StorageConfig {
                max_hierarchy_depth: 10,
                default_temperature_zone: "room_temperature".to_string(),
                auto_generate_barcodes: true,
                enable_capacity_alerts: true,
                capacity_warning_threshold: 0.8,
                capacity_critical_threshold: 0.95,
            },
            security: SecurityConfig {
                enable_authentication: false,
                jwt_secret: None,
                cors_origins: vec!["*".to_string()],
                rate_limiting: false,
                max_requests_per_minute: None,
            },
            features: FeatureConfig {
                enable_hierarchical_storage: true,
                enable_sample_tracking: true,
                enable_capacity_management: true,
                enable_analytics: true,
                enable_bulk_operations: true,
                enable_audit_logging: true,
            },
        }
    }

    pub fn test_config() -> Self {
        Self {
            database_url: "postgres://postgres:postgres@localhost:5432/test_db".to_string(),
            ..Self::default()
        }
    }
}
