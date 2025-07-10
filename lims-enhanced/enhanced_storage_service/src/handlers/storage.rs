use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    Json,
};
use serde::Deserialize;
use tracing::info;
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
pub async fn create_storage_location(
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
pub async fn list_storage_locations(
    State(state): State<AppState>,
    Query(_query): Query<LocationQuery>,
) -> StorageResult<Json<ApiResponse<Vec<StorageLocation>>>> {
    info!("Listing storage locations");

    let locations = state.storage_service.list_storage_locations().await?;

    Ok(Json(ApiResponse::success(locations)))
}

/// Get a specific storage location
/// GET /storage/locations/:location_id
pub async fn get_storage_location(
    State(state): State<AppState>,
    Path(location_id): Path<Uuid>,
) -> StorageResult<Json<ApiResponse<StorageLocation>>> {
    info!("Getting storage location: {}", location_id);

    let location = state.storage_service.get_storage_location(location_id).await?;

    Ok(Json(ApiResponse::success(location)))
}

/// Update a storage location
/// PUT /storage/locations/:location_id
pub async fn update_storage_location(
    State(state): State<AppState>,
    Path(location_id): Path<Uuid>,
    Json(request): Json<UpdateStorageLocationRequest>,
) -> StorageResult<Json<ApiResponse<StorageLocation>>> {
    info!("Updating storage location: {}", location_id);

    let updated_location = state.storage_service.update_storage_location(location_id, request).await?;

    Ok(Json(ApiResponse::success(updated_location)))
}

/// Delete a storage location
/// DELETE /storage/locations/:location_id
pub async fn delete_storage_location(
    State(state): State<AppState>,
    Path(location_id): Path<Uuid>,
) -> StorageResult<Json<ApiResponse<String>>> {
    info!("Deleting storage location: {}", location_id);

    state.storage_service.delete_storage_location(location_id).await?;

    Ok(Json(ApiResponse::success("Location deleted successfully".to_string())))
}



/// Store a sample in a location (placeholder)
/// POST /storage/samples
pub async fn store_sample(
    State(_state): State<AppState>,
    Json(_request): Json<StoreSampleRequest>,
) -> Result<(StatusCode, Json<ApiResponse<String>>), StorageError> {
    Ok((StatusCode::NOT_IMPLEMENTED, Json(ApiResponse::success("Sample storage not yet implemented".to_string()))))
}

/// Get the location of a sample (placeholder)
/// GET /storage/samples/:sample_id/location
pub async fn get_sample_location(
    State(_state): State<AppState>,
    Path(_sample_id): Path<Uuid>,
) -> StorageResult<Json<ApiResponse<String>>> {
    Ok(Json(ApiResponse::success("Sample location lookup not yet implemented".to_string())))
}

/// Retrieve a sample from storage (placeholder)
/// POST /storage/samples/:sample_id/retrieve
pub async fn retrieve_sample(
    State(_state): State<AppState>,
    Path(_sample_id): Path<Uuid>,
) -> StorageResult<Json<ApiResponse<String>>> {
    Ok(Json(ApiResponse::success("Sample retrieval not yet implemented".to_string())))
}

/// Get sample by ID (placeholder)
/// GET /storage/samples/:sample_id
pub async fn get_sample(
    State(_state): State<AppState>,
    Path(_sample_id): Path<Uuid>,
) -> StorageResult<Json<ApiResponse<String>>> {
    Ok(Json(ApiResponse::success("Sample lookup not yet implemented".to_string())))
}

// ============================================================================
// Missing Handler Functions
// ============================================================================

/// Health check endpoint
/// GET /health
pub async fn health_check() -> Json<serde_json::Value> {
    Json(serde_json::json!({
        "status": "healthy",
        "service": "enhanced-storage-service",
        "timestamp": chrono::Utc::now()
    }))
}

/// Readiness check endpoint
/// GET /health/ready
pub async fn readiness_check(
    State(state): State<AppState>,
) -> Result<Json<serde_json::Value>, StorageError> {
    // Test database connectivity
    match state.storage_service.db.test_connection().await {
        Ok(_) => Ok(Json(serde_json::json!({
            "status": "ready",
            "service": "enhanced-storage-service",
            "timestamp": chrono::Utc::now()
        }))),
        Err(_) => Err(StorageError::Internal("Database not ready".to_string()))
    }
}

/// Metrics check endpoint
/// GET /health/metrics
pub async fn metrics_check(
    State(state): State<AppState>,
) -> StorageResult<Json<serde_json::Value>> {
    let health_info = state.storage_service.health_check().await?;
    Ok(Json(health_info))
}

/// Get location capacity information
/// GET /storage/locations/:location_id/capacity
pub async fn get_location_capacity(
    State(state): State<AppState>,
    Path(location_id): Path<Uuid>,
) -> StorageResult<Json<ApiResponse<CapacityInfo>>> {
    info!("Getting capacity for location: {}", location_id);

    let location = state.storage_service.get_storage_location(location_id).await?;

    let capacity_info = CapacityInfo {
        location_id,
        max_capacity: location.max_capacity,
        current_capacity: location.current_capacity,
        utilization_percentage: if location.max_capacity > 0 {
            (location.current_capacity as f64 / location.max_capacity as f64) * 100.0
        } else {
            0.0
        },
        available_capacity: location.max_capacity - location.current_capacity,
        status: if location.max_capacity > 0 {
            let utilization = location.current_capacity as f64 / location.max_capacity as f64;
            if utilization > 0.95 {
                "critical".to_string()
            } else if utilization > 0.8 {
                "warning".to_string()
            } else {
                "normal".to_string()
            }
        } else {
            "normal".to_string()
        },
    };

    Ok(Json(ApiResponse::success(capacity_info)))
}

/// Get capacity summary
/// GET /storage/capacity/summary
pub async fn get_capacity_summary(
    State(state): State<AppState>,
) -> StorageResult<Json<ApiResponse<Vec<StorageCapacitySummary>>>> {
    let summary = state.storage_service.get_capacity_summary().await?;
    Ok(Json(ApiResponse::success(summary)))
}

/// Get utilization report
/// GET /storage/utilization
pub async fn get_utilization_report(
    State(state): State<AppState>,
) -> StorageResult<Json<ApiResponse<serde_json::Value>>> {
    let report = state.storage_service.get_utilization_report().await?;
    Ok(Json(ApiResponse::success(report)))
}

/// Get available positions
/// GET /storage/containers/available
pub async fn get_available_positions(
    State(state): State<AppState>,
    Query(query): Query<serde_json::Value>,
) -> StorageResult<Json<ApiResponse<Vec<StorageContainer>>>> {
    let temperature_zone = query.get("temperature_zone").and_then(|v| v.as_str());
    let container_type = query.get("container_type").and_then(|v| v.as_str()).unwrap_or("position");

    let mut query_builder = sqlx::QueryBuilder::new(
        "SELECT * FROM storage_containers WHERE container_type = "
    );
    query_builder.push_bind(container_type);
    query_builder.push(" AND occupied_count < capacity");

    if let Some(temp_zone) = temperature_zone {
        query_builder.push(" AND temperature_zone = ");
        query_builder.push_bind(temp_zone);
    }

    query_builder.push(" ORDER BY (capacity - occupied_count) DESC");

    let available_positions = query_builder
        .build_query_as::<StorageContainer>()
        .fetch_all(&state.storage_service.db.pool)
        .await?;

    Ok(Json(ApiResponse::success(available_positions)))
}

/// Get containers by temperature zone
/// GET /storage/containers/by-temperature/:temperature_zone
pub async fn get_containers_by_temperature(
    State(state): State<AppState>,
    Path(temperature_zone): Path<String>,
) -> StorageResult<Json<ApiResponse<Vec<StorageContainer>>>> {
    let containers = sqlx::query_as::<_, StorageContainer>(
        "SELECT * FROM storage_containers WHERE temperature_zone = $1 ORDER BY name"
    )
    .bind(&temperature_zone)
    .fetch_all(&state.storage_service.db.pool)
    .await?;

    Ok(Json(ApiResponse::success(containers)))
}

// ============================================================================
// Helper Structs
// ============================================================================

/// Helper struct for capacity information
#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct CapacityInfo {
    pub location_id: Uuid,
    pub max_capacity: i32,
    pub current_capacity: i32,
    pub utilization_percentage: f64,
    pub available_capacity: i32,
    pub status: String,
}
