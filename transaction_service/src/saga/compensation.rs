//! Compensation step interface and implementations for saga rollback logic.

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

use super::{SagaError, TransactionContext};

/// Result of a compensation step execution
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompensationResult {
    /// Compensation execution identifier
    pub execution_id: Uuid,
    
    /// Compensation step name
    pub step_name: String,
    
    /// Execution status
    pub status: CompensationStatus,
    
    /// Execution start time
    pub started_at: DateTime<Utc>,
    
    /// Execution end time
    pub completed_at: Option<DateTime<Utc>>,
    
    /// Compensation output data
    pub output_data: HashMap<String, serde_json::Value>,
    
    /// Execution metadata
    pub metadata: HashMap<String, String>,
    
    /// Error message if failed
    pub error_message: Option<String>,
    
    /// Retry count
    pub retry_count: u32,
}

/// Compensation execution status
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum CompensationStatus {
    /// Compensation is pending execution
    Pending,
    
    /// Compensation is currently executing
    Executing,
    
    /// Compensation completed successfully
    Completed,
    
    /// Compensation failed
    Failed,
    
    /// Compensation was skipped (not applicable)
    Skipped,
    
    /// Compensation is retrying after failure
    Retrying,
}

impl CompensationResult {
    /// Create a new compensation result
    pub fn new(step_name: String) -> Self {
        Self {
            execution_id: Uuid::new_v4(),
            step_name,
            status: CompensationStatus::Pending,
            started_at: Utc::now(),
            completed_at: None,
            output_data: HashMap::new(),
            metadata: HashMap::new(),
            error_message: None,
            retry_count: 0,
        }
    }

    /// Mark compensation as started
    pub fn start(&mut self) {
        self.status = CompensationStatus::Executing;
        self.started_at = Utc::now();
    }

    /// Mark compensation as completed successfully
    pub fn complete(mut self) -> Self {
        self.status = CompensationStatus::Completed;
        self.completed_at = Some(Utc::now());
        self
    }

    /// Mark compensation as failed
    pub fn fail(mut self, error: &str) -> Self {
        self.status = CompensationStatus::Failed;
        self.completed_at = Some(Utc::now());
        self.error_message = Some(error.to_string());
        self
    }

    /// Mark compensation as skipped
    pub fn skip(mut self, reason: &str) -> Self {
        self.status = CompensationStatus::Skipped;
        self.completed_at = Some(Utc::now());
        self.metadata.insert("skip_reason".to_string(), reason.to_string());
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

/// Trait for implementing compensation steps
#[async_trait]
pub trait CompensationStep: Send + Sync + std::fmt::Debug {
    /// Execute the compensation logic
    async fn compensate(&self, context: &TransactionContext) -> Result<CompensationResult, SagaError>;
    
    /// Get compensation step name for identification
    fn name(&self) -> &str;
    
    /// Get step description
    fn description(&self) -> &str {
        "No description provided"
    }
    
    /// Get step timeout in milliseconds
    fn timeout_ms(&self) -> u64 {
        30000 // 30 seconds default
    }
    
    /// Check if compensation supports retry
    fn is_retryable(&self) -> bool {
        true
    }
    
    /// Get maximum retry attempts
    fn max_retries(&self) -> u32 {
        2 // Fewer retries for compensation
    }
    
    /// Check if compensation is mandatory (must succeed)
    fn is_mandatory(&self) -> bool {
        true
    }
    
    /// Validate compensation can execute with given context
    async fn validate(&self, context: &TransactionContext) -> Result<(), SagaError> {
        // Default implementation - no validation
        Ok(())
    }
    
    /// Pre-compensation hook
    async fn before_compensate(&self, context: &TransactionContext) -> Result<(), SagaError> {
        // Default implementation - no pre-compensation logic
        Ok(())
    }
    
    /// Post-compensation hook
    async fn after_compensate(&self, context: &TransactionContext, result: &CompensationResult) -> Result<(), SagaError> {
        // Default implementation - no post-compensation logic
        Ok(())
    }
}

/// Laboratory-specific compensation steps for TracSeq operations

/// Sample deletion compensation step
#[derive(Debug)]
pub struct DeleteSampleCompensation {
    pub sample_id: Uuid,
    pub force_delete: bool,
}

#[async_trait]
impl CompensationStep for DeleteSampleCompensation {
    async fn compensate(&self, context: &TransactionContext) -> Result<CompensationResult, SagaError> {
        let mut result = CompensationResult::new("delete_sample_compensation".to_string());
        result.start();

        // Simulate sample service API call to delete the sample
        let client = reqwest::Client::new();
        let sample_service_url = std::env::var("SAMPLE_SERVICE_URL")
            .unwrap_or_else(|_| "http://localhost:8081".to_string());

        let delete_request = serde_json::json!({
            "force": self.force_delete,
            "reason": "saga_compensation"
        });

        let response = client
            .delete(&format!("{}/api/v1/samples/{}", sample_service_url, self.sample_id))
            .json(&delete_request)
            .header("X-Transaction-ID", context.transaction_id.to_string())
            .send()
            .await
            .map_err(|e| SagaError::ServiceCommunicationFailed {
                service: "sample-service".to_string(),
                reason: e.to_string(),
            })?;

        if response.status().is_success() {
            result = result
                .complete()
                .with_output("deleted_sample_id".to_string(), serde_json::Value::String(self.sample_id.to_string()))
                .with_metadata("compensation_type".to_string(), "delete_sample".to_string());
        } else if response.status() == reqwest::StatusCode::NOT_FOUND {
            // Sample already doesn't exist, compensation is successful
            result = result
                .skip("Sample already deleted or doesn't exist")
                .with_output("sample_id".to_string(), serde_json::Value::String(self.sample_id.to_string()));
        } else {
            let error_text = response.text().await.unwrap_or_default();
            result = result.fail(&format!("Sample deletion compensation failed: {}", error_text));
        }

        Ok(result)
    }

    fn name(&self) -> &str {
        "delete_sample_compensation"
    }

    fn description(&self) -> &str {
        "Delete sample created during failed saga execution"
    }

    fn timeout_ms(&self) -> u64 {
        15000 // 15 seconds
    }

    fn is_mandatory(&self) -> bool {
        true // Sample deletion should succeed to maintain consistency
    }
}

/// Storage release compensation step
#[derive(Debug)]
pub struct ReleaseStorageCompensation {
    pub sample_id: Uuid,
    pub location_id: Option<Uuid>,
}

#[async_trait]
impl CompensationStep for ReleaseStorageCompensation {
    async fn compensate(&self, context: &TransactionContext) -> Result<CompensationResult, SagaError> {
        let mut result = CompensationResult::new("release_storage_compensation".to_string());
        result.start();

        // Simulate storage service API call to release storage
        let client = reqwest::Client::new();
        let storage_service_url = std::env::var("STORAGE_SERVICE_URL")
            .unwrap_or_else(|_| "http://localhost:8082".to_string());

        let release_request = serde_json::json!({
            "sample_id": self.sample_id,
            "location_id": self.location_id,
            "reason": "saga_compensation"
        });

        let response = client
            .post(&format!("{}/api/v1/storage/release", storage_service_url))
            .json(&release_request)
            .header("X-Transaction-ID", context.transaction_id.to_string())
            .send()
            .await
            .map_err(|e| SagaError::ServiceCommunicationFailed {
                service: "storage-service".to_string(),
                reason: e.to_string(),
            })?;

        if response.status().is_success() {
            let release_response: serde_json::Value = response.json().await.map_err(|e| {
                SagaError::SerializationError {
                    reason: e.to_string(),
                }
            })?;

            result = result
                .complete()
                .with_output("released_location_id".to_string(), release_response["location_id"].clone())
                .with_output("released_at".to_string(), serde_json::Value::String(Utc::now().to_rfc3339()))
                .with_metadata("compensation_type".to_string(), "release_storage".to_string());
        } else if response.status() == reqwest::StatusCode::NOT_FOUND {
            // Storage wasn't assigned, compensation is successful
            result = result
                .skip("Storage location was not assigned")
                .with_output("sample_id".to_string(), serde_json::Value::String(self.sample_id.to_string()));
        } else {
            let error_text = response.text().await.unwrap_or_default();
            result = result.fail(&format!("Storage release compensation failed: {}", error_text));
        }

        Ok(result)
    }

    fn name(&self) -> &str {
        "release_storage_compensation"
    }

    fn description(&self) -> &str {
        "Release storage location assigned during failed saga execution"
    }

    fn timeout_ms(&self) -> u64 {
        10000 // 10 seconds
    }

    fn is_mandatory(&self) -> bool {
        false // Storage release is important but not critical for consistency
    }
}

/// Validation reversal compensation step
#[derive(Debug)]
pub struct ReverseValidationCompensation {
    pub sample_id: Uuid,
    pub original_status: String,
}

#[async_trait]
impl CompensationStep for ReverseValidationCompensation {
    async fn compensate(&self, context: &TransactionContext) -> Result<CompensationResult, SagaError> {
        let mut result = CompensationResult::new("reverse_validation_compensation".to_string());
        result.start();

        // Simulate sample service API call to reverse validation
        let client = reqwest::Client::new();
        let sample_service_url = std::env::var("SAMPLE_SERVICE_URL")
            .unwrap_or_else(|_| "http://localhost:8081".to_string());

        let reverse_request = serde_json::json!({
            "sample_id": self.sample_id,
            "new_status": self.original_status,
            "reason": "saga_compensation",
            "reverted_by": context.user_id
        });

        let response = client
            .put(&format!("{}/api/v1/samples/{}/status", sample_service_url, self.sample_id))
            .json(&reverse_request)
            .header("X-Transaction-ID", context.transaction_id.to_string())
            .send()
            .await
            .map_err(|e| SagaError::ServiceCommunicationFailed {
                service: "sample-service".to_string(),
                reason: e.to_string(),
            })?;

        if response.status().is_success() {
            result = result
                .complete()
                .with_output("reverted_sample_id".to_string(), serde_json::Value::String(self.sample_id.to_string()))
                .with_output("reverted_to_status".to_string(), serde_json::Value::String(self.original_status.clone()))
                .with_metadata("compensation_type".to_string(), "reverse_validation".to_string());
        } else {
            let error_text = response.text().await.unwrap_or_default();
            result = result.fail(&format!("Validation reversal compensation failed: {}", error_text));
        }

        Ok(result)
    }

    fn name(&self) -> &str {
        "reverse_validation_compensation"
    }

    fn description(&self) -> &str {
        "Reverse sample validation to original status"
    }

    fn timeout_ms(&self) -> u64 {
        8000 // 8 seconds
    }

    fn is_mandatory(&self) -> bool {
        true // Status reversal is critical for data consistency
    }
}

/// Notification cancellation compensation step
#[derive(Debug)]
pub struct CancelNotificationCompensation {
    pub notification_ids: Vec<String>,
    pub cancellation_reason: String,
}

#[async_trait]
impl CompensationStep for CancelNotificationCompensation {
    async fn compensate(&self, context: &TransactionContext) -> Result<CompensationResult, SagaError> {
        let mut result = CompensationResult::new("cancel_notification_compensation".to_string());
        result.start();

        // Simulate notification service API call to cancel notifications
        let client = reqwest::Client::new();
        let notification_service_url = std::env::var("NOTIFICATION_SERVICE_URL")
            .unwrap_or_else(|_| "http://localhost:8085".to_string());

        let cancel_request = serde_json::json!({
            "notification_ids": self.notification_ids,
            "reason": self.cancellation_reason
        });

        let response = client
            .post(&format!("{}/api/v1/notifications/cancel", notification_service_url))
            .json(&cancel_request)
            .header("X-Transaction-ID", context.transaction_id.to_string())
            .send()
            .await
            .map_err(|e| SagaError::ServiceCommunicationFailed {
                service: "notification-service".to_string(),
                reason: e.to_string(),
            })?;

        if response.status().is_success() {
            let cancel_response: serde_json::Value = response.json().await.map_err(|e| {
                SagaError::SerializationError {
                    reason: e.to_string(),
                }
            })?;

            result = result
                .complete()
                .with_output("cancelled_notifications".to_string(), cancel_response["cancelled_count"].clone())
                .with_output("failed_cancellations".to_string(), cancel_response["failed_count"].clone())
                .with_metadata("compensation_type".to_string(), "cancel_notifications".to_string());
        } else {
            let error_text = response.text().await.unwrap_or_default();
            result = result.fail(&format!("Notification cancellation compensation failed: {}", error_text));
        }

        Ok(result)
    }

    fn name(&self) -> &str {
        "cancel_notification_compensation"
    }

    fn description(&self) -> &str {
        "Cancel notifications sent during failed saga execution"
    }

    fn timeout_ms(&self) -> u64 {
        5000 // 5 seconds
    }

    fn is_mandatory(&self) -> bool {
        false // Notification cancellation is nice-to-have but not critical
    }

    fn max_retries(&self) -> u32 {
        1 // Limited retries for notifications
    }
}

/// Generic HTTP rollback compensation step
#[derive(Debug)]
pub struct HttpRollbackCompensation {
    pub service_name: String,
    pub rollback_url: String,
    pub rollback_payload: serde_json::Value,
    pub is_critical: bool,
}

#[async_trait]
impl CompensationStep for HttpRollbackCompensation {
    async fn compensate(&self, context: &TransactionContext) -> Result<CompensationResult, SagaError> {
        let mut result = CompensationResult::new("http_rollback_compensation".to_string());
        result.start();

        let client = reqwest::Client::new();

        let response = client
            .post(&self.rollback_url)
            .json(&self.rollback_payload)
            .header("X-Transaction-ID", context.transaction_id.to_string())
            .send()
            .await
            .map_err(|e| SagaError::ServiceCommunicationFailed {
                service: self.service_name.clone(),
                reason: e.to_string(),
            })?;

        if response.status().is_success() {
            let rollback_response: serde_json::Value = response.json().await.map_err(|e| {
                SagaError::SerializationError {
                    reason: e.to_string(),
                }
            })?;

            result = result
                .complete()
                .with_output("rollback_response".to_string(), rollback_response)
                .with_metadata("service".to_string(), self.service_name.clone())
                .with_metadata("compensation_type".to_string(), "http_rollback".to_string());
        } else {
            let error_text = response.text().await.unwrap_or_default();
            result = result.fail(&format!("HTTP rollback compensation failed: {}", error_text));
        }

        Ok(result)
    }

    fn name(&self) -> &str {
        "http_rollback_compensation"
    }

    fn description(&self) -> &str {
        "Generic HTTP-based rollback compensation"
    }

    fn is_mandatory(&self) -> bool {
        self.is_critical
    }

    fn timeout_ms(&self) -> u64 {
        20000 // 20 seconds for generic HTTP calls
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_compensation_result_creation() {
        let result = CompensationResult::new("test_compensation".to_string());
        assert_eq!(result.step_name, "test_compensation");
        assert_eq!(result.status, CompensationStatus::Pending);
        assert_eq!(result.retry_count, 0);
    }

    #[test]
    fn test_compensation_result_completion() {
        let mut result = CompensationResult::new("test_compensation".to_string());
        result.start();
        assert_eq!(result.status, CompensationStatus::Executing);

        let completed_result = result.complete();
        assert_eq!(completed_result.status, CompensationStatus::Completed);
        assert!(completed_result.completed_at.is_some());
    }

    #[test]
    fn test_compensation_result_skip() {
        let result = CompensationResult::new("test_compensation".to_string());
        let skipped_result = result.skip("Not applicable");
        
        assert_eq!(skipped_result.status, CompensationStatus::Skipped);
        assert_eq!(skipped_result.metadata.get("skip_reason"), Some(&"Not applicable".to_string()));
    }

    #[test]
    fn test_delete_sample_compensation() {
        let compensation = DeleteSampleCompensation {
            sample_id: Uuid::new_v4(),
            force_delete: true,
        };

        assert_eq!(compensation.name(), "delete_sample_compensation");
        assert!(compensation.is_mandatory());
        assert_eq!(compensation.timeout_ms(), 15000);
    }

    #[test]
    fn test_release_storage_compensation() {
        let compensation = ReleaseStorageCompensation {
            sample_id: Uuid::new_v4(),
            location_id: Some(Uuid::new_v4()),
        };

        assert_eq!(compensation.name(), "release_storage_compensation");
        assert!(!compensation.is_mandatory());
        assert_eq!(compensation.max_retries(), 2);
    }
}
