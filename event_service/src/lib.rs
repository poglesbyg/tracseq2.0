//! TracSeq Event Service Library
//! 
//! This library provides event-driven communication capabilities for TracSeq microservices.
//! It includes event definitions, publication, subscription, and processing capabilities.

pub mod events;
pub mod services;

// Re-export commonly used types for easier access
pub use events::{Event, EventContext, EventHandler, EventPublicationResult, SubscriptionConfig};
pub use services::{
    event_bus::{EventBus, RedisEventBus},
    client::EventServiceClient,
};