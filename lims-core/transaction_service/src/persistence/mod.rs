//! Database persistence layer for distributed transaction saga state.

pub mod models;
pub mod repository;

use anyhow::Result;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::{Pool, Postgres};
use uuid::Uuid;

use crate::saga::{SagaState, SagaStatus, TransactionSaga};

pub use models::*;
pub use repository::SagaRepository;

/// Database configuration for saga persistence
#[derive(Debug, Clone)]
pub struct DatabaseConfig {
    pub connection_string: String,
    pub max_connections: u32,
    pub min_connections: u32,
    pub connection_timeout_seconds: u64,
}

impl Default for DatabaseConfig {
    fn default() -> Self {
        Self {
            connection_string: "postgresql://tracseq:tracseq@localhost:5432/tracseq_transactions"
                .to_string(),
            max_connections: 20,
            min_connections: 5,
            connection_timeout_seconds: 30,
        }
    }
}

/// Main database service for saga persistence
#[derive(Clone)]
pub struct SagaPersistenceService {
    pool: Pool<Postgres>,
    repository: SagaRepository,
}

impl SagaPersistenceService {
    /// Create a new persistence service with database connection
    pub async fn new(config: DatabaseConfig) -> Result<Self> {
        let pool = sqlx::postgres::PgPoolOptions::new()
            .max_connections(config.max_connections)
            .min_connections(config.min_connections)
            .acquire_timeout(std::time::Duration::from_secs(
                config.connection_timeout_seconds,
            ))
            .connect(&config.connection_string)
            .await?;

        // Run migrations
        sqlx::migrate!("./migrations").run(&pool).await?;

        let repository = SagaRepository::new(pool.clone());

        Ok(Self { pool, repository })
    }

    /// Save a new saga to the database
    pub async fn save_saga(&self, saga: &TransactionSaga) -> Result<()> {
        let saga_record = SagaRecord::from_saga(saga);
        self.repository.insert_saga(saga_record).await
    }

    /// Update saga state in the database
    pub async fn update_saga(&self, saga: &TransactionSaga) -> Result<()> {
        let saga_record = SagaRecord::from_saga(saga);
        self.repository.update_saga(saga_record).await
    }

    /// Load saga from database by ID
    pub async fn load_saga(&self, saga_id: Uuid) -> Result<Option<TransactionSaga>> {
        if let Some(saga_record) = self.repository.get_saga_by_id(saga_id).await? {
            let saga = saga_record.to_saga()?;
            Ok(Some(saga))
        } else {
            Ok(None)
        }
    }

    /// Get saga status without full reconstruction
    pub async fn get_saga_status(&self, saga_id: Uuid) -> Result<Option<SagaStatusRecord>> {
        self.repository.get_saga_status(saga_id).await
    }

    /// List active sagas
    pub async fn list_active_sagas(&self) -> Result<Vec<SagaStatusRecord>> {
        self.repository.list_active_sagas().await
    }

    /// Cleanup old completed sagas
    pub async fn cleanup_old_sagas(&self, older_than_hours: i32) -> Result<u64> {
        self.repository.cleanup_old_sagas(older_than_hours).await
    }

    /// Get database statistics for monitoring
    pub async fn get_statistics(&self) -> Result<DatabaseStatistics> {
        let active_sagas = self.repository.count_active_sagas().await? as u64;
        let completed_sagas = self.repository.count_completed_sagas().await? as u64;
        let failed_sagas = self.repository.count_failed_sagas().await? as u64;
        let total_sagas = self.repository.count_total_sagas().await? as u64;

        Ok(DatabaseStatistics {
            active_sagas,
            completed_sagas,
            failed_sagas,
            total_sagas,
        })
    }

    /// Perform database health check
    pub async fn health_check(&self) -> Result<DatabaseHealth> {
        let start_time = std::time::Instant::now();

        let result = sqlx::query("SELECT 1 as test").fetch_one(&self.pool).await;

        let response_time_ms = start_time.elapsed().as_millis() as u64;

        match result {
            Ok(_) => {
                let pool_status = self.pool.size();
                let idle_connections = self.pool.num_idle();

                Ok(DatabaseHealth {
                    is_connected: true,
                    response_time_ms,
                    active_connections: pool_status.saturating_sub(idle_connections as u32),
                    idle_connections: idle_connections as u32,
                    max_connections: pool_status,
                    error_message: None,
                })
            }
            Err(e) => Ok(DatabaseHealth {
                is_connected: false,
                response_time_ms,
                active_connections: 0,
                idle_connections: 0,
                max_connections: 0,
                error_message: Some(e.to_string()),
            }),
        }
    }
}

/// Database health information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DatabaseHealth {
    pub is_connected: bool,
    pub response_time_ms: u64,
    pub active_connections: u32,
    pub idle_connections: u32,
    pub max_connections: u32,
    pub error_message: Option<String>,
}

/// Database statistics for monitoring
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DatabaseStatistics {
    pub active_sagas: u64,
    pub completed_sagas: u64,
    pub failed_sagas: u64,
    pub total_sagas: u64,
}
