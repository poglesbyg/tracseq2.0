use async_trait::async_trait;
use std::collections::HashMap;
use uuid::Uuid;

use crate::{
    sample_submission::{CreateSample, Sample, SampleSubmissionManager},
    services::{HealthCheck, HealthStatus, Service, ServiceConfig, ServiceHealth},
};

pub struct SampleService {
    manager: SampleSubmissionManager,
}

impl SampleService {
    pub fn new(manager: SampleSubmissionManager) -> Self {
        Self { manager }
    }

    pub async fn create_sample(&self, sample: CreateSample) -> Result<Sample, sqlx::Error> {
        self.manager.create_sample(sample).await
    }

    pub async fn validate_sample(&self, sample_id: Uuid) -> Result<Sample, sqlx::Error> {
        self.manager.validate_sample(sample_id).await
    }

    pub async fn get_sample(&self, sample_id: Uuid) -> Result<Sample, sqlx::Error> {
        self.manager.get_sample(sample_id).await
    }

    pub async fn list_samples(&self) -> Result<Vec<Sample>, sqlx::Error> {
        self.manager.list_samples().await
    }
}

#[async_trait]
impl Service for SampleService {
    fn name(&self) -> &'static str {
        "sample_service"
    }

    async fn health_check(&self) -> ServiceHealth {
        let mut checks = HashMap::new();

        // Test database connectivity by listing samples
        let start = std::time::Instant::now();
        let db_check = match self.manager.list_samples().await {
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
            message: Some("Sample service health check".to_string()),
            checks,
        }
    }

    fn config(&self) -> ServiceConfig {
        ServiceConfig {
            name: "sample_service".to_string(),
            version: "1.0.0".to_string(),
            dependencies: vec!["database".to_string()],
            settings: HashMap::new(),
        }
    }
}
