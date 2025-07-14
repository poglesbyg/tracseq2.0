use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DashboardStats {
    pub total_samples: i64,
    pub active_sequencing_jobs: i64,
    pub storage_utilization: f64,
    pub pending_reviews: i64,
    pub timestamp: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemHealth {
    pub services: ServiceHealthStatus,
    pub database: String,
    pub timestamp: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceHealthStatus {
    pub auth: String,
    pub sample: String,
    pub storage: String,
    pub sequencing: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceMetrics {
    pub service_name: String,
    pub response_time_ms: f64,
    pub error_rate: f64,
    pub throughput: f64,
    pub timestamp: DateTime<Utc>,
} 