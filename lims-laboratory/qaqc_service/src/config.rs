use serde::{Deserialize, Serialize};
use std::env;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub host: String,
    pub port: u16,
    pub database_url: String,
    pub jwt_secret: String,
    pub metrics_port: u16,

    // Service URLs for integration
    pub auth_service_url: String,
    pub sample_service_url: String,
    pub sequencing_service_url: String,
    pub spreadsheet_versioning_service_url: String,

    // QAQC Configuration
    pub default_quality_threshold: f64,
    pub enable_real_time_monitoring: bool,
    pub alert_notification_url: Option<String>,
    pub compliance_standards: Vec<String>,

    // Performance settings
    pub max_concurrent_workflows: usize,
    pub workflow_timeout_seconds: u64,
    pub metrics_retention_days: u32,
}

impl Config {
    pub fn from_env() -> Result<Self, env::VarError> {
        Ok(Self {
            host: env::var("QAQC_HOST").unwrap_or_else(|_| "0.0.0.0".to_string()),
            port: env::var("QAQC_PORT")
                .unwrap_or_else(|_| "8089".to_string())
                .parse()
                .unwrap_or(8089),
            database_url: env::var("DATABASE_URL")?,
            jwt_secret: env::var("JWT_SECRET")?,
            metrics_port: env::var("METRICS_PORT")
                .unwrap_or_else(|_| "9089".to_string())
                .parse()
                .unwrap_or(9089),

            // Service URLs
            auth_service_url: env::var("AUTH_SERVICE_URL")
                .unwrap_or_else(|_| "http://auth-service:8080".to_string()),
            sample_service_url: env::var("SAMPLE_SERVICE_URL")
                .unwrap_or_else(|_| "http://sample-service:8081".to_string()),
            sequencing_service_url: env::var("SEQUENCING_SERVICE_URL")
                .unwrap_or_else(|_| "http://sequencing-service:8084".to_string()),
            spreadsheet_versioning_service_url: env::var("SPREADSHEET_VERSIONING_SERVICE_URL")
                .unwrap_or_else(|_| "http://spreadsheet-versioning-service:8088".to_string()),

            // QAQC settings
            default_quality_threshold: env::var("DEFAULT_QUALITY_THRESHOLD")
                .unwrap_or_else(|_| "80.0".to_string())
                .parse()
                .unwrap_or(80.0),
            enable_real_time_monitoring: env::var("ENABLE_REAL_TIME_MONITORING")
                .unwrap_or_else(|_| "true".to_string())
                .parse()
                .unwrap_or(true),
            alert_notification_url: env::var("ALERT_NOTIFICATION_URL").ok(),
            compliance_standards: env::var("COMPLIANCE_STANDARDS")
                .unwrap_or_else(|_| "ISO15189,CLIA,CAP".to_string())
                .split(',')
                .map(|s| s.trim().to_string())
                .collect(),

            // Performance settings
            max_concurrent_workflows: env::var("MAX_CONCURRENT_WORKFLOWS")
                .unwrap_or_else(|_| "10".to_string())
                .parse()
                .unwrap_or(10),
            workflow_timeout_seconds: env::var("WORKFLOW_TIMEOUT_SECONDS")
                .unwrap_or_else(|_| "3600".to_string())
                .parse()
                .unwrap_or(3600),
            metrics_retention_days: env::var("METRICS_RETENTION_DAYS")
                .unwrap_or_else(|_| "90".to_string())
                .parse()
                .unwrap_or(90),
        })
    }
}
