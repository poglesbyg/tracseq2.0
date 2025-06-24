use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde::{Deserialize, Serialize};
use thiserror::Error;

/// Sample service error types
#[derive(Error, Debug)]
pub enum SampleServiceError {
    #[error("Database error: {0}")]
    Database(#[from] sqlx::Error),

    #[error("Validation error: {0}")]
    Validation(String),

    #[error("Sample not found: {sample_id}")]
    SampleNotFound { sample_id: String },

    #[error("Barcode not found: {barcode}")]
    BarcodeNotFound { barcode: String },

    #[error("Duplicate barcode: {barcode}")]
    DuplicateBarcode { barcode: String },

    #[error("Invalid workflow transition from {current_status} to {requested_status}")]
    InvalidWorkflowTransition {
        current_status: String,
        requested_status: String,
    },

    #[error("Template not found: {template_id}")]
    TemplateNotFound { template_id: String },

    #[error("Template validation failed: {0}")]
    TemplateValidation(String),

    #[error("Barcode generation failed: {0}")]
    BarcodeGeneration(String),

    #[error("Batch processing error: {0}")]
    BatchProcessing(String),

    #[error("External service error: {service} - {message}")]
    ExternalService { service: String, message: String },

    #[error("Authentication error: {0}")]
    Authentication(String),

    #[error("Authorization error: {0}")]
    Authorization(String),

    #[error("Configuration error: {0}")]
    Configuration(String),

    #[error("Business rule violation: {0}")]
    BusinessRule(String),

    #[error("Concurrent modification detected")]
    ConcurrentModification,

    #[error("Resource limit exceeded: {0}")]
    ResourceLimit(String),

    #[error("Internal server error: {0}")]
    Internal(String),
}

/// API error response structure
#[derive(Debug, Serialize, Deserialize)]
pub struct ErrorResponse {
    pub error: String,
    pub message: String,
    pub details: Option<serde_json::Value>,
    pub timestamp: chrono::DateTime<chrono::Utc>,
    pub request_id: Option<String>,
}

impl ErrorResponse {
    pub fn new(error: &str, message: &str) -> Self {
        Self {
            error: error.to_string(),
            message: message.to_string(),
            details: None,
            timestamp: chrono::Utc::now(),
            request_id: None,
        }
    }

    pub fn with_details(mut self, details: serde_json::Value) -> Self {
        self.details = Some(details);
        self
    }

    pub fn with_request_id(mut self, request_id: String) -> Self {
        self.request_id = Some(request_id);
        self
    }
}

/// Convert validation errors to API error response
impl From<validator::ValidationErrors> for SampleServiceError {
    fn from(errors: validator::ValidationErrors) -> Self {
        let error_messages: Vec<String> = errors
            .field_errors()
            .iter()
            .flat_map(|(field, field_errors)| {
                field_errors.iter().map(move |error| {
                    format!(
                        "{}: {}",
                        field,
                        error.message.as_ref().unwrap_or(&"Invalid value".into())
                    )
                })
            })
            .collect();

        SampleServiceError::Validation(error_messages.join(", "))
    }
}

/// Convert anyhow errors to sample service errors
impl From<anyhow::Error> for SampleServiceError {
    fn from(error: anyhow::Error) -> Self {
        SampleServiceError::Internal(error.to_string())
    }
}

/// Convert JSON errors to sample service errors
impl From<serde_json::Error> for SampleServiceError {
    fn from(error: serde_json::Error) -> Self {
        SampleServiceError::Validation(format!("JSON parsing error: {}", error))
    }
}

/// Convert HTTP client errors to sample service errors
impl From<reqwest::Error> for SampleServiceError {
    fn from(error: reqwest::Error) -> Self {
        SampleServiceError::ExternalService {
            service: "HTTP Client".to_string(),
            message: error.to_string(),
        }
    }
}

/// Implement IntoResponse for SampleServiceError
impl IntoResponse for SampleServiceError {
    fn into_response(self) -> Response {
        let (status_code, error_type, message) = match &self {
            SampleServiceError::Database(_) => {
                tracing::error!("Database error: {}", self);
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "database_error",
                    "A database error occurred",
                )
            }
            SampleServiceError::Validation(msg) => {
                (StatusCode::BAD_REQUEST, "validation_error", msg.as_str())
            }
            SampleServiceError::SampleNotFound { .. } => {
                (StatusCode::NOT_FOUND, "sample_not_found", "Sample not found")
            }
            SampleServiceError::BarcodeNotFound { .. } => (
                StatusCode::NOT_FOUND,
                "barcode_not_found",
                "Barcode not found",
            ),
            SampleServiceError::DuplicateBarcode { .. } => {
                (StatusCode::CONFLICT, "duplicate_barcode", "Duplicate barcode")
            }
            SampleServiceError::InvalidWorkflowTransition { .. } => (
                StatusCode::BAD_REQUEST,
                "invalid_workflow_transition",
                "Invalid workflow transition",
            ),
            SampleServiceError::TemplateNotFound { .. } => (
                StatusCode::NOT_FOUND,
                "template_not_found",
                "Template not found",
            ),
            SampleServiceError::TemplateValidation(msg) => (
                StatusCode::BAD_REQUEST,
                "template_validation_error",
                msg.as_str(),
            ),
            SampleServiceError::BarcodeGeneration(msg) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                "barcode_generation_error",
                msg.as_str(),
            ),
            SampleServiceError::BatchProcessing(msg) => (
                StatusCode::BAD_REQUEST,
                "batch_processing_error",
                msg.as_str(),
            ),
            SampleServiceError::ExternalService { .. } => {
                tracing::error!("External service error: {}", self);
                (
                    StatusCode::BAD_GATEWAY,
                    "external_service_error",
                    "External service unavailable",
                )
            }
            SampleServiceError::Authentication(msg) => (
                StatusCode::UNAUTHORIZED,
                "authentication_error",
                msg.as_str(),
            ),
            SampleServiceError::Authorization(msg) => {
                (StatusCode::FORBIDDEN, "authorization_error", msg.as_str())
            }
            SampleServiceError::Configuration(msg) => {
                tracing::error!("Configuration error: {}", self);
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "configuration_error",
                    "Service configuration error",
                )
            }
            SampleServiceError::BusinessRule(msg) => (
                StatusCode::BAD_REQUEST,
                "business_rule_violation",
                msg.as_str(),
            ),
            SampleServiceError::ConcurrentModification => (
                StatusCode::CONFLICT,
                "concurrent_modification",
                "Resource was modified by another request",
            ),
            SampleServiceError::ResourceLimit(msg) => (
                StatusCode::TOO_MANY_REQUESTS,
                "resource_limit_exceeded",
                msg.as_str(),
            ),
            SampleServiceError::Internal(msg) => {
                tracing::error!("Internal error: {}", self);
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "internal_error",
                    "An internal error occurred",
                )
            }
        };

        let error_response = ErrorResponse::new(error_type, message);
        (status_code, Json(error_response)).into_response()
    }
}

/// Result type alias for sample service operations
pub type SampleResult<T> = Result<T, SampleServiceError>;

/// Helper function to create validation errors
pub fn validation_error(message: &str) -> SampleServiceError {
    SampleServiceError::Validation(message.to_string())
}

/// Helper function to create business rule errors
pub fn business_rule_error(message: &str) -> SampleServiceError {
    SampleServiceError::BusinessRule(message.to_string())
}

/// Helper function to create external service errors
pub fn external_service_error(service: &str, message: &str) -> SampleServiceError {
    SampleServiceError::ExternalService {
        service: service.to_string(),
        message: message.to_string(),
    }
}

/// Helper function to create barcode errors
pub fn barcode_error(message: &str) -> SampleServiceError {
    SampleServiceError::BarcodeGeneration(message.to_string())
}

/// Helper function to create workflow errors
pub fn workflow_error(current: &str, requested: &str) -> SampleServiceError {
    SampleServiceError::InvalidWorkflowTransition {
        current_status: current.to_string(),
        requested_status: requested.to_string(),
    }
}
