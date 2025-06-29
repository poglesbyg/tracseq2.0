// TracSeq 2.0 - Enhanced Saga Orchestrator
// Advanced distributed transaction management with compensation

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use uuid::Uuid;

use crate::event_store::{Event, EventStore};
use crate::kafka::{KafkaEventProducer, EventEnvelope};

// Saga Definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SagaDefinition {
    pub saga_type: String,
    pub steps: Vec<SagaStep>,
    pub timeout_seconds: u64,
    pub retry_policy: RetryPolicy,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SagaStep {
    pub name: String,
    pub service: String,
    pub command: String,
    pub compensation_command: Option<String>,
    pub timeout_seconds: u64,
    pub can_retry: bool,
    pub depends_on: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RetryPolicy {
    pub max_retries: u32,
    pub backoff_seconds: u64,
    pub exponential_backoff: bool,
}

// Saga State
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SagaState {
    pub saga_id: Uuid,
    pub saga_type: String,
    pub status: SagaStatus,
    pub current_step: Option<String>,
    pub completed_steps: Vec<String>,
    pub failed_step: Option<String>,
    pub context: serde_json::Value,
    pub correlation_id: Uuid,
    pub started_at: DateTime<Utc>,
    pub completed_at: Option<DateTime<Utc>>,
    pub retry_count: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum SagaStatus {
    Running,
    Completed,
    Failed,
    Compensating,
    Compensated,
    TimedOut,
}

// Saga Orchestrator
pub struct SagaOrchestrator {
    definitions: Arc<RwLock<HashMap<String, SagaDefinition>>>,
    active_sagas: Arc<RwLock<HashMap<Uuid, SagaState>>>,
    event_store: Arc<EventStore>,
    kafka_producer: Arc<KafkaEventProducer>,
    step_handlers: Arc<RwLock<HashMap<String, Box<dyn StepHandler>>>>,
}

#[async_trait]
pub trait StepHandler: Send + Sync {
    async fn execute(&self, context: &serde_json::Value) -> Result<serde_json::Value, SagaError>;
    async fn compensate(&self, context: &serde_json::Value) -> Result<(), SagaError>;
}

impl SagaOrchestrator {
    pub fn new(
        event_store: Arc<EventStore>,
        kafka_producer: Arc<KafkaEventProducer>,
    ) -> Self {
        Self {
            definitions: Arc::new(RwLock::new(HashMap::new())),
            active_sagas: Arc::new(RwLock::new(HashMap::new())),
            event_store,
            kafka_producer,
            step_handlers: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    pub async fn register_saga_definition(&self, definition: SagaDefinition) {
        let mut definitions = self.definitions.write().await;
        definitions.insert(definition.saga_type.clone(), definition);
    }

    pub async fn register_step_handler(
        &self,
        step_name: String,
        handler: Box<dyn StepHandler>,
    ) {
        let mut handlers = self.step_handlers.write().await;
        handlers.insert(step_name, handler);
    }

    pub async fn start_saga(
        &self,
        saga_type: String,
        initial_context: serde_json::Value,
        correlation_id: Uuid,
    ) -> Result<Uuid, SagaError> {
        let definitions = self.definitions.read().await;
        let definition = definitions
            .get(&saga_type)
            .ok_or_else(|| SagaError::DefinitionNotFound(saga_type.clone()))?
            .clone();

        let saga_id = Uuid::new_v4();
        let saga_state = SagaState {
            saga_id,
            saga_type: saga_type.clone(),
            status: SagaStatus::Running,
            current_step: None,
            completed_steps: Vec::new(),
            failed_step: None,
            context: initial_context,
            correlation_id,
            started_at: Utc::now(),
            completed_at: None,
            retry_count: 0,
        };

        // Store initial state
        let mut active_sagas = self.active_sagas.write().await;
        active_sagas.insert(saga_id, saga_state.clone());
        drop(active_sagas);

        // Emit SagaStarted event
        self.emit_saga_event(SagaEvent::Started {
            saga_id,
            saga_type,
            correlation_id,
        })
        .await?;

        // Start execution
        tokio::spawn({
            let orchestrator = self.clone();
            async move {
                if let Err(e) = orchestrator.execute_saga(saga_id, definition).await {
                    tracing::error!("Saga {} execution failed: {:?}", saga_id, e);
                }
            }
        });

        Ok(saga_id)
    }

    async fn execute_saga(
        &self,
        saga_id: Uuid,
        definition: SagaDefinition,
    ) -> Result<(), SagaError> {
        for step in &definition.steps {
            if !self.can_execute_step(saga_id, step).await? {
                continue;
            }

            match self.execute_step(saga_id, step).await {
                Ok(result) => {
                    self.update_saga_context(saga_id, &result).await?;
                    self.mark_step_completed(saga_id, &step.name).await?;
                }
                Err(e) => {
                    tracing::error!("Step {} failed: {:?}", step.name, e);
                    self.handle_step_failure(saga_id, step, e).await?;
                    return Err(SagaError::StepFailed(step.name.clone()));
                }
            }
        }

        self.complete_saga(saga_id).await?;
        Ok(())
    }

    async fn execute_step(
        &self,
        saga_id: Uuid,
        step: &SagaStep,
    ) -> Result<serde_json::Value, SagaError> {
        let sagas = self.active_sagas.read().await;
        let saga_state = sagas
            .get(&saga_id)
            .ok_or(SagaError::SagaNotFound(saga_id))?;
        let context = saga_state.context.clone();
        drop(sagas);

        // Update current step
        let mut sagas = self.active_sagas.write().await;
        if let Some(saga) = sagas.get_mut(&saga_id) {
            saga.current_step = Some(step.name.clone());
        }
        drop(sagas);

        // Execute step with timeout
        let handlers = self.step_handlers.read().await;
        let handler = handlers
            .get(&step.name)
            .ok_or_else(|| SagaError::HandlerNotFound(step.name.clone()))?;

        let result = tokio::time::timeout(
            std::time::Duration::from_secs(step.timeout_seconds),
            handler.execute(&context),
        )
        .await
        .map_err(|_| SagaError::StepTimeout(step.name.clone()))?
        .map_err(|e| SagaError::StepExecutionError(format!("{:?}", e)))?;

        // Emit StepCompleted event
        self.emit_saga_event(SagaEvent::StepCompleted {
            saga_id,
            step_name: step.name.clone(),
            result: result.clone(),
        })
        .await?;

        Ok(result)
    }

    async fn handle_step_failure(
        &self,
        saga_id: Uuid,
        failed_step: &SagaStep,
        error: SagaError,
    ) -> Result<(), SagaError> {
        // Mark step as failed
        let mut sagas = self.active_sagas.write().await;
        if let Some(saga) = sagas.get_mut(&saga_id) {
            saga.failed_step = Some(failed_step.name.clone());
            saga.status = SagaStatus::Failed;
        }
        drop(sagas);

        // Emit StepFailed event
        self.emit_saga_event(SagaEvent::StepFailed {
            saga_id,
            step_name: failed_step.name.clone(),
            error: error.to_string(),
        })
        .await?;

        // Start compensation
        self.start_compensation(saga_id).await?;
        Ok(())
    }

    async fn start_compensation(&self, saga_id: Uuid) -> Result<(), SagaError> {
        let mut sagas = self.active_sagas.write().await;
        if let Some(saga) = sagas.get_mut(&saga_id) {
            saga.status = SagaStatus::Compensating;
        }
        drop(sagas);

        // Emit CompensationStarted event
        self.emit_saga_event(SagaEvent::CompensationStarted { saga_id }).await?;

        // Get completed steps in reverse order
        let sagas = self.active_sagas.read().await;
        let saga = sagas
            .get(&saga_id)
            .ok_or(SagaError::SagaNotFound(saga_id))?;
        let completed_steps = saga.completed_steps.clone();
        let context = saga.context.clone();
        drop(sagas);

        // Compensate in reverse order
        for step_name in completed_steps.iter().rev() {
            if let Err(e) = self.compensate_step(saga_id, step_name, &context).await {
                tracing::error!("Compensation failed for step {}: {:?}", step_name, e);
                // Continue with other compensations
            }
        }

        // Mark as compensated
        let mut sagas = self.active_sagas.write().await;
        if let Some(saga) = sagas.get_mut(&saga_id) {
            saga.status = SagaStatus::Compensated;
            saga.completed_at = Some(Utc::now());
        }
        drop(sagas);

        // Emit CompensationCompleted event
        self.emit_saga_event(SagaEvent::CompensationCompleted { saga_id }).await?;

        Ok(())
    }

    async fn compensate_step(
        &self,
        saga_id: Uuid,
        step_name: &str,
        context: &serde_json::Value,
    ) -> Result<(), SagaError> {
        let handlers = self.step_handlers.read().await;
        let handler = handlers
            .get(step_name)
            .ok_or_else(|| SagaError::HandlerNotFound(step_name.to_string()))?;

        handler.compensate(context).await?;

        // Emit StepCompensated event
        self.emit_saga_event(SagaEvent::StepCompensated {
            saga_id,
            step_name: step_name.to_string(),
        })
        .await?;

        Ok(())
    }

    async fn can_execute_step(&self, saga_id: Uuid, step: &SagaStep) -> Result<bool, SagaError> {
        let sagas = self.active_sagas.read().await;
        let saga = sagas
            .get(&saga_id)
            .ok_or(SagaError::SagaNotFound(saga_id))?;

        // Check if dependencies are completed
        for dep in &step.depends_on {
            if !saga.completed_steps.contains(dep) {
                return Ok(false);
            }
        }

        // Check if already completed
        if saga.completed_steps.contains(&step.name) {
            return Ok(false);
        }

        Ok(true)
    }

    async fn update_saga_context(
        &self,
        saga_id: Uuid,
        new_data: &serde_json::Value,
    ) -> Result<(), SagaError> {
        let mut sagas = self.active_sagas.write().await;
        if let Some(saga) = sagas.get_mut(&saga_id) {
            // Merge new data into context
            if let serde_json::Value::Object(ref mut ctx) = saga.context {
                if let serde_json::Value::Object(new) = new_data {
                    for (k, v) in new {
                        ctx.insert(k.clone(), v.clone());
                    }
                }
            }
        }
        Ok(())
    }

    async fn mark_step_completed(
        &self,
        saga_id: Uuid,
        step_name: &str,
    ) -> Result<(), SagaError> {
        let mut sagas = self.active_sagas.write().await;
        if let Some(saga) = sagas.get_mut(&saga_id) {
            saga.completed_steps.push(step_name.to_string());
            saga.current_step = None;
        }
        Ok(())
    }

    async fn complete_saga(&self, saga_id: Uuid) -> Result<(), SagaError> {
        let mut sagas = self.active_sagas.write().await;
        if let Some(saga) = sagas.get_mut(&saga_id) {
            saga.status = SagaStatus::Completed;
            saga.completed_at = Some(Utc::now());
        }
        drop(sagas);

        // Emit SagaCompleted event
        self.emit_saga_event(SagaEvent::Completed { saga_id }).await?;

        Ok(())
    }

    async fn emit_saga_event(&self, event: SagaEvent) -> Result<(), SagaError> {
        let envelope = EventEnvelope {
            event_id: Uuid::new_v4(),
            event_type: format!("Saga{}", event.event_type()),
            aggregate_id: event.saga_id(),
            aggregate_type: "Saga".to_string(),
            event_version: 1,
            payload: serde_json::to_value(&event)?,
            metadata: crate::kafka::EventMetadata {
                correlation_id: Uuid::new_v4(),
                causation_id: None,
                user_id: None,
                tenant_id: None,
                source_service: "saga-orchestrator".to_string(),
            },
            timestamp: Utc::now(),
        };

        self.kafka_producer
            .publish_event(crate::kafka::Topics::SAGA_EVENTS, envelope)
            .await
            .map_err(|e| SagaError::EventPublishError(e.to_string()))?;

        Ok(())
    }
}

impl Clone for SagaOrchestrator {
    fn clone(&self) -> Self {
        Self {
            definitions: Arc::clone(&self.definitions),
            active_sagas: Arc::clone(&self.active_sagas),
            event_store: Arc::clone(&self.event_store),
            kafka_producer: Arc::clone(&self.kafka_producer),
            step_handlers: Arc::clone(&self.step_handlers),
        }
    }
}

// Saga Events
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum SagaEvent {
    Started {
        saga_id: Uuid,
        saga_type: String,
        correlation_id: Uuid,
    },
    StepCompleted {
        saga_id: Uuid,
        step_name: String,
        result: serde_json::Value,
    },
    StepFailed {
        saga_id: Uuid,
        step_name: String,
        error: String,
    },
    StepCompensated {
        saga_id: Uuid,
        step_name: String,
    },
    CompensationStarted {
        saga_id: Uuid,
    },
    CompensationCompleted {
        saga_id: Uuid,
    },
    Completed {
        saga_id: Uuid,
    },
}

impl SagaEvent {
    fn event_type(&self) -> &'static str {
        match self {
            SagaEvent::Started { .. } => "Started",
            SagaEvent::StepCompleted { .. } => "StepCompleted",
            SagaEvent::StepFailed { .. } => "StepFailed",
            SagaEvent::StepCompensated { .. } => "StepCompensated",
            SagaEvent::CompensationStarted { .. } => "CompensationStarted",
            SagaEvent::CompensationCompleted { .. } => "CompensationCompleted",
            SagaEvent::Completed { .. } => "Completed",
        }
    }

    fn saga_id(&self) -> Uuid {
        match self {
            SagaEvent::Started { saga_id, .. }
            | SagaEvent::StepCompleted { saga_id, .. }
            | SagaEvent::StepFailed { saga_id, .. }
            | SagaEvent::StepCompensated { saga_id, .. }
            | SagaEvent::CompensationStarted { saga_id }
            | SagaEvent::CompensationCompleted { saga_id }
            | SagaEvent::Completed { saga_id } => *saga_id,
        }
    }
}

// Saga Errors
#[derive(Debug, thiserror::Error)]
pub enum SagaError {
    #[error("Saga definition not found: {0}")]
    DefinitionNotFound(String),
    #[error("Saga not found: {0}")]
    SagaNotFound(Uuid),
    #[error("Step handler not found: {0}")]
    HandlerNotFound(String),
    #[error("Step failed: {0}")]
    StepFailed(String),
    #[error("Step timeout: {0}")]
    StepTimeout(String),
    #[error("Step execution error: {0}")]
    StepExecutionError(String),
    #[error("Event publish error: {0}")]
    EventPublishError(String),
    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),
}