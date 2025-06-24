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

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::Type, PartialEq, Eq, Hash)]
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

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::Type, PartialEq, Eq, Hash)]
#[sqlx(type_name = "priority", rename_all = "snake_case")]
pub enum Priority {
    Low,
    Medium,
    High,
    Critical,
    Urgent,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::Type, PartialEq, Eq, Hash)]
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

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::Type, PartialEq, Eq, Hash)]
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

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::Type, PartialEq, Eq)]
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
// Subscription Models
// ================================

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct NotificationSubscription {
    pub id: Uuid,
    pub user_id: Uuid,
    pub event_type: String,
    pub channels: Vec<Channel>,
    pub enabled: bool,
    pub filters: serde_json::Value,
    pub preferences: NotificationPreferences,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NotificationPreferences {
    pub quiet_hours: Option<QuietHours>,
    pub frequency_limit: Option<FrequencyLimit>,
    pub priority_threshold: Option<Priority>,
    pub group_similar: bool,
    pub digest_mode: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QuietHours {
    pub start_time: String, // HH:MM format
    pub end_time: String,   // HH:MM format
    pub timezone: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FrequencyLimit {
    pub max_per_hour: Option<u32>,
    pub max_per_day: Option<u32>,
}

// ================================
// Metrics and Statistics Models
// ================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NotificationMetrics {
    pub notifications_sent: u64,
    pub notifications_failed: u64,
    pub delivery_rate: f64,
    pub avg_delivery_time_ms: u64,
    pub email_sent: u64,
    pub email_failed: u64,
    pub email_avg_response_time_ms: u64,
    pub sms_sent: u64,
    pub sms_failed: u64,
    pub sms_avg_response_time_ms: u64,
    pub slack_sent: u64,
    pub slack_failed: u64,
    pub slack_avg_response_time_ms: u64,
    pub teams_sent: u64,
    pub teams_failed: u64,
    pub teams_avg_response_time_ms: u64,
}

// ================================
// Template Extensions
// ================================

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Template {
    pub id: Uuid,
    pub name: String,
    pub description: Option<String>,
    pub template_type: TemplateType,
    pub subject: Option<String>,
    pub body_html: Option<String>,
    pub body_text: String,
    pub variables: Vec<String>,
    pub metadata: serde_json::Value,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Serialize)]
pub struct TemplatePreviewResponse {
    pub subject: Option<String>,
    pub body_html: Option<String>,
    pub body_text: String,
    pub missing_variables: Vec<String>,
}

#[derive(Debug, Serialize)]
pub struct TemplateValidationResult {
    pub is_valid: bool,
    pub errors: Vec<String>,
    pub warnings: Vec<String>,
}

// ================================
// Channel Configuration Models
// ================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChannelConfigResponse {
    pub channel: Channel,
    pub enabled: bool,
    pub config: serde_json::Value,
    pub rate_limit: Option<RateLimit>,
    pub last_test: Option<DateTime<Utc>>,
    pub last_test_result: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RateLimit {
    pub requests_per_minute: u32,
    pub requests_per_hour: u32,
    pub requests_per_day: u32,
}

// ================================
// Delivery Status Models
// ================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChannelDeliveryStatus {
    pub channel: Channel,
    pub status: DeliveryStatus,
    pub delivered_at: Option<DateTime<Utc>>,
    pub error_message: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum DeliveryStatus {
    Pending,
    Sent,
    Delivered,
    Failed,
    Bounced,
    Opened,
    Clicked,
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

impl Default for NotificationPreferences {
    fn default() -> Self {
        Self {
            quiet_hours: None,
            frequency_limit: None,
            priority_threshold: Some(Priority::Low),
            group_similar: false,
            digest_mode: false,
        }
    }
}

// ================================
// Trait Implementations
// ================================

// PostgreSQL array support for Channel enum
impl sqlx::postgres::PgHasArrayType for Channel {
    fn array_type_info() -> sqlx::postgres::PgTypeInfo {
        sqlx::postgres::PgTypeInfo::with_name("_channel")
    }
}
