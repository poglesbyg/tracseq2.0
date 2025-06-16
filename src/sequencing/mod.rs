use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize, sqlx::FromRow)]
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
    #[serde(default)]
    pub sample_sheet_path: Option<String>,
    pub sample_ids: Option<Vec<i32>>,
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
        // Generate sample sheet path if not provided but sample_ids are available
        let sample_sheet_path = job.sample_sheet_path.unwrap_or_else(|| {
            if let Some(ref sample_ids) = job.sample_ids {
                format!(
                    "/sample_sheets/job_{}_{}.csv",
                    chrono::Utc::now().format("%Y%m%d_%H%M%S"),
                    sample_ids.len()
                )
            } else {
                "/sample_sheets/default.csv".to_string()
            }
        });

        // Add sample_ids to metadata if provided
        let mut metadata = job.metadata.unwrap_or(serde_json::json!({}));
        if let Some(sample_ids) = job.sample_ids {
            if let serde_json::Value::Object(ref mut map) = metadata {
                map.insert("sample_ids".to_string(), serde_json::json!(sample_ids));
            }
        }

        sqlx::query_as::<_, SequencingJob>(
            r#"
            INSERT INTO sequencing_jobs (name, status, sample_sheet_path, metadata)
            VALUES ($1, 'pending', $2, $3)
            RETURNING id, name, status, sample_sheet_path, created_at, updated_at, metadata
            "#,
        )
        .bind(&job.name)
        .bind(&sample_sheet_path)
        .bind(metadata)
        .fetch_one(&self.pool)
        .await
    }

    pub async fn update_job_status(
        &self,
        job_id: Uuid,
        status: JobStatus,
    ) -> Result<SequencingJob, sqlx::Error> {
        sqlx::query_as::<_, SequencingJob>(
            r#"
            UPDATE sequencing_jobs
            SET status = $1, updated_at = NOW()
            WHERE id = $2
            RETURNING id, name, status, sample_sheet_path, created_at, updated_at, metadata
            "#,
        )
        .bind(status)
        .bind(job_id)
        .fetch_one(&self.pool)
        .await
    }

    pub async fn get_job(&self, job_id: Uuid) -> Result<SequencingJob, sqlx::Error> {
        sqlx::query_as::<_, SequencingJob>(
            r#"
            SELECT id, name, status, sample_sheet_path, created_at, updated_at, metadata
            FROM sequencing_jobs
            WHERE id = $1
            "#,
        )
        .bind(job_id)
        .fetch_one(&self.pool)
        .await
    }

    pub async fn list_jobs(&self) -> Result<Vec<SequencingJob>, sqlx::Error> {
        sqlx::query_as::<_, SequencingJob>(
            r#"
            SELECT id, name, status, sample_sheet_path, created_at, updated_at, metadata
            FROM sequencing_jobs
            ORDER BY created_at DESC
            "#,
        )
        .fetch_all(&self.pool)
        .await
    }
}
