//! TracSeq Event Service Library
//! 
//! This library provides event-driven communication capabilities for TracSeq microservices.
//! It includes event definitions, publication, subscription, and processing capabilities.

pub mod events;
pub mod handlers;
pub mod services;

// Re-exports for convenient access
pub use events::{Event, EventContext, EventFilter, EventHandler, EventPublicationResult, SubscriptionConfig};
pub use services::client::EventServiceClient;
pub use services::event_bus::{RedisEventBus, EventBus, EventBusStats};

// Application state
#[derive(Clone)]
pub struct AppState {
    pub event_bus: std::sync::Arc<dyn EventBus>,
    pub redis_url: String,
}

// Configuration for the event service
#[derive(Debug, Clone)]
pub struct Config {
    pub redis_url: String,
    pub max_events_per_batch: usize,
    pub event_retention_hours: u32,
    pub enable_metrics: bool,
}

impl Config {
    pub fn new(redis_url: String) -> Self {
        Self {
            redis_url,
            max_events_per_batch: 100,
            event_retention_hours: 24,
            enable_metrics: true,
        }
    }

    pub fn test_config() -> Self {
        Self {
            redis_url: "redis://localhost:6379/1".to_string(), // Use test database
            max_events_per_batch: 10,
            event_retention_hours: 1,
            enable_metrics: false,
        }
    }
}

// Test utilities
#[cfg(test)]
pub mod test_utils {
    use super::*;
    use anyhow::Result;
    use std::sync::Arc;
    use tokio::sync::OnceCell;

    static TEST_EVENT_BUS: OnceCell<RedisEventBus> = OnceCell::const_new();

    /// Get or create a test event bus instance
    pub async fn get_test_event_bus() -> Result<&'static RedisEventBus> {
        TEST_EVENT_BUS.get_or_try_init(|| async {
            let config = Config::test_config();
            RedisEventBus::new(&config.redis_url).await
        }).await
    }

    /// Create a test application state
    pub async fn create_test_app_state() -> Result<AppState> {
        let event_bus = get_test_event_bus().await?;
        let config = Config::test_config();
        
        Ok(AppState {
            event_bus: Arc::new(event_bus.clone()),
            redis_url: config.redis_url,
        })
    }

    /// Create a test event service client
    pub fn create_test_client() -> EventServiceClient {
        EventServiceClient::new("http://localhost:8087", "test-service")
    }

    /// Clean up test data from Redis
    pub async fn cleanup_test_events() -> Result<()> {
        use redis::AsyncCommands;
        
        let config = Config::test_config();
        let client = redis::Client::open(config.redis_url)?;
        let mut conn = client.get_async_connection().await?;
        
        // Get all stream keys that match our pattern
        let keys: Vec<String> = conn.keys("tracseq:events:*").await?;
        
        if !keys.is_empty() {
            let _: () = conn.del(&keys).await?;
        }
        
        Ok(())
    }

    /// Create a test event
    pub fn create_test_event(event_type: &str, source_service: &str) -> Event {
        Event::new(
            event_type.to_string(),
            source_service.to_string(),
            serde_json::json!({"test": "data"}),
        )
    }

    /// Create a test subscription config
    pub fn create_test_subscription_config(name: &str, event_types: Vec<&str>) -> SubscriptionConfig {
        SubscriptionConfig {
            name: name.to_string(),
            event_types: event_types.iter().map(|s| s.to_string()).collect(),
            consumer_group: format!("{}-group", name),
            consumer_name: format!("{}-consumer", name),
            batch_size: 5,
            timeout_ms: 1000,
            auto_ack: true,
            read_latest: true,
        }
    }
}
