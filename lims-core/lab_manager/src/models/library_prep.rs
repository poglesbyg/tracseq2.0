use chrono::{DateTime, Utc, NaiveDate};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct LibraryPrepProtocol {
    pub id: Uuid,
    pub name: String,
    pub version: String,
    pub protocol_type: String,
    pub kit_name: Option<String>,
    pub kit_manufacturer: Option<String>,
    pub input_requirements: serde_json::Value,
    pub protocol_steps: serde_json::Value,
    pub reagents: serde_json::Value,
    pub equipment_required: Option<Vec<String>>,
    pub estimated_duration_hours: Option<sqlx::types::Decimal>,
    pub is_active: Option<bool>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub created_by: Option<Uuid>,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct LibraryPreparation {
    pub id: Uuid,
    pub batch_id: String,
    pub project_id: Uuid,
    pub protocol_id: Uuid,
    pub sample_ids: Vec<Uuid>,
    pub status: String,
    pub prep_date: NaiveDate,
    pub operator_id: Uuid,
    pub input_metrics: Option<serde_json::Value>,
    pub output_metrics: Option<serde_json::Value>,
    pub reagent_lots: Option<serde_json::Value>,
    pub notes: Option<String>,
    pub qc_status: Option<String>,
    pub qc_metrics: Option<serde_json::Value>,
    pub completed_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

// Request/Response DTOs
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateLibraryPrepProtocolRequest {
    pub name: String,
    pub version: String,
    pub protocol_type: String,
    pub kit_name: Option<String>,
    pub kit_manufacturer: Option<String>,
    pub input_requirements: serde_json::Value,
    pub protocol_steps: serde_json::Value,
    pub reagents: serde_json::Value,
    pub equipment_required: Option<Vec<String>>,
    pub estimated_duration_hours: Option<sqlx::types::Decimal>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateLibraryPrepProtocolRequest {
    pub name: Option<String>,
    pub version: Option<String>,
    pub kit_name: Option<String>,
    pub kit_manufacturer: Option<String>,
    pub input_requirements: Option<serde_json::Value>,
    pub protocol_steps: Option<serde_json::Value>,
    pub reagents: Option<serde_json::Value>,
    pub equipment_required: Option<Vec<String>>,
    pub estimated_duration_hours: Option<sqlx::types::Decimal>,
    pub is_active: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateLibraryPreparationRequest {
    pub batch_id: String,
    pub project_id: Uuid,
    pub protocol_id: Uuid,
    pub sample_ids: Vec<Uuid>,
    pub prep_date: NaiveDate,
    pub operator_id: Uuid,
    pub input_metrics: Option<serde_json::Value>,
    pub reagent_lots: Option<serde_json::Value>,
    pub notes: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateLibraryPreparationRequest {
    pub status: Option<String>,
    pub input_metrics: Option<serde_json::Value>,
    pub output_metrics: Option<serde_json::Value>,
    pub reagent_lots: Option<serde_json::Value>,
    pub notes: Option<String>,
    pub qc_status: Option<String>,
    pub qc_metrics: Option<serde_json::Value>,
    pub completed_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ListLibraryPreparationsQuery {
    pub project_id: Option<Uuid>,
    pub protocol_id: Option<Uuid>,
    pub status: Option<String>,
    pub qc_status: Option<String>,
    pub operator_id: Option<Uuid>,
    pub prep_date_from: Option<NaiveDate>,
    pub prep_date_to: Option<NaiveDate>,
    pub batch_search: Option<String>,
    pub limit: Option<i64>,
    pub offset: Option<i64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LibraryPrepWithProtocol {
    pub preparation: LibraryPreparation,
    pub protocol: LibraryPrepProtocol,
    pub project_name: Option<String>,
    pub operator_name: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LibraryPrepMetrics {
    pub concentration_ngul: Option<f64>,
    pub volume_ul: Option<f64>,
    pub total_yield_ng: Option<f64>,
    pub fragment_size_bp: Option<i32>,
    pub fragment_size_cv: Option<f64>,
    pub quality_score: Option<f64>,
} 