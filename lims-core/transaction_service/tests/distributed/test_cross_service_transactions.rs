use transaction_service::*;
use crate::test_utils::*;
use std::time::Duration;
use uuid::Uuid;

#[test_with_transaction_cleanup]
async fn test_laboratory_workflow_transaction(test_env: &mut TestTransactionEnvironment) {
    // Set up mock services
    let sample_service = Arc::new(MockSampleService::new());
    let storage_service = Arc::new(MockStorageService::new());
    
    test_env.add_mock_service("sample-service".to_string(), sample_service);
    test_env.add_mock_service("storage-service".to_string(), storage_service);

    // Create laboratory workflow transaction
    let request = TransactionRequestFactory::create_sample_processing_request();
    let saga = SagaFactory::create_sample_processing_saga();
    let transaction_id = saga.id;
    
    test_env.track_transaction(transaction_id);

    // Execute the distributed transaction
    let (duration, result) = TransactionPerformanceUtils::measure_transaction_execution_time(
        &test_env.coordinator,
        request,
        saga,
    ).await;

    // Verify successful completion
    TransactionAssertions::assert_transaction_success(&result);
    TransactionAssertions::assert_transaction_performance(duration, 10000);

    // Verify transaction status
    let status = test_env.coordinator.get_transaction_status(transaction_id).await;
    assert!(status.is_some());
    
    let status = status.unwrap();
    assert_eq!(status.status, SagaStatus::Completed);
    assert_eq!(status.completed_steps, 4);
    assert!(status.progress >= 100.0);
}

#[test_with_transaction_cleanup]
async fn test_saga_compensation_on_failure(test_env: &mut TestTransactionEnvironment) {
    // Set up mock services - storage service will fail
    let sample_service = Arc::new(MockSampleService::new());
    let storage_service = Arc::new(MockStorageService::new().with_failure());
    
    test_env.add_mock_service("sample-service".to_string(), sample_service);
    test_env.add_mock_service("storage-service".to_string(), storage_service);

    // Create saga that will fail at storage step
    let request = TransactionRequestFactory::create_sample_processing_request();
    let saga = SagaFactory::create_compensation_test_saga();
    let transaction_id = saga.id;
    
    test_env.track_transaction(transaction_id);

    // Execute the transaction (expecting compensation)
    let result = test_env.coordinator.execute_transaction(request, saga).await;

    // Verify compensation was executed
    TransactionAssertions::assert_transaction_compensated(&result);

    // Verify transaction status shows compensation
    let status = test_env.coordinator.get_transaction_status(transaction_id).await;
    assert!(status.is_some());
    
    let status = status.unwrap();
    assert_eq!(status.status, SagaStatus::Compensated);
    assert!(status.error_message.is_some());
}

#[test_with_transaction_cleanup]
async fn test_rag_document_processing_transaction(test_env: &mut TestTransactionEnvironment) {
    // *Context added by Giga rag-algorithms*
    
    // Create RAG-specific mock services
    let rag_service = Arc::new(MockRagService::new());
    let sample_service = Arc::new(MockSampleService::new());
    
    test_env.add_mock_service("rag-service".to_string(), rag_service);
    test_env.add_mock_service("sample-service".to_string(), sample_service);

    // Create RAG processing transaction
    let request = TransactionRequestFactory::create_rag_processing_request();
    let saga = SagaFactory::create_rag_processing_saga();
    let transaction_id = saga.id;
    
    test_env.track_transaction(transaction_id);

    // Execute RAG workflow transaction
    let result = test_env.coordinator.execute_transaction(request, saga).await;

    // Verify RAG processing completed successfully
    TransactionAssertions::assert_transaction_success(&result);

    // Verify RAG-specific metrics
    if let Ok(execution_result) = result {
        assert!(execution_result.execution_time_ms > 0);
        assert!(execution_result.execution_time_ms < 60000); // Under 1 minute
        
        // Verify all RAG steps completed
        let status = test_env.coordinator.get_transaction_status(transaction_id).await.unwrap();
        assert_eq!(status.completed_steps, status.total_steps);
    }
}

#[test_with_transaction_cleanup]
async fn test_concurrent_transactions(test_env: &mut TestTransactionEnvironment) {
    // Set up reliable mock services
    let sample_service = Arc::new(MockSampleService::new());
    let storage_service = Arc::new(MockStorageService::new());
    
    test_env.add_mock_service("sample-service".to_string(), sample_service);
    test_env.add_mock_service("storage-service".to_string(), storage_service);

    // Create concurrent transactions
    let transaction_count = 5;
    let requests_and_sagas: Vec<_> = (0..transaction_count)
        .map(|_| {
            let request = TransactionRequestFactory::create_sample_processing_request();
            let saga = SagaFactory::create_sample_processing_saga();
            test_env.track_transaction(saga.id);
            (request, saga)
        })
        .collect();

    // Execute concurrent transactions
    let results = TransactionPerformanceUtils::concurrent_transaction_execution(
        &test_env.coordinator,
        requests_and_sagas,
    ).await;

    // Verify success rate for concurrent transactions
    TransactionAssertions::assert_concurrent_success_rate(&results, 0.8); // 80% success rate
}

// Mock RAG service for testing RAG workflows
pub struct MockRagService {
    pub should_fail: bool,
}

impl MockRagService {
    pub fn new() -> Self {
        Self { should_fail: false }
    }
}

#[async_trait::async_trait]
impl MockService for MockRagService {
    async fn execute_step(&self, step_name: &str, _context: &SagaContext) -> Result<serde_json::Value, SagaError> {
        if self.should_fail {
            return Err(SagaError::StepFailed {
                step_name: step_name.to_string(),
                message: "Mock RAG service failure".to_string(),
            });
        }

        match step_name {
            "process_document" => Ok(serde_json::json!({
                "document_id": Uuid::new_v4(),
                "extraction_confidence": 0.92,
                "extracted_categories": 7,
                "processing_time_ms": 2500
            })),
            "extract_samples" => Ok(serde_json::json!({
                "extracted_samples": [
                    {"sample_id": "COVID-001", "sample_type": "RNA", "confidence": 0.95},
                    {"sample_id": "COVID-002", "sample_type": "DNA", "confidence": 0.88}
                ]
            })),
            _ => Err(SagaError::StepFailed {
                step_name: step_name.to_string(),
                message: "Unknown RAG step".to_string(),
            }),
        }
    }

    async fn compensate_step(&self, _step_name: &str, _context: &SagaContext) -> Result<(), SagaError> {
        Ok(())
    }

    fn service_name(&self) -> String {
        "mock-rag-service".to_string()
    }
}

impl SagaFactory {
    pub fn create_rag_processing_saga() -> TransactionSaga {
        let context = SagaContext {
            transaction_id: Uuid::new_v4(),
            user_id: Some(Uuid::new_v4()),
            correlation_id: Some(Uuid::new_v4()),
            metadata: std::collections::HashMap::new(),
        };

        let mut saga = TransactionSaga::new(context);
        
        // RAG-specific workflow steps
        saga.add_step(SagaStep {
            name: "process_document".to_string(),
            service: "rag-service".to_string(),
            action: "process".to_string(),
            compensation_action: Some("cleanup_processing".to_string()),
            timeout_ms: 30000,
            retry_count: 2,
            parameters: serde_json::json!({
                "confidence_threshold": 0.85,
                "extraction_categories": 7
            }),
        });

        saga.add_step(SagaStep {
            name: "extract_samples".to_string(),
            service: "rag-service".to_string(),
            action: "extract".to_string(),
            compensation_action: Some("remove_extractions".to_string()),
            timeout_ms: 15000,
            retry_count: 3,
            parameters: serde_json::json!({
                "sample_types": ["DNA", "RNA", "Protein"]
            }),
        });

        saga
    }
} 
