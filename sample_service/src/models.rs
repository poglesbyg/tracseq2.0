use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;
use validator::Validate;

/// Sample status enumeration
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::Type, PartialEq, Eq)]
#[sqlx(type_name = "sample_status", rename_all = "snake_case")]
pub enum SampleStatus {
    Pending,
    Validated,
    InStorage,
    InSequencing,
    Completed,
    Failed,
    Discarded,
}

impl SampleStatus {
    /// Check if a status transition is valid
    pub fn can_transition_to(&self, new_status: &SampleStatus) -> bool {
        use SampleStatus::*;
        match (self, new_status) {
            // Forward progression
            (Pending, Validated) => true,
            (Validated, InStorage) => true,
            (InStorage, InSequencing) => true,
            (InSequencing, Completed) => true,

            // Failure paths
            (Pending, Failed) => true,
            (Validated, Failed) => true,
            (InStorage, Failed) => true,
            (InSequencing, Failed) => true,

            // Discard from any state
            (_, Discarded) => true,

            // Backward transitions (for corrections)
            (Validated, Pending) => true,
            (InStorage, Validated) => true,

            // Same status (updates)
            (s1, s2) if s1 == s2 => true,

            _ => false,
        }
    }

    /// Get the next possible statuses
    pub fn next_statuses(&self) -> Vec<SampleStatus> {
        use SampleStatus::*;
        match self {
            Pending => vec![Validated, Failed, Discarded],
            Validated => vec![InStorage, Failed, Discarded, Pending],
            InStorage => vec![InSequencing, Failed, Discarded, Validated],
            InSequencing => vec![Completed, Failed, Discarded],
            Completed => vec![Discarded],
            Failed => vec![Pending, Discarded],
            Discarded => vec![],
        }
    }
}

/// Main sample entity
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Sample {
    pub id: Uuid,
    pub name: String,
    pub barcode: String,
    pub sample_type: String,
    pub status: SampleStatus,
    pub template_id: Option<Uuid>,
    pub source_type: Option<String>,
    pub source_identifier: Option<String>,
    pub collection_date: Option<DateTime<Utc>>,
    pub collection_location: Option<String>,
    pub collector: Option<String>,
    pub concentration: Option<rust_decimal::Decimal>,
    pub volume: Option<rust_decimal::Decimal>,
    pub unit: Option<String>,
    pub quality_score: Option<rust_decimal::Decimal>,
    pub metadata: serde_json::Value,
    pub notes: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub created_by: Option<String>,
    pub updated_by: Option<String>,
}

/// Sample creation request
#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct CreateSampleRequest {
    #[validate(length(
        min = 1,
        max = 255,
        message = "Sample name must be between 1 and 255 characters"
    ))]
    pub name: String,

    #[validate(length(
        min = 1,
        max = 50,
        message = "Sample type must be between 1 and 50 characters"
    ))]
    pub sample_type: String,

    pub barcode: Option<String>,
    pub template_id: Option<Uuid>,
    pub source_type: Option<String>,
    pub source_identifier: Option<String>,
    pub collection_date: Option<DateTime<Utc>>,
    pub collection_location: Option<String>,
    pub collector: Option<String>,

    #[validate(range(min = 0.0, message = "Concentration must be positive"))]
    pub concentration: Option<f64>,

    #[validate(range(min = 0.0, message = "Volume must be positive"))]
    pub volume: Option<f64>,

    pub unit: Option<String>,

    #[validate(range(
        min = 0.0,
        max = 1.0,
        message = "Quality score must be between 0 and 1"
    ))]
    pub quality_score: Option<f64>,

    pub metadata: Option<serde_json::Value>,
    pub notes: Option<String>,
}

/// Sample update request
#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct UpdateSampleRequest {
    #[validate(length(
        min = 1,
        max = 255,
        message = "Sample name must be between 1 and 255 characters"
    ))]
    pub name: Option<String>,

    #[validate(length(
        min = 1,
        max = 50,
        message = "Sample type must be between 1 and 50 characters"
    ))]
    pub sample_type: Option<String>,

    pub barcode: Option<String>,
    pub source_type: Option<String>,
    pub source_identifier: Option<String>,
    pub collection_date: Option<DateTime<Utc>>,
    pub collection_location: Option<String>,
    pub collector: Option<String>,

    #[validate(range(min = 0.0, message = "Concentration must be positive"))]
    pub concentration: Option<f64>,

    #[validate(range(min = 0.0, message = "Volume must be positive"))]
    pub volume: Option<f64>,

    pub unit: Option<String>,

    #[validate(range(
        min = 0.0,
        max = 1.0,
        message = "Quality score must be between 0 and 1"
    ))]
    pub quality_score: Option<f64>,

    pub metadata: Option<serde_json::Value>,
    pub notes: Option<String>,
}

/// Batch sample creation request
#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct CreateBatchSamplesRequest {
    #[validate(length(
        min = 1,
        max = 1000,
        message = "Batch must contain between 1 and 1000 samples"
    ))]
    pub samples: Vec<CreateSampleRequest>,

    pub template_id: Option<Uuid>,
    pub batch_name: Option<String>,
    pub auto_generate_barcodes: Option<bool>,
}

/// Sample status history
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct SampleStatusHistory {
    pub id: i32,
    pub sample_id: Uuid,
    pub previous_status: Option<SampleStatus>,
    pub new_status: SampleStatus,
    pub changed_at: DateTime<Utc>,
    pub changed_by: Option<String>,
    pub reason: Option<String>,
    pub automated: bool,
    pub metadata: serde_json::Value,
}

/// Sample validation rule
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct SampleValidationRule {
    pub id: i32,
    pub rule_name: String,
    pub sample_type: Option<String>,
    pub rule_expression: String,
    pub error_message: Option<String>,
    pub is_active: bool,
    pub severity: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Sample validation result
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct SampleValidationResult {
    pub id: i32,
    pub sample_id: Uuid,
    pub rule_id: Option<i32>,
    pub validation_passed: bool,
    pub error_message: Option<String>,
    pub validated_at: DateTime<Utc>,
    pub validated_by: Option<String>,
}

/// Barcode information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BarcodeInfo {
    pub barcode: String,
    pub prefix: String,
    pub timestamp: Option<String>,
    pub sequence: Option<String>,
    pub checksum: Option<String>,
    pub is_valid: bool,
}

/// Barcode generation request
#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct GenerateBarcodeRequest {
    #[validate(length(
        min = 1,
        max = 20,
        message = "Prefix must be between 1 and 20 characters"
    ))]
    pub prefix: Option<String>,

    pub sample_type: Option<String>,
    pub template_id: Option<Uuid>,
    pub include_timestamp: Option<bool>,
    pub include_sequence: Option<bool>,
}

/// Barcode validation request
#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct ValidateBarcodeRequest {
    #[validate(length(
        min = 1,
        max = 100,
        message = "Barcode must be between 1 and 100 characters"
    ))]
    pub barcode: String,
}

/// Sample relationship for batch tracking
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct SampleRelationship {
    pub id: i32,
    pub parent_sample_id: Uuid,
    pub child_sample_id: Uuid,
    pub relationship_type: String,
    pub created_at: DateTime<Utc>,
}

/// Sample search filters
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct SampleSearchFilters {
    pub status: Option<SampleStatus>,
    pub sample_type: Option<String>,
    pub template_id: Option<Uuid>,
    pub created_after: Option<DateTime<Utc>>,
    pub created_before: Option<DateTime<Utc>>,
    pub created_by: Option<String>,
    pub barcode_prefix: Option<String>,
    pub has_metadata_key: Option<String>,
    pub limit: Option<i64>,
    pub offset: Option<i64>,
}

/// Sample statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SampleStatistics {
    pub total_samples: i64,
    pub status_counts: std::collections::HashMap<String, i64>,
    pub sample_type_counts: std::collections::HashMap<String, i64>,
    pub samples_created_today: i64,
    pub samples_created_this_week: i64,
    pub samples_created_this_month: i64,
    pub average_processing_time_hours: Option<f64>,
    pub validation_success_rate: Option<f64>,
}

/// Workflow transition request
#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct WorkflowTransitionRequest {
    pub new_status: SampleStatus,

    #[validate(length(max = 500, message = "Reason must be at most 500 characters"))]
    pub reason: Option<String>,

    pub metadata: Option<serde_json::Value>,
    pub notify: Option<bool>,
}

/// Template integration data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TemplateData {
    pub template_id: Uuid,
    pub template_name: String,
    pub required_fields: Vec<String>,
    pub optional_fields: Vec<String>,
    pub field_types: std::collections::HashMap<String, String>,
    pub validation_rules: Vec<String>,
}

/// Template sample creation request
#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct CreateSampleFromTemplateRequest {
    pub template_id: Uuid,
    pub sample_data: std::collections::HashMap<String, serde_json::Value>,
    pub generate_barcode: Option<bool>,
}

/// API response wrappers
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SampleResponse {
    pub sample: Sample,
    pub validation_results: Option<Vec<SampleValidationResult>>,
    pub status_history: Option<Vec<SampleStatusHistory>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BatchSampleResponse {
    pub created_samples: Vec<Sample>,
    pub failed_samples: Vec<BatchSampleError>,
    pub total_created: usize,
    pub total_failed: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BatchSampleError {
    pub index: usize,
    pub sample_data: CreateSampleRequest,
    pub error: String,
}

/// Pagination response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PaginatedSampleResponse {
    pub samples: Vec<Sample>,
    pub total_count: i64,
    pub page: i64,
    pub page_size: i64,
    pub total_pages: i64,
}

/// Sample audit log entry
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct SampleAuditLog {
    pub id: i32,
    pub sample_id: Option<Uuid>,
    pub action: String,
    pub old_values: Option<serde_json::Value>,
    pub new_values: Option<serde_json::Value>,
    pub performed_by: Option<String>,
    pub performed_at: DateTime<Utc>,
    pub session_id: Option<String>,
    pub ip_address: Option<std::net::IpAddr>,
    pub user_agent: Option<String>,
}
