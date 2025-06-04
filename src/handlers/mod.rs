use axum::{
    extract::{Multipart, Path, State},
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use serde_json::json;
use uuid::Uuid;

use crate::{
    models::template::{CreateTemplate, Template, TemplateResponse},
    sample_submission::{CreateSample, Sample},
    sequencing::{CreateJob, JobStatus, SequencingJob},
    AppState,
};

#[derive(serde::Serialize)]
pub struct DashboardStats {
    total_templates: i64,
    total_samples: i64,
    pending_sequencing: i64,
    completed_sequencing: i64,
}

pub async fn health_check() -> impl IntoResponse {
    StatusCode::OK
}

pub async fn upload_template(
    State(state): State<AppState>,
    mut multipart: Multipart,
) -> Result<Json<TemplateResponse>, (StatusCode, String)> {
    let mut file_content = Vec::new();
    let mut template_data = None;

    while let Some(field) = multipart.next_field().await.map_err(|e| {
        (
            StatusCode::BAD_REQUEST,
            format!("Failed to process multipart form: {}", e),
        )
    })? {
        let name = field.name().unwrap_or_default();

        if name == "file" {
            file_content = field
                .bytes()
                .await
                .map_err(|e| {
                    (
                        StatusCode::BAD_REQUEST,
                        format!("Failed to read file: {}", e),
                    )
                })?
                .to_vec();
        } else if name == "template" {
            let json_str = field.text().await.map_err(|e| {
                (
                    StatusCode::BAD_REQUEST,
                    format!("Failed to read template data: {}", e),
                )
            })?;
            template_data = Some(serde_json::from_str::<CreateTemplate>(&json_str).map_err(
                |e| {
                    (
                        StatusCode::BAD_REQUEST,
                        format!("Invalid template data: {}", e),
                    )
                },
            )?);
        }
    }

    let template_data =
        template_data.ok_or((StatusCode::BAD_REQUEST, "Missing template data".to_string()))?;

    // Save file to storage
    let file_path = state
        .storage
        .save_file("template.xlsx", &file_content)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    // Save template metadata to database
    let template = sqlx::query_as!(
        Template,
        r#"
        INSERT INTO templates (name, description, file_path, file_type, metadata)
        VALUES ($1, $2, $3, $4, $5)
        RETURNING *
        "#,
        template_data.name,
        template_data.description,
        &file_path.to_string_lossy(),
        "xlsx",
        template_data.metadata.unwrap_or(json!({}))
    )
    .fetch_one(&state.pool)
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    Ok(Json(TemplateResponse {
        id: template.id,
        name: template.name,
        description: template.description,
        created_at: template.created_at,
        metadata: template.metadata,
    }))
}

pub async fn list_templates(
    State(state): State<AppState>,
) -> Result<Json<Vec<TemplateResponse>>, (StatusCode, String)> {
    let templates = sqlx::query_as!(
        Template,
        r#"
        SELECT * FROM templates
        ORDER BY created_at DESC
        "#
    )
    .fetch_all(&state.pool)
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    let responses = templates
        .into_iter()
        .map(|t| TemplateResponse {
            id: t.id,
            name: t.name,
            description: t.description,
            created_at: t.created_at,
            metadata: t.metadata,
        })
        .collect();

    Ok(Json(responses))
}

pub async fn create_sample(
    State(state): State<AppState>,
    Json(sample): Json<CreateSample>,
) -> Result<Json<Sample>, (StatusCode, String)> {
    state
        .sample_manager
        .create_sample(sample)
        .await
        .map(Json)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))
}

pub async fn list_samples(
    State(state): State<AppState>,
) -> Result<Json<Vec<Sample>>, (StatusCode, String)> {
    state
        .sample_manager
        .list_samples()
        .await
        .map(Json)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))
}

pub async fn validate_sample(
    State(state): State<AppState>,
    Path(sample_id): Path<Uuid>,
) -> Result<Json<Sample>, (StatusCode, String)> {
    state
        .sample_manager
        .validate_sample(sample_id)
        .await
        .map(Json)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))
}

pub async fn create_sequencing_job(
    State(state): State<AppState>,
    Json(job): Json<CreateJob>,
) -> Result<Json<SequencingJob>, (StatusCode, String)> {
    state
        .sequencing_manager
        .create_job(job)
        .await
        .map(Json)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))
}

pub async fn list_sequencing_jobs(
    State(state): State<AppState>,
) -> Result<Json<Vec<SequencingJob>>, (StatusCode, String)> {
    state
        .sequencing_manager
        .list_jobs()
        .await
        .map(Json)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))
}

pub async fn update_job_status(
    State(state): State<AppState>,
    Path(job_id): Path<Uuid>,
    Json(status): Json<JobStatus>,
) -> Result<Json<SequencingJob>, (StatusCode, String)> {
    state
        .sequencing_manager
        .update_job_status(job_id, status)
        .await
        .map(Json)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))
}

pub async fn get_dashboard_stats(
    State(state): State<AppState>,
) -> Result<Json<DashboardStats>, (StatusCode, String)> {
    let stats = sqlx::query!(
        r#"
        WITH stats AS (
            SELECT
                (SELECT COUNT(*) FROM templates) as total_templates,
                (SELECT COUNT(*) FROM samples) as total_samples,
                (SELECT COUNT(*) FROM sequencing_jobs WHERE status = 'pending') as pending_sequencing,
                (SELECT COUNT(*) FROM sequencing_jobs WHERE status = 'completed') as completed_sequencing
        )
        SELECT * FROM stats
        "#
    )
    .fetch_one(&state.pool)
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    Ok(Json(DashboardStats {
        total_templates: stats.total_templates.unwrap_or(0),
        total_samples: stats.total_samples.unwrap_or(0),
        pending_sequencing: stats.pending_sequencing.unwrap_or(0),
        completed_sequencing: stats.completed_sequencing.unwrap_or(0),
    }))
}
