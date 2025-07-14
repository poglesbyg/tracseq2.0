use serde::{Deserialize, Serialize};
use std::env;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Settings {
    pub database_url: String,
    pub port: u16,
    pub service_urls: ServiceUrls,
    pub cache_ttl: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceUrls {
    pub auth_service: String,
    pub sample_service: String,
    pub storage_service: String,
    pub sequencing_service: String,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            database_url: env::var("DATABASE_URL")
                .unwrap_or_else(|_| "postgres://postgres:postgres@localhost:5432/lab_manager".to_string()),
            port: env::var("PORT")
                .unwrap_or_else(|_| "8080".to_string())
                .parse()
                .unwrap_or(8080),
            service_urls: ServiceUrls::default(),
            cache_ttl: 300, // 5 minutes
        }
    }
}

impl Default for ServiceUrls {
    fn default() -> Self {
        Self {
            auth_service: env::var("AUTH_SERVICE_URL")
                .unwrap_or_else(|_| "http://localhost:8001".to_string()),
            sample_service: env::var("SAMPLE_SERVICE_URL")
                .unwrap_or_else(|_| "http://localhost:8002".to_string()),
            storage_service: env::var("STORAGE_SERVICE_URL")
                .unwrap_or_else(|_| "http://localhost:8003".to_string()),
            sequencing_service: env::var("SEQUENCING_SERVICE_URL")
                .unwrap_or_else(|_| "http://localhost:8004".to_string()),
        }
    }
} 