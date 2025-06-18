use crate::config::Config;
use crate::database::DatabasePool;
use crate::error::{NotificationError, Result};
use crate::models::*;
use chrono::{DateTime, Utc};
use reqwest::Client;
use serde_json::json;
use sqlx::query_as;
use std::collections::HashMap;
use tokio::time::{Duration};
use tracing::{error, info, warn};
use uuid::Uuid;

#[derive(Clone)]
pub struct NotificationService {
    pub config: Config,
    pub database: DatabasePool,
    pub http_client: Client,
}

impl NotificationService {
    pub async fn new(config: Config, database: DatabasePool) -> Result<Self> {
        let http_client = Client::new();

        Ok(Self {
            config,
            database,
            http_client,
        })
    }

    pub async fn send_notification(&self, request: CreateNotificationRequest, created_by: Uuid) -> Result<NotificationResponse> {
        // Validate the request
        request.validate().map_err(NotificationError::validation)?;

        // Create notification record
        let notification_id = Uuid::new_v4();
        let now = Utc::now();

        let notification = Notification {
            id: notification_id,
            title: request.title.clone(),
            message: request.message.clone(),
            notification_type: request.notification_type.clone(),
            priority: request.priority.clone(),
            status: NotificationStatus::Pending,
            channels: request.channels.clone(),
            recipients: request.recipients.clone(),
            template_id: request.template_id,
            template_data: request.template_data.clone(),
            scheduled_at: request.scheduled_at,
            sent_at: None,
            delivery_attempts: 0,
            metadata: request.metadata.unwrap_or_else(|| json!({})),
            created_by,
            created_at: now,
            updated_at: now,
        };

        // Save to database
        self.save_notification(&notification).await?;

        // If scheduled, don't send immediately
        if request.scheduled_at.is_some() && request.scheduled_at.unwrap() > now {
            return Ok(NotificationResponse {
                notification_id,
                status: NotificationStatus::Scheduled,
                message: "Notification scheduled successfully".to_string(),
                scheduled_at: request.scheduled_at,
            });
        }

        // Send immediately
        let delivery_results = self.deliver_notification(&notification).await;
        
        // Update notification status based on results
        let overall_success = delivery_results.iter().any(|r| r.is_ok());
        let status = if overall_success {
            NotificationStatus::Sent
        } else {
            NotificationStatus::Failed
        };

        self.update_notification_status(notification_id, status).await?;

        Ok(NotificationResponse {
            notification_id,
            status,
            message: if overall_success {
                "Notification sent successfully".to_string()
            } else {
                "Notification failed to send".to_string()
            },
            scheduled_at: None,
        })
    }

    async fn deliver_notification(&self, notification: &Notification) -> Vec<Result<()>> {
        let mut results = Vec::new();

        for channel in &notification.channels {
            for recipient in &notification.recipients {
                let result = match channel {
                    Channel::Email => self.send_email(notification, recipient).await,
                    Channel::Sms => self.send_sms(notification, recipient).await,
                    Channel::Slack => self.send_slack(notification, recipient).await,
                    Channel::Teams => self.send_teams(notification, recipient).await,
                    Channel::Webhook => self.send_webhook(notification, recipient).await,
                    _ => Err(NotificationError::channel(
                        format!("{:?}", channel),
                        "Channel not implemented yet".to_string(),
                    )),
                };

                // Log delivery attempt
                self.log_delivery_attempt(
                    notification.id,
                    channel.clone(),
                    recipient.clone(),
                    &result,
                ).await;

                results.push(result);
            }
        }

        results
    }

    async fn send_email(&self, notification: &Notification, recipient: &str) -> Result<()> {
        if !self.config.email.enabled {
            return Err(NotificationError::channel("email".to_string(), "Email not enabled".to_string()));
        }

        // For now, just log the email (would implement SMTP in production)
        info!("Would send email to {} with subject: {}", recipient, notification.title);
        Ok(())
    }

    async fn send_sms(&self, notification: &Notification, recipient: &str) -> Result<()> {
        if !self.config.sms.enabled {
            return Err(NotificationError::channel("sms".to_string(), "SMS not enabled".to_string()));
        }

        // Twilio SMS implementation stub
        info!("Would send SMS to {}: {}", recipient, notification.message);
        Ok(())
    }

    async fn send_slack(&self, notification: &Notification, channel: &str) -> Result<()> {
        if !self.config.slack.enabled {
            return Err(NotificationError::channel("slack".to_string(), "Slack not enabled".to_string()));
        }

        let payload = json!({
            "channel": channel,
            "text": notification.message,
            "attachments": [{
                "color": self.get_color_for_type(&notification.notification_type),
                "title": notification.title,
                "text": notification.message,
                "ts": notification.created_at.timestamp()
            }]
        });

        // Try webhook first
        if !self.config.slack.webhook_urls.is_empty() {
            for webhook_url in &self.config.slack.webhook_urls {
                let response = self.http_client
                    .post(webhook_url)
                    .json(&payload)
                    .send()
                    .await?;

                if response.status().is_success() {
                    info!("Slack message sent successfully via webhook");
                    return Ok(());
                }
            }
        }

        Err(NotificationError::channel("slack".to_string(), "No valid Slack webhook configured".to_string()))
    }

    async fn send_teams(&self, notification: &Notification, webhook_url: &str) -> Result<()> {
        if !self.config.teams.enabled {
            return Err(NotificationError::channel("teams".to_string(), "Teams not enabled".to_string()));
        }

        let payload = json!({
            "@type": "MessageCard",
            "@context": "http://schema.org/extensions",
            "themeColor": self.get_color_for_type(&notification.notification_type),
            "summary": notification.title,
            "sections": [{
                "activityTitle": notification.title,
                "activitySubtitle": format!("Priority: {:?}", notification.priority),
                "text": notification.message
            }]
        });

        let response = self.http_client
            .post(webhook_url)
            .json(&payload)
            .send()
            .await?;

        if response.status().is_success() {
            info!("Teams message sent successfully");
            Ok(())
        } else {
            let error_text = response.text().await.unwrap_or_default();
            Err(NotificationError::TeamsDelivery(format!("Teams failed: {}", error_text)))
        }
    }

    async fn send_webhook(&self, notification: &Notification, url: &str) -> Result<()> {
        if !self.config.webhook.enabled {
            return Err(NotificationError::channel("webhook".to_string(), "Webhook not enabled".to_string()));
        }

        let payload = json!({
            "id": notification.id,
            "title": notification.title,
            "message": notification.message,
            "type": notification.notification_type,
            "priority": notification.priority,
            "timestamp": notification.created_at,
            "metadata": notification.metadata
        });

        let response = self.http_client
            .post(url)
            .json(&payload)
            .timeout(Duration::from_secs(self.config.webhook.timeout_seconds))
            .send()
            .await?;

        if response.status().is_success() {
            info!("Webhook sent successfully to {}", url);
            Ok(())
        } else {
            let error_text = response.text().await.unwrap_or_default();
            Err(NotificationError::WebhookDelivery(format!("Webhook failed: {}", error_text)))
        }
    }

    async fn save_notification(&self, notification: &Notification) -> Result<()> {
        sqlx::query!(
            r#"
            INSERT INTO notifications (
                id, title, message, notification_type, priority, status, 
                channels, recipients, template_id, template_data, 
                scheduled_at, delivery_attempts, metadata, created_by
            ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14)
            "#,
            notification.id,
            notification.title,
            notification.message,
            notification.notification_type as NotificationType,
            notification.priority as Priority,
            notification.status as NotificationStatus,
            &notification.channels as &Vec<Channel>,
            &notification.recipients,
            notification.template_id,
            notification.template_data,
            notification.scheduled_at,
            notification.delivery_attempts,
            notification.metadata,
            notification.created_by
        )
        .execute(&self.database.pool)
        .await?;

        Ok(())
    }

    async fn update_notification_status(&self, notification_id: Uuid, status: NotificationStatus) -> Result<()> {
        sqlx::query!(
            "UPDATE notifications SET status = $1, updated_at = NOW() WHERE id = $2",
            status as NotificationStatus,
            notification_id
        )
        .execute(&self.database.pool)
        .await?;

        Ok(())
    }

    async fn log_delivery_attempt(
        &self,
        notification_id: Uuid,
        channel: Channel,
        recipient: String,
        result: &Result<()>,
    ) {
        let (status, error_message) = match result {
            Ok(_) => ("sent".to_string(), None),
            Err(e) => ("failed".to_string(), Some(e.to_string())),
        };

        if let Err(e) = sqlx::query!(
            r#"
            INSERT INTO delivery_attempts (
                notification_id, channel, recipient, status, 
                attempt_number, error_message
            ) VALUES ($1, $2, $3, $4::delivery_status, 1, $5)
            "#,
            notification_id,
            channel as Channel,
            recipient,
            status,
            error_message
        )
        .execute(&self.database.pool)
        .await
        {
            error!("Failed to log delivery attempt: {}", e);
        }
    }

    fn get_color_for_type(&self, notification_type: &NotificationType) -> &'static str {
        match notification_type {
            NotificationType::Error => "danger",
            NotificationType::Warning => "warning",
            NotificationType::Success => "good",
            NotificationType::Info => "#36a64f",
            NotificationType::Alert => "#ff0000",
            _ => "#36a64f",
        }
    }

    pub async fn get_notification(&self, notification_id: Uuid) -> Result<Notification> {
        let notification = query_as!(
            Notification,
            "SELECT * FROM notifications WHERE id = $1",
            notification_id
        )
        .fetch_optional(&self.database.pool)
        .await?
        .ok_or_else(|| NotificationError::NotificationNotFound(notification_id.to_string()))?;

        Ok(notification)
    }

    pub async fn list_notifications(&self, limit: Option<i64>, offset: Option<i64>) -> Result<Vec<Notification>> {
        let limit = limit.unwrap_or(50);
        let offset = offset.unwrap_or(0);

        let notifications = query_as!(
            Notification,
            "SELECT * FROM notifications ORDER BY created_at DESC LIMIT $1 OFFSET $2",
            limit,
            offset
        )
        .fetch_all(&self.database.pool)
        .await?;

        Ok(notifications)
    }
}
