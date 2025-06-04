use crate::errors::{ComponentError, ErrorSeverity};

/// API-specific errors
#[derive(Debug, thiserror::Error)]
pub enum ApiError {
    #[error("Invalid request format")]
    InvalidRequest,
    #[error("Authentication required")]
    Unauthorized,
    #[error("Access forbidden")]
    Forbidden,
    #[error("Resource not found")]
    NotFound,
    #[error("Rate limit exceeded")]
    RateLimited,
    #[error("Internal server error")]
    InternalError,
}

impl ComponentError for ApiError {
    fn error_code(&self) -> &'static str {
        match self {
            Self::InvalidRequest => "API_INVALID_REQUEST",
            Self::Unauthorized => "API_UNAUTHORIZED",
            Self::Forbidden => "API_FORBIDDEN",
            Self::NotFound => "API_NOT_FOUND",
            Self::RateLimited => "API_RATE_LIMITED",
            Self::InternalError => "API_INTERNAL_ERROR",
        }
    }

    fn severity(&self) -> ErrorSeverity {
        match self {
            Self::InvalidRequest => ErrorSeverity::Low,
            Self::Unauthorized | Self::Forbidden => ErrorSeverity::Medium,
            Self::NotFound => ErrorSeverity::Low,
            Self::RateLimited => ErrorSeverity::Medium,
            Self::InternalError => ErrorSeverity::High,
        }
    }

    fn is_retryable(&self) -> bool {
        matches!(self, Self::RateLimited | Self::InternalError)
    }
}
