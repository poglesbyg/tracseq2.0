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

/// Create multiple samples in a batch from template data
pub async fn create_samples_batch(
    State(state): State<AppComponents>,
    Json(batch_request): Json<BatchCreateSamplesRequest>,
) -> Result<Json<BatchCreateSamplesResponse>, (StatusCode, String)> {
    let mut created_samples = Vec::new();
    let mut errors = Vec::new();

    for (index, sample_data) in batch_request.samples.iter().enumerate() {
        match state
            .sample_processing
            .manager
            .create_sample((*sample_data).clone())
            .await
        {
            Ok(sample) => created_samples.push(sample),
            Err(e) => errors.push(BatchError {
                index,
                error: e.to_string(),
            }),
        }
    }

    let response = BatchCreateSamplesResponse {
        created: created_samples.len(),
        failed: errors.len(),
        samples: created_samples,
        errors,
    };

    Ok(Json(response))
}

#[derive(Debug, serde::Deserialize)]
pub struct BatchCreateSamplesRequest {
    pub samples: Vec<CreateSample>,
}

#[derive(Debug, serde::Serialize)]
pub struct BatchCreateSamplesResponse {
    pub created: usize,
    pub failed: usize,
    pub samples: Vec<Sample>,
    pub errors: Vec<BatchError>,
}

#[derive(Debug, serde::Serialize)]
pub struct BatchError {
    pub index: usize,
    pub error: String,
}
