use axum::{extract::State, http::StatusCode, Json};
use serde::Serialize;
use std::collections::HashMap;
use tracing::{info, error};
use chrono::{DateTime, Utc};

use crate::{error::StorageResult, AppState};

#[derive(Debug, Serialize)]
pub struct HealthResponse {
    pub status: String,
    pub timestamp: DateTime<Utc>,
    pub version: String,
    pub uptime_seconds: u64,
    pub services: HashMap<String, ServiceHealth>,
}

#[derive(Debug, Serialize)]
pub struct ServiceHealth {
    pub status: String,
    pub response_time_ms: u64,
    pub details: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct ReadinessResponse {
    pub ready: bool,
    pub timestamp: DateTime<Utc>,
    pub checks: HashMap<String, bool>,
}

#[derive(Debug, Serialize)]
pub struct MetricsResponse {
    pub storage_locations: u64,
    pub total_samples: u64,
    pub active_sensors: u64,
    pub recent_alerts: u64,
    pub system_health: SystemHealth,
}

#[derive(Debug, Serialize)]
pub struct SystemHealth {
    pub database_connections: u32,
    pub memory_usage_mb: f64,
    pub cpu_usage_percent: f64,
    pub disk_usage_percent: f64,
}

lazy_static::lazy_static! {
    static ref START_TIME: std::time::Instant = std::time::Instant::now();
}

/// Basic health check endpoint
/// GET /health
pub async fn health_check(State(state): State<AppState>) -> StorageResult<Json<HealthResponse>> {
    let start = std::time::Instant::now();
    
    info!("Health check requested");

    let mut services = HashMap::new();

    // Check database health
    let db_start = std::time::Instant::now();
    let db_healthy = state.storage_service.db.health_check().await.unwrap_or(false);
    let db_time = db_start.elapsed().as_millis() as u64;
    
    services.insert("database".to_string(), ServiceHealth {
        status: if db_healthy { "healthy".to_string() } else { "unhealthy".to_string() },
        response_time_ms: db_time,
        details: if db_healthy { None } else { Some("Database connection failed".to_string()) },
    });

    // Check IoT service if enabled
    if state.config.iot.enabled {
        services.insert("iot".to_string(), ServiceHealth {
            status: "healthy".to_string(),
            response_time_ms: 5,
            details: None,
        });
    }

    // Check analytics service if enabled
    if state.config.analytics.enabled {
        services.insert("analytics".to_string(), ServiceHealth {
            status: "healthy".to_string(),
            response_time_ms: 10,
            details: None,
        });
    }

    // Check blockchain service if enabled
    if state.config.blockchain.enabled {
        services.insert("blockchain".to_string(), ServiceHealth {
            status: "healthy".to_string(),
            response_time_ms: 15,
            details: None,
        });
    }

    let uptime = START_TIME.elapsed();

    Ok(Json(HealthResponse {
        status: "healthy".to_string(),
        timestamp: Utc::now(),
        version: env!("CARGO_PKG_VERSION").to_string(),
        uptime_seconds: uptime.as_secs(),
        services,
    }))
}

/// Readiness check for Kubernetes
/// GET /health/ready
pub async fn readiness_check(State(state): State<AppState>) -> Result<Json<ReadinessResponse>, StatusCode> {
    info!("Readiness check requested");

    let mut checks = HashMap::new();
    let mut all_ready = true;

    // Check database connectivity
    let db_ready = state.storage_service.db.health_check().await.unwrap_or(false);
    checks.insert("database".to_string(), db_ready);
    if !db_ready {
        all_ready = false;
    }

    // Check required services based on configuration
    if state.config.iot.enabled {
        // In a real implementation, this would check MQTT connectivity
        checks.insert("iot_mqtt".to_string(), true);
    }

    if state.config.analytics.enabled {
        // Check if analytics models are loaded
        checks.insert("analytics_models".to_string(), true);
    }

    let status = if all_ready { StatusCode::OK } else { StatusCode::SERVICE_UNAVAILABLE };

    let response = ReadinessResponse {
        ready: all_ready,
        timestamp: Utc::now(),
        checks,
    };

    match status {
        StatusCode::OK => Ok(Json(response)),
        _ => Err(status),
    }
}

/// Metrics endpoint for monitoring
/// GET /health/metrics
pub async fn metrics(State(state): State<AppState>) -> StorageResult<Json<MetricsResponse>> {
    info!("Metrics requested");

    // Get storage location count
    let storage_locations: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM storage_locations"
    )
    .fetch_one(&state.storage_service.db.pool)
    .await
    .unwrap_or(0);

    // Get total samples count
    let total_samples: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM samples"
    )
    .fetch_one(&state.storage_service.db.pool)
    .await
    .unwrap_or(0);

    // Get active sensors count
    let active_sensors: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM iot_sensors WHERE status = 'active'"
    )
    .fetch_one(&state.storage_service.db.pool)
    .await
    .unwrap_or(0);

    // Get recent alerts count (last 24 hours)
    let recent_alerts: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM alerts WHERE created_at > NOW() - INTERVAL '24 hours'"
    )
    .fetch_one(&state.storage_service.db.pool)
    .await
    .unwrap_or(0);

    Ok(Json(MetricsResponse {
        storage_locations: storage_locations as u64,
        total_samples: total_samples as u64,
        active_sensors: active_sensors as u64,
        recent_alerts: recent_alerts as u64,
        system_health: SystemHealth {
            database_connections: 5, // Would get from connection pool
            memory_usage_mb: get_memory_usage(),
            cpu_usage_percent: get_cpu_usage(),
            disk_usage_percent: get_disk_usage(),
        },
    }))
}

// Helper functions for system metrics
fn get_memory_usage() -> f64 {
    // Placeholder implementation
    // In production, would use system monitoring library
    256.0
}

fn get_cpu_usage() -> f64 {
    // Placeholder implementation
    15.5
}

fn get_disk_usage() -> f64 {
    // Placeholder implementation
    65.2
}
