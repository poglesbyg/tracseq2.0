use axum::{extract::State, Json};

use crate::{
    error::ServiceResult,
    models::*,
    services::DiffEngine,
    AppState,
};

pub async fn compare_versions(
    State(state): State<AppState>,
    Json(request): Json<DiffRequest>,
) -> ServiceResult<Json<DiffResponse>> {
    let diff_engine = DiffEngine::new(state.database.clone());
    let response = diff_engine.generate_diff(request).await?;
    Ok(Json(response))
}

pub async fn merge_versions(
    State(state): State<AppState>,
    Json(request): Json<MergeRequest>,
) -> ServiceResult<Json<MergeResponse>> {
    // TODO: Implement merge functionality
    todo!("Merge functionality to be implemented")
}

pub async fn detect_conflicts(
    State(state): State<AppState>,
    Json(request): Json<ConflictDetectionRequest>,
) -> ServiceResult<Json<Vec<VersionConflict>>> {
    // TODO: Implement conflict detection
    todo!("Conflict detection to be implemented")
} 
