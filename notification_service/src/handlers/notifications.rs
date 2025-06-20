use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    Json,
};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::{
    error::{NotificationError, Result},
    models::*,
    AppState,
};

#[derive(Debug, Deserialize)]
pub struct ListNotificationsQuery {
    pub limit: Option<i64>,
    pub offset: Option<i64>,
    pub status: Option<NotificationStatus>,
    pub channel: Option<Channel>,
    pub priority: Option<Priority>,
    pub notification_type: Option<NotificationType>,
    pub user_id: Option<Uuid>,
    pub created_after: Option<chrono::DateTime<chrono::Utc>>,
    pub created_before: Option<chrono::DateTime<chrono::Utc>>,
}

#[derive(Debug, Deserialize)]
pub struct SendNotificationRequest {
    pub title: String,
    pub message: String,
    pub notification_type: NotificationType,
    pub priority: Priority,
    pub channels: Vec<Channel>,
    pub recipients: Vec<String>,
    pub template_id: Option<Uuid>,
    pub template_data: Option<serde_json::Value>,
    pub scheduled_at: Option<chrono::DateTime<chrono::Utc>>,
    pub metadata: Option<serde_json::Value>,
}

#[derive(Debug, Deserialize)]
pub struct BulkNotificationRequest {
    pub notifications: Vec<SendNotificationRequest>,
    pub batch_id: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct NotificationResponse {
    pub id: Uuid,
    pub status: NotificationStatus,
    pub message: String,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub scheduled_at: Option<chrono::DateTime<chrono::Utc>>,
    pub sent_at: Option<chrono::DateTime<chrono::Utc>>,
}

#[derive(Debug, Serialize)]
pub struct BulkNotificationResponse {
    pub batch_id: String,
    pub total_notifications: usize,
    pub successful: usize,
    pub failed: usize,
    pub notifications: Vec<NotificationResponse>,
}

/// Send a single notification
/// POST /notifications
pub async fn send_notification(
    State(state): State<AppState>,
    Json(request): Json<SendNotificationRequest>,
) -> Result<Json<NotificationResponse>> {
    // TODO: Extract user ID from JWT token
    let user_id = Uuid::new_v4(); // Placeholder
    
    let notification = CreateNotificationRequest {
        title: request.title,
        message: request.message,
        notification_type: request.notification_type,
        priority: request.priority,
        channels: request.channels,
        recipients: request.recipients,
        template_id: request.template_id,
        template_data: request.template_data,
        scheduled_at: request.scheduled_at,
        metadata: request.metadata.unwrap_or_default(),
    };

    let created_notification = state.notification_service
        .create_notification(notification, user_id)
        .await?;

    // Send notification if not scheduled
    if created_notification.scheduled_at.is_none() {
        let _ = state.notification_service
            .send_notification_by_id(created_notification.id)
            .await;
    }

    Ok(Json(NotificationResponse {
        id: created_notification.id,
        status: created_notification.status,
        message: "Notification created and queued for sending".to_string(),
        created_at: created_notification.created_at,
        scheduled_at: created_notification.scheduled_at,
        sent_at: created_notification.sent_at,
    }))
}

/// Send bulk notifications
/// POST /notifications/bulk
pub async fn send_bulk_notifications(
    State(state): State<AppState>,
    Json(request): Json<BulkNotificationRequest>,
) -> Result<Json<BulkNotificationResponse>> {
    let user_id = Uuid::new_v4(); // TODO: Extract from JWT
    let batch_id = request.batch_id.unwrap_or_else(|| Uuid::new_v4().to_string());
    
    let mut responses = Vec::new();
    let mut successful = 0;
    let mut failed = 0;

    for notification_request in request.notifications {
        let notification = CreateNotificationRequest {
            title: notification_request.title,
            message: notification_request.message,
            notification_type: notification_request.notification_type,
            priority: notification_request.priority,
            channels: notification_request.channels,
            recipients: notification_request.recipients,
            template_id: notification_request.template_id,
            template_data: notification_request.template_data,
            scheduled_at: notification_request.scheduled_at,
            metadata: notification_request.metadata.unwrap_or_default(),
        };

        match state.notification_service.create_notification(notification, user_id).await {
            Ok(created_notification) => {
                // Send notification if not scheduled
                if created_notification.scheduled_at.is_none() {
                    let _ = state.notification_service
                        .send_notification_by_id(created_notification.id)
                        .await;
                }

                responses.push(NotificationResponse {
                    id: created_notification.id,
                    status: created_notification.status,
                    message: "Notification created and queued".to_string(),
                    created_at: created_notification.created_at,
                    scheduled_at: created_notification.scheduled_at,
                    sent_at: created_notification.sent_at,
                });
                successful += 1;
            }
            Err(e) => {
                responses.push(NotificationResponse {
                    id: Uuid::new_v4(),
                    status: NotificationStatus::Failed,
                    message: format!("Failed to create notification: {}", e),
                    created_at: chrono::Utc::now(),
                    scheduled_at: None,
                    sent_at: None,
                });
                failed += 1;
            }
        }
    }

    Ok(Json(BulkNotificationResponse {
        batch_id,
        total_notifications: responses.len(),
        successful,
        failed,
        notifications: responses,
    }))
}

/// List notifications with filtering
/// GET /notifications
pub async fn list_notifications(
    State(state): State<AppState>,
    Query(query): Query<ListNotificationsQuery>,
) -> Result<Json<Vec<Notification>>> {
    let limit = query.limit.unwrap_or(50).min(1000);
    let offset = query.offset.unwrap_or(0);

    let notifications = state.notification_service
        .list_notifications(limit, offset, query.status, query.channel, query.priority)
        .await?;

    Ok(Json(notifications))
}

/// Get a specific notification
/// GET /notifications/{id}
pub async fn get_notification(
    State(state): State<AppState>,
    Path(notification_id): Path<Uuid>,
) -> Result<Json<Notification>> {
    let notification = state.notification_service
        .get_notification(notification_id)
        .await?;

    Ok(Json(notification))
}

/// Get notification status and delivery information
/// GET /notifications/{id}/status
pub async fn get_notification_status(
    State(state): State<AppState>,
    Path(notification_id): Path<Uuid>,
) -> Result<Json<NotificationStatusResponse>> {
    let notification = state.notification_service
        .get_notification(notification_id)
        .await?;

    let delivery_status = state.notification_service
        .get_delivery_status(notification_id)
        .await?;

    Ok(Json(NotificationStatusResponse {
        id: notification.id,
        status: notification.status,
        delivery_attempts: notification.delivery_attempts,
        sent_at: notification.sent_at,
        channels: notification.channels,
        delivery_status,
        error_message: None, // TODO: Add error tracking
    }))
}

/// Retry a failed notification
/// POST /notifications/{id}/retry
pub async fn retry_notification(
    State(state): State<AppState>,
    Path(notification_id): Path<Uuid>,
) -> Result<Json<NotificationResponse>> {
    let notification = state.notification_service
        .retry_notification(notification_id)
        .await?;

    Ok(Json(NotificationResponse {
        id: notification.id,
        status: notification.status,
        message: "Notification retry initiated".to_string(),
        created_at: notification.created_at,
        scheduled_at: notification.scheduled_at,
        sent_at: notification.sent_at,
    }))
}

#[derive(Debug, Serialize)]
pub struct NotificationStatusResponse {
    pub id: Uuid,
    pub status: NotificationStatus,
    pub delivery_attempts: i32,
    pub sent_at: Option<chrono::DateTime<chrono::Utc>>,
    pub channels: Vec<Channel>,
    pub delivery_status: Vec<ChannelDeliveryStatus>,
    pub error_message: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct ChannelDeliveryStatus {
    pub channel: Channel,
    pub status: DeliveryStatus,
    pub delivered_at: Option<chrono::DateTime<chrono::Utc>>,
    pub error_message: Option<String>,
}

#[derive(Debug, Serialize)]
pub enum DeliveryStatus {
    Pending,
    Sent,
    Delivered,
    Failed,
    Bounced,
    Opened,
    Clicked,
} 
