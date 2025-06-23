use axum::{http::StatusCode, response::IntoResponse, Json};
use serde_json::json;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum StorageError {
    #[error("Database error: {0}")]
    Database(#[from] sqlx::Error),

    #[error("Validation error: {0}")]
    Validation(String),

    #[error("Storage location not found: {0}")]
    LocationNotFound(String),

    #[error("Sample not found: {0}")]
    SampleNotFound(String),

    #[error("Storage capacity exceeded")]
    CapacityExceeded,

    #[error("Temperature zone violation: {0}")]
    TemperatureViolation(String),

    #[error("IoT sensor error: {0}")]
    IoTSensorError(String),

    #[error("Analytics model error: {0}")]
    AnalyticsError(String),

    #[error("Digital twin simulation error: {0}")]
    DigitalTwinError(String),

    #[error("Blockchain error: {0}")]
    BlockchainError(String),

    #[error("Automation system error: {0}")]
    AutomationError(String),

    #[error("Energy optimization error: {0}")]
    EnergyError(String),

    #[error("Mobile integration error: {0}")]
    MobileError(String),

    #[error("Compliance violation: {0}")]
    ComplianceViolation(String),

    #[error("Authorization failed: {0}")]
    Unauthorized(String),

    #[error("Access forbidden: {0}")]
    Forbidden(String),

    #[error("Configuration error: {0}")]
    Config(String),

    #[error("External service error: {0}")]
    ExternalService(String),

    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),

    #[error("HTTP client error: {0}")]
    HttpClient(#[from] reqwest::Error),

    #[error("Internal server error: {0}")]
    Internal(String),

    #[error("Alert not found: {0}")]
    AlertNotFound(String),

    #[error("Sensor not found: {0}")]
    SensorNotFound(String),
}

impl IntoResponse for StorageError {
    fn into_response(self) -> axum::response::Response {
        let (status, error_message) = match self {
            StorageError::Database(ref e) => {
                tracing::error!("Database error: {}", e);
                (StatusCode::INTERNAL_SERVER_ERROR, "Database error occurred")
            }
            StorageError::Validation(ref msg) => (StatusCode::BAD_REQUEST, msg.as_str()),
            StorageError::LocationNotFound(ref id) => {
                (StatusCode::NOT_FOUND, "Storage location not found")
            }
            StorageError::SampleNotFound(ref id) => (StatusCode::NOT_FOUND, "Sample not found"),
            StorageError::CapacityExceeded => (StatusCode::CONFLICT, "Storage capacity exceeded"),
            StorageError::TemperatureViolation(ref msg) => (StatusCode::BAD_REQUEST, msg.as_str()),
            StorageError::IoTSensorError(ref msg) => {
                tracing::error!("IoT sensor error: {}", msg);
                (StatusCode::SERVICE_UNAVAILABLE, "IoT sensor error")
            }
            StorageError::AnalyticsError(ref msg) => {
                tracing::error!("Analytics error: {}", msg);
                (StatusCode::INTERNAL_SERVER_ERROR, "Analytics error")
            }
            StorageError::DigitalTwinError(ref msg) => {
                tracing::error!("Digital twin error: {}", msg);
                (StatusCode::INTERNAL_SERVER_ERROR, "Digital twin error")
            }
            StorageError::BlockchainError(ref msg) => {
                tracing::error!("Blockchain error: {}", msg);
                (StatusCode::INTERNAL_SERVER_ERROR, "Blockchain error")
            }
            StorageError::AutomationError(ref msg) => {
                tracing::error!("Automation error: {}", msg);
                (StatusCode::SERVICE_UNAVAILABLE, "Automation system error")
            }
            StorageError::EnergyError(ref msg) => {
                tracing::error!("Energy optimization error: {}", msg);
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "Energy optimization error",
                )
            }
            StorageError::MobileError(ref msg) => {
                tracing::error!("Mobile integration error: {}", msg);
                (StatusCode::BAD_REQUEST, "Mobile integration error")
            }
            StorageError::ComplianceViolation(ref msg) => (StatusCode::FORBIDDEN, msg.as_str()),
            StorageError::Unauthorized(ref msg) => (StatusCode::UNAUTHORIZED, msg.as_str()),
            StorageError::Forbidden(ref msg) => (StatusCode::FORBIDDEN, msg.as_str()),
            StorageError::Config(ref msg) => {
                tracing::error!("Configuration error: {}", msg);
                (StatusCode::INTERNAL_SERVER_ERROR, "Configuration error")
            }
            StorageError::ExternalService(ref msg) => {
                tracing::error!("External service error: {}", msg);
                (StatusCode::SERVICE_UNAVAILABLE, "External service error")
            }
            StorageError::Serialization(ref e) => {
                tracing::error!("Serialization error: {}", e);
                (StatusCode::INTERNAL_SERVER_ERROR, "Serialization error")
            }
            StorageError::HttpClient(ref e) => {
                tracing::error!("HTTP client error: {}", e);
                (StatusCode::INTERNAL_SERVER_ERROR, "HTTP client error")
            }
            StorageError::Internal(ref msg) => {
                tracing::error!("Internal error: {}", msg);
                (StatusCode::INTERNAL_SERVER_ERROR, "Internal server error")
            }
            StorageError::AlertNotFound(ref _id) => {
                (StatusCode::NOT_FOUND, "Alert not found")
            }
            StorageError::SensorNotFound(ref _id) => {
                (StatusCode::NOT_FOUND, "Sensor not found")
            }
        };

        let body = Json(json!({
            "error": error_message,
            "timestamp": chrono::Utc::now().to_rfc3339(),
        }));

        (status, body).into_response()
    }
}

// Result type alias for convenience
pub type StorageResult<T> = Result<T, StorageError>;
