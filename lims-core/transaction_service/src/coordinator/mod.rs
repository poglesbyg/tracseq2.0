//! Transaction coordinator for orchestrating distributed transactions across TracSeq services.

#[cfg(feature = "database-persistence")]
use crate::persistence::{DatabaseConfig, SagaPersistenceService};
use crate::saga::{SagaError, SagaExecutionResult, SagaStatus, TransactionSaga};

use anyhow::Result;
use async_trait::async_trait;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use uuid::Uuid;

// Dummy event service client for now
#[derive(Clone)]
struct EventServiceClient;

impl EventServiceClient {
    fn new(_url: &str, _name: &str) -> Self {
        Self
    }
    
    async fn publish_event(&self, event_type: &str, payload: serde_json::Value) -> Result<()> {
        tracing::info!("Event published: {} with payload: {}", event_type, payload);
        Ok(())
    }
}

/// Trait for saga persistence operations
#[async_trait]
pub trait SagaPersistence: Send + Sync {
    async fn save_saga(&self, saga: &TransactionSaga) -> Result<()>;
    async fn update_saga(&self, saga: &TransactionSaga) -> Result<()>;
    async fn get_saga_status(&self, saga_id: Uuid) -> Result<Option<SagaStatusRecord>>;
    async fn list_active_sagas(&self) -> Result<Vec<SagaStatusRecord>>;
    async fn get_statistics(&self) -> Result<PersistenceStatistics>;
    async fn cleanup_old_sagas(&self, cleanup_after_hours: i32) -> Result<u32>;
}

/// Database-backed persistence implementation
#[cfg(feature = "database-persistence")]
pub struct DatabasePersistence {
    service: SagaPersistenceService,
}

#[cfg(feature = "database-persistence")]
#[async_trait]
impl SagaPersistence for DatabasePersistence {
    async fn save_saga(&self, saga: &TransactionSaga) -> Result<()> {
        self.service.save_saga(saga).await
    }

    async fn update_saga(&self, saga: &TransactionSaga) -> Result<()> {
        self.service.update_saga(saga).await
    }

    async fn get_saga_status(&self, saga_id: Uuid) -> Result<Option<SagaStatusRecord>> {
        self.service.get_saga_status(saga_id).await
    }

    async fn list_active_sagas(&self) -> Result<Vec<SagaStatusRecord>> {
        self.service.list_active_sagas().await
    }

    async fn get_statistics(&self) -> Result<PersistenceStatistics> {
        self.service.get_statistics().await
    }

    async fn cleanup_old_sagas(&self, cleanup_after_hours: i32) -> Result<u32> {
        self.service.cleanup_old_sagas(cleanup_after_hours).await
    }
}

/// No-op persistence implementation for when database persistence is disabled
pub struct NoOpPersistence;

#[async_trait]
impl SagaPersistence for NoOpPersistence {
    async fn save_saga(&self, _saga: &TransactionSaga) -> Result<()> {
        Ok(())
    }

    async fn update_saga(&self, _saga: &TransactionSaga) -> Result<()> {
        Ok(())
    }

    async fn get_saga_status(&self, _saga_id: Uuid) -> Result<Option<SagaStatusRecord>> {
        Ok(None)
    }

    async fn list_active_sagas(&self) -> Result<Vec<SagaStatusRecord>> {
        Ok(vec![])
    }

    async fn get_statistics(&self) -> Result<PersistenceStatistics> {
        Ok(PersistenceStatistics {
            total_sagas: 0,
            active_sagas: 0,
            completed_sagas: 0,
            failed_sagas: 0,
        })
    }

    async fn cleanup_old_sagas(&self, _cleanup_after_hours: i32) -> Result<u32> {
        Ok(0)
    }
}

/// Saga status record for persistence queries
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SagaStatusRecord {
    pub id: Uuid,
    pub transaction_id: Uuid,
    pub status: String,
    pub progress_percentage: Option<f64>,
    pub current_step: Option<String>,
    pub completed_steps: i32,
    pub total_steps: i32,
    pub started_at: Option<DateTime<Utc>>,
    pub updated_at: DateTime<Utc>,
}

/// Persistence statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PersistenceStatistics {
    pub total_sagas: i64,
    pub active_sagas: i64,
    pub completed_sagas: i64,
    pub failed_sagas: i64,
}

/// Transaction coordinator that manages saga execution
#[derive(Clone)]
pub struct TransactionCoordinator {
    /// Active sagas being managed (in-memory for performance)
    active_sagas: Arc<RwLock<HashMap<Uuid, Arc<RwLock<TransactionSaga>>>>>,

    /// Persistence service
    persistence: Option<Arc<dyn SagaPersistence>>,

    /// Event client for publishing transaction events
    event_client: Option<Arc<EventServiceClient>>,

    /// Coordinator configuration
    config: CoordinatorConfig,
}

/// Configuration for the transaction coordinator
#[derive(Debug, Clone)]
pub struct CoordinatorConfig {
    /// Maximum number of concurrent sagas
    pub max_concurrent_sagas: usize,

    /// Default saga timeout in milliseconds
    pub default_timeout_ms: u64,

    /// Enable event publishing
    pub enable_events: bool,

    /// Event service URL
    pub event_service_url: String,

    /// Saga persistence enabled
    pub enable_persistence: bool,

    /// Database configuration for persistence
    #[cfg(feature = "database-persistence")]
    pub database: DatabaseConfig,

    #[cfg(not(feature = "database-persistence"))]
    pub database: (),

    /// Cleanup completed sagas after this duration
    pub cleanup_after_hours: u32,
}

impl Default for CoordinatorConfig {
    fn default() -> Self {
        Self {
            max_concurrent_sagas: 100,
            default_timeout_ms: 300000, // 5 minutes
            enable_events: true,
            event_service_url: "http://localhost:8087".to_string(),
            enable_persistence: true,
            #[cfg(feature = "database-persistence")]
            database: DatabaseConfig::default(),
            #[cfg(not(feature = "database-persistence"))]
            database: (),
            cleanup_after_hours: 24,
        }
    }
}

/// Transaction execution request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransactionRequest {
    /// Transaction name/identifier
    pub name: String,

    /// Transaction type
    pub transaction_type: String,

    /// User initiating the transaction
    pub user_id: Option<Uuid>,

    /// Correlation ID for tracing
    pub correlation_id: Option<Uuid>,

    /// Transaction timeout in milliseconds
    pub timeout_ms: Option<u64>,

    /// Transaction metadata
    pub metadata: HashMap<String, serde_json::Value>,

    /// Custom context data
    pub context_data: HashMap<String, serde_json::Value>,
}

/// Transaction status information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransactionStatus {
    /// Transaction ID
    pub transaction_id: Uuid,

    /// Saga ID
    pub saga_id: Uuid,

    /// Current status
    pub status: SagaStatus,

    /// Progress percentage (0-100)
    pub progress: f64,

    /// Current step being executed
    pub current_step: Option<String>,

    /// Completed steps count
    pub completed_steps: u32,

    /// Total steps count
    pub total_steps: u32,

    /// Started timestamp
    pub started_at: Option<DateTime<Utc>>,

    /// Last updated timestamp
    pub updated_at: DateTime<Utc>,

    /// Error message if failed
    pub error_message: Option<String>,
}

/// Coordinator statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CoordinatorStatistics {
    /// Number of active transactions
    pub active_transactions: usize,

    /// Number of completed transactions
    pub completed_transactions: usize,

    /// Number of failed transactions
    pub failed_transactions: usize,

    /// Number of compensated transactions
    pub compensated_transactions: usize,

    /// Total transactions processed
    pub total_transactions: usize,

    /// Average execution time in milliseconds
    pub average_execution_time_ms: f64,
}

impl TransactionCoordinator {
    /// Create a new transaction coordinator
    pub fn new(config: CoordinatorConfig) -> Self {
        let event_client = if config.enable_events {
            Some(Arc::new(
                EventServiceClient::new(
                    &config.event_service_url,
                    "transaction-service",
                ),
            ))
        } else {
            None
        };

        let persistence = if config.enable_persistence {
            Some(Arc::new(NoOpPersistence) as Arc<dyn SagaPersistence>)
        } else {
            None
        };

        Self {
            active_sagas: Arc::new(RwLock::new(HashMap::new())),
            persistence,
            event_client,
            config,
        }
    }

    /// Create a new transaction coordinator with database persistence
    pub async fn with_persistence(config: CoordinatorConfig) -> Result<Self, anyhow::Error> {
        let event_client = if config.enable_events {
            Some(Arc::new(
                EventServiceClient::new(
                    &config.event_service_url,
                    "transaction-service",
                ),
            ))
        } else {
            None
        };

        let persistence = if config.enable_persistence {
            #[cfg(feature = "database-persistence")]
            {
                let persistence_service =
                    SagaPersistenceService::new(config.database.clone()).await?;
                Some(Arc::new(DatabasePersistence {
                    service: persistence_service,
                }) as Arc<dyn SagaPersistence>)
            }
            #[cfg(not(feature = "database-persistence"))]
            {
                Some(Arc::new(NoOpPersistence) as Arc<dyn SagaPersistence>)
            }
        } else {
            None
        };

        Ok(Self {
            active_sagas: Arc::new(RwLock::new(HashMap::new())),
            persistence,
            event_client,
            config,
        })
    }

    /// Execute a transaction using the saga pattern
    pub async fn execute_transaction(
        &self,
        request: TransactionRequest,
        saga: TransactionSaga,
    ) -> Result<SagaExecutionResult, SagaError> {
        // Check if we've reached the maximum number of concurrent sagas
        {
            let active_sagas = self.active_sagas.read().await;
            if active_sagas.len() >= self.config.max_concurrent_sagas {
                return Err(SagaError::Generic {
                    message: "Maximum concurrent sagas limit reached".to_string(),
                });
            }
        }

        let saga_id = saga.id;
        let transaction_id = request.correlation_id.unwrap_or_else(Uuid::new_v4);

        // Publish transaction started event
        if let Some(event_client) = &self.event_client {
            let _ = event_client
                .publish_event(
                    "transaction.started",
                    serde_json::json!({
                        "transaction_id": transaction_id,
                        "saga_id": saga_id,
                        "transaction_type": request.transaction_type,
                        "user_id": request.user_id,
                        "started_at": Utc::now()
                    }),
                )
                .await;
        }

        // Save saga to persistence if enabled
        if let Some(persistence) = &self.persistence {
            if let Err(e) = persistence.save_saga(&saga).await {
                tracing::error!("Failed to save saga to persistence: {}", e);
                // Continue execution even if persistence fails
            }
        }

        // Add saga to active tracking
        let saga_arc = Arc::new(RwLock::new(saga));
        {
            let mut active_sagas = self.active_sagas.write().await;
            active_sagas.insert(saga_id, saga_arc.clone());
        }

        // Execute the saga
        let execution_result = {
            let mut saga_guard = saga_arc.write().await;
            let result = saga_guard.execute().await;

            // Update saga state in persistence during execution
            if let Some(persistence) = &self.persistence {
                if let Err(e) = persistence.update_saga(&*saga_guard).await {
                    tracing::error!("Failed to update saga in persistence: {}", e);
                }
            }

            result
        };

        // Remove from active tracking
        {
            let mut active_sagas = self.active_sagas.write().await;
            active_sagas.remove(&saga_id);
        }

        // Handle execution result
        match &execution_result {
            Ok(result) => {
                // Final saga state update in persistence
                if let Some(persistence) = &self.persistence {
                    if let Ok(saga_arc) = saga_arc.try_read() {
                        if let Err(e) = persistence.update_saga(&*saga_arc).await {
                            tracing::error!(
                                "Failed to update completed saga in persistence: {}",
                                e
                            );
                        }
                    }
                }

                // Publish transaction completed event
                if let Some(event_client) = &self.event_client {
                    let _ = event_client
                        .publish_event(
                            "transaction.completed",
                            serde_json::json!({
                                "transaction_id": transaction_id,
                                "saga_id": saga_id,
                                "status": result.status,
                                "execution_time_ms": result.execution_time_ms,
                                "compensation_executed": result.compensation_executed,
                                "completed_at": Utc::now()
                            }),
                        )
                        .await;
                }
            }
            Err(error) => {
                // Final saga state update in persistence for failed saga
                if let Some(persistence) = &self.persistence {
                    if let Ok(saga_arc) = saga_arc.try_read() {
                        if let Err(e) = persistence.update_saga(&*saga_arc).await {
                            tracing::error!("Failed to update failed saga in persistence: {}", e);
                        }
                    }
                }

                // Publish transaction failed event
                if let Some(event_client) = &self.event_client {
                    let _ = event_client
                        .publish_event(
                            "transaction.failed",
                            serde_json::json!({
                                "transaction_id": transaction_id,
                                "saga_id": saga_id,
                                "error": error.to_string(),
                                "error_category": error.category(),
                                "failed_at": Utc::now()
                            }),
                        )
                        .await;
                }
            }
        }

        execution_result
    }

    /// Get the status of a transaction
    pub async fn get_transaction_status(&self, saga_id: Uuid) -> Option<TransactionStatus> {
        // Check active sagas first (fastest lookup)
        {
            let active_sagas = self.active_sagas.read().await;
            if let Some(saga_arc) = active_sagas.get(&saga_id) {
                let saga = saga_arc.read().await;
                return Some(TransactionStatus {
                    transaction_id: saga.context.transaction_id,
                    saga_id: saga.id,
                    status: saga.state.status.clone(),
                    progress: saga.progress(),
                    current_step: saga.state.current_step.clone(),
                    completed_steps: saga.state.completed_steps,
                    total_steps: saga.state.total_steps,
                    started_at: saga.state.started_at,
                    updated_at: saga.state.updated_at,
                    error_message: None,
                });
            }
        }

        // Check persistence for completed sagas
        if let Some(persistence) = &self.persistence {
            if let Ok(Some(saga_status)) = persistence.get_saga_status(saga_id).await {
                return Some(TransactionStatus {
                    transaction_id: saga_status.transaction_id,
                    saga_id: saga_status.id,
                    status: match saga_status.status.as_str() {
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
                    },
                    progress: saga_status.progress_percentage.unwrap_or(0.0),
                    current_step: saga_status.current_step,
                    completed_steps: saga_status.completed_steps as u32,
                    total_steps: saga_status.total_steps as u32,
                    started_at: saga_status.started_at,
                    updated_at: saga_status.updated_at,
                    error_message: None, // Would need to load full saga for error details
                });
            }
        }

        None
    }

    /// List all active transactions
    pub async fn list_active_transactions(&self) -> Vec<TransactionStatus> {
        let mut transactions = Vec::new();

        // Get in-memory active sagas first
        {
            let active_sagas = self.active_sagas.read().await;
            for saga_arc in active_sagas.values() {
                let saga = saga_arc.read().await;
                transactions.push(TransactionStatus {
                    transaction_id: saga.context.transaction_id,
                    saga_id: saga.id,
                    status: saga.state.status.clone(),
                    progress: saga.progress(),
                    current_step: saga.state.current_step.clone(),
                    completed_steps: saga.state.completed_steps,
                    total_steps: saga.state.total_steps,
                    started_at: saga.state.started_at,
                    updated_at: saga.state.updated_at,
                    error_message: None,
                });
            }
        }

        // Also get active sagas from persistence that might not be in memory
        if let Some(persistence) = &self.persistence {
            if let Ok(active_saga_records) = persistence.list_active_sagas().await {
                for saga_status in active_saga_records {
                    // Skip if already in memory
                    if transactions.iter().any(|t| t.saga_id == saga_status.id) {
                        continue;
                    }

                    transactions.push(TransactionStatus {
                        transaction_id: saga_status.transaction_id,
                        saga_id: saga_status.id,
                        status: match saga_status.status.as_str() {
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
                        },
                        progress: saga_status.progress_percentage.unwrap_or(0.0),
                        current_step: saga_status.current_step,
                        completed_steps: saga_status.completed_steps as u32,
                        total_steps: saga_status.total_steps as u32,
                        started_at: saga_status.started_at,
                        updated_at: saga_status.updated_at,
                        error_message: None,
                    });
                }
            }
        }

        // Sort by creation time (newest first)
        transactions.sort_by(|a, b| b.updated_at.cmp(&a.updated_at));
        transactions
    }

    /// Cancel a transaction by saga ID
    pub async fn cancel_transaction(&self, saga_id: Uuid) -> Result<(), SagaError> {
        // Try to find and cancel the saga in active transactions
        let active_sagas = self.active_sagas.read().await;
        if let Some(saga_arc) = active_sagas.get(&saga_id) {
            let mut saga = saga_arc.write().await;

            // Check if saga can be cancelled
            if saga.state.status.can_cancel() {
                saga.state.status = SagaStatus::Cancelled;
                saga.state.updated_at = Utc::now();
                saga.updated_at = Utc::now();

                // Publish cancellation event
                if let Some(event_client) = &self.event_client {
                    let _ = event_client
                        .publish_event(
                            "transaction.cancelled",
                            serde_json::json!({
                                "transaction_id": saga.context.transaction_id,
                                "saga_id": saga_id,
                                "cancelled_at": Utc::now()
                            }),
                        )
                        .await;
                }

                // Update persistence if available
                if let Some(persistence) = &self.persistence {
                    if let Err(e) = persistence.update_saga(&*saga).await {
                        tracing::error!("Failed to update cancelled saga in persistence: {}", e);
                    }
                }

                return Ok(());
            } else {
                return Err(SagaError::Generic {
                    message: format!(
                        "Saga {} cannot be cancelled in current state: {}",
                        saga_id, saga.state.status
                    ),
                });
            }
        }

        Err(SagaError::Generic {
            message: format!("Saga {} not found", saga_id),
        })
    }

    /// Get coordinator statistics
    pub async fn get_statistics(&self) -> CoordinatorStatistics {
        let active_sagas = self.active_sagas.read().await;
        let in_memory_active = active_sagas.len();

        let mut stats = CoordinatorStatistics {
            active_transactions: in_memory_active,
            completed_transactions: 0,
            failed_transactions: 0,
            compensated_transactions: 0,
            total_transactions: in_memory_active,
            average_execution_time_ms: 0.0,
        };

        // Get statistics from persistence if available
        if let Some(persistence) = &self.persistence {
            if let Ok(db_stats) = persistence.get_statistics().await {
                stats.active_transactions = db_stats.active_sagas as usize;
                stats.completed_transactions = db_stats.completed_sagas as usize;
                stats.failed_transactions = db_stats.failed_sagas as usize;
                stats.total_transactions = db_stats.total_sagas as usize;

                // Compensated transactions would be included in completed count
                // This is a simplification - in a real implementation you'd have separate counts
                stats.compensated_transactions = 0; // Could query specifically for compensated status
            }
        }

        stats
    }

    /// Clean up old completed sagas
    pub async fn cleanup_old_sagas(&self) -> usize {
        if let Some(persistence) = &self.persistence {
            match persistence
                .cleanup_old_sagas(self.config.cleanup_after_hours as i32)
                .await
            {
                Ok(deleted_count) => deleted_count as usize,
                Err(e) => {
                    tracing::error!("Failed to cleanup old sagas: {}", e);
                    0
                }
            }
        } else {
            // No persistence enabled, nothing to cleanup
            0
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::saga::SagaBuilder;

    #[tokio::test]
    async fn test_coordinator_creation() {
        let config = CoordinatorConfig::default();
        let coordinator = TransactionCoordinator::new(config);

        let stats = coordinator.get_statistics().await;
        assert_eq!(stats.active_transactions, 0);
        assert_eq!(stats.total_transactions, 0);
    }

    #[tokio::test]
    async fn test_transaction_request() {
        let request = TransactionRequest {
            name: "test_transaction".to_string(),
            transaction_type: "sample_processing".to_string(),
            user_id: Some(Uuid::new_v4()),
            correlation_id: None,
            timeout_ms: None,
            metadata: HashMap::new(),
            context_data: HashMap::new(),
        };

        assert_eq!(request.name, "test_transaction");
        assert_eq!(request.transaction_type, "sample_processing");
    }

    #[tokio::test]
    async fn test_active_transactions_list() {
        let config = CoordinatorConfig {
            enable_events: false,
            ..Default::default()
        };
        let coordinator = TransactionCoordinator::new(config);

        let transactions = coordinator.list_active_transactions().await;
        assert!(transactions.is_empty());
    }
}
