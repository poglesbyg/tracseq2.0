use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    Json,
};
use serde::Deserialize;
use tracing::{info, error};
use uuid::Uuid;
use validator::Validate;

use crate::{
    error::{StorageError, StorageResult},
    models::*,
    AppState,
};

// ============================================================================
// Query Parameters
// ============================================================================

#[derive(Debug, Deserialize)]
pub struct ContainerQuery {
    pub page: Option<i32>,
    pub per_page: Option<i32>,
    pub container_type: Option<String>,
    pub temperature_zone: Option<String>,
    pub parent_id: Option<Uuid>,
    pub status: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct HierarchyQuery {
    pub include_samples: Option<bool>,
    pub max_depth: Option<i32>,
}

#[derive(Debug, Deserialize)]
pub struct GridQuery {
    pub include_empty: Option<bool>,
    pub show_reserved: Option<bool>,
}

// ============================================================================
// Storage Container Management
// ============================================================================

/// Create a new storage container
/// POST /storage/containers
pub async fn create_container(
    State(state): State<AppState>,
    Json(request): Json<CreateStorageContainerRequest>,
) -> Result<(StatusCode, Json<ApiResponse<StorageContainer>>), StorageError> {
    info!("Creating storage container: {} (type: {})", request.name, request.container_type);

    // Validate request
    request.validate().map_err(|e| {
        StorageError::Validation(format!("Validation failed: {}", e))
    })?;

    // Validate container type
    if !["freezer", "rack", "box", "position"].contains(&request.container_type.as_str()) {
        return Err(StorageError::Validation("Invalid container type".to_string()));
    }

    // Validate parent-child relationship
    if let Some(parent_id) = request.parent_container_id {
        let parent = get_container_by_id(&state, parent_id).await?;
        
        // Validate hierarchy rules
        match (parent.container_type.as_str(), request.container_type.as_str()) {
            ("freezer", "rack") | ("rack", "box") | ("box", "position") => {
                // Valid hierarchy
            }
            _ => {
                return Err(StorageError::Validation(
                    format!("Invalid hierarchy: {} cannot contain {}", parent.container_type, request.container_type)
                ));
            }
        }
    }

    let container_id = Uuid::new_v4();
    let metadata = request.container_metadata.unwrap_or_else(|| serde_json::json!({}));
    let access_restrictions = request.access_restrictions.unwrap_or_else(|| serde_json::json!({}));

    let container = sqlx::query_as::<_, StorageContainer>(
        r#"
        INSERT INTO storage_containers (
            id, name, container_type, parent_container_id, location_id,
            grid_position, dimensions, capacity, temperature_zone, barcode,
            description, container_metadata, access_restrictions
        )
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13)
        RETURNING *
        "#,
    )
    .bind(container_id)
    .bind(&request.name)
    .bind(&request.container_type)
    .bind(request.parent_container_id)
    .bind(request.location_id)
    .bind(request.grid_position.as_ref())
    .bind(request.dimensions.as_ref())
    .bind(request.capacity)
    .bind(request.temperature_zone.as_deref())
    .bind(request.barcode.as_deref())
    .bind(request.description.as_deref())
    .bind(&metadata)
    .bind(&access_restrictions)
    .fetch_one(&state.storage_service.db.pool)
    .await
    .map_err(|e| {
        error!("Failed to create storage container: {}", e);
        StorageError::Database(e)
    })?;

    info!("Storage container created with ID: {}", container.id);
    Ok((StatusCode::CREATED, Json(ApiResponse::success(container))))
}

/// Get storage containers with optional filtering
/// GET /storage/containers
pub async fn list_containers(
    State(state): State<AppState>,
    Query(query): Query<ContainerQuery>,
) -> StorageResult<Json<ApiResponse<PaginatedResponse<StorageContainer>>>> {
    info!("Listing storage containers");

    let page = query.page.unwrap_or(1);
    let per_page = query.per_page.unwrap_or(20);
    let offset = (page - 1) * per_page;

    let mut query_builder = sqlx::QueryBuilder::new(
        "SELECT * FROM storage_containers WHERE 1=1"
    );

    // Add filters
    if let Some(container_type) = &query.container_type {
        query_builder.push(" AND container_type = ");
        query_builder.push_bind(container_type);
    }

    if let Some(temperature_zone) = &query.temperature_zone {
        query_builder.push(" AND temperature_zone = ");
        query_builder.push_bind(temperature_zone);
    }

    if let Some(parent_id) = query.parent_id {
        query_builder.push(" AND parent_container_id = ");
        query_builder.push_bind(parent_id);
    }

    if let Some(status) = &query.status {
        query_builder.push(" AND status = ");
        query_builder.push_bind(status);
    }

    query_builder.push(" ORDER BY created_at DESC LIMIT ");
    query_builder.push_bind(per_page);
    query_builder.push(" OFFSET ");
    query_builder.push_bind(offset);

    let containers = query_builder
        .build_query_as::<StorageContainer>()
        .fetch_all(&state.storage_service.db.pool)
        .await?;

    // Get total count for pagination
    let total_count: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM storage_containers"
    )
    .fetch_one(&state.storage_service.db.pool)
    .await?;

    let total_pages = (total_count as i32 + per_page - 1) / per_page;

    let response = PaginatedResponse {
        data: containers,
        pagination: PaginationInfo {
            page,
            per_page,
            total_pages,
            total_items: total_count,
            has_next: page < total_pages,
            has_prev: page > 1,
        },
    };

    Ok(Json(ApiResponse::success(response)))
}

/// Get a specific storage container with its children and samples
/// GET /storage/containers/:container_id
pub async fn get_container_with_details(
    State(state): State<AppState>,
    Path(container_id): Path<Uuid>,
    Query(query): Query<HierarchyQuery>,
) -> StorageResult<Json<ApiResponse<ContainerWithChildren>>> {
    info!("Getting container details: {}", container_id);

    let container = get_container_by_id(&state, container_id).await?;

    // Get child containers
    let children = sqlx::query_as::<_, StorageContainer>(
        "SELECT * FROM storage_containers WHERE parent_container_id = $1 ORDER BY name"
    )
    .bind(container_id)
    .fetch_all(&state.storage_service.db.pool)
    .await?;

    // Get samples if requested
    let samples = if query.include_samples.unwrap_or(true) {
        sqlx::query_as::<_, SamplePosition>(
            "SELECT * FROM sample_positions WHERE container_id = $1 AND removed_at IS NULL"
        )
        .bind(container_id)
        .fetch_all(&state.storage_service.db.pool)
        .await?
    } else {
        Vec::new()
    };

    // Calculate capacity info
    let capacity_info = ContainerCapacityInfo {
        total_capacity: container.capacity,
        occupied_count: container.occupied_count,
        available_count: container.capacity - container.occupied_count,
        utilization_percentage: if container.capacity > 0 {
            (container.occupied_count as f64 / container.capacity as f64) * 100.0
        } else {
            0.0
        },
        capacity_status: if container.capacity > 0 {
            let utilization = container.occupied_count as f64 / container.capacity as f64;
            if utilization >= 0.95 {
                "critical".to_string()
            } else if utilization >= 0.80 {
                "warning".to_string()
            } else {
                "normal".to_string()
            }
        } else {
            "normal".to_string()
        },
        child_containers_count: children.len() as i32,
        direct_samples_count: samples.len() as i32,
    };

    let response = ContainerWithChildren {
        container,
        children,
        samples,
        capacity_info,
    };

    Ok(Json(ApiResponse::success(response)))
}

/// Get storage hierarchy from a specific container
/// GET /storage/containers/:container_id/hierarchy
pub async fn get_storage_hierarchy(
    State(state): State<AppState>,
    Path(container_id): Path<Uuid>,
    Query(query): Query<HierarchyQuery>,
) -> StorageResult<Json<ApiResponse<Vec<StorageHierarchy>>>> {
    info!("Getting storage hierarchy for container: {}", container_id);

    let max_depth = query.max_depth.unwrap_or(10);

    let hierarchy = sqlx::query_as::<_, StorageHierarchy>(
        r#"
        WITH RECURSIVE hierarchy AS (
            SELECT 
                id, name, container_type, parent_container_id, location_id,
                grid_position, capacity, occupied_count, temperature_zone,
                barcode, status, 1 as level,
                ARRAY[name] as path, name as full_path
            FROM storage_containers 
            WHERE id = $1
            
            UNION ALL
            
            SELECT 
                sc.id, sc.name, sc.container_type, sc.parent_container_id, sc.location_id,
                sc.grid_position, sc.capacity, sc.occupied_count, sc.temperature_zone,
                sc.barcode, sc.status, h.level + 1,
                h.path || sc.name, h.full_path || ' > ' || sc.name
            FROM storage_containers sc
            JOIN hierarchy h ON sc.parent_container_id = h.id
            WHERE h.level < $2
        )
        SELECT * FROM hierarchy ORDER BY level, name
        "#
    )
    .bind(container_id)
    .bind(max_depth)
    .fetch_all(&state.storage_service.db.pool)
    .await?;

    Ok(Json(ApiResponse::success(hierarchy)))
}

/// Get grid view for a container (typically for boxes showing positions)
/// GET /storage/containers/:container_id/grid
pub async fn get_container_grid(
    State(state): State<AppState>,
    Path(container_id): Path<Uuid>,
    Query(query): Query<GridQuery>,
) -> StorageResult<Json<ApiResponse<StorageGridView>>> {
    info!("Getting grid view for container: {}", container_id);

    let container = get_container_by_id(&state, container_id).await?;

    // Get child containers (positions) with their grid positions
    let positions_query = if query.include_empty.unwrap_or(true) {
        sqlx::query_as::<_, (Uuid, String, serde_json::Value, Option<Uuid>, Option<String>, Option<String>, String, Option<String>)>(
            r#"
            SELECT 
                sc.id as container_id,
                sc.name as position_identifier,
                sc.grid_position,
                sp.sample_id,
                NULL as sample_barcode,
                NULL as sample_type,
                COALESCE(sp.status, 'available') as status,
                sc.temperature_zone
            FROM storage_containers sc
            LEFT JOIN sample_positions sp ON sc.id = sp.container_id AND sp.removed_at IS NULL
            WHERE sc.parent_container_id = $1 AND sc.container_type = 'position'
            ORDER BY sc.grid_position
            "#
        )
    } else {
        sqlx::query_as::<_, (Uuid, String, serde_json::Value, Option<Uuid>, Option<String>, Option<String>, String, Option<String>)>(
            r#"
            SELECT 
                sc.id as container_id,
                sc.name as position_identifier,
                sc.grid_position,
                sp.sample_id,
                NULL as sample_barcode,
                NULL as sample_type,
                sp.status,
                sc.temperature_zone
            FROM storage_containers sc
            JOIN sample_positions sp ON sc.id = sp.container_id AND sp.removed_at IS NULL
            WHERE sc.parent_container_id = $1 AND sc.container_type = 'position'
            ORDER BY sc.grid_position
            "#
        )
    };

    let position_data = positions_query
        .bind(container_id)
        .fetch_all(&state.storage_service.db.pool)
        .await?;

    let mut positions = Vec::new();
    let mut max_row = 0;
    let mut max_col = 0;

    for (container_id, position_identifier, grid_position, sample_id, sample_barcode, sample_type, status, temperature_zone) in position_data {
        if let Some(pos) = grid_position.as_object() {
            let row = pos.get("row").and_then(|v| v.as_i64()).unwrap_or(1) as i32;
            let column = pos.get("column").and_then(|v| v.as_i64()).unwrap_or(1) as i32;
            
            max_row = max_row.max(row);
            max_col = max_col.max(column);

            positions.push(GridPosition {
                container_id,
                position_identifier,
                row,
                column,
                is_occupied: sample_id.is_some(),
                sample_id,
                sample_barcode,
                sample_type,
                status,
                temperature_zone,
            });
        }
    }

    let grid_view = StorageGridView {
        container_id: container.id,
        container_name: container.name,
        container_type: container.container_type,
        grid_dimensions: GridDimensions {
            rows: max_row,
            columns: max_col,
            total_positions: positions.len() as i32,
        },
        positions,
    };

    Ok(Json(ApiResponse::success(grid_view)))
}

// ============================================================================
// Sample Position Management
// ============================================================================

/// Assign a sample to a specific position
/// POST /storage/samples/assign
pub async fn assign_sample_to_position(
    State(state): State<AppState>,
    Json(request): Json<AssignSampleToPositionRequest>,
) -> Result<(StatusCode, Json<ApiResponse<SamplePosition>>), StorageError> {
    info!("Assigning sample {} to position {}", request.sample_id, request.container_id);

    // Validate request
    request.validate().map_err(|e| {
        StorageError::Validation(format!("Validation failed: {}", e))
    })?;

    // Verify container is a position type
    let container = get_container_by_id(&state, request.container_id).await?;
    if container.container_type != "position" {
        return Err(StorageError::Validation(
            "Can only assign samples to position-type containers".to_string()
        ));
    }

    // Check if position is available
    let existing_assignment: Option<(Uuid,)> = sqlx::query_as(
        "SELECT id FROM sample_positions WHERE container_id = $1 AND removed_at IS NULL"
    )
    .bind(request.container_id)
    .fetch_optional(&state.storage_service.db.pool)
    .await?;

    if existing_assignment.is_some() {
        return Err(StorageError::Validation(
            "Position is already occupied".to_string()
        ));
    }

    // Check if sample is already assigned elsewhere
    let existing_sample: Option<(Uuid,)> = sqlx::query_as(
        "SELECT id FROM sample_positions WHERE sample_id = $1 AND removed_at IS NULL"
    )
    .bind(request.sample_id)
    .fetch_optional(&state.storage_service.db.pool)
    .await?;

    if existing_sample.is_some() {
        return Err(StorageError::Validation(
            "Sample is already assigned to another position".to_string()
        ));
    }

    let position_id = Uuid::new_v4();
    let storage_conditions = request.storage_conditions.unwrap_or_else(|| serde_json::json!({}));
    let special_requirements = request.special_requirements.unwrap_or_else(|| serde_json::json!({}));

    let sample_position = sqlx::query_as::<_, SamplePosition>(
        r#"
        INSERT INTO sample_positions (
            id, sample_id, container_id, position_identifier, assigned_by,
            storage_conditions, special_requirements, notes, chain_of_custody
        )
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)
        RETURNING *
        "#,
    )
    .bind(position_id)
    .bind(request.sample_id)
    .bind(request.container_id)
    .bind(request.position_identifier.as_deref())
    .bind(request.assigned_by)
    .bind(&storage_conditions)
    .bind(&special_requirements)
    .bind(request.notes.as_deref())
    .bind(serde_json::json!([{
        "action": "assigned",
        "timestamp": chrono::Utc::now(),
        "assigned_by": request.assigned_by,
        "container_id": request.container_id
    }]))
    .fetch_one(&state.storage_service.db.pool)
    .await
    .map_err(|e| {
        error!("Failed to assign sample to position: {}", e);
        StorageError::Database(e)
    })?;

    info!("Sample assigned to position successfully");
    Ok((StatusCode::CREATED, Json(ApiResponse::success(sample_position))))
}

/// Move a sample to a new position
/// PUT /storage/samples/:sample_id/move
pub async fn move_sample_to_position(
    State(state): State<AppState>,
    Path(sample_id): Path<Uuid>,
    Json(request): Json<MoveSampleRequest>,
) -> StorageResult<Json<ApiResponse<SamplePosition>>> {
    info!("Moving sample {} to new position {}", sample_id, request.new_container_id);

    // Get current sample position
    let current_position = sqlx::query_as::<_, SamplePosition>(
        "SELECT * FROM sample_positions WHERE sample_id = $1 AND removed_at IS NULL"
    )
    .bind(sample_id)
    .fetch_optional(&state.storage_service.db.pool)
    .await?
    .ok_or_else(|| StorageError::SampleNotFound(sample_id.to_string()))?;

    // Verify new container is a position type
    let new_container = get_container_by_id(&state, request.new_container_id).await?;
    if new_container.container_type != "position" {
        return Err(StorageError::Validation(
            "Can only move samples to position-type containers".to_string()
        ));
    }

    // Check if new position is available
    let existing_assignment: Option<(Uuid,)> = sqlx::query_as(
        "SELECT id FROM sample_positions WHERE container_id = $1 AND removed_at IS NULL"
    )
    .bind(request.new_container_id)
    .fetch_optional(&state.storage_service.db.pool)
    .await?;

    if existing_assignment.is_some() {
        return Err(StorageError::Validation(
            "New position is already occupied".to_string()
        ));
    }

    // Start transaction
    let mut tx = state.storage_service.db.pool.begin().await?;

    // Remove from current position
    sqlx::query(
        "UPDATE sample_positions SET removed_at = NOW(), removed_by = $1 WHERE id = $2"
    )
    .bind(request.moved_by)
    .bind(current_position.id)
    .execute(&mut *tx)
    .await?;

    // Add to new position
    let new_chain_of_custody = {
        let mut custody = current_position.chain_of_custody.as_array().unwrap_or(&vec![]).clone();
        custody.push(serde_json::json!({
            "action": "moved",
            "timestamp": chrono::Utc::now(),
            "moved_by": request.moved_by,
            "from_container_id": current_position.container_id,
            "to_container_id": request.new_container_id,
            "reason": request.reason
        }));
        serde_json::Value::Array(custody)
    };

    let new_position = sqlx::query_as::<_, SamplePosition>(
        r#"
        INSERT INTO sample_positions (
            id, sample_id, container_id, position_identifier, assigned_by,
            storage_conditions, special_requirements, notes, chain_of_custody
        )
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)
        RETURNING *
        "#,
    )
    .bind(Uuid::new_v4())
    .bind(sample_id)
    .bind(request.new_container_id)
    .bind(request.new_position_identifier.as_deref())
    .bind(request.moved_by)
    .bind(&current_position.storage_conditions)
    .bind(&current_position.special_requirements)
    .bind(request.notes.as_deref())
    .bind(&new_chain_of_custody)
    .fetch_one(&mut *tx)
    .await?;

    tx.commit().await?;

    info!("Sample moved to new position successfully");
    Ok(Json(ApiResponse::success(new_position)))
}

/// Get sample location with full hierarchy information
/// GET /storage/samples/:sample_id/location
pub async fn get_sample_location_detailed(
    State(state): State<AppState>,
    Path(sample_id): Path<Uuid>,
) -> StorageResult<Json<ApiResponse<Option<SampleLocationDetailed>>>> {
    info!("Getting detailed location for sample: {}", sample_id);

    let location = sqlx::query_as::<_, SampleLocationDetailed>(
        "SELECT * FROM sample_locations_detailed WHERE sample_id = $1"
    )
    .bind(sample_id)
    .fetch_optional(&state.storage_service.db.pool)
    .await?;

    Ok(Json(ApiResponse::success(location)))
}

/// Remove sample from storage
/// DELETE /storage/samples/:sample_id/position
pub async fn remove_sample_from_position(
    State(state): State<AppState>,
    Path(sample_id): Path<Uuid>,
    Query(query): Query<serde_json::Value>,
) -> StorageResult<Json<ApiResponse<String>>> {
    info!("Removing sample from position: {}", sample_id);

    let removed_by = query.get("removed_by")
        .and_then(|v| v.as_str())
        .and_then(|s| Uuid::parse_str(s).ok());

    let reason = query.get("reason")
        .and_then(|v| v.as_str())
        .unwrap_or("Sample removed from storage");

    let updated = sqlx::query(
        "UPDATE sample_positions SET removed_at = NOW(), removed_by = $1, notes = $2 WHERE sample_id = $3 AND removed_at IS NULL"
    )
    .bind(removed_by)
    .bind(reason)
    .bind(sample_id)
    .execute(&state.storage_service.db.pool)
    .await?;

    if updated.rows_affected() == 0 {
        return Err(StorageError::SampleNotFound(sample_id.to_string()));
    }

    Ok(Json(ApiResponse::success("Sample removed from storage successfully".to_string())))
}

// ============================================================================
// Helper Functions
// ============================================================================

async fn get_container_by_id(state: &AppState, container_id: Uuid) -> StorageResult<StorageContainer> {
    sqlx::query_as::<_, StorageContainer>(
        "SELECT * FROM storage_containers WHERE id = $1"
    )
    .bind(container_id)
    .fetch_optional(&state.storage_service.db.pool)
    .await?
    .ok_or_else(|| StorageError::LocationNotFound(container_id.to_string()))
} 