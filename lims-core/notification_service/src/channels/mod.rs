//! Notification channels for different delivery methods

pub mod email;
pub mod webhook;
pub mod in_app;

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use crate::error::Result;

/// Trait for notification channels
#[async_trait]
pub trait NotificationChannel: Send + Sync {
    /// Send a notification through this channel
    async fn send(&self, notification: &ChannelNotification) -> Result<DeliveryResult>;
    
    /// Check if the channel is available
    async fn is_available(&self) -> bool;
    
    /// Get the channel type
    fn channel_type(&self) -> ChannelType;
}

/// Notification to be sent through a channel
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChannelNotification {
    pub recipient: String,
    pub subject: String,
    pub body: String,
    pub metadata: serde_json::Value,
    pub priority: NotificationPriority,
}

/// Result of attempting to deliver a notification
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeliveryResult {
    pub success: bool,
    pub channel: ChannelType,
    pub message_id: Option<String>,
    pub error: Option<String>,
    pub delivered_at: Option<chrono::DateTime<chrono::Utc>>,
}

/// Types of notification channels
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ChannelType {
    Email,
    Webhook,
    InApp,
    Sms,
    Slack,
}

/// Notification priority levels
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum NotificationPriority {
    Low,
    Normal,
    High,
    Urgent,
}

impl Default for NotificationPriority {
    fn default() -> Self {
        Self::Normal
    }
}