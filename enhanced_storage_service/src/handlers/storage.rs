use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    Json,
};
use chrono::Utc;
use serde::{Deserialize, Serialize};
use tracing::{info, error};
use uuid::Uuid;
use validator::Validate;

use crate::{
    error::{StorageError, StorageResult},
    models::*,
    AppState,
};

#[derive(Debug, Deserialize)]
pub struct LocationQuery {
    pub page: Option<i32>,
    pub per_page: Option<i32>,
    pub temperature_zone: Option<String>,
    pub location_type: Option<String>,
}

/// Create a new storage location
/// POST /storage/locations
pub async fn create_location(
    State(state): State<AppState>,
    Json(request): Json<CreateStorageLocationRequest>,
) -> Result<(StatusCode, Json<ApiResponse<StorageLocation>>), StorageError> {
    info!("Creating storage location: {}", request.name);

    // Validate request
    request.validate().map_err(|e| {
        StorageError::Validation(format!("Validation failed: {}", e))
    })?;

    let location = state.storage_service.create_storage_location(request).await?;

    Ok((StatusCode::CREATED, Json(ApiResponse::success(location))))
}

/// Get all storage locations with optional filtering
/// GET /storage/locations
pub async fn list_locations(
    State(state): State<AppState>,
    Query(query): Query<LocationQuery>,
) -> StorageResult<Json<ApiResponse<PaginatedResponse<StorageLocation>>>> {
    info!("Listing storage locations");

    let response = state.storage_service.list_storage_locations(
        query.page,
        query.per_page,
    ).await?;

    Ok(Json(ApiResponse::success(response)))
}

/// Get a specific storage location
/// GET /storage/locations/:location_id
pub async fn get_location(
    State(state): State<AppState>,
    Path(location_id): Path<Uuid>,
) -> StorageResult<Json<ApiResponse<StorageLocation>>> {
    info!("Getting storage location: {}", location_id);

    let location = state.storage_service.get_storage_location(location_id).await?;

    Ok(Json(ApiResponse::success(location)))
}

/// Update a storage location
/// PUT /storage/locations/:location_id
pub async fn update_location(
    State(state): State<AppState>,
    Path(location_id): Path<Uuid>,
    Json(request): Json<UpdateStorageLocationRequest>,
) -> StorageResult<Json<ApiResponse<StorageLocation>>> {
    info!("Updating storage location: {}", location_id);

    // Get current location
    let mut location = state.storage_service.get_storage_location(location_id).await?;

    // Update fields if provided
    if let Some(name) = request.name {
        location.name = name;
    }
    if let Some(description) = request.description {
        location.description = Some(description);
    }
    if let Some(location_type) = request.location_type {
        location.location_type = location_type;
    }
    if let Some(temperature_zone) = request.temperature_zone {
        location.temperature_zone = temperature_zone;
    }
    if let Some(max_capacity) = request.max_capacity {
        location.max_capacity = max_capacity;
    }
    if let Some(coordinates) = request.coordinates {
        location.coordinates = Some(coordinates);
    }
    if let Some(status) = request.status {
        location.status = status;
    }
    if let Some(metadata) = request.metadata {
        location.metadata = metadata;
    }

    // Update in database
    let updated_location = sqlx::query_as::<_, StorageLocation>(
        r#"
        UPDATE storage_locations 
        SET name = $1, description = $2, location_type = $3, temperature_zone = $4,
            max_capacity = $5, coordinates = $6, status = $7, metadata = $8, updated_at = NOW()
        WHERE id = $9
        RETURNING *
        "#,
    )
    .bind(&location.name)
    .bind(location.description.as_deref())
    .bind(&location.location_type)
    .bind(&location.temperature_zone)
    .bind(location.max_capacity)
    .bind(location.coordinates.as_ref())
    .bind(&location.status)
    .bind(&location.metadata)
    .bind(location_id)
    .fetch_one(&state.storage_service.db.pool)
    .await?;

    Ok(Json(ApiResponse::success(updated_location)))
}

/// Delete a storage location
/// DELETE /storage/locations/:location_id
pub async fn delete_location(
    State(state): State<AppState>,
    Path(location_id): Path<Uuid>,
) -> StorageResult<Json<ApiResponse<String>>> {
    info!("Deleting storage location: {}", location_id);

    // Check if location has samples
    let sample_count: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM samples WHERE storage_location_id = $1"
    )
    .bind(location_id)
    .fetch_one(&state.storage_service.db.pool)
    .await?;

    if sample_count > 0 {
        return Err(StorageError::Validation(
            "Cannot delete location with samples".to_string()
        ));
    }

    // Delete location
    let deleted = sqlx::query(
        "DELETE FROM storage_locations WHERE id = $1"
    )
    .bind(location_id)
    .execute(&state.storage_service.db.pool)
    .await?;

    if deleted.rows_affected() == 0 {
        return Err(StorageError::LocationNotFound(location_id.to_string()));
    }

    Ok(Json(ApiResponse::success("Location deleted successfully".to_string())))
}

/// Get capacity information for a location
/// GET /storage/locations/:location_id/capacity
pub async fn get_capacity(
    State(state): State<AppState>,
    Path(location_id): Path<Uuid>,
) -> StorageResult<Json<ApiResponse<CapacityInfo>>> {
    info!("Getting capacity for location: {}", location_id);

    let utilization = state.storage_service.get_location_capacity(location_id).await?;
    
    let location = state.storage_service.get_storage_location(location_id).await?;

    let capacity_info = CapacityInfo {
        location_id,
        max_capacity: location.max_capacity,
        current_capacity: location.current_capacity,
        utilization_percentage: utilization * 100.0,
        available_capacity: location.max_capacity - location.current_capacity,
        status: if utilization > 0.95 {
            "critical".to_string()
        } else if utilization > 0.8 {
            "warning".to_string()
        } else {
            "normal".to_string()
        },
    };

    Ok(Json(ApiResponse::success(capacity_info)))
}

/// Store a sample in a location
/// POST /storage/samples
pub async fn store_sample(
    State(state): State<AppState>,
    Json(request): Json<StoreSampleRequest>,
) -> Result<(StatusCode, Json<ApiResponse<Sample>>), StorageError> {
    info!("Storing sample with barcode: {}", request.barcode);

    // Validate request
    request.validate().map_err(|e| {
        StorageError::Validation(format!("Validation failed: {}", e))
    })?;

    let sample = state.storage_service.store_sample(request).await?;

    Ok((StatusCode::CREATED, Json(ApiResponse::success(sample))))
}

/// Get the location of a sample
/// GET /storage/samples/:sample_id/location
pub async fn get_sample_location(
    State(state): State<AppState>,
    Path(sample_id): Path<Uuid>,
) -> StorageResult<Json<ApiResponse<Option<StorageLocation>>>> {
    info!("Getting location for sample: {}", sample_id);

    let location = state.storage_service.get_sample_location(sample_id).await?;

    Ok(Json(ApiResponse::success(location)))
}

/// Move a sample to a new location
/// POST /storage/samples/:sample_id/move
pub async fn move_sample(
    State(state): State<AppState>,
    Path(sample_id): Path<Uuid>,
    Json(request): Json<MoveSampleRequest>,
) -> StorageResult<Json<ApiResponse<Sample>>> {
    info!("Moving sample {} to location {}", sample_id, request.new_location_id);

    let sample = state.storage_service.move_sample(sample_id, request).await?;

    Ok(Json(ApiResponse::success(sample)))
}

/// Retrieve a sample from storage
/// POST /storage/samples/:sample_id/retrieve
pub async fn retrieve_sample(
    State(state): State<AppState>,
    Path(sample_id): Path<Uuid>,
) -> StorageResult<Json<ApiResponse<Sample>>> {
    info!("Retrieving sample: {}", sample_id);

    // Get current sample
    let sample = sqlx::query_as::<_, Sample>(
        "SELECT * FROM samples WHERE id = $1"
    )
    .bind(sample_id)
    .fetch_optional(&state.storage_service.db.pool)
    .await?
    .ok_or_else(|| StorageError::SampleNotFound(sample_id.to_string()))?;

    // Update sample status and add to chain of custody
    let updated_sample = sqlx::query_as::<_, Sample>(
        r#"
        UPDATE samples 
        SET status = 'retrieved',
            storage_location_id = NULL,
            position = NULL,
            chain_of_custody = chain_of_custody || $1,
            updated_at = NOW()
        WHERE id = $2
        RETURNING *
        "#,
    )
    .bind(serde_json::json!({
        "action": "retrieved",
        "timestamp": chrono::Utc::now(),
        "from_location_id": sample.storage_location_id
    }))
    .bind(sample_id)
    .fetch_one(&state.storage_service.db.pool)
    .await?;

    // Update location capacity if sample was in a location
    if let Some(location_id) = sample.storage_location_id {
        sqlx::query(
            "UPDATE storage_locations SET current_capacity = current_capacity - 1 WHERE id = $1"
        )
        .bind(location_id)
        .execute(&state.storage_service.db.pool)
        .await?;
    }

    Ok(Json(ApiResponse::success(updated_sample)))
}

/// Get sample by ID
/// GET /storage/samples/:sample_id
pub async fn get_sample(
    State(state): State<AppState>,
    Path(sample_id): Path<Uuid>,
) -> StorageResult<Json<ApiResponse<Sample>>> {
    info!("Getting sample: {}", sample_id);

    let sample = state.storage_service.get_sample(sample_id).await?;

    Ok(Json(ApiResponse::success(sample)))
}

// Helper struct for capacity information
#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct CapacityInfo {
    pub location_id: Uuid,
    pub max_capacity: i32,
    pub current_capacity: i32,
    pub utilization_percentage: f64,
    pub available_capacity: i32,
    pub status: String,
}
