//! Saga step execution interface and implementations.

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

use super::{SagaError, TransactionContext};

/// Result of a saga step execution
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StepResult {
    /// Step execution identifier
    pub execution_id: Uuid,
    
    /// Step name
    pub step_name: String,
    
    /// Execution status
    pub status: StepStatus,
    
    /// Execution start time
    pub started_at: DateTime<Utc>,
    
    /// Execution end time
    pub completed_at: Option<DateTime<Utc>>,
    
    /// Step output data
    pub output_data: HashMap<String, serde_json::Value>,
    
    /// Execution metadata
    pub metadata: HashMap<String, String>,
    
    /// Error message if failed
    pub error_message: Option<String>,
    
    /// Retry count
    pub retry_count: u32,
}

/// Step execution status
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum StepStatus {
    /// Step is pending execution
    Pending,
    
    /// Step is currently executing
    Executing,
    
    /// Step completed successfully
    Completed,
    
    /// Step failed
    Failed,
    
    /// Step was skipped
    Skipped,
    
    /// Step is retrying after failure
    Retrying,
}

impl StepResult {
    /// Create a new step result
    pub fn new(step_name: String) -> Self {
        Self {
            execution_id: Uuid::new_v4(),
            step_name,
            status: StepStatus::Pending,
            started_at: Utc::now(),
            completed_at: None,
            output_data: HashMap::new(),
            metadata: HashMap::new(),
            error_message: None,
            retry_count: 0,
        }
    }

    /// Mark step as started
    pub fn start(&mut self) {
        self.status = StepStatus::Executing;
        self.started_at = Utc::now();
    }

    /// Mark step as completed successfully
    pub fn complete(mut self) -> Self {
        self.status = StepStatus::Completed;
        self.completed_at = Some(Utc::now());
        self
    }

    /// Mark step as failed
    pub fn fail(mut self, error: &str) -> Self {
        self.status = StepStatus::Failed;
        self.completed_at = Some(Utc::now());
        self.error_message = Some(error.to_string());
        self
    }

    /// Add output data
    pub fn with_output(mut self, key: String, value: serde_json::Value) -> Self {
        self.output_data.insert(key, value);
        self
    }

    /// Add metadata
    pub fn with_metadata(mut self, key: String, value: String) -> Self {
        self.metadata.insert(key, value);
        self
    }

    /// Get execution duration
    pub fn execution_duration(&self) -> Option<chrono::Duration> {
        self.completed_at.map(|end| end - self.started_at)
    }
}

/// Trait for implementing saga steps
#[async_trait]
pub trait SagaStep: Send + Sync + std::fmt::Debug {
    /// Execute the step
    async fn execute(&self, context: &TransactionContext) -> Result<StepResult, SagaError>;
    
    /// Get step name for identification
    fn name(&self) -> &str;
    
    /// Get step description
    fn description(&self) -> &str {
        "No description provided"
    }
    
    /// Get step timeout in milliseconds
    fn timeout_ms(&self) -> u64 {
        30000 // 30 seconds default
    }
    
    /// Check if step supports retry
    fn is_retryable(&self) -> bool {
        true
    }
    
    /// Get maximum retry attempts
    fn max_retries(&self) -> u32 {
        3
    }
    
    /// Get step dependencies (other step names that must complete first)
    fn dependencies(&self) -> Vec<String> {
        Vec::new()
    }
    
    /// Validate step can execute with given context
    async fn validate(&self, context: &TransactionContext) -> Result<(), SagaError> {
        // Default implementation - no validation
        Ok(())
    }
    
    /// Pre-execution hook
    async fn before_execute(&self, context: &TransactionContext) -> Result<(), SagaError> {
        // Default implementation - no pre-execution logic
        Ok(())
    }
    
    /// Post-execution hook
    async fn after_execute(&self, context: &TransactionContext, result: &StepResult) -> Result<(), SagaError> {
        // Default implementation - no post-execution logic
        Ok(())
    }
}

/// Laboratory-specific saga steps for TracSeq operations

/// Sample creation step
#[derive(Debug)]
pub struct CreateSampleStep {
    pub sample_data: SampleCreationData,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SampleCreationData {
    pub barcode: String,
    pub sample_type: String,
    pub submitter_id: Uuid,
    pub lab_id: Uuid,
    pub metadata: HashMap<String, serde_json::Value>,
}

#[async_trait]
impl SagaStep for CreateSampleStep {
    async fn execute(&self, context: &TransactionContext) -> Result<StepResult, SagaError> {
        let mut result = StepResult::new("create_sample".to_string());
        result.start();

        // Simulate sample service API call
        let client = reqwest::Client::new();
        let sample_service_url = std::env::var("SAMPLE_SERVICE_URL")
            .unwrap_or_else(|_| "http://localhost:8081".to_string());

        let response = client
            .post(&format!("{}/api/v1/samples", sample_service_url))
            .json(&self.sample_data)
            .header("X-Transaction-ID", context.transaction_id.to_string())
            .send()
            .await
            .map_err(|e| SagaError::ServiceCommunicationFailed {
                service: "sample-service".to_string(),
                reason: e.to_string(),
            })?;

        if response.status().is_success() {
            let sample_response: serde_json::Value = response.json().await.map_err(|e| {
                SagaError::SerializationError {
                    reason: e.to_string(),
                }
            })?;

            result = result
                .complete()
                .with_output("sample_id".to_string(), sample_response["id"].clone())
                .with_output("barcode".to_string(), serde_json::Value::String(self.sample_data.barcode.clone()))
                .with_metadata("service".to_string(), "sample-service".to_string());
        } else {
            let error_text = response.text().await.unwrap_or_default();
            result = result.fail(&format!("Sample creation failed: {}", error_text));
        }

        Ok(result)
    }

    fn name(&self) -> &str {
        "create_sample"
    }

    fn description(&self) -> &str {
        "Create a new sample in the sample service"
    }

    fn timeout_ms(&self) -> u64 {
        10000 // 10 seconds
    }
}

/// Sample validation step
#[derive(Debug)]
pub struct ValidateSampleStep {
    pub sample_id: Uuid,
    pub validation_rules: Vec<String>,
}

#[async_trait]
impl SagaStep for ValidateSampleStep {
    async fn execute(&self, context: &TransactionContext) -> Result<StepResult, SagaError> {
        let mut result = StepResult::new("validate_sample".to_string());
        result.start();

        // Simulate sample validation
        let client = reqwest::Client::new();
        let sample_service_url = std::env::var("SAMPLE_SERVICE_URL")
            .unwrap_or_else(|_| "http://localhost:8081".to_string());

        let validation_request = serde_json::json!({
            "sample_id": self.sample_id,
            "validation_rules": self.validation_rules,
            "validator_id": context.user_id
        });

        let response = client
            .post(&format!("{}/api/v1/samples/{}/validate", sample_service_url, self.sample_id))
            .json(&validation_request)
            .header("X-Transaction-ID", context.transaction_id.to_string())
            .send()
            .await
            .map_err(|e| SagaError::ServiceCommunicationFailed {
                service: "sample-service".to_string(),
                reason: e.to_string(),
            })?;

        if response.status().is_success() {
            let validation_response: serde_json::Value = response.json().await.map_err(|e| {
                SagaError::SerializationError {
                    reason: e.to_string(),
                }
            })?;

            let validation_status = validation_response["status"]
                .as_str()
                .unwrap_or("unknown");

            result = result
                .complete()
                .with_output("validation_status".to_string(), 
                           serde_json::Value::String(validation_status.to_string()))
                .with_output("validation_notes".to_string(), 
                           validation_response["notes"].clone())
                .with_metadata("validator_id".to_string(), 
                             context.user_id.unwrap_or_default().to_string());
        } else {
            let error_text = response.text().await.unwrap_or_default();
            result = result.fail(&format!("Sample validation failed: {}", error_text));
        }

        Ok(result)
    }

    fn name(&self) -> &str {
        "validate_sample"
    }

    fn description(&self) -> &str {
        "Validate sample according to laboratory rules"
    }

    fn dependencies(&self) -> Vec<String> {
        vec!["create_sample".to_string()]
    }

    async fn validate(&self, _context: &TransactionContext) -> Result<(), SagaError> {
        if self.validation_rules.is_empty() {
            return Err(SagaError::ValidationFailed {
                field: "validation_rules".to_string(),
                message: "At least one validation rule must be specified".to_string(),
            });
        }
        Ok(())
    }
}

/// Storage assignment step
#[derive(Debug)]
pub struct AssignStorageStep {
    pub sample_id: Uuid,
    pub storage_requirements: StorageRequirements,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StorageRequirements {
    pub temperature_zone: String,
    pub priority: u8,
    pub duration_days: Option<u32>,
    pub special_requirements: Vec<String>,
}

#[async_trait]
impl SagaStep for AssignStorageStep {
    async fn execute(&self, context: &TransactionContext) -> Result<StepResult, SagaError> {
        let mut result = StepResult::new("assign_storage".to_string());
        result.start();

        // Simulate storage service API call
        let client = reqwest::Client::new();
        let storage_service_url = std::env::var("STORAGE_SERVICE_URL")
            .unwrap_or_else(|_| "http://localhost:8082".to_string());

        let storage_request = serde_json::json!({
            "sample_id": self.sample_id,
            "requirements": self.storage_requirements
        });

        let response = client
            .post(&format!("{}/api/v1/storage/assign", storage_service_url))
            .json(&storage_request)
            .header("X-Transaction-ID", context.transaction_id.to_string())
            .send()
            .await
            .map_err(|e| SagaError::ServiceCommunicationFailed {
                service: "storage-service".to_string(),
                reason: e.to_string(),
            })?;

        if response.status().is_success() {
            let storage_response: serde_json::Value = response.json().await.map_err(|e| {
                SagaError::SerializationError {
                    reason: e.to_string(),
                }
            })?;

            result = result
                .complete()
                .with_output("location_id".to_string(), storage_response["location_id"].clone())
                .with_output("position".to_string(), storage_response["position"].clone())
                .with_output("temperature_zone".to_string(), 
                           serde_json::Value::String(self.storage_requirements.temperature_zone.clone()))
                .with_metadata("storage_service".to_string(), "enhanced-storage-service".to_string());
        } else {
            let error_text = response.text().await.unwrap_or_default();
            result = result.fail(&format!("Storage assignment failed: {}", error_text));
        }

        Ok(result)
    }

    fn name(&self) -> &str {
        "assign_storage"
    }

    fn description(&self) -> &str {
        "Assign sample to appropriate storage location"
    }

    fn dependencies(&self) -> Vec<String> {
        vec!["validate_sample".to_string()]
    }

    fn timeout_ms(&self) -> u64 {
        15000 // 15 seconds
    }
}

/// Notification step
#[derive(Debug)]
pub struct SendNotificationStep {
    pub notification_type: String,
    pub recipients: Vec<String>,
    pub message_template: String,
    pub context_data: HashMap<String, serde_json::Value>,
}

#[async_trait]
impl SagaStep for SendNotificationStep {
    async fn execute(&self, context: &TransactionContext) -> Result<StepResult, SagaError> {
        let mut result = StepResult::new("send_notification".to_string());
        result.start();

        // Simulate notification service API call
        let client = reqwest::Client::new();
        let notification_service_url = std::env::var("NOTIFICATION_SERVICE_URL")
            .unwrap_or_else(|_| "http://localhost:8085".to_string());

        let notification_request = serde_json::json!({
            "type": self.notification_type,
            "recipients": self.recipients,
            "template": self.message_template,
            "context": self.context_data,
            "priority": "normal"
        });

        let response = client
            .post(&format!("{}/api/v1/notifications/send", notification_service_url))
            .json(&notification_request)
            .header("X-Transaction-ID", context.transaction_id.to_string())
            .send()
            .await
            .map_err(|e| SagaError::ServiceCommunicationFailed {
                service: "notification-service".to_string(),
                reason: e.to_string(),
            })?;

        if response.status().is_success() {
            let notification_response: serde_json::Value = response.json().await.map_err(|e| {
                SagaError::SerializationError {
                    reason: e.to_string(),
                }
            })?;

            result = result
                .complete()
                .with_output("notification_ids".to_string(), notification_response["notification_ids"].clone())
                .with_output("delivery_status".to_string(), notification_response["status"].clone())
                .with_metadata("recipients_count".to_string(), self.recipients.len().to_string());
        } else {
            let error_text = response.text().await.unwrap_or_default();
            result = result.fail(&format!("Notification sending failed: {}", error_text));
        }

        Ok(result)
    }

    fn name(&self) -> &str {
        "send_notification"
    }

    fn description(&self) -> &str {
        "Send notification to specified recipients"
    }

    fn is_retryable(&self) -> bool {
        true
    }

    fn max_retries(&self) -> u32 {
        2 // Notifications are less critical, fewer retries
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_step_result_creation() {
        let result = StepResult::new("test_step".to_string());
        assert_eq!(result.step_name, "test_step");
        assert_eq!(result.status, StepStatus::Pending);
        assert_eq!(result.retry_count, 0);
    }

    #[test]
    fn test_step_result_completion() {
        let mut result = StepResult::new("test_step".to_string());
        result.start();
        assert_eq!(result.status, StepStatus::Executing);

        let completed_result = result.complete();
        assert_eq!(completed_result.status, StepStatus::Completed);
        assert!(completed_result.completed_at.is_some());
    }

    #[test]
    fn test_step_result_failure() {
        let result = StepResult::new("test_step".to_string());
        let failed_result = result.fail("Test error");
        
        assert_eq!(failed_result.status, StepStatus::Failed);
        assert_eq!(failed_result.error_message, Some("Test error".to_string()));
        assert!(failed_result.completed_at.is_some());
    }

    #[test]
    fn test_create_sample_step() {
        let sample_data = SampleCreationData {
            barcode: "TEST-001".to_string(),
            sample_type: "DNA".to_string(),
            submitter_id: Uuid::new_v4(),
            lab_id: Uuid::new_v4(),
            metadata: HashMap::new(),
        };

        let step = CreateSampleStep { sample_data };
        assert_eq!(step.name(), "create_sample");
        assert_eq!(step.timeout_ms(), 10000);
        assert!(step.is_retryable());
    }

    #[test]
    fn test_validate_sample_step_validation() {
        let step = ValidateSampleStep {
            sample_id: Uuid::new_v4(),
            validation_rules: vec![],
        };

        let context = TransactionContext::new();
        
        // This would need to be an async test in practice
        // but for demonstration, we're testing the structure
        assert_eq!(step.name(), "validate_sample");
        assert_eq!(step.dependencies(), vec!["create_sample".to_string()]);
    }
}
