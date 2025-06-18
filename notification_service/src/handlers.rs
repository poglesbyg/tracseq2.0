use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    Json,
};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

use crate::error::{NotificationError, Result};
use crate::models::*;
use crate::service::NotificationService;

// ================================
// Query Parameters
// ================================

#[derive(Debug, Deserialize)]
pub struct ListQuery {
    pub limit: Option<i64>,
    pub offset: Option<i64>,
    pub status: Option<String>,
    pub channel: Option<String>,
    pub priority: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct TestChannelQuery {
    pub recipient: String,
    pub message: Option<String>,
}

// ================================
// Health Check Handler
// ================================

pub async fn health_check() -> Result<Json<serde_json::Value>> {
    Ok(Json(serde_json::json!({
        "status": "healthy",
        "service": "notification-service",
        "timestamp": chrono::Utc::now(),
        "version": env!("CARGO_PKG_VERSION")
    })))
}

// ================================
// Notification Handlers
// ================================

/// Send a notification
/// POST /notifications
pub async fn send_notification(
    State(service): State<NotificationService>,
    Json(request): Json<CreateNotificationRequest>,
) -> Result<Json<NotificationResponse>> {
    // TODO: Extract user ID from JWT token
    let created_by = Uuid::new_v4(); // Placeholder

    let response = service.send_notification(request, created_by).await?;
    Ok(Json(response))
}

/// Get a specific notification
/// GET /notifications/{id}
pub async fn get_notification(
    State(service): State<NotificationService>,
    Path(notification_id): Path<Uuid>,
) -> Result<Json<Notification>> {
    let notification = service.get_notification(notification_id).await?;
    Ok(Json(notification))
}

/// List notifications with filtering
/// GET /notifications
pub async fn list_notifications(
    State(service): State<NotificationService>,
    Query(query): Query<ListQuery>,
) -> Result<Json<Vec<Notification>>> {
    let notifications = service.list_notifications(query.limit, query.offset).await?;
    Ok(Json(notifications))
}

/// Test email channel
/// POST /channels/email/test
pub async fn test_email_channel(
    State(service): State<NotificationService>,
    Query(query): Query<TestChannelQuery>,
) -> Result<Json<ChannelTestResponse>> {
    let start_time = std::time::Instant::now();
    
    let test_notification = Notification {
        id: Uuid::new_v4(),
        title: "Test Email".to_string(),
        message: query.message.unwrap_or_else(|| "This is a test email from the notification service.".to_string()),
        notification_type: NotificationType::Info,
        priority: Priority::Low,
        status: NotificationStatus::Pending,
        channels: vec![Channel::Email],
        recipients: vec![query.recipient.clone()],
        template_id: None,
        template_data: None,
        scheduled_at: None,
        sent_at: None,
        delivery_attempts: 0,
        metadata: serde_json::json!({}),
        created_by: Uuid::new_v4(),
        created_at: chrono::Utc::now(),
        updated_at: chrono::Utc::now(),
    };

    let result = service.send_email(&test_notification, &query.recipient).await;
    let response_time = start_time.elapsed().as_millis() as u64;

    Ok(Json(ChannelTestResponse {
        channel: Channel::Email,
        success: result.is_ok(),
        message: if result.is_ok() {
            "Test email sent successfully".to_string()
        } else {
            result.unwrap_err().to_string()
        },
        response_time_ms: response_time,
        error_details: None,
    }))
}

/// Get notification statistics
/// GET /analytics/stats
pub async fn get_notification_stats(
    State(service): State<NotificationService>,
    Query(query): Query<HashMap<String, String>>,
) -> Result<Json<serde_json::Value>> {
    // TODO: Implement analytics
    Ok(Json(serde_json::json!({
        "total_notifications": 0,
        "sent_today": 0,
        "failed_today": 0,
        "delivery_rate": 0.0,
        "channels": {
            "email": { "sent": 0, "failed": 0 },
            "sms": { "sent": 0, "failed": 0 },
            "slack": { "sent": 0, "failed": 0 }
        }
    })))
}
