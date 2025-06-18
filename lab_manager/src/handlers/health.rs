use axum::{extract::State, http::StatusCode, Json};
use serde::Serialize;
use std::collections::HashMap;
use tracing::{error, info};

use crate::{assembly::AppComponents, config::database::health_check as db_health_check};

// Local type definitions as workaround for import issues
#[derive(Debug, Clone, Serialize)]
pub enum ServiceStatus {
    Healthy,
    Degraded,
    Unhealthy,
    Unknown,
}

#[derive(Debug, Clone, Serialize)]
pub enum MetricValue {
    Counter(u64),
    Gauge(f64),
    Histogram(Vec<f64>),
    Timer(std::time::Duration),
}

#[derive(Debug, Clone, Serialize)]
pub struct HealthStatus {
    pub is_healthy: bool,
    pub status: ServiceStatus,
    pub message: Option<String>,
    pub response_time_ms: u64,
    pub last_checked: chrono::DateTime<chrono::Utc>,
}

/// Basic health check response
#[derive(Debug, Serialize)]
pub struct HealthResponse {
    pub status: String,
    pub timestamp: chrono::DateTime<chrono::Utc>,
    pub version: String,
    pub uptime_seconds: u64,
}

/// Comprehensive system health check response
#[derive(Debug, Serialize)]
pub struct SystemHealthResponse {
    pub overall_status: ServiceStatus,
    pub timestamp: chrono::DateTime<chrono::Utc>,
    pub version: String,
    pub uptime_seconds: u64,
    pub services: HashMap<String, HealthStatus>,
    pub system_info: SystemInfo,
}

/// System information
#[derive(Debug, Serialize)]
pub struct SystemInfo {
    pub hostname: String,
    pub platform: String,
    pub architecture: String,
    pub rust_version: String,
    pub environment: String,
}

/// Database health details
#[derive(Debug, Serialize)]
pub struct DatabaseHealthDetails {
    pub is_connected: bool,
    pub response_time_ms: u64,
    pub active_connections: u32,
    pub idle_connections: u32,
    pub max_connections: u32,
    pub database_version: Option<String>,
}

/// Application metrics for monitoring
#[derive(Debug, Serialize)]
pub struct ApplicationMetrics {
    pub total_requests: u64,
    pub active_requests: u32,
    pub error_rate_percent: f64,
    pub average_response_time_ms: f64,
    pub memory_usage_mb: f64,
    pub cpu_usage_percent: f64,
}

lazy_static::lazy_static! {
    static ref START_TIME: std::time::Instant = std::time::Instant::now();
}

/// Basic health check endpoint
/// GET /health
pub async fn health_check() -> Result<Json<HealthResponse>, StatusCode> {
    let uptime = START_TIME.elapsed();

    info!("Health check requested");

    Ok(Json(HealthResponse {
        status: "healthy".to_string(),
        timestamp: chrono::Utc::now(),
        version: env!("CARGO_PKG_VERSION").to_string(),
        uptime_seconds: uptime.as_secs(),
    }))
}

/// Comprehensive system health check
/// GET /health/system
pub async fn system_health_check(
    State(app): State<AppComponents>,
) -> Result<Json<SystemHealthResponse>, StatusCode> {
    let uptime = START_TIME.elapsed();

    info!("System health check requested");

    // Check all registered health checks
    let service_results = app.observability.health_checker.check_all().await;

    // Convert types for compatibility
    let converted_results: HashMap<String, HealthStatus> = service_results
        .into_iter()
        .map(|(k, v)| (k, convert_health_status(v)))
        .collect();

    // Determine overall system health
    let overall_status = determine_overall_status(&converted_results);

    // Gather system information
    let system_info = SystemInfo {
        hostname: get_hostname(),
        platform: std::env::consts::OS.to_string(),
        architecture: std::env::consts::ARCH.to_string(),
        rust_version: get_rust_version(),
        environment: std::env::var("ENVIRONMENT").unwrap_or_else(|_| "unknown".to_string()),
    };

    let response = SystemHealthResponse {
        overall_status,
        timestamp: chrono::Utc::now(),
        version: env!("CARGO_PKG_VERSION").to_string(),
        uptime_seconds: uptime.as_secs(),
        services: converted_results,
        system_info,
    };

    // Return appropriate status code based on health
    let _status_code = match response.overall_status {
        ServiceStatus::Healthy => StatusCode::OK,
        ServiceStatus::Degraded => StatusCode::OK, // Still operational
        ServiceStatus::Unhealthy => StatusCode::SERVICE_UNAVAILABLE,
        ServiceStatus::Unknown => StatusCode::INTERNAL_SERVER_ERROR,
    };

    Ok(Json(response))
}

/// Database-specific health check
/// GET /health/database
pub async fn database_health_check(
    State(app): State<AppComponents>,
) -> Result<Json<DatabaseHealthDetails>, StatusCode> {
    info!("Database health check requested");

    match db_health_check(&app.database.pool).await {
        Ok(health) => {
            let database_version = get_database_version(&app.database.pool).await;

            Ok(Json(DatabaseHealthDetails {
                is_connected: health.is_healthy,
                response_time_ms: health.response_time_ms,
                active_connections: health.active_connections,
                idle_connections: health.idle_connections,
                max_connections: health.max_connections,
                database_version,
            }))
        }
        Err(e) => {
            error!("Database health check failed: {}", e);
            Err(StatusCode::SERVICE_UNAVAILABLE)
        }
    }
}

/// Application metrics endpoint
/// GET /health/metrics
pub async fn application_metrics(
    State(app): State<AppComponents>,
) -> Result<Json<ApplicationMetrics>, StatusCode> {
    info!("Application metrics requested");

    let obs_metrics = app.observability.metrics.get_all_metrics().await;

    // Convert metrics types for compatibility
    let metrics = convert_metrics_map(obs_metrics);

    // Extract specific metrics (these would be collected by the metrics system)
    let total_requests = extract_counter_metric(&metrics, "http_requests_total").unwrap_or(0);
    let active_requests =
        extract_gauge_metric(&metrics, "http_requests_active").unwrap_or(0.0) as u32;
    let error_rate = calculate_error_rate(&metrics);
    let avg_response_time =
        extract_histogram_average(&metrics, "http_request_duration_ms").unwrap_or(0.0);
    let memory_usage = get_memory_usage_mb();
    let cpu_usage = get_cpu_usage_percent();

    Ok(Json(ApplicationMetrics {
        total_requests,
        active_requests,
        error_rate_percent: error_rate,
        average_response_time_ms: avg_response_time,
        memory_usage_mb: memory_usage,
        cpu_usage_percent: cpu_usage,
    }))
}

/// Readiness probe for Kubernetes
/// GET /health/ready
pub async fn readiness_check(
    State(app): State<AppComponents>,
) -> Result<Json<HealthResponse>, StatusCode> {
    info!("Readiness check requested");

    // Check critical dependencies
    match db_health_check(&app.database.pool).await {
        Ok(health) if health.is_healthy => {
            // Check RAG service if enabled
            let rag_status = app
                .observability
                .health_checker
                .check_single("rag_service")
                .await;

            if let Some(rag_health) = rag_status {
                if !rag_health.is_healthy {
                    return Err(StatusCode::SERVICE_UNAVAILABLE);
                }
            }

            let uptime = START_TIME.elapsed();
            Ok(Json(HealthResponse {
                status: "ready".to_string(),
                timestamp: chrono::Utc::now(),
                version: env!("CARGO_PKG_VERSION").to_string(),
                uptime_seconds: uptime.as_secs(),
            }))
        }
        _ => {
            error!("Readiness check failed - database not healthy");
            Err(StatusCode::SERVICE_UNAVAILABLE)
        }
    }
}

/// Liveness probe for Kubernetes
/// GET /health/live
pub async fn liveness_check() -> Result<Json<HealthResponse>, StatusCode> {
    info!("Liveness check requested");

    let uptime = START_TIME.elapsed();

    // Basic liveness - if we can respond, we're alive
    Ok(Json(HealthResponse {
        status: "alive".to_string(),
        timestamp: chrono::Utc::now(),
        version: env!("CARGO_PKG_VERSION").to_string(),
        uptime_seconds: uptime.as_secs(),
    }))
}

/// Determine overall system status from individual service results
fn determine_overall_status(services: &HashMap<String, HealthStatus>) -> ServiceStatus {
    if services.is_empty() {
        return ServiceStatus::Unknown;
    }

    let mut healthy_count = 0;
    let mut degraded_count = 0;
    let mut unhealthy_count = 0;

    for health in services.values() {
        match health.status {
            ServiceStatus::Healthy => healthy_count += 1,
            ServiceStatus::Degraded => degraded_count += 1,
            ServiceStatus::Unhealthy => unhealthy_count += 1,
            ServiceStatus::Unknown => {}
        }
    }

    // Determine overall status
    if unhealthy_count > 0 {
        ServiceStatus::Unhealthy
    } else if degraded_count > 0 {
        ServiceStatus::Degraded
    } else if healthy_count > 0 {
        ServiceStatus::Healthy
    } else {
        ServiceStatus::Unknown
    }
}

/// Get database version information
async fn get_database_version(pool: &sqlx::PgPool) -> Option<String> {
    match sqlx::query_scalar::<_, String>("SELECT version()")
        .fetch_one(pool)
        .await
    {
        Ok(version) => Some(version),
        Err(e) => {
            error!("Failed to get database version: {}", e);
            None
        }
    }
}

/// Extract counter metric value
fn extract_counter_metric(metrics: &HashMap<String, MetricValue>, name: &str) -> Option<u64> {
    if let Some(MetricValue::Counter(value)) = metrics.get(name) {
        Some(*value)
    } else {
        None
    }
}

/// Extract gauge metric value
fn extract_gauge_metric(metrics: &HashMap<String, MetricValue>, name: &str) -> Option<f64> {
    if let Some(MetricValue::Gauge(value)) = metrics.get(name) {
        Some(*value)
    } else {
        None
    }
}

/// Calculate error rate from metrics
fn calculate_error_rate(metrics: &HashMap<String, MetricValue>) -> f64 {
    let total_requests = extract_counter_metric(metrics, "http_requests_total").unwrap_or(0) as f64;
    let error_requests =
        extract_counter_metric(metrics, "http_requests_errors").unwrap_or(0) as f64;

    if total_requests > 0.0 {
        (error_requests / total_requests) * 100.0
    } else {
        0.0
    }
}

/// Extract histogram average
fn extract_histogram_average(metrics: &HashMap<String, MetricValue>, name: &str) -> Option<f64> {
    if let Some(MetricValue::Histogram(values)) = metrics.get(name) {
        if !values.is_empty() {
            let sum: f64 = values.iter().sum();
            Some(sum / values.len() as f64)
        } else {
            Some(0.0)
        }
    } else {
        None
    }
}

/// Get system hostname
fn get_hostname() -> String {
    std::env::var("HOSTNAME")
        .or_else(|_| std::env::var("COMPUTERNAME"))
        .unwrap_or_else(|_| "unknown".to_string())
}

/// Get Rust version
fn get_rust_version() -> String {
    // Use the standard rustc version environment variable, or provide fallback
    option_env!("RUSTC_VERSION")
        .or_else(|| option_env!("RUSTUP_TOOLCHAIN"))
        .unwrap_or("unknown")
        .to_string()
}

/// Get memory usage in MB (placeholder implementation)
fn get_memory_usage_mb() -> f64 {
    // This would require a system monitoring library like `sysinfo`
    // For now, return a placeholder value
    0.0
}

/// Get CPU usage percentage (placeholder implementation)
fn get_cpu_usage_percent() -> f64 {
    // This would require a system monitoring library like `sysinfo`
    // For now, return a placeholder value
    0.0
}

// Type conversion functions to bridge observability and local types
fn convert_health_status(obs_status: crate::assembly::HealthStatus) -> HealthStatus {
    HealthStatus {
        is_healthy: obs_status.is_healthy,
        status: match obs_status.status {
            crate::assembly::ServiceStatus::Healthy => ServiceStatus::Healthy,
            crate::assembly::ServiceStatus::Degraded => ServiceStatus::Degraded,
            crate::assembly::ServiceStatus::Unhealthy => ServiceStatus::Unhealthy,
            crate::assembly::ServiceStatus::Unknown => ServiceStatus::Unknown,
        },
        message: obs_status.message,
        response_time_ms: obs_status.response_time_ms,
        last_checked: obs_status.last_checked,
    }
}

fn convert_metric_value(obs_metric: crate::assembly::MetricValue) -> MetricValue {
    match obs_metric {
        crate::assembly::MetricValue::Counter(v) => MetricValue::Counter(v),
        crate::assembly::MetricValue::Gauge(v) => MetricValue::Gauge(v),
        crate::assembly::MetricValue::Histogram(v) => MetricValue::Histogram(v),
        crate::assembly::MetricValue::Timer(v) => MetricValue::Timer(v),
    }
}

fn convert_metrics_map(
    obs_metrics: HashMap<String, crate::assembly::MetricValue>,
) -> HashMap<String, MetricValue> {
    obs_metrics
        .into_iter()
        .map(|(k, v)| (k, convert_metric_value(v)))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_determine_overall_status() {
        let mut services = HashMap::new();

        // All healthy
        services.insert(
            "db".to_string(),
            HealthStatus {
                is_healthy: true,
                status: ServiceStatus::Healthy,
                message: None,
                response_time_ms: 10,
                last_checked: chrono::Utc::now(),
            },
        );

        assert!(matches!(
            determine_overall_status(&services),
            ServiceStatus::Healthy
        ));

        // One degraded
        services.insert(
            "rag".to_string(),
            HealthStatus {
                is_healthy: true,
                status: ServiceStatus::Degraded,
                message: None,
                response_time_ms: 100,
                last_checked: chrono::Utc::now(),
            },
        );

        assert!(matches!(
            determine_overall_status(&services),
            ServiceStatus::Degraded
        ));

        // One unhealthy
        services.insert(
            "external".to_string(),
            HealthStatus {
                is_healthy: false,
                status: ServiceStatus::Unhealthy,
                message: None,
                response_time_ms: 0,
                last_checked: chrono::Utc::now(),
            },
        );

        assert!(matches!(
            determine_overall_status(&services),
            ServiceStatus::Unhealthy
        ));
    }
}
