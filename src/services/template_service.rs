use async_trait::async_trait;
use sqlx::PgPool;
use std::collections::HashMap;
use uuid::Uuid;

use crate::{
    models::template::{CreateTemplate, Template},
    services::{HealthCheck, HealthStatus, Service, ServiceConfig, ServiceHealth},
};

pub struct TemplateService {
    pool: PgPool,
}

impl TemplateService {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn create_template(&self, template: CreateTemplate) -> Result<Template, sqlx::Error> {
        sqlx::query_as::<_, Template>(
            r#"
            INSERT INTO templates (name, description, file_path, file_type, metadata)
            VALUES ($1, $2, $3, $4, $5)
            RETURNING *
            "#,
        )
        .bind(&template.name)
        .bind(&template.description)
        .bind("") // file_path will be set by upload handler
        .bind("xlsx")
        .bind(&template.metadata.unwrap_or(serde_json::json!({})))
        .fetch_one(&self.pool)
        .await
    }

    pub async fn get_template(&self, template_id: Uuid) -> Result<Template, sqlx::Error> {
        sqlx::query_as::<_, Template>(
            r#"
            SELECT * FROM templates WHERE id = $1
            "#,
        )
        .bind(template_id)
        .fetch_one(&self.pool)
        .await
    }

    pub async fn list_templates(&self) -> Result<Vec<Template>, sqlx::Error> {
        sqlx::query_as::<_, Template>(
            r#"
            SELECT * FROM templates ORDER BY created_at DESC
            "#,
        )
        .fetch_all(&self.pool)
        .await
    }

    pub async fn delete_template(&self, template_id: Uuid) -> Result<(), sqlx::Error> {
        sqlx::query(
            r#"
            DELETE FROM templates WHERE id = $1
            "#,
        )
        .bind(template_id)
        .execute(&self.pool)
        .await?;
        Ok(())
    }
}

#[async_trait]
impl Service for TemplateService {
    fn name(&self) -> &'static str {
        "template_service"
    }

    async fn health_check(&self) -> ServiceHealth {
        let mut checks = HashMap::new();

        // Test database connectivity by listing templates
        let start = std::time::Instant::now();
        let db_check = match self.list_templates().await {
            Ok(_) => HealthCheck {
                status: HealthStatus::Healthy,
                duration_ms: start.elapsed().as_millis() as u64,
                details: Some("Database connection successful".to_string()),
            },
            Err(e) => HealthCheck {
                status: HealthStatus::Unhealthy,
                duration_ms: start.elapsed().as_millis() as u64,
                details: Some(format!("Database error: {}", e)),
            },
        };

        checks.insert("database".to_string(), db_check.clone());

        ServiceHealth {
            status: db_check.status,
            message: Some("Template service health check".to_string()),
            checks,
        }
    }

    fn config(&self) -> ServiceConfig {
        ServiceConfig {
            name: "template_service".to_string(),
            version: "1.0.0".to_string(),
            dependencies: vec!["database".to_string()],
            settings: HashMap::new(),
        }
    }
}
