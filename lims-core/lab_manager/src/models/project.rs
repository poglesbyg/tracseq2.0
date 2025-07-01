use chrono::{DateTime, Utc, NaiveDate};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Project {
    pub id: Uuid,
    pub project_code: String,
    pub name: String,
    pub description: Option<String>,
    pub project_type: String,
    pub status: String,
    pub priority: Option<String>,
    pub start_date: Option<NaiveDate>,
    pub target_end_date: Option<NaiveDate>,
    pub actual_end_date: Option<NaiveDate>,
    pub principal_investigator_id: Uuid,
    pub project_manager_id: Option<Uuid>,
    pub department: Option<String>,
    pub budget_approved: Option<sqlx::types::Decimal>,
    pub budget_used: Option<sqlx::types::Decimal>,
    pub metadata: Option<serde_json::Value>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub created_by: Uuid,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct ProjectTeamMember {
    pub id: Uuid,
    pub project_id: Uuid,
    pub user_id: Uuid,
    pub role: String,
    pub permissions: Option<serde_json::Value>,
    pub joined_at: DateTime<Utc>,
    pub left_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Batch {
    pub id: Uuid,
    pub batch_number: String,
    pub project_id: Uuid,
    pub batch_type: String,
    pub status: String,
    pub priority: Option<String>,
    pub sample_count: Option<i32>,
    pub metadata: Option<serde_json::Value>,
    pub notes: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub created_by: Uuid,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct BatchWorkflowStep {
    pub id: Uuid,
    pub batch_id: Uuid,
    pub step_order: i32,
    pub step_name: String,
    pub step_type: String,
    pub status: String,
    pub required: Option<bool>,
    pub assigned_to: Option<Uuid>,
    pub started_at: Option<DateTime<Utc>>,
    pub completed_at: Option<DateTime<Utc>>,
    pub completed_by: Option<Uuid>,
    pub results: Option<serde_json::Value>,
    pub notes: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct ProjectSignoff {
    pub id: Uuid,
    pub project_id: Uuid,
    pub batch_id: Option<Uuid>,
    pub signoff_type: String,
    pub signoff_level: String,
    pub status: String,
    pub required_by: Uuid,
    pub signed_by: Option<Uuid>,
    pub signed_at: Option<DateTime<Utc>>,
    pub comments: Option<String>,
    pub conditions: Option<serde_json::Value>,
    pub expiry_date: Option<NaiveDate>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct TemplateRepository {
    pub id: Uuid,
    pub name: String,
    pub category: String,
    pub file_type: String,
    pub version: String,
    pub description: Option<String>,
    pub file_path: String,
    pub file_size_bytes: Option<i64>,
    pub checksum: Option<String>,
    pub tags: Option<Vec<String>>,
    pub metadata: Option<serde_json::Value>,
    pub is_active: Option<bool>,
    pub download_count: Option<i32>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub created_by: Uuid,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct ProjectFile {
    pub id: Uuid,
    pub project_id: Uuid,
    pub batch_id: Option<Uuid>,
    pub parent_folder_id: Option<Uuid>,
    pub name: String,
    pub file_type: String,
    pub file_extension: Option<String>,
    pub file_path: String,
    pub file_size_bytes: Option<i64>,
    pub mime_type: Option<String>,
    pub checksum: Option<String>,
    pub metadata: Option<serde_json::Value>,
    pub is_deleted: Option<bool>,
    pub deleted_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub created_by: Uuid,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct PermissionQueue {
    pub id: Uuid,
    pub batch_id: Uuid,
    pub permission_type: String,
    pub requested_by: Uuid,
    pub requested_at: DateTime<Utc>,
    pub status: String,
    pub priority: Option<String>,
    pub reason: String,
    pub approved_by: Option<Uuid>,
    pub approved_at: Option<DateTime<Utc>>,
    pub rejection_reason: Option<String>,
    pub expires_at: Option<DateTime<Utc>>,
    pub metadata: Option<serde_json::Value>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

// Request/Response DTOs
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateProjectRequest {
    pub project_code: String,
    pub name: String,
    pub description: Option<String>,
    pub project_type: String,
    pub priority: Option<String>,
    pub start_date: Option<NaiveDate>,
    pub target_end_date: Option<NaiveDate>,
    pub principal_investigator_id: Uuid,
    pub project_manager_id: Option<Uuid>,
    pub department: Option<String>,
    pub budget_approved: Option<sqlx::types::Decimal>,
    pub metadata: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateProjectRequest {
    pub name: Option<String>,
    pub description: Option<String>,
    pub status: Option<String>,
    pub priority: Option<String>,
    pub start_date: Option<NaiveDate>,
    pub target_end_date: Option<NaiveDate>,
    pub actual_end_date: Option<NaiveDate>,
    pub project_manager_id: Option<Uuid>,
    pub department: Option<String>,
    pub budget_approved: Option<sqlx::types::Decimal>,
    pub metadata: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateBatchRequest {
    pub batch_number: String,
    pub project_id: Uuid,
    pub batch_type: String,
    pub priority: Option<String>,
    pub sample_count: Option<i32>,
    pub metadata: Option<serde_json::Value>,
    pub notes: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateSignoffRequest {
    pub project_id: Uuid,
    pub batch_id: Option<Uuid>,
    pub signoff_type: String,
    pub signoff_level: String,
    pub required_by: Uuid,
    pub comments: Option<String>,
    pub conditions: Option<serde_json::Value>,
    pub expiry_date: Option<NaiveDate>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SignoffDecisionRequest {
    pub status: String, // 'approved', 'rejected', 'conditional'
    pub comments: Option<String>,
    pub conditions: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ListProjectsQuery {
    pub status: Option<String>,
    pub priority: Option<String>,
    pub project_type: Option<String>,
    pub department: Option<String>,
    pub principal_investigator_id: Option<Uuid>,
    pub search: Option<String>,
    pub limit: Option<i64>,
    pub offset: Option<i64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ListBatchesQuery {
    pub project_id: Option<Uuid>,
    pub batch_type: Option<String>,
    pub status: Option<String>,
    pub search: Option<String>, // For batch number search
    pub limit: Option<i64>,
    pub offset: Option<i64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UploadFileRequest {
    pub project_id: Uuid,
    pub batch_id: Option<Uuid>,
    pub parent_folder_id: Option<Uuid>,
    pub name: String,
    pub file_type: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreatePermissionRequest {
    pub batch_id: Uuid,
    pub permission_type: String,
    pub priority: Option<String>,
    pub reason: String,
    pub expires_at: Option<DateTime<Utc>>,
    pub metadata: Option<serde_json::Value>,
} 