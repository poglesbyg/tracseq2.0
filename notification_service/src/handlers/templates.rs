use axum::{
    Json,
    extract::{Path, Query, State},
};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::{AppState, error::Result, models::*};

#[derive(Debug, Deserialize)]
pub struct CreateTemplateRequest {
    pub name: String,
    pub description: Option<String>,
    pub template_type: TemplateType,
    pub subject: Option<String>,
    pub body_html: Option<String>,
    pub body_text: String,
    pub variables: Vec<String>,
    pub metadata: Option<serde_json::Value>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateTemplateRequest {
    pub name: Option<String>,
    pub description: Option<String>,
    pub subject: Option<String>,
    pub body_html: Option<String>,
    pub body_text: Option<String>,
    pub variables: Option<Vec<String>>,
    pub metadata: Option<serde_json::Value>,
}

#[derive(Debug, Deserialize)]
pub struct PreviewTemplateRequest {
    pub template_data: serde_json::Value,
}

#[derive(Debug, Serialize)]
pub struct TemplateResponse {
    pub id: Uuid,
    pub name: String,
    pub description: Option<String>,
    pub template_type: TemplateType,
    pub subject: Option<String>,
    pub body_html: Option<String>,
    pub body_text: String,
    pub variables: Vec<String>,
    pub metadata: serde_json::Value,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

/// Create a new template
/// POST /templates
pub async fn create_template(
    State(state): State<AppState>,
    Json(request): Json<CreateTemplateRequest>,
) -> Result<Json<TemplateResponse>> {
    let user_id = Uuid::new_v4(); // TODO: Extract from JWT

    let template = state
        .notification_service
        .create_template(
            request.name,
            request.description,
            request.template_type,
            request.subject,
            request.body_html,
            request.body_text,
            request.variables,
            request.metadata.unwrap_or_default(),
            user_id,
        )
        .await?;

    Ok(Json(TemplateResponse {
        id: template.id,
        name: template.name,
        description: template.description,
        template_type: template.template_type,
        subject: template.subject,
        body_html: template.body_html,
        body_text: template.body_text,
        variables: template.variables,
        metadata: template.metadata,
        created_at: template.created_at,
        updated_at: template.updated_at,
    }))
}

/// List templates
/// GET /templates
pub async fn list_templates(
    State(state): State<AppState>,
    Query(query): Query<ListQuery>,
) -> Result<Json<Vec<TemplateResponse>>> {
    let limit = query.limit.unwrap_or(50).min(1000);
    let offset = query.offset.unwrap_or(0);

    let templates = state
        .notification_service
        .list_templates(limit, offset)
        .await?;

    let responses = templates
        .into_iter()
        .map(|template| TemplateResponse {
            id: template.id,
            name: template.name,
            description: template.description,
            template_type: template.template_type,
            subject: template.subject,
            body_html: template.body_html,
            body_text: template.body_text,
            variables: template.variables,
            metadata: template.metadata,
            created_at: template.created_at,
            updated_at: template.updated_at,
        })
        .collect();

    Ok(Json(responses))
}

/// Get a specific template
/// GET /templates/{id}
pub async fn get_template(
    State(state): State<AppState>,
    Path(template_id): Path<Uuid>,
) -> Result<Json<TemplateResponse>> {
    let template = state.notification_service.get_template(template_id).await?;

    Ok(Json(TemplateResponse {
        id: template.id,
        name: template.name,
        description: template.description,
        template_type: template.template_type,
        subject: template.subject,
        body_html: template.body_html,
        body_text: template.body_text,
        variables: template.variables,
        metadata: template.metadata,
        created_at: template.created_at,
        updated_at: template.updated_at,
    }))
}

/// Update a template
/// PUT /templates/{id}
pub async fn update_template(
    State(state): State<AppState>,
    Path(template_id): Path<Uuid>,
    Json(request): Json<UpdateTemplateRequest>,
) -> Result<Json<TemplateResponse>> {
    let template = state
        .notification_service
        .update_template(
            template_id,
            request.name,
            request.description,
            request.subject,
            request.body_html,
            request.body_text,
            request.variables,
            request.metadata,
        )
        .await?;

    Ok(Json(TemplateResponse {
        id: template.id,
        name: template.name,
        description: template.description,
        template_type: template.template_type,
        subject: template.subject,
        body_html: template.body_html,
        body_text: template.body_text,
        variables: template.variables,
        metadata: template.metadata,
        created_at: template.created_at,
        updated_at: template.updated_at,
    }))
}

/// Delete a template
/// DELETE /templates/{id}
pub async fn delete_template(
    State(state): State<AppState>,
    Path(template_id): Path<Uuid>,
) -> Result<Json<serde_json::Value>> {
    state
        .notification_service
        .delete_template(template_id)
        .await?;

    Ok(Json(serde_json::json!({
        "message": "Template deleted successfully",
        "template_id": template_id
    })))
}

/// Preview a template with data
/// POST /templates/{id}/preview
pub async fn preview_template(
    State(state): State<AppState>,
    Path(template_id): Path<Uuid>,
    Json(request): Json<PreviewTemplateRequest>,
) -> Result<Json<TemplatePreviewResponse>> {
    let preview = state
        .notification_service
        .preview_template(template_id, request.template_data)
        .await?;

    Ok(Json(preview))
}

/// Validate a template
/// POST /templates/{id}/validate
pub async fn validate_template(
    State(state): State<AppState>,
    Path(template_id): Path<Uuid>,
) -> Result<Json<serde_json::Value>> {
    let validation_result = state
        .notification_service
        .validate_template(template_id)
        .await?;

    Ok(Json(serde_json::json!({
        "valid": validation_result.is_valid,
        "errors": validation_result.errors,
        "warnings": validation_result.warnings
    })))
}

#[derive(Debug, Deserialize)]
pub struct ListQuery {
    pub limit: Option<i64>,
    pub offset: Option<i64>,
}
