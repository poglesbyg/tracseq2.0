//! Error types for Saga pattern operations.

use thiserror::Error;
use uuid::Uuid;

/// Comprehensive error types for saga operations
#[derive(Error, Debug, Clone)]
pub enum SagaError {
    /// Error during step execution
    #[error("Step '{step_name}' (index {step_index}) failed: {reason}")]
    StepExecutionFailed {
        step_name: String,
        step_index: usize,
        reason: String,
    },

    /// Error during compensation execution
    #[error("Compensation failed for saga {saga_id}: {errors:?}")]
    CompensationFailed { saga_id: Uuid, errors: Vec<String> },

    /// Timeout error
    #[error("Saga {saga_id} timed out after {timeout_ms}ms")]
    SagaTimeout { saga_id: Uuid, timeout_ms: u64 },

    /// Invalid saga state transition
    #[error("Invalid state transition from {from} to {to} for saga {saga_id}")]
    InvalidStateTransition {
        saga_id: Uuid,
        from: String,
        to: String,
    },

    /// Service communication error
    #[error("Service communication failed: {service}: {reason}")]
    ServiceCommunicationFailed { service: String, reason: String },

    /// Data consistency error
    #[error("Data consistency violation: {details}")]
    DataConsistencyViolation { details: String },

    /// Resource not found
    #[error("Resource not found: {resource_type} with id {resource_id}")]
    ResourceNotFound {
        resource_type: String,
        resource_id: String,
    },

    /// Validation error
    #[error("Validation failed: {field}: {message}")]
    ValidationFailed { field: String, message: String },

    /// Concurrent modification error
    #[error("Concurrent modification detected for {resource}")]
    ConcurrentModification { resource: String },

    /// Configuration error
    #[error("Configuration error: {message}")]
    ConfigurationError { message: String },

    /// Database error
    #[error("Database error: {operation}: {reason}")]
    DatabaseError { operation: String, reason: String },

    /// Network error
    #[error("Network error: {endpoint}: {reason}")]
    NetworkError { endpoint: String, reason: String },

    /// Serialization error
    #[error("Serialization error: {reason}")]
    SerializationError { reason: String },

    /// Generic error for catch-all cases
    #[error("Saga error: {message}")]
    Generic { message: String },
}

impl SagaError {
    /// Check if the error is retryable
    pub fn is_retryable(&self) -> bool {
        match self {
            SagaError::ServiceCommunicationFailed { .. } => true,
            SagaError::NetworkError { .. } => true,
            SagaError::DatabaseError { .. } => true,
            SagaError::SagaTimeout { .. } => false,
            SagaError::ValidationFailed { .. } => false,
            SagaError::DataConsistencyViolation { .. } => false,
            SagaError::ResourceNotFound { .. } => false,
            SagaError::ConcurrentModification { .. } => true,
            SagaError::ConfigurationError { .. } => false,
            SagaError::SerializationError { .. } => false,
            SagaError::InvalidStateTransition { .. } => false,
            SagaError::StepExecutionFailed { .. } => true,
            SagaError::CompensationFailed { .. } => false,
            SagaError::Generic { .. } => false,
        }
    }

    /// Get error severity level
    pub fn severity(&self) -> ErrorSeverity {
        match self {
            SagaError::SagaTimeout { .. } => ErrorSeverity::High,
            SagaError::CompensationFailed { .. } => ErrorSeverity::Critical,
            SagaError::DataConsistencyViolation { .. } => ErrorSeverity::Critical,
            SagaError::InvalidStateTransition { .. } => ErrorSeverity::High,
            SagaError::ServiceCommunicationFailed { .. } => ErrorSeverity::Medium,
            SagaError::NetworkError { .. } => ErrorSeverity::Medium,
            SagaError::DatabaseError { .. } => ErrorSeverity::High,
            SagaError::ValidationFailed { .. } => ErrorSeverity::Low,
            SagaError::ResourceNotFound { .. } => ErrorSeverity::Medium,
            SagaError::ConcurrentModification { .. } => ErrorSeverity::Medium,
            SagaError::ConfigurationError { .. } => ErrorSeverity::High,
            SagaError::SerializationError { .. } => ErrorSeverity::Low,
            SagaError::StepExecutionFailed { .. } => ErrorSeverity::Medium,
            SagaError::Generic { .. } => ErrorSeverity::Medium,
        }
    }

    /// Get error category for metrics and monitoring
    pub fn category(&self) -> &'static str {
        match self {
            SagaError::StepExecutionFailed { .. } => "step_execution",
            SagaError::CompensationFailed { .. } => "compensation",
            SagaError::SagaTimeout { .. } => "timeout",
            SagaError::InvalidStateTransition { .. } => "state_management",
            SagaError::ServiceCommunicationFailed { .. } => "service_communication",
            SagaError::DataConsistencyViolation { .. } => "data_consistency",
            SagaError::ResourceNotFound { .. } => "resource_management",
            SagaError::ValidationFailed { .. } => "validation",
            SagaError::ConcurrentModification { .. } => "concurrency",
            SagaError::ConfigurationError { .. } => "configuration",
            SagaError::DatabaseError { .. } => "database",
            SagaError::NetworkError { .. } => "network",
            SagaError::SerializationError { .. } => "serialization",
            SagaError::Generic { .. } => "generic",
        }
    }
}

/// Error severity levels for monitoring and alerting
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ErrorSeverity {
    Low,
    Medium,
    High,
    Critical,
}

impl ErrorSeverity {
    pub fn as_str(&self) -> &'static str {
        match self {
            ErrorSeverity::Low => "low",
            ErrorSeverity::Medium => "medium",
            ErrorSeverity::High => "high",
            ErrorSeverity::Critical => "critical",
        }
    }
}

/// Convert from common error types
impl From<sqlx::Error> for SagaError {
    fn from(err: sqlx::Error) -> Self {
        SagaError::DatabaseError {
            operation: "database_operation".to_string(),
            reason: err.to_string(),
        }
    }
}

impl From<reqwest::Error> for SagaError {
    fn from(err: reqwest::Error) -> Self {
        SagaError::NetworkError {
            endpoint: err.url().map(|u| u.to_string()).unwrap_or_default(),
            reason: err.to_string(),
        }
    }
}

impl From<serde_json::Error> for SagaError {
    fn from(err: serde_json::Error) -> Self {
        SagaError::SerializationError {
            reason: err.to_string(),
        }
    }
}

impl From<uuid::Error> for SagaError {
    fn from(err: uuid::Error) -> Self {
        SagaError::ValidationFailed {
            field: "uuid".to_string(),
            message: err.to_string(),
        }
    }
}

/// Helper macros for creating specific error types
#[macro_export]
macro_rules! saga_error {
    (step_failed: $step:expr, $index:expr, $reason:expr) => {
        SagaError::StepExecutionFailed {
            step_name: $step.to_string(),
            step_index: $index,
            reason: $reason.to_string(),
        }
    };

    (service_failed: $service:expr, $reason:expr) => {
        SagaError::ServiceCommunicationFailed {
            service: $service.to_string(),
            reason: $reason.to_string(),
        }
    };

    (validation_failed: $field:expr, $message:expr) => {
        SagaError::ValidationFailed {
            field: $field.to_string(),
            message: $message.to_string(),
        }
    };

    (not_found: $type:expr, $id:expr) => {
        SagaError::ResourceNotFound {
            resource_type: $type.to_string(),
            resource_id: $id.to_string(),
        }
    };
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_retryability() {
        let retryable_error = SagaError::ServiceCommunicationFailed {
            service: "test-service".to_string(),
            reason: "connection failed".to_string(),
        };
        assert!(retryable_error.is_retryable());

        let non_retryable_error = SagaError::ValidationFailed {
            field: "sample_id".to_string(),
            message: "invalid format".to_string(),
        };
        assert!(!non_retryable_error.is_retryable());
    }

    #[test]
    fn test_error_severity() {
        let critical_error = SagaError::CompensationFailed {
            saga_id: Uuid::new_v4(),
            errors: vec!["test error".to_string()],
        };
        assert_eq!(critical_error.severity(), ErrorSeverity::Critical);

        let low_error = SagaError::ValidationFailed {
            field: "test".to_string(),
            message: "test message".to_string(),
        };
        assert_eq!(low_error.severity(), ErrorSeverity::Low);
    }

    #[test]
    fn test_error_categories() {
        let step_error = SagaError::StepExecutionFailed {
            step_name: "test_step".to_string(),
            step_index: 0,
            reason: "test reason".to_string(),
        };
        assert_eq!(step_error.category(), "step_execution");

        let network_error = SagaError::NetworkError {
            endpoint: "http://test.com".to_string(),
            reason: "timeout".to_string(),
        };
        assert_eq!(network_error.category(), "network");
    }
}
