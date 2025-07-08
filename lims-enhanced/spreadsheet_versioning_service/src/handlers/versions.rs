use axum::{
    extract::{Path, State},
    http::StatusCode,
    Json,
};
use uuid::Uuid;
use tracing::{info, warn, error, instrument};

use crate::{
    error::ServiceResult,
    models::*,
    AppState,
};

#[instrument(skip(state), fields(spreadsheet_id = %request.spreadsheet_id))]
pub async fn create_version(
    State(state): State<AppState>,
    Json(request): Json<CreateVersionRequest>,
) -> ServiceResult<Json<SpreadsheetVersion>> {
    info!("Creating new version for spreadsheet: {}", request.spreadsheet_id);
    
    match state.versioning_service.create_version(request).await {
        Ok(version) => {
            info!("Successfully created version: {}", version.id);
            Ok(Json(version))
        }
        Err(e) => {
            error!("Failed to create version: {}", e);
            Err(e)
        }
    }
}

#[instrument(skip(state), fields(version_id = %version_id))]
pub async fn get_version(
    State(state): State<AppState>,
    Path(version_id): Path<Uuid>,
) -> ServiceResult<Json<SpreadsheetVersion>> {
    info!("Retrieving version: {}", version_id);
    
    match state.versioning_service.get_version(version_id).await {
        Ok(version) => {
            info!("Successfully retrieved version: {}", version_id);
            Ok(Json(version))
        }
        Err(e) => {
            warn!("Version not found or error retrieving: {} - {}", version_id, e);
            Err(e)
        }
    }
}

#[instrument(skip(state), fields(version_id = %version_id))]
pub async fn update_version(
    State(state): State<AppState>,
    Path(version_id): Path<Uuid>,
    Json(request): Json<UpdateVersionRequest>,
) -> ServiceResult<Json<SpreadsheetVersion>> {
    info!("Updating version: {}", version_id);
    
    match state.versioning_service.update_version(version_id, request).await {
        Ok(version) => {
            info!("Successfully updated version: {}", version_id);
            Ok(Json(version))
        }
        Err(e) => {
            error!("Failed to update version: {} - {}", version_id, e);
            Err(e)
        }
    }
}

#[instrument(skip(state), fields(version_id = %version_id))]
pub async fn delete_version(
    State(state): State<AppState>,
    Path(version_id): Path<Uuid>,
) -> ServiceResult<StatusCode> {
    info!("Deleting version: {}", version_id);
    
    match state.versioning_service.delete_version(version_id).await {
        Ok(_) => {
            info!("Successfully deleted version: {}", version_id);
            Ok(StatusCode::NO_CONTENT)
        }
        Err(e) => {
            error!("Failed to delete version: {} - {}", version_id, e);
            Err(e)
        }
    }
} 
