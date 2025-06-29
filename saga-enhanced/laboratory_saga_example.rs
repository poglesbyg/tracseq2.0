// TracSeq 2.0 - Laboratory Workflow Saga Example
// Demonstrates integration of Event Sourcing, CQRS, Kafka, and Saga patterns

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::sync::Arc;
use uuid::Uuid;

use crate::saga_orchestrator::{
    SagaDefinition, SagaStep, RetryPolicy, StepHandler, SagaError
};
use crate::commands::{
    CreateSampleCommand, ValidateSampleCommand, StoreSampleCommand,
    StartSequencingCommand, CommandMetadata
};
use crate::kafka::{KafkaEventProducer, EventEnvelope};

// Laboratory Processing Saga Definition
pub fn create_laboratory_processing_saga() -> SagaDefinition {
    SagaDefinition {
        saga_type: "LaboratoryProcessing".to_string(),
        steps: vec![
            SagaStep {
                name: "CreateSample".to_string(),
                service: "sample-service".to_string(),
                command: "CreateSampleCommand".to_string(),
                compensation_command: Some("DeleteSampleCommand".to_string()),
                timeout_seconds: 30,
                can_retry: true,
                depends_on: vec![],
            },
            SagaStep {
                name: "ValidateSample".to_string(),
                service: "sample-service".to_string(),
                command: "ValidateSampleCommand".to_string(),
                compensation_command: Some("RevertValidationCommand".to_string()),
                timeout_seconds: 60,
                can_retry: true,
                depends_on: vec!["CreateSample".to_string()],
            },
            SagaStep {
                name: "AllocateStorage".to_string(),
                service: "storage-service".to_string(),
                command: "AllocateStorageCommand".to_string(),
                compensation_command: Some("ReleaseStorageCommand".to_string()),
                timeout_seconds: 45,
                can_retry: true,
                depends_on: vec!["ValidateSample".to_string()],
            },
            SagaStep {
                name: "StoreSample".to_string(),
                service: "storage-service".to_string(),
                command: "StoreSampleCommand".to_string(),
                compensation_command: Some("RemoveSampleFromStorageCommand".to_string()),
                timeout_seconds: 30,
                can_retry: false,
                depends_on: vec!["AllocateStorage".to_string()],
            },
            SagaStep {
                name: "ScheduleSequencing".to_string(),
                service: "sequencing-service".to_string(),
                command: "ScheduleSequencingCommand".to_string(),
                compensation_command: Some("CancelSequencingCommand".to_string()),
                timeout_seconds: 30,
                can_retry: true,
                depends_on: vec!["StoreSample".to_string()],
            },
            SagaStep {
                name: "SendNotifications".to_string(),
                service: "notification-service".to_string(),
                command: "SendProcessingNotificationCommand".to_string(),
                compensation_command: None, // No compensation needed
                timeout_seconds: 15,
                can_retry: true,
                depends_on: vec!["ScheduleSequencing".to_string()],
            },
        ],
        timeout_seconds: 300, // 5 minutes total
        retry_policy: RetryPolicy {
            max_retries: 3,
            backoff_seconds: 5,
            exponential_backoff: true,
        },
    }
}

// Step Handlers Implementation
pub struct CreateSampleStepHandler {
    sample_service_client: Arc<dyn SampleServiceClient>,
    event_producer: Arc<KafkaEventProducer>,
}

#[async_trait]
impl StepHandler for CreateSampleStepHandler {
    async fn execute(&self, context: &serde_json::Value) -> Result<serde_json::Value, SagaError> {
        // Extract data from context
        let barcode = context["barcode"]
            .as_str()
            .ok_or_else(|| SagaError::StepExecutionError("Missing barcode".to_string()))?;
        
        let sample_type = context["sample_type"]
            .as_str()
            .ok_or_else(|| SagaError::StepExecutionError("Missing sample type".to_string()))?;
        
        let patient_id = context["patient_id"]
            .as_str()
            .and_then(|s| Uuid::parse_str(s).ok());
        
        // Create command
        let command = CreateSampleCommand {
            sample_id: Uuid::new_v4(),
            barcode: barcode.to_string(),
            sample_type: sample_type.to_string(),
            patient_id,
            volume_ml: context["volume_ml"].as_f64().unwrap_or(5.0) as f32,
            collection_date: Utc::now(),
            metadata: CommandMetadata {
                user_id: Uuid::parse_str(context["user_id"].as_str().unwrap_or_default())
                    .unwrap_or_default(),
                correlation_id: Uuid::parse_str(context["correlation_id"].as_str().unwrap_or_default())
                    .unwrap_or_default(),
                causation_id: None,
                tenant_id: context["tenant_id"].as_str().map(String::from),
                ip_address: context["ip_address"].as_str().map(String::from),
            },
        };
        
        // Execute command
        let result = self.sample_service_client
            .create_sample(command.clone())
            .await
            .map_err(|e| SagaError::StepExecutionError(e.to_string()))?;
        
        // Publish event to Kafka
        let event = EventEnvelope {
            event_id: Uuid::new_v4(),
            event_type: "SampleCreatedInSaga".to_string(),
            aggregate_id: command.sample_id,
            aggregate_type: "Sample".to_string(),
            event_version: 1,
            payload: serde_json::to_value(&command)?,
            metadata: crate::kafka::EventMetadata {
                correlation_id: command.metadata.correlation_id,
                causation_id: Some(command.metadata.correlation_id),
                user_id: Some(command.metadata.user_id),
                tenant_id: command.metadata.tenant_id,
                source_service: "saga-orchestrator".to_string(),
            },
            timestamp: Utc::now(),
        };
        
        self.event_producer
            .publish_event(crate::kafka::Topics::SAMPLE_EVENTS, event)
            .await
            .map_err(|e| SagaError::EventPublishError(e.to_string()))?;
        
        // Return updated context
        Ok(json!({
            "sample_id": result.aggregate_id,
            "sample_version": result.version,
            "barcode": barcode,
            "sample_type": sample_type,
        }))
    }
    
    async fn compensate(&self, context: &serde_json::Value) -> Result<(), SagaError> {
        if let Some(sample_id) = context["sample_id"]
            .as_str()
            .and_then(|s| Uuid::parse_str(s).ok())
        {
            // Delete the sample
            self.sample_service_client
                .delete_sample(sample_id)
                .await
                .map_err(|e| SagaError::StepExecutionError(e.to_string()))?;
            
            // Publish compensation event
            let event = EventEnvelope {
                event_id: Uuid::new_v4(),
                event_type: "SampleDeletedInCompensation".to_string(),
                aggregate_id: sample_id,
                aggregate_type: "Sample".to_string(),
                event_version: 1,
                payload: json!({ "sample_id": sample_id }),
                metadata: crate::kafka::EventMetadata {
                    correlation_id: Uuid::new_v4(),
                    causation_id: None,
                    user_id: None,
                    tenant_id: None,
                    source_service: "saga-orchestrator".to_string(),
                },
                timestamp: Utc::now(),
            };
            
            self.event_producer
                .publish_event(crate::kafka::Topics::SAMPLE_EVENTS, event)
                .await
                .map_err(|e| SagaError::EventPublishError(e.to_string()))?;
        }
        
        Ok(())
    }
}

// Service Client Traits
#[async_trait]
pub trait SampleServiceClient: Send + Sync {
    async fn create_sample(&self, command: CreateSampleCommand) -> Result<CommandResult, ClientError>;
    async fn validate_sample(&self, command: ValidateSampleCommand) -> Result<CommandResult, ClientError>;
    async fn delete_sample(&self, sample_id: Uuid) -> Result<(), ClientError>;
}

#[async_trait]
pub trait StorageServiceClient: Send + Sync {
    async fn allocate_storage(&self, sample_id: Uuid, requirements: StorageRequirements) -> Result<StorageAllocation, ClientError>;
    async fn store_sample(&self, command: StoreSampleCommand) -> Result<CommandResult, ClientError>;
    async fn release_storage(&self, allocation_id: Uuid) -> Result<(), ClientError>;
}

// Data Types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StorageRequirements {
    pub sample_type: String,
    pub volume_ml: f32,
    pub temperature_celsius: f32,
    pub duration_days: Option<i32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StorageAllocation {
    pub allocation_id: Uuid,
    pub location_id: Uuid,
    pub position: String,
    pub temperature: f32,
    pub expires_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommandResult {
    pub success: bool,
    pub aggregate_id: Uuid,
    pub version: i32,
}

#[derive(Debug, thiserror::Error)]
pub enum ClientError {
    #[error("Service unavailable: {0}")]
    ServiceUnavailable(String),
    #[error("Command failed: {0}")]
    CommandFailed(String),
    #[error("Network error: {0}")]
    NetworkError(String),
}

// Example Usage
pub async fn process_laboratory_sample(
    saga_orchestrator: Arc<SagaOrchestrator>,
    barcode: String,
    sample_type: String,
    patient_id: Option<Uuid>,
    user_id: Uuid,
) -> Result<Uuid, Box<dyn std::error::Error>> {
    // Prepare initial context
    let initial_context = json!({
        "barcode": barcode,
        "sample_type": sample_type,
        "patient_id": patient_id.map(|id| id.to_string()),
        "user_id": user_id.to_string(),
        "correlation_id": Uuid::new_v4().to_string(),
        "volume_ml": 5.0,
        "temperature_requirement": -80.0,
        "tenant_id": "default",
        "ip_address": "192.168.1.100",
        "requested_at": Utc::now().to_rfc3339(),
    });
    
    // Start the saga
    let saga_id = saga_orchestrator
        .start_saga(
            "LaboratoryProcessing".to_string(),
            initial_context,
            Uuid::new_v4(),
        )
        .await?;
    
    tracing::info!(
        "Started laboratory processing saga {} for sample {}",
        saga_id,
        barcode
    );
    
    Ok(saga_id)
}

// Integration with Event Store and CQRS
pub async fn query_sample_processing_status(
    query_handler: Arc<dyn QueryHandler<GetSagaStatusQuery>>,
    saga_id: Uuid,
) -> Result<SagaStatusReadModel, QueryError> {
    let query = GetSagaStatusQuery { saga_id };
    query_handler.handle(query).await
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GetSagaStatusQuery {
    pub saga_id: Uuid,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SagaStatusReadModel {
    pub saga_id: Uuid,
    pub status: String,
    pub current_step: Option<String>,
    pub completed_steps: Vec<String>,
    pub started_at: DateTime<Utc>,
    pub completed_at: Option<DateTime<Utc>>,
    pub sample_id: Option<Uuid>,
    pub barcode: Option<String>,
    pub storage_location: Option<String>,
}

use crate::queries::{QueryHandler, QueryError};
use crate::saga_orchestrator::SagaOrchestrator;