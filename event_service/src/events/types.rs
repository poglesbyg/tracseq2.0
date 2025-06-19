//! Laboratory-specific event types for TracSeq microservices.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

// =============================================================================
// Sample Events
// =============================================================================

/// Sample lifecycle events
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum SampleEvent {
    /// Sample was created
    SampleCreated {
        sample_id: Uuid,
        barcode: String,
        sample_type: String,
        submitter_id: Uuid,
        lab_id: Uuid,
        metadata: HashMap<String, serde_json::Value>,
        created_at: DateTime<Utc>,
    },

    /// Sample validation completed
    SampleValidated {
        sample_id: Uuid,
        barcode: String,
        validator_id: Uuid,
        validation_status: String,
        validation_notes: Option<String>,
        validated_at: DateTime<Utc>,
    },

    /// Sample status changed
    SampleStatusChanged {
        sample_id: Uuid,
        barcode: String,
        old_status: String,
        new_status: String,
        changed_by: Uuid,
        reason: Option<String>,
        changed_at: DateTime<Utc>,
    },

    /// Sample stored in location
    SampleStored {
        sample_id: Uuid,
        barcode: String,
        location_id: Uuid,
        storage_zone: String,
        position: Option<String>,
        stored_by: Uuid,
        stored_at: DateTime<Utc>,
    },

    /// Sample assigned to sequencing job
    SampleAssignedToSequencing {
        sample_id: Uuid,
        barcode: String,
        sequencing_job_id: Uuid,
        assigned_by: Uuid,
        assigned_at: DateTime<Utc>,
    },

    /// Sample processing completed
    SampleCompleted {
        sample_id: Uuid,
        barcode: String,
        completion_status: String,
        results_location: Option<String>,
        completed_by: Uuid,
        completed_at: DateTime<Utc>,
    },
}

// =============================================================================
// Storage Events
// =============================================================================

/// Storage and location events
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum StorageEvent {
    /// Temperature zone alert
    TemperatureAlert {
        location_id: Uuid,
        zone_name: String,
        current_temperature: f64,
        target_temperature: f64,
        threshold_type: String,
        sensor_id: String,
        alert_at: DateTime<Utc>,
    },

    /// Storage capacity warning
    CapacityWarning {
        location_id: Uuid,
        location_name: String,
        current_capacity: i32,
        max_capacity: i32,
        utilization_percent: f64,
        warning_at: DateTime<Utc>,
    },

    /// IoT sensor data received
    SensorDataReceived {
        sensor_id: String,
        location_id: Uuid,
        sensor_type: String,
        measurements: HashMap<String, f64>,
        timestamp: DateTime<Utc>,
        received_at: DateTime<Utc>,
    },
}

// =============================================================================
// Authentication Events
// =============================================================================

/// Authentication and authorization events
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum AuthEvent {
    /// User logged in
    UserLoggedIn {
        user_id: Uuid,
        username: String,
        session_id: String,
        ip_address: Option<String>,
        login_at: DateTime<Utc>,
    },

    /// User logged out
    UserLoggedOut {
        user_id: Uuid,
        username: String,
        session_id: String,
        logout_reason: String,
        logout_at: DateTime<Utc>,
    },

    /// Login attempt failed
    LoginFailed {
        username: String,
        failure_reason: String,
        ip_address: Option<String>,
        failed_at: DateTime<Utc>,
    },
}

// =============================================================================
// Sequencing Events
// =============================================================================

/// Sequencing workflow events
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum SequencingEvent {
    /// Sequencing job created
    JobCreated {
        job_id: Uuid,
        job_name: String,
        job_type: String,
        priority: u8,
        samples: Vec<Uuid>,
        created_by: Uuid,
        created_at: DateTime<Utc>,
    },

    /// Job status changed
    JobStatusChanged {
        job_id: Uuid,
        job_name: String,
        old_status: String,
        new_status: String,
        progress_percent: f64,
        updated_at: DateTime<Utc>,
    },

    /// Job completed
    JobCompleted {
        job_id: Uuid,
        job_name: String,
        completion_status: String,
        total_samples: i32,
        successful_samples: i32,
        failed_samples: i32,
        completed_at: DateTime<Utc>,
    },
}

// =============================================================================
// Document Events
// =============================================================================

/// Document processing and RAG events
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum DocumentEvent {
    /// Document uploaded
    DocumentUploaded {
        document_id: Uuid,
        filename: String,
        file_type: String,
        file_size: u64,
        uploaded_by: Uuid,
        uploaded_at: DateTime<Utc>,
    },

    /// Document processing completed
    ProcessingCompleted {
        document_id: Uuid,
        filename: String,
        processing_status: String,
        extracted_data: HashMap<String, serde_json::Value>,
        confidence_score: f64,
        completed_at: DateTime<Utc>,
    },

    /// Information extracted from document
    InformationExtracted {
        document_id: Uuid,
        extraction_type: String,
        extracted_fields: HashMap<String, serde_json::Value>,
        confidence_scores: HashMap<String, f64>,
        extracted_at: DateTime<Utc>,
    },
}

// =============================================================================
// Event Type Constants
// =============================================================================

/// Event type constants for easy reference
pub mod event_types {
    // Sample events
    pub const SAMPLE_CREATED: &str = "sample.created";
    pub const SAMPLE_VALIDATED: &str = "sample.validated";
    pub const SAMPLE_STATUS_CHANGED: &str = "sample.status_changed";
    pub const SAMPLE_STORED: &str = "sample.stored";
    pub const SAMPLE_ASSIGNED_TO_SEQUENCING: &str = "sample.assigned_to_sequencing";
    pub const SAMPLE_COMPLETED: &str = "sample.completed";

    // Storage events
    pub const TEMPERATURE_ALERT: &str = "storage.temperature_alert";
    pub const CAPACITY_WARNING: &str = "storage.capacity_warning";
    pub const SENSOR_DATA_RECEIVED: &str = "storage.sensor_data_received";

    // Auth events
    pub const USER_LOGGED_IN: &str = "auth.user_logged_in";
    pub const USER_LOGGED_OUT: &str = "auth.user_logged_out";
    pub const LOGIN_FAILED: &str = "auth.login_failed";

    // Sequencing events
    pub const JOB_CREATED: &str = "sequencing.job_created";
    pub const JOB_STATUS_CHANGED: &str = "sequencing.job_status_changed";
    pub const JOB_COMPLETED: &str = "sequencing.job_completed";

    // Document events
    pub const DOCUMENT_UPLOADED: &str = "document.uploaded";
    pub const PROCESSING_COMPLETED: &str = "document.processing_completed";
    pub const INFORMATION_EXTRACTED: &str = "document.information_extracted";
}
