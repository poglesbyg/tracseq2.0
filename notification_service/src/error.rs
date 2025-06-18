use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::Json;
use serde_json::json;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum NotificationError {
    #[error("Database error: {0}")]
    Database(#[from] sqlx::Error),

    #[error("Configuration error: {0}")]
    Configuration(String),

    #[error("Validation error: {0}")]
    Validation(String),

    #[error("Authentication error: {0}")]
    Authentication(String),

    #[error("Authorization error: {0}")]
    Authorization(String),

    #[error("Template error: {0}")]
    Template(String),

    #[error("Channel error: {channel}: {message}")]
    Channel { channel: String, message: String },

    #[error("Rate limit exceeded: {0}")]
    RateLimit(String),

    #[error("Notification not found: {0}")]
    NotificationNotFound(String),

    #[error("Template not found: {0}")]
    TemplateNotFound(String),

    #[error("Subscription not found: {0}")]
    SubscriptionNotFound(String),

    #[error("External service error: {service}: {message}")]
    ExternalService { service: String, message: String },

    #[error("Email delivery error: {0}")]
    EmailDelivery(String),

    #[error("SMS delivery error: {0}")]
    SmsDelivery(String),

    #[error("Slack delivery error: {0}")]
    SlackDelivery(String),

    #[error("Teams delivery error: {0}")]
    TeamsDelivery(String),

    #[error("Discord delivery error: {0}")]
    DiscordDelivery(String),

    #[error("Webhook delivery error: {0}")]
    WebhookDelivery(String),

    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),

    #[error("HTTP client error: {0}")]
    HttpClient(#[from] reqwest::Error),

    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Internal server error: {0}")]
    Internal(String),
}

impl NotificationError {
    pub fn status_code(&self) -> StatusCode {
        match self {
            NotificationError::Database(_) => StatusCode::INTERNAL_SERVER_ERROR,
            NotificationError::Configuration(_) => StatusCode::INTERNAL_SERVER_ERROR,
            NotificationError::Validation(_) => StatusCode::BAD_REQUEST,
            NotificationError::Authentication(_) => StatusCode::UNAUTHORIZED,
            NotificationError::Authorization(_) => StatusCode::FORBIDDEN,
            NotificationError::Template(_) => StatusCode::BAD_REQUEST,
            NotificationError::Channel { .. } => StatusCode::BAD_GATEWAY,
            NotificationError::RateLimit(_) => StatusCode::TOO_MANY_REQUESTS,
            NotificationError::NotificationNotFound(_) => StatusCode::NOT_FOUND,
            NotificationError::TemplateNotFound(_) => StatusCode::NOT_FOUND,
            NotificationError::SubscriptionNotFound(_) => StatusCode::NOT_FOUND,
            NotificationError::ExternalService { .. } => StatusCode::BAD_GATEWAY,
            NotificationError::EmailDelivery(_) => StatusCode::BAD_GATEWAY,
            NotificationError::SmsDelivery(_) => StatusCode::BAD_GATEWAY,
            NotificationError::SlackDelivery(_) => StatusCode::BAD_GATEWAY,
            NotificationError::TeamsDelivery(_) => StatusCode::BAD_GATEWAY,
            NotificationError::DiscordDelivery(_) => StatusCode::BAD_GATEWAY,
            NotificationError::WebhookDelivery(_) => StatusCode::BAD_GATEWAY,
            NotificationError::Serialization(_) => StatusCode::INTERNAL_SERVER_ERROR,
            NotificationError::HttpClient(_) => StatusCode::BAD_GATEWAY,
            NotificationError::Io(_) => StatusCode::INTERNAL_SERVER_ERROR,
            NotificationError::Internal(_) => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }

    pub fn error_code(&self) -> &'static str {
        match self {
            NotificationError::Database(_) => "DATABASE_ERROR",
            NotificationError::Configuration(_) => "CONFIGURATION_ERROR",
            NotificationError::Validation(_) => "VALIDATION_ERROR",
            NotificationError::Authentication(_) => "AUTHENTICATION_ERROR",
            NotificationError::Authorization(_) => "AUTHORIZATION_ERROR",
            NotificationError::Template(_) => "TEMPLATE_ERROR",
            NotificationError::Channel { .. } => "CHANNEL_ERROR",
            NotificationError::RateLimit(_) => "RATE_LIMIT_EXCEEDED",
            NotificationError::NotificationNotFound(_) => "NOTIFICATION_NOT_FOUND",
            NotificationError::TemplateNotFound(_) => "TEMPLATE_NOT_FOUND",
            NotificationError::SubscriptionNotFound(_) => "SUBSCRIPTION_NOT_FOUND",
            NotificationError::ExternalService { .. } => "EXTERNAL_SERVICE_ERROR",
            NotificationError::EmailDelivery(_) => "EMAIL_DELIVERY_ERROR",
            NotificationError::SmsDelivery(_) => "SMS_DELIVERY_ERROR",
            NotificationError::SlackDelivery(_) => "SLACK_DELIVERY_ERROR",
            NotificationError::TeamsDelivery(_) => "TEAMS_DELIVERY_ERROR",
            NotificationError::DiscordDelivery(_) => "DISCORD_DELIVERY_ERROR",
            NotificationError::WebhookDelivery(_) => "WEBHOOK_DELIVERY_ERROR",
            NotificationError::Serialization(_) => "SERIALIZATION_ERROR",
            NotificationError::HttpClient(_) => "HTTP_CLIENT_ERROR",
            NotificationError::Io(_) => "IO_ERROR",
            NotificationError::Internal(_) => "INTERNAL_ERROR",
        }
    }

    pub fn is_retryable(&self) -> bool {
        match self {
            NotificationError::Database(_) => true,
            NotificationError::Configuration(_) => false,
            NotificationError::Validation(_) => false,
            NotificationError::Authentication(_) => false,
            NotificationError::Authorization(_) => false,
            NotificationError::Template(_) => false,
            NotificationError::Channel { .. } => true,
            NotificationError::RateLimit(_) => true,
            NotificationError::NotificationNotFound(_) => false,
            NotificationError::TemplateNotFound(_) => false,
            NotificationError::SubscriptionNotFound(_) => false,
            NotificationError::ExternalService { .. } => true,
            NotificationError::EmailDelivery(_) => true,
            NotificationError::SmsDelivery(_) => true,
            NotificationError::SlackDelivery(_) => true,
            NotificationError::TeamsDelivery(_) => true,
            NotificationError::DiscordDelivery(_) => true,
            NotificationError::WebhookDelivery(_) => true,
            NotificationError::Serialization(_) => false,
            NotificationError::HttpClient(_) => true,
            NotificationError::Io(_) => true,
            NotificationError::Internal(_) => false,
        }
    }
}

impl IntoResponse for NotificationError {
    fn into_response(self) -> Response {
        let status = self.status_code();
        let error_code = self.error_code();
        let message = self.to_string();
        let is_retryable = self.is_retryable();

        tracing::error!(
            error = %self,
            error_code = error_code,
            status_code = %status,
            is_retryable = is_retryable,
            "Notification service error"
        );

        let body = Json(json!({
            "error": {
                "code": error_code,
                "message": message,
                "retryable": is_retryable,
                "timestamp": chrono::Utc::now(),
            }
        }));

        (status, body).into_response()
    }
}

pub type Result<T> = std::result::Result<T, NotificationError>;

// Convenience functions for creating specific errors
impl NotificationError {
    pub fn validation<S: Into<String>>(message: S) -> Self {
        NotificationError::Validation(message.into())
    }

    pub fn template<S: Into<String>>(message: S) -> Self {
        NotificationError::Template(message.into())
    }

    pub fn channel<S: Into<String>>(channel: S, message: S) -> Self {
        NotificationError::Channel {
            channel: channel.into(),
            message: message.into(),
        }
    }

    pub fn external_service<S: Into<String>>(service: S, message: S) -> Self {
        NotificationError::ExternalService {
            service: service.into(),
            message: message.into(),
        }
    }

    pub fn internal<S: Into<String>>(message: S) -> Self {
        NotificationError::Internal(message.into())
    }
}
