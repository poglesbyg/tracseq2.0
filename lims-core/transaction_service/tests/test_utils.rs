use transaction_service::{coordinator::*, saga::*, test_utils::*, models::*};
use fake::{Fake, Faker};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;
use uuid::Uuid;
use async_trait::async_trait;

/// Test environment for transaction service testing
pub struct TestTransactionEnvironment {
    pub coordinator: Arc<TransactionCoordinator>,
    pub config: Arc<CoordinatorConfig>,
    pub active_transactions: Vec<Uuid>,
    pub mock_services: HashMap<String, Arc<dyn MockService>>,
}

impl TestTransactionEnvironment {
    pub async fn new() -> Self {
        let coordinator = get_test_coordinator().await;
        let config = CoordinatorConfig::test_config();
        
        Self {
            coordinator: Arc::new(coordinator.clone()),
            config: Arc::new(config),
            active_transactions: Vec::new(),
            mock_services: HashMap::new(),
        }
    }

    pub async fn cleanup(&mut self) {
        // Cancel any active transactions
        for transaction_id in &self.active_transactions {
            let _ = self.coordinator.cancel_transaction(*transaction_id).await;
        }
        
        self.active_transactions.clear();
        self.mock_services.clear();
    }

    pub fn track_transaction(&mut self, transaction_id: Uuid) {
        self.active_transactions.push(transaction_id);
    }

    pub fn add_mock_service(&mut self, name: String, service: Arc<dyn MockService>) {
        self.mock_services.insert(name, service);
    }
}

/// Mock service trait for testing distributed transactions
#[async_trait]
pub trait MockService: Send + Sync {
    async fn execute_step(&self, step_name: &str, context: &SagaContext) -> Result<serde_json::Value, SagaError>;
    async fn compensate_step(&self, step_name: &str, context: &SagaContext) -> Result<(), SagaError>;
    fn service_name(&self) -> String;
}

/// Mock sample service for testing
pub struct MockSampleService {
    pub should_fail: bool,
    pub execution_delay_ms: u64,
    pub compensation_delay_ms: u64,
}

impl MockSampleService {
    pub fn new() -> Self {
        Self {
            should_fail: false,
            execution_delay_ms: 10,
            compensation_delay_ms: 5,
        }
    }

    pub fn with_failure(mut self) -> Self {
        self.should_fail = true;
        self
    }

    pub fn with_delays(mut self, execution_ms: u64, compensation_ms: u64) -> Self {
        self.execution_delay_ms = execution_ms;
        self.compensation_delay_ms = compensation_ms;
        self
    }
}

#[async_trait]
impl MockService for MockSampleService {
    async fn execute_step(&self, step_name: &str, context: &SagaContext) -> Result<serde_json::Value, SagaError> {
        tokio::time::sleep(Duration::from_millis(self.execution_delay_ms)).await;
        
        if self.should_fail {
            return Err(SagaError::StepFailed {
                step_name: step_name.to_string(),
                message: "Mock sample service failure".to_string(),
            });
        }

        match step_name {
            "create_sample" => Ok(serde_json::json!({
                "sample_id": Uuid::new_v4(),
                "barcode": "TEST-001",
                "status": "Created"
            })),
            "validate_sample" => Ok(serde_json::json!({
                "validation_result": "Passed",
                "validated_at": chrono::Utc::now()
            })),
            "store_sample" => Ok(serde_json::json!({
                "storage_location": "A1-01",
                "stored_at": chrono::Utc::now()
            })),
            _ => Err(SagaError::StepFailed {
                step_name: step_name.to_string(),
                message: "Unknown step".to_string(),
            }),
        }
    }

    async fn compensate_step(&self, step_name: &str, _context: &SagaContext) -> Result<(), SagaError> {
        tokio::time::sleep(Duration::from_millis(self.compensation_delay_ms)).await;
        
        if self.should_fail {
            return Err(SagaError::CompensationFailed {
                step_name: step_name.to_string(),
                message: "Mock compensation failure".to_string(),
            });
        }

        // Simulate successful compensation
        Ok(())
    }

    fn service_name(&self) -> String {
        "mock-sample-service".to_string()
    }
}

/// Mock storage service for testing
pub struct MockStorageService {
    pub should_fail: bool,
    pub capacity_full: bool,
}

impl MockStorageService {
    pub fn new() -> Self {
        Self {
            should_fail: false,
            capacity_full: false,
        }
    }

    pub fn with_failure(mut self) -> Self {
        self.should_fail = true;
        self
    }

    pub fn with_full_capacity(mut self) -> Self {
        self.capacity_full = true;
        self
    }
}

#[async_trait]
impl MockService for MockStorageService {
    async fn execute_step(&self, step_name: &str, _context: &SagaContext) -> Result<serde_json::Value, SagaError> {
        if self.should_fail {
            return Err(SagaError::StepFailed {
                step_name: step_name.to_string(),
                message: "Mock storage service failure".to_string(),
            });
        }

        if self.capacity_full && step_name == "reserve_storage" {
            return Err(SagaError::StepFailed {
                step_name: step_name.to_string(),
                message: "Storage capacity full".to_string(),
            });
        }

        match step_name {
            "reserve_storage" => Ok(serde_json::json!({
                "reservation_id": Uuid::new_v4(),
                "location": "B2-05",
                "reserved_at": chrono::Utc::now()
            })),
            "move_sample" => Ok(serde_json::json!({
                "move_id": Uuid::new_v4(),
                "from_location": "A1-01",
                "to_location": "B2-05",
                "moved_at": chrono::Utc::now()
            })),
            _ => Err(SagaError::StepFailed {
                step_name: step_name.to_string(),
                message: "Unknown storage step".to_string(),
            }),
        }
    }

    async fn compensate_step(&self, step_name: &str, _context: &SagaContext) -> Result<(), SagaError> {
        // Simulate compensation (e.g., cancel reservation, restore location)
        Ok(())
    }

    fn service_name(&self) -> String {
        "mock-storage-service".to_string()
    }
}

/// Factory for creating test transaction requests
pub struct TransactionRequestFactory;

impl TransactionRequestFactory {
    pub fn create_sample_processing_request() -> TransactionRequest {
        TransactionRequest {
            name: "Sample Processing Workflow".to_string(),
            transaction_type: "sample_processing".to_string(),
            user_id: Some(Uuid::new_v4()),
            correlation_id: Some(Uuid::new_v4()),
            timeout_ms: Some(30000),
            metadata: {
                let mut metadata = HashMap::new();
                metadata.insert("sample_type".to_string(), serde_json::json!("DNA"));
                metadata.insert("priority".to_string(), serde_json::json!("High"));
                metadata
            },
            context_data: {
                let mut context = HashMap::new();
                context.insert("submitter_id".to_string(), serde_json::json!(Uuid::new_v4()));
                context.insert("lab_id".to_string(), serde_json::json!("LAB-001"));
                context
            },
        }
    }

    pub fn create_rag_processing_request() -> TransactionRequest {
        TransactionRequest {
            name: "RAG Document Processing".to_string(),
            transaction_type: "rag_processing".to_string(),
            user_id: Some(Uuid::new_v4()),
            correlation_id: Some(Uuid::new_v4()),
            timeout_ms: Some(60000), // Longer timeout for AI processing
            metadata: {
                let mut metadata = HashMap::new();
                metadata.insert("document_type".to_string(), serde_json::json!("lab_submission"));
                metadata.insert("confidence_threshold".to_string(), serde_json::json!(0.85));
                metadata
            },
            context_data: {
                let mut context = HashMap::new();
                context.insert("document_id".to_string(), serde_json::json!(Uuid::new_v4()));
                context.insert("extraction_categories".to_string(), serde_json::json!(7));
                context
            },
        }
    }

    pub fn create_storage_workflow_request() -> TransactionRequest {
        TransactionRequest {
            name: "Storage Management Workflow".to_string(),
            transaction_type: "storage_management".to_string(),
            user_id: Some(Uuid::new_v4()),
            correlation_id: Some(Uuid::new_v4()),
            timeout_ms: Some(20000),
            metadata: {
                let mut metadata = HashMap::new();
                metadata.insert("storage_type".to_string(), serde_json::json!("freezer"));
                metadata.insert("temperature".to_string(), serde_json::json!(-80));
                metadata
            },
            context_data: {
                let mut context = HashMap::new();
                context.insert("samples_count".to_string(), serde_json::json!(5));
                context.insert("robot_id".to_string(), serde_json::json!("ROBOT-01"));
                context
            },
        }
    }

    pub fn create_batch_request(batch_size: usize) -> Vec<TransactionRequest> {
        (0..batch_size)
            .map(|i| TransactionRequest {
                name: format!("Batch Transaction {}", i),
                transaction_type: "batch_processing".to_string(),
                user_id: Some(Uuid::new_v4()),
                correlation_id: Some(Uuid::new_v4()),
                timeout_ms: Some(15000),
                metadata: {
                    let mut metadata = HashMap::new();
                    metadata.insert("batch_index".to_string(), serde_json::json!(i));
                    metadata.insert("batch_size".to_string(), serde_json::json!(batch_size));
                    metadata
                },
                context_data: HashMap::new(),
            })
            .collect()
    }
}

/// Factory for creating test sagas
pub struct SagaFactory;

impl SagaFactory {
    pub fn create_sample_processing_saga() -> TransactionSaga {
        let context = SagaContext {
            transaction_id: Uuid::new_v4(),
            user_id: Some(Uuid::new_v4()),
            correlation_id: Some(Uuid::new_v4()),
            metadata: HashMap::new(),
        };

        let mut saga = TransactionSaga::new(context);
        
        // Add steps for sample processing workflow
        saga.add_step(SagaStep {
            name: "create_sample".to_string(),
            service: "sample-service".to_string(),
            action: "create".to_string(),
            compensation_action: Some("delete".to_string()),
            timeout_ms: 5000,
            retry_count: 3,
            parameters: serde_json::json!({
                "sample_type": "DNA",
                "submitter": "test-user"
            }),
        });

        saga.add_step(SagaStep {
            name: "validate_sample".to_string(),
            service: "sample-service".to_string(),
            action: "validate".to_string(),
            compensation_action: Some("invalidate".to_string()),
            timeout_ms: 3000,
            retry_count: 2,
            parameters: serde_json::json!({
                "validation_rules": ["type", "quality", "integrity"]
            }),
        });

        saga.add_step(SagaStep {
            name: "reserve_storage".to_string(),
            service: "storage-service".to_string(),
            action: "reserve".to_string(),
            compensation_action: Some("cancel_reservation".to_string()),
            timeout_ms: 2000,
            retry_count: 3,
            parameters: serde_json::json!({
                "storage_type": "freezer",
                "temperature": -80
            }),
        });

        saga.add_step(SagaStep {
            name: "store_sample".to_string(),
            service: "storage-service".to_string(),
            action: "store".to_string(),
            compensation_action: Some("remove".to_string()),
            timeout_ms: 4000,
            retry_count: 2,
            parameters: serde_json::json!({
                "location": "A1-01"
            }),
        });

        saga
    }

    pub fn create_failing_saga() -> TransactionSaga {
        let context = SagaContext {
            transaction_id: Uuid::new_v4(),
            user_id: Some(Uuid::new_v4()),
            correlation_id: Some(Uuid::new_v4()),
            metadata: HashMap::new(),
        };

        let mut saga = TransactionSaga::new(context);
        
        // Add step that will fail
        saga.add_step(SagaStep {
            name: "failing_step".to_string(),
            service: "failing-service".to_string(),
            action: "fail".to_string(),
            compensation_action: Some("compensate".to_string()),
            timeout_ms: 1000,
            retry_count: 1,
            parameters: serde_json::json!({
                "should_fail": true
            }),
        });

        saga
    }

    pub fn create_compensation_test_saga() -> TransactionSaga {
        let context = SagaContext {
            transaction_id: Uuid::new_v4(),
            user_id: Some(Uuid::new_v4()),
            correlation_id: Some(Uuid::new_v4()),
            metadata: HashMap::new(),
        };

        let mut saga = TransactionSaga::new(context);
        
        // Add multiple steps where later ones will fail
        saga.add_step(SagaStep {
            name: "step_1_success".to_string(),
            service: "test-service".to_string(),
            action: "execute".to_string(),
            compensation_action: Some("compensate_1".to_string()),
            timeout_ms: 1000,
            retry_count: 1,
            parameters: serde_json::json!({}),
        });

        saga.add_step(SagaStep {
            name: "step_2_success".to_string(),
            service: "test-service".to_string(),
            action: "execute".to_string(),
            compensation_action: Some("compensate_2".to_string()),
            timeout_ms: 1000,
            retry_count: 1,
            parameters: serde_json::json!({}),
        });

        saga.add_step(SagaStep {
            name: "step_3_fail".to_string(),
            service: "failing-service".to_string(),
            action: "fail".to_string(),
            compensation_action: Some("compensate_3".to_string()),
            timeout_ms: 1000,
            retry_count: 1,
            parameters: serde_json::json!({
                "should_fail": true
            }),
        });

        saga
    }
}

/// Performance testing utilities
pub struct TransactionPerformanceUtils;

impl TransactionPerformanceUtils {
    pub async fn measure_transaction_execution_time(
        coordinator: &TransactionCoordinator,
        request: TransactionRequest,
        saga: TransactionSaga,
    ) -> (std::time::Duration, Result<SagaExecutionResult, SagaError>) {
        let start = std::time::Instant::now();
        let result = coordinator.execute_transaction(request, saga).await;
        let duration = start.elapsed();
        (duration, result)
    }

    pub async fn concurrent_transaction_execution(
        coordinator: &TransactionCoordinator,
        requests_and_sagas: Vec<(TransactionRequest, TransactionSaga)>,
    ) -> Vec<Result<SagaExecutionResult, SagaError>> {
        let tasks: Vec<_> = requests_and_sagas
            .into_iter()
            .map(|(request, saga)| {
                let coordinator = coordinator.clone();
                tokio::spawn(async move {
                    coordinator.execute_transaction(request, saga).await
                })
            })
            .collect();

        futures::future::join_all(tasks)
            .await
            .into_iter()
            .map(|result| result.unwrap_or_else(|e| Err(SagaError::Generic {
                message: format!("Task error: {}", e),
            })))
            .collect()
    }

    pub async fn measure_saga_throughput(
        coordinator: &TransactionCoordinator,
        transaction_count: usize,
    ) -> (std::time::Duration, usize) {
        let start = std::time::Instant::now();
        let mut successful_count = 0;

        for _ in 0..transaction_count {
            let request = TransactionRequestFactory::create_sample_processing_request();
            let saga = SagaFactory::create_sample_processing_saga();
            
            match coordinator.execute_transaction(request, saga).await {
                Ok(_) => successful_count += 1,
                Err(_) => {},
            }
        }

        (start.elapsed(), successful_count)
    }
}

/// Assertions for transaction testing
pub struct TransactionAssertions;

impl TransactionAssertions {
    pub fn assert_transaction_success(result: &Result<SagaExecutionResult, SagaError>) {
        assert!(result.is_ok(), "Transaction should complete successfully");
        if let Ok(execution_result) = result {
            assert_eq!(execution_result.status, SagaStatus::Completed);
        }
    }

    pub fn assert_transaction_compensated(result: &Result<SagaExecutionResult, SagaError>) {
        assert!(result.is_ok(), "Transaction should complete with compensation");
        if let Ok(execution_result) = result {
            assert_eq!(execution_result.status, SagaStatus::Compensated);
            assert!(execution_result.compensation_executed);
        }
    }

    pub fn assert_transaction_failed(result: &Result<SagaExecutionResult, SagaError>) {
        assert!(result.is_err(), "Transaction should fail");
    }

    pub fn assert_transaction_performance(duration: std::time::Duration, max_ms: u64) {
        assert!(
            duration.as_millis() <= max_ms as u128,
            "Transaction took {}ms, expected <= {}ms",
            duration.as_millis(),
            max_ms
        );
    }

    pub fn assert_saga_step_count(saga: &TransactionSaga, expected_count: usize) {
        assert_eq!(
            saga.steps.len(),
            expected_count,
            "Expected {} steps, but saga has {}",
            expected_count,
            saga.steps.len()
        );
    }

    pub fn assert_concurrent_success_rate(
        results: &[Result<SagaExecutionResult, SagaError>],
        min_success_rate: f64,
    ) {
        let successful_count = results.iter().filter(|r| r.is_ok()).count();
        let success_rate = successful_count as f64 / results.len() as f64;
        
        assert!(
            success_rate >= min_success_rate,
            "Success rate {:.2}% is below minimum {:.2}%",
            success_rate * 100.0,
            min_success_rate * 100.0
        );
    }
} 
