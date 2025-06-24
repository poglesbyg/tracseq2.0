//! TracSeq Event Service Library
//! 
//! This library provides event-driven communication capabilities for TracSeq microservices.
//! It includes event definitions, publication, subscription, and processing capabilities.

pub mod events;
pub mod handlers;
pub mod models;
pub mod services;

// Re-exports for convenient testing access
pub use events::{Event, EventContext, EventFilter, EventHandler, EventPublicationResult, SubscriptionConfig};
pub use models::*;
pub use services::*;

// Application state for testing
#[derive(Clone)]
pub struct AppState {
    pub event_service: std::sync::Arc<services::EventService>,
    pub config: std::sync::Arc<Config>,
}

// Configuration
#[derive(Debug, Clone)]
pub struct Config {
    pub redis_url: String,
    pub max_events_per_batch: usize,
    pub event_retention_hours: u32,
    pub enable_metrics: bool,
}

impl Config {
    pub fn test_config() -> Self {
        Self {
            redis_url: "redis://localhost:6379/1".to_string(), // Use test database
            max_events_per_batch: 100,
            event_retention_hours: 24,
            enable_metrics: true,
        }
    }
}

// Test utilities
#[cfg(test)]
pub mod test_utils {
    use super::*;
    use tokio::sync::OnceCell;
    use std::sync::Arc;

    static TEST_EVENT_SERVICE: OnceCell<services::EventService> = OnceCell::const_new();

    pub async fn get_test_event_service() -> &'static services::EventService {
        TEST_EVENT_SERVICE.get_or_init(|| async {
            let config = Config::test_config();
            services::EventService::new(config).await
                .expect("Failed to create test event service")
        }).await
    }

    pub async fn create_test_app_state() -> AppState {
        let event_service = get_test_event_service().await;
        let config = Config::test_config();
        
        AppState {
            event_service: Arc::new(event_service.clone()),
            config: Arc::new(config),
        }
    }

    pub async fn cleanup_test_events() {
        if let Ok(mut conn) = redis::Client::open("redis://localhost:6379/1")
            .and_then(|client| client.get_connection())
        {
            let _: Result<(), redis::RedisError> = redis::cmd("FLUSHDB").query(&mut conn);
        }
    }
}
