use serde::Deserialize;
use std::env;

#[derive(Debug, Clone, Deserialize)]
pub struct Config {
    pub host: String,
    pub port: u16,
    pub database_url: String,
    pub auth_secret: String,
    pub log_level: String,
}

impl Config {
    pub fn from_env() -> Result<Self, Box<dyn std::error::Error>> {
        let host = env::var("LIBRARY_DETAILS_HOST").unwrap_or_else(|_| "0.0.0.0".to_string());
        let port = env::var("LIBRARY_DETAILS_PORT")
            .unwrap_or_else(|_| "8080".to_string())
            .parse::<u16>()?;
        let database_url = env::var("DATABASE_URL")
            .or_else(|_| env::var("LIBRARY_DETAILS_DATABASE_URL"))
            .unwrap_or_else(|_| "postgresql://localhost/tracseq".to_string());
        let auth_secret = env::var("AUTH_SECRET")
            .unwrap_or_else(|_| "default-secret-change-in-production".to_string());
        let log_level = env::var("LOG_LEVEL")
            .unwrap_or_else(|_| "info".to_string());

        Ok(Config {
            host,
            port,
            database_url,
            auth_secret,
            log_level,
        })
    }
}