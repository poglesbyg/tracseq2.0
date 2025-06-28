//! Email notification channel implementation

use async_trait::async_trait;
use crate::error::Result;
use super::{NotificationChannel, ChannelNotification, DeliveryResult, ChannelType};

/// Email channel for sending notifications via SMTP
pub struct EmailChannel {
    smtp_host: String,
    smtp_port: u16,
    from_address: String,
}

impl EmailChannel {
    /// Create a new email channel
    pub fn new(smtp_host: String, smtp_port: u16, from_address: String) -> Self {
        Self {
            smtp_host,
            smtp_port,
            from_address,
        }
    }
}

#[async_trait]
impl NotificationChannel for EmailChannel {
    async fn send(&self, notification: &ChannelNotification) -> Result<DeliveryResult> {
        // Stub implementation - in production this would use an SMTP library
        tracing::info!(
            "Sending email to {} with subject: {}",
            notification.recipient,
            notification.subject
        );
        
        Ok(DeliveryResult {
            success: true,
            channel: ChannelType::Email,
            message_id: Some(uuid::Uuid::new_v4().to_string()),
            error: None,
            delivered_at: Some(chrono::Utc::now()),
        })
    }
    
    async fn is_available(&self) -> bool {
        // Stub - would check SMTP connection
        true
    }
    
    fn channel_type(&self) -> ChannelType {
        ChannelType::Email
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    async fn test_email_channel_send() {
        let channel = EmailChannel::new(
            "smtp.example.com".to_string(),
            587,
            "noreply@example.com".to_string()
        );
        
        let notification = ChannelNotification {
            recipient: "user@example.com".to_string(),
            subject: "Test Subject".to_string(),
            body: "Test Body".to_string(),
            metadata: serde_json::json!({}),
            priority: super::NotificationPriority::Normal,
        };
        
        let result = channel.send(&notification).await.unwrap();
        assert!(result.success);
        assert_eq!(result.channel, ChannelType::Email);
    }
}