use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;
use bigdecimal::BigDecimal;

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct FlowCellType {
    pub id: Uuid,
    pub name: String,
    pub manufacturer: String,
    pub model: String,
    pub lane_count: i32,
    pub reads_per_lane: Option<i64>,
    pub chemistry_version: Option<String>,
    pub compatible_sequencers: Option<Vec<String>>,
    pub specifications: Option<serde_json::Value>,
    pub is_active: Option<bool>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct FlowCellDesign {
    pub id: Uuid,
    pub name: String,
    pub flow_cell_type_id: Uuid,
    pub project_id: Uuid,
    pub sequencing_run_id: Option<Uuid>,
    pub design_version: i32,
    pub status: String,
    pub lane_assignments: serde_json::Value,
    pub pooling_strategy: Option<serde_json::Value>,
    pub expected_coverage: Option<serde_json::Value>,
    pub ai_optimization_score: Option<BigDecimal>,
    pub ai_suggestions: Option<serde_json::Value>,
    pub approved_by: Option<Uuid>,
    pub approved_at: Option<DateTime<Utc>>,
    pub notes: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub created_by: Uuid,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct FlowCellLane {
    pub id: Uuid,
    pub flow_cell_design_id: Uuid,
    pub lane_number: i32,
    pub library_prep_ids: Vec<Uuid>,
    pub sample_sheet_data: Option<serde_json::Value>,
    pub target_reads: Option<i64>,
    pub index_type: Option<String>,
    pub index_sequences: Option<serde_json::Value>,
    pub loading_concentration_pm: Option<BigDecimal>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

// Request/Response DTOs
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateFlowCellDesignRequest {
    pub name: String,
    pub flow_cell_type_id: Uuid,
    pub project_id: Uuid,
    pub lane_assignments: Vec<LaneAssignmentRequest>,
    pub pooling_strategy: Option<serde_json::Value>,
    pub expected_coverage: Option<serde_json::Value>,
    pub notes: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateFlowCellDesignRequest {
    pub name: Option<String>,
    pub status: Option<String>,
    pub lane_assignments: Option<Vec<LaneAssignmentRequest>>,
    pub pooling_strategy: Option<serde_json::Value>,
    pub expected_coverage: Option<serde_json::Value>,
    pub notes: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LaneAssignmentRequest {
    pub lane_number: i32,
    pub library_prep_ids: Vec<Uuid>,
    pub target_reads: Option<i64>,
    pub index_type: Option<String>,
    pub index_sequences: Option<Vec<IndexSequence>>,
    pub loading_concentration_pm: Option<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IndexSequence {
    pub index_name: String,
    pub i7_sequence: Option<String>,
    pub i5_sequence: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApproveFlowCellDesignRequest {
    pub comments: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ListFlowCellDesignsQuery {
    pub project_id: Option<Uuid>,
    pub flow_cell_type_id: Option<Uuid>,
    pub status: Option<String>,
    pub created_by: Option<Uuid>,
    pub approved_by: Option<Uuid>,
    pub limit: Option<i64>,
    pub offset: Option<i64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FlowCellDesignWithDetails {
    pub design: FlowCellDesign,
    pub flow_cell_type: FlowCellType,
    pub lanes: Vec<FlowCellLane>,
    pub project_name: Option<String>,
    pub creator_name: Option<String>,
    pub approver_name: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct OptimizeFlowCellRequest {
    pub flow_cell_type_id: Uuid,
    pub libraries: Vec<LibraryOptimizationInfo>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct LibraryOptimizationInfo {
    pub id: Uuid,
    pub concentration: f64,
    pub fragment_size: i32,
    pub index_type: String,
    pub project_id: Option<Uuid>,
    pub priority: Option<i32>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct OptimizeFlowCellResponse {
    pub lane_assignments: Vec<LaneAssignment>,
    pub optimization_score: f64,
    pub balance_score: f64,
    pub index_diversity_score: f64,
    pub suggestions: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LaneAssignment {
    pub lane_number: i32,
    pub library_prep_ids: Vec<Uuid>,
    pub target_reads: i64,
    pub loading_concentration: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FlowCellMetrics {
    pub total_reads: i64,
    pub reads_per_sample: Vec<SampleReadCount>,
    pub lane_balance_score: f64,
    pub index_balance_score: f64,
    pub estimated_cost: Option<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SampleReadCount {
    pub library_prep_id: Uuid,
    pub expected_reads: i64,
    pub expected_coverage: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AlternativeDesign {
    pub design_name: String,
    pub optimization_score: f64,
    pub lane_assignments: Vec<LaneAssignmentRequest>,
    pub trade_offs: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FlowCellTypeStats {
    pub flow_cell_type: FlowCellType,
    pub total_runs: i64,
    pub average_yield_gb: f64,
    pub average_q30_percent: f64,
    pub success_rate: f64,
} 