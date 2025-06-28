//! Webhook notification channel implementation

use async_trait::async_trait;
use crate::error::Result;
use super::{NotificationChannel, ChannelNotification, DeliveryResult, ChannelType};

/// Webhook channel for sending notifications to HTTP endpoints
pub struct WebhookChannel {
    http_client: reqwest::Client,
}

impl WebhookChannel {
    /// Create a new webhook channel
    pub fn new() -> Self {
        Self {
            http_client: reqwest::Client::new(),
        }
    }
}

#[async_trait]
impl NotificationChannel for WebhookChannel {
    async fn send(&self, notification: &ChannelNotification) -> Result<DeliveryResult> {
        // Extract webhook URL from recipient field
        let webhook_url = &notification.recipient;
        
        tracing::info!("Sending webhook to {}", webhook_url);
        
        // Stub implementation
        Ok(DeliveryResult {
            success: true,
            channel: ChannelType::Webhook,
            message_id: Some(uuid::Uuid::new_v4().to_string()),
            error: None,
            delivered_at: Some(chrono::Utc::now()),
        })
    }
    
    async fn is_available(&self) -> bool {
        true
    }
    
    fn channel_type(&self) -> ChannelType {
        ChannelType::Webhook
    }
}