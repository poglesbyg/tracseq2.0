use axum::{
    extract::{Path, State},
    http::StatusCode,
    Json,
};
use uuid::Uuid;

use crate::{
    sequencing::{CreateJob, JobStatus, SequencingJob},
    AppComponents,
};

/// Create a new sequencing job
pub async fn create_sequencing_job(
    State(state): State<AppComponents>,
    Json(job): Json<CreateJob>,
) -> Result<Json<SequencingJob>, (StatusCode, String)> {
    state
        .sequencing
        .manager
        .create_job(job)
        .await
        .map(Json)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))
}

/// Get a sequencing job by ID
pub async fn get_sequencing_job(
    State(state): State<AppComponents>,
    Path(job_id): Path<Uuid>,
) -> Result<Json<SequencingJob>, (StatusCode, String)> {
    match state.sequencing.manager.get_job(job_id).await {
        Ok(job) => Ok(Json(job)),
        Err(sqlx::Error::RowNotFound) => Err((
            StatusCode::NOT_FOUND,
            format!("Sequencing job {} not found", job_id),
        )),
        Err(e) => Err((StatusCode::INTERNAL_SERVER_ERROR, e.to_string())),
    }
}

/// List all sequencing jobs
pub async fn list_sequencing_jobs(
    State(state): State<AppComponents>,
) -> Result<Json<Vec<SequencingJob>>, (StatusCode, String)> {
    state
        .sequencing
        .manager
        .list_jobs()
        .await
        .map(Json)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))
}

/// Update the status of a sequencing job
pub async fn update_job_status(
    State(state): State<AppComponents>,
    Path(job_id): Path<Uuid>,
    Json(status): Json<JobStatus>,
) -> Result<Json<SequencingJob>, (StatusCode, String)> {
    state
        .sequencing
        .manager
        .update_job_status(job_id, status)
        .await
        .map(Json)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))
}
