use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde_json::json;
use thiserror::Error;
use uuid::Uuid;

#[derive(Error, Debug)]
pub enum ServiceError {
    #[error("Database error: {0}")]
    Database(#[from] sqlx::Error),
    
    #[error("Library not found: {id}")]
    LibraryNotFound { id: Uuid },
    
    #[error("Protocol not found: {id}")]
    ProtocolNotFound { id: Uuid },
    
    #[error("Kit not found: {id}")]
    KitNotFound { id: Uuid },
    
    #[error("Platform not found: {id}")]
    PlatformNotFound { id: Uuid },
    
    #[error("Validation error: {message}")]
    Validation { message: String },
    
    #[error("Quality control failed: {reason}")]
    QualityControlFailed { reason: String },
    
    #[error("Protocol validation failed: {errors:?}")]
    ProtocolValidationFailed { errors: Vec<String> },
    
    #[error("Incompatible kit and platform: kit {kit_id}, platform {platform_id}")]
    IncompatibleKitPlatform { kit_id: Uuid, platform_id: Uuid },
    
    #[error("Insufficient library concentration: {current} ng/μL, required: {required} ng/μL")]
    InsufficientConcentration { current: f64, required: f64 },
    
    #[error("Invalid fragment size: {size} bp, expected range: {min}-{max} bp")]
    InvalidFragmentSize { size: i32, min: i32, max: i32 },
    
    #[error("Authentication failed: {message}")]
    Authentication { message: String },
    
    #[error("Authorization failed: {message}")]
    Authorization { message: String },
    
    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),
    
    #[error("UUID parsing error: {0}")]
    UuidParsing(#[from] uuid::Error),
    
    #[error("HTTP client error: {0}")]
    HttpClient(#[from] reqwest::Error),
    
    #[error("Configuration error: {message}")]
    Configuration { message: String },
    
    #[error("Internal server error: {message}")]
    Internal { message: String },
}

impl IntoResponse for ServiceError {
    fn into_response(self) -> Response {
        let (status_code, error_message, error_code) = match self {
            ServiceError::Database(ref e) => {
                tracing::error!("Database error: {}", e);
                (StatusCode::INTERNAL_SERVER_ERROR, "Internal server error".to_string(), "DATABASE_ERROR")
            }
            ServiceError::LibraryNotFound { id } => {
                (StatusCode::NOT_FOUND, format!("Library not found: {}", id), "LIBRARY_NOT_FOUND")
            }
            ServiceError::ProtocolNotFound { id } => {
                (StatusCode::NOT_FOUND, format!("Protocol not found: {}", id), "PROTOCOL_NOT_FOUND")
            }
            ServiceError::KitNotFound { id } => {
                (StatusCode::NOT_FOUND, format!("Kit not found: {}", id), "KIT_NOT_FOUND")
            }
            ServiceError::PlatformNotFound { id } => {
                (StatusCode::NOT_FOUND, format!("Platform not found: {}", id), "PLATFORM_NOT_FOUND")
            }
            ServiceError::Validation { ref message } => {
                (StatusCode::BAD_REQUEST, message.clone(), "VALIDATION_ERROR")
            }
            ServiceError::QualityControlFailed { ref reason } => {
                (StatusCode::UNPROCESSABLE_ENTITY, format!("Quality control failed: {}", reason), "QC_FAILED")
            }
            ServiceError::ProtocolValidationFailed { ref errors } => {
                (StatusCode::BAD_REQUEST, format!("Protocol validation failed: {}", errors.join(", ")), "PROTOCOL_VALIDATION_FAILED")
            }
            ServiceError::IncompatibleKitPlatform { kit_id, platform_id } => {
                (StatusCode::BAD_REQUEST, format!("Kit {} is not compatible with platform {}", kit_id, platform_id), "INCOMPATIBLE_KIT_PLATFORM")
            }
            ServiceError::InsufficientConcentration { current, required } => {
                (StatusCode::BAD_REQUEST, format!("Insufficient concentration: {} ng/μL, required: {} ng/μL", current, required), "INSUFFICIENT_CONCENTRATION")
            }
            ServiceError::InvalidFragmentSize { size, min, max } => {
                (StatusCode::BAD_REQUEST, format!("Invalid fragment size: {} bp, expected: {}-{} bp", size, min, max), "INVALID_FRAGMENT_SIZE")
            }
            ServiceError::Authentication { ref message } => {
                (StatusCode::UNAUTHORIZED, message.clone(), "AUTHENTICATION_FAILED")
            }
            ServiceError::Authorization { ref message } => {
                (StatusCode::FORBIDDEN, message.clone(), "AUTHORIZATION_FAILED")
            }
            ServiceError::Serialization(ref e) => {
                tracing::error!("Serialization error: {}", e);
                (StatusCode::INTERNAL_SERVER_ERROR, "Internal server error".to_string(), "SERIALIZATION_ERROR")
            }
            ServiceError::UuidParsing(ref e) => {
                (StatusCode::BAD_REQUEST, format!("Invalid UUID format: {}", e), "INVALID_UUID")
            }
            ServiceError::HttpClient(ref e) => {
                tracing::error!("HTTP client error: {}", e);
                (StatusCode::INTERNAL_SERVER_ERROR, "External service error".to_string(), "HTTP_CLIENT_ERROR")
            }
            ServiceError::Configuration { ref message } => {
                tracing::error!("Configuration error: {}", message);
                (StatusCode::INTERNAL_SERVER_ERROR, "Configuration error".to_string(), "CONFIGURATION_ERROR")
            }
            ServiceError::Internal { ref message } => {
                tracing::error!("Internal error: {}", message);
                (StatusCode::INTERNAL_SERVER_ERROR, message.clone(), "INTERNAL_ERROR")
            }
        };

        let body = Json(json!({
            "error": {
                "code": error_code,
                "message": error_message,
                "timestamp": chrono::Utc::now().to_rfc3339()
            }
        }));

        (status_code, body).into_response()
    }
}

pub type Result<T> = std::result::Result<T, ServiceError>;