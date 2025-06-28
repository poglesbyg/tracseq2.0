use axum::{
    extract::{Path, State},
    http::StatusCode,
    Json,
};
use uuid::Uuid;

use crate::{
    error::ServiceResult,
    models::*,
    AppState,
};

pub async fn create_version(
    State(state): State<AppState>,
    Json(request): Json<CreateVersionRequest>,
) -> ServiceResult<Json<SpreadsheetVersion>> {
    let version = state.versioning_service.create_version(request).await?;
    Ok(Json(version))
}

pub async fn get_version(
    State(state): State<AppState>,
    Path(version_id): Path<Uuid>,
) -> ServiceResult<Json<SpreadsheetVersion>> {
    let version = state.versioning_service.get_version(version_id).await?;
    Ok(Json(version))
}

pub async fn update_version(
    State(state): State<AppState>,
    Path(version_id): Path<Uuid>,
    Json(request): Json<UpdateVersionRequest>,
) -> ServiceResult<Json<SpreadsheetVersion>> {
    let version = state.versioning_service.update_version(version_id, request).await?;
    Ok(Json(version))
}

pub async fn delete_version(
    State(state): State<AppState>,
    Path(version_id): Path<Uuid>,
) -> ServiceResult<StatusCode> {
    state.versioning_service.delete_version(version_id).await?;
    Ok(StatusCode::NO_CONTENT)
} 
