// TracSeq 2.0 - CQRS Query Handler
// Read model queries optimized for performance

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::{PgPool, Row};
use std::sync::Arc;
use uuid::Uuid;

#[async_trait]
pub trait Query: Send + Sync {
    type Result: Send;
}

#[async_trait]
pub trait QueryHandler<Q: Query>: Send + Sync {
    async fn handle(&self, query: Q) -> Result<Q::Result, QueryError>;
}

#[derive(Debug)]
pub struct QueryBus {
    pool: Arc<PgPool>,
}

impl QueryBus {
    pub fn new(pool: Arc<PgPool>) -> Self {
        Self { pool }
    }
}

// Sample Queries
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GetSampleByIdQuery {
    pub sample_id: Uuid,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GetSamplesByPatientQuery {
    pub patient_id: Uuid,
    pub limit: Option<i32>,
    pub offset: Option<i32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchSamplesQuery {
    pub barcode_pattern: Option<String>,
    pub sample_type: Option<String>,
    pub status: Option<String>,
    pub from_date: Option<DateTime<Utc>>,
    pub to_date: Option<DateTime<Utc>>,
    pub limit: Option<i32>,
    pub offset: Option<i32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GetSampleHistoryQuery {
    pub sample_id: Uuid,
    pub include_events: bool,
}

// Storage Queries
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GetStorageCapacityQuery {
    pub location_id: Option<Uuid>,
    pub temperature_zone: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FindAvailableStorageQuery {
    pub sample_type: String,
    pub required_temperature: f32,
    pub volume_ml: f32,
}

// Sequencing Queries
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GetSequencingRunsQuery {
    pub machine_id: Option<String>,
    pub status: Option<String>,
    pub from_date: Option<DateTime<Utc>>,
    pub to_date: Option<DateTime<Utc>>,
    pub limit: Option<i32>,
    pub offset: Option<i32>,
}

// Read Models
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SampleReadModel {
    pub sample_id: Uuid,
    pub barcode: String,
    pub sample_type: String,
    pub patient_id: Option<Uuid>,
    pub volume_ml: f32,
    pub collection_date: DateTime<Utc>,
    pub status: String,
    pub location: Option<StorageLocation>,
    pub validation_status: Option<ValidationStatus>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StorageLocation {
    pub location_id: Uuid,
    pub name: String,
    pub position: String,
    pub temperature: f32,
    pub stored_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationStatus {
    pub is_valid: bool,
    pub validation_type: String,
    pub validated_at: DateTime<Utc>,
    pub validated_by: Uuid,
    pub results: serde_json::Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StorageCapacityReadModel {
    pub location_id: Uuid,
    pub location_name: String,
    pub temperature_zone: String,
    pub total_capacity: i32,
    pub used_capacity: i32,
    pub available_capacity: i32,
    pub capacity_percentage: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SequencingRunReadModel {
    pub sequencing_id: Uuid,
    pub sample_id: Uuid,
    pub sample_barcode: String,
    pub protocol: String,
    pub machine_id: String,
    pub operator_id: Uuid,
    pub status: String,
    pub started_at: DateTime<Utc>,
    pub completed_at: Option<DateTime<Utc>>,
    pub quality_score: Option<f32>,
    pub read_count: Option<i64>,
    pub results_url: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SampleHistoryReadModel {
    pub sample: SampleReadModel,
    pub events: Vec<SampleEvent>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SampleEvent {
    pub event_id: Uuid,
    pub event_type: String,
    pub event_data: serde_json::Value,
    pub created_at: DateTime<Utc>,
    pub created_by: Option<Uuid>,
}

// Query Handlers
pub struct SampleQueryHandler {
    pool: Arc<PgPool>,
}

impl SampleQueryHandler {
    pub fn new(pool: Arc<PgPool>) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl QueryHandler<GetSampleByIdQuery> for SampleQueryHandler {
    async fn handle(
        &self,
        query: GetSampleByIdQuery,
    ) -> Result<SampleReadModel, QueryError> {
        let row = sqlx::query(
            r#"
            SELECT 
                s.*,
                l.location_id, l.name as location_name, l.position, 
                l.temperature, l.stored_at,
                v.is_valid, v.validation_type, v.validated_at, 
                v.validated_by, v.results as validation_results
            FROM samples_read_model s
            LEFT JOIN storage_locations_read_model l ON s.sample_id = l.sample_id
            LEFT JOIN validations_read_model v ON s.sample_id = v.sample_id
            WHERE s.sample_id = $1
            "#,
        )
        .bind(query.sample_id)
        .fetch_optional(&*self.pool)
        .await?
        .ok_or(QueryError::NotFound(format!(
            "Sample {} not found",
            query.sample_id
        )))?;

        Ok(self.map_to_sample_read_model(row)?)
    }
}

#[async_trait]
impl QueryHandler<SearchSamplesQuery> for SampleQueryHandler {
    async fn handle(
        &self,
        query: SearchSamplesQuery,
    ) -> Result<Vec<SampleReadModel>, QueryError> {
        let limit = query.limit.unwrap_or(100).min(1000);
        let offset = query.offset.unwrap_or(0);

        let mut sql = String::from(
            r#"
            SELECT 
                s.*,
                l.location_id, l.name as location_name, l.position, 
                l.temperature, l.stored_at,
                v.is_valid, v.validation_type, v.validated_at, 
                v.validated_by, v.results as validation_results
            FROM samples_read_model s
            LEFT JOIN storage_locations_read_model l ON s.sample_id = l.sample_id
            LEFT JOIN validations_read_model v ON s.sample_id = v.sample_id
            WHERE 1=1
            "#,
        );

        let mut binds = vec![];
        let mut bind_count = 1;

        if let Some(pattern) = &query.barcode_pattern {
            sql.push_str(&format!(" AND s.barcode LIKE ${}", bind_count));
            binds.push(format!("%{}%", pattern));
            bind_count += 1;
        }

        if let Some(sample_type) = &query.sample_type {
            sql.push_str(&format!(" AND s.sample_type = ${}", bind_count));
            binds.push(sample_type.clone());
            bind_count += 1;
        }

        if let Some(status) = &query.status {
            sql.push_str(&format!(" AND s.status = ${}", bind_count));
            binds.push(status.clone());
            bind_count += 1;
        }

        sql.push_str(&format!(
            " ORDER BY s.created_at DESC LIMIT ${} OFFSET ${}",
            bind_count,
            bind_count + 1
        ));

        let mut query_builder = sqlx::query(&sql);
        for bind in binds {
            query_builder = query_builder.bind(bind);
        }
        query_builder = query_builder.bind(limit).bind(offset);

        let rows = query_builder.fetch_all(&*self.pool).await?;
        
        rows.into_iter()
            .map(|row| self.map_to_sample_read_model(row))
            .collect::<Result<Vec<_>, _>>()
    }
}

impl SampleQueryHandler {
    fn map_to_sample_read_model(
        &self,
        row: sqlx::postgres::PgRow,
    ) -> Result<SampleReadModel, QueryError> {
        let location = if let Some(location_id) = row.try_get::<Option<Uuid>, _>("location_id")? {
            Some(StorageLocation {
                location_id,
                name: row.get("location_name"),
                position: row.get("position"),
                temperature: row.get("temperature"),
                stored_at: row.get("stored_at"),
            })
        } else {
            None
        };

        let validation_status = if let Some(is_valid) = row.try_get::<Option<bool>, _>("is_valid")? {
            Some(ValidationStatus {
                is_valid,
                validation_type: row.get("validation_type"),
                validated_at: row.get("validated_at"),
                validated_by: row.get("validated_by"),
                results: row.get("validation_results"),
            })
        } else {
            None
        };

        Ok(SampleReadModel {
            sample_id: row.get("sample_id"),
            barcode: row.get("barcode"),
            sample_type: row.get("sample_type"),
            patient_id: row.try_get("patient_id")?,
            volume_ml: row.get("volume_ml"),
            collection_date: row.get("collection_date"),
            status: row.get("status"),
            location,
            validation_status,
            created_at: row.get("created_at"),
            updated_at: row.get("updated_at"),
        })
    }
}

// Storage Query Handler
pub struct StorageQueryHandler {
    pool: Arc<PgPool>,
}

impl StorageQueryHandler {
    pub fn new(pool: Arc<PgPool>) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl QueryHandler<GetStorageCapacityQuery> for StorageQueryHandler {
    async fn handle(
        &self,
        query: GetStorageCapacityQuery,
    ) -> Result<Vec<StorageCapacityReadModel>, QueryError> {
        let mut sql = String::from(
            r#"
            SELECT 
                location_id,
                location_name,
                temperature_zone,
                total_capacity,
                used_capacity,
                (total_capacity - used_capacity) as available_capacity,
                (used_capacity::float / total_capacity::float * 100) as capacity_percentage
            FROM storage_capacity_read_model
            WHERE 1=1
            "#,
        );

        if query.location_id.is_some() {
            sql.push_str(" AND location_id = $1");
        }

        if query.temperature_zone.is_some() {
            sql.push_str(" AND temperature_zone = $2");
        }

        sql.push_str(" ORDER BY location_name");

        let rows = if let (Some(location_id), Some(temp_zone)) = 
            (query.location_id, query.temperature_zone.as_ref()) {
            sqlx::query(&sql)
                .bind(location_id)
                .bind(temp_zone)
                .fetch_all(&*self.pool)
                .await?
        } else if let Some(location_id) = query.location_id {
            sqlx::query(&sql)
                .bind(location_id)
                .fetch_all(&*self.pool)
                .await?
        } else if let Some(temp_zone) = query.temperature_zone.as_ref() {
            sqlx::query(&sql)
                .bind(temp_zone)
                .fetch_all(&*self.pool)
                .await?
        } else {
            sqlx::query(&sql)
                .fetch_all(&*self.pool)
                .await?
        };

        rows.into_iter()
            .map(|row| {
                Ok(StorageCapacityReadModel {
                    location_id: row.get("location_id"),
                    location_name: row.get("location_name"),
                    temperature_zone: row.get("temperature_zone"),
                    total_capacity: row.get("total_capacity"),
                    used_capacity: row.get("used_capacity"),
                    available_capacity: row.get("available_capacity"),
                    capacity_percentage: row.get("capacity_percentage"),
                })
            })
            .collect::<Result<Vec<_>, QueryError>>()
    }
}

// Query Error Types
#[derive(Debug, thiserror::Error)]
pub enum QueryError {
    #[error("Not found: {0}")]
    NotFound(String),
    #[error("Database error: {0}")]
    Database(#[from] sqlx::Error),
    #[error("Invalid query parameters: {0}")]
    InvalidParameters(String),
}

// Query Result Pagination
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PaginatedResult<T> {
    pub items: Vec<T>,
    pub total_count: i64,
    pub page: i32,
    pub page_size: i32,
    pub has_next: bool,
    pub has_previous: bool,
}