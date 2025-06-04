pub mod bus;
pub mod types;

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

pub use bus::EventBus;
pub use types::*;

/// Concrete enum holding all possible event types for type-safe event bus
#[derive(Debug, Clone)]
pub enum EventPayload {
    TemplateUploaded(types::TemplateUploadedEvent),
    SampleCreated(types::SampleCreatedEvent),
    SampleStatusChanged(types::SampleStatusChangedEvent),
    FileStored(types::FileStoredEvent),
    StorageQuotaWarning(types::StorageQuotaWarningEvent),
    SequencingJobCreated(types::SequencingJobCreatedEvent),
    SequencingJobCompleted(types::SequencingJobCompletedEvent),
    ComponentHealthChanged(types::ComponentHealthChangedEvent),
    ConfigurationChanged(types::ConfigurationChangedEvent),
}

impl Event for EventPayload {
    fn event_type(&self) -> &'static str {
        match self {
            Self::TemplateUploaded(e) => e.event_type(),
            Self::SampleCreated(e) => e.event_type(),
            Self::SampleStatusChanged(e) => e.event_type(),
            Self::FileStored(e) => e.event_type(),
            Self::StorageQuotaWarning(e) => e.event_type(),
            Self::SequencingJobCreated(e) => e.event_type(),
            Self::SequencingJobCompleted(e) => e.event_type(),
            Self::ComponentHealthChanged(e) => e.event_type(),
            Self::ConfigurationChanged(e) => e.event_type(),
        }
    }

    fn source(&self) -> &str {
        match self {
            Self::TemplateUploaded(e) => e.source(),
            Self::SampleCreated(e) => e.source(),
            Self::SampleStatusChanged(e) => e.source(),
            Self::FileStored(e) => e.source(),
            Self::StorageQuotaWarning(e) => e.source(),
            Self::SequencingJobCreated(e) => e.source(),
            Self::SequencingJobCompleted(e) => e.source(),
            Self::ComponentHealthChanged(e) => e.source(),
            Self::ConfigurationChanged(e) => e.source(),
        }
    }

    fn timestamp(&self) -> chrono::DateTime<chrono::Utc> {
        match self {
            Self::TemplateUploaded(e) => e.timestamp(),
            Self::SampleCreated(e) => e.timestamp(),
            Self::SampleStatusChanged(e) => e.timestamp(),
            Self::FileStored(e) => e.timestamp(),
            Self::StorageQuotaWarning(e) => e.timestamp(),
            Self::SequencingJobCreated(e) => e.timestamp(),
            Self::SequencingJobCompleted(e) => e.timestamp(),
            Self::ComponentHealthChanged(e) => e.timestamp(),
            Self::ConfigurationChanged(e) => e.timestamp(),
        }
    }

    fn priority(&self) -> EventPriority {
        match self {
            Self::TemplateUploaded(e) => e.priority(),
            Self::SampleCreated(e) => e.priority(),
            Self::SampleStatusChanged(e) => e.priority(),
            Self::FileStored(e) => e.priority(),
            Self::StorageQuotaWarning(e) => e.priority(),
            Self::SequencingJobCreated(e) => e.priority(),
            Self::SequencingJobCompleted(e) => e.priority(),
            Self::ComponentHealthChanged(e) => e.priority(),
            Self::ConfigurationChanged(e) => e.priority(),
        }
    }

    fn metadata(&self) -> &HashMap<String, String> {
        match self {
            Self::TemplateUploaded(e) => e.metadata(),
            Self::SampleCreated(e) => e.metadata(),
            Self::SampleStatusChanged(e) => e.metadata(),
            Self::FileStored(e) => e.metadata(),
            Self::StorageQuotaWarning(e) => e.metadata(),
            Self::SequencingJobCreated(e) => e.metadata(),
            Self::SequencingJobCompleted(e) => e.metadata(),
            Self::ComponentHealthChanged(e) => e.metadata(),
            Self::ConfigurationChanged(e) => e.metadata(),
        }
    }
}

/// Core event trait that all events must implement
pub trait Event: Send + Sync + Clone {
    /// Get the event type identifier
    fn event_type(&self) -> &'static str;

    /// Get the event source component
    fn source(&self) -> &str;

    /// Get the event timestamp
    fn timestamp(&self) -> chrono::DateTime<chrono::Utc>;

    /// Get event metadata
    fn metadata(&self) -> &HashMap<String, String>;

    /// Get event priority
    fn priority(&self) -> EventPriority {
        EventPriority::Normal
    }
}

/// Event priority levels
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum EventPriority {
    Low = 0,
    Normal = 1,
    High = 2,
    Critical = 3,
}

/// Event handler trait for processing events
#[async_trait]
pub trait EventHandler<E: Event>: Send + Sync {
    /// Handle an event
    async fn handle(&self, event: &E) -> Result<(), EventError>;

    /// Get handler configuration
    fn config(&self) -> EventHandlerConfig;
}

/// Event handler configuration
#[derive(Debug, Clone)]
pub struct EventHandlerConfig {
    pub name: String,
    pub event_types: Vec<String>,
    pub priority: EventPriority,
    pub retry_count: u32,
    pub timeout_ms: u64,
}

/// Event processing errors
#[derive(Debug, thiserror::Error)]
pub enum EventError {
    #[error("Handler not found for event type: {0}")]
    HandlerNotFound(String),
    #[error("Event processing failed: {0}")]
    ProcessingFailed(String),
    #[error("Event handler timeout")]
    Timeout,
    #[error("Event serialization failed: {0}")]
    SerializationFailed(String),
    #[error("Event bus error: {0}")]
    BusError(String),
}

/// Event filter for selective event processing
#[derive(Debug, Clone)]
pub struct EventFilter {
    pub event_types: Option<Vec<String>>,
    pub sources: Option<Vec<String>>,
    pub priority: Option<EventPriority>,
    pub since: Option<chrono::DateTime<chrono::Utc>>,
}

impl EventFilter {
    pub fn new() -> Self {
        Self {
            event_types: None,
            sources: None,
            priority: None,
            since: None,
        }
    }

    pub fn with_event_types(mut self, types: Vec<String>) -> Self {
        self.event_types = Some(types);
        self
    }

    pub fn with_sources(mut self, sources: Vec<String>) -> Self {
        self.sources = Some(sources);
        self
    }

    pub fn with_priority(mut self, priority: EventPriority) -> Self {
        self.priority = Some(priority);
        self
    }

    pub fn matches<E: Event>(&self, event: &E) -> bool {
        if let Some(ref types) = self.event_types {
            if !types.contains(&event.event_type().to_string()) {
                return false;
            }
        }

        if let Some(ref sources) = self.sources {
            if !sources.contains(&event.source().to_string()) {
                return false;
            }
        }

        if let Some(priority) = self.priority {
            if event.priority() < priority {
                return false;
            }
        }

        if let Some(since) = self.since {
            if event.timestamp() < since {
                return false;
            }
        }

        true
    }
}

impl Default for EventFilter {
    fn default() -> Self {
        Self::new()
    }
}

/// Event subscription for listening to specific events
#[derive(Debug, Clone)]
pub struct EventSubscription {
    pub id: Uuid,
    pub filter: EventFilter,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

impl EventSubscription {
    pub fn new(filter: EventFilter) -> Self {
        Self {
            id: Uuid::new_v4(),
            filter,
            created_at: chrono::Utc::now(),
        }
    }
}

/// Event statistics for monitoring
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EventStats {
    pub total_events: u64,
    pub events_by_type: HashMap<String, u64>,
    pub events_by_source: HashMap<String, u64>,
    pub successful_events: u64,
    pub failed_events: u64,
    pub average_processing_time_ms: f64,
}

/// Event middleware trait for intercepting events
#[async_trait]
pub trait EventMiddleware: Send + Sync {
    /// Process event before handlers
    async fn before_handle(&self, event: &EventPayload) -> Result<(), EventError>;

    /// Process event after handlers
    async fn after_handle(&self, event: &EventPayload, result: &Result<(), EventError>);
}
