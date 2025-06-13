use axum::{
    extract::{Multipart, Path, State},
    http::StatusCode,
    Json,
};
use serde_json::json;
use uuid::Uuid;

use crate::{
    models::template::{CreateTemplate, ParsedTemplateResponse, TemplateResponse, UpdateTemplate},
    repositories::{Repository, RepositoryFactory},
    services::template_service::TemplateService,
    AppComponents,
};

/// Upload a new template file with metadata
pub async fn upload_template(
    State(state): State<AppComponents>,
    mut multipart: Multipart,
) -> Result<Json<TemplateResponse>, (StatusCode, String)> {
    let mut file_content = Vec::new();
    let mut template_data = None;
    let mut original_filename = String::from("template.xlsx");

    while let Some(field) = multipart.next_field().await.map_err(|e| {
        (
            StatusCode::BAD_REQUEST,
            format!("Failed to process multipart form: {}", e),
        )
    })? {
        let name = field.name().unwrap_or_default();

        if name == "file" {
            // Get the original filename
            if let Some(filename) = field.file_name() {
                original_filename = filename.to_string();
            }

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
        .save_file(&original_filename, &file_content)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    // Determine file type from extension
    let file_type = std::path::Path::new(&original_filename)
        .extension()
        .and_then(|ext| ext.to_str())
        .unwrap_or("unknown");

    // Save template metadata to database using repository
    let template_repo = state.repositories.factory.template_repository();
    let mut create_template = template_data;

    // Set the file path and type directly in the struct
    create_template.file_path = file_path.to_string_lossy().to_string();
    create_template.file_type = file_type.to_string();

    // Keep original metadata, removing the file info since it's now in dedicated fields
    create_template.metadata = Some(json!({
        "fileSize": file_content.len(),
        "originalFileName": original_filename,
        "uploadedAt": chrono::Utc::now().to_rfc3339(),
        "original_metadata": create_template.metadata.unwrap_or(json!({}))
    }));

    let template = template_repo
        .create(create_template)
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
    let template_repo = state.repositories.factory.template_repository();
    let templates = template_repo
        .list(None, None)
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

/// Get a single template by ID
pub async fn get_template(
    State(state): State<AppComponents>,
    Path(template_id): Path<Uuid>,
) -> Result<Json<TemplateResponse>, (StatusCode, String)> {
    let template_repo = state.repositories.factory.template_repository();
    let template_service = TemplateService::new(template_repo);

    let template = template_service
        .get_template(template_id)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?
        .ok_or((StatusCode::NOT_FOUND, "Template not found".to_string()))?;

    Ok(Json(TemplateResponse {
        id: template.id,
        name: template.name,
        description: template.description,
        created_at: template.created_at,
        metadata: template.metadata,
    }))
}

/// Update a template by ID
pub async fn update_template(
    State(state): State<AppComponents>,
    Path(template_id): Path<Uuid>,
    Json(updates): Json<UpdateTemplate>,
) -> Result<Json<TemplateResponse>, (StatusCode, String)> {
    let template_repo = state.repositories.factory.template_repository();
    let template_service = TemplateService::new(template_repo);

    let template = template_service
        .update_template(template_id, updates)
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

/// Get parsed spreadsheet data for a specific template
pub async fn get_template_data(
    State(state): State<AppComponents>,
    Path(template_id): Path<Uuid>,
) -> Result<Json<ParsedTemplateResponse>, (StatusCode, String)> {
    // Create template service instance with repository
    let template_repo = state.repositories.factory.template_repository();
    let template_service = TemplateService::new(template_repo);

    // Get template from database
    let template = template_service
        .get_template(template_id)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?
        .ok_or((StatusCode::NOT_FOUND, "Template not found".to_string()))?;

    // Parse the spreadsheet data
    let spreadsheet_data = template_service
        .parse_spreadsheet(&template.file_path)
        .await
        .map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("Failed to parse spreadsheet: {}", e),
            )
        })?;

    let template_response = TemplateResponse {
        id: template.id,
        name: template.name,
        description: template.description,
        created_at: template.created_at,
        metadata: template.metadata,
    };

    Ok(Json(ParsedTemplateResponse {
        template: template_response,
        data: spreadsheet_data,
    }))
}

/// Delete a template by ID
pub async fn delete_template(
    State(state): State<AppComponents>,
    Path(template_id): Path<Uuid>,
) -> Result<StatusCode, (StatusCode, String)> {
    // Create template service instance with repository
    let template_repo = state.repositories.factory.template_repository();
    let template_service = TemplateService::new(template_repo);

    // Get template to find the file path before deleting
    let template = template_service
        .get_template(template_id)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?
        .ok_or((StatusCode::NOT_FOUND, "Template not found".to_string()))?;

    // Delete the template from database
    template_service
        .delete_template(template_id)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    // Delete the physical file from storage
    let file_path = std::path::Path::new(&template.file_path);
    if file_path.exists() {
        if let Err(e) = std::fs::remove_file(file_path) {
            // Log the error but don't fail the request since the DB deletion succeeded
            eprintln!(
                "Warning: Failed to delete template file {}: {}",
                template.file_path, e
            );
        }
    }

    Ok(StatusCode::NO_CONTENT)
}
