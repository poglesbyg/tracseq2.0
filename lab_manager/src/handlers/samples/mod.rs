use axum::{
    extract::{Path, State},
    http::StatusCode,
    Json,
};
use uuid::Uuid;

use crate::{
    repositories::storage_repository::StorageRepository,
    sample_submission::{CreateSample, Sample, UpdateSample},
    AppComponents,
};

// Re-export RAG enhanced functionality
pub mod rag_enhanced_handlers;
pub use rag_enhanced_handlers::*;

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

#[derive(Debug, serde::Deserialize)]
pub struct BatchCreateSamplesRequest {
    pub samples: Vec<CreateSample>,
    pub storage_location_id: Option<i32>,
    pub template_name: Option<String>,
    pub stored_by: Option<String>,
}

#[derive(Debug, serde::Serialize)]
pub struct BatchCreateSamplesResponse {
    pub created: usize,
    pub failed: usize,
    pub stored_in_storage: usize,
    pub samples: Vec<Sample>,
    pub storage_errors: Vec<String>,
    pub errors: Vec<BatchError>,
}

#[derive(Debug, serde::Serialize)]
pub struct BatchError {
    pub index: usize,
    pub error: String,
}

/// Create multiple samples in a batch from template data
pub async fn create_samples_batch(
    State(state): State<AppComponents>,
    Json(batch_request): Json<BatchCreateSamplesRequest>,
) -> Result<Json<BatchCreateSamplesResponse>, (StatusCode, String)> {
    tracing::info!("Creating batch of {} samples", batch_request.samples.len());

    let mut created_samples = Vec::new();
    let mut errors = Vec::new();
    let mut storage_errors = Vec::new();
    let mut stored_in_storage_count = 0;

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

        // Create the sample first
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

                let sample_id = sample.id;

                created_samples.push(sample);

                // If storage location is provided, store the sample in the storage system
                if let Some(location_id) = batch_request.storage_location_id {
                    match store_sample_in_storage(
                        &state,
                        sample_id,
                        location_id,
                        &sample_data.name,
                        batch_request.template_name.as_deref(),
                        batch_request.stored_by.as_deref().unwrap_or("system"),
                        None, // position
                    )
                    .await
                    {
                        Ok(_) => {
                            // Update the sample status to indicate it's now in storage
                            let update_sample = crate::sample_submission::UpdateSample {
                                name: None,
                                barcode: None,
                                location: None,
                                status: Some(crate::sample_submission::SampleStatus::InStorage),
                                metadata: None,
                            };

                            if let Err(update_error) = state
                                .sample_processing
                                .manager
                                .update_sample(sample_id, update_sample)
                                .await
                            {
                                tracing::warn!(
                                    "Failed to update sample {} status to InStorage: {}",
                                    sample_id,
                                    update_error
                                );
                            }

                            stored_in_storage_count += 1;
                            tracing::debug!(
                                "Sample {} stored in storage location {}",
                                sample_id,
                                location_id
                            );
                        }
                        Err(storage_error) => {
                            storage_errors.push(format!(
                                "Sample {} ({}): {}",
                                sample_data.name, sample_id, storage_error
                            ));
                            tracing::warn!(
                                "Failed to store sample {} in storage: {}",
                                sample_id,
                                storage_error
                            );
                        }
                    }
                }
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
        stored_in_storage: stored_in_storage_count,
        samples: created_samples,
        storage_errors,
        errors,
    };

    tracing::info!(
        "Batch creation completed: {} created, {} failed, {} stored in storage",
        response.created,
        response.failed,
        response.stored_in_storage
    );
    Ok(Json(response))
}

/// Helper function to store a sample in the storage system
async fn store_sample_in_storage(
    state: &AppComponents,
    sample_id: uuid::Uuid,
    location_id: i32,
    sample_type: &str,
    template_name: Option<&str>,
    stored_by: &str,
    position: Option<String>,
) -> Result<(), String> {
    use crate::repositories::storage_repository::{PostgresStorageRepository, StorageRepository};
    use crate::services::barcode_service::BarcodeService;
    use std::sync::Arc;
    use tokio::sync::RwLock;

    // Get the sample details to extract the barcode
    let sample = state
        .sample_processing
        .manager
        .get_sample(sample_id)
        .await
        .map_err(|e| format!("Failed to get sample details: {}", e))?;

    // Create storage service (this should ideally be injected as a component)
    let storage_repo = Arc::new(PostgresStorageRepository::new(state.database.pool.clone()));
    let barcode_service = Arc::new(RwLock::new(BarcodeService::with_default_config()));

    // Create a sample location entry directly using the repository
    let create_sample_location = crate::repositories::storage_repository::CreateSampleLocation {
        sample_id, // Use UUID directly now that we've fixed the schema
        location_id,
        barcode: sample.barcode.clone(),
        position,
        storage_state: crate::models::storage::StorageState::InStorage,
        stored_by: Some(stored_by.to_string()),
        notes: template_name.map(|t| format!("Created from template: {}", t)),
    };

    // Store the sample location
    storage_repo
        .store_sample(create_sample_location)
        .await
        .map_err(|e| e.to_string())?;

    // Record the movement history
    let movement = crate::repositories::storage_repository::CreateMovementHistory {
        sample_id, // Use UUID directly now that we've fixed the schema
        barcode: sample.barcode.clone(),
        from_location_id: None,
        to_location_id: location_id,
        from_state: Some(crate::models::storage::StorageState::Validated),
        to_state: crate::models::storage::StorageState::InStorage,
        movement_reason: "Initial storage from template".to_string(),
        moved_by: stored_by.to_string(),
        notes: template_name.map(|t| format!("Sample created from template: {}", t)),
    };

    storage_repo
        .record_movement(movement)
        .await
        .map_err(|e| e.to_string())?;

    Ok(())
}
