use axum::{
    extract::{Path, State, Multipart},
    http::StatusCode,
    response::Json,
};
use serde_json::{json, Value};
use uuid::Uuid;
use crate::{
    AppState,
    models::{CreateTemplateRequest, UpdateTemplateRequest, TemplateSearchFilters, CreateFieldRequest, UpdateFieldRequest}
};

pub mod health {
    use super::*;

    pub async fn health_check() -> Json<Value> {
        Json(json!({"status": "healthy", "service": "template_service"}))
    }

    pub async fn readiness_check() -> Json<Value> {
        Json(json!({"status": "ready", "service": "template_service"}))
    }

    pub async fn metrics() -> Json<Value> {
        Json(json!({"metrics": {}}))
    }
}

pub mod templates {
    use super::*;

    pub async fn create_template(
        State(state): State<AppState>,
        Json(payload): Json<CreateTemplateRequest>
    ) -> Result<Json<Value>, StatusCode> {
        // TODO: Extract user from authentication middleware
        let created_by = "system"; // Temporary - should come from auth

        match state.template_service.create_template(payload, created_by).await {
            Ok(template) => Ok(Json(json!({
                "success": true,
                "template": template,
                "message": "Template created successfully"
            }))),
            Err(e) => {
                eprintln!("Error creating template: {}", e);
                Err(StatusCode::INTERNAL_SERVER_ERROR)
            }
        }
    }

    pub async fn list_templates(
        State(state): State<AppState>
    ) -> Result<Json<Value>, StatusCode> {
        let filters = TemplateSearchFilters::default();
        
        match state.template_service.list_templates(filters).await {
            Ok(response) => Ok(Json(json!({
                "success": true,
                "data": response.templates,
                "pagination": {
                    "total_count": response.total_count,
                    "page": response.page,
                    "page_size": response.page_size,
                    "total_pages": response.total_pages
                }
            }))),
            Err(e) => {
                eprintln!("Error listing templates: {}", e);
                Err(StatusCode::INTERNAL_SERVER_ERROR)
            }
        }
    }

    pub async fn get_template(
        State(state): State<AppState>,
        Path(template_id): Path<String>
    ) -> Result<Json<Value>, StatusCode> {
        let template_uuid = match Uuid::parse_str(&template_id) {
            Ok(uuid) => uuid,
            Err(_) => return Err(StatusCode::BAD_REQUEST),
        };

        match state.template_service.get_template(template_uuid).await {
            Ok(Some(template)) => Ok(Json(json!({
                "success": true,
                "template": template
            }))),
            Ok(None) => Err(StatusCode::NOT_FOUND),
            Err(e) => {
                eprintln!("Error getting template: {}", e);
                Err(StatusCode::INTERNAL_SERVER_ERROR)
            }
        }
    }

    pub async fn update_template(
        State(state): State<AppState>,
        Path(template_id): Path<String>,
        Json(payload): Json<UpdateTemplateRequest>
    ) -> Result<Json<Value>, StatusCode> {
        let template_uuid = match Uuid::parse_str(&template_id) {
            Ok(uuid) => uuid,
            Err(_) => return Err(StatusCode::BAD_REQUEST),
        };

        // TODO: Extract user from authentication middleware
        let updated_by = "system"; // Temporary - should come from auth

        match state.template_service.update_template(template_uuid, payload, updated_by).await {
            Ok(Some(template)) => Ok(Json(json!({
                "success": true,
                "template": template,
                "message": "Template updated successfully"
            }))),
            Ok(None) => Err(StatusCode::NOT_FOUND),
            Err(e) => {
                eprintln!("Error updating template: {}", e);
                Err(StatusCode::INTERNAL_SERVER_ERROR)
            }
        }
    }

    pub async fn delete_template(
        State(state): State<AppState>,
        Path(template_id): Path<String>
    ) -> Result<Json<Value>, StatusCode> {
        let template_uuid = match Uuid::parse_str(&template_id) {
            Ok(uuid) => uuid,
            Err(_) => return Err(StatusCode::BAD_REQUEST),
        };

        match state.template_service.delete_template(template_uuid).await {
            Ok(true) => Ok(Json(json!({
                "success": true,
                "message": "Template deleted successfully"
            }))),
            Ok(false) => Err(StatusCode::NOT_FOUND),
            Err(e) => {
                eprintln!("Error deleting template: {}", e);
                Err(StatusCode::INTERNAL_SERVER_ERROR)
            }
        }
    }

    pub async fn get_template_data(
        State(state): State<AppState>,
        Path(template_id): Path<String>
    ) -> Result<Json<Value>, StatusCode> {
        let template_uuid = match Uuid::parse_str(&template_id) {
            Ok(uuid) => uuid,
            Err(_) => return Err(StatusCode::BAD_REQUEST),
        };

        match state.template_service.get_template(template_uuid).await {
            Ok(Some(template)) => {
                Ok(Json(json!({
                    "id": template.id,
                    "name": template.name,
                    "description": template.description,
                    "template_type": template.template_type,
                    "status": template.status,
                    "version": template.version,
                    "category": template.category,
                    "tags": template.tags,
                    "is_public": template.is_public,
                    "is_system": template.is_system,
                    "created_at": template.created_at,
                    "updated_at": template.updated_at,
                    "created_by": template.created_by,
                    "updated_by": template.updated_by,
                    "field_count": template.field_count,
                    "usage_count": template.usage_count,
                    "content": "Template data available"
                })))
            }
            Ok(None) => Err(StatusCode::NOT_FOUND),
            Err(e) => {
                eprintln!("Error getting template data: {}", e);
                Err(StatusCode::INTERNAL_SERVER_ERROR)
            }
        }
    }

    pub async fn clone_template(Path(_template_id): Path<String>) -> Result<Json<Value>, StatusCode> {
        // TODO: Implement template cloning
        Ok(Json(json!({"message": "Template cloning not yet implemented"})))
    }
}

pub mod template_fields {
    use super::*;

    pub async fn create_field(
        State(state): State<AppState>,
        Path(template_id): Path<String>,
        Json(payload): Json<CreateFieldRequest>
    ) -> Result<Json<Value>, StatusCode> {
        let template_uuid = match Uuid::parse_str(&template_id) {
            Ok(uuid) => uuid,
            Err(_) => return Err(StatusCode::BAD_REQUEST),
        };

        match state.template_service.create_field(template_uuid, payload).await {
            Ok(field) => Ok(Json(json!({
                "success": true,
                "field": field,
                "message": "Field created successfully"
            }))),
            Err(e) => {
                eprintln!("Error creating field: {}", e);
                Err(StatusCode::INTERNAL_SERVER_ERROR)
            }
        }
    }

    pub async fn list_fields(
        State(state): State<AppState>,
        Path(template_id): Path<String>
    ) -> Result<Json<Value>, StatusCode> {
        let template_uuid = match Uuid::parse_str(&template_id) {
            Ok(uuid) => uuid,
            Err(_) => return Err(StatusCode::BAD_REQUEST),
        };

        match state.template_service.list_fields(template_uuid).await {
            Ok(fields) => Ok(Json(json!({
                "success": true,
                "fields": fields,
                "count": fields.len()
            }))),
            Err(e) => {
                eprintln!("Error listing fields: {}", e);
                Err(StatusCode::INTERNAL_SERVER_ERROR)
            }
        }
    }

    pub async fn get_field(
        State(state): State<AppState>,
        Path((template_id, field_id)): Path<(String, String)>
    ) -> Result<Json<Value>, StatusCode> {
        let template_uuid = match Uuid::parse_str(&template_id) {
            Ok(uuid) => uuid,
            Err(_) => return Err(StatusCode::BAD_REQUEST),
        };

        let field_uuid = match Uuid::parse_str(&field_id) {
            Ok(uuid) => uuid,
            Err(_) => return Err(StatusCode::BAD_REQUEST),
        };

        match state.template_service.get_field(template_uuid, field_uuid).await {
            Ok(Some(field)) => Ok(Json(json!({
                "success": true,
                "field": field
            }))),
            Ok(None) => Err(StatusCode::NOT_FOUND),
            Err(e) => {
                eprintln!("Error getting field: {}", e);
                Err(StatusCode::INTERNAL_SERVER_ERROR)
            }
        }
    }

    pub async fn update_field(
        State(state): State<AppState>,
        Path((template_id, field_id)): Path<(String, String)>,
        Json(payload): Json<UpdateFieldRequest>
    ) -> Result<Json<Value>, StatusCode> {
        let template_uuid = match Uuid::parse_str(&template_id) {
            Ok(uuid) => uuid,
            Err(_) => return Err(StatusCode::BAD_REQUEST),
        };

        let field_uuid = match Uuid::parse_str(&field_id) {
            Ok(uuid) => uuid,
            Err(_) => return Err(StatusCode::BAD_REQUEST),
        };

        match state.template_service.update_field(template_uuid, field_uuid, payload).await {
            Ok(Some(field)) => Ok(Json(json!({
                "success": true,
                "field": field,
                "message": "Field updated successfully"
            }))),
            Ok(None) => Err(StatusCode::NOT_FOUND),
            Err(e) => {
                eprintln!("Error updating field: {}", e);
                Err(StatusCode::INTERNAL_SERVER_ERROR)
            }
        }
    }

    pub async fn delete_field(
        State(state): State<AppState>,
        Path((template_id, field_id)): Path<(String, String)>
    ) -> Result<Json<Value>, StatusCode> {
        let template_uuid = match Uuid::parse_str(&template_id) {
            Ok(uuid) => uuid,
            Err(_) => return Err(StatusCode::BAD_REQUEST),
        };

        let field_uuid = match Uuid::parse_str(&field_id) {
            Ok(uuid) => uuid,
            Err(_) => return Err(StatusCode::BAD_REQUEST),
        };

        match state.template_service.delete_field(template_uuid, field_uuid).await {
            Ok(true) => Ok(Json(json!({
                "success": true,
                "message": "Field deleted successfully"
            }))),
            Ok(false) => Err(StatusCode::NOT_FOUND),
            Err(e) => {
                eprintln!("Error deleting field: {}", e);
                Err(StatusCode::INTERNAL_SERVER_ERROR)
            }
        }
    }
}

pub mod files {
    use super::*;
    use std::path::PathBuf;
    use tokio::fs;
    use tokio::io::AsyncWriteExt;

    pub async fn upload_template(
        State(state): State<AppState>,
        mut multipart: Multipart
    ) -> Result<Json<Value>, StatusCode> {
        let mut file_data: Option<Vec<u8>> = None;
        let mut file_name: Option<String> = None;
        let mut template_data: Option<CreateTemplateRequest> = None;

        // Process multipart form data
        while let Some(field) = multipart.next_field().await.map_err(|e| {
            eprintln!("Error reading multipart field: {}", e);
            StatusCode::BAD_REQUEST
        })? {
            let name = field.name().unwrap_or("").to_string();
            
            match name.as_str() {
                "file" => {
                    // Get the file name
                    file_name = field.file_name().map(|s| s.to_string());
                    
                    // Read file data
                    file_data = Some(field.bytes().await.map_err(|e| {
                        eprintln!("Error reading file data: {}", e);
                        StatusCode::BAD_REQUEST
                    })?.to_vec());
                }
                "template" => {
                    // Parse template metadata
                    let data = field.text().await.map_err(|e| {
                        eprintln!("Error reading template data: {}", e);
                        StatusCode::BAD_REQUEST
                    })?;
                    
                    template_data = serde_json::from_str(&data).ok();
                }
                _ => {
                    // Ignore other fields
                }
            }
        }

        // Validate required data
        let file_data = file_data.ok_or_else(|| {
            eprintln!("No file data received");
            StatusCode::BAD_REQUEST
        })?;
        
        let file_name = file_name.ok_or_else(|| {
            eprintln!("No file name received");
            StatusCode::BAD_REQUEST
        })?;

        // Create uploads directory if it doesn't exist
        let upload_dir = PathBuf::from("uploads/templates");
        fs::create_dir_all(&upload_dir).await.map_err(|e| {
            eprintln!("Error creating upload directory: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

        // Generate unique file name
        let file_id = Uuid::new_v4();
        let path_buf = PathBuf::from(&file_name);
        let file_extension = path_buf
            .extension()
            .and_then(|ext| ext.to_str())
            .unwrap_or("xlsx");
        let stored_file_name = format!("{}.{}", file_id, file_extension);
        let file_path = upload_dir.join(&stored_file_name);

        // Save file to disk
        let mut file = fs::File::create(&file_path).await.map_err(|e| {
            eprintln!("Error creating file: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;
        
        file.write_all(&file_data).await.map_err(|e| {
            eprintln!("Error writing file: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

        // Determine file type based on extension
        let file_type = match file_extension {
            "xlsx" | "xls" => "spreadsheet",
            "csv" => "csv",
            "docx" | "doc" => "document",
            "pdf" => "pdf",
            _ => "other",
        };

        // Convert file path to string for reuse
        let file_path_str = file_path.to_string_lossy().to_string();

        // Create template request
        let mut create_request = template_data.unwrap_or_else(|| CreateTemplateRequest {
            name: file_name.clone(),
            description: Some(format!("Uploaded file: {}", file_name)),
            template_type: file_type.to_string(),
            category: Some("uploaded".to_string()),
            tags: Some(vec![file_type.to_string()]),
            is_public: Some(false),
            form_config: None,
            metadata: Some(serde_json::json!({
                "original_filename": file_name.clone(),
                "file_path": file_path_str.clone(),
                "file_type": file_type,
                "file_size": file_data.len(),
                "uploaded_at": chrono::Utc::now()
            })),
        });

        // Ensure metadata includes file path and type
        if let Some(ref mut metadata) = create_request.metadata {
            metadata["file_path"] = serde_json::json!(file_path.to_string_lossy().to_string());
            metadata["file_type"] = serde_json::json!(file_type);
        }

        // Create template record
        let created_by = "system"; // TODO: Get from auth context
        
        match state.template_service.create_template(create_request, created_by).await {
            Ok(template) => Ok(Json(json!({
                "success": true,
                "template": template,
                "message": "Template uploaded successfully"
            }))),
            Err(e) => {
                // Clean up uploaded file on error
                let _ = fs::remove_file(&file_path).await;
                eprintln!("Error creating template: {}", e);
                Err(StatusCode::INTERNAL_SERVER_ERROR)
            }
        }
    }

    pub async fn download_template(Path(_template_id): Path<String>) -> Result<Json<Value>, StatusCode> {
        Ok(Json(json!({"message": "Template downloaded"})))
    }

    pub async fn export_template(Path(_template_id): Path<String>) -> Result<Json<Value>, StatusCode> {
        Ok(Json(json!({"message": "Template exported"})))
    }

    pub async fn import_templates(Json(_payload): Json<Value>) -> Result<Json<Value>, StatusCode> {
        Ok(Json(json!({"message": "Templates imported"})))
    }
}

pub mod versions {
    use super::*;

    pub async fn list_versions(Path(_template_id): Path<String>) -> Result<Json<Value>, StatusCode> {
        Ok(Json(json!({"versions": []})))
    }

    pub async fn create_version(Path(_template_id): Path<String>) -> Result<Json<Value>, StatusCode> {
        Ok(Json(json!({"message": "Version created"})))
    }

    pub async fn get_version(Path((_template_id, _version)): Path<(String, String)>) -> Result<Json<Value>, StatusCode> {
        Ok(Json(json!({"version": {}})))
    }

    pub async fn delete_version(Path((_template_id, _version)): Path<(String, String)>) -> Result<Json<Value>, StatusCode> {
        Ok(Json(json!({"message": "Version deleted"})))
    }

    pub async fn restore_version(Path((_template_id, _version)): Path<(String, String)>) -> Result<Json<Value>, StatusCode> {
        Ok(Json(json!({"message": "Version restored"})))
    }
}

pub mod forms {
    use super::*;

    pub async fn generate_form(Path(_template_id): Path<String>) -> Result<Json<Value>, StatusCode> {
        Ok(Json(json!({"form": {}})))
    }

    pub async fn validate_form_data(Path(_template_id): Path<String>) -> Result<Json<Value>, StatusCode> {
        Ok(Json(json!({"valid": true})))
    }

    pub async fn preview_form(Path(_template_id): Path<String>) -> Result<Json<Value>, StatusCode> {
        Ok(Json(json!({"preview": {}})))
    }

    pub async fn render_form(Path(_template_id): Path<String>) -> Result<Json<Value>, StatusCode> {
        Ok(Json(json!({"rendered": {}})))
    }
}

pub mod fields {
    use super::*;

    pub async fn list_fields(Path(_template_id): Path<String>) -> Result<Json<Value>, StatusCode> {
        Ok(Json(json!({"fields": []})))
    }

    pub async fn create_field(Path(_template_id): Path<String>) -> Result<Json<Value>, StatusCode> {
        Ok(Json(json!({"message": "Field created"})))
    }

    pub async fn get_field(Path((_template_id, _field_id)): Path<(String, String)>) -> Result<Json<Value>, StatusCode> {
        Ok(Json(json!({"field": {}})))
    }

    pub async fn update_field(Path((_template_id, _field_id)): Path<(String, String)>) -> Result<Json<Value>, StatusCode> {
        Ok(Json(json!({"message": "Field updated"})))
    }

    pub async fn delete_field(Path((_template_id, _field_id)): Path<(String, String)>) -> Result<Json<Value>, StatusCode> {
        Ok(Json(json!({"message": "Field deleted"})))
    }

    pub async fn reorder_fields(Path(_template_id): Path<String>) -> Result<Json<Value>, StatusCode> {
        Ok(Json(json!({"message": "Fields reordered"})))
    }
}

pub mod validation {
    use super::*;

    pub async fn get_validation_rules(Path(_template_id): Path<String>) -> Result<Json<Value>, StatusCode> {
        Ok(Json(json!({"rules": []})))
    }

    pub async fn create_validation_rule(Path(_template_id): Path<String>) -> Result<Json<Value>, StatusCode> {
        Ok(Json(json!({"message": "Validation rule created"})))
    }

    pub async fn update_validation_rule(Path((_template_id, _rule_id)): Path<(String, String)>) -> Result<Json<Value>, StatusCode> {
        Ok(Json(json!({"message": "Validation rule updated"})))
    }

    pub async fn delete_validation_rule(Path((_template_id, _rule_id)): Path<(String, String)>) -> Result<Json<Value>, StatusCode> {
        Ok(Json(json!({"message": "Validation rule deleted"})))
    }

    pub async fn validate_template_data(Path(_template_id): Path<String>) -> Result<Json<Value>, StatusCode> {
        Ok(Json(json!({"valid": true})))
    }
}

pub mod integration {
    use super::*;

    pub async fn create_sample_from_template(Json(_payload): Json<Value>) -> Result<Json<Value>, StatusCode> {
        Ok(Json(json!({"message": "Sample created from template"})))
    }

    pub async fn validate_sample_data(Json(_payload): Json<Value>) -> Result<Json<Value>, StatusCode> {
        Ok(Json(json!({"valid": true})))
    }

    pub async fn get_templates_for_samples() -> Result<Json<Value>, StatusCode> {
        Ok(Json(json!({"templates": []})))
    }
}

pub mod schemas {
    use super::*;

    pub async fn list_schemas() -> Result<Json<Value>, StatusCode> {
        Ok(Json(json!({"schemas": []})))
    }

    pub async fn get_schema(Path(_schema_name): Path<String>) -> Result<Json<Value>, StatusCode> {
        Ok(Json(json!({"schema": {}})))
    }

    pub async fn get_template_schema(Path(_template_id): Path<String>) -> Result<Json<Value>, StatusCode> {
        Ok(Json(json!({"schema": {}})))
    }

    pub async fn validate_template_schema(Path(_template_id): Path<String>) -> Result<Json<Value>, StatusCode> {
        Ok(Json(json!({"valid": true})))
    }
}

pub mod admin {
    use super::*;

    pub async fn get_template_statistics() -> Result<Json<Value>, StatusCode> {
        Ok(Json(json!({"statistics": {}})))
    }

    pub async fn cleanup_templates() -> Result<Json<Value>, StatusCode> {
        Ok(Json(json!({"message": "Templates cleaned up"})))
    }

    pub async fn migrate_templates() -> Result<Json<Value>, StatusCode> {
        Ok(Json(json!({"message": "Templates migrated"})))
    }

    pub async fn get_usage_statistics() -> Result<Json<Value>, StatusCode> {
        Ok(Json(json!({"usage": {}})))
    }

    pub async fn test_validation_rules() -> Result<Json<Value>, StatusCode> {
        Ok(Json(json!({"test_result": "passed"})))
    }
}