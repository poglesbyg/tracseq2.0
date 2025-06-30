pub mod api;
pub mod database;
pub mod storage;
pub mod validation;

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

/// Core error trait that all component errors must implement
pub trait ComponentError: std::error::Error + Send + Sync {
    /// Get the error code for API responses
    fn error_code(&self) -> &'static str;

    /// Get the error severity level
    fn severity(&self) -> ErrorSeverity;

    /// Get additional context data
    fn context(&self) -> HashMap<String, String> {
        HashMap::new()
    }

    /// Check if the error is retryable
    fn is_retryable(&self) -> bool {
        false
    }
}

/// Error severity levels for monitoring and alerting
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ErrorSeverity {
    Low,      // Informational, user input errors
    Medium,   // Business logic errors, expected failures
    High,     // System errors, unexpected failures
    Critical, // Service-breaking errors, security issues
}

/// Unified error response for API consistency
#[derive(Debug, Serialize, Deserialize)]
pub struct ErrorResponse {
    pub error_id: Uuid,
    pub error_code: String,
    pub message: String,
    pub severity: ErrorSeverity,
    pub context: HashMap<String, String>,
    pub retryable: bool,
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

impl ErrorResponse {
    pub fn from_component_error<E: ComponentError>(error: E) -> Self {
        Self {
            error_id: Uuid::new_v4(),
            error_code: error.error_code().to_string(),
            message: error.to_string(),
            severity: error.severity(),
            context: error.context(),
            retryable: error.is_retryable(),
            timestamp: chrono::Utc::now(),
        }
    }
}

/// Error aggregator for collecting multiple component errors
#[derive(Debug)]
pub struct ErrorCollector {
    errors: Vec<ErrorResponse>,
}

impl ErrorCollector {
    pub fn new() -> Self {
        Self { errors: Vec::new() }
    }

    pub fn add_error<E: ComponentError>(&mut self, error: E) {
        self.errors.push(ErrorResponse::from_component_error(error));
    }

    pub fn has_errors(&self) -> bool {
        !self.errors.is_empty()
    }

    pub fn has_critical_errors(&self) -> bool {
        self.errors
            .iter()
            .any(|e| matches!(e.severity, ErrorSeverity::Critical))
    }

    pub fn into_errors(self) -> Vec<ErrorResponse> {
        self.errors
    }
}

impl Default for ErrorCollector {
    fn default() -> Self {
        Self::new()
    }
}

/// Result type alias for component operations
pub type ComponentResult<T, E> = Result<T, E>;

/// Macro for creating component-specific error types
#[macro_export]
macro_rules! component_error {
    (
        $name:ident {
            $(
                $variant:ident {
                    code: $code:expr,
                    severity: $severity:expr,
                    message: $message:expr,
                    $(retryable: $retryable:expr,)?
                }
            ),* $(,)?
        }
    ) => {
        #[derive(Debug, thiserror::Error)]
        pub enum $name {
            $(
                #[error($message)]
                $variant,
            )*
        }

        impl $crate::errors::ComponentError for $name {
            fn error_code(&self) -> &'static str {
                match self {
                    $(
                        Self::$variant => $code,
                    )*
                }
            }

            fn severity(&self) -> $crate::errors::ErrorSeverity {
                match self {
                    $(
                        Self::$variant => $severity,
                    )*
                }
            }

            fn is_retryable(&self) -> bool {
                match self {
                    $(
                        Self::$variant => component_error!(@retryable $($retryable)?),
                    )*
                }
            }
        }
    };

    (@retryable) => { false };
    (@retryable $retryable:expr) => { $retryable };
}

/// Error handler trait for different response formats
pub trait ErrorHandler<T> {
    fn handle_error<E: ComponentError>(error: E) -> T;
}

/// HTTP error handler for API responses
pub struct HttpErrorHandler;

impl ErrorHandler<(axum::http::StatusCode, axum::Json<ErrorResponse>)> for HttpErrorHandler {
    fn handle_error<E: ComponentError>(
        error: E,
    ) -> (axum::http::StatusCode, axum::Json<ErrorResponse>) {
        use axum::http::StatusCode;

        let status = match error.severity() {
            ErrorSeverity::Low => StatusCode::BAD_REQUEST,
            ErrorSeverity::Medium => StatusCode::UNPROCESSABLE_ENTITY,
            ErrorSeverity::High => StatusCode::INTERNAL_SERVER_ERROR,
            ErrorSeverity::Critical => StatusCode::SERVICE_UNAVAILABLE,
        };

        let response = ErrorResponse::from_component_error(error);
        (status, axum::Json(response))
    }
}

/// Standardized error wrapper for all API responses
#[derive(Debug, Serialize, Deserialize)]
pub struct ApiErrorResponse {
    pub error: ErrorResponse,
    pub request_id: Uuid,
    pub trace_id: Option<String>,
}

/// Error context builder for better debugging
pub struct ErrorContextBuilder {
    error: Box<dyn ComponentError>,
    context: HashMap<String, String>,
    trace_id: Option<String>,
}

impl ErrorContextBuilder {
    pub fn new<E: ComponentError + 'static>(error: E) -> Self {
        Self {
            error: Box::new(error),
            context: HashMap::new(),
            trace_id: None,
        }
    }

    pub fn with_context(mut self, key: &str, value: &str) -> Self {
        self.context.insert(key.to_string(), value.to_string());
        self
    }

    pub fn with_trace_id(mut self, trace_id: String) -> Self {
        self.trace_id = Some(trace_id);
        self
    }

    pub fn build(self) -> ApiErrorResponse {
        let mut error_response = ErrorResponse {
            error_id: Uuid::new_v4(),
            error_code: self.error.error_code().to_string(),
            message: self.error.to_string(),
            severity: self.error.severity(),
            context: self.error.context(),
            retryable: self.error.is_retryable(),
            timestamp: chrono::Utc::now(),
        };
        error_response.context.extend(self.context);

        ApiErrorResponse {
            error: error_response,
            request_id: Uuid::new_v4(),
            trace_id: self.trace_id,
        }
    }
}

/// Result type with enhanced error information
pub type ApiResult<T> = Result<T, ApiErrorResponse>;
