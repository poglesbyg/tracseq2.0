use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize)]
pub struct SequencingJob {
    pub id: Uuid,
    pub name: String,
    pub status: JobStatus,
    pub sample_sheet_path: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub metadata: serde_json::Value,
}

#[derive(Debug, Serialize, Deserialize, sqlx::Type)]
#[sqlx(type_name = "job_status", rename_all = "snake_case")]
pub enum JobStatus {
    Pending,
    InProgress,
    Completed,
    Failed,
}

#[derive(Debug, Deserialize)]
pub struct CreateJob {
    pub name: String,
    pub sample_sheet_path: String,
    pub metadata: Option<serde_json::Value>,
}

pub struct SequencingManager {
    pool: PgPool,
}

impl SequencingManager {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn create_job(&self, job: CreateJob) -> Result<SequencingJob, sqlx::Error> {
        sqlx::query_as!(
            SequencingJob,
            r#"
            INSERT INTO sequencing_jobs (name, status, sample_sheet_path, metadata)
            VALUES ($1, 'pending', $2, $3)
            RETURNING id, name, status as "status: JobStatus", sample_sheet_path, created_at, updated_at, metadata
            "#,
            job.name,
            job.sample_sheet_path,
            job.metadata.unwrap_or(serde_json::json!({}))
        )
        .fetch_one(&self.pool)
        .await
    }

    pub async fn update_job_status(
        &self,
        job_id: Uuid,
        status: JobStatus,
    ) -> Result<SequencingJob, sqlx::Error> {
        sqlx::query_as!(
            SequencingJob,
            r#"
            UPDATE sequencing_jobs
            SET status = $1, updated_at = NOW()
            WHERE id = $2
            RETURNING id, name, status as "status: JobStatus", sample_sheet_path, created_at, updated_at, metadata
            "#,
            status as JobStatus,
            job_id
        )
        .fetch_one(&self.pool)
        .await
    }

    pub async fn get_job(&self, job_id: Uuid) -> Result<SequencingJob, sqlx::Error> {
        sqlx::query_as!(
            SequencingJob,
            r#"
            SELECT id, name, status as "status: JobStatus", sample_sheet_path, created_at, updated_at, metadata
            FROM sequencing_jobs
            WHERE id = $1
            "#,
            job_id
        )
        .fetch_one(&self.pool)
        .await
    }

    pub async fn list_jobs(&self) -> Result<Vec<SequencingJob>, sqlx::Error> {
        sqlx::query_as!(
            SequencingJob,
            r#"
            SELECT id, name, status as "status: JobStatus", sample_sheet_path, created_at, updated_at, metadata
            FROM sequencing_jobs
            ORDER BY created_at DESC
            "#
        )
        .fetch_all(&self.pool)
        .await
    }
}
