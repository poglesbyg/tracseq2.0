// TracSeq 2.0 - CQRS Command Handler
// Command processing with event sourcing integration

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use uuid::Uuid;

use crate::event_store::{Event, EventStore, EventMetadata, LaboratoryEvent};

#[async_trait]
pub trait Command: Send + Sync {
    type Result: Send;
    type Error: std::error::Error + Send;
}

#[async_trait]
pub trait CommandHandler<C: Command>: Send + Sync {
    async fn handle(&self, command: C) -> Result<C::Result, C::Error>;
}

#[derive(Debug)]
pub struct CommandBus {
    event_store: Arc<EventStore>,
}

impl CommandBus {
    pub fn new(event_store: Arc<EventStore>) -> Self {
        Self { event_store }
    }
}

// Sample Management Commands
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateSampleCommand {
    pub sample_id: Uuid,
    pub barcode: String,
    pub sample_type: String,
    pub patient_id: Option<Uuid>,
    pub volume_ml: f32,
    pub collection_date: DateTime<Utc>,
    pub metadata: CommandMetadata,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidateSampleCommand {
    pub sample_id: Uuid,
    pub validation_type: String,
    pub validation_data: serde_json::Value,
    pub validated_by: Uuid,
    pub metadata: CommandMetadata,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StoreSampleCommand {
    pub sample_id: Uuid,
    pub location_id: Uuid,
    pub position: String,
    pub temperature: f32,
    pub stored_by: Uuid,
    pub metadata: CommandMetadata,
}

// Sequencing Commands
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StartSequencingCommand {
    pub sequencing_id: Uuid,
    pub sample_id: Uuid,
    pub protocol: String,
    pub machine_id: String,
    pub operator_id: Uuid,
    pub metadata: CommandMetadata,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompleteSequencingCommand {
    pub sequencing_id: Uuid,
    pub results_url: String,
    pub quality_score: f32,
    pub read_count: i64,
    pub metadata: CommandMetadata,
}

// Command Metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommandMetadata {
    pub user_id: Uuid,
    pub correlation_id: Uuid,
    pub causation_id: Option<Uuid>,
    pub tenant_id: Option<String>,
    pub ip_address: Option<String>,
}

// Command Results
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommandResult {
    pub success: bool,
    pub aggregate_id: Uuid,
    pub version: i32,
    pub events: Vec<Uuid>,
    pub timestamp: DateTime<Utc>,
}

// Command Handlers Implementation
pub struct SampleCommandHandler {
    event_store: Arc<EventStore>,
}

impl SampleCommandHandler {
    pub fn new(event_store: Arc<EventStore>) -> Self {
        Self { event_store }
    }
}

#[async_trait]
impl CommandHandler<CreateSampleCommand> for SampleCommandHandler {
    async fn handle(
        &self,
        command: CreateSampleCommand,
    ) -> Result<CommandResult, CommandError> {
        // Validate command
        self.validate_create_sample(&command)?;

        // Create event
        let event = Event {
            id: Uuid::new_v4(),
            aggregate_id: command.sample_id,
            aggregate_type: "Sample".to_string(),
            event_type: "SampleCreated".to_string(),
            event_version: 1,
            event_data: serde_json::to_value(LaboratoryEvent::SampleCreated {
                sample_id: command.sample_id,
                barcode: command.barcode.clone(),
                sample_type: command.sample_type.clone(),
                patient_id: command.patient_id,
            })?,
            metadata: EventMetadata {
                user_id: Some(command.metadata.user_id),
                correlation_id: command.metadata.correlation_id,
                causation_id: command.metadata.causation_id,
                tenant_id: command.metadata.tenant_id,
                ip_address: command.metadata.ip_address,
                user_agent: None,
            },
            created_at: Utc::now(),
            sequence_number: 0, // Will be set by database
        };

        // Append to event store
        self.event_store
            .append_events(vec![event.clone()], None)
            .await?;

        Ok(CommandResult {
            success: true,
            aggregate_id: command.sample_id,
            version: 1,
            events: vec![event.id],
            timestamp: event.created_at,
        })
    }
}

#[async_trait]
impl CommandHandler<ValidateSampleCommand> for SampleCommandHandler {
    async fn handle(
        &self,
        command: ValidateSampleCommand,
    ) -> Result<CommandResult, CommandError> {
        // Get current version
        let events = self
            .event_store
            .get_events(command.sample_id, None)
            .await?;
        
        let current_version = events.len() as i32;

        // Create validation event
        let event = Event {
            id: Uuid::new_v4(),
            aggregate_id: command.sample_id,
            aggregate_type: "Sample".to_string(),
            event_type: "SampleValidated".to_string(),
            event_version: current_version + 1,
            event_data: serde_json::to_value(LaboratoryEvent::SampleValidated {
                sample_id: command.sample_id,
                validation_results: command.validation_data.clone(),
                validated_by: command.validated_by,
            })?,
            metadata: EventMetadata {
                user_id: Some(command.metadata.user_id),
                correlation_id: command.metadata.correlation_id,
                causation_id: command.metadata.causation_id,
                tenant_id: command.metadata.tenant_id,
                ip_address: command.metadata.ip_address,
                user_agent: None,
            },
            created_at: Utc::now(),
            sequence_number: 0,
        };

        // Append event with optimistic concurrency control
        self.event_store
            .append_events(vec![event.clone()], Some(current_version))
            .await?;

        Ok(CommandResult {
            success: true,
            aggregate_id: command.sample_id,
            version: current_version + 1,
            events: vec![event.id],
            timestamp: event.created_at,
        })
    }
}

impl SampleCommandHandler {
    fn validate_create_sample(&self, command: &CreateSampleCommand) -> Result<(), CommandError> {
        // Validate barcode format
        if command.barcode.is_empty() || command.barcode.len() > 50 {
            return Err(CommandError::ValidationError(
                "Invalid barcode format".to_string(),
            ));
        }

        // Validate sample type
        let valid_types = ["blood", "tissue", "dna", "rna", "plasma", "serum"];
        if !valid_types.contains(&command.sample_type.as_str()) {
            return Err(CommandError::ValidationError(
                "Invalid sample type".to_string(),
            ));
        }

        // Validate volume
        if command.volume_ml <= 0.0 || command.volume_ml > 1000.0 {
            return Err(CommandError::ValidationError(
                "Invalid sample volume".to_string(),
            ));
        }

        Ok(())
    }
}

// Sequencing Command Handler
pub struct SequencingCommandHandler {
    event_store: Arc<EventStore>,
}

impl SequencingCommandHandler {
    pub fn new(event_store: Arc<EventStore>) -> Self {
        Self { event_store }
    }
}

#[async_trait]
impl CommandHandler<StartSequencingCommand> for SequencingCommandHandler {
    async fn handle(
        &self,
        command: StartSequencingCommand,
    ) -> Result<CommandResult, CommandError> {
        let event = Event {
            id: Uuid::new_v4(),
            aggregate_id: command.sequencing_id,
            aggregate_type: "Sequencing".to_string(),
            event_type: "SequencingStarted".to_string(),
            event_version: 1,
            event_data: serde_json::to_value(LaboratoryEvent::SequencingStarted {
                sequencing_id: command.sequencing_id,
                sample_id: command.sample_id,
                protocol: command.protocol.clone(),
                machine_id: command.machine_id.clone(),
            })?,
            metadata: EventMetadata {
                user_id: Some(command.metadata.user_id),
                correlation_id: command.metadata.correlation_id,
                causation_id: command.metadata.causation_id,
                tenant_id: command.metadata.tenant_id,
                ip_address: command.metadata.ip_address,
                user_agent: None,
            },
            created_at: Utc::now(),
            sequence_number: 0,
        };

        self.event_store
            .append_events(vec![event.clone()], None)
            .await?;

        Ok(CommandResult {
            success: true,
            aggregate_id: command.sequencing_id,
            version: 1,
            events: vec![event.id],
            timestamp: event.created_at,
        })
    }
}

// Command Error Types
#[derive(Debug, thiserror::Error)]
pub enum CommandError {
    #[error("Validation error: {0}")]
    ValidationError(String),
    #[error("Aggregate not found: {0}")]
    AggregateNotFound(Uuid),
    #[error("Concurrency conflict: {0}")]
    ConcurrencyConflict(String),
    #[error("Event store error: {0}")]
    EventStore(#[from] crate::event_store::EventStoreError),
    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),
}

// Command Validation Middleware
pub struct CommandValidationMiddleware<C: Command> {
    inner: Box<dyn CommandHandler<C>>,
}

impl<C: Command> CommandValidationMiddleware<C> {
    pub fn new(handler: Box<dyn CommandHandler<C>>) -> Self {
        Self { inner: handler }
    }
}

#[async_trait]
impl<C: Command> CommandHandler<C> for CommandValidationMiddleware<C> {
    async fn handle(&self, command: C) -> Result<C::Result, C::Error> {
        // Add common validation logic here
        self.inner.handle(command).await
    }
}