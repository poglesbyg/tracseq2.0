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

        let protocol = sqlx::query_as!(
            Protocol,
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
            id,
            request.name,
            request.version,
            request.library_type,
            request.description,
            request.steps,
            request.parameters,
            request.kit_id,
            request.platform_compatibility,
            request.quality_thresholds,
            true,
            now,
            now
        )
        .fetch_one(&self.pool)
        .await?;

        tracing::info!("Created protocol: {} v{}", protocol.name, protocol.version);
        Ok(protocol)
    }

    pub async fn get_protocol(&self, id: Uuid) -> Result<Protocol> {
        let protocol = sqlx::query_as!(
            Protocol,
            r#"
            SELECT id, name, version, library_type, description, steps,
                   parameters, kit_id, platform_compatibility, quality_thresholds,
                   is_active, created_at, updated_at
            FROM protocols
            WHERE id = $1
            "#,
            id
        )
        .fetch_optional(&self.pool)
        .await?
        .ok_or(ServiceError::ProtocolNotFound { id })?;

        Ok(protocol)
    }

    pub async fn list_protocols(&self, library_type: Option<String>, is_active: Option<bool>) -> Result<Vec<Protocol>> {
        let protocols = sqlx::query_as!(
            Protocol,
            r#"
            SELECT id, name, version, library_type, description, steps,
                   parameters, kit_id, platform_compatibility, quality_thresholds,
                   is_active, created_at, updated_at
            FROM protocols
            WHERE ($1::text IS NULL OR library_type = $1)
              AND ($2::bool IS NULL OR is_active = $2)
            ORDER BY name, version DESC
            "#,
            library_type,
            is_active
        )
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

        let protocol = sqlx::query_as!(
            Protocol,
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
            id,
            request.name,
            request.version,
            request.library_type,
            request.description,
            request.steps,
            request.parameters,
            request.kit_id,
            request.platform_compatibility,
            request.quality_thresholds,
            now
        )
        .fetch_one(&self.pool)
        .await?;

        tracing::info!("Updated protocol: {} v{}", protocol.name, protocol.version);
        Ok(protocol)
    }

    pub async fn validate_protocol(&self, id: Uuid) -> Result<Vec<String>> {
        let protocol = self.get_protocol(id).await?;
        let mut errors = Vec::new();

        // Validate protocol steps
        if let Some(steps) = protocol.steps.as_array() {
            if steps.is_empty() {
                errors.push("Protocol must have at least one step".to_string());
            }

            for (idx, step) in steps.iter().enumerate() {
                if !step.is_object() {
                    errors.push(format!("Step {} must be an object", idx + 1));
                    continue;
                }

                let step_obj = step.as_object().unwrap();
                
                if !step_obj.contains_key("name") {
                    errors.push(format!("Step {} must have a name", idx + 1));
                }

                if !step_obj.contains_key("duration") {
                    errors.push(format!("Step {} must have a duration", idx + 1));
                }

                if !step_obj.contains_key("temperature") {
                    errors.push(format!("Step {} must have a temperature", idx + 1));
                }
            }
        } else {
            errors.push("Protocol steps must be an array".to_string());
        }

        // Validate quality thresholds
        if let Some(thresholds) = &protocol.quality_thresholds {
            if let Some(thresholds_obj) = thresholds.as_object() {
                for (metric, threshold) in thresholds_obj {
                    if !threshold.is_object() {
                        errors.push(format!("Quality threshold for {} must be an object", metric));
                        continue;
                    }

                    let threshold_obj = threshold.as_object().unwrap();
                    if !threshold_obj.contains_key("min") && !threshold_obj.contains_key("max") {
                        errors.push(format!("Quality threshold for {} must have min or max value", metric));
                    }
                }
            }
        }

        if !errors.is_empty() {
            return Err(ServiceError::ProtocolValidationFailed { errors });
        }

        Ok(errors)
    }

    pub async fn get_protocol_steps(&self, id: Uuid) -> Result<serde_json::Value> {
        let protocol = self.get_protocol(id).await?;
        Ok(protocol.steps)
    }

    pub async fn recommend_protocol(&self, library_type: String, kit_id: Option<Uuid>, platform_id: Option<Uuid>) -> Result<Vec<ProtocolRecommendation>> {
        let protocols = self.list_protocols(Some(library_type.clone()), Some(true)).await?;
        let mut recommendations = Vec::new();

        for protocol in protocols {
            let mut compatibility_score = 0.5; // Base score
            let mut reasons = Vec::new();

            // Library type match
            if protocol.library_type == library_type {
                compatibility_score += 0.3;
                reasons.push("Library type matches".to_string());
            }

            // Kit compatibility
            if let Some(kit_id) = kit_id {
                if protocol.kit_id == Some(kit_id) {
                    compatibility_score += 0.2;
                    reasons.push("Compatible with specified kit".to_string());
                }
            }

            // Platform compatibility
            if let Some(platform_id) = platform_id {
                if let Some(platform_compat) = &protocol.platform_compatibility {
                    if let Some(platforms) = platform_compat.as_array() {
                        let platform_id_str = platform_id.to_string();
                        if platforms.iter().any(|p| p.as_str() == Some(&platform_id_str)) {
                            compatibility_score += 0.2;
                            reasons.push("Compatible with specified platform".to_string());
                        }
                    }
                }
            }

            if compatibility_score >= 0.6 {
                recommendations.push(ProtocolRecommendation {
                    protocol_id: protocol.id,
                    protocol_name: format!("{} v{}", protocol.name, protocol.version),
                    compatibility_score,
                    reasons,
                });
            }
        }

        // Sort by compatibility score (highest first)
        recommendations.sort_by(|a, b| b.compatibility_score.partial_cmp(&a.compatibility_score).unwrap());

        Ok(recommendations)
    }

    pub async fn deactivate_protocol(&self, id: Uuid) -> Result<Protocol> {
        let now = Utc::now();

        let protocol = sqlx::query_as!(
            Protocol,
            r#"
            UPDATE protocols
            SET is_active = false,
                updated_at = $2
            WHERE id = $1
            RETURNING id, name, version, library_type, description, steps,
                      parameters, kit_id, platform_compatibility, quality_thresholds,
                      is_active, created_at, updated_at
            "#,
            id,
            now
        )
        .fetch_optional(&self.pool)
        .await?
        .ok_or(ServiceError::ProtocolNotFound { id })?;

        tracing::info!("Deactivated protocol: {}", protocol.id);
        Ok(protocol)
    }
}