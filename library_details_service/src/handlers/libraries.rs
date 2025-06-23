use crate::error::{Result, ServiceError};
use crate::models::*;
use crate::services::LibraryService;
use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    Json,
};
use serde::Deserialize;
use std::sync::Arc;
use uuid::Uuid;
use validator::Validate;



#[derive(Debug, Deserialize)]
pub struct ListLibrariesQuery {
    sample_id: Option<Uuid>,
    status: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct NormalizeLibraryRequest {
    target_concentration: f64,
}

pub async fn list_libraries(
    State(service): State<Arc<LibraryService>>,
    Query(params): Query<ListLibrariesQuery>,
) -> Result<Json<Vec<Library>>> {
    let status = if let Some(status_str) = params.status {
        match status_str.to_lowercase().as_str() {
            "pending" => Some(LibraryStatus::Pending),
            "inpreparation" => Some(LibraryStatus::InPreparation),
            "qualitycontrol" => Some(LibraryStatus::QualityControl),
            "approved" => Some(LibraryStatus::Approved),
            "failed" => Some(LibraryStatus::Failed),
            "sequencing" => Some(LibraryStatus::Sequencing),
            "completed" => Some(LibraryStatus::Completed),
            _ => None,
        }
    } else {
        None
    };

    let libraries = service.list_libraries(params.sample_id, status).await?;
    Ok(Json(libraries))
}

pub async fn create_library(
    State(service): State<Arc<LibraryService>>,
    Json(request): Json<CreateLibraryRequest>,
) -> Result<(StatusCode, Json<Library>)> {
    request.validate().map_err(|e| ServiceError::Validation {
        message: e.to_string(),
    })?;

    let library = service.create_library(request).await?;
    Ok((StatusCode::CREATED, Json(library)))
}

pub async fn get_library(
    State(service): State<Arc<LibraryService>>,
    Path(id): Path<Uuid>,
) -> Result<Json<Library>> {
    let library = service.get_library(id).await?;
    Ok(Json(library))
}

pub async fn update_library(
    State(service): State<Arc<LibraryService>>,
    Path(id): Path<Uuid>,
    Json(request): Json<UpdateLibraryRequest>,
) -> Result<Json<Library>> {
    request.validate().map_err(|e| ServiceError::Validation {
        message: e.to_string(),
    })?;

    let library = service.update_library(id, request).await?;
    Ok(Json(library))
}

pub async fn delete_library(
    State(service): State<Arc<LibraryService>>,
    Path(id): Path<Uuid>,
) -> Result<StatusCode> {
    service.delete_library(id).await?;
    Ok(StatusCode::NO_CONTENT)
}

pub async fn calculate_library_metrics(
    State(service): State<Arc<LibraryService>>,
    Path(id): Path<Uuid>,
) -> Result<Json<LibraryMetrics>> {
    let metrics = service.calculate_library_metrics(id).await?;
    Ok(Json(metrics))
}

pub async fn normalize_library(
    State(service): State<Arc<LibraryService>>,
    Path(id): Path<Uuid>,
    Json(request): Json<NormalizeLibraryRequest>,
) -> Result<Json<Library>> {
    if request.target_concentration <= 0.0 {
        return Err(ServiceError::Validation {
            message: "Target concentration must be positive".to_string(),
        });
    }

    let library = service.normalize_library(id, request.target_concentration).await?;
    Ok(Json(library))
}

pub async fn create_batch_libraries(
    State(service): State<Arc<LibraryService>>,
    Json(request): Json<BatchLibraryRequest>,
) -> Result<(StatusCode, Json<Vec<Library>>)> {
    // Validate each library request
    for lib_request in &request.libraries {
        lib_request.validate().map_err(|e| ServiceError::Validation {
            message: format!("Library validation failed: {}", e),
        })?;
    }

    let libraries = service.create_batch_libraries(request).await?;
    Ok((StatusCode::CREATED, Json(libraries)))
}

pub async fn get_libraries_for_sample(
    State(service): State<Arc<LibraryService>>,
    Path(sample_id): Path<Uuid>,
) -> Result<Json<Vec<Library>>> {
    let libraries = service.get_libraries_for_sample(sample_id).await?;
    Ok(Json(libraries))
}

pub async fn get_libraries_for_sequencing_job(
    State(service): State<Arc<LibraryService>>,
    Path(job_id): Path<Uuid>,
) -> Result<Json<Vec<Library>>> {
    let libraries = service.get_libraries_for_sequencing_job(job_id).await?;
    Ok(Json(libraries))
}