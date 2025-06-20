use axum::{
    extract::State,
    Json,
};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::{
    error::Result,
    models::*,
    AppState,
};

#[derive(Debug, Deserialize)]
pub struct LabEventRequest {
    pub event_type: String,
    pub event_id: Uuid,
    pub lab_id: Uuid,
    pub user_id: Option<Uuid>,
    pub title: String,
    pub description: String,
    pub priority: Priority,
    pub metadata: serde_json::Value,
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Deserialize)]
pub struct SampleEventRequest {
    pub event_type: String,
    pub event_id: Uuid,
    pub sample_id: Uuid,
    pub user_id: Option<Uuid>,
    pub title: String,
    pub description: String,
    pub priority: Priority,
    pub metadata: serde_json::Value,
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Deserialize)]
pub struct SequencingEventRequest {
    pub event_type: String,
    pub event_id: Uuid,
    pub sequencing_job_id: Uuid,
    pub user_id: Option<Uuid>,
    pub title: String,
    pub description: String,
    pub priority: Priority,
    pub metadata: serde_json::Value,
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Deserialize)]
pub struct TemplateEventRequest {
    pub event_type: String,
    pub event_id: Uuid,
    pub template_id: Uuid,
    pub user_id: Option<Uuid>,
    pub title: String,
    pub description: String,
    pub priority: Priority,
    pub metadata: serde_json::Value,
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Deserialize)]
pub struct SystemAlertRequest {
    pub alert_type: String,
    pub alert_id: Uuid,
    pub service_name: String,
    pub severity: AlertSeverity,
    pub title: String,
    pub description: String,
    pub metadata: serde_json::Value,
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Serialize)]
pub struct EventResponse {
    pub event_id: Uuid,
    pub notifications_sent: usize,
    pub channels_used: Vec<Channel>,
    pub message: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub enum AlertSeverity {
    Low,
    Medium,
    High,
    Critical,
}

/// Handle lab events
/// POST /integration/lab-events
pub async fn handle_lab_event(
    State(state): State<AppState>,
    Json(request): Json<LabEventRequest>,
) -> Result<Json<EventResponse>> {
    let notifications_sent = state.notification_service.handle_lab_event(
        request.event_type,
        request.event_id,
        request.lab_id,
        request.user_id,
        request.title,
        request.description,
        request.priority,
        request.metadata,
        request.timestamp,
    ).await?;

    Ok(Json(EventResponse {
        event_id: request.event_id,
        notifications_sent: notifications_sent.len(),
        channels_used: extract_channels_from_notifications(&notifications_sent),
        message: "Lab event processed and notifications sent".to_string(),
    }))
}

/// Handle sample events
/// POST /integration/sample-events
pub async fn handle_sample_event(
    State(state): State<AppState>,
    Json(request): Json<SampleEventRequest>,
) -> Result<Json<EventResponse>> {
    let notifications_sent = state.notification_service.handle_sample_event(
        request.event_type,
        request.event_id,
        request.sample_id,
        request.user_id,
        request.title,
        request.description,
        request.priority,
        request.metadata,
        request.timestamp,
    ).await?;

    Ok(Json(EventResponse {
        event_id: request.event_id,
        notifications_sent: notifications_sent.len(),
        channels_used: extract_channels_from_notifications(&notifications_sent),
        message: "Sample event processed and notifications sent".to_string(),
    }))
}

/// Handle sequencing events
/// POST /integration/sequencing-events
pub async fn handle_sequencing_event(
    State(state): State<AppState>,
    Json(request): Json<SequencingEventRequest>,
) -> Result<Json<EventResponse>> {
    let notifications_sent = state.notification_service.handle_sequencing_event(
        request.event_type,
        request.event_id,
        request.sequencing_job_id,
        request.user_id,
        request.title,
        request.description,
        request.priority,
        request.metadata,
        request.timestamp,
    ).await?;

    Ok(Json(EventResponse {
        event_id: request.event_id,
        notifications_sent: notifications_sent.len(),
        channels_used: extract_channels_from_notifications(&notifications_sent),
        message: "Sequencing event processed and notifications sent".to_string(),
    }))
}

/// Handle template events
/// POST /integration/template-events
pub async fn handle_template_event(
    State(state): State<AppState>,
    Json(request): Json<TemplateEventRequest>,
) -> Result<Json<EventResponse>> {
    let notifications_sent = state.notification_service.handle_template_event(
        request.event_type,
        request.event_id,
        request.template_id,
        request.user_id,
        request.title,
        request.description,
        request.priority,
        request.metadata,
        request.timestamp,
    ).await?;

    Ok(Json(EventResponse {
        event_id: request.event_id,
        notifications_sent: notifications_sent.len(),
        channels_used: extract_channels_from_notifications(&notifications_sent),
        message: "Template event processed and notifications sent".to_string(),
    }))
}

/// Handle system alerts
/// POST /integration/system-alerts
pub async fn handle_system_alert(
    State(state): State<AppState>,
    Json(request): Json<SystemAlertRequest>,
) -> Result<Json<EventResponse>> {
    let priority = match request.severity {
        AlertSeverity::Low => Priority::Low,
        AlertSeverity::Medium => Priority::Medium,
        AlertSeverity::High => Priority::High,
        AlertSeverity::Critical => Priority::Critical,
    };

    let notifications_sent = state.notification_service.handle_system_alert(
        request.alert_type,
        request.alert_id,
        request.service_name,
        request.severity,
        request.title,
        request.description,
        request.metadata,
        request.timestamp,
    ).await?;

    Ok(Json(EventResponse {
        event_id: request.alert_id,
        notifications_sent: notifications_sent.len(),
        channels_used: extract_channels_from_notifications(&notifications_sent),
        message: "System alert processed and notifications sent".to_string(),
    }))
}

/// Helper function to extract channels from notifications
fn extract_channels_from_notifications(notifications: &[Notification]) -> Vec<Channel> {
    let mut channels = Vec::new();
    for notification in notifications {
        for channel in &notification.channels {
            if !channels.contains(channel) {
                channels.push(channel.clone());
            }
        }
    }
    channels
} 
