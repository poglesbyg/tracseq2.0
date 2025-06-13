use axum::{
    extract::{Path, State},
    http::StatusCode,
    Json,
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;

use crate::models::storage::{StorageRequirement, TemperatureZone};
use crate::repositories::storage_repository::{
    CreateStorageLocation, PostgresStorageRepository, StorageRepository,
};
use crate::services::storage_management_service::{
    CapacityOverview, StorageManagementError, StorageManagementService,
};
use crate::AppComponents;

// Response DTOs for API
#[derive(Debug, Serialize, Deserialize)]
pub struct StorageLocationResponse {
    pub id: i32,
    pub name: String,
    pub description: Option<String>,
    pub temperature_zone: String,
    pub capacity: i32,
    pub available: i32,
    pub utilization_percentage: f64,
    pub is_active: bool,
    pub samples: Vec<StoredSampleResponse>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct StoredSampleResponse {
    pub id: i32,
    pub sample_id: i32,
    pub name: String,
    pub barcode: String,
    pub position: Option<String>,
    pub storage_state: String,
    pub stored_at: String,
    pub stored_by: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct MoveSampleRequest {
    pub barcode: String,
    pub location_id: i32,
    pub reason: Option<String>,
    pub moved_by: String,
}

#[derive(Debug, Serialize)]
pub struct MoveSampleResponse {
    pub success: bool,
    pub message: String,
    pub new_location: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct RemoveSampleRequest {
    pub barcode: String,
    pub reason: Option<String>,
    pub removed_by: String,
}

#[derive(Debug, Serialize)]
pub struct RemoveSampleResponse {
    pub success: bool,
    pub message: String,
    pub removed_sample: RemovedSampleInfo,
}

#[derive(Debug, Serialize)]
pub struct RemovedSampleInfo {
    pub sample_id: i32,
    pub barcode: String,
    pub location_name: String,
    pub removed_at: String,
    pub removed_by: String,
}

#[derive(Debug, Serialize)]
pub struct ScannedSampleInfo {
    pub sample_id: i32,
    pub barcode: String,
    pub location_name: String,
    pub location_id: i32,
    pub temperature_zone: String,
    pub position: Option<String>,
    pub storage_state: String,
    pub stored_at: String,
    pub stored_by: Option<String>,
    pub notes: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct StoreSampleRequest {
    pub sample_id: i32,
    pub location_id: i32,
    pub sample_type: String,
    pub template_name: Option<String>,
    pub stored_by: String,
    pub position: Option<String>,
    pub temperature_requirement: Option<String>,
    pub special_conditions: Option<Vec<String>>,
}

#[derive(Debug, Serialize)]
pub struct StoreSampleResponse {
    pub success: bool,
    pub barcode: String,
    pub location_name: String,
    pub warnings: Vec<String>,
    pub message: String,
}

#[derive(Debug, Deserialize)]
pub struct CreateLocationRequest {
    pub name: String,
    pub description: Option<String>,
    pub temperature_zone: String,
    pub capacity: i32,
    pub container_type: String,
    pub location_path: Option<String>,
}

/// List all available storage locations with real data
pub async fn list_storage_locations(
    State(components): State<AppComponents>,
) -> Result<Json<Vec<StorageLocationResponse>>, (StatusCode, String)> {
    // Get storage management service from components
    let storage_service = get_storage_service(&components).await?;

    match storage_service.get_capacity_overview().await {
        Ok(overview) => {
            let mut locations = Vec::new();

            for location_stats in overview.location_stats {
                // Get samples in this location
                let samples = match storage_service
                    .get_samples_in_location(location_stats.location_id)
                    .await
                {
                    Ok(samples) => samples
                        .into_iter()
                        .map(|sample| StoredSampleResponse {
                            id: sample.id,
                            sample_id: sample.sample_id,
                            name: format!("Sample {}", sample.sample_id), // Would come from samples table
                            barcode: sample.barcode,
                            position: sample.position,
                            storage_state: format!("{:?}", sample.storage_state),
                            stored_at: sample.stored_at.format("%Y-%m-%d %H:%M:%S UTC").to_string(),
                            stored_by: sample.stored_by,
                        })
                        .collect(),
                    Err(_) => Vec::new(),
                };

                locations.push(StorageLocationResponse {
                    id: location_stats.location_id,
                    name: location_stats.location_name,
                    description: None, // Would need to get from full location data
                    temperature_zone: location_stats.temperature_zone.display_name().to_string(),
                    capacity: location_stats.total_capacity,
                    available: location_stats.available_capacity,
                    utilization_percentage: location_stats.utilization_percentage,
                    is_active: true,
                    samples,
                });
            }

            Ok(Json(locations))
        }
        Err(e) => {
            eprintln!("Error getting storage locations: {}", e);
            Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                "Failed to retrieve storage locations".to_string(),
            ))
        }
    }
}

/// Store a sample in a storage location
pub async fn store_sample(
    State(components): State<AppComponents>,
    Json(request): Json<StoreSampleRequest>,
) -> Result<Json<StoreSampleResponse>, (StatusCode, String)> {
    let storage_service = get_storage_service(&components).await?;

    // Parse temperature requirement if provided
    let requirements = if let Some(temp_req) = request.temperature_requirement {
        let temperature_zone =
            parse_temperature_zone(&temp_req).map_err(|e| (StatusCode::BAD_REQUEST, e))?;

        Some(StorageRequirement {
            temperature_zone,
            container_type: None,
            special_conditions: request.special_conditions.unwrap_or_default(),
            max_storage_duration_days: None,
        })
    } else {
        None
    };

    match storage_service
        .store_sample(
            request.sample_id,
            request.location_id,
            &request.sample_type,
            request.template_name.as_deref(),
            &request.stored_by,
            request.position,
            requirements,
        )
        .await
    {
        Ok(result) => {
            let barcode = result.barcode.clone();
            Ok(Json(StoreSampleResponse {
                success: true,
                barcode: result.barcode,
                location_name: result.location.name,
                warnings: result.warnings,
                message: format!(
                    "Sample {} successfully stored with barcode {}",
                    request.sample_id, barcode
                ),
            }))
        }
        Err(e) => {
            let (status, message) = map_storage_error(&e);
            Err((status, message))
        }
    }
}

/// Move a sample from one storage location to another
pub async fn move_sample(
    State(components): State<AppComponents>,
    Json(request): Json<MoveSampleRequest>,
) -> Result<Json<MoveSampleResponse>, (StatusCode, String)> {
    let storage_service = get_storage_service(&components).await?;

    let reason = request.reason.as_deref().unwrap_or("Sample relocation");

    match storage_service
        .move_sample_by_barcode(
            &request.barcode,
            request.location_id,
            &request.moved_by,
            reason,
            None, // No specific requirements for moving
        )
        .await
    {
        Ok(_moved_sample) => {
            // Get new location name by looking up the location directly
            let storage_repo = Arc::new(PostgresStorageRepository::new(
                components.database.pool.clone(),
            ));
            let new_location_name =
                match storage_repo.get_storage_location(request.location_id).await {
                    Ok(Some(location)) => Some(location.name),
                    _ => None,
                };

            Ok(Json(MoveSampleResponse {
                success: true,
                message: format!(
                    "Sample {} successfully moved to location {}",
                    request.barcode, request.location_id
                ),
                new_location: new_location_name,
            }))
        }
        Err(e) => {
            let (status, message) = map_storage_error(&e);
            Err((status, message))
        }
    }
}

/// Scan a sample barcode to get its information
pub async fn scan_sample_barcode(
    State(components): State<AppComponents>,
    Path(barcode): Path<String>,
) -> Result<Json<ScannedSampleInfo>, (StatusCode, String)> {
    if barcode.is_empty() {
        return Err((
            StatusCode::BAD_REQUEST,
            "Barcode cannot be empty".to_string(),
        ));
    }

    let storage_service = get_storage_service(&components).await?;

    match storage_service.scan_barcode(&barcode).await {
        Ok(scan_result) => Ok(Json(ScannedSampleInfo {
            sample_id: scan_result.sample_location.sample_id,
            barcode: scan_result.sample_location.barcode,
            location_name: scan_result.location.name,
            location_id: scan_result.location.id,
            temperature_zone: scan_result
                .location
                .temperature_zone
                .display_name()
                .to_string(),
            position: scan_result.sample_location.position,
            storage_state: format!("{:?}", scan_result.sample_location.storage_state),
            stored_at: scan_result
                .sample_location
                .stored_at
                .format("%Y-%m-%d %H:%M:%S UTC")
                .to_string(),
            stored_by: scan_result.sample_location.stored_by,
            notes: scan_result.sample_location.notes,
        })),
        Err(StorageManagementError::BarcodeNotFound(_)) => {
            Err((StatusCode::NOT_FOUND, "Sample not found".to_string()))
        }
        Err(e) => {
            let (status, message) = map_storage_error(&e);
            Err((status, message))
        }
    }
}

/// Create a new storage location
pub async fn create_storage_location(
    State(components): State<AppComponents>,
    Json(request): Json<CreateLocationRequest>,
) -> Result<Json<StorageLocationResponse>, (StatusCode, String)> {
    let storage_service = get_storage_service(&components).await?;

    let temperature_zone = parse_temperature_zone(&request.temperature_zone)
        .map_err(|e| (StatusCode::BAD_REQUEST, e))?;

    let container_type =
        parse_container_type(&request.container_type).map_err(|e| (StatusCode::BAD_REQUEST, e))?;

    let location_data = CreateStorageLocation {
        name: request.name,
        description: request.description,
        temperature_zone,
        capacity: request.capacity,
        container_type,
        location_path: request.location_path,
    };

    match storage_service.create_storage_location(location_data).await {
        Ok(location) => {
            let description = location.description.clone();
            let name = location.name.clone();
            Ok(Json(StorageLocationResponse {
                id: location.id,
                name,
                description,
                temperature_zone: location.temperature_zone.display_name().to_string(),
                capacity: location.capacity,
                available: location.available_capacity(),
                utilization_percentage: location.utilization_percentage(),
                is_active: location.is_active,
                samples: Vec::new(), // New location has no samples
            }))
        }
        Err(e) => {
            let (status, message) = map_storage_error(&e);
            Err((status, message))
        }
    }
}

/// Get storage capacity overview
pub async fn get_capacity_overview(
    State(components): State<AppComponents>,
) -> Result<Json<CapacityOverview>, (StatusCode, String)> {
    let storage_service = get_storage_service(&components).await?;

    match storage_service.get_capacity_overview().await {
        Ok(overview) => Ok(Json(overview)),
        Err(e) => {
            let (status, message) = map_storage_error(&e);
            Err((status, message))
        }
    }
}

/// Remove a sample from storage
pub async fn remove_sample(
    State(components): State<AppComponents>,
    Json(request): Json<RemoveSampleRequest>,
) -> Result<Json<RemoveSampleResponse>, (StatusCode, String)> {
    let storage_service = get_storage_service(&components).await?;

    let reason = request.reason.as_deref().unwrap_or("Sample removal");

    match storage_service
        .remove_sample(&request.barcode, &request.removed_by, reason)
        .await
    {
        Ok(result) => Ok(Json(RemoveSampleResponse {
            success: true,
            message: format!(
                "Sample {} successfully removed from storage",
                request.barcode
            ),
            removed_sample: RemovedSampleInfo {
                sample_id: result.sample_location.sample_id,
                barcode: result.sample_location.barcode,
                location_name: result.location.name,
                removed_at: chrono::Utc::now()
                    .format("%Y-%m-%d %H:%M:%S UTC")
                    .to_string(),
                removed_by: request.removed_by,
            },
        })),
        Err(e) => {
            let (status, message) = map_storage_error(&e);
            Err((status, message))
        }
    }
}

// Helper functions
async fn get_storage_service(
    components: &AppComponents,
) -> Result<StorageManagementService<PostgresStorageRepository>, (StatusCode, String)> {
    // This would normally be injected as a component
    // For now, create it using the database pool from components
    use crate::services::barcode_service::BarcodeService;
    use std::sync::Arc;
    use tokio::sync::RwLock;

    // Get database pool from components
    let storage_repo = Arc::new(PostgresStorageRepository::new(
        components.database.pool.clone(),
    ));
    let barcode_service = Arc::new(RwLock::new(BarcodeService::with_default_config()));

    Ok(StorageManagementService::new(storage_repo, barcode_service))
}

fn parse_temperature_zone(zone_str: &str) -> Result<TemperatureZone, String> {
    match zone_str {
        "-80C" | "ultra_low_freezer" => Ok(TemperatureZone::UltraLowFreezer),
        "-20C" | "freezer" => Ok(TemperatureZone::Freezer),
        "4C" | "refrigerator" => Ok(TemperatureZone::Refrigerator),
        "RT" | "room_temperature" => Ok(TemperatureZone::RoomTemperature),
        "37C" | "incubator" => Ok(TemperatureZone::Incubator),
        _ => Err(format!("Invalid temperature zone: {}", zone_str)),
    }
}

fn parse_container_type(
    container_str: &str,
) -> Result<crate::models::storage::ContainerType, String> {
    use crate::models::storage::ContainerType;

    match container_str.to_lowercase().as_str() {
        "tube" => Ok(ContainerType::Tube),
        "plate" => Ok(ContainerType::Plate),
        "box" => Ok(ContainerType::Box),
        "rack" => Ok(ContainerType::Rack),
        "bag" => Ok(ContainerType::Bag),
        _ => Err(format!("Invalid container type: {}", container_str)),
    }
}

fn map_storage_error(error: &StorageManagementError) -> (StatusCode, String) {
    match error {
        StorageManagementError::LocationNotFound(id) => (
            StatusCode::NOT_FOUND,
            format!("Storage location {} not found", id),
        ),
        StorageManagementError::SampleNotFound(id) => {
            (StatusCode::NOT_FOUND, format!("Sample {} not found", id))
        }
        StorageManagementError::BarcodeNotFound(barcode) => (
            StatusCode::NOT_FOUND,
            format!("Barcode {} not found", barcode),
        ),
        StorageManagementError::InsufficientCapacity {
            location_id,
            requested,
            available,
        } => (
            StatusCode::BAD_REQUEST,
            format!(
                "Insufficient capacity in location {}: requested {}, available {}",
                location_id, requested, available
            ),
        ),
        StorageManagementError::IncompatibleTemperature {
            sample_temp_requirement,
            location_temp,
        } => (
            StatusCode::BAD_REQUEST,
            format!(
                "Temperature incompatible: sample requires {:?}, location has {:?}",
                sample_temp_requirement, location_temp
            ),
        ),
        StorageManagementError::LocationInactive(id) => (
            StatusCode::BAD_REQUEST,
            format!("Storage location {} is inactive", id),
        ),
        StorageManagementError::InvalidStateTransition {
            current_state,
            requested_state,
        } => (
            StatusCode::BAD_REQUEST,
            format!(
                "Invalid state transition from {:?} to {:?}",
                current_state, requested_state
            ),
        ),
        StorageManagementError::BarcodeGenerationError(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("Barcode generation failed: {}", e),
        ),
        StorageManagementError::DatabaseError(e) => {
            eprintln!("Database error: {}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                "Internal server error".to_string(),
            )
        }
    }
}
