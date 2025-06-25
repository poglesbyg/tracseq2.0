use axum::{
    extract::{Path, State},
    response::Json,
};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::{
    assembly::AppComponents,
    errors::api::ApiError,
    models::storage::{StorageLocation, TemperatureZone, ContainerType, StorageState, StorageCapacityStats},
    repositories::storage_repository::CreateStorageLocation,
};

/// Storage location information for API responses
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StorageLocationInfo {
    pub id: i32,
    pub name: String,
    pub description: Option<String>,
    pub temperature_zone: TemperatureZone,
    pub capacity: i32,
    pub current_usage: i32,
    pub available_capacity: i32,
    pub utilization_percentage: f64,
    pub container_type: ContainerType,
    pub is_active: bool,
    pub location_path: Option<String>,
}

impl From<StorageLocation> for StorageLocationInfo {
    fn from(location: StorageLocation) -> Self {
        let available_capacity = location.available_capacity();
        let utilization_percentage = location.utilization_percentage();
        
        Self {
            id: location.id,
            name: location.name,
            description: location.description,
            temperature_zone: location.temperature_zone,
            capacity: location.capacity,
            current_usage: location.current_usage,
            available_capacity,
            utilization_percentage,
            container_type: location.container_type,
            is_active: location.is_active,
            location_path: location.location_path,
        }
    }
}

/// Get all storage locations
pub async fn get_storage_locations(
    State(app): State<AppComponents>,
) -> Result<Json<Vec<StorageLocationInfo>>, ApiError> {
    let locations = app.storage_management_service
        .list_storage_locations()
        .await
        .map_err(|e| ApiError::InternalServerError(format!("Failed to fetch storage locations: {}", e)))?;

    let location_info: Vec<StorageLocationInfo> = locations
        .into_iter()
        .map(StorageLocationInfo::from)
        .collect();

    Ok(Json(location_info))
}

/// Create a new storage location
pub async fn create_storage_location(
    State(app): State<AppComponents>,
    Json(create_data): Json<CreateStorageLocationRequest>,
) -> Result<Json<StorageLocationInfo>, ApiError> {
    let create_location = CreateStorageLocation {
        name: create_data.name,
        description: create_data.description,
        temperature_zone: create_data.temperature_zone,
        capacity: create_data.capacity,
        container_type: create_data.container_type,
        location_path: create_data.location_path,
    };

    let location = app.storage_management_service
        .create_storage_location(create_location)
        .await
        .map_err(|e| ApiError::InternalServerError(format!("Failed to create storage location: {}", e)))?;

    Ok(Json(StorageLocationInfo::from(location)))
}

/// Update a storage location
pub async fn update_storage_location(
    State(app): State<AppComponents>,
    Path(location_id): Path<i32>,
    Json(update_data): Json<UpdateStorageLocationRequest>,
) -> Result<Json<StorageLocationInfo>, ApiError> {
    let updated_location = app.storage_management_service
        .update_storage_location(location_id, update_data.into())
        .await
        .map_err(|e| ApiError::InternalServerError(format!("Failed to update storage location: {}", e)))?;

    Ok(Json(StorageLocationInfo::from(updated_location)))
}

/// Store a sample in a storage location
pub async fn store_sample(
    State(app): State<AppComponents>,
    Json(store_data): Json<StoreSampleRequest>,
) -> Result<Json<SampleLocationResponse>, ApiError> {
    let stored_by = store_data.stored_by.clone().unwrap_or_else(|| "system".to_string());
    let position = store_data.position.clone();

    let stored_result = app.storage_management_service
        .store_sample(
            store_data.sample_id,
            store_data.location_id,
            "unknown", // sample_type - would need to be provided in request
            None, // template_name
            &stored_by,
            position,
            None, // requirements
        )
        .await
        .map_err(|e| ApiError::InternalServerError(format!("Failed to store sample: {}", e)))?;

    Ok(Json(SampleLocationResponse::from(stored_result.sample_location)))
}

/// Move a sample to a different location
pub async fn move_sample(
    State(app): State<AppComponents>,
    Json(move_data): Json<MoveSampleRequest>,
) -> Result<Json<SampleLocationResponse>, ApiError> {
    let sample_location = app.storage_management_service
        .move_sample(
            move_data.sample_id,
            move_data.new_location_id,
            &move_data.moved_by,
            &move_data.reason,
            None, // requirements
        )
        .await
        .map_err(|e| ApiError::InternalServerError(format!("Failed to move sample: {}", e)))?;

    Ok(Json(SampleLocationResponse::from(sample_location)))
}

/// Remove a sample from storage
pub async fn remove_sample(
    State(app): State<AppComponents>,
    Json(remove_data): Json<RemoveSampleRequest>,
) -> Result<Json<SampleLocationResponse>, ApiError> {
    let removed_result = app.storage_management_service
        .remove_sample(
            &remove_data.sample_id.to_string(), // Convert to barcode string - would need proper barcode lookup
            &remove_data.removed_by,
            &remove_data.reason,
        )
        .await
        .map_err(|e| ApiError::InternalServerError(format!("Failed to remove sample: {}", e)))?;

    Ok(Json(SampleLocationResponse::from(removed_result.sample_location)))
}

/// Scan a sample barcode to get its location
pub async fn scan_sample_barcode(
    State(app): State<AppComponents>,
    Path(barcode): Path<String>,
) -> Result<Json<SampleLocationResponse>, ApiError> {
    let scan_result = app.storage_management_service
        .scan_barcode(&barcode)
        .await
        .map_err(|e| ApiError::InternalServerError(format!("Failed to scan barcode: {}", e)))?;

    Ok(Json(SampleLocationResponse::from(scan_result.sample_location)))
}

/// Get storage capacity overview
pub async fn get_capacity_overview(
    State(app): State<AppComponents>,
) -> Result<Json<Vec<StorageCapacityInfo>>, ApiError> {
    let capacity_overview = app.storage_management_service
        .get_capacity_overview()
        .await
        .map_err(|e| ApiError::InternalServerError(format!("Failed to get capacity stats: {}", e)))?;

    let capacity_info: Vec<StorageCapacityInfo> = capacity_overview.location_stats
        .into_iter()
        .map(StorageCapacityInfo::from)
        .collect();

    Ok(Json(capacity_info))
}

// Request/Response structures

/// Request structure for creating storage locations
#[derive(Debug, Deserialize)]
pub struct CreateStorageLocationRequest {
    pub name: String,
    pub description: Option<String>,
    pub temperature_zone: TemperatureZone,
    pub capacity: i32,
    pub container_type: ContainerType,
    pub location_path: Option<String>,
}

/// Request structure for updating storage locations
#[derive(Debug, Deserialize)]
pub struct UpdateStorageLocationRequest {
    pub name: Option<String>,
    pub description: Option<String>,
    pub capacity: Option<i32>,
    pub is_active: Option<bool>,
    pub location_path: Option<String>,
}

/// Request structure for storing samples
#[derive(Debug, Deserialize)]
pub struct StoreSampleRequest {
    pub sample_id: Uuid,
    pub location_id: i32,
    pub barcode: String,
    pub position: Option<String>,
    pub storage_state: Option<StorageState>,
    pub stored_by: Option<String>,
    pub notes: Option<String>,
}

/// Request structure for moving samples
#[derive(Debug, Deserialize)]
pub struct MoveSampleRequest {
    pub sample_id: Uuid,
    pub new_location_id: i32,
    pub moved_by: String,
    pub reason: String,
}

/// Request structure for removing samples
#[derive(Debug, Deserialize)]
pub struct RemoveSampleRequest {
    pub sample_id: Uuid,
    pub removed_by: String,
    pub reason: String,
}

/// Response structure for sample locations
#[derive(Debug, Serialize)]
pub struct SampleLocationResponse {
    pub sample_id: Uuid,
    pub location_id: i32,
    pub barcode: String,
    pub position: Option<String>,
    pub storage_state: StorageState,
    pub stored_by: Option<String>,
    pub stored_at: chrono::DateTime<chrono::Utc>,
    pub moved_by: Option<String>,
    pub moved_at: Option<chrono::DateTime<chrono::Utc>>,
    pub notes: Option<String>,
}

impl From<crate::models::storage::SampleLocation> for SampleLocationResponse {
    fn from(location: crate::models::storage::SampleLocation) -> Self {
        Self {
            sample_id: location.sample_id,
            location_id: location.location_id,
            barcode: location.barcode,
            position: location.position,
            storage_state: location.storage_state,
            stored_by: location.stored_by,
            stored_at: location.stored_at,
            moved_by: location.moved_by,
            moved_at: location.moved_at,
            notes: location.notes,
        }
    }
}

/// Response structure for storage capacity information
#[derive(Debug, Serialize)]
pub struct StorageCapacityInfo {
    pub location_id: i32,
    pub location_name: String,
    pub temperature_zone: TemperatureZone,
    pub total_capacity: i32,
    pub current_usage: i32,
    pub available_capacity: i32,
    pub utilization_percentage: f64,
    pub sample_count: i32,
    pub is_near_capacity: bool,
}

impl From<StorageCapacityStats> for StorageCapacityInfo {
    fn from(stats: StorageCapacityStats) -> Self {
        Self {
            location_id: stats.location_id,
            location_name: stats.location_name,
            temperature_zone: stats.temperature_zone,
            total_capacity: stats.total_capacity,
            current_usage: stats.current_usage,
            available_capacity: stats.available_capacity,
            utilization_percentage: stats.utilization_percentage,
            sample_count: stats.sample_count,
            is_near_capacity: stats.is_near_capacity,
        }
    }
}

/// Storage location update data for the service layer
pub struct StorageLocationUpdate {
    pub name: Option<String>,
    pub description: Option<String>,
    pub capacity: Option<i32>,
    pub is_active: Option<bool>,
    pub location_path: Option<String>,
}

impl From<UpdateStorageLocationRequest> for StorageLocationUpdate {
    fn from(request: UpdateStorageLocationRequest) -> Self {
        Self {
            name: request.name,
            description: request.description,
            capacity: request.capacity,
            is_active: request.is_active,
            location_path: request.location_path,
        }
    }
}

impl From<UpdateStorageLocationRequest> for crate::repositories::storage_repository::UpdateStorageLocation {
    fn from(request: UpdateStorageLocationRequest) -> Self {
        Self {
            name: request.name,
            description: request.description,
            capacity: request.capacity,
            is_active: request.is_active,
            location_path: request.location_path,
        }
    }
}