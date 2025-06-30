//! Database repository for saga operations.

use anyhow::Result;
use sqlx::{Pool, Postgres, Row};
use uuid::Uuid;

use super::models::*;
use crate::saga::step::StepStatus;

/// Repository for saga database operations
#[derive(Clone)]
pub struct SagaRepository {
    pool: Pool<Postgres>,
}

impl SagaRepository {
    /// Create a new repository with database pool
    pub fn new(pool: Pool<Postgres>) -> Self {
        Self { pool }
    }

    /// Insert a new saga record
    pub async fn insert_saga(&self, saga: SagaRecord) -> Result<()> {
        sqlx::query!(
            r#"
            INSERT INTO sagas (
                id, name, status, transaction_id, user_id, correlation_id,
                completed_steps, total_steps, current_step, failed_step,
                created_at, started_at, completed_at, updated_at,
                timeout_ms, max_retries, transaction_context, metadata, custom_data,
                execution_time_ms, retry_attempts, error_message, error_category,
                compensation_errors
            ) VALUES (
                $1, $2, $3::saga_status, $4, $5, $6,
                $7, $8, $9, $10,
                $11, $12, $13, $14,
                $15, $16, $17, $18, $19,
                $20, $21, $22, $23,
                $24
            )
            "#,
            saga.id,
            saga.name,
            saga.status,
            saga.transaction_id,
            saga.user_id,
            saga.correlation_id,
            saga.completed_steps,
            saga.total_steps,
            saga.current_step,
            saga.failed_step,
            saga.created_at,
            saga.started_at,
            saga.completed_at,
            saga.updated_at,
            saga.timeout_ms,
            saga.max_retries,
            saga.transaction_context,
            saga.metadata,
            saga.custom_data,
            saga.execution_time_ms,
            saga.retry_attempts,
            saga.error_message,
            saga.error_category,
            saga.compensation_errors
        )
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    /// Update an existing saga record
    pub async fn update_saga(&self, saga: SagaRecord) -> Result<()> {
        sqlx::query!(
            r#"
            UPDATE sagas SET
                status = $2::saga_status,
                completed_steps = $3,
                current_step = $4,
                failed_step = $5,
                started_at = $6,
                completed_at = $7,
                updated_at = $8,
                execution_time_ms = $9,
                retry_attempts = $10,
                error_message = $11,
                error_category = $12,
                custom_data = $13,
                compensation_errors = $14
            WHERE id = $1
            "#,
            saga.id,
            saga.status,
            saga.completed_steps,
            saga.current_step,
            saga.failed_step,
            saga.started_at,
            saga.completed_at,
            saga.updated_at,
            saga.execution_time_ms,
            saga.retry_attempts,
            saga.error_message,
            saga.error_category,
            saga.custom_data,
            saga.compensation_errors
        )
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    /// Get saga by ID
    pub async fn get_saga_by_id(&self, saga_id: Uuid) -> Result<Option<SagaRecord>> {
        let record = sqlx::query_as!(
            SagaRecord,
            r#"
            SELECT 
                id, name, status as "status!", transaction_id, user_id, correlation_id,
                completed_steps, total_steps, current_step, failed_step,
                created_at, started_at, completed_at, updated_at,
                timeout_ms, max_retries, transaction_context, metadata, custom_data,
                execution_time_ms, retry_attempts, error_message, error_category,
                compensation_errors
            FROM sagas WHERE id = $1
            "#,
            saga_id
        )
        .fetch_optional(&self.pool)
        .await?;

        Ok(record)
    }

    /// Get saga status only
    pub async fn get_saga_status(&self, saga_id: Uuid) -> Result<Option<SagaStatusRecord>> {
        let record = sqlx::query_as!(
            SagaStatusRecord,
            r#"
            SELECT 
                id, name, status as "status!", transaction_id, user_id,
                completed_steps, total_steps, current_step,
                created_at, started_at, updated_at,
                CASE 
                    WHEN total_steps > 0 THEN (completed_steps::FLOAT / total_steps::FLOAT) * 100
                    ELSE 0
                END as progress_percentage,
                execution_time_ms
            FROM sagas WHERE id = $1
            "#,
            saga_id
        )
        .fetch_optional(&self.pool)
        .await?;

        Ok(record)
    }

    /// List all active sagas
    pub async fn list_active_sagas(&self) -> Result<Vec<SagaStatusRecord>> {
        let records = sqlx::query_as!(
            SagaStatusRecord,
            r#"
            SELECT 
                id, name, status as "status!", transaction_id, user_id,
                completed_steps, total_steps, current_step,
                created_at, started_at, updated_at,
                CASE 
                    WHEN total_steps > 0 THEN (completed_steps::FLOAT / total_steps::FLOAT) * 100
                    ELSE 0
                END as progress_percentage,
                execution_time_ms
            FROM active_sagas
            ORDER BY created_at DESC
            "#
        )
        .fetch_all(&self.pool)
        .await?;

        Ok(records)
    }

    /// Insert saga step record
    pub async fn insert_saga_step(&self, step: SagaStepRecord) -> Result<()> {
        sqlx::query!(
            r#"
            INSERT INTO saga_steps (
                id, saga_id, step_name, step_index, status, execution_id,
                started_at, completed_at, retry_count, input_data, output_data,
                step_metadata, error_message, execution_duration_ms,
                created_at, updated_at
            ) VALUES (
                $1, $2, $3, $4, $5::step_status, $6,
                $7, $8, $9, $10, $11,
                $12, $13, $14,
                $15, $16
            )
            "#,
            step.id,
            step.saga_id,
            step.step_name,
            step.step_index,
            step.status,
            step.execution_id,
            step.started_at,
            step.completed_at,
            step.retry_count,
            step.input_data,
            step.output_data,
            step.step_metadata,
            step.error_message,
            step.execution_duration_ms,
            step.created_at,
            step.updated_at
        )
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    /// Update step status
    pub async fn update_step_status(&self, saga_id: Uuid, step_index: usize, status: StepStatus) -> Result<()> {
        let status_str = match status {
            StepStatus::Pending => "Pending",
            StepStatus::Executing => "Executing",
            StepStatus::Completed => "Completed",
            StepStatus::Failed => "Failed",
            StepStatus::Skipped => "Skipped",
            StepStatus::Retrying => "Retrying",
        };

        sqlx::query!(
            r#"
            UPDATE saga_steps SET
                status = $3::step_status,
                updated_at = NOW()
            WHERE saga_id = $1 AND step_index = $2
            "#,
            saga_id,
            step_index as i32,
            status_str
        )
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    /// Get saga steps
    pub async fn get_saga_steps(&self, saga_id: Uuid) -> Result<Vec<SagaStepRecord>> {
        let records = sqlx::query_as!(
            SagaStepRecord,
            r#"
            SELECT 
                id, saga_id, step_name, step_index, status as "status!", execution_id,
                started_at, completed_at, retry_count, input_data, output_data,
                step_metadata, error_message, execution_duration_ms,
                created_at, updated_at
            FROM saga_steps 
            WHERE saga_id = $1 
            ORDER BY step_index
            "#,
            saga_id
        )
        .fetch_all(&self.pool)
        .await?;

        Ok(records)
    }

    /// Insert saga compensation record
    pub async fn insert_saga_compensation(&self, compensation: SagaCompensationRecord) -> Result<()> {
        sqlx::query!(
            r#"
            INSERT INTO saga_compensations (
                id, saga_id, step_name, step_index, status, execution_id,
                started_at, completed_at, retry_count, input_data, output_data,
                compensation_metadata, error_message, skip_reason, execution_duration_ms,
                created_at, updated_at
            ) VALUES (
                $1, $2, $3, $4, $5::compensation_status, $6,
                $7, $8, $9, $10, $11,
                $12, $13, $14, $15,
                $16, $17
            )
            "#,
            compensation.id,
            compensation.saga_id,
            compensation.step_name,
            compensation.step_index,
            compensation.status,
            compensation.execution_id,
            compensation.started_at,
            compensation.completed_at,
            compensation.retry_count,
            compensation.input_data,
            compensation.output_data,
            compensation.compensation_metadata,
            compensation.error_message,
            compensation.skip_reason,
            compensation.execution_duration_ms,
            compensation.created_at,
            compensation.updated_at
        )
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    /// Get saga compensations
    pub async fn get_saga_compensations(&self, saga_id: Uuid) -> Result<Vec<SagaCompensationRecord>> {
        let records = sqlx::query_as!(
            SagaCompensationRecord,
            r#"
            SELECT 
                id, saga_id, step_name, step_index, status as "status!", execution_id,
                started_at, completed_at, retry_count, input_data, output_data,
                compensation_metadata, error_message, skip_reason, execution_duration_ms,
                created_at, updated_at
            FROM saga_compensations 
            WHERE saga_id = $1 
            ORDER BY step_index DESC
            "#,
            saga_id
        )
        .fetch_all(&self.pool)
        .await?;

        Ok(records)
    }

    /// Insert saga checkpoint
    pub async fn insert_saga_checkpoint(&self, checkpoint: SagaCheckpointRecord) -> Result<()> {
        sqlx::query!(
            r#"
            INSERT INTO saga_checkpoints (
                id, saga_id, checkpoint_type, step_index, 
                state_snapshot, checkpoint_metadata, created_at
            ) VALUES (
                $1, $2, $3, $4, $5, $6, $7
            )
            "#,
            checkpoint.id,
            checkpoint.saga_id,
            checkpoint.checkpoint_type,
            checkpoint.step_index,
            checkpoint.state_snapshot,
            checkpoint.checkpoint_metadata,
            checkpoint.created_at
        )
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    /// Get saga checkpoints
    pub async fn get_saga_checkpoints(&self, saga_id: Uuid) -> Result<Vec<SagaCheckpointRecord>> {
        let records = sqlx::query_as!(
            SagaCheckpointRecord,
            r#"
            SELECT 
                id, saga_id, checkpoint_type, step_index,
                state_snapshot, checkpoint_metadata, created_at
            FROM saga_checkpoints 
            WHERE saga_id = $1 
            ORDER BY created_at
            "#,
            saga_id
        )
        .fetch_all(&self.pool)
        .await?;

        Ok(records)
    }

    /// Insert saga event
    pub async fn insert_saga_event(&self, event: SagaEventRecord) -> Result<()> {
        sqlx::query!(
            r#"
            INSERT INTO saga_events (
                id, saga_id, event_type, event_data,
                event_source, correlation_id, created_at
            ) VALUES (
                $1, $2, $3, $4, $5, $6, $7
            )
            "#,
            event.id,
            event.saga_id,
            event.event_type,
            event.event_data,
            event.event_source,
            event.correlation_id,
            event.created_at
        )
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    /// Get saga events
    pub async fn get_saga_events(&self, saga_id: Uuid) -> Result<Vec<SagaEventRecord>> {
        let records = sqlx::query_as!(
            SagaEventRecord,
            r#"
            SELECT 
                id, saga_id, event_type, event_data,
                event_source, correlation_id, created_at
            FROM saga_events 
            WHERE saga_id = $1 
            ORDER BY created_at
            "#,
            saga_id
        )
        .fetch_all(&self.pool)
        .await?;

        Ok(records)
    }

    /// Cleanup old completed sagas
    pub async fn cleanup_old_sagas(&self, older_than_hours: i32) -> Result<u64> {
        let result = sqlx::query!(
            r#"
            SELECT cleanup_old_sagas($1) as deleted_count
            "#,
            older_than_hours
        )
        .fetch_one(&self.pool)
        .await?;

        Ok(result.deleted_count.unwrap_or(0) as u64)
    }

    /// Count total sagas
    pub async fn count_total_sagas(&self) -> Result<i64> {
        let result = sqlx::query!("SELECT COUNT(*) as count FROM sagas")
            .fetch_one(&self.pool)
            .await?;

        Ok(result.count.unwrap_or(0))
    }

    /// Count active sagas
    pub async fn count_active_sagas(&self) -> Result<i64> {
        let result = sqlx::query!(
            r#"
            SELECT COUNT(*) as count FROM sagas 
            WHERE status IN ('Created', 'Executing', 'Compensating', 'Paused')
            "#
        )
        .fetch_one(&self.pool)
        .await?;

        Ok(result.count.unwrap_or(0))
    }

    /// Count completed sagas
    pub async fn count_completed_sagas(&self) -> Result<i64> {
        let result = sqlx::query!(
            r#"
            SELECT COUNT(*) as count FROM sagas 
            WHERE status IN ('Completed', 'Compensated')
            "#
        )
        .fetch_one(&self.pool)
        .await?;

        Ok(result.count.unwrap_or(0))
    }

    /// Count failed sagas
    pub async fn count_failed_sagas(&self) -> Result<i64> {
        let result = sqlx::query!(
            r#"
            SELECT COUNT(*) as count FROM sagas 
            WHERE status IN ('Failed', 'Cancelled', 'TimedOut')
            "#
        )
        .fetch_one(&self.pool)
        .await?;

        Ok(result.count.unwrap_or(0))
    }
}
