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

#[derive(Debug, Serialize, Deserialize, sqlx::Type)]
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
        .bind(&sample.metadata.unwrap_or(serde_json::json!({})))
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
}
