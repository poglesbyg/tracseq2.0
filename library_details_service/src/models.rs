use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;
use chrono::{DateTime, Utc, NaiveDate};
use std::collections::HashMap;
use validator::Validate;


#[derive(Debug, Clone, Serialize, Deserialize, FromRow, Validate)]
pub struct Library {
    pub id: Uuid,
    #[validate(length(min = 1))]
    pub name: String,
    pub sample_id: Option<Uuid>,
    #[validate(length(min = 1))]
    pub library_type: String,
    #[validate(range(min = 0.0))]
    pub concentration: Option<f64>,
    #[validate(range(min = 0.0))]
    pub volume: Option<f64>,
    pub fragment_size_min: Option<i32>,
    pub fragment_size_max: Option<i32>,
    pub preparation_protocol_id: Option<Uuid>,
    pub preparation_date: Option<DateTime<Utc>>,
    pub barcode: Option<String>,
    pub adapter_sequence: Option<String>,
    #[validate(range(min = 0.0, max = 1.0))]
    pub quality_score: Option<f64>,
    pub status: LibraryStatus,
    pub metadata: Option<serde_json::Value>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct CreateLibraryRequest {
    #[validate(length(min = 1))]
    pub name: String,
    pub sample_id: Option<Uuid>,
    #[validate(length(min = 1))]
    pub library_type: String,
    #[validate(range(min = 0.0))]
    pub concentration: Option<f64>,
    #[validate(range(min = 0.0))]
    pub volume: Option<f64>,
    pub fragment_size_min: Option<i32>,
    pub fragment_size_max: Option<i32>,
    pub preparation_protocol_id: Option<Uuid>,
    pub preparation_date: Option<DateTime<Utc>>,
    pub barcode: Option<String>,
    pub adapter_sequence: Option<String>,
    pub metadata: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct UpdateLibraryRequest {
    #[validate(length(min = 1))]
    pub name: Option<String>,
    #[validate(length(min = 1))]
    pub library_type: Option<String>,
    #[validate(range(min = 0.0))]
    pub concentration: Option<f64>,
    #[validate(range(min = 0.0))]
    pub volume: Option<f64>,
    pub fragment_size_min: Option<i32>,
    pub fragment_size_max: Option<i32>,
    pub preparation_protocol_id: Option<Uuid>,
    pub preparation_date: Option<DateTime<Utc>>,
    pub barcode: Option<String>,
    pub adapter_sequence: Option<String>,
    pub status: Option<LibraryStatus>,
    pub metadata: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::Type, PartialEq)]
#[sqlx(type_name = "library_status", rename_all = "lowercase")]
pub enum LibraryStatus {
    Pending,
    InPreparation,
    QualityControl,
    Approved,
    Failed,
    Sequencing,
    Completed,
}

impl Default for LibraryStatus {
    fn default() -> Self {
        LibraryStatus::Pending
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow, Validate)]
pub struct Protocol {
    pub id: Uuid,
    #[validate(length(min = 1))]
    pub name: String,
    #[validate(length(min = 1))]
    pub version: String,
    #[validate(length(min = 1))]
    pub library_type: String,
    pub description: Option<String>,
    pub steps: serde_json::Value,
    pub parameters: Option<serde_json::Value>,
    pub kit_id: Option<Uuid>,
    pub platform_compatibility: Option<serde_json::Value>,
    pub quality_thresholds: Option<serde_json::Value>,
    pub is_active: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct CreateProtocolRequest {
    #[validate(length(min = 1))]
    pub name: String,
    #[validate(length(min = 1))]
    pub version: String,
    #[validate(length(min = 1))]
    pub library_type: String,
    pub description: Option<String>,
    pub steps: serde_json::Value,
    pub parameters: Option<serde_json::Value>,
    pub kit_id: Option<Uuid>,
    pub platform_compatibility: Option<serde_json::Value>,
    pub quality_thresholds: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow, Validate)]
pub struct Kit {
    pub id: Uuid,
    #[validate(length(min = 1))]
    pub name: String,
    #[validate(length(min = 1))]
    pub manufacturer: String,
    pub catalog_number: Option<String>,
    pub version: Option<String>,
    pub library_types: serde_json::Value,
    pub reagents: Option<serde_json::Value>,
    pub specifications: Option<serde_json::Value>,
    pub storage_conditions: Option<String>,
    pub expiry_date: Option<NaiveDate>,
    pub cost_per_reaction: Option<f64>,
    pub throughput_capacity: Option<i32>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct CreateKitRequest {
    #[validate(length(min = 1))]
    pub name: String,
    #[validate(length(min = 1))]
    pub manufacturer: String,
    pub catalog_number: Option<String>,
    pub version: Option<String>,
    pub library_types: serde_json::Value,
    pub reagents: Option<serde_json::Value>,
    pub specifications: Option<serde_json::Value>,
    pub storage_conditions: Option<String>,
    pub expiry_date: Option<NaiveDate>,
    pub cost_per_reaction: Option<f64>,
    pub throughput_capacity: Option<i32>,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow, Validate)]
pub struct Platform {
    pub id: Uuid,
    #[validate(length(min = 1))]
    pub name: String,
    #[validate(length(min = 1))]
    pub manufacturer: String,
    #[validate(length(min = 1))]
    pub model: String,
    pub capabilities: serde_json::Value,
    pub supported_library_types: serde_json::Value,
    pub read_configurations: Option<serde_json::Value>,
    pub flow_cell_types: Option<serde_json::Value>,
    pub throughput_specs: Option<serde_json::Value>,
    pub quality_metrics: Option<serde_json::Value>,
    pub is_active: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct CreatePlatformRequest {
    #[validate(length(min = 1))]
    pub name: String,
    #[validate(length(min = 1))]
    pub manufacturer: String,
    #[validate(length(min = 1))]
    pub model: String,
    pub capabilities: serde_json::Value,
    pub supported_library_types: serde_json::Value,
    pub read_configurations: Option<serde_json::Value>,
    pub flow_cell_types: Option<serde_json::Value>,
    pub throughput_specs: Option<serde_json::Value>,
    pub quality_metrics: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow, Validate)]
pub struct QualityControlMetric {
    pub id: Uuid,
    pub library_id: Uuid,
    #[validate(length(min = 1))]
    pub metric_type: String,
    pub value: f64,
    pub unit: Option<String>,
    pub threshold_min: Option<f64>,
    pub threshold_max: Option<f64>,
    pub status: QCStatus,
    pub measured_at: DateTime<Utc>,
    pub equipment_id: Option<String>,
    pub operator_id: Option<String>,
    pub notes: Option<String>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct CreateQCMetricRequest {
    pub library_id: Uuid,
    #[validate(length(min = 1))]
    pub metric_type: String,
    pub value: f64,
    pub unit: Option<String>,
    pub threshold_min: Option<f64>,
    pub threshold_max: Option<f64>,
    pub equipment_id: Option<String>,
    pub operator_id: Option<String>,
    pub notes: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::Type, PartialEq)]
#[sqlx(type_name = "qc_status", rename_all = "lowercase")]
pub enum QCStatus {
    Pass,
    Fail,
    Warning,
    Pending,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LibraryMetrics {
    pub concentration_mean: Option<f64>,
    pub concentration_std: Option<f64>,
    pub fragment_size_distribution: Option<HashMap<String, f64>>,
    pub quality_score_distribution: Option<HashMap<String, i32>>,
    pub success_rate: Option<f64>,
    pub throughput: Option<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QualityReport {
    pub library_id: Uuid,
    pub overall_status: QCStatus,
    pub metrics: Vec<QualityControlMetric>,
    pub recommendations: Vec<String>,
    pub generated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BatchLibraryRequest {
    pub libraries: Vec<CreateLibraryRequest>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProtocolRecommendation {
    pub protocol_id: Uuid,
    pub protocol_name: String,
    pub compatibility_score: f64,
    pub reasons: Vec<String>,
}