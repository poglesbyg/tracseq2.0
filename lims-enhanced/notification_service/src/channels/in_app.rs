//! In-app notification channel implementation

use async_trait::async_trait;
use crate::error::Result;
use super::{NotificationChannel, ChannelNotification, DeliveryResult, ChannelType};

/// In-app notification channel
pub struct InAppChannel {
    // In production, this would store notifications in a database
}

impl InAppChannel {
    /// Create a new in-app notification channel
    pub fn new() -> Self {
        Self {}
    }
}

#[async_trait]
impl NotificationChannel for InAppChannel {
    async fn send(&self, notification: &ChannelNotification) -> Result<DeliveryResult> {
        tracing::info!(
            "Storing in-app notification for user: {}",
            notification.recipient
        );
        
        // Stub implementation - would store in database
        Ok(DeliveryResult {
            success: true,
            channel: ChannelType::InApp,
            message_id: Some(uuid::Uuid::new_v4().to_string()),
            error: None,
            delivered_at: Some(chrono::Utc::now()),
        })
    }
    
    async fn is_available(&self) -> bool {
        true
    }
    
    fn channel_type(&self) -> ChannelType {
        ChannelType::InApp
    }
}