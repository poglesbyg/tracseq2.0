use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use std::collections::HashMap;
use uuid::Uuid;

// ================================
// Core Notification Models
// ================================

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Notification {
    pub id: Uuid,
    pub title: String,
    pub message: String,
    pub notification_type: NotificationType,
    pub priority: Priority,
    pub status: NotificationStatus,
    pub channels: Vec<Channel>,
    pub recipients: Vec<String>,
    pub template_id: Option<Uuid>,
    pub template_data: Option<serde_json::Value>,
    pub scheduled_at: Option<DateTime<Utc>>,
    pub sent_at: Option<DateTime<Utc>>,
    pub delivery_attempts: i32,
    pub metadata: serde_json::Value,
    pub created_by: Uuid,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::Type)]
#[sqlx(type_name = "notification_type", rename_all = "snake_case")]
pub enum NotificationType {
    Alert,
    Info,
    Warning,
    Error,
    Success,
    Reminder,
    Update,
    System,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::Type)]
#[sqlx(type_name = "priority", rename_all = "snake_case")]
pub enum Priority {
    Low,
    Medium,
    High,
    Critical,
    Urgent,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::Type)]
#[sqlx(type_name = "notification_status", rename_all = "snake_case")]
pub enum NotificationStatus {
    Pending,
    Scheduled,
    Sending,
    Sent,
    Delivered,
    Failed,
    Cancelled,
    Retrying,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::Type)]
#[sqlx(type_name = "channel", rename_all = "snake_case")]
pub enum Channel {
    Email,
    Sms,
    Slack,
    Teams,
    Discord,
    Webhook,
    Push,
    InApp,
}

// ================================
// Template Models
// ================================

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct NotificationTemplate {
    pub id: Uuid,
    pub name: String,
    pub description: Option<String>,
    pub template_type: TemplateType,
    pub channels: Vec<Channel>,
    pub subject_template: String,
    pub body_template: String,
    pub variables: Vec<String>,
    pub default_data: serde_json::Value,
    pub is_active: bool,
    pub version: i32,
    pub created_by: Uuid,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::Type)]
#[sqlx(type_name = "template_type", rename_all = "snake_case")]
pub enum TemplateType {
    Email,
    Sms,
    Slack,
    Teams,
    Discord,
    Webhook,
    Universal,
}

// ================================
// Request/Response Models
// ================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateNotificationRequest {
    pub title: String,
    pub message: String,
    pub notification_type: NotificationType,
    pub priority: Priority,
    pub channels: Vec<Channel>,
    pub recipients: Vec<String>,
    pub template_id: Option<Uuid>,
    pub template_data: Option<serde_json::Value>,
    pub scheduled_at: Option<DateTime<Utc>>,
    pub metadata: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NotificationResponse {
    pub notification_id: Uuid,
    pub status: NotificationStatus,
    pub message: String,
    pub scheduled_at: Option<DateTime<Utc>>,
}

// ================================
// Channel-Specific Models
// ================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmailMessage {
    pub to: Vec<String>,
    pub subject: String,
    pub body_text: Option<String>,
    pub body_html: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SlackMessage {
    pub channel: String,
    pub text: String,
    pub blocks: Option<serde_json::Value>,
}

// ================================
// Validation Helpers
// ================================

impl CreateNotificationRequest {
    pub fn validate(&self) -> Result<(), String> {
        if self.title.trim().is_empty() {
            return Err("Title cannot be empty".to_string());
        }

        if self.message.trim().is_empty() {
            return Err("Message cannot be empty".to_string());
        }

        if self.recipients.is_empty() {
            return Err("At least one recipient is required".to_string());
        }

        if self.channels.is_empty() {
            return Err("At least one channel is required".to_string());
        }

        Ok(())
    }
}
