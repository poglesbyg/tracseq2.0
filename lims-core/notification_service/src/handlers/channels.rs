use axum::{
    Json,
    extract::{Path, State},
};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::{AppState, error::Result, models::*};

#[derive(Debug, Deserialize)]
pub struct TestChannelRequest {
    pub recipient: String,
    pub message: Option<String>,
    pub subject: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct ChannelTestResponse {
    pub channel: Channel,
    pub success: bool,
    pub message: String,
    pub response_time_ms: u64,
    pub error_details: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct ChannelConfigResponse {
    pub channel: Channel,
    pub enabled: bool,
    pub config: serde_json::Value,
    pub rate_limit: Option<crate::models::RateLimit>,
    pub last_test: Option<chrono::DateTime<chrono::Utc>>,
    pub last_test_result: Option<bool>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateChannelConfigRequest {
    pub enabled: Option<bool>,
    pub config: Option<serde_json::Value>,
    pub rate_limit: Option<crate::models::RateLimit>,
}

#[derive(Debug, Deserialize)]
pub struct CreateSlackWebhookRequest {
    pub name: String,
    pub webhook_url: String,
    pub channel: String,
    pub username: Option<String>,
    pub icon_emoji: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct SlackWebhookResponse {
    pub id: Uuid,
    pub name: String,
    pub webhook_url: String,
    pub channel: String,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

/// List all available channels
/// GET /channels
pub async fn list_channels(
    State(state): State<AppState>,
) -> Result<Json<Vec<ChannelConfigResponse>>> {
    let channels = state.notification_service.list_channels().await?;
    Ok(Json(channels))
}

/// Test email channel
/// POST /channels/email/test
pub async fn test_email_channel(
    State(state): State<AppState>,
    Json(request): Json<TestChannelRequest>,
) -> Result<Json<ChannelTestResponse>> {
    let start_time = std::time::Instant::now();

    let result = state
        .notification_service
        .test_email_channel(
            &request.recipient,
            request.subject.as_deref().unwrap_or("Test Email"),
            request
                .message
                .as_deref()
                .unwrap_or("This is a test email from TracSeq Notification Service"),
        )
        .await;

    let response_time = start_time.elapsed().as_millis() as u64;

    Ok(Json(ChannelTestResponse {
        channel: Channel::Email,
        success: result.is_ok(),
        message: if result.is_ok() {
            "Test email sent successfully".to_string()
        } else {
            "Failed to send test email".to_string()
        },
        response_time_ms: response_time,
        error_details: result.err().map(|e| e.to_string()),
    }))
}

/// Test SMS channel
/// POST /channels/sms/test
pub async fn test_sms_channel(
    State(state): State<AppState>,
    Json(request): Json<TestChannelRequest>,
) -> Result<Json<ChannelTestResponse>> {
    let start_time = std::time::Instant::now();

    let result = state
        .notification_service
        .test_sms_channel(
            &request.recipient,
            request
                .message
                .as_deref()
                .unwrap_or("This is a test SMS from TracSeq Notification Service"),
        )
        .await;

    let response_time = start_time.elapsed().as_millis() as u64;

    Ok(Json(ChannelTestResponse {
        channel: Channel::Sms,
        success: result.is_ok(),
        message: if result.is_ok() {
            "Test SMS sent successfully".to_string()
        } else {
            "Failed to send test SMS".to_string()
        },
        response_time_ms: response_time,
        error_details: result.err().map(|e| e.to_string()),
    }))
}

/// Test Slack channel
/// POST /channels/slack/test
pub async fn test_slack_channel(
    State(state): State<AppState>,
    Json(request): Json<TestChannelRequest>,
) -> Result<Json<ChannelTestResponse>> {
    let start_time = std::time::Instant::now();

    let result = state
        .notification_service
        .test_slack_channel(
            &request.recipient,
            request
                .message
                .as_deref()
                .unwrap_or("This is a test message from TracSeq Notification Service"),
        )
        .await;

    let response_time = start_time.elapsed().as_millis() as u64;

    Ok(Json(ChannelTestResponse {
        channel: Channel::Slack,
        success: result.is_ok(),
        message: if result.is_ok() {
            "Test Slack message sent successfully".to_string()
        } else {
            "Failed to send test Slack message".to_string()
        },
        response_time_ms: response_time,
        error_details: result.err().map(|e| e.to_string()),
    }))
}

/// Test Teams channel
/// POST /channels/teams/test
pub async fn test_teams_channel(
    State(state): State<AppState>,
    Json(request): Json<TestChannelRequest>,
) -> Result<Json<ChannelTestResponse>> {
    let start_time = std::time::Instant::now();

    let result = state
        .notification_service
        .test_teams_channel(
            &request.recipient,
            request
                .message
                .as_deref()
                .unwrap_or("This is a test message from TracSeq Notification Service"),
        )
        .await;

    let response_time = start_time.elapsed().as_millis() as u64;

    Ok(Json(ChannelTestResponse {
        channel: Channel::Teams,
        success: result.is_ok(),
        message: if result.is_ok() {
            "Test Teams message sent successfully".to_string()
        } else {
            "Failed to send test Teams message".to_string()
        },
        response_time_ms: response_time,
        error_details: result.err().map(|e| e.to_string()),
    }))
}

/// Get channel configuration
/// GET /channels/{channel_type}/config
pub async fn get_channel_config(
    State(state): State<AppState>,
    Path(channel_type): Path<String>,
) -> Result<Json<ChannelConfigResponse>> {
    let channel = match channel_type.as_str() {
        "email" => Channel::Email,
        "sms" => Channel::Sms,
        "slack" => Channel::Slack,
        "teams" => Channel::Teams,
        _ => {
            return Err(crate::error::NotificationError::InvalidChannel(
                channel_type,
            ));
        }
    };

    let config = state
        .notification_service
        .get_channel_config(channel)
        .await?;
    Ok(Json(config))
}

/// Update channel configuration
/// PUT /channels/{channel_type}/config
pub async fn update_channel_config(
    State(state): State<AppState>,
    Path(channel_type): Path<String>,
    Json(request): Json<UpdateChannelConfigRequest>,
) -> Result<Json<ChannelConfigResponse>> {
    let channel = match channel_type.as_str() {
        "email" => Channel::Email,
        "sms" => Channel::Sms,
        "slack" => Channel::Slack,
        "teams" => Channel::Teams,
        _ => {
            return Err(crate::error::NotificationError::InvalidChannel(
                channel_type,
            ));
        }
    };

    let config = state
        .notification_service
        .update_channel_config(channel, request.enabled, request.config, request.rate_limit)
        .await?;

    Ok(Json(config))
}

/// List email templates
/// GET /channels/email/templates
pub async fn list_email_templates(
    State(state): State<AppState>,
) -> Result<Json<Vec<EmailTemplate>>> {
    let templates = state.notification_service.list_email_templates().await?;
    Ok(Json(templates))
}

/// Create Slack webhook
/// POST /channels/slack/webhooks
pub async fn create_slack_webhook(
    State(state): State<AppState>,
    Json(request): Json<CreateSlackWebhookRequest>,
) -> Result<Json<SlackWebhookResponse>> {
    let webhook = state
        .notification_service
        .create_slack_webhook(
            request.name,
            request.webhook_url,
            request.channel,
            request.username,
            request.icon_emoji,
        )
        .await?;

    Ok(Json(SlackWebhookResponse {
        id: webhook.id,
        name: webhook.name,
        webhook_url: webhook.webhook_url,
        channel: webhook.channel,
        created_at: webhook.created_at,
    }))
}

#[derive(Debug, Serialize)]
pub struct EmailTemplate {
    pub id: Uuid,
    pub name: String,
    pub subject: String,
    pub body_html: String,
    pub body_text: String,
    pub variables: Vec<String>,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Serialize)]
pub struct SlackWebhook {
    pub id: Uuid,
    pub name: String,
    pub webhook_url: String,
    pub channel: String,
    pub username: Option<String>,
    pub icon_emoji: Option<String>,
    pub created_at: chrono::DateTime<chrono::Utc>,
}
