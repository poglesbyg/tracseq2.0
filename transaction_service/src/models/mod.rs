//! Data models and structures for the transaction service.

pub use crate::coordinator::{CoordinatorStatistics, TransactionRequest, TransactionStatus};
pub use crate::saga::TransactionContext;

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Laboratory-specific transaction types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum LabTransactionType {
    /// Sample submission and processing workflow
    SampleSubmission,

    /// Sample validation and storage workflow
    SampleValidation,

    /// Sample sequencing workflow
    SampleSequencing,

    /// Sample retrieval and analysis workflow
    SampleRetrieval,

    /// Template creation and validation workflow
    TemplateManagement,

    /// Bulk sample operations
    BulkSampleOperation,

    /// Data export and reporting workflow
    DataExport,

    /// User management operations
    UserManagement,
}

impl std::fmt::Display for LabTransactionType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            LabTransactionType::SampleSubmission => write!(f, "sample_submission"),
            LabTransactionType::SampleValidation => write!(f, "sample_validation"),
            LabTransactionType::SampleSequencing => write!(f, "sample_sequencing"),
            LabTransactionType::SampleRetrieval => write!(f, "sample_retrieval"),
            LabTransactionType::TemplateManagement => write!(f, "template_management"),
            LabTransactionType::BulkSampleOperation => write!(f, "bulk_sample_operation"),
            LabTransactionType::DataExport => write!(f, "data_export"),
            LabTransactionType::UserManagement => write!(f, "user_management"),
        }
    }
}

/// Pre-built transaction workflows for common laboratory operations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LabTransactionWorkflow {
    /// Workflow identifier
    pub id: Uuid,

    /// Workflow name
    pub name: String,

    /// Workflow type
    pub transaction_type: LabTransactionType,

    /// Workflow description
    pub description: String,

    /// Expected execution time in milliseconds
    pub estimated_duration_ms: u64,

    /// Workflow version
    pub version: String,

    /// Whether this workflow is active
    pub is_active: bool,

    /// Required parameters for the workflow
    pub required_parameters: Vec<WorkflowParameter>,

    /// Workflow steps configuration
    pub steps_config: serde_json::Value,

    /// Created timestamp
    pub created_at: DateTime<Utc>,

    /// Last updated timestamp
    pub updated_at: DateTime<Utc>,
}

/// Workflow parameter definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkflowParameter {
    /// Parameter name
    pub name: String,

    /// Parameter type
    pub param_type: ParameterType,

    /// Whether parameter is required
    pub required: bool,

    /// Parameter description
    pub description: String,

    /// Default value if any
    pub default_value: Option<serde_json::Value>,

    /// Validation rules
    pub validation_rules: Vec<String>,
}

/// Parameter types supported in workflows
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ParameterType {
    String,
    Integer,
    Float,
    Boolean,
    Uuid,
    DateTime,
    Array(Box<ParameterType>),
    Object,
}

/// Transaction execution metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransactionMetrics {
    /// Transaction ID
    pub transaction_id: Uuid,

    /// Saga ID
    pub saga_id: Uuid,

    /// Transaction type
    pub transaction_type: String,

    /// User who initiated the transaction
    pub user_id: Option<Uuid>,

    /// Execution start time
    pub started_at: DateTime<Utc>,

    /// Execution end time
    pub completed_at: Option<DateTime<Utc>>,

    /// Total execution duration in milliseconds
    pub execution_duration_ms: u64,

    /// Number of steps executed
    pub steps_executed: u32,

    /// Number of failed steps
    pub steps_failed: u32,

    /// Number of retry attempts
    pub retry_attempts: u32,

    /// Whether compensation was executed
    pub compensation_executed: bool,

    /// Final transaction status
    pub final_status: String,

    /// Error information if failed
    pub error_details: Option<TransactionError>,

    /// Custom metrics data
    pub custom_metrics: serde_json::Value,
}

/// Transaction error details
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransactionError {
    /// Error code
    pub error_code: String,

    /// Error message
    pub error_message: String,

    /// Error category
    pub error_category: String,

    /// Failed step name
    pub failed_step: Option<String>,

    /// Service that caused the error
    pub failed_service: Option<String>,

    /// Whether the error is retryable
    pub is_retryable: bool,

    /// Stack trace or additional debug info
    pub debug_info: Option<String>,
}

/// Health check response for the transaction service
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransactionServiceHealth {
    /// Service status
    pub status: String,

    /// Current timestamp
    pub timestamp: DateTime<Utc>,

    /// Service version
    pub version: String,

    /// Uptime in seconds
    pub uptime_seconds: u64,

    /// Number of active transactions
    pub active_transactions: usize,

    /// Service dependencies status
    pub dependencies: std::collections::HashMap<String, DependencyHealth>,
}

/// Dependency health status
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DependencyHealth {
    /// Dependency name
    pub name: String,

    /// Health status
    pub status: String,

    /// Response time in milliseconds
    pub response_time_ms: u64,

    /// Last check timestamp
    pub last_checked: DateTime<Utc>,

    /// Error message if unhealthy
    pub error_message: Option<String>,
}

impl Default for TransactionServiceHealth {
    fn default() -> Self {
        Self {
            status: "healthy".to_string(),
            timestamp: Utc::now(),
            version: env!("CARGO_PKG_VERSION").to_string(),
            uptime_seconds: 0,
            active_transactions: 0,
            dependencies: std::collections::HashMap::new(),
        }
    }
}

impl LabTransactionWorkflow {
    /// Create a new workflow definition
    pub fn new(name: String, transaction_type: LabTransactionType, description: String) -> Self {
        Self {
            id: Uuid::new_v4(),
            name,
            transaction_type,
            description,
            estimated_duration_ms: 60000, // 1 minute default
            version: "1.0.0".to_string(),
            is_active: true,
            required_parameters: Vec::new(),
            steps_config: serde_json::json!({}),
            created_at: Utc::now(),
            updated_at: Utc::now(),
        }
    }

    /// Add a required parameter to the workflow
    pub fn add_parameter(mut self, parameter: WorkflowParameter) -> Self {
        self.required_parameters.push(parameter);
        self.updated_at = Utc::now();
        self
    }

    /// Set the steps configuration
    pub fn with_steps_config(mut self, config: serde_json::Value) -> Self {
        self.steps_config = config;
        self.updated_at = Utc::now();
        self
    }

    /// Set the estimated duration
    pub fn with_duration(mut self, duration_ms: u64) -> Self {
        self.estimated_duration_ms = duration_ms;
        self.updated_at = Utc::now();
        self
    }
}

impl WorkflowParameter {
    /// Create a new required string parameter
    pub fn required_string(name: String, description: String) -> Self {
        Self {
            name,
            param_type: ParameterType::String,
            required: true,
            description,
            default_value: None,
            validation_rules: Vec::new(),
        }
    }

    /// Create a new optional parameter with default value
    pub fn optional_with_default(
        name: String,
        param_type: ParameterType,
        description: String,
        default_value: serde_json::Value,
    ) -> Self {
        Self {
            name,
            param_type,
            required: false,
            description,
            default_value: Some(default_value),
            validation_rules: Vec::new(),
        }
    }

    /// Add validation rules
    pub fn with_validation(mut self, rules: Vec<String>) -> Self {
        self.validation_rules = rules;
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_lab_transaction_type_display() {
        assert_eq!(
            LabTransactionType::SampleSubmission.to_string(),
            "sample_submission"
        );
        assert_eq!(
            LabTransactionType::SampleSequencing.to_string(),
            "sample_sequencing"
        );
    }

    #[test]
    fn test_workflow_creation() {
        let workflow = LabTransactionWorkflow::new(
            "Test Workflow".to_string(),
            LabTransactionType::SampleSubmission,
            "A test workflow".to_string(),
        );

        assert_eq!(workflow.name, "Test Workflow");
        assert_eq!(workflow.version, "1.0.0");
        assert!(workflow.is_active);
        assert_eq!(workflow.required_parameters.len(), 0);
    }

    #[test]
    fn test_workflow_parameter_creation() {
        let param = WorkflowParameter::required_string(
            "sample_id".to_string(),
            "The sample identifier".to_string(),
        );

        assert_eq!(param.name, "sample_id");
        assert!(param.required);
        assert!(matches!(param.param_type, ParameterType::String));
    }

    #[test]
    fn test_workflow_with_parameters() {
        let param1 =
            WorkflowParameter::required_string("sample_id".to_string(), "Sample ID".to_string());

        let param2 = WorkflowParameter::optional_with_default(
            "priority".to_string(),
            ParameterType::Integer,
            "Processing priority".to_string(),
            serde_json::json!(1),
        );

        let workflow = LabTransactionWorkflow::new(
            "Sample Processing".to_string(),
            LabTransactionType::SampleSubmission,
            "Process a sample submission".to_string(),
        )
        .add_parameter(param1)
        .add_parameter(param2)
        .with_duration(120000);

        assert_eq!(workflow.required_parameters.len(), 2);
        assert_eq!(workflow.estimated_duration_ms, 120000);
    }

    #[test]
    fn test_transaction_service_health_default() {
        let health = TransactionServiceHealth::default();
        assert_eq!(health.status, "healthy");
        assert_eq!(health.active_transactions, 0);
        assert!(health.dependencies.is_empty());
    }
}
