use async_trait::async_trait;
use std::collections::HashMap;
use uuid::Uuid;

use crate::{
    sequencing::{CreateJob, JobStatus, SequencingJob, SequencingManager},
    services::{HealthCheck, HealthStatus, Service, ServiceConfig, ServiceHealth},
};

pub struct SequencingService {
    manager: SequencingManager,
}

impl SequencingService {
    pub fn new(manager: SequencingManager) -> Self {
        Self { manager }
    }

    pub async fn create_job(&self, job: CreateJob) -> Result<SequencingJob, sqlx::Error> {
        self.manager.create_job(job).await
    }

    pub async fn update_job_status(
        &self,
        job_id: Uuid,
        status: JobStatus,
    ) -> Result<SequencingJob, sqlx::Error> {
        self.manager.update_job_status(job_id, status).await
    }

    pub async fn get_job(&self, job_id: Uuid) -> Result<SequencingJob, sqlx::Error> {
        self.manager.get_job(job_id).await
    }

    pub async fn list_jobs(&self) -> Result<Vec<SequencingJob>, sqlx::Error> {
        self.manager.list_jobs().await
    }
}

#[async_trait]
impl Service for SequencingService {
    fn name(&self) -> &'static str {
        "sequencing_service"
    }

    async fn health_check(&self) -> ServiceHealth {
        let mut checks = HashMap::new();

        // Test database connectivity by listing jobs
        let start = std::time::Instant::now();
        let db_check = match self.manager.list_jobs().await {
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
            message: Some("Sequencing service health check".to_string()),
            checks,
        }
    }

    fn config(&self) -> ServiceConfig {
        ServiceConfig {
            name: "sequencing_service".to_string(),
            version: "1.0.0".to_string(),
            dependencies: vec!["database".to_string()],
            settings: HashMap::new(),
        }
    }
}
