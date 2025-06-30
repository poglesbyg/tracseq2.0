#[cfg(test)]
use std::collections::HashMap;
use uuid::Uuid;

use crate::config::database::Database;
use crate::sample_submission::{CreateSample, SampleSubmissionManager};
use crate::sequencing::{CreateJob, JobStatus, SequencingManager};
use crate::services::{
    sample_service::SampleService, sequencing_service::SequencingService, HealthStatus, Service,
};

/// Test helper to create database connection
async fn create_test_database() -> Database {
    let database_url = std::env::var("TEST_DATABASE_URL")
        .unwrap_or_else(|_| "postgres://postgres:password@localhost/lab_manager_test".to_string());

    Database::new(&database_url)
        .await
        .expect("Failed to connect to test database")
}

/// Setup test data for service tests
async fn setup_sample_test_data(database: &Database) -> Result<(), sqlx::Error> {
    // Clean up existing test data
    sqlx::query("DELETE FROM samples WHERE name LIKE 'test_service_%'")
        .execute(&database.pool)
        .await?;

    // Create a test sample
    let sample_id = Uuid::new_v4();
    sqlx::query(
        "INSERT INTO samples (id, name, barcode, location, status, metadata, created_at, updated_at)
         VALUES ($1, $2, $3, $4, $5, $6, NOW(), NOW())"
    )
    .bind(sample_id)
    .bind("test_service_sample")
    .bind("TEST-SVC-001")
    .bind("Test Service Location")
    .bind("pending")
    .bind(serde_json::json!({"template_name": "test_template"}))
    .execute(&database.pool)
    .await?;

    Ok(())
}

/// Setup test data for sequencing service tests
async fn setup_sequencing_test_data(database: &Database) -> Result<(), sqlx::Error> {
    // Clean up existing test data
    sqlx::query("DELETE FROM sequencing_jobs WHERE name LIKE 'test_service_%'")
        .execute(&database.pool)
        .await?;

    // Create a test sequencing job
    let job_id = Uuid::new_v4();
    sqlx::query(
        "INSERT INTO sequencing_jobs (id, name, description, status, created_at, updated_at)
         VALUES ($1, $2, $3, $4, NOW(), NOW())",
    )
    .bind(job_id)
    .bind("test_service_job")
    .bind("Test service sequencing job")
    .bind("pending")
    .execute(&database.pool)
    .await?;

    Ok(())
}

/// Cleanup test data
async fn cleanup_service_test_data(database: &Database) -> Result<(), sqlx::Error> {
    sqlx::query("DELETE FROM samples WHERE name LIKE 'test_service_%'")
        .execute(&database.pool)
        .await?;
    sqlx::query("DELETE FROM sequencing_jobs WHERE name LIKE 'test_service_%'")
        .execute(&database.pool)
        .await?;
    Ok(())
}

// Sample Service Tests

#[tokio::test]
async fn test_sample_service_creation() {
    let database = create_test_database().await;
    let manager = SampleSubmissionManager::new(database.clone());
    let service = SampleService::new(manager);

    assert_eq!(service.name(), "sample_service");

    let config = service.config();
    assert_eq!(config.name, "sample_service");
    assert_eq!(config.version, "1.0.0");
    assert!(config.dependencies.contains(&"database".to_string()));
}

#[tokio::test]
async fn test_sample_service_create_sample() {
    let database = create_test_database().await;
    setup_sample_test_data(&database).await.unwrap();

    let manager = SampleSubmissionManager::new(database.clone());
    let service = SampleService::new(manager);

    let create_sample = CreateSample {
        name: "test_service_new_sample".to_string(),
        barcode: Some("TEST-SVC-NEW-001".to_string()),
        location: Some("New Test Location".to_string()),
        metadata: Some(serde_json::json!({"test": "data"})),
    };

    let result = service.create_sample(create_sample).await;
    assert!(result.is_ok());

    let sample = result.unwrap();
    assert_eq!(sample.name, "test_service_new_sample");
    assert_eq!(sample.barcode.unwrap(), "TEST-SVC-NEW-001");
    assert_eq!(sample.location.unwrap(), "New Test Location");

    cleanup_service_test_data(&database).await.unwrap();
}

#[tokio::test]
async fn test_sample_service_list_samples() {
    let database = create_test_database().await;
    setup_sample_test_data(&database).await.unwrap();

    let manager = SampleSubmissionManager::new(database.clone());
    let service = SampleService::new(manager);

    let result = service.list_samples().await;
    assert!(result.is_ok());

    let samples = result.unwrap();
    let test_samples: Vec<_> = samples
        .iter()
        .filter(|s| s.name.starts_with("test_service_"))
        .collect();

    assert!(test_samples.len() >= 1);

    cleanup_service_test_data(&database).await.unwrap();
}

#[tokio::test]
async fn test_sample_service_get_sample() {
    let database = create_test_database().await;
    setup_sample_test_data(&database).await.unwrap();

    let manager = SampleSubmissionManager::new(database.clone());
    let service = SampleService::new(manager);

    // First, get the ID of our test sample
    let samples = service.list_samples().await.unwrap();
    let test_sample = samples
        .iter()
        .find(|s| s.name == "test_service_sample")
        .expect("Should find test sample");

    let result = service.get_sample(test_sample.id).await;
    assert!(result.is_ok());

    let sample = result.unwrap();
    assert_eq!(sample.id, test_sample.id);
    assert_eq!(sample.name, "test_service_sample");

    cleanup_service_test_data(&database).await.unwrap();
}

#[tokio::test]
async fn test_sample_service_get_nonexistent_sample() {
    let database = create_test_database().await;
    let manager = SampleSubmissionManager::new(database.clone());
    let service = SampleService::new(manager);

    let nonexistent_id = Uuid::new_v4();
    let result = service.get_sample(nonexistent_id).await;
    assert!(result.is_err());
}

#[tokio::test]
async fn test_sample_service_validate_sample() {
    let database = create_test_database().await;
    setup_sample_test_data(&database).await.unwrap();

    let manager = SampleSubmissionManager::new(database.clone());
    let service = SampleService::new(manager);

    // Get our test sample
    let samples = service.list_samples().await.unwrap();
    let test_sample = samples
        .iter()
        .find(|s| s.name == "test_service_sample")
        .expect("Should find test sample");

    let result = service.validate_sample(test_sample.id).await;
    assert!(result.is_ok());

    let validated_sample = result.unwrap();
    assert_eq!(validated_sample.id, test_sample.id);
    // Status should be updated to validated
    assert_eq!(validated_sample.status, "validated");

    cleanup_service_test_data(&database).await.unwrap();
}

#[tokio::test]
async fn test_sample_service_health_check() {
    let database = create_test_database().await;
    let manager = SampleSubmissionManager::new(database.clone());
    let service = SampleService::new(manager);

    let health = service.health_check().await;

    assert_eq!(health.status, HealthStatus::Healthy);
    assert!(health.message.is_some());
    assert!(health.checks.contains_key("database"));

    let db_check = &health.checks["database"];
    assert_eq!(db_check.status, HealthStatus::Healthy);
    assert!(db_check.duration_ms > 0);
    assert!(db_check.details.is_some());
}

// Sequencing Service Tests

#[tokio::test]
async fn test_sequencing_service_creation() {
    let database = create_test_database().await;
    let manager = SequencingManager::new(database.clone());
    let service = SequencingService::new(manager);

    assert_eq!(service.name(), "sequencing_service");

    let config = service.config();
    assert_eq!(config.name, "sequencing_service");
    assert_eq!(config.version, "1.0.0");
    assert!(config.dependencies.contains(&"database".to_string()));
}

#[tokio::test]
async fn test_sequencing_service_create_job() {
    let database = create_test_database().await;
    setup_sequencing_test_data(&database).await.unwrap();

    let manager = SequencingManager::new(database.clone());
    let service = SequencingService::new(manager);

    let create_job = CreateJob {
        name: "test_service_new_job".to_string(),
        description: "Test service new sequencing job".to_string(),
        sample_ids: vec![Uuid::new_v4()],
    };

    let result = service.create_job(create_job).await;
    assert!(result.is_ok());

    let job = result.unwrap();
    assert_eq!(job.name, "test_service_new_job");
    assert_eq!(job.description, "Test service new sequencing job");
    assert_eq!(job.status, JobStatus::Pending);

    cleanup_service_test_data(&database).await.unwrap();
}

#[tokio::test]
async fn test_sequencing_service_list_jobs() {
    let database = create_test_database().await;
    setup_sequencing_test_data(&database).await.unwrap();

    let manager = SequencingManager::new(database.clone());
    let service = SequencingService::new(manager);

    let result = service.list_jobs().await;
    assert!(result.is_ok());

    let jobs = result.unwrap();
    let test_jobs: Vec<_> = jobs
        .iter()
        .filter(|j| j.name.starts_with("test_service_"))
        .collect();

    assert!(test_jobs.len() >= 1);

    cleanup_service_test_data(&database).await.unwrap();
}

#[tokio::test]
async fn test_sequencing_service_get_job() {
    let database = create_test_database().await;
    setup_sequencing_test_data(&database).await.unwrap();

    let manager = SequencingManager::new(database.clone());
    let service = SequencingService::new(manager);

    // First, get the ID of our test job
    let jobs = service.list_jobs().await.unwrap();
    let test_job = jobs
        .iter()
        .find(|j| j.name == "test_service_job")
        .expect("Should find test job");

    let result = service.get_job(test_job.id).await;
    assert!(result.is_ok());

    let job = result.unwrap();
    assert_eq!(job.id, test_job.id);
    assert_eq!(job.name, "test_service_job");

    cleanup_service_test_data(&database).await.unwrap();
}

#[tokio::test]
async fn test_sequencing_service_get_nonexistent_job() {
    let database = create_test_database().await;
    let manager = SequencingManager::new(database.clone());
    let service = SequencingService::new(manager);

    let nonexistent_id = Uuid::new_v4();
    let result = service.get_job(nonexistent_id).await;
    assert!(result.is_err());
}

#[tokio::test]
async fn test_sequencing_service_update_job_status() {
    let database = create_test_database().await;
    setup_sequencing_test_data(&database).await.unwrap();

    let manager = SequencingManager::new(database.clone());
    let service = SequencingService::new(manager);

    // Get our test job
    let jobs = service.list_jobs().await.unwrap();
    let test_job = jobs
        .iter()
        .find(|j| j.name == "test_service_job")
        .expect("Should find test job");

    let result = service
        .update_job_status(test_job.id, JobStatus::Running)
        .await;
    assert!(result.is_ok());

    let updated_job = result.unwrap();
    assert_eq!(updated_job.id, test_job.id);
    assert_eq!(updated_job.status, JobStatus::Running);

    // Test updating to completed
    let result = service
        .update_job_status(test_job.id, JobStatus::Completed)
        .await;
    assert!(result.is_ok());

    let completed_job = result.unwrap();
    assert_eq!(completed_job.status, JobStatus::Completed);

    cleanup_service_test_data(&database).await.unwrap();
}

#[tokio::test]
async fn test_sequencing_service_health_check() {
    let database = create_test_database().await;
    let manager = SequencingManager::new(database.clone());
    let service = SequencingService::new(manager);

    let health = service.health_check().await;

    assert_eq!(health.status, HealthStatus::Healthy);
    assert!(health.message.is_some());
    assert!(health.checks.contains_key("database"));

    let db_check = &health.checks["database"];
    assert_eq!(db_check.status, HealthStatus::Healthy);
    assert!(db_check.duration_ms > 0);
    assert!(db_check.details.is_some());
}

// Cross-service integration tests

#[tokio::test]
async fn test_sample_to_sequencing_workflow() {
    let database = create_test_database().await;
    let sample_manager = SampleSubmissionManager::new(database.clone());
    let sequencing_manager = SequencingManager::new(database.clone());

    let sample_service = SampleService::new(sample_manager);
    let sequencing_service = SequencingService::new(sequencing_manager);

    // Create a sample
    let create_sample = CreateSample {
        name: "test_workflow_sample".to_string(),
        barcode: Some("TEST-WORKFLOW-001".to_string()),
        location: Some("Workflow Test Location".to_string()),
        metadata: Some(serde_json::json!({"workflow": "test"})),
    };

    let sample = sample_service.create_sample(create_sample).await.unwrap();

    // Validate the sample
    let validated_sample = sample_service.validate_sample(sample.id).await.unwrap();
    assert_eq!(validated_sample.status, "validated");

    // Create a sequencing job for the sample
    let create_job = CreateJob {
        name: "test_workflow_job".to_string(),
        description: "Test workflow sequencing job".to_string(),
        sample_ids: vec![validated_sample.id],
    };

    let job = sequencing_service.create_job(create_job).await.unwrap();
    assert_eq!(job.status, JobStatus::Pending);

    // Update job status through the workflow
    let running_job = sequencing_service
        .update_job_status(job.id, JobStatus::Running)
        .await
        .unwrap();
    assert_eq!(running_job.status, JobStatus::Running);

    let completed_job = sequencing_service
        .update_job_status(job.id, JobStatus::Completed)
        .await
        .unwrap();
    assert_eq!(completed_job.status, JobStatus::Completed);

    // Cleanup
    cleanup_service_test_data(&database).await.unwrap();
}

#[tokio::test]
async fn test_service_configurations_consistency() {
    let database = create_test_database().await;

    let sample_manager = SampleSubmissionManager::new(database.clone());
    let sequencing_manager = SequencingManager::new(database.clone());

    let sample_service = SampleService::new(sample_manager);
    let sequencing_service = SequencingService::new(sequencing_manager);

    let sample_config = sample_service.config();
    let sequencing_config = sequencing_service.config();

    // Both services should depend on database
    assert!(sample_config.dependencies.contains(&"database".to_string()));
    assert!(sequencing_config
        .dependencies
        .contains(&"database".to_string()));

    // Both should have valid versions
    assert!(!sample_config.version.is_empty());
    assert!(!sequencing_config.version.is_empty());

    // Names should be unique
    assert_ne!(sample_config.name, sequencing_config.name);
}

#[tokio::test]
async fn test_service_error_handling() {
    let database = create_test_database().await;
    let sample_manager = SampleSubmissionManager::new(database.clone());
    let sample_service = SampleService::new(sample_manager);

    // Test invalid sample creation (duplicate barcode)
    let create_sample1 = CreateSample {
        name: "test_error_sample1".to_string(),
        barcode: Some("TEST-ERROR-DUPLICATE".to_string()),
        location: Some("Error Test Location".to_string()),
        metadata: None,
    };

    let create_sample2 = CreateSample {
        name: "test_error_sample2".to_string(),
        barcode: Some("TEST-ERROR-DUPLICATE".to_string()), // Same barcode
        location: Some("Error Test Location".to_string()),
        metadata: None,
    };

    // First sample should succeed
    let result1 = sample_service.create_sample(create_sample1).await;
    assert!(result1.is_ok());

    // Second sample with duplicate barcode should fail
    let result2 = sample_service.create_sample(create_sample2).await;
    assert!(result2.is_err());

    cleanup_service_test_data(&database).await.unwrap();
}
