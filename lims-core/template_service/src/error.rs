use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde::{Deserialize, Serialize};
use thiserror::Error;

/// Template service error types
#[derive(Error, Debug)]
pub enum TemplateServiceError {
    #[error("Database error: {0}")]
    Database(#[from] sqlx::Error),

    #[error("Validation error: {0}")]
    Validation(String),

    #[error("Template not found: {template_id}")]
    TemplateNotFound { template_id: String },

    #[error("Field not found: {field_id}")]
    FieldNotFound { field_id: String },

    #[error("Validation rule not found: {rule_id}")]
    ValidationRuleNotFound { rule_id: String },

    #[error("Template version not found: {template_id} version {version}")]
    TemplateVersionNotFound {
        template_id: String,
        version: String,
    },

    #[error("Duplicate template name: {name}")]
    DuplicateTemplateName { name: String },

    #[error("Invalid template status transition from {current_status} to {requested_status}")]
    InvalidStatusTransition {
        current_status: String,
        requested_status: String,
    },

    #[error("Template field validation failed: {field_name} - {message}")]
    FieldValidationFailed { field_name: String, message: String },

    #[error("Form validation failed: {0}")]
    FormValidation(String),

    #[error("Template parsing error: {0}")]
    TemplateParsing(String),

    #[error("File processing error: {0}")]
    FileProcessing(String),

    #[error("File upload error: {0}")]
    FileUpload(String),

    #[error("File not found: {file_path}")]
    FileNotFound { file_path: String },

    #[error("Invalid file format: {format} - expected {expected}")]
    InvalidFileFormat { format: String, expected: String },

    #[error("File too large: {size_mb}MB - maximum allowed {max_mb}MB")]
    FileTooLarge { size_mb: u64, max_mb: u64 },

    #[error("Template version conflict: {template_id} - version {version} already exists")]
    VersionConflict {
        template_id: String,
        version: String,
    },

    #[error("Maximum versions exceeded for template: {template_id} - limit {limit}")]
    MaxVersionsExceeded { template_id: String, limit: usize },

    #[error("Template dependency error: {0}")]
    DependencyError(String),

    #[error("Form generation error: {0}")]
    FormGeneration(String),

    #[error("Template compilation error: {0}")]
    TemplateCompilation(String),

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

    #[error("Concurrent modification detected for template: {template_id}")]
    ConcurrentModification { template_id: String },

    #[error("Resource limit exceeded: {0}")]
    ResourceLimit(String),

    #[error("Template feature not enabled: {feature}")]
    FeatureNotEnabled { feature: String },

    #[error("Template quota exceeded for user: {user_id} - limit {limit}")]
    TemplateQuotaExceeded { user_id: String, limit: usize },

    #[error("Field limit exceeded for template: {template_id} - limit {limit}")]
    FieldLimitExceeded { template_id: String, limit: usize },

    #[error("Validation rule limit exceeded for field: {field_id} - limit {limit}")]
    ValidationRuleLimitExceeded { field_id: String, limit: usize },

    #[error("Invalid field type transition: {field_id} from {current_type} to {new_type}")]
    InvalidFieldTypeTransition {
        field_id: String,
        current_type: String,
        new_type: String,
    },

    #[error("Circular dependency detected in template fields")]
    CircularDependency,

    #[error("Template schema validation failed: {0}")]
    SchemaValidation(String),

    #[error("Template rendering error: {0}")]
    TemplateRendering(String),

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
    pub field_errors: Option<std::collections::HashMap<String, Vec<String>>>,
}

impl ErrorResponse {
    pub fn new(error: &str, message: &str) -> Self {
        Self {
            error: error.to_string(),
            message: message.to_string(),
            details: None,
            timestamp: chrono::Utc::now(),
            request_id: None,
            field_errors: None,
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

    pub fn with_field_errors(
        mut self,
        field_errors: std::collections::HashMap<String, Vec<String>>,
    ) -> Self {
        self.field_errors = Some(field_errors);
        self
    }
}

/// Convert validation errors to template service errors
impl From<validator::ValidationErrors> for TemplateServiceError {
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

        TemplateServiceError::Validation(error_messages.join(", "))
    }
}

/// Convert anyhow errors to template service errors
impl From<anyhow::Error> for TemplateServiceError {
    fn from(error: anyhow::Error) -> Self {
        TemplateServiceError::Internal(error.to_string())
    }
}

/// Convert JSON errors to template service errors
impl From<serde_json::Error> for TemplateServiceError {
    fn from(error: serde_json::Error) -> Self {
        TemplateServiceError::Validation(format!("JSON parsing error: {}", error))
    }
}

/// Convert HTTP client errors to template service errors
impl From<reqwest::Error> for TemplateServiceError {
    fn from(error: reqwest::Error) -> Self {
        TemplateServiceError::ExternalService {
            service: "HTTP Client".to_string(),
            message: error.to_string(),
        }
    }
}

/// Convert IO errors to template service errors
impl From<std::io::Error> for TemplateServiceError {
    fn from(error: std::io::Error) -> Self {
        TemplateServiceError::FileProcessing(format!("IO error: {}", error))
    }
}

/// Implement IntoResponse for TemplateServiceError
impl IntoResponse for TemplateServiceError {
    fn into_response(self) -> Response {
        let (status_code, error_type, message) = match &self {
            TemplateServiceError::Database(_) => {
                tracing::error!("Database error: {}", self);
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "database_error",
                    "A database error occurred",
                )
            }
            TemplateServiceError::Validation(msg) => {
                (StatusCode::BAD_REQUEST, "validation_error", msg.as_str())
            }
            TemplateServiceError::TemplateNotFound { .. } => (
                StatusCode::NOT_FOUND,
                "template_not_found",
                "Template not found",
            ),
            TemplateServiceError::FieldNotFound { .. } => {
                (StatusCode::NOT_FOUND, "field_not_found", "Field not found")
            }
            TemplateServiceError::ValidationRuleNotFound { .. } => (
                StatusCode::NOT_FOUND,
                "validation_rule_not_found",
                "Error message",
            ),
            TemplateServiceError::TemplateVersionNotFound { .. } => (
                StatusCode::NOT_FOUND,
                "template_version_not_found",
                "Error message",
            ),
            TemplateServiceError::DuplicateTemplateName { .. } => (
                StatusCode::CONFLICT,
                "duplicate_template_name",
                "Error message",
            ),
            TemplateServiceError::InvalidStatusTransition { .. } => (
                StatusCode::BAD_REQUEST,
                "invalid_status_transition",
                "Error message",
            ),
            TemplateServiceError::FieldValidationFailed { .. } => (
                StatusCode::BAD_REQUEST,
                "field_validation_failed",
                "Error message",
            ),
            TemplateServiceError::FormValidation(msg) => (
                StatusCode::BAD_REQUEST,
                "form_validation_error",
                msg.as_str(),
            ),
            TemplateServiceError::TemplateParsing(msg) => (
                StatusCode::BAD_REQUEST,
                "template_parsing_error",
                msg.as_str(),
            ),
            TemplateServiceError::FileProcessing(msg) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                "file_processing_error",
                msg.as_str(),
            ),
            TemplateServiceError::FileUpload(msg) => {
                (StatusCode::BAD_REQUEST, "file_upload_error", msg.as_str())
            }
            TemplateServiceError::FileNotFound { .. } => {
                (StatusCode::NOT_FOUND, "file_not_found", "Error message")
            }
            TemplateServiceError::InvalidFileFormat { .. } => (
                StatusCode::BAD_REQUEST,
                "invalid_file_format",
                "Error message",
            ),
            TemplateServiceError::FileTooLarge { .. } => (
                StatusCode::PAYLOAD_TOO_LARGE,
                "file_too_large",
                "Error message",
            ),
            TemplateServiceError::VersionConflict { .. } => {
                (StatusCode::CONFLICT, "version_conflict", "Error message")
            }
            TemplateServiceError::MaxVersionsExceeded { .. } => (
                StatusCode::BAD_REQUEST,
                "max_versions_exceeded",
                "Error message",
            ),
            TemplateServiceError::DependencyError(msg) => {
                (StatusCode::BAD_REQUEST, "dependency_error", msg.as_str())
            }
            TemplateServiceError::FormGeneration(msg) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                "form_generation_error",
                msg.as_str(),
            ),
            TemplateServiceError::TemplateCompilation(msg) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                "template_compilation_error",
                msg.as_str(),
            ),
            TemplateServiceError::ExternalService { .. } => {
                tracing::error!("External service error: {}", self);
                (
                    StatusCode::BAD_GATEWAY,
                    "external_service_error",
                    "External service unavailable",
                )
            }
            TemplateServiceError::Authentication(msg) => (
                StatusCode::UNAUTHORIZED,
                "authentication_error",
                msg.as_str(),
            ),
            TemplateServiceError::Authorization(msg) => {
                (StatusCode::FORBIDDEN, "authorization_error", msg.as_str())
            }
            TemplateServiceError::Configuration(msg) => {
                tracing::error!("Configuration error: {}", self);
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "configuration_error",
                    "Service configuration error",
                )
            }
            TemplateServiceError::BusinessRule(msg) => (
                StatusCode::BAD_REQUEST,
                "business_rule_violation",
                msg.as_str(),
            ),
            TemplateServiceError::ConcurrentModification { .. } => (
                StatusCode::CONFLICT,
                "concurrent_modification",
                "Error message",
            ),
            TemplateServiceError::ResourceLimit(msg) => (
                StatusCode::TOO_MANY_REQUESTS,
                "resource_limit_exceeded",
                msg.as_str(),
            ),
            TemplateServiceError::FeatureNotEnabled { .. } => (
                StatusCode::FORBIDDEN,
                "feature_not_enabled",
                "Error message",
            ),
            TemplateServiceError::TemplateQuotaExceeded { .. } => (
                StatusCode::TOO_MANY_REQUESTS,
                "template_quota_exceeded",
                "Error message",
            ),
            TemplateServiceError::FieldLimitExceeded { .. } => (
                StatusCode::BAD_REQUEST,
                "field_limit_exceeded",
                "Error message",
            ),
            TemplateServiceError::ValidationRuleLimitExceeded { .. } => (
                StatusCode::BAD_REQUEST,
                "validation_rule_limit_exceeded",
                "Error message",
            ),
            TemplateServiceError::InvalidFieldTypeTransition { .. } => (
                StatusCode::BAD_REQUEST,
                "invalid_field_type_transition",
                "Error message",
            ),
            TemplateServiceError::CircularDependency => (
                StatusCode::BAD_REQUEST,
                "circular_dependency",
                "Circular dependency detected in template fields",
            ),
            TemplateServiceError::SchemaValidation(msg) => (
                StatusCode::BAD_REQUEST,
                "schema_validation_error",
                msg.as_str(),
            ),
            TemplateServiceError::TemplateRendering(msg) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                "template_rendering_error",
                msg.as_str(),
            ),
            TemplateServiceError::Internal(msg) => {
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

/// Result type alias for template service operations
pub type TemplateResult<T> = Result<T, TemplateServiceError>;

/// Helper function to create validation errors
pub fn validation_error(message: &str) -> TemplateServiceError {
    TemplateServiceError::Validation(message.to_string())
}

/// Helper function to create business rule errors
pub fn business_rule_error(message: &str) -> TemplateServiceError {
    TemplateServiceError::BusinessRule(message.to_string())
}

/// Helper function to create external service errors
pub fn external_service_error(service: &str, message: &str) -> TemplateServiceError {
    TemplateServiceError::ExternalService {
        service: service.to_string(),
        message: message.to_string(),
    }
}

/// Helper function to create template not found errors
pub fn template_not_found_error(template_id: &str) -> TemplateServiceError {
    TemplateServiceError::TemplateNotFound {
        template_id: template_id.to_string(),
    }
}

/// Helper function to create field not found errors
pub fn field_not_found_error(field_id: &str) -> TemplateServiceError {
    TemplateServiceError::FieldNotFound {
        field_id: field_id.to_string(),
    }
}

/// Helper function to create file processing errors
pub fn file_processing_error(message: &str) -> TemplateServiceError {
    TemplateServiceError::FileProcessing(message.to_string())
}

/// Helper function to create form validation errors
pub fn form_validation_error(message: &str) -> TemplateServiceError {
    TemplateServiceError::FormValidation(message.to_string())
}

/// Helper function to create feature not enabled errors
pub fn feature_not_enabled_error(feature: &str) -> TemplateServiceError {
    TemplateServiceError::FeatureNotEnabled {
        feature: feature.to_string(),
    }
}
