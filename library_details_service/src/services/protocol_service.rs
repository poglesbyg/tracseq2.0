use crate::error::{Result, ServiceError};
use crate::models::*;
use sqlx::PgPool;
use uuid::Uuid;
use chrono::Utc;
use validator::Validate;

#[derive(Clone)]
pub struct ProtocolService {
    pool: PgPool,
}

impl ProtocolService {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn create_protocol(&self, request: CreateProtocolRequest) -> Result<Protocol> {
        request.validate().map_err(|e| ServiceError::Validation {
            message: e.to_string(),
        })?;

        let id = Uuid::new_v4();
        let now = Utc::now();

        let protocol = sqlx::query_as::<_, Protocol>(
            r#"
            INSERT INTO protocols (
                id, name, version, library_type, description, steps,
                parameters, kit_id, platform_compatibility, quality_thresholds,
                is_active, created_at, updated_at
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13)
            RETURNING id, name, version, library_type, description, steps,
                      parameters, kit_id, platform_compatibility, quality_thresholds,
                      is_active, created_at, updated_at
            "#,
        )
        .bind(id)
        .bind(&request.name)
        .bind(&request.version)
        .bind(&request.library_type)
        .bind(&request.description)
        .bind(&request.steps)
        .bind(&request.parameters)
        .bind(request.kit_id)
        .bind(&request.platform_compatibility)
        .bind(&request.quality_thresholds)
        .bind(true)
        .bind(now)
        .bind(now)
        .fetch_one(&self.pool)
        .await?;

        tracing::info!("Created protocol: {}", protocol.id);
        Ok(protocol)
    }

    pub async fn get_protocol(&self, id: Uuid) -> Result<Protocol> {
        let protocol = sqlx::query_as::<_, Protocol>(
            r#"
            SELECT id, name, version, library_type, description, steps,
                   parameters, kit_id, platform_compatibility, quality_thresholds,
                   is_active, created_at, updated_at
            FROM protocols
            WHERE id = $1
            "#,
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await?
        .ok_or(ServiceError::ProtocolNotFound { id })?;

        Ok(protocol)
    }

    pub async fn list_protocols(&self, library_type: Option<String>, is_active: Option<bool>) -> Result<Vec<Protocol>> {
        let protocols = sqlx::query_as::<_, Protocol>(
            r#"
            SELECT id, name, version, library_type, description, steps,
                   parameters, kit_id, platform_compatibility, quality_thresholds,
                   is_active, created_at, updated_at
            FROM protocols
            WHERE ($1::text IS NULL OR library_type = $1) 
              AND ($2::boolean IS NULL OR is_active = $2)
            ORDER BY created_at DESC
            "#,
        )
        .bind(&library_type)
        .bind(is_active)
        .fetch_all(&self.pool)
        .await?;

        Ok(protocols)
    }

    pub async fn update_protocol(&self, id: Uuid, request: CreateProtocolRequest) -> Result<Protocol> {
        request.validate().map_err(|e| ServiceError::Validation {
            message: e.to_string(),
        })?;

        // Check if protocol exists
        self.get_protocol(id).await?;

        let now = Utc::now();

        let protocol = sqlx::query_as::<_, Protocol>(
            r#"
            UPDATE protocols
            SET name = $2,
                version = $3,
                library_type = $4,
                description = $5,
                steps = $6,
                parameters = $7,
                kit_id = $8,
                platform_compatibility = $9,
                quality_thresholds = $10,
                updated_at = $11
            WHERE id = $1
            RETURNING id, name, version, library_type, description, steps,
                      parameters, kit_id, platform_compatibility, quality_thresholds,
                      is_active, created_at, updated_at
            "#,
        )
        .bind(id)
        .bind(&request.name)
        .bind(&request.version)
        .bind(&request.library_type)
        .bind(&request.description)
        .bind(&request.steps)
        .bind(&request.parameters)
        .bind(request.kit_id)
        .bind(&request.platform_compatibility)
        .bind(&request.quality_thresholds)
        .bind(now)
        .fetch_one(&self.pool)
        .await?;

        tracing::info!("Updated protocol: {}", protocol.id);
        Ok(protocol)
    }

    pub async fn validate_protocol(&self, id: Uuid) -> Result<Vec<String>> {
        let protocol = self.get_protocol(id).await?;
        let mut errors = Vec::new();

        // Validate protocol structure
        if protocol.steps.is_null() || protocol.steps.as_array().unwrap_or(&vec![]).is_empty() {
            errors.push("Protocol must have at least one step".to_string());
        }

        // Validate each step has required fields
        if let Some(steps) = protocol.steps.as_array() {
            for (i, step) in steps.iter().enumerate() {
                if step.get("name").is_none() {
                    errors.push(format!("Step {} is missing name", i + 1));
                }
                if step.get("description").is_none() {
                    errors.push(format!("Step {} is missing description", i + 1));
                }
            }
        }

        if !errors.is_empty() {
            return Err(ServiceError::ProtocolValidationFailed { errors });
        }

        Ok(vec!["Protocol validation passed".to_string()])
    }

    pub async fn get_protocol_steps(&self, id: Uuid) -> Result<serde_json::Value> {
        let protocol = self.get_protocol(id).await?;
        Ok(protocol.steps)
    }

    pub async fn recommend_protocol(&self, library_type: String, sample_requirements: serde_json::Value) -> Result<Vec<ProtocolRecommendation>> {
        let protocols = self.list_protocols(Some(library_type.clone()), Some(true)).await?;
        
        let mut recommendations = Vec::new();

        for protocol in protocols {
            let mut compatibility_score = 0.5; // Base score
            let mut reasons = Vec::new();

            // Check library type match
            if protocol.library_type == library_type {
                compatibility_score += 0.3;
                reasons.push("Library type matches".to_string());
            }

            // Check if protocol has quality thresholds
            if protocol.quality_thresholds.is_some() {
                compatibility_score += 0.2;
                reasons.push("Has quality control thresholds".to_string());
            }

            // Additional logic based on sample requirements could be added here
            if sample_requirements.get("concentration").is_some() {
                compatibility_score += 0.1;
                reasons.push("Concentration requirements considered".to_string());
            }

            recommendations.push(ProtocolRecommendation {
                protocol_id: protocol.id,
                protocol_name: protocol.name,
                compatibility_score,
                reasons,
            });
        }

        // Sort by compatibility score (highest first)
        recommendations.sort_by(|a, b| b.compatibility_score.partial_cmp(&a.compatibility_score).unwrap());

        Ok(recommendations)
    }

    pub async fn activate_protocol(&self, id: Uuid) -> Result<Protocol> {
        let now = Utc::now();

        let protocol = sqlx::query_as::<_, Protocol>(
            r#"
            UPDATE protocols
            SET is_active = true,
                updated_at = $2
            WHERE id = $1
            RETURNING id, name, version, library_type, description, steps,
                      parameters, kit_id, platform_compatibility, quality_thresholds,
                      is_active, created_at, updated_at
            "#,
        )
        .bind(id)
        .bind(now)
        .fetch_optional(&self.pool)
        .await?
        .ok_or(ServiceError::ProtocolNotFound { id })?;

        tracing::info!("Activated protocol: {}", protocol.id);
        Ok(protocol)
    }

    pub async fn deactivate_protocol(&self, id: Uuid) -> Result<Protocol> {
        let now = Utc::now();

        let protocol = sqlx::query_as::<_, Protocol>(
            r#"
            UPDATE protocols
            SET is_active = false,
                updated_at = $2
            WHERE id = $1
            RETURNING id, name, version, library_type, description, steps,
                      parameters, kit_id, platform_compatibility, quality_thresholds,
                      is_active, created_at, updated_at
            "#,
        )
        .bind(id)
        .bind(now)
        .fetch_optional(&self.pool)
        .await?
        .ok_or(ServiceError::ProtocolNotFound { id })?;

        tracing::info!("Deactivated protocol: {}", protocol.id);
        Ok(protocol)
    }
}