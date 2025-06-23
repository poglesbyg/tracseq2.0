use axum::{
    extract::{Path, Query, State},
    Json,
};
use serde::Deserialize;
use uuid::Uuid;

use crate::{
    error::ServiceResult,
    models::*,
    AppState,
};

#[derive(Deserialize)]
pub struct ListVersionsQuery {
    pub limit: Option<i64>,
    pub offset: Option<i64>,
}

pub async fn list_versions(
    State(state): State<AppState>,
    Path(spreadsheet_id): Path<Uuid>,
    Query(query): Query<ListVersionsQuery>,
) -> ServiceResult<Json<VersionListResponse>> {
    let response = state
        .versioning_service
        .list_versions(spreadsheet_id, query.limit, query.offset)
        .await?;
    Ok(Json(response))
}

pub async fn create_version(
    State(state): State<AppState>,
    Path(spreadsheet_id): Path<Uuid>,
    Json(mut request): Json<CreateVersionRequest>,
) -> ServiceResult<Json<SpreadsheetVersion>> {
    request.spreadsheet_id = spreadsheet_id;
    let version = state.versioning_service.create_version(request).await?;
    Ok(Json(version))
}

pub async fn get_version(
    State(state): State<AppState>,
    Path((spreadsheet_id, version_id)): Path<(Uuid, Uuid)>,
) -> ServiceResult<Json<SpreadsheetVersion>> {
    // TODO: Verify that the version belongs to the spreadsheet
    let version = state.versioning_service.get_version(version_id).await?;
    Ok(Json(version))
} 
