use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

use super::{Event, EventPriority};

/// Template-related events
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TemplateUploadedEvent {
    pub template_id: Uuid,
    pub template_name: String,
    pub file_path: String,
    pub uploaded_by: String,
    pub source: String,
    pub timestamp: chrono::DateTime<chrono::Utc>,
    pub metadata: HashMap<String, String>,
}

impl Event for TemplateUploadedEvent {
    fn event_type(&self) -> &'static str {
        "template.uploaded"
    }

    fn source(&self) -> &str {
        &self.source
    }

    fn timestamp(&self) -> chrono::DateTime<chrono::Utc> {
        self.timestamp
    }

    fn metadata(&self) -> &HashMap<String, String> {
        &self.metadata
    }

    fn priority(&self) -> EventPriority {
        EventPriority::Normal
    }
}

/// Sample-related events
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SampleCreatedEvent {
    pub sample_id: Uuid,
    pub sample_name: String,
    pub barcode: String,
    pub location: String,
    pub created_by: String,
    pub source: String,
    pub timestamp: chrono::DateTime<chrono::Utc>,
    pub metadata: HashMap<String, String>,
}

impl Event for SampleCreatedEvent {
    fn event_type(&self) -> &'static str {
        "sample.created"
    }

    fn source(&self) -> &str {
        &self.source
    }

    fn timestamp(&self) -> chrono::DateTime<chrono::Utc> {
        self.timestamp
    }

    fn metadata(&self) -> &HashMap<String, String> {
        &self.metadata
    }

    fn priority(&self) -> EventPriority {
        EventPriority::High // Sample creation is important
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SampleStatusChangedEvent {
    pub sample_id: Uuid,
    pub old_status: String,
    pub new_status: String,
    pub changed_by: String,
    pub reason: Option<String>,
    pub source: String,
    pub timestamp: chrono::DateTime<chrono::Utc>,
    pub metadata: HashMap<String, String>,
}

impl Event for SampleStatusChangedEvent {
    fn event_type(&self) -> &'static str {
        "sample.status_changed"
    }

    fn source(&self) -> &str {
        &self.source
    }

    fn timestamp(&self) -> chrono::DateTime<chrono::Utc> {
        self.timestamp
    }

    fn metadata(&self) -> &HashMap<String, String> {
        &self.metadata
    }

    fn priority(&self) -> EventPriority {
        match self.new_status.as_str() {
            "failed" | "error" => EventPriority::Critical,
            "completed" => EventPriority::High,
            _ => EventPriority::Normal,
        }
    }
}

/// Storage-related events
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileStoredEvent {
    pub file_id: Uuid,
    pub file_name: String,
    pub file_path: String,
    pub file_size: u64,
    pub checksum: Option<String>,
    pub stored_by: String,
    pub source: String,
    pub timestamp: chrono::DateTime<chrono::Utc>,
    pub metadata: HashMap<String, String>,
}

impl Event for FileStoredEvent {
    fn event_type(&self) -> &'static str {
        "storage.file_stored"
    }

    fn source(&self) -> &str {
        &self.source
    }

    fn timestamp(&self) -> chrono::DateTime<chrono::Utc> {
        self.timestamp
    }

    fn metadata(&self) -> &HashMap<String, String> {
        &self.metadata
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StorageQuotaWarningEvent {
    pub current_usage: u64,
    pub total_capacity: u64,
    pub usage_percentage: f64,
    pub threshold_percentage: f64,
    pub source: String,
    pub timestamp: chrono::DateTime<chrono::Utc>,
    pub metadata: HashMap<String, String>,
}

impl Event for StorageQuotaWarningEvent {
    fn event_type(&self) -> &'static str {
        "storage.quota_warning"
    }

    fn source(&self) -> &str {
        &self.source
    }

    fn timestamp(&self) -> chrono::DateTime<chrono::Utc> {
        self.timestamp
    }

    fn metadata(&self) -> &HashMap<String, String> {
        &self.metadata
    }

    fn priority(&self) -> EventPriority {
        if self.usage_percentage > 95.0 {
            EventPriority::Critical
        } else if self.usage_percentage > 85.0 {
            EventPriority::High
        } else {
            EventPriority::Normal
        }
    }
}

/// Sequencing-related events
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SequencingJobCreatedEvent {
    pub job_id: Uuid,
    pub job_name: String,
    pub sample_count: u32,
    pub created_by: String,
    pub source: String,
    pub timestamp: chrono::DateTime<chrono::Utc>,
    pub metadata: HashMap<String, String>,
}

impl Event for SequencingJobCreatedEvent {
    fn event_type(&self) -> &'static str {
        "sequencing.job_created"
    }

    fn source(&self) -> &str {
        &self.source
    }

    fn timestamp(&self) -> chrono::DateTime<chrono::Utc> {
        self.timestamp
    }

    fn metadata(&self) -> &HashMap<String, String> {
        &self.metadata
    }

    fn priority(&self) -> EventPriority {
        EventPriority::High
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SequencingJobCompletedEvent {
    pub job_id: Uuid,
    pub job_name: String,
    pub duration_seconds: u64,
    pub successful_samples: u32,
    pub failed_samples: u32,
    pub output_path: String,
    pub source: String,
    pub timestamp: chrono::DateTime<chrono::Utc>,
    pub metadata: HashMap<String, String>,
}

impl Event for SequencingJobCompletedEvent {
    fn event_type(&self) -> &'static str {
        "sequencing.job_completed"
    }

    fn source(&self) -> &str {
        &self.source
    }

    fn timestamp(&self) -> chrono::DateTime<chrono::Utc> {
        self.timestamp
    }

    fn metadata(&self) -> &HashMap<String, String> {
        &self.metadata
    }

    fn priority(&self) -> EventPriority {
        if self.failed_samples > 0 {
            EventPriority::High
        } else {
            EventPriority::Normal
        }
    }
}

/// System-related events
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComponentHealthChangedEvent {
    pub component_name: String,
    pub old_status: String,
    pub new_status: String,
    pub details: Option<String>,
    pub source: String,
    pub timestamp: chrono::DateTime<chrono::Utc>,
    pub metadata: HashMap<String, String>,
}

impl Event for ComponentHealthChangedEvent {
    fn event_type(&self) -> &'static str {
        "system.component_health_changed"
    }

    fn source(&self) -> &str {
        &self.source
    }

    fn timestamp(&self) -> chrono::DateTime<chrono::Utc> {
        self.timestamp
    }

    fn metadata(&self) -> &HashMap<String, String> {
        &self.metadata
    }

    fn priority(&self) -> EventPriority {
        match self.new_status.as_str() {
            "unhealthy" => EventPriority::Critical,
            "degraded" => EventPriority::High,
            "healthy" => EventPriority::Normal,
            _ => EventPriority::Normal,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConfigurationChangedEvent {
    pub component_name: String,
    pub setting_key: String,
    pub old_value: Option<String>,
    pub new_value: String,
    pub changed_by: String,
    pub source: String,
    pub timestamp: chrono::DateTime<chrono::Utc>,
    pub metadata: HashMap<String, String>,
}

impl Event for ConfigurationChangedEvent {
    fn event_type(&self) -> &'static str {
        "system.configuration_changed"
    }

    fn source(&self) -> &str {
        &self.source
    }

    fn timestamp(&self) -> chrono::DateTime<chrono::Utc> {
        self.timestamp
    }

    fn metadata(&self) -> &HashMap<String, String> {
        &self.metadata
    }

    fn priority(&self) -> EventPriority {
        // Security-related configuration changes are critical
        if self.setting_key.contains("password")
            || self.setting_key.contains("secret")
            || self.setting_key.contains("key")
        {
            EventPriority::Critical
        } else {
            EventPriority::Normal
        }
    }
}

/// Utility functions for creating events
impl TemplateUploadedEvent {
    pub fn new(
        template_id: Uuid,
        template_name: String,
        file_path: String,
        uploaded_by: String,
    ) -> Self {
        Self {
            template_id,
            template_name,
            file_path,
            uploaded_by,
            source: "template_service".to_string(),
            timestamp: chrono::Utc::now(),
            metadata: HashMap::new(),
        }
    }
}

impl SampleCreatedEvent {
    pub fn new(
        sample_id: Uuid,
        sample_name: String,
        barcode: String,
        location: String,
        created_by: String,
    ) -> Self {
        Self {
            sample_id,
            sample_name,
            barcode,
            location,
            created_by,
            source: "sample_service".to_string(),
            timestamp: chrono::Utc::now(),
            metadata: HashMap::new(),
        }
    }
}

impl FileStoredEvent {
    pub fn new(
        file_id: Uuid,
        file_name: String,
        file_path: String,
        file_size: u64,
        stored_by: String,
    ) -> Self {
        Self {
            file_id,
            file_name,
            file_path,
            file_size,
            checksum: None,
            stored_by,
            source: "storage_service".to_string(),
            timestamp: chrono::Utc::now(),
            metadata: HashMap::new(),
        }
    }
}
