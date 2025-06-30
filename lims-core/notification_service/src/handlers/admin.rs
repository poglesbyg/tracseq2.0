use axum::{
    extract::{Query, State},
    Json,
};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::{
    error::Result,
    models::*,
    AppState,
};

#[derive(Debug, Deserialize)]
pub struct StatisticsQuery {
    pub start_date: Option<chrono::DateTime<chrono::Utc>>,
    pub end_date: Option<chrono::DateTime<chrono::Utc>>,
    pub channel: Option<Channel>,
    pub priority: Option<Priority>,
}

#[derive(Debug, Serialize)]
pub struct NotificationStatistics {
    pub total_notifications: u64,
    pub sent_notifications: u64,
    pub failed_notifications: u64,
    pub pending_notifications: u64,
    pub delivery_rate: f64,
    pub avg_delivery_time_ms: u64,
    pub by_channel: HashMap<Channel, ChannelStatistics>,
    pub by_priority: HashMap<Priority, PriorityStatistics>,
    pub by_type: HashMap<NotificationType, TypeStatistics>,
    pub hourly_distribution: Vec<HourlyStats>,
    pub daily_distribution: Vec<DailyStats>,
}

#[derive(Debug, Serialize)]
pub struct ChannelStatistics {
    pub sent: u64,
    pub failed: u64,
    pub avg_response_time_ms: u64,
    pub success_rate: f64,
}

#[derive(Debug, Serialize)]
pub struct PriorityStatistics {
    pub sent: u64,
    pub failed: u64,
    pub avg_delivery_time_ms: u64,
}

#[derive(Debug, Serialize)]
pub struct TypeStatistics {
    pub sent: u64,
    pub failed: u64,
    pub most_common_channels: Vec<Channel>,
}

#[derive(Debug, Serialize)]
pub struct HourlyStats {
    pub hour: u32,
    pub count: u64,
}

#[derive(Debug, Serialize)]
pub struct DailyStats {
    pub date: chrono::NaiveDate,
    pub count: u64,
}

#[derive(Debug, Serialize)]
pub struct FailedNotificationResponse {
    pub id: uuid::Uuid,
    pub title: String,
    pub channels: Vec<Channel>,
    pub recipients: Vec<String>,
    pub error_message: Option<String>,
    pub last_attempt: chrono::DateTime<chrono::Utc>,
    pub attempts: i32,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Serialize)]
pub struct ChannelHealthResponse {
    pub channel: Channel,
    pub status: HealthStatus,
    pub last_test: Option<chrono::DateTime<chrono::Utc>>,
    pub last_success: Option<chrono::DateTime<chrono::Utc>>,
    pub error_rate: f64,
    pub avg_response_time_ms: u64,
}

#[derive(Debug, Serialize)]
pub enum HealthStatus {
    Healthy,
    Degraded,
    Unhealthy,
    Unknown,
}

#[derive(Debug, Serialize)]
pub struct RateLimitResponse {
    pub channel: Channel,
    pub requests_per_minute: u32,
    pub requests_per_hour: u32,
    pub requests_per_day: u32,
    pub current_usage: RateLimitUsage,
}

#[derive(Debug, Serialize)]
pub struct RateLimitUsage {
    pub minute: u32,
    pub hour: u32,
    pub day: u32,
}

#[derive(Debug, Deserialize)]
pub struct UpdateRateLimitRequest {
    pub channel: Channel,
    pub requests_per_minute: Option<u32>,
    pub requests_per_hour: Option<u32>,
    pub requests_per_day: Option<u32>,
}

/// Get notification statistics
/// GET /admin/statistics
pub async fn get_notification_statistics(
    State(state): State<AppState>,
    Query(query): Query<StatisticsQuery>,
) -> Result<Json<NotificationStatistics>> {
    let stats = state.notification_service.get_statistics(
        query.start_date,
        query.end_date,
        query.channel,
        query.priority,
    ).await?;

    Ok(Json(stats))
}

/// Get failed notifications
/// GET /admin/failed-notifications
pub async fn get_failed_notifications(
    State(state): State<AppState>,
    Query(query): Query<PaginationQuery>,
) -> Result<Json<Vec<FailedNotificationResponse>>> {
    let limit = query.limit.unwrap_or(50).min(1000);
    let offset = query.offset.unwrap_or(0);

    let failed_notifications = state.notification_service
        .get_failed_notifications(limit, offset)
        .await?;

    let responses = failed_notifications.into_iter().map(|notification| {
        FailedNotificationResponse {
            id: notification.id,
            title: notification.title,
            channels: notification.channels,
            recipients: notification.recipients,
            error_message: None, // TODO: Add error tracking
            last_attempt: notification.updated_at,
            attempts: notification.delivery_attempts,
            created_at: notification.created_at,
        }
    }).collect();

    Ok(Json(responses))
}

/// Retry failed notifications
/// POST /admin/retry-failed
pub async fn retry_failed_notifications(
    State(state): State<AppState>,
) -> Result<Json<serde_json::Value>> {
    let retried_count = state.notification_service.retry_failed_notifications().await?;

    Ok(Json(serde_json::json!({
        "message": "Failed notifications retry initiated",
        "retried_count": retried_count
    })))
}

/// Cleanup old notifications
/// POST /admin/cleanup
pub async fn cleanup_old_notifications(
    State(state): State<AppState>,
    Json(request): Json<CleanupRequest>,
) -> Result<Json<serde_json::Value>> {
    let deleted_count = state.notification_service.cleanup_old_notifications(
        request.older_than_days,
        request.keep_failed,
    ).await?;

    Ok(Json(serde_json::json!({
        "message": "Cleanup completed",
        "deleted_count": deleted_count
    })))
}

/// Check channel health
/// GET /admin/channels/health
pub async fn check_channel_health(
    State(state): State<AppState>,
) -> Result<Json<Vec<ChannelHealthResponse>>> {
    let health_checks = state.notification_service.check_all_channels_health().await?;
    
    Ok(Json(health_checks))
}

/// Get rate limits
/// GET /admin/rate-limits
pub async fn get_rate_limits(
    State(state): State<AppState>,
) -> Result<Json<Vec<RateLimitResponse>>> {
    let rate_limits = state.notification_service.get_rate_limits().await?;
    
    Ok(Json(rate_limits))
}

/// Update rate limits
/// PUT /admin/rate-limits
pub async fn update_rate_limits(
    State(state): State<AppState>,
    Json(request): Json<UpdateRateLimitRequest>,
) -> Result<Json<RateLimitResponse>> {
    let updated_limit = state.notification_service.update_rate_limit(
        request.channel,
        request.requests_per_minute,
        request.requests_per_hour,
        request.requests_per_day,
    ).await?;
    
    Ok(Json(updated_limit))
}

#[derive(Debug, Deserialize)]
pub struct PaginationQuery {
    pub limit: Option<i64>,
    pub offset: Option<i64>,
}

#[derive(Debug, Deserialize)]
pub struct CleanupRequest {
    pub older_than_days: u32,
    pub keep_failed: bool,
} 
