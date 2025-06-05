use axum::{
    extract::{Path, State},
    http::StatusCode,
    Json,
};
use uuid::Uuid;

use crate::{
    sample_submission::{CreateSample, Sample, UpdateSample},
    AppComponents,
};

/// Create a new sample from the provided data
pub async fn create_sample(
    State(state): State<AppComponents>,
    Json(sample): Json<CreateSample>,
) -> Result<Json<Sample>, (StatusCode, String)> {
    // Validate required fields
    if sample.name.trim().is_empty() {
        return Err((
            StatusCode::UNPROCESSABLE_ENTITY,
            "Sample name cannot be empty".to_string(),
        ));
    }
    if sample.barcode.trim().is_empty() {
        return Err((
            StatusCode::UNPROCESSABLE_ENTITY,
            "Sample barcode cannot be empty".to_string(),
        ));
    }
    if sample.location.trim().is_empty() {
        return Err((
            StatusCode::UNPROCESSABLE_ENTITY,
            "Sample location cannot be empty".to_string(),
        ));
    }

    state
        .sample_processing
        .manager
        .create_sample(sample)
        .await
        .map(Json)
        .map_err(|e| {
            if e.to_string().contains("unique") || e.to_string().contains("duplicate") {
                (
                    StatusCode::UNPROCESSABLE_ENTITY,
                    format!("Barcode already exists: {}", e),
                )
            } else {
                (StatusCode::INTERNAL_SERVER_ERROR, e.to_string())
            }
        })
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

/// Update a sample by its ID
pub async fn update_sample(
    State(state): State<AppComponents>,
    Path(sample_id): Path<Uuid>,
    Json(updates): Json<UpdateSample>,
) -> Result<Json<Sample>, (StatusCode, String)> {
    state
        .sample_processing
        .manager
        .update_sample(sample_id, updates)
        .await
        .map(Json)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))
}

/// Get a single sample by its ID
pub async fn get_sample(
    State(state): State<AppComponents>,
    Path(sample_id): Path<Uuid>,
) -> Result<Json<Sample>, (StatusCode, String)> {
    state
        .sample_processing
        .manager
        .get_sample(sample_id)
        .await
        .map(Json)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))
}

/// Create multiple samples in a batch from template data
pub async fn create_samples_batch(
    State(state): State<AppComponents>,
    Json(batch_request): Json<BatchCreateSamplesRequest>,
) -> Result<Json<BatchCreateSamplesResponse>, (StatusCode, String)> {
    tracing::info!("Creating batch of {} samples", batch_request.samples.len());

    let mut created_samples = Vec::new();
    let mut errors = Vec::new();

    for (index, sample_data) in batch_request.samples.iter().enumerate() {
        // Validate required fields
        if sample_data.name.trim().is_empty() {
            errors.push(BatchError {
                index,
                error: "Sample name cannot be empty".to_string(),
            });
            continue;
        }
        if sample_data.barcode.trim().is_empty() {
            errors.push(BatchError {
                index,
                error: "Sample barcode cannot be empty".to_string(),
            });
            continue;
        }
        if sample_data.location.trim().is_empty() {
            errors.push(BatchError {
                index,
                error: "Sample location cannot be empty".to_string(),
            });
            continue;
        }

        match state
            .sample_processing
            .manager
            .create_sample((*sample_data).clone())
            .await
        {
            Ok(sample) => {
                tracing::debug!(
                    "Created sample: {} with barcode: {}",
                    sample.name,
                    sample.barcode
                );
                created_samples.push(sample);
            }
            Err(e) => {
                let error_msg =
                    if e.to_string().contains("unique") || e.to_string().contains("duplicate") {
                        format!("Barcode '{}' already exists", sample_data.barcode)
                    } else {
                        e.to_string()
                    };
                tracing::warn!("Failed to create sample at index {}: {}", index, error_msg);
                errors.push(BatchError {
                    index,
                    error: error_msg,
                });
            }
        }
    }

    let response = BatchCreateSamplesResponse {
        created: created_samples.len(),
        failed: errors.len(),
        samples: created_samples,
        errors,
    };

    tracing::info!(
        "Batch creation completed: {} created, {} failed",
        response.created,
        response.failed
    );
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
