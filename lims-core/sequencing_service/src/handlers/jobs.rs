use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::Json,
};
use chrono::Utc;
use serde::Deserialize;
use serde_json::{Value, json};
use tracing::{info, warn};
use uuid::Uuid;

use crate::{
    AppState,
    error::{Result, SequencingError},
    models::{CreateJobRequest, JobResponse, JobStatus, Priority, SequencingJob, UpdateJobRequest},
};

#[derive(Debug, Deserialize)]
pub struct ListJobsQuery {
    pub status: Option<String>,
    pub priority: Option<String>,
    pub platform: Option<String>,
    pub created_by: Option<Uuid>,
    pub page: Option<u32>,
    pub limit: Option<u32>,
    pub sort: Option<String>,
    pub order: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateJobStatusRequest {
    pub status: JobStatus,
    pub notes: Option<String>,
}

/// Create a new sequencing job
pub async fn create_job(
    State(state): State<AppState>,
    Json(payload): Json<CreateJobRequest>,
) -> Result<(StatusCode, Json<Value>)> {
    info!("Creating new sequencing job: {}", payload.name);

    // Validate the request
    payload.validate().map_err(SequencingError::Validation)?;

    // Check if platform exists in configuration
    if state.config.get_platform(&payload.platform_id).is_none() {
        return Err(SequencingError::PlatformNotFound(
            payload.platform_id.clone(),
        ));
    }

    let job_id = Uuid::new_v4();
    let now = Utc::now();

    // Insert the job into database
    let job = sqlx::query_as::<_, SequencingJob>(
        r#"
        INSERT INTO sequencing_jobs (
            id, name, description, status, priority, platform_id,
            workflow_id, sample_sheet_id, created_by, estimated_start,
            metadata, created_at, updated_at
        )
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13)
        RETURNING *
        "#,
    )
    .bind(job_id)
    .bind(&payload.name)
    .bind(&payload.description)
    .bind(JobStatus::Draft)
    .bind(payload.priority)
    .bind(&payload.platform_id)
    .bind(&payload.workflow_id)
    .bind(&payload.sample_sheet_id)
    .bind(Uuid::new_v4()) // TODO: Get from auth context
    .bind(payload.estimated_start)
    .bind(payload.metadata.unwrap_or_else(|| json!({})))
    .bind(now)
    .bind(now)
    .fetch_one(&state.db_pool.pool)
    .await?;

    // Send notification about job creation
    if let Err(e) = state
        .notification_client
        .send_job_created_notification(&job)
        .await
    {
        warn!("Failed to send job creation notification: {}", e);
    }

    info!("Created sequencing job: {} (ID: {})", job.name, job.id);

    let response = JobResponse {
        id: job.id,
        name: job.name,
        status: job.status,
        priority: job.priority,
        platform_id: job.platform_id,
        workflow_id: job.workflow_id,
        progress_percentage: Some(0.0),
        estimated_completion: job.estimated_completion,
        created_at: job.created_at,
    };

    Ok((StatusCode::CREATED, Json(json!(response))))
}

/// List sequencing jobs with filtering and pagination
pub async fn list_jobs(
    State(state): State<AppState>,
    Query(params): Query<ListJobsQuery>,
) -> Result<Json<Value>> {
    info!("Listing sequencing jobs with filters: {:?}", params);

    let page = params.page.unwrap_or(1);
    let limit = params.limit.unwrap_or(50).min(100); // Cap at 100
    let offset = (page.saturating_sub(1)) * limit;

    // Note: For now using simplified query, dynamic filtering can be added later

    // Execute query (simplified version for now)
    let jobs = sqlx::query_as::<_, SequencingJob>(
        r#"
        SELECT 
            id, name, description, status, 
            priority, platform_id, workflow_id,
            sample_sheet_id, run_id, created_by, assigned_to,
            estimated_start, estimated_completion, actual_start,
            actual_completion, metadata, created_at, updated_at,
            platform_id as platform, name as job_name,
            actual_start as started_at, actual_completion as completed_at
        FROM sequencing_jobs 
        ORDER BY created_at DESC 
        LIMIT $1 OFFSET $2
        "#,
    )
    .bind(limit as i64)
    .bind(offset as i64)
    .fetch_all(&state.db_pool.pool)
    .await?;

    // Get total count for pagination
    let total_count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM sequencing_jobs")
        .fetch_one(&state.db_pool.pool)
        .await?;

    let job_responses: Vec<JobResponse> = jobs
        .into_iter()
        .map(|job| JobResponse {
            id: job.id,
            name: job.name.clone(),
            status: job.status.clone(),
            priority: job.priority.clone(),
            platform_id: job.platform_id,
            workflow_id: job.workflow_id,
            progress_percentage: calculate_job_progress(&job.status),
            estimated_completion: job.estimated_completion,
            created_at: job.created_at,
        })
        .collect();

    info!(
        "Retrieved {} jobs (page {}, limit {})",
        job_responses.len(),
        page,
        limit
    );

    Ok(Json(json!({
        "jobs": job_responses,
        "pagination": {
            "page": page,
            "limit": limit,
            "total": total_count,
            "pages": (total_count as f64 / limit as f64).ceil() as u32
        }
    })))
}

/// Get a specific sequencing job by ID
pub async fn get_job(
    State(state): State<AppState>,
    Path(job_id): Path<Uuid>,
) -> Result<Json<SequencingJob>> {
    info!("Retrieving sequencing job: {}", job_id);

    let job = sqlx::query_as::<_, SequencingJob>(
        r#"
        SELECT 
            id, name, description, status, 
            priority, platform_id, workflow_id,
            sample_sheet_id, run_id, created_by, assigned_to,
            estimated_start, estimated_completion, actual_start,
            actual_completion, metadata, created_at, updated_at,
            platform_id as platform, name as job_name,
            actual_start as started_at, actual_completion as completed_at
        FROM sequencing_jobs 
        WHERE id = $1
        "#,
    )
    .bind(job_id)
    .fetch_optional(&state.db_pool.pool)
    .await?
    .ok_or_else(|| SequencingError::JobNotFound(job_id.to_string()))?;

    info!("Retrieved job: {} ({})", job.name, job.id);

    Ok(Json(job))
}

/// Update a sequencing job
pub async fn update_job(
    State(state): State<AppState>,
    Path(job_id): Path<Uuid>,
    Json(payload): Json<UpdateJobRequest>,
) -> Result<Json<SequencingJob>> {
    info!("Updating sequencing job: {}", job_id);

    // Check if job exists
    let existing_job = sqlx::query_as::<_, SequencingJob>(
        r#"
        SELECT 
            id, name, description, status, 
            priority, platform_id, workflow_id,
            sample_sheet_id, run_id, created_by, assigned_to,
            estimated_start, estimated_completion, actual_start,
            actual_completion, metadata, created_at, updated_at,
            platform_id as platform, name as job_name,
            actual_start as started_at, actual_completion as completed_at
        FROM sequencing_jobs 
        WHERE id = $1
        "#,
    )
    .bind(job_id)
    .fetch_optional(&state.db_pool.pool)
    .await?
    .ok_or_else(|| SequencingError::JobNotFound(job_id.to_string()))?;

    // Update only provided fields
    let updated_job = sqlx::query_as::<_, SequencingJob>(
        r#"
        UPDATE sequencing_jobs 
        SET 
            name = COALESCE($2, name),
            description = COALESCE($3, description),
            priority = COALESCE($4, priority),
            assigned_to = COALESCE($5, assigned_to),
            estimated_start = COALESCE($6, estimated_start),
            metadata = COALESCE($7, metadata),
            updated_at = NOW()
        WHERE id = $1
        RETURNING 
            id, name, description, status, 
            priority, platform_id, workflow_id,
            sample_sheet_id, run_id, created_by, assigned_to,
            estimated_start, estimated_completion, actual_start,
            actual_completion, metadata, created_at, updated_at,
            platform_id as platform, name as job_name,
            actual_start as started_at, actual_completion as completed_at
        "#,
    )
    .bind(job_id)
    .bind(payload.name)
    .bind(payload.description)
    .bind(payload.priority.map(|p| p as Priority))
    .bind(payload.assigned_to)
    .bind(payload.estimated_start)
    .bind(payload.metadata)
    .fetch_one(&state.db_pool.pool)
    .await?;

    info!("Updated job: {} ({})", updated_job.name, updated_job.id);

    Ok(Json(updated_job))
}

/// Delete a sequencing job
pub async fn delete_job(
    State(state): State<AppState>,
    Path(job_id): Path<Uuid>,
) -> Result<StatusCode> {
    info!("Deleting sequencing job: {}", job_id);

    // Check if job exists and is not in a terminal state
    let job = sqlx::query_as::<_, SequencingJob>(
        r#"
        SELECT 
            id, name, description, status, 
            priority, platform_id, workflow_id,
            sample_sheet_id, run_id, created_by, assigned_to,
            estimated_start, estimated_completion, actual_start,
            actual_completion, metadata, created_at, updated_at,
            platform_id as platform, name as job_name,
            actual_start as started_at, actual_completion as completed_at
        FROM sequencing_jobs 
        WHERE id = $1
        "#,
    )
    .bind(job_id)
    .fetch_optional(&state.db_pool.pool)
    .await?
    .ok_or_else(|| SequencingError::JobNotFound(job_id.to_string()))?;

    // Don't allow deletion of active jobs
    if job.status.is_active() {
        return Err(SequencingError::InvalidOperation(format!(
            "Cannot delete job {} - job is currently {}",
            job_id,
            serde_json::to_string(&job.status).unwrap_or_default()
        )));
    }

    // Delete the job
    let deleted_rows = sqlx::query("DELETE FROM sequencing_jobs WHERE id = $1")
        .bind(job_id)
        .execute(&state.db_pool.pool)
        .await?
        .rows_affected();

    if deleted_rows == 0 {
        return Err(SequencingError::JobNotFound(job_id.to_string()));
    }

    info!("Deleted job: {} ({})", job.name, job.id);

    Ok(StatusCode::NO_CONTENT)
}

/// Update job status with validation
pub async fn update_job_status(
    State(state): State<AppState>,
    Path(job_id): Path<Uuid>,
    Json(payload): Json<UpdateJobStatusRequest>,
) -> Result<Json<SequencingJob>> {
    info!("Updating job status: {} -> {:?}", job_id, payload.status);

    // Get current job
    let current_job = sqlx::query_as::<_, SequencingJob>(
        r#"
        SELECT 
            id, name, description, status, 
            priority, platform_id, workflow_id,
            sample_sheet_id, run_id, created_by, assigned_to,
            estimated_start, estimated_completion, actual_start,
            actual_completion, metadata, created_at, updated_at,
            platform_id as platform, name as job_name,
            actual_start as started_at, actual_completion as completed_at
        FROM sequencing_jobs 
        WHERE id = $1
        "#,
    )
    .bind(job_id)
    .fetch_optional(&state.db_pool.pool)
    .await?
    .ok_or_else(|| SequencingError::JobNotFound(job_id.to_string()))?;

    // Validate status transition
    if !current_job.status.can_transition_to(&payload.status) {
        return Err(SequencingError::InvalidStatusTransition {
            from: serde_json::to_string(&current_job.status).unwrap_or_default(),
            to: serde_json::to_string(&payload.status).unwrap_or_default(),
        });
    }

    // Update status and related timestamps
    let (actual_start, actual_completion) = match payload.status {
        JobStatus::Running => (Some(Utc::now()), None),
        JobStatus::Completed | JobStatus::Failed | JobStatus::Cancelled => {
            (current_job.actual_start, Some(Utc::now()))
        }
        _ => (current_job.actual_start, current_job.actual_completion),
    };

    let updated_job = sqlx::query_as::<_, SequencingJob>(
        r#"
        UPDATE sequencing_jobs 
        SET 
            status = $2,
            actual_start = $3,
            actual_completion = $4,
            updated_at = NOW()
        WHERE id = $1
        RETURNING 
            id, name, description, status, 
            priority, platform_id, workflow_id,
            sample_sheet_id, run_id, created_by, assigned_to,
            estimated_start, estimated_completion, actual_start,
            actual_completion, metadata, created_at, updated_at,
            platform_id as platform, name as job_name,
            actual_start as started_at, actual_completion as completed_at
        "#,
    )
    .bind(job_id)
    .bind(payload.status.clone() as JobStatus)
    .bind(actual_start)
    .bind(actual_completion)
    .fetch_one(&state.db_pool.pool)
    .await?;

    // Send status change notification
    if let Err(e) = state
        .notification_client
        .send_job_status_notification(&updated_job)
        .await
    {
        warn!("Failed to send job status notification: {}", e);
    }

    info!("Updated job status: {} -> {:?}", job_id, payload.status);

    Ok(Json(updated_job))
}

/// Clone an existing job
pub async fn clone_job(
    State(state): State<AppState>,
    Path(job_id): Path<Uuid>,
) -> Result<(StatusCode, Json<Value>)> {
    info!("Cloning sequencing job: {}", job_id);

    // Get the job to clone
    let original_job = sqlx::query_as::<_, SequencingJob>(
        r#"
        SELECT 
            id, name, description, status, 
            priority, platform_id, workflow_id,
            sample_sheet_id, run_id, created_by, assigned_to,
            estimated_start, estimated_completion, actual_start,
            actual_completion, metadata, created_at, updated_at,
            platform_id as platform, name as job_name,
            actual_start as started_at, actual_completion as completed_at
        FROM sequencing_jobs 
        WHERE id = $1
        "#,
    )
    .bind(job_id)
    .fetch_optional(&state.db_pool.pool)
    .await?
    .ok_or_else(|| SequencingError::JobNotFound(job_id.to_string()))?;

    // Create cloned job
    let cloned_job_id = Uuid::new_v4();
    let now = Utc::now();
    let cloned_name = format!("{} (Copy)", original_job.name);

    let cloned_job = sqlx::query_as::<_, SequencingJob>(
        r#"
        INSERT INTO sequencing_jobs (
            id, name, description, status, priority, platform_id,
            workflow_id, sample_sheet_id, created_by, metadata,
            created_at, updated_at
        )
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12)
        RETURNING 
            id, name, description, status, 
            priority, platform_id, workflow_id,
            sample_sheet_id, run_id, created_by, assigned_to,
            estimated_start, estimated_completion, actual_start,
            actual_completion, metadata, created_at, updated_at,
            platform_id as platform, name as job_name,
            actual_start as started_at, actual_completion as completed_at
        "#,
    )
    .bind(cloned_job_id)
    .bind(cloned_name)
    .bind(original_job.description)
    .bind(JobStatus::Draft as JobStatus)
    .bind(original_job.priority as Priority)
    .bind(original_job.platform_id)
    .bind(original_job.workflow_id)
    .bind(original_job.sample_sheet_id)
    .bind(original_job.created_by)
    .bind(original_job.metadata)
    .bind(now)
    .bind(now)
    .fetch_one(&state.db_pool.pool)
    .await?;

    info!("Cloned job: {} -> {}", job_id, cloned_job.id);

    let response = JobResponse {
        id: cloned_job.id,
        name: cloned_job.name,
        status: cloned_job.status,
        priority: cloned_job.priority,
        platform_id: cloned_job.platform_id,
        workflow_id: cloned_job.workflow_id,
        progress_percentage: Some(0.0),
        estimated_completion: cloned_job.estimated_completion,
        created_at: cloned_job.created_at,
    };

    Ok((StatusCode::CREATED, Json(json!(response))))
}

/// Cancel a job
pub async fn cancel_job(
    State(state): State<AppState>,
    Path(job_id): Path<Uuid>,
) -> Result<Json<SequencingJob>> {
    info!("Cancelling sequencing job: {}", job_id);

    // Get current job
    let current_job = sqlx::query_as::<_, SequencingJob>(
        r#"
        SELECT 
            id, name, description, status, 
            priority, platform_id, workflow_id,
            sample_sheet_id, run_id, created_by, assigned_to,
            estimated_start, estimated_completion, actual_start,
            actual_completion, metadata, created_at, updated_at,
            platform_id as platform, name as job_name,
            actual_start as started_at, actual_completion as completed_at
        FROM sequencing_jobs 
        WHERE id = $1
        "#,
    )
    .bind(job_id)
    .fetch_optional(&state.db_pool.pool)
    .await?
    .ok_or_else(|| SequencingError::JobNotFound(job_id.to_string()))?;

    // Check if job can be cancelled
    if current_job.status.is_terminal() {
        return Err(SequencingError::InvalidOperation(format!(
            "Cannot cancel job {} - job is already in terminal state: {:?}",
            job_id, current_job.status
        )));
    }

    // Update to cancelled status
    let cancelled_job = sqlx::query_as::<_, SequencingJob>(
        r#"
        UPDATE sequencing_jobs 
        SET 
            status = 'cancelled',
            actual_completion = NOW(),
            updated_at = NOW()
        WHERE id = $1
        RETURNING 
            id, name, description, status, 
            priority, platform_id, workflow_id,
            sample_sheet_id, run_id, created_by, assigned_to,
            estimated_start, estimated_completion, actual_start,
            actual_completion, metadata, created_at, updated_at,
            platform_id as platform, name as job_name,
            actual_start as started_at, actual_completion as completed_at
        "#,
    )
    .bind(job_id)
    .fetch_one(&state.db_pool.pool)
    .await?;

    // Send cancellation notification
    if let Err(e) = state
        .notification_client
        .send_job_cancelled_notification(&cancelled_job)
        .await
    {
        warn!("Failed to send job cancellation notification: {}", e);
    }

    info!(
        "Cancelled job: {} ({})",
        cancelled_job.name, cancelled_job.id
    );

    Ok(Json(cancelled_job))
}

// Helper function to calculate job progress based on status
fn calculate_job_progress(status: &JobStatus) -> Option<f64> {
    match status {
        JobStatus::Draft => Some(0.0),
        JobStatus::Submitted => Some(10.0),
        JobStatus::Validated => Some(20.0),
        JobStatus::Queued => Some(30.0),
        JobStatus::Running => Some(50.0), // Could be more sophisticated
        JobStatus::Completed => Some(100.0),
        JobStatus::Failed => Some(0.0),
        JobStatus::Cancelled => Some(0.0),
        JobStatus::OnHold => Some(25.0),
    }
}
