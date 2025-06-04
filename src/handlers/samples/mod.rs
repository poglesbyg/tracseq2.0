use axum::{
    extract::{Path, State},
    http::StatusCode,
    Json,
};
use uuid::Uuid;

use crate::{
    sample_submission::{CreateSample, Sample},
    AppComponents,
};

/// Create a new sample from the provided data
pub async fn create_sample(
    State(state): State<AppComponents>,
    Json(sample): Json<CreateSample>,
) -> Result<Json<Sample>, (StatusCode, String)> {
    state
        .sample_processing
        .manager
        .create_sample(sample)
        .await
        .map(Json)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))
}

/// List all samples in the system
pub async fn list_samples(
    State(state): State<AppComponents>,
) -> Result<Json<Vec<Sample>>, (StatusCode, String)> {
    state
        .sample_processing
        .manager
        .list_samples()
        .await
        .map(Json)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))
}

/// Validate a sample by its ID
pub async fn validate_sample(
    State(state): State<AppComponents>,
    Path(sample_id): Path<Uuid>,
) -> Result<Json<Sample>, (StatusCode, String)> {
    state
        .sample_processing
        .manager
        .validate_sample(sample_id)
        .await
        .map(Json)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))
}
