use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde_json::json;
use thiserror::Error;

/// Result type alias for barcode service operations
pub type Result<T> = std::result::Result<T, BarcodeError>;

/// Barcode service error types
#[derive(Error, Debug)]
pub enum BarcodeError {
    #[error("Database error: {0}")]
    DatabaseError(#[from] sqlx::Error),

    #[error("Configuration error: {0}")]
    ConfigurationError(String),

    #[error("Validation error: {0}")]
    ValidationError(String),

    #[error("Barcode generation failed: {0}")]
    GenerationFailed(String),

    #[error("Barcode not found: {0}")]
    BarcodeNotFound(String),

    #[error("Barcode already reserved: {0}")]
    BarcodeAlreadyReserved(String),

    #[error("Barcode not reserved: {0}")]
    BarcodeNotReserved(String),

    #[error("Invalid request: {0}")]
    InvalidRequest(String),

    #[error("Internal server error: {0}")]
    InternalError(String),
}

impl IntoResponse for BarcodeError {
    fn into_response(self) -> Response {
        let (status, error_message) = match self {
            BarcodeError::ValidationError(msg) => (StatusCode::BAD_REQUEST, msg),
            BarcodeError::BarcodeNotFound(msg) => (StatusCode::NOT_FOUND, msg),
            BarcodeError::BarcodeAlreadyReserved(msg) => (StatusCode::CONFLICT, msg),
            BarcodeError::BarcodeNotReserved(msg) => (StatusCode::CONFLICT, msg),
            BarcodeError::InvalidRequest(msg) => (StatusCode::BAD_REQUEST, msg),
            BarcodeError::GenerationFailed(msg) => (StatusCode::INTERNAL_SERVER_ERROR, msg),
            BarcodeError::ConfigurationError(msg) => (StatusCode::INTERNAL_SERVER_ERROR, msg),
            BarcodeError::DatabaseError(ref e) => {
                tracing::error!("Database error: {}", e);
                (StatusCode::INTERNAL_SERVER_ERROR, "Database error".to_string())
            }
            BarcodeError::InternalError(msg) => (StatusCode::INTERNAL_SERVER_ERROR, msg),
        };

        let body = Json(json!({
            "error": {
                "message": error_message,
                "type": self.error_type(),
                "timestamp": chrono::Utc::now(),
            }
        }));

        (status, body).into_response()
    }
}

impl BarcodeError {
    fn error_type(&self) -> &'static str {
        match self {
            BarcodeError::DatabaseError(_) => "database_error",
            BarcodeError::ConfigurationError(_) => "configuration_error",
            BarcodeError::ValidationError(_) => "validation_error",
            BarcodeError::GenerationFailed(_) => "generation_failed",
            BarcodeError::BarcodeNotFound(_) => "barcode_not_found",
            BarcodeError::BarcodeAlreadyReserved(_) => "barcode_already_reserved",
            BarcodeError::BarcodeNotReserved(_) => "barcode_not_reserved",
            BarcodeError::InvalidRequest(_) => "invalid_request",
            BarcodeError::InternalError(_) => "internal_error",
        }
    }
} 