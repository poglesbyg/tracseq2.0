use crate::error::{Result, ServiceError};
use crate::models::*;
use sqlx::PgPool;
use uuid::Uuid;
use chrono::Utc;

use validator::Validate;

#[derive(Clone)]
pub struct LibraryService {
    pool: PgPool,
}

impl LibraryService {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn create_library(&self, request: CreateLibraryRequest) -> Result<Library> {
        request.validate().map_err(|e| ServiceError::Validation {
            message: e.to_string(),
        })?;

        let id = Uuid::new_v4();
        let now = Utc::now();

        let library = sqlx::query_as::<_, Library>(
            r#"
            INSERT INTO libraries (
                id, name, sample_id, library_type, concentration, volume,
                fragment_size_min, fragment_size_max, preparation_protocol_id,
                preparation_date, barcode, adapter_sequence, metadata,
                status, created_at, updated_at
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15, $16)
            RETURNING id, name, sample_id, library_type, concentration, volume,
                      fragment_size_min, fragment_size_max, preparation_protocol_id,
                      preparation_date, barcode, adapter_sequence, 
                      quality_score, status, metadata, created_at, updated_at
            "#,
        )
        .bind(id)
        .bind(&request.name)
        .bind(request.sample_id)
        .bind(&request.library_type)
        .bind(request.concentration)
        .bind(request.volume)
        .bind(request.fragment_size_min)
        .bind(request.fragment_size_max)
        .bind(request.preparation_protocol_id)
        .bind(request.preparation_date)
        .bind(&request.barcode)
        .bind(&request.adapter_sequence)
        .bind(&request.metadata)
        .bind("pending")
        .bind(now)
        .bind(now)
        .fetch_one(&self.pool)
        .await?;

        tracing::info!("Created library: {}", library.id);
        Ok(library)
    }

    pub async fn get_library(&self, id: Uuid) -> Result<Library> {
        let library = sqlx::query_as::<_, Library>(
            r#"
            SELECT id, name, sample_id, library_type, concentration, volume,
                   fragment_size_min, fragment_size_max, preparation_protocol_id,
                   preparation_date, barcode, adapter_sequence, quality_score,
                   status, metadata, created_at, updated_at
            FROM libraries
            WHERE id = $1
            "#,
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await?
        .ok_or(ServiceError::LibraryNotFound { id })?;

        Ok(library)
    }

    pub async fn list_libraries(&self, sample_id: Option<Uuid>, status: Option<LibraryStatus>) -> Result<Vec<Library>> {
        let libraries = if let Some(sample_id) = sample_id {
            sqlx::query_as::<_, Library>(
                r#"
                SELECT id, name, sample_id, library_type, concentration, volume,
                       fragment_size_min, fragment_size_max, preparation_protocol_id,
                       preparation_date, barcode, adapter_sequence, quality_score,
                       status, metadata, created_at, updated_at
                FROM libraries
                WHERE sample_id = $1 AND ($2::text IS NULL OR status = $2)
                ORDER BY created_at DESC
                "#,
            )
            .bind(sample_id)
            .bind(status.as_ref().map(|s| format!("{:?}", s).to_lowercase()))
            .fetch_all(&self.pool)
            .await?
        } else {
            sqlx::query_as::<_, Library>(
                r#"
                SELECT id, name, sample_id, library_type, concentration, volume,
                       fragment_size_min, fragment_size_max, preparation_protocol_id,
                       preparation_date, barcode, adapter_sequence, quality_score,
                       status, metadata, created_at, updated_at
                FROM libraries
                WHERE ($1::text IS NULL OR status = $1)
                ORDER BY created_at DESC
                "#,
            )
            .bind(status.as_ref().map(|s| format!("{:?}", s).to_lowercase()))
            .fetch_all(&self.pool)
            .await?
        };

        Ok(libraries)
    }

    pub async fn update_library(&self, id: Uuid, request: UpdateLibraryRequest) -> Result<Library> {
        request.validate().map_err(|e| ServiceError::Validation {
            message: e.to_string(),
        })?;

        // Check if library exists
        self.get_library(id).await?;

        let now = Utc::now();

        let library = sqlx::query_as::<_, Library>(
            r#"
            UPDATE libraries
            SET name = COALESCE($2, name),
                library_type = COALESCE($3, library_type),
                concentration = COALESCE($4, concentration),
                volume = COALESCE($5, volume),
                fragment_size_min = COALESCE($6, fragment_size_min),
                fragment_size_max = COALESCE($7, fragment_size_max),
                preparation_protocol_id = COALESCE($8, preparation_protocol_id),
                preparation_date = COALESCE($9, preparation_date),
                barcode = COALESCE($10, barcode),
                adapter_sequence = COALESCE($11, adapter_sequence),
                status = COALESCE($12, status),
                metadata = COALESCE($13, metadata),
                updated_at = $14
            WHERE id = $1
            RETURNING id, name, sample_id, library_type, concentration, volume,
                      fragment_size_min, fragment_size_max, preparation_protocol_id,
                      preparation_date, barcode, adapter_sequence, quality_score,
                      status, metadata, created_at, updated_at
            "#,
        )
        .bind(id)
        .bind(&request.name)
        .bind(&request.library_type)
        .bind(request.concentration)
        .bind(request.volume)
        .bind(request.fragment_size_min)
        .bind(request.fragment_size_max)
        .bind(request.preparation_protocol_id)
        .bind(request.preparation_date)
        .bind(&request.barcode)
        .bind(&request.adapter_sequence)
        .bind(request.status.as_ref().map(|s| format!("{:?}", s).to_lowercase()))
        .bind(&request.metadata)
        .bind(now)
        .fetch_one(&self.pool)
        .await?;

        tracing::info!("Updated library: {}", library.id);
        Ok(library)
    }

    pub async fn delete_library(&self, id: Uuid) -> Result<()> {
        let result = sqlx::query("DELETE FROM libraries WHERE id = $1")
            .bind(id)
            .execute(&self.pool)
            .await?;

        if result.rows_affected() == 0 {
            return Err(ServiceError::LibraryNotFound { id });
        }

        tracing::info!("Deleted library: {}", id);
        Ok(())
    }

    pub async fn calculate_library_metrics(&self, id: Uuid) -> Result<LibraryMetrics> {
        let library = self.get_library(id).await?;

        // Placeholder for actual metrics calculation
        let metrics = LibraryMetrics {
            concentration_mean: library.concentration,
            concentration_std: None,
            fragment_size_distribution: None,
            quality_score_distribution: None,
            success_rate: library.quality_score,
            throughput: None,
        };

        Ok(metrics)
    }

    pub async fn normalize_library(&self, id: Uuid, target_concentration: f64) -> Result<Library> {
        let mut library = self.get_library(id).await?;

        if let Some(current_concentration) = library.concentration {
            if current_concentration < target_concentration {
                return Err(ServiceError::InsufficientConcentration {
                    current: current_concentration,
                    required: target_concentration,
                });
            }

            // Calculate dilution factor
            let dilution_factor = current_concentration / target_concentration;
            
            // Update library with normalized values
            if let Some(current_volume) = library.volume {
                library.volume = Some(current_volume * dilution_factor);
            }
            library.concentration = Some(target_concentration);

            let update_request = UpdateLibraryRequest {
                concentration: Some(target_concentration),
                volume: library.volume,
                status: Some(LibraryStatus::QualityControl),
                ..Default::default()
            };

            self.update_library(id, update_request).await
        } else {
            Err(ServiceError::Validation {
                message: "Library concentration is required for normalization".to_string(),
            })
        }
    }

    pub async fn create_batch_libraries(&self, request: BatchLibraryRequest) -> Result<Vec<Library>> {
        let mut libraries = Vec::new();

        for lib_request in request.libraries {
            let library = self.create_library(lib_request).await?;
            libraries.push(library);
        }

        Ok(libraries)
    }

    pub async fn get_libraries_for_sample(&self, sample_id: Uuid) -> Result<Vec<Library>> {
        self.list_libraries(Some(sample_id), None).await
    }

    pub async fn get_libraries_for_sequencing_job(&self, _job_id: Uuid) -> Result<Vec<Library>> {
        // This would need additional logic to link libraries to sequencing jobs
        // For now, return empty list
        Ok(Vec::new())
    }
}

impl Default for UpdateLibraryRequest {
    fn default() -> Self {
        Self {
            name: None,
            library_type: None,
            concentration: None,
            volume: None,
            fragment_size_min: None,
            fragment_size_max: None,
            preparation_protocol_id: None,
            preparation_date: None,
            barcode: None,
            adapter_sequence: None,
            status: None,
            metadata: None,
        }
    }
}