use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Event priority levels
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum EventPriority {
    Low,
    Normal,
    High,
    Critical,
}

/// Event status
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum EventStatus {
    Pending,
    Processing,
    Completed,
    Failed,
    Retrying,
}

/// Core event model
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EventModel {
    pub id: Uuid,
    pub event_type: String,
    pub source_service: String,
    pub target_service: Option<String>,
    pub payload: serde_json::Value,
    pub priority: EventPriority,
    pub status: EventStatus,
    pub created_at: DateTime<Utc>,
    pub processed_at: Option<DateTime<Utc>>,
    pub retry_count: u32,
    pub max_retries: u32,
    pub correlation_id: Option<Uuid>,
}

/// Event subscription configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EventSubscription {
    pub id: Uuid,
    pub service_name: String,
    pub event_types: Vec<String>,
    pub callback_url: String,
    pub active: bool,
    pub created_at: DateTime<Utc>,
}

/// Event processing result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EventProcessingResult {
    pub event_id: Uuid,
    pub success: bool,
    pub error_message: Option<String>,
    pub processed_at: DateTime<Utc>,
    pub processing_duration_ms: u64,
}

impl Default for EventPriority {
    fn default() -> Self {
        Self::Normal
    }
}

impl Default for EventStatus {
    fn default() -> Self {
        Self::Pending
    }
}
