//! Database models for saga persistence.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

use crate::saga::{SagaBuilder, SagaStatus, TransactionContext, TransactionSaga};

/// Database record for saga state
#[derive(Debug, Clone, FromRow, Serialize, Deserialize)]
pub struct SagaRecord {
    pub id: Uuid,
    pub name: String,
    pub status: String, // Will be converted to/from SagaStatus
    pub transaction_id: Uuid,
    pub user_id: Option<Uuid>,
    pub correlation_id: Option<Uuid>,
    pub completed_steps: i32,
    pub total_steps: i32,
    pub current_step: Option<String>,
    pub failed_step: Option<String>,
    pub created_at: DateTime<Utc>,
    pub started_at: Option<DateTime<Utc>>,
    pub completed_at: Option<DateTime<Utc>>,
    pub updated_at: DateTime<Utc>,
    pub timeout_ms: i64,
    pub max_retries: i32,
    pub transaction_context: serde_json::Value,
    pub metadata: serde_json::Value,
    pub custom_data: serde_json::Value,
    pub execution_time_ms: Option<i64>,
    pub retry_attempts: Option<i32>,
    pub error_message: Option<String>,
    pub error_category: Option<String>,
    pub compensation_errors: serde_json::Value,
}

impl SagaRecord {
    /// Convert from TransactionSaga to database record
    pub fn from_saga(saga: &TransactionSaga) -> Self {
        Self {
            id: saga.id,
            name: saga.name.clone(),
            status: saga.state.status.to_string(),
            transaction_id: saga.context.transaction_id,
            user_id: saga.context.user_id,
            correlation_id: saga.context.correlation_id,
            completed_steps: saga.state.completed_steps as i32,
            total_steps: saga.state.total_steps as i32,
            current_step: saga.state.current_step.clone(),
            failed_step: saga.state.failed_step.clone(),
            created_at: saga.created_at,
            started_at: saga.state.started_at,
            completed_at: saga.state.completed_at,
            updated_at: saga.updated_at,
            timeout_ms: saga.timeout_ms as i64,
            max_retries: saga.max_retries as i32,
            transaction_context: serde_json::to_value(&saga.context).unwrap_or_default(),
            metadata: serde_json::Value::Object(serde_json::Map::new()),
            custom_data: serde_json::to_value(&saga.state.custom_data).unwrap_or_default(),
            execution_time_ms: Some(saga.state.metrics.execution_time_ms as i64),
            retry_attempts: Some(saga.state.metrics.retry_attempts as i32),
            error_message: None,
            error_category: None,
            compensation_errors: serde_json::to_value(&saga.state.compensation_errors)
                .unwrap_or_default(),
        }
    }

    /// Convert database record to TransactionSaga
    pub fn to_saga(&self) -> anyhow::Result<TransactionSaga> {
        // Parse context
        let context: TransactionContext = serde_json::from_value(self.transaction_context.clone())?;

        // Parse status
        let status = match self.status.as_str() {
            "Created" => SagaStatus::Created,
            "Executing" => SagaStatus::Executing,
            "Compensating" => SagaStatus::Compensating,
            "Completed" => SagaStatus::Completed,
            "Compensated" => SagaStatus::Compensated,
            "Failed" => SagaStatus::Failed,
            "Paused" => SagaStatus::Paused,
            "Cancelled" => SagaStatus::Cancelled,
            "TimedOut" => SagaStatus::TimedOut,
            _ => SagaStatus::Created,
        };

        // Create saga using builder pattern
        let mut saga = SagaBuilder::new(&self.name)
            .with_context(context)
            .with_timeout(self.timeout_ms as u64)
            .with_max_retries(self.max_retries as u32)
            .build();

        // Update reconstructed saga state
        saga.id = self.id;
        saga.state.status = status;
        saga.state.completed_steps = self.completed_steps as u32;
        saga.state.total_steps = self.total_steps as u32;
        saga.state.current_step = self.current_step.clone();
        saga.state.failed_step = self.failed_step.clone();
        saga.state.started_at = self.started_at;
        saga.state.completed_at = self.completed_at;
        saga.state.updated_at = self.updated_at;
        saga.created_at = self.created_at;
        saga.updated_at = self.updated_at;

        // Parse custom data
        if let Ok(custom_data) = serde_json::from_value(self.custom_data.clone()) {
            saga.state.custom_data = custom_data;
        }

        // Parse compensation errors
        if let Ok(errors) = serde_json::from_value(self.compensation_errors.clone()) {
            saga.state.compensation_errors = errors;
        }

        Ok(saga)
    }
}

/// Simplified saga status for queries
#[derive(Debug, Clone, FromRow, Serialize, Deserialize)]
pub struct SagaStatusRecord {
    pub id: Uuid,
    pub name: String,
    pub status: String,
    pub transaction_id: Uuid,
    pub user_id: Option<Uuid>,
    pub completed_steps: i32,
    pub total_steps: i32,
    pub current_step: Option<String>,
    pub created_at: DateTime<Utc>,
    pub started_at: Option<DateTime<Utc>>,
    pub updated_at: DateTime<Utc>,
    pub progress_percentage: Option<f64>,
    pub execution_time_ms: Option<i64>,
}

/// Database record for step execution
#[derive(Debug, Clone, FromRow, Serialize, Deserialize)]
pub struct SagaStepRecord {
    pub id: Uuid,
    pub saga_id: Uuid,
    pub step_name: String,
    pub step_index: i32,
    pub status: String,
    pub execution_id: Uuid,
    pub started_at: Option<DateTime<Utc>>,
    pub completed_at: Option<DateTime<Utc>>,
    pub retry_count: i32,
    pub input_data: serde_json::Value,
    pub output_data: serde_json::Value,
    pub step_metadata: serde_json::Value,
    pub error_message: Option<String>,
    pub execution_duration_ms: Option<i64>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Database record for compensation execution
#[derive(Debug, Clone, FromRow, Serialize, Deserialize)]
pub struct SagaCompensationRecord {
    pub id: Uuid,
    pub saga_id: Uuid,
    pub step_name: String,
    pub step_index: i32,
    pub status: String,
    pub execution_id: Uuid,
    pub started_at: Option<DateTime<Utc>>,
    pub completed_at: Option<DateTime<Utc>>,
    pub retry_count: i32,
    pub input_data: serde_json::Value,
    pub output_data: serde_json::Value,
    pub compensation_metadata: serde_json::Value,
    pub error_message: Option<String>,
    pub skip_reason: Option<String>,
    pub execution_duration_ms: Option<i64>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Database record for saga checkpoints
#[derive(Debug, Clone, FromRow, Serialize, Deserialize)]
pub struct SagaCheckpointRecord {
    pub id: Uuid,
    pub saga_id: Uuid,
    pub checkpoint_type: String,
    pub step_index: i32,
    pub state_snapshot: serde_json::Value,
    pub checkpoint_metadata: serde_json::Value,
    pub created_at: DateTime<Utc>,
}

/// Database record for saga events (audit trail)
#[derive(Debug, Clone, FromRow, Serialize, Deserialize)]
pub struct SagaEventRecord {
    pub id: Uuid,
    pub saga_id: Uuid,
    pub event_type: String,
    pub event_data: serde_json::Value,
    pub event_source: Option<String>,
    pub correlation_id: Option<Uuid>,
    pub created_at: DateTime<Utc>,
}
