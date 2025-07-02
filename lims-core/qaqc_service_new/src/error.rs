use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde_json::json;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum QaqcError {
    #[error("Database error: {0}")]
    Database(#[from] sqlx::Error),

    #[error("Validation error: {0}")]
    Validation(#[from] validator::ValidationErrors),

    #[error("Authentication error: {0}")]
    Authentication(String),

    #[error("Authorization error: {0}")]
    Authorization(String),

    #[error("Not found: {0}")]
    NotFound(String),

    #[error("Conflict: {0}")]
    Conflict(String),

    #[error("Internal server error: {0}")]
    Internal(#[from] anyhow::Error),

    #[error("Service unavailable: {0}")]
    ServiceUnavailable(String),

    #[error("Workflow execution error: {0}")]
    WorkflowExecution(String),

    #[error("Quality check failed: {0}")]
    QualityCheckFailed(String),

    #[error("Compliance violation: {0}")]
    ComplianceViolation(String),

    #[error("Configuration error: {0}")]
    Configuration(String),

    #[error("External service error: {0}")]
    ExternalService(String),

    #[error("Timeout error: {0}")]
    Timeout(String),

    #[error("Data processing error: {0}")]
    DataProcessing(String),
}

impl IntoResponse for QaqcError {
    fn into_response(self) -> Response {
        let (status, error_message, error_code) = match self {
            QaqcError::Database(e) => {
                tracing::error!("Database error: {}", e);
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "Database error occurred",
                    "DATABASE_ERROR",
                )
            }
            QaqcError::Validation(e) => {
                tracing::warn!("Validation error: {}", e);
                (
                    StatusCode::BAD_REQUEST,
                    "Validation failed",
                    "VALIDATION_ERROR",
                )
            }
            QaqcError::Authentication(msg) => {
                tracing::warn!("Authentication error: {}", msg);
                (
                    StatusCode::UNAUTHORIZED,
                    "Authentication required",
                    "AUTH_ERROR",
                )
            }
            QaqcError::Authorization(msg) => {
                tracing::warn!("Authorization error: {}", msg);
                (
                    StatusCode::FORBIDDEN,
                    "Access denied",
                    "AUTHORIZATION_ERROR",
                )
            }
            QaqcError::NotFound(msg) => {
                tracing::info!("Not found: {}", msg);
                (StatusCode::NOT_FOUND, "Resource not found", "NOT_FOUND")
            }
            QaqcError::Conflict(msg) => {
                tracing::warn!("Conflict: {}", msg);
                (StatusCode::CONFLICT, "Resource conflict", "CONFLICT")
            }
            QaqcError::Internal(e) => {
                tracing::error!("Internal error: {}", e);
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "Internal server error",
                    "INTERNAL_ERROR",
                )
            }
            QaqcError::ServiceUnavailable(msg) => {
                tracing::error!("Service unavailable: {}", msg);
                (
                    StatusCode::SERVICE_UNAVAILABLE,
                    "Service temporarily unavailable",
                    "SERVICE_UNAVAILABLE",
                )
            }
            QaqcError::WorkflowExecution(msg) => {
                tracing::error!("Workflow execution error: {}", msg);
                (
                    StatusCode::UNPROCESSABLE_ENTITY,
                    "Workflow execution failed",
                    "WORKFLOW_ERROR",
                )
            }
            QaqcError::QualityCheckFailed(msg) => {
                tracing::warn!("Quality check failed: {}", msg);
                (
                    StatusCode::UNPROCESSABLE_ENTITY,
                    "Quality check failed",
                    "QUALITY_CHECK_FAILED",
                )
            }
            QaqcError::ComplianceViolation(msg) => {
                tracing::warn!("Compliance violation: {}", msg);
                (
                    StatusCode::UNPROCESSABLE_ENTITY,
                    "Compliance violation",
                    "COMPLIANCE_VIOLATION",
                )
            }
            QaqcError::Configuration(msg) => {
                tracing::error!("Configuration error: {}", msg);
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "Configuration error",
                    "CONFIG_ERROR",
                )
            }
            QaqcError::ExternalService(msg) => {
                tracing::error!("External service error: {}", msg);
                (
                    StatusCode::BAD_GATEWAY,
                    "External service error",
                    "EXTERNAL_SERVICE_ERROR",
                )
            }
            QaqcError::Timeout(msg) => {
                tracing::warn!("Timeout error: {}", msg);
                (StatusCode::REQUEST_TIMEOUT, "Request timeout", "TIMEOUT")
            }
            QaqcError::DataProcessing(msg) => {
                tracing::error!("Data processing error: {}", msg);
                (
                    StatusCode::UNPROCESSABLE_ENTITY,
                    "Data processing error",
                    "DATA_PROCESSING_ERROR",
                )
            }
        };

        let body = Json(json!({
            "error": {
                "code": error_code,
                "message": error_message,
                "status": status.as_u16()
            }
        }));

        (status, body).into_response()
    }
}

pub type Result<T> = std::result::Result<T, QaqcError>;

// Helper functions for common error cases
impl QaqcError {
    pub fn not_found(resource: &str, id: &str) -> Self {
        Self::NotFound(format!("{} with id '{}' not found", resource, id))
    }

    pub fn already_exists(resource: &str, identifier: &str) -> Self {
        Self::Conflict(format!(
            "{} with identifier '{}' already exists",
            resource, identifier
        ))
    }

    pub fn unauthorized_action(action: &str) -> Self {
        Self::Authorization(format!("Not authorized to perform action: {}", action))
    }

    pub fn invalid_workflow_state(
        workflow_id: &str,
        current_state: &str,
        required_state: &str,
    ) -> Self {
        Self::WorkflowExecution(format!(
            "Workflow {} is in state '{}', but operation requires state '{}'",
            workflow_id, current_state, required_state
        ))
    }

    pub fn quality_threshold_violation(metric: &str, value: f64, threshold: f64) -> Self {
        Self::QualityCheckFailed(format!(
            "Quality metric '{}' value {} violates threshold {}",
            metric, value, threshold
        ))
    }

    pub fn compliance_rule_violation(rule: &str, standard: &str) -> Self {
        Self::ComplianceViolation(format!(
            "Compliance rule '{}' violated for standard '{}'",
            rule, standard
        ))
    }

    pub fn external_service_timeout(service: &str) -> Self {
        Self::Timeout(format!(
            "Timeout while communicating with service: {}",
            service
        ))
    }

    pub fn invalid_configuration(setting: &str, value: &str) -> Self {
        Self::Configuration(format!(
            "Invalid configuration for '{}': {}",
            setting, value
        ))
    }
}
