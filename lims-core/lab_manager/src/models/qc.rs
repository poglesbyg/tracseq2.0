use chrono::{DateTime, Utc, NaiveDate};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct QcMetricDefinition {
    pub id: Uuid,
    pub name: String,
    pub metric_type: String,
    pub data_type: String,
    pub unit: Option<String>,
    pub min_value: Option<sqlx::types::Decimal>,
    pub max_value: Option<sqlx::types::Decimal>,
    pub warning_threshold_low: Option<sqlx::types::Decimal>,
    pub warning_threshold_high: Option<sqlx::types::Decimal>,
    pub fail_threshold_low: Option<sqlx::types::Decimal>,
    pub fail_threshold_high: Option<sqlx::types::Decimal>,
    pub description: Option<String>,
    pub calculation_method: Option<String>,
    pub is_required: Option<bool>,
    pub is_active: Option<bool>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct LibraryPrepQc {
    pub id: Uuid,
    pub library_prep_id: Uuid,
    pub qc_batch_id: String,
    pub qc_date: NaiveDate,
    pub performed_by: Uuid,
    pub overall_status: String,
    pub concentration_ngul: Option<sqlx::types::Decimal>,
    pub volume_ul: Option<sqlx::types::Decimal>,
    pub total_yield_ng: Option<sqlx::types::Decimal>,
    pub fragment_size_bp: Option<i32>,
    pub fragment_size_cv: Option<sqlx::types::Decimal>,
    pub bioanalyzer_rin: Option<sqlx::types::Decimal>,
    pub bioanalyzer_trace_path: Option<String>,
    pub qubit_concentration: Option<sqlx::types::Decimal>,
    pub nanodrop_260_280: Option<sqlx::types::Decimal>,
    pub nanodrop_260_230: Option<sqlx::types::Decimal>,
    pub contamination_status: Option<String>,
    pub adapter_contamination: Option<sqlx::types::Decimal>,
    pub notes: Option<String>,
    pub raw_data: Option<serde_json::Value>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct SequencingRunQc {
    pub id: Uuid,
    pub sequencing_run_id: Uuid,
    pub flow_cell_id: Option<String>,
    pub qc_type: String,
    pub qc_timestamp: DateTime<Utc>,
    pub overall_status: String,
    pub cluster_density_k_per_mm2: Option<sqlx::types::Decimal>,
    pub cluster_pf_percent: Option<sqlx::types::Decimal>,
    pub phix_aligned_percent: Option<sqlx::types::Decimal>,
    pub error_rate_percent: Option<sqlx::types::Decimal>,
    pub q30_percent: Option<sqlx::types::Decimal>,
    pub total_reads_pf_m: Option<sqlx::types::Decimal>,
    pub total_yield_gb: Option<sqlx::types::Decimal>,
    pub intensity_cycle_1: Option<serde_json::Value>,
    pub index_metrics: Option<serde_json::Value>,
    pub lane_metrics: Option<serde_json::Value>,
    pub run_summary: Option<serde_json::Value>,
    pub alerts: Option<Vec<String>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct QcReview {
    pub id: Uuid,
    pub entity_type: String,
    pub entity_id: Uuid,
    pub review_type: String,
    pub reviewer_id: Option<Uuid>,
    pub review_status: String,
    pub decision: Option<String>,
    pub review_criteria: Option<serde_json::Value>,
    pub review_results: Option<serde_json::Value>,
    pub comments: Option<String>,
    pub conditions_for_approval: Option<String>,
    pub review_started_at: Option<DateTime<Utc>>,
    pub review_completed_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct QcMetricHistory {
    pub id: Uuid,
    pub metric_definition_id: Uuid,
    pub entity_type: String,
    pub entity_id: Uuid,
    pub metric_value: Option<sqlx::types::Decimal>,
    pub metric_value_json: Option<serde_json::Value>,
    pub status: String,
    pub recorded_at: DateTime<Utc>,
    pub recorded_by: Option<Uuid>,
    pub instrument_id: Option<String>,
    pub batch_id: Option<String>,
    pub notes: Option<String>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct QcControlSample {
    pub id: Uuid,
    pub control_type: String,
    pub control_name: String,
    pub lot_number: Option<String>,
    pub expected_values: serde_json::Value,
    pub tolerance_range: Option<serde_json::Value>,
    pub expiry_date: Option<NaiveDate>,
    pub storage_location: Option<String>,
    pub is_active: Option<bool>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct QcControlResult {
    pub id: Uuid,
    pub control_sample_id: Uuid,
    pub run_id: Uuid,
    pub run_type: String,
    pub measured_values: serde_json::Value,
    pub passed: bool,
    pub deviation_from_expected: Option<serde_json::Value>,
    pub notes: Option<String>,
    pub created_at: DateTime<Utc>,
}

// Request/Response DTOs
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateLibraryPrepQcRequest {
    pub library_prep_id: Uuid,
    pub qc_batch_id: String,
    pub qc_date: NaiveDate,
    pub concentration_ngul: Option<sqlx::types::Decimal>,
    pub volume_ul: Option<sqlx::types::Decimal>,
    pub fragment_size_bp: Option<i32>,
    pub fragment_size_cv: Option<sqlx::types::Decimal>,
    pub bioanalyzer_rin: Option<sqlx::types::Decimal>,
    pub bioanalyzer_trace_path: Option<String>,
    pub qubit_concentration: Option<sqlx::types::Decimal>,
    pub nanodrop_260_280: Option<sqlx::types::Decimal>,
    pub nanodrop_260_230: Option<sqlx::types::Decimal>,
    pub contamination_status: Option<String>,
    pub adapter_contamination: Option<sqlx::types::Decimal>,
    pub notes: Option<String>,
    pub raw_data: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateQcReviewRequest {
    pub entity_type: String,
    pub entity_id: Uuid,
    pub review_type: String,
    pub review_criteria: Option<serde_json::Value>,
    pub comments: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompleteQcReviewRequest {
    pub decision: String, // 'approved', 'rejected', 'conditional', 'repeat_required'
    pub review_results: Option<serde_json::Value>,
    pub comments: Option<String>,
    pub conditions_for_approval: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ListQcReviewsQuery {
    pub entity_type: Option<String>,
    pub entity_id: Option<Uuid>,
    pub review_status: Option<String>,
    pub reviewer_id: Option<Uuid>,
    pub decision: Option<String>,
    pub created_after: Option<DateTime<Utc>>,
    pub created_before: Option<DateTime<Utc>>,
    pub limit: Option<i64>,
    pub offset: Option<i64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QcDashboardStats {
    pub pending_reviews: i64,
    pub completed_today: i64,
    pub failed_today: i64,
    pub pass_rate_week: f64,
    pub average_turnaround_hours: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QcMetricTrend {
    pub metric_name: String,
    pub data_points: Vec<QcMetricDataPoint>,
    pub trend_direction: String, // 'up', 'down', 'stable'
    pub trend_percentage: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QcMetricDataPoint {
    pub timestamp: DateTime<Utc>,
    pub value: f64,
    pub status: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateControlSampleRequest {
    pub control_type: String,
    pub control_name: String,
    pub lot_number: Option<String>,
    pub expected_values: serde_json::Value,
    pub tolerance_range: Option<serde_json::Value>,
    pub expiry_date: Option<NaiveDate>,
    pub storage_location: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecordControlResultRequest {
    pub control_sample_id: Uuid,
    pub run_id: Uuid,
    pub run_type: String,
    pub measured_values: serde_json::Value,
    pub notes: Option<String>,
} 