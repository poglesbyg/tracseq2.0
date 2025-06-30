//! Event definitions and schemas for TracSeq laboratory management system.
//! 
//! This module defines all event types that can be published and consumed
//! across the TracSeq microservices ecosystem.

pub mod schemas;
pub mod types;
pub mod handlers;

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;
use validator::Validate;

/// Base event envelope that wraps all domain events
#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct Event {
    /// Unique event identifier
    pub id: Uuid,
    
    /// Event type identifier
    pub event_type: String,
    
    /// Service that published the event
    pub source_service: String,
    
    /// Event version for schema evolution
    pub version: String,
    
    /// Timestamp when event was created
    pub timestamp: DateTime<Utc>,
    
    /// Correlation ID for request tracing
    pub correlation_id: Option<Uuid>,
    
    /// Event payload
    pub payload: serde_json::Value,
    
    /// Optional metadata
    pub metadata: HashMap<String, String>,
    
    /// Subject/entity the event is about
    pub subject: Option<String>,
    
    /// Event priority (1=highest, 5=lowest)
    #[validate(range(min = 1, max = 5))]
    pub priority: u8,
}

impl Event {
    /// Create a new event
    #[allow(dead_code)]
    pub fn new(
        event_type: String,
        source_service: String,
        payload: serde_json::Value,
    ) -> Self {
        Self {
            id: Uuid::new_v4(),
            event_type,
            source_service,
            version: "1.0".to_string(),
            timestamp: Utc::now(),
            correlation_id: None,
            payload,
            metadata: HashMap::new(),
            subject: None,
            priority: 3, // Normal priority
        }
    }
    
    /// Set correlation ID for request tracing
    #[allow(dead_code)]
    pub fn with_correlation_id(mut self, correlation_id: Uuid) -> Self {
        self.correlation_id = Some(correlation_id);
        self
    }
    
    /// Set event subject
    #[allow(dead_code)]
    pub fn with_subject(mut self, subject: String) -> Self {
        self.subject = Some(subject);
        self
    }
    
    /// Set event priority
    #[allow(dead_code)]
    pub fn with_priority(mut self, priority: u8) -> Self {
        self.priority = priority;
        self
    }
    
    /// Add metadata
    #[allow(dead_code)]
    pub fn with_metadata(mut self, key: String, value: String) -> Self {
        self.metadata.insert(key, value);
        self
    }
    
    /// Get event stream name based on type
    #[allow(dead_code)]
    pub fn stream_name(&self) -> String {
        format!("tracseq:events:{}", self.event_type.replace('.', ":"))
    }
}

/// Event publication result
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct EventPublicationResult {
    pub event_id: Uuid,
    pub stream_id: String,
    pub published_at: DateTime<Utc>,
}

/// Event subscription configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SubscriptionConfig {
    /// Subscription name
    pub name: String,
    
    /// Event types to subscribe to (patterns supported)
    pub event_types: Vec<String>,
    
    /// Consumer group name
    pub consumer_group: String,
    
    /// Consumer name within the group
    pub consumer_name: String,
    
    /// Maximum number of events to process in batch
    pub batch_size: usize,
    
    /// Processing timeout in milliseconds
    pub timeout_ms: u64,
    
    /// Auto-acknowledge events after successful processing
    pub auto_ack: bool,
    
    /// Start reading from latest events (vs. beginning)
    pub read_latest: bool,
}

impl Default for SubscriptionConfig {
    fn default() -> Self {
        Self {
            name: "default".to_string(),
            event_types: vec!["*".to_string()],
            consumer_group: "default-group".to_string(),
            consumer_name: "default-consumer".to_string(),
            batch_size: 10,
            timeout_ms: 5000,
            auto_ack: true,
            read_latest: true,
        }
    }
}

/// Event processing context
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct EventContext {
    pub event: Event,
    pub delivery_count: u32,
    pub subscription: String,
    pub stream_id: String,
}

/// Event handler trait for processing events
#[async_trait::async_trait]
#[allow(dead_code)]
pub trait EventHandler: Send + Sync {
    /// Handle an incoming event
    async fn handle(&self, context: EventContext) -> Result<(), Box<dyn std::error::Error + Send + Sync>>;
    
    /// Get event types this handler can process
    fn event_types(&self) -> Vec<String>;
    
    /// Get handler name for logging and monitoring
    fn name(&self) -> String;
}

/// Event filter for conditional processing
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EventFilter {
    /// Event types to match (supports wildcards)
    pub event_types: Vec<String>,
    
    /// Source services to match
    pub source_services: Vec<String>,
    
    /// Metadata filters (key-value pairs)
    pub metadata_filters: HashMap<String, String>,
    
    /// Subject patterns to match
    pub subject_patterns: Vec<String>,
    
    /// Priority range (min, max)
    pub priority_range: Option<(u8, u8)>,
}

impl EventFilter {
    /// Check if event matches this filter
    #[allow(dead_code)]
    pub fn matches(&self, event: &Event) -> bool {
        // Check event types
        if !self.event_types.is_empty() && !self.matches_patterns(&self.event_types, &event.event_type) {
            return false;
        }
        
        // Check source services
        if !self.source_services.is_empty() && !self.source_services.contains(&event.source_service) {
            return false;
        }
        
        // Check metadata filters
        for (key, value) in &self.metadata_filters {
            if event.metadata.get(key) != Some(value) {
                return false;
            }
        }
        
        // Check subject patterns
        if !self.subject_patterns.is_empty() {
            if let Some(subject) = &event.subject {
                if !self.matches_patterns(&self.subject_patterns, subject) {
                    return false;
                }
            } else {
                return false;
            }
        }
        
        // Check priority range
        if let Some((min, max)) = self.priority_range {
            if event.priority < min || event.priority > max {
                return false;
            }
        }
        
        true
    }
    
    /// Check if value matches any of the patterns (supports wildcards)
    #[allow(dead_code)]
    fn matches_patterns(&self, patterns: &[String], value: &str) -> bool {
        patterns.iter().any(|pattern| {
            if pattern == "*" {
                true
            } else if pattern.contains('*') {
                // Simple wildcard matching
                let regex_pattern = pattern.replace('*', ".*");
                regex::Regex::new(&format!("^{}$", regex_pattern))
                    .map(|r| r.is_match(value))
                    .unwrap_or(false)
            } else {
                pattern == value
            }
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_event_creation() {
        let payload = serde_json::json!({"test": "data"});
        let event = Event::new(
            "test.event".to_string(),
            "test-service".to_string(),
            payload,
        );
        
        assert_eq!(event.event_type, "test.event");
        assert_eq!(event.source_service, "test-service");
        assert_eq!(event.version, "1.0");
        assert_eq!(event.priority, 3);
    }
    
    #[test]
    fn test_event_filter_matching() {
        let event = Event::new(
            "sample.created".to_string(),
            "sample-service".to_string(),
            serde_json::json!({}),
        ).with_subject("sample-123".to_string());
        
        let filter = EventFilter {
            event_types: vec!["sample.*".to_string()],
            source_services: vec!["sample-service".to_string()],
            metadata_filters: HashMap::new(),
            subject_patterns: vec!["sample-*".to_string()],
            priority_range: None,
        };
        
        assert!(filter.matches(&event));
    }
} 
