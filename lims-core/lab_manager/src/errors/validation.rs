use crate::errors::{ComponentError, ErrorSeverity};

/// Validation-specific errors
#[derive(Debug, thiserror::Error)]
pub enum ValidationError {
    #[error("Required field missing: {field}")]
    RequiredFieldMissing { field: String },
    #[error("Invalid format for field: {field}")]
    InvalidFormat { field: String },
    #[error("Value out of range for field: {field}")]
    OutOfRange { field: String },
    #[error("Invalid length for field: {field}")]
    InvalidLength { field: String },
    #[error("Schema validation failed: {0}")]
    SchemaValidationFailed(String),
    #[error("Business rule violation: {0}")]
    BusinessRuleViolation(String),
    #[error("Duplicate value for field: {field}")]
    DuplicateValue { field: String },
}

impl ComponentError for ValidationError {
    fn error_code(&self) -> &'static str {
        match self {
            Self::RequiredFieldMissing { .. } => "VALIDATION_REQUIRED_FIELD_MISSING",
            Self::InvalidFormat { .. } => "VALIDATION_INVALID_FORMAT",
            Self::OutOfRange { .. } => "VALIDATION_OUT_OF_RANGE",
            Self::InvalidLength { .. } => "VALIDATION_INVALID_LENGTH",
            Self::SchemaValidationFailed(_) => "VALIDATION_SCHEMA_FAILED",
            Self::BusinessRuleViolation(_) => "VALIDATION_BUSINESS_RULE_VIOLATION",
            Self::DuplicateValue { .. } => "VALIDATION_DUPLICATE_VALUE",
        }
    }

    fn severity(&self) -> ErrorSeverity {
        match self {
            Self::RequiredFieldMissing { .. }
            | Self::InvalidFormat { .. }
            | Self::OutOfRange { .. }
            | Self::InvalidLength { .. }
            | Self::DuplicateValue { .. } => ErrorSeverity::Low,
            Self::SchemaValidationFailed(_) => ErrorSeverity::Medium,
            Self::BusinessRuleViolation(_) => ErrorSeverity::Medium,
        }
    }

    fn is_retryable(&self) -> bool {
        false // Validation errors are typically not retryable
    }
}
