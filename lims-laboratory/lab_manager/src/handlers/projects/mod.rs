use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    Json,
};
use serde_json::json;
use uuid::Uuid;
use chrono::{Utc, NaiveDate};
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use std::sync::Arc;

use crate::{
    assembly::AppComponents,
    models::project::*,
};

// Request/Query structs
#[derive(Debug, Deserialize)]
pub struct ListProjectsQuery {
    pub status: Option<String>,
    pub priority: Option<String>,
    pub search: Option<String>,
    pub limit: Option<i32>,
    pub offset: Option<i32>,
}

#[derive(Debug, Deserialize)]
pub struct ProjectListFilters {
    pub status: Option<String>,
    pub priority: Option<String>,
    pub department: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct ListBatchesQuery {
    pub project_id: Option<Uuid>,
    pub status: Option<String>,
    pub search: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct CreateProjectRequest {
    pub project_code: String,
    pub name: String,
    pub description: Option<String>,
    pub project_type: String,
    pub status: Option<String>,
    pub priority: Option<String>,
    pub start_date: Option<NaiveDate>,
    pub target_end_date: Option<NaiveDate>,
    pub principal_investigator_id: Uuid,
    pub project_manager_id: Option<Uuid>,
    pub department: Option<String>,
    pub budget_approved: Option<sqlx::types::BigDecimal>,
    pub metadata: Option<serde_json::Value>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateProjectRequest {
    pub name: Option<String>,
    pub description: Option<String>,
    pub status: Option<String>,
    pub priority: Option<String>,
    pub target_end_date: Option<NaiveDate>,
    pub actual_end_date: Option<NaiveDate>,
    pub budget_used: Option<sqlx::types::BigDecimal>,
    pub metadata: Option<serde_json::Value>,
}

#[derive(Debug, Deserialize)]
pub struct CreateBatchRequest {
    pub batch_number: String,
    pub project_id: Uuid,
    pub batch_type: String,
    pub status: Option<String>,
    pub priority: Option<String>,
    pub sample_count: Option<i32>,
    pub metadata: Option<serde_json::Value>,
    pub notes: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct CreateFileRequest {
    pub project_id: Uuid,
    pub parent_id: Option<Uuid>,
    pub name: String,
    pub file_type: String,
    pub file_path: Option<String>,
    pub file_size: Option<i64>,
    pub mime_type: Option<String>,
    pub description: Option<String>,
    pub tags: Option<Vec<String>>,
    pub metadata: Option<serde_json::Value>,
}

#[derive(Debug, Deserialize)]
pub struct UploadFileRequest {
    pub parent_id: Option<Uuid>,
    pub name: String,
    pub description: Option<String>,
    pub tags: Option<Vec<String>>,
}

#[derive(Debug, Deserialize)]
pub struct CreateSignoffRequest {
    pub project_id: Uuid,
    pub batch_id: Option<Uuid>,
    pub signoff_type: String,
    pub signoff_level: String,
    pub comments: Option<String>,
    pub is_conditional: Option<bool>,
    pub conditions: Option<serde_json::Value>,
    pub expiry_date: Option<NaiveDate>,
}

#[derive(Debug, Deserialize)]
pub struct SignoffDecisionRequest {
    pub decision: String,
    pub comments: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct CreatePermissionRequest {
    pub project_id: Uuid,
    pub batch_id: Uuid,
    pub permission_type: String,
    pub reason: String,
    pub priority: Option<String>,
    pub metadata: Option<serde_json::Value>,
}

// Manager struct for project database operations
#[derive(Clone)]
pub struct ProjectManager {
    pool: PgPool,
}

impl ProjectManager {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    // Project CRUD operations
    pub async fn create_project(&self, request: CreateProjectRequest, created_by: Uuid) -> Result<Project, sqlx::Error> {
        sqlx::query_as::<_, Project>(
            r#"
            INSERT INTO projects (
                project_code, name, description, project_type, status, priority,
                start_date, target_end_date, principal_investigator_id, project_manager_id,
                department, budget_approved, metadata, created_by
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14)
            RETURNING *
            "#,
        )
        .bind(&request.project_code)
        .bind(&request.name)
        .bind(&request.description)
        .bind(&request.project_type)
        .bind(&request.status.unwrap_or("active".to_string()))
        .bind(&request.priority)
        .bind(&request.start_date)
        .bind(&request.target_end_date)
        .bind(&request.principal_investigator_id)
        .bind(&request.project_manager_id)
        .bind(&request.department)
        .bind(&request.budget_approved)
        .bind(&request.metadata)
        .bind(created_by)
        .fetch_one(&self.pool)
        .await
    }

    pub async fn get_project(&self, project_id: Uuid) -> Result<Option<Project>, sqlx::Error> {
        sqlx::query_as::<_, Project>(
            "SELECT * FROM projects WHERE id = $1"
        )
        .bind(project_id)
        .fetch_optional(&self.pool)
        .await
    }

    pub async fn list_projects(&self, filters: ProjectListFilters) -> Result<Vec<Project>, sqlx::Error> {
        let mut query = String::from("SELECT * FROM projects WHERE 1=1");
        let mut bindings = vec![];
        let mut param_count = 1;

        if let Some(status) = filters.status {
            query.push_str(&format!(" AND status = ${}", param_count));
            bindings.push(status);
            param_count += 1;
        }

        if let Some(priority) = filters.priority {
            query.push_str(&format!(" AND priority = ${}", param_count));
            bindings.push(priority);
            param_count += 1;
        }

        if let Some(department) = filters.department {
            query.push_str(&format!(" AND department = ${}", param_count));
            bindings.push(department);
            param_count += 1;
        }

        query.push_str(" ORDER BY created_at DESC LIMIT 100");

        let mut query_builder = sqlx::query_as::<_, Project>(&query);
        for binding in bindings {
            query_builder = query_builder.bind(binding);
        }

        query_builder.fetch_all(&self.pool).await
    }

    pub async fn update_project(&self, project_id: Uuid, request: UpdateProjectRequest) -> Result<Project, sqlx::Error> {
        sqlx::query_as::<_, Project>(
            r#"
            UPDATE projects 
            SET name = COALESCE($2, name),
                description = COALESCE($3, description),
                status = COALESCE($4, status),
                priority = COALESCE($5, priority),
                target_end_date = COALESCE($6, target_end_date),
                actual_end_date = COALESCE($7, actual_end_date),
                budget_used = COALESCE($8, budget_used),
                metadata = COALESCE($9, metadata),
                updated_at = NOW()
            WHERE id = $1
            RETURNING *
            "#,
        )
        .bind(project_id)
        .bind(&request.name)
        .bind(&request.description)
        .bind(&request.status)
        .bind(&request.priority)
        .bind(&request.target_end_date)
        .bind(&request.actual_end_date)
        .bind(&request.budget_used)
        .bind(&request.metadata)
        .fetch_one(&self.pool)
        .await
    }

    // Batch operations
    pub async fn create_batch(&self, request: CreateBatchRequest, created_by: Uuid) -> Result<Batch, sqlx::Error> {
        sqlx::query_as::<_, Batch>(
            r#"
            INSERT INTO batches (
                batch_number, project_id, batch_type, status, priority,
                sample_count, metadata, notes, created_by
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)
            RETURNING *
            "#,
        )
        .bind(&request.batch_number)
        .bind(&request.project_id)
        .bind(&request.batch_type)
        .bind(&request.status.unwrap_or("pending".to_string()))
        .bind(&request.priority)
        .bind(&request.sample_count)
        .bind(&request.metadata)
        .bind(&request.notes)
        .bind(created_by)
        .fetch_one(&self.pool)
        .await
    }

    pub async fn get_batch(&self, batch_id: Uuid) -> Result<Option<Batch>, sqlx::Error> {
        sqlx::query_as::<_, Batch>(
            "SELECT * FROM batches WHERE id = $1"
        )
        .bind(batch_id)
        .fetch_optional(&self.pool)
        .await
    }

    pub async fn list_project_batches(&self, project_id: Uuid) -> Result<Vec<Batch>, sqlx::Error> {
        sqlx::query_as::<_, Batch>(
            "SELECT * FROM batches WHERE project_id = $1 ORDER BY created_at DESC"
        )
        .bind(project_id)
        .fetch_all(&self.pool)
        .await
    }

    pub async fn search_batches(&self, search_term: &str) -> Result<Vec<Batch>, sqlx::Error> {
        sqlx::query_as::<_, Batch>(
            r#"
            SELECT * FROM batches 
            WHERE batch_number ILIKE $1 
               OR notes ILIKE $1
            ORDER BY created_at DESC
            LIMIT 50
            "#
        )
        .bind(format!("%{}%", search_term))
        .fetch_all(&self.pool)
        .await
    }

    // File operations
    pub async fn create_file(&self, request: CreateFileRequest, uploaded_by: Uuid) -> Result<ProjectFile, sqlx::Error> {
        sqlx::query_as::<_, ProjectFile>(
            r#"
            INSERT INTO project_files (
                project_id, parent_id, name, file_type, file_path,
                file_size, mime_type, description, tags, metadata, uploaded_by
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11)
            RETURNING *
            "#,
        )
        .bind(&request.project_id)
        .bind(&request.parent_id)
        .bind(&request.name)
        .bind(&request.file_type)
        .bind(&request.file_path)
        .bind(&request.file_size)
        .bind(&request.mime_type)
        .bind(&request.description)
        .bind(&request.tags)
        .bind(&request.metadata)
        .bind(uploaded_by)
        .fetch_one(&self.pool)
        .await
    }

    pub async fn list_project_files(&self, project_id: Uuid, parent_id: Option<Uuid>) -> Result<Vec<ProjectFile>, sqlx::Error> {
        let query = if parent_id.is_some() {
            "SELECT * FROM project_files WHERE project_id = $1 AND parent_id = $2 ORDER BY file_type DESC, name ASC"
        } else {
            "SELECT * FROM project_files WHERE project_id = $1 AND parent_id IS NULL ORDER BY file_type DESC, name ASC"
        };

        let mut query_builder = sqlx::query_as::<_, ProjectFile>(query).bind(project_id);
        
        if let Some(pid) = parent_id {
            query_builder = query_builder.bind(pid);
        }

        query_builder.fetch_all(&self.pool).await
    }

    // Template operations
    pub async fn list_templates(&self, category: Option<String>) -> Result<Vec<TemplateRepository>, sqlx::Error> {
        let query = if category.is_some() {
            "SELECT * FROM template_repository WHERE category = $1 AND is_active = true ORDER BY name"
        } else {
            "SELECT * FROM template_repository WHERE is_active = true ORDER BY category, name"
        };

        let mut query_builder = sqlx::query_as::<_, TemplateRepository>(query);
        
        if let Some(cat) = category {
            query_builder = query_builder.bind(cat);
        }

        query_builder.fetch_all(&self.pool).await
    }

    pub async fn get_template(&self, template_id: Uuid) -> Result<Option<TemplateRepository>, sqlx::Error> {
        // Increment download count
        sqlx::query(
            "UPDATE template_repository SET download_count = download_count + 1 WHERE id = $1"
        )
        .bind(template_id)
        .execute(&self.pool)
        .await?;

        sqlx::query_as::<_, TemplateRepository>(
            "SELECT * FROM template_repository WHERE id = $1"
        )
        .bind(template_id)
        .fetch_optional(&self.pool)
        .await
    }

    // Sign-off operations
    pub async fn create_signoff(&self, request: CreateSignoffRequest, signed_by: Uuid) -> Result<ProjectSignoff, sqlx::Error> {
        sqlx::query_as::<_, ProjectSignoff>(
            r#"
            INSERT INTO project_signoffs (
                project_id, batch_id, signoff_type, signoff_level,
                comments, is_conditional, conditions, expiry_date, signed_by
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)
            RETURNING *
            "#,
        )
        .bind(&request.project_id)
        .bind(&request.batch_id)
        .bind(&request.signoff_type)
        .bind(&request.signoff_level)
        .bind(&request.comments)
        .bind(&request.is_conditional.unwrap_or(false))
        .bind(&request.conditions)
        .bind(&request.expiry_date)
        .bind(signed_by)
        .fetch_one(&self.pool)
        .await
    }

    pub async fn list_signoffs(&self, project_id: Uuid, batch_id: Option<Uuid>) -> Result<Vec<ProjectSignoff>, sqlx::Error> {
        let query = if batch_id.is_some() {
            "SELECT * FROM project_signoffs WHERE project_id = $1 AND batch_id = $2 ORDER BY signed_at DESC"
        } else {
            "SELECT * FROM project_signoffs WHERE project_id = $1 ORDER BY signed_at DESC"
        };

        let mut query_builder = sqlx::query_as::<_, ProjectSignoff>(query).bind(project_id);
        
        if let Some(bid) = batch_id {
            query_builder = query_builder.bind(bid);
        }

        query_builder.fetch_all(&self.pool).await
    }

    // Permission queue operations
    pub async fn create_permission_request(&self, request: CreatePermissionRequest, requested_by: Uuid) -> Result<PermissionQueue, sqlx::Error> {
        sqlx::query_as::<_, PermissionQueue>(
            r#"
            INSERT INTO permission_queue (
                project_id, batch_id, permission_type, reason,
                priority, metadata, requested_by
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7)
            RETURNING *
            "#,
        )
        .bind(&request.project_id)
        .bind(&request.batch_id)
        .bind(&request.permission_type)
        .bind(&request.reason)
        .bind(&request.priority.unwrap_or("normal".to_string()))
        .bind(&request.metadata)
        .bind(requested_by)
        .fetch_one(&self.pool)
        .await
    }

    pub async fn update_permission_status(&self, permission_id: Uuid, status: String, reviewed_by: Uuid, review_comments: Option<String>) -> Result<PermissionQueue, sqlx::Error> {
        sqlx::query_as::<_, PermissionQueue>(
            r#"
            UPDATE permission_queue
            SET status = $2,
                reviewed_by = $3,
                reviewed_at = NOW(),
                review_comments = $4,
                updated_at = NOW()
            WHERE id = $1
            RETURNING *
            "#,
        )
        .bind(permission_id)
        .bind(status)
        .bind(reviewed_by)
        .bind(review_comments)
        .fetch_one(&self.pool)
        .await
    }

    pub async fn list_pending_permissions(&self) -> Result<Vec<PermissionQueue>, sqlx::Error> {
        sqlx::query_as::<_, PermissionQueue>(
            "SELECT * FROM permission_queue WHERE status = 'pending' ORDER BY created_at ASC"
        )
        .fetch_all(&self.pool)
        .await
    }
}

/// List projects with optional filters
pub async fn list_projects(
    State(state): State<Arc<AppComponents>>,
    Query(query): Query<ListProjectsQuery>,
) -> Result<Json<Vec<Project>>, (StatusCode, String)> {
    let pool = &state.database.pool;
    
    let manager = ProjectManager::new(pool.clone());
    
    let filters = ProjectListFilters {
        status: query.status,
        priority: query.priority,
        department: None, // Add to query if needed
    };
    
    match manager.list_projects(filters).await {
        Ok(projects) => Ok(Json(projects)),
        Err(e) => Err((StatusCode::INTERNAL_SERVER_ERROR, format!("Database error: {}", e)))
    }
}

/// Get a project by ID
pub async fn get_project(
    State(state): State<Arc<AppComponents>>,
    Path(project_id): Path<Uuid>,
) -> Result<Json<Project>, (StatusCode, String)> {
    let pool = &state.database.pool;
    
    let manager = ProjectManager::new(pool.clone());
    
    match manager.get_project(project_id).await {
        Ok(Some(project)) => Ok(Json(project)),
        Ok(None) => Err((StatusCode::NOT_FOUND, "Project not found".to_string())),
        Err(e) => Err((StatusCode::INTERNAL_SERVER_ERROR, format!("Database error: {}", e)))
    }
}

/// Create a new project
pub async fn create_project(
    State(state): State<Arc<AppComponents>>,
    Json(request): Json<CreateProjectRequest>,
) -> Result<Json<Project>, (StatusCode, String)> {
    let pool = &state.database.pool;
    
    let manager = ProjectManager::new(pool.clone());
    
    // TODO: Get actual user ID from authentication
    let created_by = Uuid::new_v4();
    
    match manager.create_project(request, created_by).await {
        Ok(project) => Ok(Json(project)),
        Err(e) => Err((StatusCode::INTERNAL_SERVER_ERROR, format!("Database error: {}", e)))
    }
}

/// Update a project
pub async fn update_project(
    State(state): State<Arc<AppComponents>>,
    Path(project_id): Path<Uuid>,
    Json(request): Json<UpdateProjectRequest>,
) -> Result<Json<Project>, (StatusCode, String)> {
    let pool = &state.database.pool;
    
    let manager = ProjectManager::new(pool.clone());
    
    match manager.update_project(project_id, request).await {
        Ok(project) => Ok(Json(project)),
        Err(e) => Err((StatusCode::INTERNAL_SERVER_ERROR, format!("Database error: {}", e)))
    }
}

/// Delete a project
pub async fn delete_project(
    State(state): State<Arc<AppComponents>>,
    Path(project_id): Path<Uuid>,
) -> Result<StatusCode, (StatusCode, String)> {
    // TODO: Implement project deletion
    Err((StatusCode::NOT_IMPLEMENTED, "Not implemented yet".to_string()))
}

/// List batches with optional filters
pub async fn list_batches(
    State(state): State<Arc<AppComponents>>,
    Query(query): Query<ListBatchesQuery>,
) -> Result<Json<Vec<Batch>>, (StatusCode, String)> {
    let pool = &state.database.pool;
    
    let manager = ProjectManager::new(pool.clone());
    
    if let Some(search) = query.search {
        match manager.search_batches(&search).await {
            Ok(batches) => Ok(Json(batches)),
            Err(e) => Err((StatusCode::INTERNAL_SERVER_ERROR, format!("Database error: {}", e)))
        }
    } else if let Some(project_id) = query.project_id {
        match manager.list_project_batches(project_id).await {
            Ok(batches) => Ok(Json(batches)),
            Err(e) => Err((StatusCode::INTERNAL_SERVER_ERROR, format!("Database error: {}", e)))
        }
    } else {
        // Return empty list if no filters provided
        Ok(Json(vec![]))
    }
}

/// Get a batch by ID
pub async fn get_batch(
    State(state): State<Arc<AppComponents>>,
    Path(batch_id): Path<Uuid>,
) -> Result<Json<Batch>, (StatusCode, String)> {
    let pool = &state.database.pool;
    
    let manager = ProjectManager::new(pool.clone());
    
    match manager.get_batch(batch_id).await {
        Ok(Some(batch)) => Ok(Json(batch)),
        Ok(None) => Err((StatusCode::NOT_FOUND, "Batch not found".to_string())),
        Err(e) => Err((StatusCode::INTERNAL_SERVER_ERROR, format!("Database error: {}", e)))
    }
}

/// Create a new batch
pub async fn create_batch(
    State(state): State<Arc<AppComponents>>,
    Json(request): Json<CreateBatchRequest>,
) -> Result<Json<Batch>, (StatusCode, String)> {
    let pool = &state.database.pool;
    
    let manager = ProjectManager::new(pool.clone());
    
    // TODO: Get actual user ID from authentication
    let created_by = Uuid::new_v4();
    
    match manager.create_batch(request, created_by).await {
        Ok(batch) => Ok(Json(batch)),
        Err(e) => Err((StatusCode::INTERNAL_SERVER_ERROR, format!("Database error: {}", e)))
    }
}

/// Get project files in a folder structure
pub async fn get_project_files(
    State(state): State<Arc<AppComponents>>,
    Path(project_id): Path<Uuid>,
    Query(params): Query<serde_json::Value>,
) -> Result<Json<Vec<ProjectFile>>, (StatusCode, String)> {
    let pool = &state.database.pool;
    
    let manager = ProjectManager::new(pool.clone());
    
    let parent_id = params.get("parent_id")
        .and_then(|v| v.as_str())
        .and_then(|s| Uuid::parse_str(s).ok());
    
    match manager.list_project_files(project_id, parent_id).await {
        Ok(files) => Ok(Json(files)),
        Err(e) => Err((StatusCode::INTERNAL_SERVER_ERROR, format!("Database error: {}", e)))
    }
}

/// Upload a file to a project
pub async fn upload_project_file(
    State(state): State<Arc<AppComponents>>,
    Path(project_id): Path<Uuid>,
    Json(request): Json<UploadFileRequest>,
) -> Result<Json<ProjectFile>, (StatusCode, String)> {
    // TODO: Implement file upload
    Err((StatusCode::NOT_IMPLEMENTED, "Not implemented yet".to_string()))
}

/// List template repository items
pub async fn list_templates_repository(
    State(state): State<Arc<AppComponents>>,
    Query(params): Query<serde_json::Value>,
) -> Result<Json<Vec<TemplateRepository>>, (StatusCode, String)> {
    let pool = &state.database.pool;
    
    let manager = ProjectManager::new(pool.clone());
    
    let category = params.get("category")
        .and_then(|v| v.as_str())
        .map(|s| s.to_string());
    
    match manager.list_templates(category).await {
        Ok(templates) => Ok(Json(templates)),
        Err(e) => Err((StatusCode::INTERNAL_SERVER_ERROR, format!("Database error: {}", e)))
    }
}

/// Download a template from repository
pub async fn download_template_repository(
    State(state): State<Arc<AppComponents>>,
    Path(template_id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    let pool = &state.database.pool;
    
    let manager = ProjectManager::new(pool.clone());
    
    match manager.get_template(template_id).await {
        Ok(Some(template)) => {
            // TODO: Generate presigned URL or actual file download response
            Ok(Json(json!({
                "template": template,
                "download_url": format!("/api/downloads/templates/{}", template_id)
            })))
        },
        Ok(None) => Err((StatusCode::NOT_FOUND, "Template not found".to_string())),
        Err(e) => Err((StatusCode::INTERNAL_SERVER_ERROR, format!("Database error: {}", e)))
    }
}

/// List project signoffs
pub async fn list_project_signoffs(
    State(state): State<Arc<AppComponents>>,
    Path(project_id): Path<Uuid>,
) -> Result<Json<Vec<ProjectSignoff>>, (StatusCode, String)> {
    let pool = &state.database.pool;
    
    let manager = ProjectManager::new(pool.clone());
    
    match manager.list_signoffs(project_id, None).await {
        Ok(signoffs) => Ok(Json(signoffs)),
        Err(e) => Err((StatusCode::INTERNAL_SERVER_ERROR, format!("Database error: {}", e)))
    }
}

/// Create a signoff request
pub async fn create_signoff(
    State(state): State<Arc<AppComponents>>,
    Json(request): Json<CreateSignoffRequest>,
) -> Result<Json<ProjectSignoff>, (StatusCode, String)> {
    let pool = &state.database.pool;
    
    let manager = ProjectManager::new(pool.clone());
    
    // TODO: Get actual user ID from authentication
    let signed_by = Uuid::new_v4();
    
    match manager.create_signoff(request, signed_by).await {
        Ok(signoff) => Ok(Json(signoff)),
        Err(e) => Err((StatusCode::INTERNAL_SERVER_ERROR, format!("Database error: {}", e)))
    }
}

/// Make a signoff decision (approve/reject)
pub async fn update_signoff(
    State(state): State<Arc<AppComponents>>,
    Path(signoff_id): Path<Uuid>,
    Json(request): Json<SignoffDecisionRequest>,
) -> Result<Json<ProjectSignoff>, (StatusCode, String)> {
    // TODO: Implement signoff decision
    Err((StatusCode::NOT_IMPLEMENTED, "Not implemented yet".to_string()))
}

/// List permission queue items
pub async fn list_permission_queue(
    State(state): State<Arc<AppComponents>>,
    Query(params): Query<serde_json::Value>,
) -> Result<Json<Vec<PermissionQueue>>, (StatusCode, String)> {
    let pool = &state.database.pool;
    
    let manager = ProjectManager::new(pool.clone());
    
    match manager.list_pending_permissions().await {
        Ok(permissions) => Ok(Json(permissions)),
        Err(e) => Err((StatusCode::INTERNAL_SERVER_ERROR, format!("Database error: {}", e)))
    }
}

/// Create a permission request
pub async fn create_permission_request(
    State(state): State<Arc<AppComponents>>,
    Json(request): Json<CreatePermissionRequest>,
) -> Result<Json<PermissionQueue>, (StatusCode, String)> {
    let pool = &state.database.pool;
    
    let manager = ProjectManager::new(pool.clone());
    
    // TODO: Get actual user ID from authentication
    let requested_by = Uuid::new_v4();
    
    match manager.create_permission_request(request, requested_by).await {
        Ok(permission) => Ok(Json(permission)),
        Err(e) => Err((StatusCode::INTERNAL_SERVER_ERROR, format!("Database error: {}", e)))
    }
}

/// Approve or reject a permission request
pub async fn update_permission_request(
    State(state): State<Arc<AppComponents>>,
    Path(permission_id): Path<Uuid>,
    Json(request): Json<serde_json::Value>,
) -> Result<Json<PermissionQueue>, (StatusCode, String)> {
    let pool = &state.database.pool;
    
    let manager = ProjectManager::new(pool.clone());
    
    // Extract status and comments from request
    let status = request.get("status")
        .and_then(|v| v.as_str())
        .unwrap_or("rejected")
        .to_string();
    
    let review_comments = request.get("comments")
        .and_then(|v| v.as_str())
        .map(|s| s.to_string());
    
    // TODO: Get actual user ID from authentication
    let reviewed_by = Uuid::new_v4();
    
    match manager.update_permission_status(permission_id, status, reviewed_by, review_comments).await {
        Ok(permission) => Ok(Json(permission)),
        Err(e) => Err((StatusCode::INTERNAL_SERVER_ERROR, format!("Database error: {}", e)))
    }
} 