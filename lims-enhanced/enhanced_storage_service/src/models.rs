use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;
use validator::Validate;

// ============================================================================
// Storage Location Models (existing)
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct StorageLocation {
    pub id: Uuid,
    pub name: String,
    pub description: Option<String>,
    pub location_type: String,
    pub temperature_zone: String,
    pub max_capacity: i32,
    pub current_capacity: i32,
    pub coordinates: Option<serde_json::Value>,
    pub status: String,
    pub metadata: serde_json::Value,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct CreateStorageLocationRequest {
    #[validate(length(min = 1, max = 255))]
    pub name: String,
    pub description: Option<String>,
    #[validate(length(min = 1))]
    pub location_type: String,
    #[validate(length(min = 1))]
    pub temperature_zone: String,
    #[validate(range(min = 1))]
    pub max_capacity: i32,
    pub coordinates: Option<serde_json::Value>,
    pub metadata: Option<serde_json::Value>,
}

#[derive(Debug, Serialize, Deserialize, Validate)]
pub struct UpdateStorageLocationRequest {
    pub name: Option<String>,
    pub description: Option<String>,
    pub location_type: Option<String>,
    pub temperature_zone: Option<String>,
    pub max_capacity: Option<i32>,
    pub coordinates: Option<serde_json::Value>,
    pub status: Option<String>,
    pub metadata: Option<serde_json::Value>,
}

// ============================================================================
// Hierarchical Storage Container Models (new)
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct StorageContainer {
    pub id: Uuid,
    pub name: String,
    pub container_type: String, // 'freezer', 'rack', 'box', 'position'
    pub parent_container_id: Option<Uuid>,
    pub location_id: Option<Uuid>,
    pub grid_position: Option<serde_json::Value>,
    pub dimensions: Option<serde_json::Value>,
    pub capacity: i32,
    pub occupied_count: i32,
    pub temperature_zone: Option<String>,
    pub barcode: Option<String>,
    pub description: Option<String>,
    pub status: String,
    pub installation_date: Option<DateTime<Utc>>,
    pub last_maintenance_date: Option<DateTime<Utc>>,
    pub next_maintenance_date: Option<DateTime<Utc>>,
    pub container_metadata: serde_json::Value,
    pub access_restrictions: serde_json::Value,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub created_by: Option<Uuid>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct CreateStorageContainerRequest {
    #[validate(length(min = 1, max = 255))]
    pub name: String,
    #[validate(length(min = 1))]
    pub container_type: String, // 'freezer', 'rack', 'box', 'position'
    pub parent_container_id: Option<Uuid>,
    pub location_id: Option<Uuid>,
    pub grid_position: Option<serde_json::Value>,
    pub dimensions: Option<serde_json::Value>,
    #[validate(range(min = 1))]
    pub capacity: i32,
    pub temperature_zone: Option<String>,
    pub barcode: Option<String>,
    pub description: Option<String>,
    pub container_metadata: Option<serde_json::Value>,
    pub access_restrictions: Option<serde_json::Value>,
}

#[derive(Debug, Serialize, Deserialize, Validate)]
pub struct UpdateStorageContainerRequest {
    pub name: Option<String>,
    pub grid_position: Option<serde_json::Value>,
    pub dimensions: Option<serde_json::Value>,
    pub capacity: Option<i32>,
    pub temperature_zone: Option<String>,
    pub barcode: Option<String>,
    pub description: Option<String>,
    pub status: Option<String>,
    pub last_maintenance_date: Option<DateTime<Utc>>,
    pub next_maintenance_date: Option<DateTime<Utc>>,
    pub container_metadata: Option<serde_json::Value>,
    pub access_restrictions: Option<serde_json::Value>,
}

// ============================================================================
// Sample Position Models (new)
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct SamplePosition {
    pub id: Uuid,
    pub sample_id: Uuid,
    pub container_id: Uuid,
    pub position_identifier: Option<String>,
    pub assigned_at: DateTime<Utc>,
    pub assigned_by: Option<Uuid>,
    pub removed_at: Option<DateTime<Utc>>,
    pub removed_by: Option<Uuid>,
    pub status: String, // 'available', 'occupied', 'reserved', 'maintenance'
    pub reservation_expires_at: Option<DateTime<Utc>>,
    pub storage_conditions: serde_json::Value,
    pub special_requirements: serde_json::Value,
    pub chain_of_custody: serde_json::Value,
    pub position_metadata: serde_json::Value,
    pub notes: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct AssignSampleToPositionRequest {
    pub sample_id: Uuid,
    pub container_id: Uuid, // Must be a 'position' type container
    pub position_identifier: Option<String>,
    pub assigned_by: Option<Uuid>,
    pub storage_conditions: Option<serde_json::Value>,
    pub special_requirements: Option<serde_json::Value>,
    pub notes: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct MoveSampleRequest {
    pub new_container_id: Uuid,
    pub new_position_identifier: Option<String>,
    pub moved_by: Option<Uuid>,
    pub reason: Option<String>,
    pub notes: Option<String>,
}

// ============================================================================
// Hierarchy and Navigation Models (new)
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct StorageHierarchy {
    pub id: Uuid,
    pub name: String,
    pub container_type: String,
    pub parent_container_id: Option<Uuid>,
    pub location_id: Option<Uuid>,
    pub grid_position: Option<serde_json::Value>,
    pub capacity: i32,
    pub occupied_count: i32,
    pub temperature_zone: Option<String>,
    pub barcode: Option<String>,
    pub status: String,
    pub level: i32,
    pub path: Vec<String>,
    pub full_path: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct StorageCapacitySummary {
    pub id: Uuid,
    pub name: String,
    pub container_type: String,
    pub capacity: i32,
    pub occupied_count: i32,
    pub available_count: i32,
    pub utilization_percentage: f64,
    pub capacity_status: String, // 'normal', 'warning', 'critical'
    pub temperature_zone: Option<String>,
    pub status: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct SampleLocationDetailed {
    pub position_id: Uuid,
    pub sample_id: Uuid,
    pub position_identifier: Option<String>,
    pub assigned_at: DateTime<Utc>,
    pub assigned_by: Option<Uuid>,
    pub position_status: String,
    pub storage_conditions: serde_json::Value,
    pub special_requirements: serde_json::Value,
    pub notes: Option<String>,
    
    // Container information
    pub container_id: Uuid,
    pub container_name: String,
    pub container_type: String,
    pub container_barcode: Option<String>,
    pub temperature_zone: Option<String>,
    
    // Hierarchy information
    pub full_path: String,
    pub level: i32,
    pub path: Vec<String>,
    
    // Location information
    pub location_name: Option<String>,
    pub location_zone_type: Option<String>,
}

// ============================================================================
// Container Navigation Models (new)
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContainerWithChildren {
    pub container: StorageContainer,
    pub children: Vec<StorageContainer>,
    pub samples: Vec<SamplePosition>,
    pub capacity_info: ContainerCapacityInfo,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContainerCapacityInfo {
    pub total_capacity: i32,
    pub occupied_count: i32,
    pub available_count: i32,
    pub utilization_percentage: f64,
    pub capacity_status: String,
    pub child_containers_count: i32,
    pub direct_samples_count: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StorageGridView {
    pub container_id: Uuid,
    pub container_name: String,
    pub container_type: String,
    pub grid_dimensions: GridDimensions,
    pub positions: Vec<GridPosition>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GridDimensions {
    pub rows: i32,
    pub columns: i32,
    pub total_positions: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GridPosition {
    pub container_id: Uuid,
    pub position_identifier: String,
    pub row: i32,
    pub column: i32,
    pub is_occupied: bool,
    pub sample_id: Option<Uuid>,
    pub sample_barcode: Option<String>,
    pub sample_type: Option<String>,
    pub status: String,
    pub temperature_zone: Option<String>,
}

// ============================================================================
// Sample Models (existing, updated)
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Sample {
    pub id: Uuid,
    pub barcode: String,
    pub sample_type: String,
    pub storage_location_id: Option<Uuid>,
    pub position: Option<serde_json::Value>,
    pub temperature_requirements: Option<String>,
    pub status: String,
    pub metadata: serde_json::Value,
    pub chain_of_custody: serde_json::Value,
    pub stored_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct StoreSampleRequest {
    #[validate(length(min = 1, max = 255))]
    pub barcode: String,
    #[validate(length(min = 1))]
    pub sample_type: String,
    pub storage_location_id: Option<Uuid>,
    pub position: Option<serde_json::Value>,
    pub temperature_requirements: Option<String>,
    pub metadata: Option<serde_json::Value>,
}

// ============================================================================
// API Response Models (existing)
// ============================================================================

#[derive(Debug, Serialize, Deserialize)]
pub struct ApiResponse<T> {
    pub success: bool,
    pub data: T,
    pub message: Option<String>,
    pub timestamp: DateTime<Utc>,
}

impl<T> ApiResponse<T> {
    pub fn success(data: T) -> Self {
        Self {
            success: true,
            data,
            message: None,
            timestamp: Utc::now(),
        }
    }

    pub fn success_with_message(data: T, message: String) -> Self {
        Self {
            success: true,
            data,
            message: Some(message),
            timestamp: Utc::now(),
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PaginatedResponse<T> {
    pub data: Vec<T>,
    pub pagination: PaginationInfo,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PaginationInfo {
    pub page: i32,
    pub per_page: i32,
    pub total_pages: i32,
    pub total_items: i64,
    pub has_next: bool,
    pub has_prev: bool,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CapacityInfo {
    pub location_id: Uuid,
    pub max_capacity: i32,
    pub current_capacity: i32,
    pub utilization_percentage: f64,
    pub available_capacity: i32,
    pub status: String,
}
