//! TracSeq Dashboard Service Library
//! 
//! This library provides dashboard aggregation and visualization
//! functionality for the TracSeq laboratory management system.

pub mod config;
pub mod handlers;
pub mod models;
pub mod services;

use axum::{
    http::StatusCode,
    response::Json,
};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::sync::Arc;

// Re-export commonly used types
pub use config::{Settings, ServiceUrls};

#[derive(Clone)]
pub struct AppState {
    pub pool: sqlx::PgPool,
    pub http_client: reqwest::Client,
    pub settings: Settings,
    pub cache: moka::future::Cache<String, DashboardData>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct DashboardData {
    pub timestamp: DateTime<Utc>,
    pub data: serde_json::Value,
    pub ttl_seconds: u64,
}