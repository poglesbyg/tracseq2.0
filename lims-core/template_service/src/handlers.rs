use axum::{
    extract::{Path, State, Multipart},
    http::StatusCode,
    response::Json,
};
use serde_json::{json, Value};
use uuid::Uuid;
use calamine::{Reader, open_workbook, Xlsx};
use std::path::PathBuf;
use sqlx::Row;
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

        // Query database directly to get metadata
        let query = r#"
            SELECT 
                id, name, description, category, status, version,
                template_data, metadata, is_active, 
                created_by, created_at, updated_at
            FROM templates 
            WHERE id = $1 AND is_active = true
        "#;

        let row = match sqlx::query(query)
            .bind(template_uuid)
            .fetch_optional(state.db_pool.get_pool())
            .await
        {
            Ok(Some(row)) => row,
            Ok(None) => return Err(StatusCode::NOT_FOUND),
            Err(e) => {
                eprintln!("Error querying template: {}", e);
                return Err(StatusCode::INTERNAL_SERVER_ERROR);
            }
        };

        // Extract metadata from database row
        let metadata: serde_json::Value = row.get::<Option<serde_json::Value>, _>("metadata")
            .unwrap_or(serde_json::json!({}));

        // Try to parse spreadsheet data if file path exists in metadata
        let parsed_data = if let Some(file_path_value) = metadata.get("file_path") {
            if let Some(file_path_str) = file_path_value.as_str() {
                parse_spreadsheet_file(file_path_str).await.unwrap_or_else(|e| {
                    eprintln!("Error parsing spreadsheet: {}", e);
                    json!({
                        "sheet_names": [],
                        "sheets": []
                    })
                })
            } else {
                json!({
                    "sheet_names": [],
                    "sheets": []
                })
            }
        } else {
            json!({
                "sheet_names": [],
                "sheets": []
            })
        };

        // Build template response manually from database row
        let template_response = json!({
            "id": row.get::<Uuid, _>("id"),
            "name": row.get::<String, _>("name"),
            "description": row.get::<Option<String>, _>("description"),
            "template_type": metadata.get("template_type")
                .and_then(|v| v.as_str())
                .unwrap_or("form"),
            "status": row.get::<crate::models::TemplateStatus, _>("status"),
            "version": row.get::<i32, _>("version").to_string(),
            "category": row.get::<Option<String>, _>("category"),
            "tags": metadata.get("tags")
                .and_then(|v| v.as_array())
                .map(|arr| arr.iter().filter_map(|v| v.as_str()).collect::<Vec<&str>>())
                .unwrap_or_default(),
            "is_public": metadata.get("is_public")
                .and_then(|v| v.as_bool())
                .unwrap_or(false),
            "is_system": metadata.get("is_system")
                .and_then(|v| v.as_bool())
                .unwrap_or(false),
            "created_at": row.get::<chrono::DateTime<chrono::Utc>, _>("created_at"),
            "updated_at": row.get::<chrono::DateTime<chrono::Utc>, _>("updated_at"),
            "created_by": row.get::<Option<String>, _>("created_by").unwrap_or_else(|| "system".to_string()),
            "updated_by": Option::<String>::None,
            "field_count": 0,
            "usage_count": 0,
            "metadata": metadata
        });

        Ok(Json(json!({
            "template": template_response,
            "data": parsed_data
        })))
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
        Ok(Json(json!({"statistics": {}})))
    }

    pub async fn test_validation_rules() -> Result<Json<Value>, StatusCode> {
        Ok(Json(json!({"message": "Validation rules tested"})))
    }
}

// Spreadsheet parsing functionality
async fn parse_spreadsheet_file(file_path: &str) -> Result<Value, Box<dyn std::error::Error + Send + Sync>> {
    let path = PathBuf::from(file_path);
    
    if !path.exists() {
        return Ok(json!({
            "sheet_names": [],
            "sheets": []
        }));
    }

    let extension = path.extension()
        .and_then(|ext| ext.to_str())
        .unwrap_or("")
        .to_lowercase();

    match extension.as_str() {
        "xlsx" => parse_excel_file(&path).await,
        "xls" => parse_excel_file(&path).await,
        "csv" => parse_csv_file(&path).await,
        _ => Ok(json!({
            "sheet_names": [],
            "sheets": [{
                "name": "Content",
                "headers": ["Content"],
                "rows": [["File type not supported for preview"]],
                "total_rows": 1,
                "total_columns": 1
            }]
        }))
    }
}

async fn parse_excel_file(path: &PathBuf) -> Result<Value, Box<dyn std::error::Error + Send + Sync>> {
    let mut workbook: Xlsx<_> = open_workbook(path)?;
    let sheet_names = workbook.sheet_names().to_owned();
    let mut sheets = Vec::new();

    for sheet_name in &sheet_names {
        if let Ok(range) = workbook.worksheet_range(sheet_name) {
            let mut headers = Vec::new();
            let mut rows = Vec::new();
            
            let mut row_iter = range.rows();
            
            // Get headers from first row
            if let Some(header_row) = row_iter.next() {
                headers = header_row.iter()
                    .map(|cell| cell.to_string())
                    .collect();
            }
            
            // Get data rows (limit to first 100 rows for performance)
            for (_i, row) in row_iter.take(100).enumerate() {
                let row_data: Vec<String> = row.iter()
                    .map(|cell| cell.to_string())
                    .collect();
                rows.push(row_data);
            }
            
            sheets.push(json!({
                "name": sheet_name,
                "headers": headers,
                "rows": rows,
                "total_rows": range.height(),
                "total_columns": range.width()
            }));
        }
    }

    Ok(json!({
        "sheet_names": sheet_names,
        "sheets": sheets
    }))
}

async fn parse_csv_file(path: &PathBuf) -> Result<Value, Box<dyn std::error::Error + Send + Sync>> {
    use std::fs::File;
    use csv::ReaderBuilder;

    let file = File::open(path)?;
    let mut reader = ReaderBuilder::new()
        .has_headers(true)
        .from_reader(file);

    let headers: Vec<String> = reader.headers()?
        .iter()
        .map(|h| h.to_string())
        .collect();

    let mut rows = Vec::new();
    let mut total_rows = 0;

    // Read data rows (limit to first 100 rows for performance)
    for (i, result) in reader.records().take(100).enumerate() {
        total_rows = i + 1;
        let record = result?;
        let row_data: Vec<String> = record.iter()
            .map(|field| field.to_string())
            .collect();
        rows.push(row_data);
    }

    let sheet_name = path.file_stem()
        .and_then(|name| name.to_str())
        .unwrap_or("Sheet1");

    Ok(json!({
        "sheet_names": [sheet_name],
        "sheets": [{
            "name": sheet_name,
            "headers": headers,
            "rows": rows,
            "total_rows": total_rows,
            "total_columns": headers.len()
        }]
    }))
}