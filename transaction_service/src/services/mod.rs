//! Business logic services for the transaction service.

use crate::coordinator::{TransactionCoordinator, TransactionRequest};
use crate::models::{TransactionServiceHealth, DependencyHealth};
use crate::saga::{
    TransactionSaga, TransactionContext,
    step::{CreateSampleStep, ValidateSampleStep, AssignStorageStep, SendNotificationStep, SampleCreationData, StorageRequirements},
    compensation::{DeleteSampleCompensation, ReleaseStorageCompensation, ReverseValidationCompensation, CancelNotificationCompensation},
};
use anyhow::Result;
use chrono::Utc;
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Instant;
use uuid::Uuid;

/// Service for health checks and monitoring
#[derive(Clone)]
pub struct HealthService {
    start_time: Instant,
    coordinator: Arc<TransactionCoordinator>,
}

impl HealthService {
    pub fn new(coordinator: Arc<TransactionCoordinator>) -> Self {
        Self {
            start_time: Instant::now(),
            coordinator,
        }
    }

    /// Get comprehensive health status
    pub async fn get_health(&self) -> TransactionServiceHealth {
        let mut health = TransactionServiceHealth::default();
        
        health.uptime_seconds = self.start_time.elapsed().as_secs();
        
        // Get active transactions count
        let stats = self.coordinator.get_statistics().await;
        health.active_transactions = stats.active_transactions;
        
        // Check dependencies
        health.dependencies = self.check_dependencies().await;
        
        // Determine overall health status
        let all_deps_healthy = health.dependencies.values()
            .all(|dep| dep.status == "healthy");
            
        if !all_deps_healthy {
            health.status = "degraded".to_string();
        }
        
        health
    }

    /// Check health of external dependencies
    async fn check_dependencies(&self) -> HashMap<String, DependencyHealth> {
        let mut dependencies = HashMap::new();
        
        // Check event service
        dependencies.insert("event_service".to_string(), 
            self.check_event_service().await);
        
        // Check other TracSeq services
        dependencies.insert("sample_service".to_string(), 
            self.check_service("http://localhost:8081", "sample_service").await);
            
        dependencies.insert("storage_service".to_string(), 
            self.check_service("http://localhost:8082", "storage_service").await);
            
        dependencies.insert("notification_service".to_string(), 
            self.check_service("http://localhost:8085", "notification_service").await);
        
        dependencies
    }

    async fn check_event_service(&self) -> DependencyHealth {
        let start_time = Instant::now();
        let event_service_url = std::env::var("EVENT_SERVICE_URL")
            .unwrap_or_else(|_| "http://localhost:8087".to_string());
        
        match self.check_service(&event_service_url, "event_service").await {
            dep if dep.status == "healthy" => dep,
            mut dep => {
                dep.error_message = Some("Event service is required for transaction coordination".to_string());
                dep
            }
        }
    }

    async fn check_service(&self, url: &str, name: &str) -> DependencyHealth {
        let start_time = Instant::now();
        let client = reqwest::Client::new();
        
        match client.get(&format!("{}/health", url))
            .timeout(tokio::time::Duration::from_millis(5000))
            .send()
            .await
        {
            Ok(response) if response.status().is_success() => {
                DependencyHealth {
                    name: name.to_string(),
                    status: "healthy".to_string(),
                    response_time_ms: start_time.elapsed().as_millis() as u64,
                    last_checked: Utc::now(),
                    error_message: None,
                }
            }
            Ok(response) => {
                DependencyHealth {
                    name: name.to_string(),
                    status: "unhealthy".to_string(),
                    response_time_ms: start_time.elapsed().as_millis() as u64,
                    last_checked: Utc::now(),
                    error_message: Some(format!("HTTP {}", response.status())),
                }
            }
            Err(e) => {
                DependencyHealth {
                    name: name.to_string(),
                    status: "unhealthy".to_string(),
                    response_time_ms: start_time.elapsed().as_millis() as u64,
                    last_checked: Utc::now(),
                    error_message: Some(e.to_string()),
                }
            }
        }
    }
}

/// Service for creating and managing pre-built laboratory workflows
#[derive(Clone)]
pub struct WorkflowService {
    coordinator: Arc<TransactionCoordinator>,
}

impl WorkflowService {
    pub fn new(coordinator: Arc<TransactionCoordinator>) -> Self {
        Self { coordinator }
    }

    /// Create a sample submission workflow
    pub async fn create_sample_submission_workflow(
        &self,
        request: TransactionRequest,
        sample_data: SampleCreationData,
        storage_requirements: StorageRequirements,
        notification_recipients: Vec<String>,
    ) -> Result<TransactionSaga> {
        let context = TransactionContext::new()
            .with_user_id(request.user_id.unwrap_or_default())
            .with_correlation_id(request.correlation_id.unwrap_or_else(Uuid::new_v4));

        // Create saga with steps and compensations
        let saga = TransactionSaga::builder("sample_submission_workflow")
            .with_context(context)
            .with_timeout(request.timeout_ms.unwrap_or(300000)) // 5 minutes
            .with_max_retries(3)
            
            // Step 1: Create sample
            .add_step(CreateSampleStep { sample_data: sample_data.clone() })
            .add_compensation(DeleteSampleCompensation {
                sample_id: Uuid::new_v4(), // This would be populated at runtime
                force_delete: true,
            })
            
            // Step 2: Validate sample
            .add_step(ValidateSampleStep {
                sample_id: Uuid::new_v4(), // This would be populated from step 1 output
                validation_rules: vec![
                    "check_barcode_format".to_string(),
                    "verify_sample_type".to_string(),
                    "validate_metadata".to_string(),
                ],
            })
            .add_compensation(ReverseValidationCompensation {
                sample_id: Uuid::new_v4(), // This would be populated at runtime
                original_status: "pending".to_string(),
            })
            
            // Step 3: Assign storage
            .add_step(AssignStorageStep {
                sample_id: Uuid::new_v4(), // This would be populated from step 1 output
                storage_requirements,
            })
            .add_compensation(ReleaseStorageCompensation {
                sample_id: Uuid::new_v4(), // This would be populated at runtime
                location_id: None, // This would be populated from step 3 output
            })
            
            // Step 4: Send notifications
            .add_step(SendNotificationStep {
                notification_type: "sample_submitted".to_string(),
                recipients: notification_recipients,
                message_template: "sample_submission_success".to_string(),
                context_data: HashMap::from([
                    ("sample_barcode".to_string(), serde_json::Value::String(sample_data.barcode)),
                    ("submitter_id".to_string(), serde_json::Value::String(sample_data.submitter_id.to_string())),
                ]),
            })
            .add_compensation(CancelNotificationCompensation {
                notification_ids: vec![], // This would be populated from step 4 output
                cancellation_reason: "Transaction failed".to_string(),
            })
            
            .build();

        Ok(saga)
    }

    /// Create a sample sequencing workflow
    pub async fn create_sample_sequencing_workflow(
        &self,
        request: TransactionRequest,
        sample_id: Uuid,
        sequencing_config: serde_json::Value,
    ) -> Result<TransactionSaga> {
        let context = TransactionContext::new()
            .with_user_id(request.user_id.unwrap_or_default())
            .with_correlation_id(request.correlation_id.unwrap_or_else(Uuid::new_v4))
            .with_metadata("sequencing_config".to_string(), sequencing_config);

        // This would be expanded with actual sequencing steps
        let saga = TransactionSaga::builder("sample_sequencing_workflow")
            .with_context(context)
            .with_timeout(request.timeout_ms.unwrap_or(1800000)) // 30 minutes
            .with_max_retries(2)
            .build();

        Ok(saga)
    }

    /// Create a bulk sample operation workflow
    pub async fn create_bulk_sample_workflow(
        &self,
        request: TransactionRequest,
        sample_ids: Vec<Uuid>,
        operation: String,
    ) -> Result<TransactionSaga> {
        let context = TransactionContext::new()
            .with_user_id(request.user_id.unwrap_or_default())
            .with_correlation_id(request.correlation_id.unwrap_or_else(Uuid::new_v4))
            .with_metadata("sample_ids".to_string(), serde_json::json!(sample_ids))
            .with_metadata("operation".to_string(), serde_json::Value::String(operation));

        // This would be expanded with actual bulk operation steps
        let saga = TransactionSaga::builder("bulk_sample_workflow")
            .with_context(context)
            .with_timeout(request.timeout_ms.unwrap_or(600000)) // 10 minutes
            .with_max_retries(1) // Fewer retries for bulk operations
            .build();

        Ok(saga)
    }

    /// Execute a workflow using the coordinator
    pub async fn execute_workflow(
        &self,
        request: TransactionRequest,
        saga: TransactionSaga,
    ) -> Result<crate::saga::SagaExecutionResult, crate::saga::SagaError> {
        self.coordinator.execute_transaction(request, saga).await
    }
}

/// Service for metrics and monitoring
#[derive(Clone)]
pub struct MetricsService {
    coordinator: Arc<TransactionCoordinator>,
}

impl MetricsService {
    pub fn new(coordinator: Arc<TransactionCoordinator>) -> Self {
        Self { coordinator }
    }

    /// Get transaction metrics
    pub async fn get_transaction_metrics(&self, saga_id: Uuid) -> Option<crate::models::TransactionMetrics> {
        if let Some(status) = self.coordinator.get_transaction_status(saga_id).await {
            // Convert status to metrics (simplified)
            Some(crate::models::TransactionMetrics {
                transaction_id: status.transaction_id,
                saga_id: status.saga_id,
                transaction_type: "unknown".to_string(), // Would be stored in context
                user_id: None, // Would be extracted from context
                started_at: status.started_at.unwrap_or_else(Utc::now),
                completed_at: None, // Would be calculated from status
                execution_duration_ms: 0, // Would be calculated
                steps_executed: status.completed_steps,
                steps_failed: 0, // Would be calculated from saga state
                retry_attempts: 0, // Would be extracted from saga state
                compensation_executed: false, // Would be determined from status
                final_status: status.status.to_string(),
                error_details: status.error_message.map(|msg| crate::models::TransactionError {
                    error_code: "UNKNOWN".to_string(),
                    error_message: msg,
                    error_category: "unknown".to_string(),
                    failed_step: None,
                    failed_service: None,
                    is_retryable: false,
                    debug_info: None,
                }),
                custom_metrics: serde_json::json!({}),
            })
        } else {
            None
        }
    }

    /// Get coordinator statistics
    pub async fn get_coordinator_statistics(&self) -> crate::coordinator::CoordinatorStatistics {
        self.coordinator.get_statistics().await
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_health_service() {
        let config = CoordinatorConfig {
            enable_events: false,
            ..Default::default()
        };
        let coordinator = Arc::new(TransactionCoordinator::new(config));
        let health_service = HealthService::new(coordinator);
        
        let health = health_service.get_health().await;
        assert_eq!(health.status, "healthy");
        assert_eq!(health.active_transactions, 0);
    }

    #[tokio::test]
    async fn test_workflow_service() {
        let config = CoordinatorConfig {
            enable_events: false,
            ..Default::default()
        };
        let coordinator = Arc::new(TransactionCoordinator::new(config));
        let workflow_service = WorkflowService::new(coordinator);
        
        let request = TransactionRequest {
            name: "test_workflow".to_string(),
            transaction_type: "sample_submission".to_string(),
            user_id: Some(Uuid::new_v4()),
            correlation_id: None,
            timeout_ms: None,
            metadata: HashMap::new(),
            context_data: HashMap::new(),
        };

        let sample_data = SampleCreationData {
            barcode: "TEST-001".to_string(),
            sample_type: "DNA".to_string(),
            submitter_id: Uuid::new_v4(),
            lab_id: Uuid::new_v4(),
            metadata: HashMap::new(),
        };

        let storage_requirements = StorageRequirements {
            temperature_zone: "-80C".to_string(),
            priority: 1,
            duration_days: Some(365),
            special_requirements: vec![],
        };

        let saga = workflow_service.create_sample_submission_workflow(
            request,
            sample_data,
            storage_requirements,
            vec!["admin@lab.com".to_string()],
        ).await;

        assert!(saga.is_ok());
        let saga = saga.unwrap();
        assert_eq!(saga.name, "sample_submission_workflow");
        assert_eq!(saga.steps.len(), 4);
        assert_eq!(saga.compensation_steps.len(), 4);
    }

    #[tokio::test]
    async fn test_metrics_service() {
        let config = CoordinatorConfig {
            enable_events: false,
            ..Default::default()
        };
        let coordinator = Arc::new(TransactionCoordinator::new(config));
        let metrics_service = MetricsService::new(coordinator);
        
        let stats = metrics_service.get_coordinator_statistics().await;
        assert_eq!(stats.active_transactions, 0);
        assert_eq!(stats.total_transactions, 0);
    }
}
