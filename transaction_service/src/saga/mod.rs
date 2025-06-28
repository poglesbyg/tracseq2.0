//! Saga pattern implementation for distributed transactions in TracSeq.

pub mod step;
pub mod compensation;
pub mod state;
pub mod error;

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

pub use error::SagaError;
pub use step::{SagaStep, StepResult};
pub use compensation::CompensationStep;
pub use state::{SagaState, SagaStatus};

/// Transaction context for saga execution
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransactionContext {
    pub transaction_id: Uuid,
    pub user_id: Option<Uuid>,
    pub correlation_id: Option<Uuid>,
    pub metadata: HashMap<String, serde_json::Value>,
    pub timeout_ms: u64,
    pub created_at: DateTime<Utc>,
}

impl TransactionContext {
    pub fn new() -> Self {
        Self {
            transaction_id: Uuid::new_v4(),
            user_id: None,
            correlation_id: None,
            metadata: HashMap::new(),
            timeout_ms: 30000,
            created_at: Utc::now(),
        }
    }

    pub fn with_user_id(mut self, user_id: Uuid) -> Self {
        self.user_id = Some(user_id);
        self
    }

    pub fn with_correlation_id(mut self, correlation_id: Uuid) -> Self {
        self.correlation_id = Some(correlation_id);
        self
    }

    pub fn with_metadata(mut self, key: String, value: serde_json::Value) -> Self {
        self.metadata.insert(key, value);
        self
    }
}

impl Default for TransactionContext {
    fn default() -> Self {
        Self::new()
    }
}

/// Core saga transaction coordinator
#[derive(Debug)]
pub struct TransactionSaga {
    pub id: Uuid,
    pub name: String,
    pub state: SagaState,
    pub steps: Vec<Box<dyn SagaStep>>,
    pub compensation_steps: Vec<Box<dyn CompensationStep>>,
    pub context: TransactionContext,
    pub timeout_ms: u64,
    pub max_retries: u32,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Saga execution result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SagaExecutionResult {
    pub saga_id: Uuid,
    pub status: SagaStatus,
    pub completed_steps: u32,
    pub failed_step: Option<String>,
    pub error_message: Option<String>,
    pub execution_time_ms: u64,
    pub compensation_executed: bool,
}

/// Saga builder for constructing complex transactions
#[derive(Debug)]
pub struct SagaBuilder {
    name: String,
    steps: Vec<Box<dyn SagaStep>>,
    compensation_steps: Vec<Box<dyn CompensationStep>>,
    context: TransactionContext,
    timeout_ms: u64,
    max_retries: u32,
}

impl TransactionSaga {
    /// Create a new saga with builder pattern
    pub fn builder(name: &str) -> SagaBuilder {
        SagaBuilder::new(name)
    }

    /// Execute the saga with all its steps
    pub async fn execute(&mut self) -> Result<SagaExecutionResult, SagaError> {
        let start_time = Utc::now();
        self.state.status = SagaStatus::Executing;
        self.updated_at = Utc::now();

        let mut step_index = 0;
        for step in &self.steps {
            match self.execute_step(step, step_index).await {
                Ok(_) => {
                    step_index += 1;
                    self.state.completed_steps = step_index as u32;
                }
                Err(error) => {
                    let failed_step_name = step.name().to_string();
                    self.state.status = SagaStatus::Compensating;
                    self.state.failed_step = Some(failed_step_name.clone());
                    
                    let compensation_result = self.execute_compensation().await;
                    
                    let execution_time = (Utc::now() - start_time).num_milliseconds() as u64;
                    
                    return Ok(SagaExecutionResult {
                        saga_id: self.id,
                        status: if compensation_result.is_ok() { 
                            SagaStatus::Compensated 
                        } else { 
                            SagaStatus::Failed 
                        },
                        completed_steps: step_index as u32,
                        failed_step: Some(failed_step_name),
                        error_message: Some(error.to_string()),
                        execution_time_ms: execution_time,
                        compensation_executed: true,
                    });
                }
            }
        }

        self.state.status = SagaStatus::Completed;
        self.updated_at = Utc::now();
        
        let execution_time = (Utc::now() - start_time).num_milliseconds() as u64;

        Ok(SagaExecutionResult {
            saga_id: self.id,
            status: SagaStatus::Completed,
            completed_steps: self.steps.len() as u32,
            failed_step: None,
            error_message: None,
            execution_time_ms: execution_time,
            compensation_executed: false,
        })
    }

    async fn execute_step(&self, step: &Box<dyn SagaStep>, step_index: usize) -> Result<StepResult, SagaError> {
        let mut attempts = 0;
        let mut last_error = None;

        while attempts < self.max_retries {
            match step.execute(&self.context).await {
                Ok(result) => {
                    return Ok(result);
                }
                Err(error) => {
                    attempts += 1;
                    last_error = Some(error.clone());
                    
                    if attempts < self.max_retries {
                        let delay_ms = 100 * (2_u64.pow(attempts - 1));
                        tokio::time::sleep(tokio::time::Duration::from_millis(delay_ms)).await;
                    }
                }
            }
        }

        Err(last_error.unwrap_or_else(|| {
            SagaError::StepExecutionFailed {
                step_name: step.name().to_string(),
                step_index,
                reason: "Maximum retries exceeded".to_string(),
            }
        }))
    }

    async fn execute_compensation(&mut self) -> Result<(), SagaError> {
        for compensation in self.compensation_steps.iter().rev() {
            if let Err(error) = compensation.compensate(&self.context).await {
                self.state.compensation_errors.push(format!("{}: {}", 
                                                           compensation.name(), error));
            }
        }

        if self.state.compensation_errors.is_empty() {
            Ok(())
        } else {
            Err(SagaError::CompensationFailed {
                saga_id: self.id,
                errors: self.state.compensation_errors.clone(),
            })
        }
    }

    pub fn status(&self) -> SagaStatus {
        self.state.status.clone()
    }

    pub fn progress(&self) -> f64 {
        if self.steps.is_empty() {
            return 1.0;
        }
        self.state.completed_steps as f64 / self.steps.len() as f64
    }
}

impl SagaBuilder {
    pub fn new(name: &str) -> Self {
        Self {
            name: name.to_string(),
            steps: Vec::new(),
            compensation_steps: Vec::new(),
            context: TransactionContext::new(),
            timeout_ms: 30000,
            max_retries: 3,
        }
    }

    pub fn add_step<S: SagaStep + 'static>(mut self, step: S) -> Self {
        self.steps.push(Box::new(step));
        self
    }

    pub fn add_compensation<C: CompensationStep + 'static>(mut self, compensation: C) -> Self {
        self.compensation_steps.push(Box::new(compensation));
        self
    }

    pub fn with_context(mut self, context: TransactionContext) -> Self {
        self.context = context;
        self
    }

    pub fn with_timeout(mut self, timeout_ms: u64) -> Self {
        self.timeout_ms = timeout_ms;
        self
    }

    pub fn with_max_retries(mut self, max_retries: u32) -> Self {
        self.max_retries = max_retries;
        self
    }

    pub fn build(self) -> TransactionSaga {
        let saga_id = Uuid::new_v4();
        let now = Utc::now();

        TransactionSaga {
            id: saga_id,
            name: self.name,
            state: SagaState::new(saga_id),
            steps: self.steps,
            compensation_steps: self.compensation_steps,
            context: self.context,
            timeout_ms: self.timeout_ms,
            max_retries: self.max_retries,
            created_at: now,
            updated_at: now,
        }
    }
}
