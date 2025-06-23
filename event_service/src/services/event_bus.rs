//! Event Bus implementation using Redis Streams for TracSeq microservices.

use crate::events::{Event, EventContext, EventHandler, EventPublicationResult, SubscriptionConfig};
use anyhow::{Context, Result};
use async_trait::async_trait;
use chrono::Utc;
use deadpool_redis::{Config, Pool, Runtime};
use redis::{streams::StreamReadOptions, AsyncCommands, RedisResult};
use serde_json;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{debug, error, info, warn};
use uuid::Uuid;

/// Redis-based event bus implementation
#[derive(Clone)]
pub struct RedisEventBus {
    /// Redis connection pool
    pool: Pool,
    
    /// Registered event handlers
    handlers: Arc<RwLock<HashMap<String, Arc<dyn EventHandler>>>>,
    
    /// Event publication statistics
    stats: Arc<RwLock<EventBusStats>>,
}

/// Event bus statistics
#[derive(Debug, Default, Clone)]
pub struct EventBusStats {
    pub events_published: u64,
    pub events_consumed: u64,
    pub events_failed: u64,
    pub handlers_registered: u64,
}

impl RedisEventBus {
    /// Create a new Redis event bus
    pub async fn new(redis_url: &str) -> Result<Self> {
        let config = Config::from_url(redis_url);
        let pool = config
            .create_pool(Some(Runtime::Tokio1))
            .context("Failed to create Redis connection pool")?;

        // Test connection
        let mut conn = pool.get().await.context("Failed to get Redis connection")?;
        let _: String = redis::cmd("PING")
            .query_async(&mut conn)
            .await
            .context("Failed to ping Redis")?;

        info!("Successfully connected to Redis at {}", redis_url);

        Ok(Self {
            pool,
            handlers: Arc::new(RwLock::new(HashMap::new())),
            stats: Arc::new(RwLock::new(EventBusStats::default())),
        })
    }

    /// Publish an event to the event bus
    pub async fn publish(&self, event: Event) -> Result<EventPublicationResult> {
        let mut conn = self.pool.get().await.context("Failed to get Redis connection")?;

        let stream_name = event.stream_name();
        let event_json = serde_json::to_string(&event).context("Failed to serialize event")?;

        // Publish to Redis Stream
        let stream_id: String = conn
            .xadd(
                &stream_name,
                "*", // Let Redis generate the ID
                &[("event", &event_json)],
            )
            .await
            .context("Failed to publish event to Redis Stream")?;

        // Update statistics
        {
            let mut stats = self.stats.write().await;
            stats.events_published += 1;
        }

        debug!(
            "Published event {} to stream {} with ID {}",
            event.id, stream_name, stream_id
        );

        Ok(EventPublicationResult {
            event_id: event.id,
            stream_id,
            published_at: Utc::now(),
        })
    }

    /// Register an event handler
    pub async fn register_handler(&self, handler: Arc<dyn EventHandler>) -> Result<()> {
        let handler_name = handler.name();
        let event_types = handler.event_types();

        {
            let mut handlers = self.handlers.write().await;
            handlers.insert(handler_name.clone(), handler);
        }

        {
            let mut stats = self.stats.write().await;
            stats.handlers_registered += 1;
        }

        info!(
            "Registered event handler '{}' for event types: {:?}",
            handler_name, event_types
        );

        Ok(())
    }

    /// Subscribe to events with the given configuration
    pub async fn subscribe(&self, config: SubscriptionConfig) -> Result<()> {
        // Create consumer group for each event type
        for event_type in &config.event_types {
            let stream_name = format!("tracseq:events:{}", event_type.replace('.', ":"));
            
            if let Err(e) = self.create_consumer_group(&stream_name, &config.consumer_group).await {
                debug!("Consumer group creation result: {}", e);
            }
        }

        // Start consumer task
        self.start_consumer(config).await?;

        Ok(())
    }

    /// Start a consumer task for processing events
    async fn start_consumer(&self, config: SubscriptionConfig) -> Result<()> {
        let bus = self.clone();

        tokio::spawn(async move {
            if let Err(e) = bus.consume_events(config).await {
                error!("Consumer task failed: {}", e);
            }
        });

        Ok(())
    }

    /// Consume events from Redis Streams
    async fn consume_events(&self, config: SubscriptionConfig) -> Result<()> {
        let mut conn = self.pool.get().await.context("Failed to get Redis connection")?;

        loop {
            // Build stream names from event types
            let stream_names: Vec<String> = config
                .event_types
                .iter()
                .map(|event_type| format!("tracseq:events:{}", event_type.replace('.', ":")))
                .collect();

            // Read from multiple streams
            let opts = StreamReadOptions::default()
                .group(&config.consumer_group, &config.consumer_name)
                .count(config.batch_size)
                .block(config.timeout_ms as usize);

            let streams_result: RedisResult<HashMap<String, Vec<HashMap<String, HashMap<String, String>>>>> = 
                conn.xread_options(&stream_names, &vec![">"; stream_names.len()], &opts).await;

            match streams_result {
                Ok(streams) => {
                    for (stream_name, messages) in streams {
                        for message in messages {
                            for (message_id, fields) in message {
                                if let Some(event_json) = fields.get("event") {
                                    match self.process_event(&stream_name, &message_id, event_json, &config).await {
                                        Ok(_) => {
                                            // Acknowledge the message
                                            if config.auto_ack {
                                                let _: RedisResult<u64> = conn
                                                    .xack(&stream_name, &config.consumer_group, &[&message_id])
                                                    .await;
                                            }
                                        }
                                        Err(e) => {
                                            error!("Failed to process event {}: {}", message_id, e);
                                            
                                            let mut stats = self.stats.write().await;
                                            stats.events_failed += 1;
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
                Err(e) => {
                    warn!("Failed to read from streams: {}", e);
                    tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
                }
            }
        }
    }

    /// Process a single event
    async fn process_event(
        &self,
        _stream_name: &str,
        message_id: &str,
        event_json: &str,
        config: &SubscriptionConfig,
    ) -> Result<()> {
        // Deserialize event
        let event: Event = serde_json::from_str(event_json)
            .context("Failed to deserialize event")?;

        // Create event context
        let context = EventContext {
            event: event.clone(),
            delivery_count: 1,
            subscription: config.name.clone(),
            stream_id: message_id.to_string(),
        };

        // Find appropriate handlers
        let handlers = self.handlers.read().await;
        let mut processed = false;

        for handler in handlers.values() {
            // Check if handler can process this event type
            if handler.event_types().iter().any(|pattern| self.matches_pattern(pattern, &event.event_type)) {
                match handler.handle(context.clone()).await {
                    Ok(_) => {
                        debug!(
                            "Successfully processed event {} with handler {}",
                            event.id,
                            handler.name()
                        );
                        processed = true;
                    }
                    Err(e) => {
                        error!(
                            "Handler {} failed to process event {}: {}",
                            handler.name(),
                            event.id,
                            e
                        );
                    }
                }
            }
        }

        if processed {
            let mut stats = self.stats.write().await;
            stats.events_consumed += 1;
        }

        Ok(())
    }

    /// Create a consumer group for a stream
    async fn create_consumer_group(&self, stream_name: &str, group_name: &str) -> Result<()> {
        let mut conn = self.pool.get().await.context("Failed to get Redis connection")?;

        let _: RedisResult<String> = conn
            .xgroup_create(stream_name, group_name, "0")
            .await;

        Ok(())
    }

    /// Check if event type matches pattern
    fn matches_pattern(&self, pattern: &str, event_type: &str) -> bool {
        if pattern == "*" {
            return true;
        }

        if pattern.contains('*') {
            let regex_pattern = pattern.replace('*', ".*");
            if let Ok(regex) = regex::Regex::new(&format!("^{}$", regex_pattern)) {
                return regex.is_match(event_type);
            }
        }

        pattern == event_type
    }

    /// Get event bus statistics
    pub async fn get_stats(&self) -> EventBusStats {
        self.stats.read().await.clone()
    }
}

/// Event bus trait for dependency injection
#[async_trait]
pub trait EventBus: Send + Sync {
    async fn publish(&self, event: Event) -> Result<EventPublicationResult>;
    async fn register_handler(&self, handler: Arc<dyn EventHandler>) -> Result<()>;
    async fn subscribe(&self, config: SubscriptionConfig) -> Result<()>;
    async fn get_stats(&self) -> EventBusStats;
}

#[async_trait]
impl EventBus for RedisEventBus {
    async fn publish(&self, event: Event) -> Result<EventPublicationResult> {
        self.publish(event).await
    }

    async fn register_handler(&self, handler: Arc<dyn EventHandler>) -> Result<()> {
        self.register_handler(handler).await
    }

    async fn subscribe(&self, config: SubscriptionConfig) -> Result<()> {
        self.subscribe(config).await
    }

    async fn get_stats(&self) -> EventBusStats {
        self.get_stats().await
    }
}
