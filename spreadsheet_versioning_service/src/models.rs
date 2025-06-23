use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

// =============================================================================
// Version Models
// =============================================================================

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, sqlx::Type)]
#[sqlx(type_name = "version_status", rename_all = "lowercase")]
pub enum VersionStatus {
    Draft,
    Active,
    Archived,
    Deleted,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct SpreadsheetVersion {
    pub id: Uuid,
    pub spreadsheet_id: Uuid,
    pub version_number: i32,
    pub version_tag: Option<String>,
    pub status: VersionStatus,
    pub parent_version_id: Option<Uuid>,

    // Spreadsheet metadata
    pub name: String,
    pub filename: String,
    pub original_filename: String,
    pub file_type: String,
    pub file_size: i64,
    pub file_hash: String,

    // Version metadata
    pub changes_summary: Option<String>,
    pub change_count: Option<i32>,
    pub created_by: Option<Uuid>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,

    // Metadata
    pub metadata: serde_json::Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateVersionRequest {
    pub spreadsheet_id: Uuid,
    pub version_tag: Option<String>,
    pub parent_version_id: Option<Uuid>,
    pub name: String,
    pub filename: String,
    pub original_filename: String,
    pub file_type: String,
    pub file_data: Vec<u8>,
    pub changes_summary: Option<String>,
    pub metadata: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateVersionRequest {
    pub version_tag: Option<String>,
    pub status: Option<VersionStatus>,
    pub changes_summary: Option<String>,
    pub metadata: Option<serde_json::Value>,
}

// =============================================================================
// Version Data Models
// =============================================================================

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct VersionData {
    pub id: Uuid,
    pub version_id: Uuid,
    pub sheet_name: String,
    pub sheet_index: i32,
    pub row_index: i32,
    pub column_index: i32,
    pub column_name: Option<String>,
    pub cell_value: Option<String>,
    pub data_type: Option<String>,
    pub formatted_value: Option<String>,
    pub cell_formula: Option<String>,
    pub cell_metadata: serde_json::Value,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateVersionDataRequest {
    pub version_id: Uuid,
    pub sheet_name: String,
    pub sheet_index: i32,
    pub row_index: i32,
    pub column_index: i32,
    pub column_name: Option<String>,
    pub cell_value: Option<String>,
    pub data_type: Option<String>,
    pub formatted_value: Option<String>,
    pub cell_formula: Option<String>,
    pub cell_metadata: Option<serde_json::Value>,
}

// =============================================================================
// Diff Models
// =============================================================================

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum DiffType {
    CellChange,
    RowAdded,
    RowDeleted,
    ColumnAdded,
    ColumnDeleted,
    SheetAdded,
    SheetDeleted,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct VersionDiff {
    pub id: Uuid,
    pub from_version_id: Uuid,
    pub to_version_id: Uuid,
    pub diff_type: String,
    pub sheet_name: Option<String>,
    pub row_index: Option<i32>,
    pub column_index: Option<i32>,
    pub column_name: Option<String>,
    pub old_value: Option<String>,
    pub new_value: Option<String>,
    pub change_metadata: serde_json::Value,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiffRequest {
    pub from_version_id: Uuid,
    pub to_version_id: Uuid,
    pub diff_options: Option<DiffOptions>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiffOptions {
    pub ignore_whitespace: bool,
    pub ignore_case: bool,
    pub include_metadata: bool,
    pub detailed_changes: bool,
}

impl Default for DiffOptions {
    fn default() -> Self {
        Self {
            ignore_whitespace: false,
            ignore_case: false,
            include_metadata: true,
            detailed_changes: true,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiffResponse {
    pub from_version: SpreadsheetVersion,
    pub to_version: SpreadsheetVersion,
    pub diffs: Vec<VersionDiff>,
    pub summary: DiffSummary,
    pub generated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiffSummary {
    pub total_changes: usize,
    pub cell_changes: usize,
    pub row_changes: usize,
    pub column_changes: usize,
    pub sheet_changes: usize,
    pub structural_changes: usize,
}

// =============================================================================
// Conflict Models
// =============================================================================

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, sqlx::Type)]
#[sqlx(type_name = "conflict_status", rename_all = "lowercase")]
pub enum ConflictStatus {
    Pending,
    Resolved,
    Rejected,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum ConflictType {
    CellConflict,
    StructuralConflict,
    MetadataConflict,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct VersionConflict {
    pub id: Uuid,
    pub base_version_id: Uuid,
    pub version_a_id: Uuid,
    pub version_b_id: Uuid,

    pub conflict_type: String,
    pub sheet_name: Option<String>,
    pub row_index: Option<i32>,
    pub column_index: Option<i32>,
    pub column_name: Option<String>,

    pub value_a: Option<String>,
    pub value_b: Option<String>,
    pub base_value: Option<String>,

    pub status: ConflictStatus,
    pub resolution_strategy: Option<String>,
    pub resolved_value: Option<String>,
    pub resolved_by: Option<Uuid>,
    pub resolved_at: Option<DateTime<Utc>>,

    pub conflict_metadata: serde_json::Value,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConflictDetectionRequest {
    pub base_version_id: Uuid,
    pub version_a_id: Uuid,
    pub version_b_id: Uuid,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConflictResolutionRequest {
    pub conflict_id: Uuid,
    pub resolution_strategy: String,
    pub resolved_value: Option<String>,
    pub resolution_notes: Option<String>,
}

// =============================================================================
// Merge Models
// =============================================================================

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct VersionMergeRequest {
    pub id: Uuid,
    pub source_version_id: Uuid,
    pub target_version_id: Uuid,
    pub merged_version_id: Option<Uuid>,

    pub title: String,
    pub description: Option<String>,
    pub status: String,

    pub requested_by: Uuid,
    pub reviewed_by: Option<Uuid>,
    pub merged_by: Option<Uuid>,

    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub reviewed_at: Option<DateTime<Utc>>,
    pub merged_at: Option<DateTime<Utc>>,

    pub merge_metadata: serde_json::Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateMergeRequest {
    pub source_version_id: Uuid,
    pub target_version_id: Uuid,
    pub title: String,
    pub description: Option<String>,
    pub requested_by: Uuid,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MergeRequest {
    pub source_version_id: Uuid,
    pub target_version_id: Uuid,
    pub merge_strategy: MergeStrategy,
    pub conflict_resolution: Option<Vec<ConflictResolutionRequest>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MergeStrategy {
    AutoMerge,
    ManualReview,
    SourceWins,
    TargetWins,
    CustomStrategy(String),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MergeResponse {
    pub merged_version_id: Uuid,
    pub conflicts: Vec<VersionConflict>,
    pub merge_summary: MergeSummary,
    pub merged_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MergeSummary {
    pub total_changes_applied: usize,
    pub conflicts_resolved: usize,
    pub conflicts_remaining: usize,
    pub merge_strategy_used: String,
}

// =============================================================================
// Utility Models
// =============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VersionListResponse {
    pub versions: Vec<SpreadsheetVersion>,
    pub total_count: usize,
    pub page: usize,
    pub per_page: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConflictListResponse {
    pub conflicts: Vec<VersionConflict>,
    pub total_count: usize,
    pub page: usize,
    pub per_page: usize,
}
