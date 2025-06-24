use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::Json;
use serde_json::json;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum SequencingError {
    #[error("Database error: {0}")]
    Database(#[from] sqlx::Error),

    #[error("Configuration error: {0}")]
    Configuration(String),

    #[error("Validation error: {0}")]
    Validation(String),

    #[error("Authentication error: {0}")]
    Authentication(String),

    #[error("Authorization error: {0}")]
    Authorization(String),

    #[error("Job not found: {0}")]
    JobNotFound(String),

    #[error("Run not found: {0}")]
    RunNotFound(String),

    #[error("Workflow not found: {0}")]
    WorkflowNotFound(String),

    #[error("Sample sheet not found: {0}")]
    SampleSheetNotFound(String),

    #[error("Platform not found")]
    PlatformNotFound(String),

    #[error("Analysis not found: {0}")]
    AnalysisNotFound { analysis_id: String },

    #[error("Quality metrics not found: {0}")]
    QualityMetricsNotFound { analysis_id: String },

    #[error("Sample sheet in use: {0}")]
    SampleSheetInUse { sheet_id: String },

    #[error("Export error: {0}")]
    ExportError { message: String },

    #[error("Invalid job state: {0}")]
    InvalidJobState { current_state: String, expected: String },

    #[error("Workflow in use: {0}")]
    WorkflowInUse { workflow_id: String },

    #[error("Workflow validation error: {0}")]
    WorkflowValidation { message: String },

    #[error("Execution not found: {0}")]
    ExecutionNotFound { execution_id: String },

    #[error("Step not found: {0}")]
    StepNotFound { step_id: String, execution_id: String },

    #[error("Integration error: {0}")]
    IntegrationError { service: String, message: String },

    #[error("Webhook not found")]
    WebhookNotFound { webhook_id: String },

    #[error("Invalid job status transition: from {from} to {to}")]
    InvalidStatusTransition { from: String, to: String },

    #[error("Resource conflict: {0}")]
    ResourceConflict(String),

    #[error("Capacity exceeded: {0}")]
    CapacityExceeded(String),

    #[error("Quality control failure: {0}")]
    QualityControlFailure(String),

    #[error("External service error: {service}: {message}")]
    ExternalService { service: String, message: String },

    #[error("Scheduling conflict: {0}")]
    SchedulingConflict(String),

    #[error("Invalid operation: {0}")]
    InvalidOperation(String),

    #[error("Service unavailable: {0}")]
    ServiceUnavailable(String),

    #[error("Not implemented: {0}")]
    NotImplemented(String),

    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),

    #[error("HTTP client error: {0}")]
    HttpClient(#[from] reqwest::Error),

    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Internal server error: {0}")]
    Internal(String),
}

impl SequencingError {
    pub fn status_code(&self) -> StatusCode {
        match self {
            SequencingError::Database(_) => StatusCode::INTERNAL_SERVER_ERROR,
            SequencingError::Configuration(_) => StatusCode::INTERNAL_SERVER_ERROR,
            SequencingError::Validation(_) => StatusCode::BAD_REQUEST,
            SequencingError::Authentication(_) => StatusCode::UNAUTHORIZED,
            SequencingError::Authorization(_) => StatusCode::FORBIDDEN,
            SequencingError::JobNotFound(_) => StatusCode::NOT_FOUND,
            SequencingError::RunNotFound(_) => StatusCode::NOT_FOUND,
            SequencingError::WorkflowNotFound(_) => StatusCode::NOT_FOUND,
            SequencingError::SampleSheetNotFound(_) => StatusCode::NOT_FOUND,
            SequencingError::PlatformNotFound(_) => StatusCode::NOT_FOUND,
            SequencingError::AnalysisNotFound { .. } => StatusCode::NOT_FOUND,
            SequencingError::QualityMetricsNotFound { .. } => StatusCode::NOT_FOUND,
            SequencingError::SampleSheetInUse { .. } => StatusCode::CONFLICT,
            SequencingError::ExportError { .. } => StatusCode::INTERNAL_SERVER_ERROR,
            SequencingError::InvalidJobState { .. } => StatusCode::BAD_REQUEST,
            SequencingError::WorkflowInUse { .. } => StatusCode::CONFLICT,
            SequencingError::WorkflowValidation { .. } => StatusCode::BAD_REQUEST,
            SequencingError::ExecutionNotFound { .. } => StatusCode::NOT_FOUND,
            SequencingError::StepNotFound { .. } => StatusCode::NOT_FOUND,
            SequencingError::IntegrationError { .. } => StatusCode::BAD_GATEWAY,
            SequencingError::WebhookNotFound { .. } => StatusCode::NOT_FOUND,
            SequencingError::InvalidStatusTransition { .. } => StatusCode::BAD_REQUEST,
            SequencingError::ResourceConflict(_) => StatusCode::CONFLICT,
            SequencingError::CapacityExceeded(_) => StatusCode::SERVICE_UNAVAILABLE,
            SequencingError::QualityControlFailure(_) => StatusCode::UNPROCESSABLE_ENTITY,
            SequencingError::ExternalService { .. } => StatusCode::BAD_GATEWAY,
            SequencingError::SchedulingConflict(_) => StatusCode::CONFLICT,
            SequencingError::InvalidOperation(_) => StatusCode::BAD_REQUEST,
            SequencingError::ServiceUnavailable(_) => StatusCode::SERVICE_UNAVAILABLE,
            SequencingError::NotImplemented(_) => StatusCode::NOT_IMPLEMENTED,
            SequencingError::Serialization(_) => StatusCode::INTERNAL_SERVER_ERROR,
            SequencingError::HttpClient(_) => StatusCode::BAD_GATEWAY,
            SequencingError::Io(_) => StatusCode::INTERNAL_SERVER_ERROR,
            SequencingError::Internal(_) => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }
}

impl IntoResponse for SequencingError {
    fn into_response(self) -> Response {
        let status = self.status_code();
        let message = self.to_string();

        let body = Json(json!({
            "error": {
                "message": message,
                "timestamp": chrono::Utc::now(),
            }
        }));

        (status, body).into_response()
    }
}

pub type Result<T> = std::result::Result<T, SequencingError>;
