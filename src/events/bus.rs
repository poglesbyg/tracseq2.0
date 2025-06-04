use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::{broadcast, RwLock};
use uuid::Uuid;

use super::{
    Event, EventError, EventFilter, EventHandler, EventMiddleware, EventPayload, EventStats,
    EventSubscription,
};

/// In-memory event bus for component communication
pub struct EventBus {
    /// Broadcast channel for publishing events
    sender: broadcast::Sender<Arc<EventPayload>>,

    /// Event handlers registry
    handlers: RwLock<HashMap<String, Vec<Arc<dyn EventHandler<EventPayload>>>>>,

    /// Event subscriptions
    subscriptions: RwLock<HashMap<Uuid, EventSubscription>>,

    /// Event middleware
    middleware: RwLock<Vec<Arc<dyn EventMiddleware>>>,

    /// Event statistics
    stats: RwLock<EventStats>,
}

impl EventBus {
    /// Create a new event bus
    pub fn new(capacity: usize) -> Self {
        let (sender, _) = broadcast::channel(capacity);

        Self {
            sender,
            handlers: RwLock::new(HashMap::new()),
            subscriptions: RwLock::new(HashMap::new()),
            middleware: RwLock::new(Vec::new()),
            stats: RwLock::new(EventStats {
                total_events: 0,
                events_by_type: HashMap::new(),
                events_by_source: HashMap::new(),
                successful_events: 0,
                failed_events: 0,
                average_processing_time_ms: 0.0,
            }),
        }
    }

    /// Publish an event to all subscribers  
    pub async fn publish(&self, event: EventPayload) -> Result<(), EventError> {
        let start_time = std::time::Instant::now();

        // Update statistics
        {
            let mut stats = self.stats.write().await;
            stats.total_events += 1;
            *stats
                .events_by_type
                .entry(event.event_type().to_string())
                .or_insert(0) += 1;
            *stats
                .events_by_source
                .entry(event.source().to_string())
                .or_insert(0) += 1;
        }

        // Run pre-middleware
        {
            let middleware = self.middleware.read().await;
            for m in middleware.iter() {
                if let Err(e) = m.before_handle(&event).await {
                    tracing::warn!("Event middleware failed: {}", e);
                }
            }
        }

        let result = async {
            // Send to broadcast channel
            let event_arc = Arc::new(event.clone());
            self.sender
                .send(event_arc)
                .map_err(|_| EventError::BusError("Failed to broadcast event".to_string()))?;

            // Process with registered handlers
            let handlers = self.handlers.read().await;
            if let Some(event_handlers) = handlers.get(event.event_type()) {
                for handler in event_handlers {
                    if let Err(e) = handler.handle(&event).await {
                        tracing::error!("Event handler failed: {}", e);
                        return Err(e);
                    }
                }
            }

            Ok(())
        }
        .await;

        // Run post-middleware
        {
            let middleware = self.middleware.read().await;
            for m in middleware.iter() {
                m.after_handle(&event, &result).await;
            }
        }

        // Update success/failure stats
        {
            let mut stats = self.stats.write().await;
            let duration = start_time.elapsed();

            match &result {
                Ok(_) => stats.successful_events += 1,
                Err(_) => stats.failed_events += 1,
            }

            // Update average processing time
            let total_processed = stats.successful_events + stats.failed_events;
            if total_processed > 1 {
                stats.average_processing_time_ms = (stats.average_processing_time_ms
                    * (total_processed - 1) as f64
                    + duration.as_millis() as f64)
                    / total_processed as f64;
            } else {
                stats.average_processing_time_ms = duration.as_millis() as f64;
            }
        }

        result
    }

    /// Subscribe to events with a filter
    pub async fn subscribe(&self, filter: EventFilter) -> EventSubscription {
        let subscription = EventSubscription::new(filter);

        {
            let mut subscriptions = self.subscriptions.write().await;
            subscriptions.insert(subscription.id, subscription.clone());
        }

        subscription
    }

    /// Unsubscribe from events
    pub async fn unsubscribe(&self, subscription_id: Uuid) -> bool {
        let mut subscriptions = self.subscriptions.write().await;
        subscriptions.remove(&subscription_id).is_some()
    }

    /// Register an event handler
    pub async fn register_handler<H: EventHandler<EventPayload> + 'static>(
        &self,
        event_type: String,
        handler: H,
    ) {
        let mut handlers = self.handlers.write().await;
        handlers
            .entry(event_type.clone())
            .or_insert_with(Vec::new)
            .push(Arc::new(handler));

        tracing::info!("Registered handler for event type: {}", event_type);
    }

    /// Add middleware to the event bus
    pub async fn add_middleware<M: EventMiddleware + 'static>(&self, middleware: M) {
        let mut middleware_vec = self.middleware.write().await;
        middleware_vec.push(Arc::new(middleware));
    }

    /// Get event statistics
    pub async fn get_stats(&self) -> EventStats {
        self.stats.read().await.clone()
    }

    /// Create a receiver for listening to all events
    pub fn create_receiver(&self) -> broadcast::Receiver<Arc<EventPayload>> {
        self.sender.subscribe()
    }

    /// Get filtered events stream
    pub async fn filtered_stream(&self, filter: EventFilter) -> FilteredEventStream {
        let receiver = self.create_receiver();
        FilteredEventStream::new(receiver, filter)
    }
}

impl Default for EventBus {
    fn default() -> Self {
        Self::new(1000) // Default capacity of 1000 events
    }
}

/// Filtered event stream for specific event types/sources
pub struct FilteredEventStream {
    receiver: broadcast::Receiver<Arc<EventPayload>>,
    filter: EventFilter,
}

impl FilteredEventStream {
    pub fn new(receiver: broadcast::Receiver<Arc<EventPayload>>, filter: EventFilter) -> Self {
        Self { receiver, filter }
    }

    /// Get the next event that matches the filter
    pub async fn next(&mut self) -> Result<Arc<EventPayload>, broadcast::error::RecvError> {
        loop {
            let event = self.receiver.recv().await?;

            // Check if event matches filter
            if self.filter.matches(event.as_ref()) {
                return Ok(event);
            }
        }
    }
}
