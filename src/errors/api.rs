use crate::errors::{ComponentError, ErrorSeverity};

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
    #[error("Resource not found")]
    NotFound,
    #[error("Rate limit exceeded")]
    RateLimited,
    #[error("Service unavailable: {0}")]
    ServiceUnavailable(String),
    #[error("Internal server error: {0}")]
    InternalServerError(String),
    #[error("Internal server error")]
    InternalError,
}

impl ComponentError for ApiError {
    fn error_code(&self) -> &'static str {
        match self {
            Self::InvalidRequest(_) => "API_INVALID_REQUEST",
            Self::BadRequest(_) => "API_BAD_REQUEST",
            Self::Unauthorized => "API_UNAUTHORIZED",
            Self::Forbidden => "API_FORBIDDEN",
            Self::NotFound => "API_NOT_FOUND",
            Self::RateLimited => "API_RATE_LIMITED",
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
            Self::NotFound => ErrorSeverity::Low,
            Self::RateLimited => ErrorSeverity::Medium,
            Self::ServiceUnavailable(_) => ErrorSeverity::Medium,
            Self::InternalServerError(_) => ErrorSeverity::High,
            Self::InternalError => ErrorSeverity::High,
        }
    }

    fn is_retryable(&self) -> bool {
        matches!(self, Self::RateLimited | Self::InternalError)
    }
}
