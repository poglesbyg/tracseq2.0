use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::Json,
};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::{
    assembly::AppComponents,
    errors::api::ApiError,
    models::storage::{StorageLocation, TemperatureZone, ContainerType},
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
    let locations = app.storage_management_service()
        .list_storage_locations()
        .await
        .map_err(|e| ApiError::InternalServerError(format!("Failed to fetch storage locations: {}", e)))?;

    let location_info: Vec<StorageLocationInfo> = locations
        .into_iter()
        .map(StorageLocationInfo::from)
        .collect();

    Ok(Json(location_info))
}

/// Update a storage location
pub async fn update_storage_location(
    State(app): State<AppComponents>,
    Path(location_id): Path<i32>,
    Json(update_data): Json<UpdateStorageLocationRequest>,
) -> Result<Json<StorageLocationInfo>, ApiError> {
    let updated_location = app.storage_management_service()
        .update_storage_location(location_id, update_data.into())
        .await
        .map_err(|e| ApiError::InternalServerError(format!("Failed to update storage location: {}", e)))?;

    Ok(Json(StorageLocationInfo::from(updated_location)))
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