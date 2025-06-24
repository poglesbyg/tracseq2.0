use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::env;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub server: ServerConfig,
    pub database_url: String,
    pub auth_service_url: String,
    pub sample_service_url: String,
    pub notification_service_url: String,
    pub template_service_url: String,
    pub storage_service_url: String,
    pub sequencing: SequencingConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerConfig {
    pub host: String,
    pub port: u16,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SequencingConfig {
    pub max_concurrent_runs: usize,
    pub default_workflow: String,
    pub data_storage_path: String,
}

impl Config {
    pub fn from_env() -> Result<Self> {
        Ok(Self {
            server: ServerConfig {
                host: env::var("HOST").unwrap_or_else(|_| "0.0.0.0".to_string()),
                port: env::var("PORT")
                    .unwrap_or_else(|_| "8084".to_string())
                    .parse()
                    .unwrap_or(8084),
            },
            database_url: env::var("DATABASE_URL").unwrap_or_else(|_| {
                "postgresql://sequencing_user:sequencing_password@postgres:5432/sequencing_db"
                    .to_string()
            }),
            auth_service_url: env::var("AUTH_SERVICE_URL")
                .unwrap_or_else(|_| "http://auth-service:8080".to_string()),
            sample_service_url: env::var("SAMPLE_SERVICE_URL")
                .unwrap_or_else(|_| "http://sample-service:8081".to_string()),
            notification_service_url: env::var("NOTIFICATION_SERVICE_URL")
                .unwrap_or_else(|_| "http://notification-service:8085".to_string()),
            template_service_url: env::var("TEMPLATE_SERVICE_URL")
                .unwrap_or_else(|_| "http://template-service:8083".to_string()),
            storage_service_url: env::var("STORAGE_SERVICE_URL")
                .unwrap_or_else(|_| "http://storage-service:8082".to_string()),
            sequencing: SequencingConfig {
                max_concurrent_runs: env::var("MAX_CONCURRENT_RUNS")
                    .unwrap_or_else(|_| "3".to_string())
                    .parse()
                    .unwrap_or(3),
                default_workflow: env::var("DEFAULT_WORKFLOW")
                    .unwrap_or_else(|_| "standard_wgs".to_string()),
                data_storage_path: env::var("DATA_STORAGE_PATH")
                    .unwrap_or_else(|_| "/data/sequencing".to_string()),
            },
        })
    }

    pub fn get_platform(&self, platform_id: &str) -> Option<PlatformConfig> {
        // For now, return a mock platform config
        // In a real implementation, this would be loaded from configuration
        match platform_id {
            "illumina_novaseq" | "illumina_hiseq" | "ion_torrent" | "nanopore" => {
                Some(PlatformConfig {
                    id: platform_id.to_string(),
                    name: platform_id.to_string(),
                    manufacturer: "Generic".to_string(),
                    max_concurrent_runs: 2,
                })
            }
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlatformConfig {
    pub id: String,
    pub name: String,
    pub manufacturer: String,
    pub max_concurrent_runs: usize,
}
