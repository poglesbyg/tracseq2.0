use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::env;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub server: ServerConfig,
    pub database_url: String,
    pub environment: String,
    pub barcode: BarcodeConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerConfig {
    pub host: String,
    pub port: u16,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BarcodeConfig {
    pub prefix: String,
    pub separator: String,
    pub min_length: usize,
    pub include_date: bool,
    pub include_sequence: bool,
    pub validation_pattern: String,
}

impl Default for BarcodeConfig {
    fn default() -> Self {
        Self {
            prefix: "LAB".to_string(),
            separator: "-".to_string(),
            min_length: 8,
            include_date: true,
            include_sequence: true,
            validation_pattern: r"^[A-Z0-9\-_]+$".to_string(),
        }
    }
}

impl Config {
    pub fn from_env() -> Result<Self> {
        Ok(Self {
            server: ServerConfig {
                host: env::var("HOST").unwrap_or_else(|_| "0.0.0.0".to_string()),
                port: env::var("PORT")
                    .unwrap_or_else(|_| "8090".to_string())
                    .parse()
                    .unwrap_or(8090),
            },
            database_url: env::var("DATABASE_URL").unwrap_or_else(|_| {
                "postgresql://barcode_user:barcode_password@postgres:5432/barcode_db".to_string()
            }),
            environment: env::var("ENVIRONMENT").unwrap_or_else(|_| "development".to_string()),
            barcode: BarcodeConfig {
                prefix: env::var("BARCODE_PREFIX").unwrap_or_else(|_| "LAB".to_string()),
                separator: env::var("BARCODE_SEPARATOR").unwrap_or_else(|_| "-".to_string()),
                min_length: env::var("BARCODE_MIN_LENGTH")
                    .unwrap_or_else(|_| "8".to_string())
                    .parse()
                    .unwrap_or(8),
                include_date: env::var("BARCODE_INCLUDE_DATE")
                    .unwrap_or_else(|_| "true".to_string())
                    .parse()
                    .unwrap_or(true),
                include_sequence: env::var("BARCODE_INCLUDE_SEQUENCE")
                    .unwrap_or_else(|_| "true".to_string())
                    .parse()
                    .unwrap_or(true),
                validation_pattern: env::var("BARCODE_VALIDATION_PATTERN")
                    .unwrap_or_else(|_| r"^[A-Z0-9\-_]+$".to_string()),
            },
        })
    }
} 