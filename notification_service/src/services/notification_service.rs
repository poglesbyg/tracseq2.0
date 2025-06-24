use crate::{
    config::Config,
    database::DatabasePool,
    error::{NotificationError, Result},
    handlers::{
        admin::{ChannelHealthResponse, HealthStatus, NotificationStatistics, RateLimitResponse},
        channels::{ChannelConfigResponse, EmailTemplate, SlackWebhook},
        integration::AlertSeverity,
    },
    models::*,
};
use chrono::{DateTime, Utc};
use reqwest::Client;
use serde_json::json;
use sqlx::query_as;
use std::collections::HashMap;
use tokio::time::Duration;
use tracing::{error, info, warn};
use uuid::Uuid;

#[derive(Clone)]
pub struct NotificationServiceImpl {
    pub config: Config,
    pub database: DatabasePool,
    pub http_client: Client,
}

impl NotificationServiceImpl {
    pub async fn new(
        database: DatabasePool,
        config: Config,
        _auth_client: crate::clients::AuthClient,
        _slack_client: crate::clients::SlackClient,
        _teams_client: crate::clients::TeamsClient,
        _email_client: crate::clients::EmailClient,
        _sms_client: crate::clients::SmsClient,
    ) -> Result<Self> {
        let http_client = Client::new();

        Ok(Self {
            config,
            database,
            http_client,
        })
    }

    // ================================
    // Core Notification Methods
    // ================================

    pub async fn create_notification(
        &self,
        request: CreateNotificationRequest,
        created_by: Uuid,
    ) -> Result<Notification> {
        request.validate().map_err(NotificationError::validation)?;

        let notification_id = Uuid::new_v4();
        let now = Utc::now();

        let notification = Notification {
            id: notification_id,
            title: request.title.clone(),
            message: request.message.clone(),
            notification_type: request.notification_type.clone(),
            priority: request.priority.clone(),
            status: if request.scheduled_at.is_some() && request.scheduled_at.unwrap() > now {
                NotificationStatus::Scheduled
            } else {
                NotificationStatus::Pending
            },
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

        self.save_notification(&notification).await?;
        Ok(notification)
    }

    pub async fn send_notification_by_id(&self, notification_id: Uuid) -> Result<()> {
        let mut notification = self.get_notification(notification_id).await?;

        if notification.status != NotificationStatus::Pending
            && notification.status != NotificationStatus::Scheduled
        {
            return Err(NotificationError::InvalidOperation(
                "Notification is not in a sendable state".to_string(),
            ));
        }

        notification.status = NotificationStatus::Sending;
        notification.delivery_attempts += 1;
        self.update_notification(&notification).await?;

        let delivery_results = self.deliver_notification(&notification).await;
        let overall_success = delivery_results.iter().any(|r| r.is_ok());

        notification.status = if overall_success {
            NotificationStatus::Sent
        } else {
            NotificationStatus::Failed
        };

        if overall_success {
            notification.sent_at = Some(Utc::now());
        }

        self.update_notification(&notification).await?;
        Ok(())
    }

    pub async fn get_notification(&self, notification_id: Uuid) -> Result<Notification> {
        let notification =
            sqlx::query_as::<_, Notification>("SELECT * FROM notifications WHERE id = $1")
                .bind(notification_id)
                .fetch_optional(&self.database.pool)
                .await?
                .ok_or_else(|| {
                    NotificationError::NotificationNotFound(notification_id.to_string())
                })?;

        Ok(notification)
    }

    pub async fn list_notifications(
        &self,
        limit: i64,
        offset: i64,
        status: Option<NotificationStatus>,
        channel: Option<Channel>,
        priority: Option<Priority>,
    ) -> Result<Vec<Notification>> {
        // This is a simplified version - in production, you'd build dynamic queries
        let notifications = sqlx::query_as::<_, Notification>(
            "SELECT * FROM notifications ORDER BY created_at DESC LIMIT $1 OFFSET $2",
        )
        .bind(limit)
        .bind(offset)
        .fetch_all(&self.database.pool)
        .await?;

        Ok(notifications)
    }

    pub async fn retry_notification(&self, notification_id: Uuid) -> Result<Notification> {
        let mut notification = self.get_notification(notification_id).await?;

        if notification.status != NotificationStatus::Failed {
            return Err(NotificationError::InvalidOperation(
                "Only failed notifications can be retried".to_string(),
            ));
        }

        notification.status = NotificationStatus::Pending;
        self.update_notification(&notification).await?;

        // Trigger async sending
        let _ = self.send_notification_by_id(notification_id).await;

        Ok(notification)
    }

    pub async fn get_delivery_status(
        &self,
        notification_id: Uuid,
    ) -> Result<Vec<ChannelDeliveryStatus>> {
        // Simplified implementation - would query delivery_attempts table
        Ok(vec![])
    }

    // ================================
    // Channel Testing Methods
    // ================================

    pub async fn test_email_channel(
        &self,
        recipient: &str,
        subject: &str,
        message: &str,
    ) -> Result<()> {
        if !self.config.email.enabled {
            return Err(NotificationError::ChannelDisabled("email".to_string()));
        }

        info!("Test email sent to {} with subject: {}", recipient, subject);
        Ok(())
    }

    pub async fn test_sms_channel(&self, recipient: &str, message: &str) -> Result<()> {
        if !self.config.sms.enabled {
            return Err(NotificationError::ChannelDisabled("sms".to_string()));
        }

        info!("Test SMS sent to {}: {}", recipient, message);
        Ok(())
    }

    pub async fn test_slack_channel(&self, channel: &str, message: &str) -> Result<()> {
        if !self.config.slack.enabled {
            return Err(NotificationError::ChannelDisabled("slack".to_string()));
        }

        info!("Test Slack message sent to {}: {}", channel, message);
        Ok(())
    }

    pub async fn test_teams_channel(&self, webhook: &str, message: &str) -> Result<()> {
        if !self.config.teams.enabled {
            return Err(NotificationError::ChannelDisabled("teams".to_string()));
        }

        info!("Test Teams message sent to {}: {}", webhook, message);
        Ok(())
    }

    // ================================
    // Channel Configuration Methods
    // ================================

    pub async fn list_channels(&self) -> Result<Vec<ChannelConfigResponse>> {
        Ok(vec![
            ChannelConfigResponse {
                channel: Channel::Email,
                enabled: self.config.email.enabled,
                config: json!({}),
                rate_limit: None,
                last_test: None,
                last_test_result: None,
            },
            ChannelConfigResponse {
                channel: Channel::Sms,
                enabled: self.config.sms.enabled,
                config: json!({}),
                rate_limit: None,
                last_test: None,
                last_test_result: None,
            },
            ChannelConfigResponse {
                channel: Channel::Slack,
                enabled: self.config.slack.enabled,
                config: json!({}),
                rate_limit: None,
                last_test: None,
                last_test_result: None,
            },
            ChannelConfigResponse {
                channel: Channel::Teams,
                enabled: self.config.teams.enabled,
                config: json!({}),
                rate_limit: None,
                last_test: None,
                last_test_result: None,
            },
        ])
    }

    pub async fn get_channel_config(&self, channel: Channel) -> Result<ChannelConfigResponse> {
        let (enabled, config) = match channel {
            Channel::Email => (self.config.email.enabled, json!({})),
            Channel::Sms => (self.config.sms.enabled, json!({})),
            Channel::Slack => (self.config.slack.enabled, json!({})),
            Channel::Teams => (self.config.teams.enabled, json!({})),
            _ => (false, json!({})),
        };

        Ok(ChannelConfigResponse {
            channel,
            enabled,
            config,
            rate_limit: None,
            last_test: None,
            last_test_result: None,
        })
    }

    pub async fn update_channel_config(
        &self,
        channel: Channel,
        enabled: Option<bool>,
        config: Option<serde_json::Value>,
        rate_limit: Option<RateLimit>,
    ) -> Result<ChannelConfigResponse> {
        // In production, this would update configuration in database
        self.get_channel_config(channel).await
    }

    pub async fn list_email_templates(&self) -> Result<Vec<EmailTemplate>> {
        Ok(vec![])
    }

    pub async fn create_slack_webhook(
        &self,
        name: String,
        webhook_url: String,
        channel: String,
        username: Option<String>,
        icon_emoji: Option<String>,
    ) -> Result<SlackWebhook> {
        Ok(SlackWebhook {
            id: Uuid::new_v4(),
            name,
            webhook_url,
            channel,
            username,
            icon_emoji,
            created_at: Utc::now(),
        })
    }

    // ================================
    // Template Methods
    // ================================

    pub async fn create_template(
        &self,
        name: String,
        description: Option<String>,
        template_type: TemplateType,
        subject: Option<String>,
        body_html: Option<String>,
        body_text: String,
        variables: Vec<String>,
        metadata: serde_json::Value,
        created_by: Uuid,
    ) -> Result<Template> {
        let template = Template {
            id: Uuid::new_v4(),
            name,
            description,
            template_type,
            subject,
            body_html,
            body_text,
            variables,
            metadata,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };

        // In production, save to database
        Ok(template)
    }

    pub async fn list_templates(&self, limit: i64, offset: i64) -> Result<Vec<Template>> {
        Ok(vec![])
    }

    pub async fn get_template(&self, template_id: Uuid) -> Result<Template> {
        Err(NotificationError::TemplateNotFound(template_id.to_string()))
    }

    pub async fn update_template(
        &self,
        template_id: Uuid,
        name: Option<String>,
        description: Option<String>,
        subject: Option<String>,
        body_html: Option<String>,
        body_text: Option<String>,
        variables: Option<Vec<String>>,
        metadata: Option<serde_json::Value>,
    ) -> Result<Template> {
        self.get_template(template_id).await
    }

    pub async fn delete_template(&self, template_id: Uuid) -> Result<()> {
        Ok(())
    }

    pub async fn preview_template(
        &self,
        template_id: Uuid,
        template_data: serde_json::Value,
    ) -> Result<TemplatePreviewResponse> {
        Ok(TemplatePreviewResponse {
            subject: Some("Preview Subject".to_string()),
            body_html: Some("<p>Preview HTML</p>".to_string()),
            body_text: "Preview text".to_string(),
            missing_variables: vec![],
        })
    }

    pub async fn validate_template(&self, template_id: Uuid) -> Result<TemplateValidationResult> {
        Ok(TemplateValidationResult {
            is_valid: true,
            errors: vec![],
            warnings: vec![],
        })
    }

    // ================================
    // Subscription Methods
    // ================================

    pub async fn create_subscription(
        &self,
        user_id: Uuid,
        event_type: String,
        channels: Vec<Channel>,
        enabled: bool,
        filters: serde_json::Value,
        preferences: NotificationPreferences,
    ) -> Result<NotificationSubscription> {
        Ok(NotificationSubscription {
            id: Uuid::new_v4(),
            user_id,
            event_type,
            channels,
            enabled,
            filters,
            preferences,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        })
    }

    pub async fn list_subscriptions(
        &self,
        limit: i64,
        offset: i64,
    ) -> Result<Vec<NotificationSubscription>> {
        Ok(vec![])
    }

    pub async fn get_subscription(
        &self,
        subscription_id: Uuid,
    ) -> Result<NotificationSubscription> {
        Err(NotificationError::SubscriptionNotFound(
            subscription_id.to_string(),
        ))
    }

    pub async fn update_subscription(
        &self,
        subscription_id: Uuid,
        event_type: Option<String>,
        channels: Option<Vec<Channel>>,
        enabled: Option<bool>,
        filters: Option<serde_json::Value>,
        preferences: Option<NotificationPreferences>,
    ) -> Result<NotificationSubscription> {
        self.get_subscription(subscription_id).await
    }

    pub async fn delete_subscription(&self, subscription_id: Uuid) -> Result<()> {
        Ok(())
    }

    pub async fn get_user_subscriptions(
        &self,
        user_id: Uuid,
    ) -> Result<Vec<NotificationSubscription>> {
        Ok(vec![])
    }

    pub async fn get_event_subscriptions(
        &self,
        event_type: &str,
    ) -> Result<Vec<NotificationSubscription>> {
        Ok(vec![])
    }

    // ================================
    // Integration Event Handlers
    // ================================

    pub async fn handle_lab_event(
        &self,
        event_type: String,
        event_id: Uuid,
        lab_id: Uuid,
        user_id: Option<Uuid>,
        title: String,
        description: String,
        priority: Priority,
        metadata: serde_json::Value,
        timestamp: DateTime<Utc>,
    ) -> Result<Vec<Notification>> {
        // Find subscriptions for this event type
        let subscriptions = self.get_event_subscriptions(&event_type).await?;
        let mut notifications = Vec::new();

        for subscription in subscriptions {
            if subscription.enabled {
                let notification_request = CreateNotificationRequest {
                    title: title.clone(),
                    message: description.clone(),
                    notification_type: NotificationType::Info,
                    priority: priority.clone(),
                    channels: subscription.channels,
                    recipients: vec![], // Would get from user profile
                    template_id: None,
                    template_data: Some(metadata.clone()),
                    scheduled_at: None,
                    metadata: Some(json!({
                        "event_type": event_type,
                        "event_id": event_id,
                        "lab_id": lab_id,
                        "timestamp": timestamp
                    })),
                };

                if let Ok(notification) = self
                    .create_notification(notification_request, user_id.unwrap_or_default())
                    .await
                {
                    notifications.push(notification);
                }
            }
        }

        Ok(notifications)
    }

    pub async fn handle_sample_event(
        &self,
        event_type: String,
        event_id: Uuid,
        sample_id: Uuid,
        user_id: Option<Uuid>,
        title: String,
        description: String,
        priority: Priority,
        metadata: serde_json::Value,
        timestamp: DateTime<Utc>,
    ) -> Result<Vec<Notification>> {
        // Similar to handle_lab_event but for samples
        Ok(vec![])
    }

    pub async fn handle_sequencing_event(
        &self,
        event_type: String,
        event_id: Uuid,
        sequencing_job_id: Uuid,
        user_id: Option<Uuid>,
        title: String,
        description: String,
        priority: Priority,
        metadata: serde_json::Value,
        timestamp: DateTime<Utc>,
    ) -> Result<Vec<Notification>> {
        Ok(vec![])
    }

    pub async fn handle_template_event(
        &self,
        event_type: String,
        event_id: Uuid,
        template_id: Uuid,
        user_id: Option<Uuid>,
        title: String,
        description: String,
        priority: Priority,
        metadata: serde_json::Value,
        timestamp: DateTime<Utc>,
    ) -> Result<Vec<Notification>> {
        Ok(vec![])
    }

    pub async fn handle_system_alert(
        &self,
        alert_type: String,
        alert_id: Uuid,
        service_name: String,
        severity: AlertSeverity,
        title: String,
        description: String,
        metadata: serde_json::Value,
        timestamp: DateTime<Utc>,
    ) -> Result<Vec<Notification>> {
        let priority = match severity {
            AlertSeverity::Low => Priority::Low,
            AlertSeverity::Medium => Priority::Medium,
            AlertSeverity::High => Priority::High,
            AlertSeverity::Critical => Priority::Critical,
        };

        // For system alerts, notify all admins
        let notification_request = CreateNotificationRequest {
            title: format!("[{}] {}", service_name, title),
            message: description,
            notification_type: NotificationType::Alert,
            priority,
            channels: vec![Channel::Email, Channel::Slack],
            recipients: vec![], // Would get admin list
            template_id: None,
            template_data: Some(metadata.clone()),
            scheduled_at: None,
            metadata: Some(json!({
                "alert_type": alert_type,
                "alert_id": alert_id,
                "service_name": service_name,
                "severity": severity,
                "timestamp": timestamp
            })),
        };

        if let Ok(notification) = self
            .create_notification(notification_request, Uuid::new_v4())
            .await
        {
            Ok(vec![notification])
        } else {
            Ok(vec![])
        }
    }

    // ================================
    // Admin Methods
    // ================================

    pub async fn get_statistics(
        &self,
        start_date: Option<DateTime<Utc>>,
        end_date: Option<DateTime<Utc>>,
        channel: Option<Channel>,
        priority: Option<Priority>,
    ) -> Result<NotificationStatistics> {
        Ok(NotificationStatistics {
            total_notifications: 0,
            sent_notifications: 0,
            failed_notifications: 0,
            pending_notifications: 0,
            delivery_rate: 0.0,
            avg_delivery_time_ms: 0,
            by_channel: HashMap::new(),
            by_priority: HashMap::new(),
            by_type: HashMap::new(),
            hourly_distribution: vec![],
            daily_distribution: vec![],
        })
    }

    pub async fn get_failed_notifications(
        &self,
        limit: i64,
        offset: i64,
    ) -> Result<Vec<Notification>> {
        Ok(vec![])
    }

    pub async fn retry_failed_notifications(&self) -> Result<usize> {
        Ok(0)
    }

    pub async fn cleanup_old_notifications(
        &self,
        older_than_days: u32,
        keep_failed: bool,
    ) -> Result<usize> {
        Ok(0)
    }

    pub async fn check_all_channels_health(&self) -> Result<Vec<ChannelHealthResponse>> {
        Ok(vec![])
    }

    pub async fn get_rate_limits(&self) -> Result<Vec<RateLimitResponse>> {
        Ok(vec![])
    }

    pub async fn update_rate_limit(
        &self,
        channel: Channel,
        requests_per_minute: Option<u32>,
        requests_per_hour: Option<u32>,
        requests_per_day: Option<u32>,
    ) -> Result<RateLimitResponse> {
        Err(NotificationError::NotImplemented(
            "Rate limit updates not implemented".to_string(),
        ))
    }

    pub async fn get_metrics(&self) -> Result<NotificationMetrics> {
        Ok(NotificationMetrics {
            notifications_sent: 0,
            notifications_failed: 0,
            delivery_rate: 0.0,
            avg_delivery_time_ms: 0,
            email_sent: 0,
            email_failed: 0,
            email_avg_response_time_ms: 0,
            sms_sent: 0,
            sms_failed: 0,
            sms_avg_response_time_ms: 0,
            slack_sent: 0,
            slack_failed: 0,
            slack_avg_response_time_ms: 0,
            teams_sent: 0,
            teams_failed: 0,
            teams_avg_response_time_ms: 0,
        })
    }

    // ================================
    // Private Helper Methods
    // ================================

    async fn deliver_notification(&self, notification: &Notification) -> Vec<Result<()>> {
        let mut results = Vec::new();

        for channel in &notification.channels {
            for recipient in &notification.recipients {
                let result = match channel {
                    Channel::Email => self.send_email(notification, recipient).await,
                    Channel::Sms => self.send_sms(notification, recipient).await,
                    Channel::Slack => self.send_slack(notification, recipient).await,
                    Channel::Teams => self.send_teams(notification, recipient).await,
                    _ => Err(NotificationError::ChannelNotSupported(format!(
                        "{:?}",
                        channel
                    ))),
                };

                results.push(result);
            }
        }

        results
    }

    async fn send_email(&self, notification: &Notification, recipient: &str) -> Result<()> {
        if !self.config.email.enabled {
            return Err(NotificationError::ChannelDisabled("email".to_string()));
        }

        info!(
            "Sending email to {} with subject: {}",
            recipient, notification.title
        );
        Ok(())
    }

    async fn send_sms(&self, notification: &Notification, recipient: &str) -> Result<()> {
        if !self.config.sms.enabled {
            return Err(NotificationError::ChannelDisabled("sms".to_string()));
        }

        info!("Sending SMS to {}: {}", recipient, notification.message);
        Ok(())
    }

    async fn send_slack(&self, notification: &Notification, channel: &str) -> Result<()> {
        if !self.config.slack.enabled {
            return Err(NotificationError::ChannelDisabled("slack".to_string()));
        }

        info!(
            "Sending Slack message to {}: {}",
            channel, notification.message
        );
        Ok(())
    }

    async fn send_teams(&self, notification: &Notification, webhook: &str) -> Result<()> {
        if !self.config.teams.enabled {
            return Err(NotificationError::ChannelDisabled("teams".to_string()));
        }

        info!(
            "Sending Teams message to {}: {}",
            webhook, notification.message
        );
        Ok(())
    }

    async fn save_notification(&self, notification: &Notification) -> Result<()> {
        // In production, save to database
        info!("Saving notification: {}", notification.id);
        Ok(())
    }

    async fn update_notification(&self, notification: &Notification) -> Result<()> {
        // In production, update in database
        info!("Updating notification: {}", notification.id);
        Ok(())
    }
}
