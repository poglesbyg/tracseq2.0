use axum::{
    extract::{Multipart, State},
    http::StatusCode,
    Json,
};
use serde_json::json;

use crate::{
    models::template::{CreateTemplate, Template, TemplateResponse},
    AppComponents,
};

/// Upload a new template file with metadata
pub async fn upload_template(
    State(state): State<AppComponents>,
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
    .fetch_one(&state.database.pool)
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

/// List all available templates
pub async fn list_templates(
    State(state): State<AppComponents>,
) -> Result<Json<Vec<TemplateResponse>>, (StatusCode, String)> {
    let templates = sqlx::query_as!(
        Template,
        r#"
        SELECT * FROM templates
        ORDER BY created_at DESC
        "#
    )
    .fetch_all(&state.database.pool)
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
