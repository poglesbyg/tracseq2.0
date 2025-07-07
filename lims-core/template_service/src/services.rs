use anyhow::Result;
use uuid::Uuid;
use chrono::Utc;
use sqlx::Row;
use crate::{
    config::Config, 
    database::DatabasePool, 
    clients::{AuthClient, SampleClient},
    models::{
        TemplateResponse, CreateTemplateRequest, UpdateTemplateRequest,
        TemplateSearchFilters, PaginatedTemplateResponse, TemplateStatus,
        CreateFieldRequest, UpdateFieldRequest, FieldResponse
    }
};

#[derive(Clone)]
pub struct TemplateServiceImpl {
    db_pool: DatabasePool,
    config: Config,
    auth_client: AuthClient,
    sample_client: SampleClient,
}

impl TemplateServiceImpl {
    pub fn new(
        db_pool: DatabasePool,
        config: Config,
        auth_client: AuthClient,
        sample_client: SampleClient,
    ) -> Result<Self> {
        Ok(Self {
            db_pool,
            config,
            auth_client,
            sample_client,
        })
    }

    /// Create a new template
    pub async fn create_template(&self, request: CreateTemplateRequest, created_by: &str) -> Result<TemplateResponse> {
        let template_id = Uuid::new_v4();
        let now = Utc::now();
        
        let tags = request.tags.clone().unwrap_or_default();
        let is_public = request.is_public.unwrap_or(false);
        let template_data = request.form_config.clone().unwrap_or(serde_json::json!({}));
        let mut metadata = request.metadata.clone().unwrap_or(serde_json::json!({}));
        let category = request.category.clone().unwrap_or_else(|| "general".to_string());

        // Store additional fields in metadata since they don't exist in the DB schema
        metadata["template_type"] = serde_json::json!(request.template_type);
        metadata["tags"] = serde_json::json!(tags);
        metadata["is_public"] = serde_json::json!(is_public);
        metadata["is_system"] = serde_json::json!(false);

        let query = r#"
            INSERT INTO templates (
                id, name, description, category, status, version, 
                template_data, metadata, 
                is_active, created_by, created_at, updated_at
            ) VALUES (
                $1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12
            )
        "#;

        sqlx::query(query)
            .bind(template_id)
            .bind(&request.name)
            .bind(&request.description)
            .bind(&category)
            .bind(TemplateStatus::Draft)
            .bind(1i32)
            .bind(&template_data)
            .bind(&metadata)
            .bind(true)
            .bind(Uuid::parse_str(created_by).ok())
            .bind(now)
            .bind(now)
            .execute(self.db_pool.get_pool())
            .await?;

        Ok(TemplateResponse {
            id: template_id,
            name: request.name,
            description: request.description,
            template_type: request.template_type,
            status: TemplateStatus::Draft,
            version: "1.0".to_string(),
            category: request.category,
            tags,
            is_public,
            is_system: false,
            created_at: now,
            updated_at: now,
            created_by: created_by.to_string(),
            updated_by: None,
            field_count: Some(0),
            usage_count: Some(0),
        })
    }

    /// List templates with filtering and pagination
    pub async fn list_templates(&self, filters: TemplateSearchFilters) -> Result<PaginatedTemplateResponse> {
        let limit = filters.limit.unwrap_or(50).min(100);
        let offset = filters.offset.unwrap_or(0);

        let query = r#"
            SELECT 
                id, name, description, category, status, version,
                template_data, metadata, is_active, 
                created_by, created_at, updated_at
            FROM templates 
            WHERE is_active = true
            ORDER BY created_at DESC
            LIMIT $1 OFFSET $2
        "#;

        let rows = sqlx::query(query)
            .bind(limit)
            .bind(offset)
            .fetch_all(self.db_pool.get_pool())
            .await?;

        let templates: Vec<TemplateResponse> = rows
            .into_iter()
            .map(|row| {
                let metadata: serde_json::Value = row.get::<Option<serde_json::Value>, _>("metadata").unwrap_or(serde_json::json!({}));
                
                TemplateResponse {
                    id: row.get("id"),
                    name: row.get("name"),
                    description: row.get("description"),
                    template_type: metadata.get("template_type")
                        .and_then(|v| v.as_str())
                        .unwrap_or("form")
                        .to_string(),
                    status: row.get("status"),
                    version: row.get::<i32, _>("version").to_string(),
                    category: row.get("category"),
                    tags: metadata.get("tags")
                        .and_then(|v| v.as_array())
                        .map(|arr| arr.iter().filter_map(|v| v.as_str().map(|s| s.to_string())).collect())
                        .unwrap_or_default(),
                    is_public: metadata.get("is_public")
                        .and_then(|v| v.as_bool())
                        .unwrap_or(false),
                    is_system: metadata.get("is_system")
                        .and_then(|v| v.as_bool())
                        .unwrap_or(false),
                    created_at: row.get("created_at"),
                    updated_at: row.get("updated_at"),
                    created_by: row.get::<Option<String>, _>("created_by").unwrap_or_else(|| "system".to_string()),
                    updated_by: None,
                    field_count: Some(0),
                    usage_count: Some(0),
                }
            })
            .collect();

        // Get total count
        let count_query = "SELECT COUNT(*) FROM templates WHERE is_active = true";
        let total_count: i64 = sqlx::query_scalar(count_query)
            .fetch_one(self.db_pool.get_pool())
            .await?;

        let total_pages = (total_count + limit - 1) / limit;

        Ok(PaginatedTemplateResponse {
            templates,
            total_count,
            page: offset / limit + 1,
            page_size: limit,
            total_pages,
        })
    }

    /// Get a template by ID
    pub async fn get_template(&self, template_id: Uuid) -> Result<Option<TemplateResponse>> {
        let query = r#"
            SELECT 
                id, name, description, category, status, version,
                template_data, metadata, is_active, 
                created_by, created_at, updated_at
            FROM templates 
            WHERE id = $1 AND is_active = true
        "#;

        let row = sqlx::query(query)
            .bind(template_id)
            .fetch_optional(self.db_pool.get_pool())
            .await?;

        if let Some(row) = row {
            let metadata: serde_json::Value = row.get::<Option<serde_json::Value>, _>("metadata").unwrap_or(serde_json::json!({}));
            
            Ok(Some(TemplateResponse {
                id: row.get("id"),
                name: row.get("name"),
                description: row.get("description"),
                template_type: metadata.get("template_type")
                    .and_then(|v| v.as_str())
                    .unwrap_or("form")
                    .to_string(),
                status: row.get("status"),
                version: row.get::<i32, _>("version").to_string(),
                category: row.get("category"),
                tags: metadata.get("tags")
                    .and_then(|v| v.as_array())
                    .map(|arr| arr.iter().filter_map(|v| v.as_str().map(|s| s.to_string())).collect())
                    .unwrap_or_default(),
                is_public: metadata.get("is_public")
                    .and_then(|v| v.as_bool())
                    .unwrap_or(false),
                is_system: metadata.get("is_system")
                    .and_then(|v| v.as_bool())
                    .unwrap_or(false),
                created_at: row.get("created_at"),
                updated_at: row.get("updated_at"),
                created_by: row.get::<Option<String>, _>("created_by").unwrap_or_else(|| "system".to_string()),
                updated_by: None,
                field_count: Some(0),
                usage_count: Some(0),
            }))
        } else {
            Ok(None)
        }
    }

    /// Update a template
    pub async fn update_template(&self, template_id: Uuid, request: UpdateTemplateRequest, updated_by: &str) -> Result<Option<TemplateResponse>> {
        let now = Utc::now();

        let query = r#"
            UPDATE templates 
            SET name = COALESCE($2, name),
                description = COALESCE($3, description),
                category = COALESCE($4, category),
                metadata = COALESCE($5, metadata),
                updated_at = $6
            WHERE id = $1 AND is_active = true
            RETURNING id, name, description, category, status, version,
                     template_data, metadata, 
                     created_by, created_at, updated_at
        "#;

        let row = sqlx::query(query)
            .bind(template_id)
            .bind(&request.name)
            .bind(&request.description)
            .bind(&request.category)
            .bind(&request.metadata.unwrap_or(serde_json::json!({})))
            .bind(now)
            .fetch_optional(self.db_pool.get_pool())
            .await?;

        if let Some(row) = row {
            let metadata: serde_json::Value = row.get::<Option<serde_json::Value>, _>("metadata").unwrap_or(serde_json::json!({}));
            
            Ok(Some(TemplateResponse {
                id: row.get("id"),
                name: row.get("name"),
                description: row.get("description"),
                template_type: metadata.get("template_type")
                    .and_then(|v| v.as_str())
                    .unwrap_or("form")
                    .to_string(),
                status: row.get("status"),
                version: row.get::<i32, _>("version").to_string(),
                category: row.get("category"),
                tags: metadata.get("tags")
                    .and_then(|v| v.as_array())
                    .map(|arr| arr.iter().filter_map(|v| v.as_str().map(|s| s.to_string())).collect())
                    .unwrap_or_default(),
                is_public: metadata.get("is_public")
                    .and_then(|v| v.as_bool())
                    .unwrap_or(false),
                is_system: metadata.get("is_system")
                    .and_then(|v| v.as_bool())
                    .unwrap_or(false),
                created_at: row.get("created_at"),
                updated_at: row.get("updated_at"),
                created_by: row.get::<Option<String>, _>("created_by").unwrap_or_else(|| "system".to_string()),
                updated_by: Some(updated_by.to_string()),
                field_count: Some(0),
                usage_count: Some(0),
            }))
        } else {
            Ok(None)
        }
    }

    /// Delete a template (soft delete)
    pub async fn delete_template(&self, template_id: Uuid) -> Result<bool> {
        let query = "UPDATE templates SET is_active = false WHERE id = $1 AND is_active = true";
        
        let result = sqlx::query(query)
            .bind(template_id)
            .execute(self.db_pool.get_pool())
            .await?;

        Ok(result.rows_affected() > 0)
    }

    // Template Field Operations

    /// Create a new template field
    pub async fn create_field(&self, template_id: Uuid, request: CreateFieldRequest) -> Result<FieldResponse> {
        let field_id = Uuid::new_v4();
        
        // Clone values to avoid move issues
        let field_type = request.field_type.clone();
        let is_required = request.is_required.unwrap_or(false);
        let field_order = request.field_order.unwrap_or(0);
        let validation_rules = request.validation_rules.clone().unwrap_or(serde_json::json!({}));
        let field_options = request.field_options.clone().unwrap_or(serde_json::json!([]));
        let field_metadata = request.field_metadata.clone().unwrap_or(serde_json::json!({}));
        
        let query = r#"
            INSERT INTO template_fields (
                id, template_id, field_name, field_type, field_label, 
                field_description, is_required, field_order, validation_rules,
                default_value, field_options, field_metadata
            ) VALUES (
                $1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12
            )
        "#;

        sqlx::query(query)
            .bind(field_id)
            .bind(template_id)
            .bind(&request.field_name)
            .bind(&field_type)
            .bind(&request.field_label)
            .bind(&request.field_description)
            .bind(is_required)
            .bind(field_order)
            .bind(&validation_rules)
            .bind(&request.default_value)
            .bind(&field_options)
            .bind(&field_metadata)
            .execute(self.db_pool.get_pool())
            .await?;

        Ok(FieldResponse {
            id: field_id,
            template_id,
            field_name: request.field_name,
            field_type,
            field_label: request.field_label,
            field_description: request.field_description,
            is_required,
            field_order,
            validation_rules,
            default_value: request.default_value,
            field_options,
            field_metadata,
        })
    }

    /// List fields for a template
    pub async fn list_fields(&self, template_id: Uuid) -> Result<Vec<FieldResponse>> {
        let query = r#"
            SELECT 
                id, template_id, field_name, field_type, field_label,
                field_description, is_required, field_order, validation_rules,
                default_value, field_options, field_metadata
            FROM template_fields 
            WHERE template_id = $1
            ORDER BY field_order ASC, field_name ASC
        "#;

        let rows = sqlx::query(query)
            .bind(template_id)
            .fetch_all(self.db_pool.get_pool())
            .await?;

        let fields: Vec<FieldResponse> = rows
            .into_iter()
            .map(|row| FieldResponse {
                id: row.get("id"),
                template_id: row.get("template_id"),
                field_name: row.get("field_name"),
                field_type: row.get("field_type"),
                field_label: row.get("field_label"),
                field_description: row.get("field_description"),
                is_required: row.get::<Option<bool>, _>("is_required").unwrap_or(false),
                field_order: row.get::<Option<i32>, _>("field_order").unwrap_or(0),
                validation_rules: row.get::<Option<serde_json::Value>, _>("validation_rules").unwrap_or(serde_json::json!({})),
                default_value: row.get("default_value"),
                field_options: row.get::<Option<serde_json::Value>, _>("field_options").unwrap_or(serde_json::json!([])),
                field_metadata: row.get::<Option<serde_json::Value>, _>("field_metadata").unwrap_or(serde_json::json!({})),
            })
            .collect();

        Ok(fields)
    }

    /// Get a specific field by ID
    pub async fn get_field(&self, template_id: Uuid, field_id: Uuid) -> Result<Option<FieldResponse>> {
        let query = r#"
            SELECT 
                id, template_id, field_name, field_type, field_label,
                field_description, is_required, field_order, validation_rules,
                default_value, field_options, field_metadata
            FROM template_fields 
            WHERE template_id = $1 AND id = $2
        "#;

        let row = sqlx::query(query)
            .bind(template_id)
            .bind(field_id)
            .fetch_optional(self.db_pool.get_pool())
            .await?;

        if let Some(row) = row {
            Ok(Some(FieldResponse {
                id: row.get("id"),
                template_id: row.get("template_id"),
                field_name: row.get("field_name"),
                field_type: row.get("field_type"),
                field_label: row.get("field_label"),
                field_description: row.get("field_description"),
                is_required: row.get::<Option<bool>, _>("is_required").unwrap_or(false),
                field_order: row.get::<Option<i32>, _>("field_order").unwrap_or(0),
                validation_rules: row.get::<Option<serde_json::Value>, _>("validation_rules").unwrap_or(serde_json::json!({})),
                default_value: row.get("default_value"),
                field_options: row.get::<Option<serde_json::Value>, _>("field_options").unwrap_or(serde_json::json!([])),
                field_metadata: row.get::<Option<serde_json::Value>, _>("field_metadata").unwrap_or(serde_json::json!({})),
            }))
        } else {
            Ok(None)
        }
    }

    /// Update a template field
    pub async fn update_field(&self, template_id: Uuid, field_id: Uuid, request: UpdateFieldRequest) -> Result<Option<FieldResponse>> {
        let query = r#"
            UPDATE template_fields 
            SET field_name = COALESCE($3, field_name),
                field_type = COALESCE($4, field_type),
                field_label = COALESCE($5, field_label),
                field_description = COALESCE($6, field_description),
                is_required = COALESCE($7, is_required),
                field_order = COALESCE($8, field_order),
                validation_rules = COALESCE($9, validation_rules),
                default_value = COALESCE($10, default_value),
                field_options = COALESCE($11, field_options),
                field_metadata = COALESCE($12, field_metadata)
            WHERE template_id = $1 AND id = $2
            RETURNING id, template_id, field_name, field_type, field_label,
                     field_description, is_required, field_order, validation_rules,
                     default_value, field_options, field_metadata
        "#;

        let row = sqlx::query(query)
            .bind(template_id)
            .bind(field_id)
            .bind(&request.field_name)
            .bind(&request.field_type)
            .bind(&request.field_label)
            .bind(&request.field_description)
            .bind(&request.is_required)
            .bind(&request.field_order)
            .bind(&request.validation_rules)
            .bind(&request.default_value)
            .bind(&request.field_options)
            .bind(&request.field_metadata)
            .fetch_optional(self.db_pool.get_pool())
            .await?;

        if let Some(row) = row {
            Ok(Some(FieldResponse {
                id: row.get("id"),
                template_id: row.get("template_id"),
                field_name: row.get("field_name"),
                field_type: row.get("field_type"),
                field_label: row.get("field_label"),
                field_description: row.get("field_description"),
                is_required: row.get::<Option<bool>, _>("is_required").unwrap_or(false),
                field_order: row.get::<Option<i32>, _>("field_order").unwrap_or(0),
                validation_rules: row.get::<Option<serde_json::Value>, _>("validation_rules").unwrap_or(serde_json::json!({})),
                default_value: row.get("default_value"),
                field_options: row.get::<Option<serde_json::Value>, _>("field_options").unwrap_or(serde_json::json!([])),
                field_metadata: row.get::<Option<serde_json::Value>, _>("field_metadata").unwrap_or(serde_json::json!({})),
            }))
        } else {
            Ok(None)
        }
    }

    /// Delete a template field
    pub async fn delete_field(&self, template_id: Uuid, field_id: Uuid) -> Result<bool> {
        let query = "DELETE FROM template_fields WHERE template_id = $1 AND id = $2";
        
        let result = sqlx::query(query)
            .bind(template_id)
            .bind(field_id)
            .execute(self.db_pool.get_pool())
            .await?;

        Ok(result.rows_affected() > 0)
    }
}