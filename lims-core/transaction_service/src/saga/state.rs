//! Saga state management for tracking transaction status and progress.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Saga execution status
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum SagaStatus {
    /// Saga has been created but not started
    Created,

    /// Saga is currently executing steps
    Executing,

    /// Saga is executing compensation steps due to failure
    Compensating,

    /// Saga completed successfully
    Completed,

    /// Saga was compensated successfully after failure
    Compensated,

    /// Saga failed and compensation also failed
    Failed,

    /// Saga was paused (manual intervention)
    Paused,

    /// Saga was cancelled by user
    Cancelled,

    /// Saga timed out
    TimedOut,
}

impl SagaStatus {
    /// Check if saga is in a terminal state
    pub fn is_terminal(&self) -> bool {
        matches!(
            self,
            SagaStatus::Completed
                | SagaStatus::Compensated
                | SagaStatus::Failed
                | SagaStatus::Cancelled
                | SagaStatus::TimedOut
        )
    }

    /// Check if saga is currently active
    pub fn is_active(&self) -> bool {
        matches!(self, SagaStatus::Executing | SagaStatus::Compensating)
    }

    /// Check if saga can be resumed
    pub fn can_resume(&self) -> bool {
        matches!(self, SagaStatus::Paused)
    }

    /// Check if saga can be cancelled
    pub fn can_cancel(&self) -> bool {
        matches!(
            self,
            SagaStatus::Created | SagaStatus::Executing | SagaStatus::Paused
        )
    }

    /// Get status priority for ordering (lower = higher priority)
    pub fn priority(&self) -> u8 {
        match self {
            SagaStatus::Failed => 0,
            SagaStatus::TimedOut => 1,
            SagaStatus::Compensating => 2,
            SagaStatus::Executing => 3,
            SagaStatus::Paused => 4,
            SagaStatus::Created => 5,
            SagaStatus::Cancelled => 6,
            SagaStatus::Compensated => 7,
            SagaStatus::Completed => 8,
        }
    }
}

impl std::fmt::Display for SagaStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SagaStatus::Created => write!(f, "Created"),
            SagaStatus::Executing => write!(f, "Executing"),
            SagaStatus::Compensating => write!(f, "Compensating"),
            SagaStatus::Completed => write!(f, "Completed"),
            SagaStatus::Compensated => write!(f, "Compensated"),
            SagaStatus::Failed => write!(f, "Failed"),
            SagaStatus::Paused => write!(f, "Paused"),
            SagaStatus::Cancelled => write!(f, "Cancelled"),
            SagaStatus::TimedOut => write!(f, "TimedOut"),
        }
    }
}

/// Comprehensive saga state tracking
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SagaState {
    /// Saga identifier
    pub saga_id: Uuid,

    /// Current execution status
    pub status: SagaStatus,

    /// Number of completed steps
    pub completed_steps: u32,

    /// Total number of steps
    pub total_steps: u32,

    /// Name of the step that failed (if any)
    pub failed_step: Option<String>,

    /// Current step being executed
    pub current_step: Option<String>,

    /// Execution start time
    pub started_at: Option<DateTime<Utc>>,

    /// Execution completion time
    pub completed_at: Option<DateTime<Utc>>,

    /// Last update timestamp
    pub updated_at: DateTime<Utc>,

    /// Compensation execution errors
    pub compensation_errors: Vec<String>,

    /// Execution metrics
    pub metrics: SagaMetrics,

    /// Checkpoints for recovery
    pub checkpoints: Vec<SagaCheckpoint>,

    /// Custom state data
    pub custom_data: std::collections::HashMap<String, serde_json::Value>,
}

/// Saga execution metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SagaMetrics {
    /// Total execution time in milliseconds
    pub execution_time_ms: u64,

    /// Time spent in compensation in milliseconds
    pub compensation_time_ms: u64,

    /// Number of retry attempts
    pub retry_attempts: u32,

    /// Number of successful steps
    pub successful_steps: u32,

    /// Number of failed steps
    pub failed_steps: u32,

    /// Average step execution time
    pub avg_step_time_ms: f64,

    /// Maximum step execution time
    pub max_step_time_ms: u64,

    /// Minimum step execution time
    pub min_step_time_ms: u64,
}

impl Default for SagaMetrics {
    fn default() -> Self {
        Self {
            execution_time_ms: 0,
            compensation_time_ms: 0,
            retry_attempts: 0,
            successful_steps: 0,
            failed_steps: 0,
            avg_step_time_ms: 0.0,
            max_step_time_ms: 0,
            min_step_time_ms: u64::MAX,
        }
    }
}

/// Saga checkpoint for recovery
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SagaCheckpoint {
    /// Checkpoint identifier
    pub id: Uuid,

    /// Step index at checkpoint
    pub step_index: u32,

    /// Checkpoint timestamp
    pub timestamp: DateTime<Utc>,

    /// Saga state at checkpoint
    pub state_snapshot: serde_json::Value,

    /// Checkpoint type
    pub checkpoint_type: CheckpointType,
}

/// Types of checkpoints
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CheckpointType {
    /// Automatic checkpoint before step execution
    BeforeStep,

    /// Automatic checkpoint after step completion
    AfterStep,

    /// Manual checkpoint created by user
    Manual,

    /// Checkpoint before compensation
    BeforeCompensation,

    /// Checkpoint for critical operations
    Critical,
}

impl SagaState {
    /// Create a new saga state
    pub fn new(saga_id: Uuid) -> Self {
        Self {
            saga_id,
            status: SagaStatus::Created,
            completed_steps: 0,
            total_steps: 0,
            failed_step: None,
            current_step: None,
            started_at: None,
            completed_at: None,
            updated_at: Utc::now(),
            compensation_errors: Vec::new(),
            metrics: SagaMetrics::default(),
            checkpoints: Vec::new(),
            custom_data: std::collections::HashMap::new(),
        }
    }

    /// Start saga execution
    pub fn start_execution(&mut self, total_steps: u32) {
        self.status = SagaStatus::Executing;
        self.total_steps = total_steps;
        self.started_at = Some(Utc::now());
        self.updated_at = Utc::now();
    }

    /// Mark saga as completed
    pub fn complete(&mut self) {
        self.status = SagaStatus::Completed;
        self.completed_at = Some(Utc::now());
        self.updated_at = Utc::now();
        self.current_step = None;

        // Update metrics
        if let Some(start_time) = self.started_at {
            self.metrics.execution_time_ms = (Utc::now() - start_time).num_milliseconds() as u64;
        }
    }

    /// Mark saga as failed and start compensation
    pub fn start_compensation(&mut self, failed_step: String) {
        self.status = SagaStatus::Compensating;
        self.failed_step = Some(failed_step);
        self.updated_at = Utc::now();
        self.metrics.failed_steps += 1;
    }

    /// Mark compensation as completed
    pub fn complete_compensation(&mut self) {
        self.status = SagaStatus::Compensated;
        self.completed_at = Some(Utc::now());
        self.updated_at = Utc::now();
        self.current_step = None;
    }

    /// Mark saga as failed
    pub fn mark_failed(&mut self) {
        self.status = SagaStatus::Failed;
        self.completed_at = Some(Utc::now());
        self.updated_at = Utc::now();
        self.current_step = None;
    }

    /// Update step progress
    pub fn update_step_progress(&mut self, step_name: String, completed: bool) {
        self.current_step = if completed { None } else { Some(step_name) };

        if completed {
            self.completed_steps += 1;
            self.metrics.successful_steps += 1;
        }

        self.updated_at = Utc::now();
    }

    /// Add a checkpoint
    pub fn add_checkpoint(
        &mut self,
        checkpoint_type: CheckpointType,
        state_snapshot: serde_json::Value,
    ) {
        let checkpoint = SagaCheckpoint {
            id: Uuid::new_v4(),
            step_index: self.completed_steps,
            timestamp: Utc::now(),
            state_snapshot,
            checkpoint_type,
        };

        self.checkpoints.push(checkpoint);
        self.updated_at = Utc::now();
    }

    /// Get progress percentage
    pub fn progress_percentage(&self) -> f64 {
        if self.total_steps == 0 {
            return 100.0;
        }
        (self.completed_steps as f64 / self.total_steps as f64) * 100.0
    }

    /// Get execution duration
    pub fn execution_duration(&self) -> Option<chrono::Duration> {
        self.started_at.map(|start| {
            let end = self.completed_at.unwrap_or_else(Utc::now);
            end - start
        })
    }

    /// Check if saga can transition to new status
    pub fn can_transition_to(&self, new_status: &SagaStatus) -> bool {
        use SagaStatus::*;

        match (&self.status, new_status) {
            (Created, Executing) => true,
            (Created, Cancelled) => true,
            (Executing, Compensating) => true,
            (Executing, Completed) => true,
            (Executing, Paused) => true,
            (Executing, TimedOut) => true,
            (Compensating, Compensated) => true,
            (Compensating, Failed) => true,
            (Paused, Executing) => true,
            (Paused, Cancelled) => true,
            _ => false,
        }
    }

    /// Update metrics with step timing
    pub fn update_step_metrics(&mut self, step_time_ms: u64) {
        self.metrics.avg_step_time_ms = if self.metrics.successful_steps > 0 {
            ((self.metrics.avg_step_time_ms * (self.metrics.successful_steps - 1) as f64)
                + step_time_ms as f64)
                / self.metrics.successful_steps as f64
        } else {
            step_time_ms as f64
        };

        self.metrics.max_step_time_ms = self.metrics.max_step_time_ms.max(step_time_ms);
        self.metrics.min_step_time_ms = self.metrics.min_step_time_ms.min(step_time_ms);
    }

    /// Set custom data
    pub fn set_custom_data(&mut self, key: String, value: serde_json::Value) {
        self.custom_data.insert(key, value);
        self.updated_at = Utc::now();
    }

    /// Get custom data
    pub fn get_custom_data(&self, key: &str) -> Option<&serde_json::Value> {
        self.custom_data.get(key)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_saga_status_properties() {
        assert!(SagaStatus::Completed.is_terminal());
        assert!(SagaStatus::Executing.is_active());
        assert!(SagaStatus::Paused.can_resume());
        assert!(SagaStatus::Created.can_cancel());
    }

    #[test]
    fn test_saga_state_transitions() {
        let mut state = SagaState::new(Uuid::new_v4());

        assert!(state.can_transition_to(&SagaStatus::Executing));
        assert!(!state.can_transition_to(&SagaStatus::Completed));

        state.status = SagaStatus::Executing;
        assert!(state.can_transition_to(&SagaStatus::Compensating));
        assert!(state.can_transition_to(&SagaStatus::Completed));
    }

    #[test]
    fn test_progress_calculation() {
        let mut state = SagaState::new(Uuid::new_v4());
        state.total_steps = 10;
        state.completed_steps = 3;

        assert_eq!(state.progress_percentage(), 30.0);
    }

    #[test]
    fn test_checkpoint_creation() {
        let mut state = SagaState::new(Uuid::new_v4());
        let snapshot = serde_json::json!({"step": "test"});

        state.add_checkpoint(CheckpointType::BeforeStep, snapshot);

        assert_eq!(state.checkpoints.len(), 1);
        assert_eq!(state.checkpoints[0].step_index, 0);
    }

    #[test]
    fn test_metrics_update() {
        let mut state = SagaState::new(Uuid::new_v4());

        state.update_step_metrics(100);
        assert_eq!(state.metrics.avg_step_time_ms, 100.0);
        assert_eq!(state.metrics.max_step_time_ms, 100);

        state.metrics.successful_steps = 1;
        state.update_step_metrics(200);
        assert_eq!(state.metrics.avg_step_time_ms, 150.0);
        assert_eq!(state.metrics.max_step_time_ms, 200);
    }
}
