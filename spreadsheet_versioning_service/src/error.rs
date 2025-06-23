use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ServiceError {
    #[error("Database error: {0}")]
    Database(#[from] sqlx::Error),

    #[error("Database connection error: {0}")]
    DatabaseConnection(String),

    #[error("Version not found: {version_id}")]
    VersionNotFound { version_id: String },

    #[error("Spreadsheet not found: {spreadsheet_id}")]
    SpreadsheetNotFound { spreadsheet_id: String },

    #[error("Conflict not found: {conflict_id}")]
    ConflictNotFound { conflict_id: String },

    #[error("Invalid version number: {version_number}")]
    InvalidVersionNumber { version_number: i32 },

    #[error("Version already exists: spreadsheet {spreadsheet_id} version {version_number}")]
    VersionAlreadyExists {
        spreadsheet_id: String,
        version_number: i32,
    },

    #[error("Maximum versions exceeded: {max_versions}")]
    MaxVersionsExceeded { max_versions: usize },

    #[error("Validation error: {0}")]
    Validation(String),

    #[error("File processing error: {0}")]
    FileProcessing(String),

    #[error("Diff generation error: {0}")]
    DiffGeneration(String),

    #[error("Merge conflict: {0}")]
    MergeConflict(String),

    #[error("Conflict resolution error: {0}")]
    ConflictResolution(String),

    #[error("Unauthorized: {0}")]
    Unauthorized(String),

    #[error("Internal server error: {0}")]
    Internal(String),

    #[error("Configuration error: {0}")]
    Configuration(String),
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ErrorResponse {
    pub error: String,
    pub message: String,
    pub details: Option<serde_json::Value>,
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

impl IntoResponse for ServiceError {
    fn into_response(self) -> Response {
        let (status, error_code, message) = match &self {
            ServiceError::Database(_) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                "database_error",
                "Database operation failed",
            ),
            ServiceError::DatabaseConnection(_) => (
                StatusCode::SERVICE_UNAVAILABLE,
                "database_connection_error",
                "Database connection failed",
            ),
            ServiceError::VersionNotFound { .. } => (
                StatusCode::NOT_FOUND,
                "version_not_found",
                "Version not found",
            ),
            ServiceError::SpreadsheetNotFound { .. } => (
                StatusCode::NOT_FOUND,
                "spreadsheet_not_found",
                "Spreadsheet not found",
            ),
            ServiceError::ConflictNotFound { .. } => (
                StatusCode::NOT_FOUND,
                "conflict_not_found",
                "Conflict not found",
            ),
            ServiceError::InvalidVersionNumber { .. } => (
                StatusCode::BAD_REQUEST,
                "invalid_version_number",
                "Invalid version number",
            ),
            ServiceError::VersionAlreadyExists { .. } => (
                StatusCode::CONFLICT,
                "version_already_exists",
                "Version already exists",
            ),
            ServiceError::MaxVersionsExceeded { .. } => (
                StatusCode::BAD_REQUEST,
                "max_versions_exceeded",
                "Maximum versions exceeded",
            ),
            ServiceError::Validation(_) => (
                StatusCode::BAD_REQUEST,
                "validation_error",
                "Validation failed",
            ),
            ServiceError::FileProcessing(_) => (
                StatusCode::BAD_REQUEST,
                "file_processing_error",
                "File processing failed",
            ),
            ServiceError::DiffGeneration(_) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                "diff_generation_error",
                "Diff generation failed",
            ),
            ServiceError::MergeConflict(_) => (
                StatusCode::CONFLICT,
                "merge_conflict",
                "Merge conflict detected",
            ),
            ServiceError::ConflictResolution(_) => (
                StatusCode::BAD_REQUEST,
                "conflict_resolution_error",
                "Conflict resolution failed",
            ),
            ServiceError::Unauthorized(_) => (
                StatusCode::UNAUTHORIZED,
                "unauthorized",
                "Unauthorized access",
            ),
            ServiceError::Internal(_) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                "internal_error",
                "Internal server error",
            ),
            ServiceError::Configuration(_) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                "configuration_error",
                "Configuration error",
            ),
        };

        let error_response = ErrorResponse {
            error: error_code.to_string(),
            message: message.to_string(),
            details: Some(serde_json::json!({
                "error_details": self.to_string()
            })),
            timestamp: chrono::Utc::now(),
        };

        (status, Json(error_response)).into_response()
    }
}

pub type ServiceResult<T> = Result<T, ServiceError>;
