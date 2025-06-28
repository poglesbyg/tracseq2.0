use axum::{
    extract::State,
    Json,
};
use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};

use crate::{error::Result, AppState};

#[derive(Debug, Serialize)]
pub struct HealthResponse {
    pub status: String,
    pub service: String,
    pub version: String,
    pub timestamp: DateTime<Utc>,
    pub database: HealthStatus,
    pub channels: ChannelHealthStatus,
}

#[derive(Debug, Serialize)]
pub struct HealthStatus {
    pub status: String,
    pub response_time_ms: u64,
}

#[derive(Debug, Serialize)]
pub struct ChannelHealthStatus {
    pub email: Option<HealthStatus>,
    pub sms: Option<HealthStatus>,
    pub slack: Option<HealthStatus>,
    pub teams: Option<HealthStatus>,
}

#[derive(Debug, Serialize)]
pub struct MetricsResponse {
    pub notifications_sent: u64,
    pub notifications_failed: u64,
    pub delivery_rate: f64,
    pub avg_delivery_time_ms: u64,
    pub channels: ChannelMetrics,
}

#[derive(Debug, Serialize)]
pub struct ChannelMetrics {
    pub email: ChannelStats,
    pub sms: ChannelStats,
    pub slack: ChannelStats,
    pub teams: ChannelStats,
}

#[derive(Debug, Serialize)]
pub struct ChannelStats {
    pub sent: u64,
    pub failed: u64,
    pub avg_response_time_ms: u64,
}

/// Basic health check
/// GET /health
pub async fn health_check(State(state): State<AppState>) -> Result<Json<HealthResponse>> {
    let start_time = std::time::Instant::now();
    
    // Test database connection
    let db_status = match sqlx::query("SELECT 1").execute(&state.notification_service.database.pool).await {
        Ok(_) => HealthStatus {
            status: "healthy".to_string(),
            response_time_ms: start_time.elapsed().as_millis() as u64,
        },
        Err(_) => HealthStatus {
            status: "unhealthy".to_string(),
            response_time_ms: start_time.elapsed().as_millis() as u64,
        },
    };

    // Test channels - simplified for now
    let channels = ChannelHealthStatus {
        email: Some(HealthStatus {
            status: "healthy".to_string(),
            response_time_ms: 0,
        }),
        sms: Some(HealthStatus {
            status: "healthy".to_string(),
            response_time_ms: 0,
        }),
        slack: Some(HealthStatus {
            status: "healthy".to_string(),
            response_time_ms: 0,
        }),
        teams: Some(HealthStatus {
            status: "healthy".to_string(),
            response_time_ms: 0,
        }),
    };

    Ok(Json(HealthResponse {
        status: "healthy".to_string(),
        service: "notification-service".to_string(),
        version: env!("CARGO_PKG_VERSION").to_string(),
        timestamp: Utc::now(),
        database: db_status,
        channels,
    }))
}

/// Readiness check
/// GET /health/ready
pub async fn readiness_check(State(state): State<AppState>) -> Result<Json<serde_json::Value>> {
    // Check if all dependencies are ready
    let db_ready = sqlx::query("SELECT 1").execute(&state.notification_service.database.pool).await.is_ok();
    
    if !db_ready {
        return Err(crate::error::NotificationError::ServiceUnavailable(
            "Database not ready".to_string()
        ));
    }

    Ok(Json(serde_json::json!({
        "status": "ready",
        "service": "notification-service",
        "timestamp": Utc::now()
    })))
}

/// Service metrics
/// GET /health/metrics
pub async fn metrics(State(state): State<AppState>) -> Result<Json<MetricsResponse>> {
    // Get metrics from service
    let metrics = state.notification_service.get_metrics().await?;
    
    Ok(Json(MetricsResponse {
        notifications_sent: metrics.notifications_sent,
        notifications_failed: metrics.notifications_failed,
        delivery_rate: metrics.delivery_rate,
        avg_delivery_time_ms: metrics.avg_delivery_time_ms,
        channels: ChannelMetrics {
            email: ChannelStats {
                sent: metrics.email_sent,
                failed: metrics.email_failed,
                avg_response_time_ms: metrics.email_avg_response_time_ms,
            },
            sms: ChannelStats {
                sent: metrics.sms_sent,
                failed: metrics.sms_failed,
                avg_response_time_ms: metrics.sms_avg_response_time_ms,
            },
            slack: ChannelStats {
                sent: metrics.slack_sent,
                failed: metrics.slack_failed,
                avg_response_time_ms: metrics.slack_avg_response_time_ms,
            },
            teams: ChannelStats {
                sent: metrics.teams_sent,
                failed: metrics.teams_failed,
                avg_response_time_ms: metrics.teams_avg_response_time_ms,
            },
        },
    }))
} 
