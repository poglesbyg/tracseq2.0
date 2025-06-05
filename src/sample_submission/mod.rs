use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize, sqlx::FromRow)]
pub struct Sample {
    pub id: Uuid,
    pub name: String,
    pub barcode: String,
    pub location: String,
    pub status: SampleStatus,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub metadata: serde_json::Value,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::Type)]
#[sqlx(type_name = "sample_status", rename_all = "snake_case")]
pub enum SampleStatus {
    Pending,
    Validated,
    InStorage,
    InSequencing,
    Completed,
}

#[derive(Debug, Deserialize, Clone)]
pub struct CreateSample {
    pub name: String,
    pub barcode: String,
    pub location: String,
    pub metadata: Option<serde_json::Value>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct UpdateSample {
    pub name: Option<String>,
    pub barcode: Option<String>,
    pub location: Option<String>,
    pub status: Option<SampleStatus>,
    pub metadata: Option<serde_json::Value>,
}

pub struct SampleSubmissionManager {
    pool: PgPool,
}

impl SampleSubmissionManager {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn create_sample(&self, sample: CreateSample) -> Result<Sample, sqlx::Error> {
        sqlx::query_as::<_, Sample>(
            r#"
            INSERT INTO samples (name, barcode, location, status, metadata)
            VALUES ($1, $2, $3, 'pending', $4)
            RETURNING id, name, barcode, location, status, created_at, updated_at, metadata
            "#,
        )
        .bind(&sample.name)
        .bind(&sample.barcode)
        .bind(&sample.location)
        .bind(sample.metadata.unwrap_or(serde_json::json!({})))
        .fetch_one(&self.pool)
        .await
    }

    pub async fn validate_sample(&self, sample_id: Uuid) -> Result<Sample, sqlx::Error> {
        sqlx::query_as::<_, Sample>(
            r#"
            UPDATE samples
            SET status = 'validated', updated_at = NOW()
            WHERE id = $1
            RETURNING id, name, barcode, location, status, created_at, updated_at, metadata
            "#,
        )
        .bind(sample_id)
        .fetch_one(&self.pool)
        .await
    }

    pub async fn get_sample(&self, sample_id: Uuid) -> Result<Sample, sqlx::Error> {
        sqlx::query_as::<_, Sample>(
            r#"
            SELECT id, name, barcode, location, status, created_at, updated_at, metadata
            FROM samples
            WHERE id = $1
            "#,
        )
        .bind(sample_id)
        .fetch_one(&self.pool)
        .await
    }

    pub async fn list_samples(&self) -> Result<Vec<Sample>, sqlx::Error> {
        sqlx::query_as::<_, Sample>(
            r#"
            SELECT id, name, barcode, location, status, created_at, updated_at, metadata
            FROM samples
            ORDER BY created_at DESC
            "#,
        )
        .fetch_all(&self.pool)
        .await
    }

    pub async fn update_sample(
        &self,
        sample_id: Uuid,
        updates: UpdateSample,
    ) -> Result<Sample, sqlx::Error> {
        // Build dynamic query based on provided fields
        let mut query_parts = Vec::new();
        let mut param_count = 1;

        if updates.name.is_some() {
            query_parts.push(format!("name = ${}", param_count));
            param_count += 1;
        }
        if updates.barcode.is_some() {
            query_parts.push(format!("barcode = ${}", param_count));
            param_count += 1;
        }
        if updates.location.is_some() {
            query_parts.push(format!("location = ${}", param_count));
            param_count += 1;
        }
        if updates.status.is_some() {
            query_parts.push(format!("status = ${}", param_count));
            param_count += 1;
        }
        if updates.metadata.is_some() {
            query_parts.push(format!("metadata = ${}", param_count));
            param_count += 1;
        }

        if query_parts.is_empty() {
            // No updates provided, just return the existing sample
            return self.get_sample(sample_id).await;
        }

        let query = format!(
            r#"
            UPDATE samples 
            SET {}, updated_at = NOW()
            WHERE id = ${}
            RETURNING id, name, barcode, location, status, created_at, updated_at, metadata
            "#,
            query_parts.join(", "),
            param_count
        );

        let mut query_builder = sqlx::query_as::<_, Sample>(&query);

        // Bind parameters in the same order they were added
        if let Some(name) = updates.name {
            query_builder = query_builder.bind(name);
        }
        if let Some(barcode) = updates.barcode {
            query_builder = query_builder.bind(barcode);
        }
        if let Some(location) = updates.location {
            query_builder = query_builder.bind(location);
        }
        if let Some(status) = updates.status {
            query_builder = query_builder.bind(status);
        }
        if let Some(metadata) = updates.metadata {
            query_builder = query_builder.bind(metadata);
        }

        // Bind the sample_id last
        query_builder = query_builder.bind(sample_id);

        query_builder.fetch_one(&self.pool).await
    }
}
