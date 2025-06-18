use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;
use validator::Validate;

/// Template status enumeration
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::Type, PartialEq, Eq)]
#[sqlx(type_name = "template_status", rename_all = "snake_case")]
pub enum TemplateStatus {
    Draft,
    Published,
    Archived,
    Deprecated,
}

/// Field type enumeration for template fields
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::Type, PartialEq, Eq)]
#[sqlx(type_name = "field_type", rename_all = "snake_case")]
pub enum FieldType {
    Text,
    Number,
    Date,
    DateTime,
    Email,
    Phone,
    Url,
    Select,
    MultiSelect,
    Radio,
    Checkbox,
    Boolean,
    File,
    TextArea,
    RichText,
    Password,
    Hidden,
}

/// Validation rule type enumeration
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::Type, PartialEq, Eq)]
#[sqlx(type_name = "validation_rule_type", rename_all = "snake_case")]
pub enum ValidationRuleType {
    Required,
    MinLength,
    MaxLength,
    Pattern,
    MinValue,
    MaxValue,
    Email,
    Phone,
    Url,
    Date,
    Custom,
    CrossField,
}

/// Main template entity
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Template {
    pub id: Uuid,
    pub name: String,
    pub description: Option<String>,
    pub template_type: String,
    pub status: TemplateStatus,
    pub version: String,
    pub category: Option<String>,
    pub tags: Vec<String>,
    pub is_public: bool,
    pub is_system: bool,
    pub form_config: serde_json::Value,
    pub metadata: serde_json::Value,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub created_by: String,
    pub updated_by: Option<String>,
    pub published_at: Option<DateTime<Utc>>,
    pub published_by: Option<String>,
}

/// Template field definition
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct TemplateField {
    pub id: Uuid,
    pub template_id: Uuid,
    pub name: String,
    pub label: String,
    pub description: Option<String>,
    pub field_type: FieldType,
    pub is_required: bool,
    pub is_readonly: bool,
    pub is_hidden: bool,
    pub default_value: Option<serde_json::Value>,
    pub placeholder: Option<String>,
    pub help_text: Option<String>,
    pub sort_order: i32,
    pub group_name: Option<String>,
    pub field_config: serde_json::Value,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Template creation request
#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct CreateTemplateRequest {
    #[validate(length(
        min = 1,
        max = 255,
        message = "Template name must be between 1 and 255 characters"
    ))]
    pub name: String,

    #[validate(length(max = 1000, message = "Description must be at most 1000 characters"))]
    pub description: Option<String>,

    #[validate(length(
        min = 1,
        max = 100,
        message = "Template type must be between 1 and 100 characters"
    ))]
    pub template_type: String,

    pub category: Option<String>,
    pub tags: Option<Vec<String>>,
    pub is_public: Option<bool>,
    pub form_config: Option<serde_json::Value>,
    pub metadata: Option<serde_json::Value>,
}

/// Template update request
#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct UpdateTemplateRequest {
    #[validate(length(
        min = 1,
        max = 255,
        message = "Template name must be between 1 and 255 characters"
    ))]
    pub name: Option<String>,

    #[validate(length(max = 1000, message = "Description must be at most 1000 characters"))]
    pub description: Option<String>,

    pub template_type: Option<String>,
    pub status: Option<TemplateStatus>,
    pub category: Option<String>,
    pub tags: Option<Vec<String>>,
    pub is_public: Option<bool>,
    pub form_config: Option<serde_json::Value>,
    pub metadata: Option<serde_json::Value>,
}

/// Template response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TemplateResponse {
    pub id: Uuid,
    pub name: String,
    pub description: Option<String>,
    pub template_type: String,
    pub status: TemplateStatus,
    pub version: String,
    pub category: Option<String>,
    pub tags: Vec<String>,
    pub is_public: bool,
    pub is_system: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub created_by: String,
    pub updated_by: Option<String>,
    pub field_count: Option<i64>,
    pub usage_count: Option<i64>,
}

/// Field creation request
#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct CreateFieldRequest {
    #[validate(length(
        min = 1,
        max = 100,
        message = "Field name must be between 1 and 100 characters"
    ))]
    pub name: String,

    #[validate(length(
        min = 1,
        max = 255,
        message = "Field label must be between 1 and 255 characters"
    ))]
    pub label: String,

    pub description: Option<String>,
    pub field_type: FieldType,
    pub is_required: Option<bool>,
    pub is_readonly: Option<bool>,
    pub is_hidden: Option<bool>,
    pub default_value: Option<serde_json::Value>,
    pub placeholder: Option<String>,
    pub help_text: Option<String>,
    pub group_name: Option<String>,
    pub field_config: Option<serde_json::Value>,
}

/// Template search filters
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct TemplateSearchFilters {
    pub status: Option<TemplateStatus>,
    pub template_type: Option<String>,
    pub category: Option<String>,
    pub tags: Option<Vec<String>>,
    pub created_by: Option<String>,
    pub is_public: Option<bool>,
    pub is_system: Option<bool>,
    pub created_after: Option<DateTime<Utc>>,
    pub created_before: Option<DateTime<Utc>>,
    pub search_text: Option<String>,
    pub limit: Option<i64>,
    pub offset: Option<i64>,
}

/// Form validation request
#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct ValidateFormDataRequest {
    pub template_id: Uuid,
    pub form_data: serde_json::Value,
    pub validate_dependencies: Option<bool>,
    pub strict_mode: Option<bool>,
}

/// Form validation response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FormValidationResponse {
    pub is_valid: bool,
    pub field_errors: std::collections::HashMap<String, Vec<String>>,
    pub global_errors: Vec<String>,
    pub warnings: Vec<String>,
    pub validated_data: Option<serde_json::Value>,
}

/// Template statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TemplateStatistics {
    pub total_templates: i64,
    pub status_counts: std::collections::HashMap<String, i64>,
    pub type_counts: std::collections::HashMap<String, i64>,
    pub category_counts: std::collections::HashMap<String, i64>,
    pub templates_created_today: i64,
    pub templates_created_this_week: i64,
    pub templates_created_this_month: i64,
}

/// Pagination response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PaginatedTemplateResponse {
    pub templates: Vec<TemplateResponse>,
    pub total_count: i64,
    pub page: i64,
    pub page_size: i64,
    pub total_pages: i64,
}
