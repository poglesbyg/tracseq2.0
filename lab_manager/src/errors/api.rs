use crate::errors::{ComponentError, ErrorSeverity};
use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde_json::json;

/// API-specific errors
#[derive(Debug, thiserror::Error)]
pub enum ApiError {
    #[error("Invalid request format: {0}")]
    InvalidRequest(String),
    #[error("Bad request: {0}")]
    BadRequest(String),
    #[error("Authentication required")]
    Unauthorized,
    #[error("Access forbidden")]
    Forbidden,
    #[error("Resource not found: {0}")]
    NotFound(String),
    #[error("Rate limit exceeded")]
    RateLimited,
    #[error("Too many requests: {0}")]
    TooManyRequests(String),
    #[error("Validation error: {0}")]
    ValidationError(String),
    #[error("Database error: {0}")]
    DatabaseError(String),
    #[error("Conflict: {0}")]
    Conflict(String),
    #[error("Service unavailable: {0}")]
    ServiceUnavailable(String),
    #[error("Internal server error: {0}")]
    InternalServerError(String),
    #[error("Internal server error")]
    InternalError,
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        let error_code = self.error_code();
        let severity = format!("{:?}", self.severity()).to_lowercase();
        let retryable = self.is_retryable();
        
        let (status, error_message) = match self {
            ApiError::InvalidRequest(msg) => (StatusCode::BAD_REQUEST, msg),
            ApiError::BadRequest(msg) => (StatusCode::BAD_REQUEST, msg),
            ApiError::Unauthorized => (StatusCode::UNAUTHORIZED, "Authentication required".to_string()),
            ApiError::Forbidden => (StatusCode::FORBIDDEN, "Access forbidden".to_string()),
            ApiError::NotFound(msg) => (StatusCode::NOT_FOUND, msg),
            ApiError::RateLimited => (StatusCode::TOO_MANY_REQUESTS, "Rate limit exceeded".to_string()),
            ApiError::TooManyRequests(msg) => (StatusCode::TOO_MANY_REQUESTS, msg),
            ApiError::ValidationError(msg) => (StatusCode::BAD_REQUEST, msg),
            ApiError::DatabaseError(msg) => (StatusCode::INTERNAL_SERVER_ERROR, format!("Database error: {}", msg)),
            ApiError::Conflict(msg) => (StatusCode::CONFLICT, msg),
            ApiError::ServiceUnavailable(msg) => (StatusCode::SERVICE_UNAVAILABLE, msg),
            ApiError::InternalServerError(msg) => (StatusCode::INTERNAL_SERVER_ERROR, msg),
            ApiError::InternalError => (StatusCode::INTERNAL_SERVER_ERROR, "Internal server error".to_string()),
        };

        let body = Json(json!({
            "error": {
                "code": error_code,
                "message": error_message,
                "severity": severity,
                "retryable": retryable
            }
        }));

        (status, body).into_response()
    }
}

impl ComponentError for ApiError {
    fn error_code(&self) -> &'static str {
        match self {
            Self::InvalidRequest(_) => "API_INVALID_REQUEST",
            Self::BadRequest(_) => "API_BAD_REQUEST",
            Self::Unauthorized => "API_UNAUTHORIZED",
            Self::Forbidden => "API_FORBIDDEN",
            Self::NotFound(_) => "API_NOT_FOUND",
            Self::RateLimited => "API_RATE_LIMITED",
            Self::TooManyRequests(_) => "API_TOO_MANY_REQUESTS",
            Self::ValidationError(_) => "API_VALIDATION_ERROR",
            Self::DatabaseError(_) => "API_DATABASE_ERROR",
            Self::Conflict(_) => "API_CONFLICT",
            Self::ServiceUnavailable(_) => "API_SERVICE_UNAVAILABLE",
            Self::InternalServerError(_) => "API_INTERNAL_SERVER_ERROR",
            Self::InternalError => "API_INTERNAL_ERROR",
        }
    }

    fn severity(&self) -> ErrorSeverity {
        match self {
            Self::InvalidRequest(_) => ErrorSeverity::Low,
            Self::BadRequest(_) => ErrorSeverity::Low,
            Self::Unauthorized | Self::Forbidden => ErrorSeverity::Medium,
            Self::NotFound(_) => ErrorSeverity::Low,
            Self::RateLimited => ErrorSeverity::Medium,
            Self::TooManyRequests(_) => ErrorSeverity::Medium,
            Self::ValidationError(_) => ErrorSeverity::Low,
            Self::DatabaseError(_) => ErrorSeverity::High,
            Self::Conflict(_) => ErrorSeverity::Medium,
            Self::ServiceUnavailable(_) => ErrorSeverity::Medium,
            Self::InternalServerError(_) => ErrorSeverity::High,
            Self::InternalError => ErrorSeverity::High,
        }
    }

    fn is_retryable(&self) -> bool {
        matches!(
            self,
            Self::RateLimited | Self::TooManyRequests(_) | Self::InternalError
        )
    }
}
