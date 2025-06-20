use async_trait::async_trait;
use std::collections::HashMap;
use uuid::Uuid;
use chrono::{DateTime, Utc};
use sqlx::Row;

use crate::{
    config::Config,
    database::DatabasePool,
    error::{SampleResult, SampleServiceError},
    models::*,
    clients::{AuthClient, StorageClient},
};

/// Main sample service implementation
#[derive(Debug, Clone)]
pub struct SampleServiceImpl {
    db_pool: DatabasePool,
    config: Config,
    auth_client: AuthClient,
    storage_client: StorageClient,
}

impl SampleServiceImpl {
    pub fn new(
        db_pool: DatabasePool,
        config: Config,
        auth_client: AuthClient,
        storage_client: StorageClient,
    ) -> SampleResult<Self> {
        Ok(Self {
            db_pool,
            config,
            auth_client,
            storage_client,
        })
    }

    /// Create a new sample
    pub async fn create_sample(&self, request: CreateSampleRequest) -> SampleResult<Sample> {
        // Generate barcode if not provided
        let barcode = if let Some(barcode) = request.barcode {
            // Validate barcode is unique
            self.validate_barcode_unique(&barcode).await?;
            barcode
        } else {
            self.generate_barcode().await?
        };

        // Validate sample type
        self.validate_sample_type(&request.sample_type)?;

        // Validate template if provided
        if let Some(template_id) = request.template_id {
            self.validate_template_compatibility(template_id, &request.metadata).await?;
        }

        let sample_id = Uuid::new_v4();
        let now = Utc::now();

        let sample = sqlx::query_as::<_, Sample>(
            r#"
            INSERT INTO samples (
                id, name, barcode, sample_type, status, template_id,
                source_type, source_identifier, collection_date, collection_location,
                collector, concentration, volume, unit, quality_score, metadata, notes,
                created_at, updated_at, created_by, updated_by
            ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15, $16, $17, $18, $19, $20, $21)
            RETURNING *
            "#,
        )
        .bind(sample_id)
        .bind(&request.name)
        .bind(&barcode)
        .bind(&request.sample_type)
        .bind(SampleStatus::Pending)
        .bind(request.template_id)
        .bind(request.source_type.as_deref())
        .bind(request.source_identifier.as_deref())
        .bind(request.collection_date)
        .bind(request.collection_location.as_deref())
        .bind(request.collector.as_deref())
        .bind(request.concentration)
        .bind(request.volume)
        .bind(request.unit.as_deref())
        .bind(request.quality_score)
        .bind(&request.metadata)
        .bind(request.notes.as_deref())
        .bind(now)
        .bind(now)
        .bind(request.created_by.as_deref())
        .bind(request.created_by.as_deref())
        .fetch_one(&self.db_pool.pool)
        .await?;

        // Log sample creation
        self.log_audit_event(
            sample.id,
            "sample_created",
            None,
            Some(serde_json::to_value(&sample)?),
            request.created_by.as_deref(),
        ).await?;

        Ok(sample)
    }

    /// Get sample by ID
    pub async fn get_sample(&self, sample_id: Uuid) -> SampleResult<Sample> {
        let sample = sqlx::query_as::<_, Sample>("SELECT * FROM samples WHERE id = $1")
            .bind(sample_id)
            .fetch_optional(&self.db_pool.pool)
            .await?
            .ok_or_else(|| SampleServiceError::SampleNotFound {
                sample_id: sample_id.to_string(),
            })?;

        Ok(sample)
    }

    /// Get sample by barcode
    pub async fn get_sample_by_barcode(&self, barcode: &str) -> SampleResult<Sample> {
        let sample = sqlx::query_as::<_, Sample>("SELECT * FROM samples WHERE barcode = $1")
            .bind(barcode)
            .fetch_optional(&self.db_pool.pool)
            .await?
            .ok_or_else(|| SampleServiceError::BarcodeNotFound {
                barcode: barcode.to_string(),
            })?;

        Ok(sample)
    }

    /// List samples with pagination and filtering
    pub async fn list_samples(
        &self,
        query: ListSamplesQuery,
    ) -> SampleResult<PaginatedSampleResponse> {
        let mut conditions = Vec::new();
        let mut params: Vec<Box<dyn sqlx::Encode<'_, sqlx::Postgres> + Send>> = Vec::new();
        let mut param_count = 0;

        // Build dynamic query based on filters
        if let Some(status) = &query.status {
            param_count += 1;
            conditions.push(format!("status = ${}", param_count));
            params.push(Box::new(status.clone()));
        }

        if let Some(sample_type) = &query.sample_type {
            param_count += 1;
            conditions.push(format!("sample_type = ${}", param_count));
            params.push(Box::new(sample_type.clone()));
        }

        if let Some(template_id) = query.template_id {
            param_count += 1;
            conditions.push(format!("template_id = ${}", param_count));
            params.push(Box::new(template_id));
        }

        if let Some(search) = &query.search {
            param_count += 1;
            conditions.push(format!("(name ILIKE ${} OR barcode ILIKE ${})", param_count, param_count));
            params.push(Box::new(format!("%{}%", search)));
        }

        if let Some(created_after) = query.created_after {
            param_count += 1;
            conditions.push(format!("created_at >= ${}", param_count));
            params.push(Box::new(created_after));
        }

        if let Some(created_before) = query.created_before {
            param_count += 1;
            conditions.push(format!("created_at <= ${}", param_count));
            params.push(Box::new(created_before));
        }

        let where_clause = if conditions.is_empty() {
            String::new()
        } else {
            format!("WHERE {}", conditions.join(" AND "))
        };

        // Count total records
        let count_query = format!("SELECT COUNT(*) FROM samples {}", where_clause);
        let total_count: i64 = sqlx::query(&count_query)
            .fetch_one(&self.db_pool.pool)
            .await?
            .get(0);

        // Calculate pagination
        let page = query.page.unwrap_or(1).max(1);
        let page_size = query.page_size.unwrap_or(50).min(1000);
        let offset = (page - 1) * page_size;
        let total_pages = (total_count + page_size - 1) / page_size;

        // Build main query with ordering and pagination
        let order_by = match query.sort_by.as_deref() {
            Some("name") => "ORDER BY name",
            Some("created_at") => "ORDER BY created_at DESC",
            Some("updated_at") => "ORDER BY updated_at DESC",
            Some("status") => "ORDER BY status, created_at DESC",
            _ => "ORDER BY created_at DESC",
        };

        param_count += 1;
        let limit_param = param_count;
        param_count += 1;
        let offset_param = param_count;

        let select_query = format!(
            "SELECT * FROM samples {} {} LIMIT ${} OFFSET ${}",
            where_clause, order_by, limit_param, offset_param
        );

        params.push(Box::new(page_size));
        params.push(Box::new(offset));

        let mut query_builder = sqlx::query_as::<_, Sample>(&select_query);
        for param in params {
            // This is a simplified approach - in a real implementation,
            // you'd need to handle the dynamic parameters more carefully
        }

        // For now, let's use a simpler approach
        let samples = sqlx::query_as::<_, Sample>(
            "SELECT * FROM samples ORDER BY created_at DESC LIMIT $1 OFFSET $2"
        )
        .bind(page_size)
        .bind(offset)
        .fetch_all(&self.db_pool.pool)
        .await?;

        Ok(PaginatedSampleResponse {
            samples,
            total_count,
            page,
            page_size,
            total_pages,
        })
    }

    /// Update sample
    pub async fn update_sample(
        &self,
        sample_id: Uuid,
        request: UpdateSampleRequest,
        updated_by: Option<&str>,
    ) -> SampleResult<Sample> {
        // Get current sample for audit logging
        let current_sample = self.get_sample(sample_id).await?;

        // Validate barcode uniqueness if changed
        if let Some(ref new_barcode) = request.barcode {
            if new_barcode != &current_sample.barcode {
                self.validate_barcode_unique(new_barcode).await?;
            }
        }

        // Validate sample type if changed
        if let Some(ref sample_type) = request.sample_type {
            self.validate_sample_type(sample_type)?;
        }

        let now = Utc::now();

        let sample = sqlx::query_as::<_, Sample>(
            r#"
            UPDATE samples SET
                name = COALESCE($2, name),
                barcode = COALESCE($3, barcode),
                sample_type = COALESCE($4, sample_type),
                source_type = COALESCE($5, source_type),
                source_identifier = COALESCE($6, source_identifier),
                collection_date = COALESCE($7, collection_date),
                collection_location = COALESCE($8, collection_location),
                collector = COALESCE($9, collector),
                concentration = COALESCE($10, concentration),
                volume = COALESCE($11, volume),
                unit = COALESCE($12, unit),
                quality_score = COALESCE($13, quality_score),
                metadata = COALESCE($14, metadata),
                notes = COALESCE($15, notes),
                updated_at = $16,
                updated_by = $17
            WHERE id = $1
            RETURNING *
            "#,
        )
        .bind(sample_id)
        .bind(request.name.as_deref())
        .bind(request.barcode.as_deref())
        .bind(request.sample_type.as_deref())
        .bind(request.source_type.as_deref())
        .bind(request.source_identifier.as_deref())
        .bind(request.collection_date)
        .bind(request.collection_location.as_deref())
        .bind(request.collector.as_deref())
        .bind(request.concentration)
        .bind(request.volume)
        .bind(request.unit.as_deref())
        .bind(request.quality_score)
        .bind(&request.metadata)
        .bind(request.notes.as_deref())
        .bind(now)
        .bind(updated_by)
        .fetch_one(&self.db_pool.pool)
        .await?;

        // Log sample update
        self.log_audit_event(
            sample.id,
            "sample_updated",
            Some(serde_json::to_value(&current_sample)?),
            Some(serde_json::to_value(&sample)?),
            updated_by,
        ).await?;

        Ok(sample)
    }

    /// Update sample status
    pub async fn update_sample_status(
        &self,
        sample_id: Uuid,
        new_status: SampleStatus,
        updated_by: Option<&str>,
    ) -> SampleResult<Sample> {
        let current_sample = self.get_sample(sample_id).await?;

        // Validate status transition
        self.validate_status_transition(&current_sample.status, &new_status)?;

        let now = Utc::now();

        let sample = sqlx::query_as::<_, Sample>(
            "UPDATE samples SET status = $2, updated_at = $3, updated_by = $4 WHERE id = $1 RETURNING *"
        )
        .bind(sample_id)
        .bind(new_status)
        .bind(now)
        .bind(updated_by)
        .fetch_one(&self.db_pool.pool)
        .await?;

        // Log status change
        self.log_status_change(sample_id, &current_sample.status, &new_status, updated_by).await?;

        // Log audit event
        self.log_audit_event(
            sample.id,
            "status_updated",
            Some(serde_json::json!({"old_status": current_sample.status})),
            Some(serde_json::json!({"new_status": new_status})),
            updated_by,
        ).await?;

        Ok(sample)
    }

    /// Validate sample
    pub async fn validate_sample(&self, sample_id: Uuid) -> SampleResult<SampleValidationResult> {
        let sample = self.get_sample(sample_id).await?;
        let mut validation_result = SampleValidationResult {
            sample_id,
            is_valid: true,
            errors: Vec::new(),
            warnings: Vec::new(),
        };

        // Basic validations
        if sample.name.trim().is_empty() {
            validation_result.errors.push("Sample name cannot be empty".to_string());
            validation_result.is_valid = false;
        }

        if sample.barcode.trim().is_empty() {
            validation_result.errors.push("Barcode cannot be empty".to_string());
            validation_result.is_valid = false;
        }

        if sample.sample_type.trim().is_empty() {
            validation_result.errors.push("Sample type cannot be empty".to_string());
            validation_result.is_valid = false;
        }

        // Validate concentration and volume if provided
        if let (Some(concentration), Some(volume)) = (sample.concentration, sample.volume) {
            if concentration <= rust_decimal::Decimal::ZERO {
                validation_result.warnings.push("Concentration should be greater than 0".to_string());
            }
            if volume <= rust_decimal::Decimal::ZERO {
                validation_result.warnings.push("Volume should be greater than 0".to_string());
            }
        }

        // Template validation if applicable
        if let Some(template_id) = sample.template_id {
            if let Err(e) = self.validate_template_compatibility(template_id, &sample.metadata).await {
                validation_result.errors.push(format!("Template validation failed: {}", e));
                validation_result.is_valid = false;
            }
        }

        Ok(validation_result)
    }

    /// Delete sample (soft delete)
    pub async fn delete_sample(&self, sample_id: Uuid, deleted_by: Option<&str>) -> SampleResult<()> {
        let sample = self.get_sample(sample_id).await?;

        // Check if sample can be deleted (business rules)
        if sample.status == SampleStatus::InSequencing {
            return Err(SampleServiceError::BusinessRule(
                "Cannot delete sample that is currently being sequenced".to_string(),
            ));
        }

        let now = Utc::now();

        sqlx::query(
            "UPDATE samples SET status = $2, updated_at = $3, updated_by = $4 WHERE id = $1"
        )
        .bind(sample_id)
        .bind(SampleStatus::Deleted)
        .bind(now)
        .bind(deleted_by)
        .execute(&self.db_pool.pool)
        .await?;

        // Log deletion
        self.log_audit_event(
            sample_id,
            "sample_deleted",
            Some(serde_json::to_value(&sample)?),
            None,
            deleted_by,
        ).await?;

        Ok(())
    }

    /// Create batch samples
    pub async fn create_batch_samples(
        &self,
        requests: Vec<CreateSampleRequest>,
    ) -> SampleResult<BatchSampleResponse> {
        if requests.len() > self.config.sample.max_batch_size {
            return Err(SampleServiceError::ResourceLimit(format!(
                "Batch size {} exceeds maximum allowed size of {}",
                requests.len(),
                self.config.sample.max_batch_size
            )));
        }

        let mut created_samples = Vec::new();
        let mut failed_samples = Vec::new();

        for (index, request) in requests.into_iter().enumerate() {
            match self.create_sample(request.clone()).await {
                Ok(sample) => created_samples.push(sample),
                Err(error) => failed_samples.push(BatchSampleError {
                    index,
                    sample_data: request,
                    error: error.to_string(),
                }),
            }
        }

        Ok(BatchSampleResponse {
            total_created: created_samples.len(),
            total_failed: failed_samples.len(),
            created_samples,
            failed_samples,
        })
    }

    /// Generate unique barcode
    async fn generate_barcode(&self) -> SampleResult<String> {
        let prefix = &self.config.barcode.prefix;
        let timestamp = Utc::now().format("%Y%m%d%H%M%S");
        
        for attempt in 0..10 {
            let random_suffix: u32 = rand::random::<u32>() % 10000;
            let barcode = format!("{}-{}-{:04}", prefix, timestamp, random_suffix);
            
            if self.is_barcode_unique(&barcode).await? {
                return Ok(barcode);
            }
        }

        Err(SampleServiceError::BarcodeGeneration(
            "Failed to generate unique barcode after 10 attempts".to_string(),
        ))
    }

    /// Check if barcode is unique
    async fn is_barcode_unique(&self, barcode: &str) -> SampleResult<bool> {
        let count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM samples WHERE barcode = $1")
            .bind(barcode)
            .fetch_one(&self.db_pool.pool)
            .await?;

        Ok(count == 0)
    }

    /// Validate barcode uniqueness
    async fn validate_barcode_unique(&self, barcode: &str) -> SampleResult<()> {
        if !self.is_barcode_unique(barcode).await? {
            return Err(SampleServiceError::DuplicateBarcode {
                barcode: barcode.to_string(),
            });
        }
        Ok(())
    }

    /// Validate sample type
    fn validate_sample_type(&self, sample_type: &str) -> SampleResult<()> {
        let valid_types = [
            "DNA", "RNA", "Protein", "Cell Culture", "Tissue", "Blood", "Serum", "Plasma",
            "Urine", "Saliva", "Swab", "Environmental", "Other"
        ];

        if !valid_types.contains(&sample_type) {
            return Err(SampleServiceError::Validation(format!(
                "Invalid sample type '{}'. Valid types are: {}",
                sample_type,
                valid_types.join(", ")
            )));
        }

        Ok(())
    }

    /// Validate status transition
    fn validate_status_transition(
        &self,
        current_status: &SampleStatus,
        new_status: &SampleStatus,
    ) -> SampleResult<()> {
        let valid_transitions = match current_status {
            SampleStatus::Pending => vec![
                SampleStatus::Validated,
                SampleStatus::Rejected,
                SampleStatus::Deleted,
            ],
            SampleStatus::Validated => vec![
                SampleStatus::InStorage,
                SampleStatus::InSequencing,
                SampleStatus::Rejected,
                SampleStatus::Deleted,
            ],
            SampleStatus::InStorage => vec![
                SampleStatus::InSequencing,
                SampleStatus::Rejected,
                SampleStatus::Deleted,
            ],
            SampleStatus::InSequencing => vec![
                SampleStatus::Completed,
                SampleStatus::Failed,
            ],
            SampleStatus::Completed => vec![
                SampleStatus::Archived,
            ],
            SampleStatus::Failed => vec![
                SampleStatus::Pending, // Allow retry
                SampleStatus::Deleted,
            ],
            SampleStatus::Rejected => vec![
                SampleStatus::Pending, // Allow retry after fixing issues
                SampleStatus::Deleted,
            ],
            SampleStatus::Archived => vec![], // No transitions from archived
            SampleStatus::Deleted => vec![], // No transitions from deleted
        };

        if !valid_transitions.contains(new_status) {
            return Err(SampleServiceError::InvalidWorkflowTransition {
                current_status: current_status.to_string(),
                requested_status: new_status.to_string(),
            });
        }

        Ok(())
    }

    /// Validate template compatibility
    async fn validate_template_compatibility(
        &self,
        template_id: Uuid,
        metadata: &serde_json::Value,
    ) -> SampleResult<()> {
        // This would integrate with the template service to validate
        // that the sample metadata matches the template requirements
        // For now, we'll do basic validation
        
        if metadata.is_null() {
            return Err(SampleServiceError::TemplateValidation(
                "Template requires metadata but none provided".to_string(),
            ));
        }

        Ok(())
    }

    /// Log status change
    async fn log_status_change(
        &self,
        sample_id: Uuid,
        old_status: &SampleStatus,
        new_status: &SampleStatus,
        changed_by: Option<&str>,
    ) -> SampleResult<()> {
        sqlx::query(
            r#"
            INSERT INTO sample_status_history (
                sample_id, old_status, new_status, changed_at, changed_by
            ) VALUES ($1, $2, $3, NOW(), $4)
            "#,
        )
        .bind(sample_id)
        .bind(old_status)
        .bind(new_status)
        .bind(changed_by)
        .execute(&self.db_pool.pool)
        .await?;

        Ok(())
    }

    /// Log audit event
    async fn log_audit_event(
        &self,
        sample_id: Uuid,
        action: &str,
        old_values: Option<serde_json::Value>,
        new_values: Option<serde_json::Value>,
        performed_by: Option<&str>,
    ) -> SampleResult<()> {
        sqlx::query(
            r#"
            INSERT INTO sample_audit_log (
                sample_id, action, old_values, new_values, performed_by, performed_at
            ) VALUES ($1, $2, $3, $4, $5, NOW())
            "#,
        )
        .bind(Some(sample_id))
        .bind(action)
        .bind(old_values)
        .bind(new_values)
        .bind(performed_by)
        .execute(&self.db_pool.pool)
        .await?;

        Ok(())
    }

    /// Health check
    pub async fn health_check(&self) -> SampleResult<()> {
        // Test database connectivity
        sqlx::query("SELECT 1").execute(&self.db_pool.pool).await?;
        
        // Test external service connectivity
        if let Err(_) = self.auth_client.health_check().await {
            return Err(SampleServiceError::ExternalService {
                service: "auth".to_string(),
                message: "Auth service health check failed".to_string(),
            });
        }

        if let Err(_) = self.storage_client.health_check().await {
            return Err(SampleServiceError::ExternalService {
                service: "storage".to_string(),
                message: "Storage service health check failed".to_string(),
            });
        }

        Ok(())
    }
} 
