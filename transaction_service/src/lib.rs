pub mod coordinator;
pub mod handlers;
pub mod models;
#[cfg(feature = "database-persistence")]
pub mod persistence;
pub mod saga;
pub mod services;
pub mod workflows;

// Re-exports for convenient testing access
pub use coordinator::{TransactionCoordinator, TransactionRequest, TransactionStatus, CoordinatorConfig};
pub use models::*;
pub use saga::{TransactionSaga, SagaError, SagaExecutionResult, SagaStatus, SagaStep};
pub use services::*;

// Application state for testing
#[derive(Clone)]
pub struct AppState {
    pub coordinator: std::sync::Arc<coordinator::TransactionCoordinator>,
    pub config: std::sync::Arc<CoordinatorConfig>,
}

// Test utilities
#[cfg(test)]
pub mod test_utils {
    use super::*;
    use tokio::sync::OnceCell;
    use std::sync::Arc;

    static TEST_COORDINATOR: OnceCell<coordinator::TransactionCoordinator> = OnceCell::const_new();

    pub async fn get_test_coordinator() -> &'static coordinator::TransactionCoordinator {
        TEST_COORDINATOR.get_or_init(|| async {
            let config = CoordinatorConfig::test_config();
            coordinator::TransactionCoordinator::new(config)
        }).await
    }

    pub async fn create_test_app_state() -> AppState {
        let coordinator = get_test_coordinator().await;
        let config = CoordinatorConfig::test_config();
        
        AppState {
            coordinator: Arc::new(coordinator.clone()),
            config: Arc::new(config),
        }
    }
}

impl CoordinatorConfig {
    pub fn test_config() -> Self {
        Self {
            max_concurrent_sagas: 50,
            default_timeout_ms: 30000, // 30 seconds for testing
            enable_events: false, // Disable for unit tests
            event_service_url: "http://localhost:8087".to_string(),
            enable_persistence: false, // Disable for unit tests
            #[cfg(feature = "database-persistence")]
            database: crate::persistence::DatabaseConfig::test_config(),
            #[cfg(not(feature = "database-persistence"))]
            database: (),
            cleanup_after_hours: 1,
        }
    }
} 
