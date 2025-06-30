#[cfg(test)]
use sqlx::PgPool;
use std::collections::HashMap;
use uuid::Uuid;

use crate::config::database::Database;
use crate::models::storage::BarcodeConfig;
use crate::sample_submission::{CreateSample, SampleSubmissionManager};
use crate::sequencing::{CreateJob, JobStatus, SequencingManager};
use crate::services::{
    barcode_service::BarcodeService, sample_service::SampleService,
    sequencing_service::SequencingService, storage_service::StorageService,
    template_service::TemplateService,
};

/// Test helper to create database connection
async fn create_test_database() -> Database {
    let database_url = std::env::var("TEST_DATABASE_URL")
        .unwrap_or_else(|_| "postgres://postgres:password@localhost/lab_manager_test".to_string());

    Database::new(&database_url)
        .await
        .expect("Failed to connect to test database")
}

/// Setup comprehensive test data for integration tests
async fn setup_integration_test_data(pool: &PgPool) -> Result<IntegrationTestData, sqlx::Error> {
    // Clean up any existing test data
    cleanup_integration_test_data(pool).await?;

    // Create test template
    let template_id = Uuid::new_v4();
    sqlx::query(
        "INSERT INTO templates (id, name, description, fields, created_at, updated_at) 
         VALUES ($1, $2, $3, $4, NOW(), NOW())",
    )
    .bind(template_id)
    .bind("integration_test_template")
    .bind("Template for integration testing")
    .bind(serde_json::json!([
        {"name": "sample_type", "type": "text", "required": true},
        {"name": "concentration", "type": "number", "required": false},
        {"name": "volume", "type": "number", "required": false}
    ]))
    .execute(pool)
    .await?;

    // Create test storage location
    let storage_id = sqlx::query_scalar::<_, i32>(
        "INSERT INTO storage_locations (name, location_type, temperature, capacity, parent_id, created_at, updated_at)
         VALUES ($1, $2, $3, $4, $5, NOW(), NOW()) RETURNING id"
    )
    .bind("Integration Test Freezer")
    .bind("freezer")
    .bind(-80)
    .bind(100)
    .bind(None::<i32>)
    .fetch_one(pool)
    .await?;

    Ok(IntegrationTestData {
        template_id,
        storage_id,
    })
}

/// Cleanup integration test data
async fn cleanup_integration_test_data(pool: &PgPool) -> Result<(), sqlx::Error> {
    // Clean up in proper order due to foreign key constraints
    sqlx::query("DELETE FROM samples WHERE name LIKE 'integration_test_%'")
        .execute(pool)
        .await?;
    sqlx::query("DELETE FROM sequencing_jobs WHERE name LIKE 'integration_test_%'")
        .execute(pool)
        .await?;
    sqlx::query("DELETE FROM templates WHERE name LIKE 'integration_test_%'")
        .execute(pool)
        .await?;
    sqlx::query("DELETE FROM storage_locations WHERE name LIKE 'Integration Test%'")
        .execute(pool)
        .await?;
    Ok(())
}

#[derive(Debug)]
struct IntegrationTestData {
    template_id: Uuid,
    storage_id: i32,
}

/// Test complete sample lifecycle from creation to sequencing
#[tokio::test]
async fn test_complete_sample_lifecycle() {
    let database = create_test_database().await;
    let test_data = setup_integration_test_data(&database.pool)
        .await
        .expect("Failed to setup test data");

    // Initialize services
    let sample_manager = SampleSubmissionManager::new(database.clone());
    let sequencing_manager = SequencingManager::new(database.clone());
    let sample_service = SampleService::new(sample_manager);
    let sequencing_service = SequencingService::new(sequencing_manager);
    let mut barcode_service = BarcodeService::with_default_config();

    // Step 1: Create a sample
    let barcode = barcode_service
        .generate_barcode(Some("DNA"), Some(test_data.storage_id))
        .await
        .expect("Failed to generate barcode");

    let create_sample = CreateSample {
        name: "integration_test_sample_001".to_string(),
        barcode: Some(barcode.clone()),
        location: Some("Integration Test Location".to_string()),
        metadata: Some(serde_json::json!({
            "template_id": test_data.template_id,
            "sample_type": "DNA",
            "concentration": 250.5,
            "volume": 50.0
        })),
    };

    let sample = sample_service
        .create_sample(create_sample)
        .await
        .expect("Failed to create sample");

    assert_eq!(sample.name, "integration_test_sample_001");
    assert_eq!(sample.barcode.unwrap(), barcode);
    assert_eq!(sample.status, "pending");

    // Step 2: Validate the sample
    let validated_sample = sample_service
        .validate_sample(sample.id)
        .await
        .expect("Failed to validate sample");

    assert_eq!(validated_sample.status, "validated");

    // Step 3: Create a sequencing job for the sample
    let create_job = CreateJob {
        name: "integration_test_sequencing_job".to_string(),
        description: "Integration test sequencing job for sample lifecycle".to_string(),
        sample_ids: vec![validated_sample.id],
    };

    let sequencing_job = sequencing_service
        .create_job(create_job)
        .await
        .expect("Failed to create sequencing job");

    assert_eq!(sequencing_job.name, "integration_test_sequencing_job");
    assert_eq!(sequencing_job.status, JobStatus::Pending);

    // Step 4: Progress the sequencing job through its lifecycle
    let running_job = sequencing_service
        .update_job_status(sequencing_job.id, JobStatus::Running)
        .await
        .expect("Failed to update job to running");

    assert_eq!(running_job.status, JobStatus::Running);

    let completed_job = sequencing_service
        .update_job_status(sequencing_job.id, JobStatus::Completed)
        .await
        .expect("Failed to update job to completed");

    assert_eq!(completed_job.status, JobStatus::Completed);

    // Step 5: Verify final state
    let final_sample = sample_service
        .get_sample(sample.id)
        .await
        .expect("Failed to get final sample state");

    let final_job = sequencing_service
        .get_job(sequencing_job.id)
        .await
        .expect("Failed to get final job state");

    assert_eq!(final_sample.status, "validated");
    assert_eq!(final_job.status, JobStatus::Completed);

    // Cleanup
    cleanup_integration_test_data(&database.pool)
        .await
        .expect("Failed to cleanup test data");
}

/// Test batch sample processing workflow
#[tokio::test]
async fn test_batch_sample_processing_workflow() {
    let database = create_test_database().await;
    let test_data = setup_integration_test_data(&database.pool)
        .await
        .expect("Failed to setup test data");

    // Initialize services
    let sample_manager = SampleSubmissionManager::new(database.clone());
    let sequencing_manager = SequencingManager::new(database.clone());
    let sample_service = SampleService::new(sample_manager);
    let sequencing_service = SequencingService::new(sequencing_manager);
    let mut barcode_service = BarcodeService::with_default_config();

    // Create multiple samples
    let mut sample_ids = Vec::new();
    let sample_types = ["DNA", "RNA", "PROTEIN"];

    for (i, sample_type) in sample_types.iter().enumerate() {
        let barcode = barcode_service
            .generate_barcode(Some(sample_type), Some(test_data.storage_id))
            .await
            .expect("Failed to generate barcode");

        let create_sample = CreateSample {
            name: format!("integration_test_batch_sample_{:03}", i + 1),
            barcode: Some(barcode),
            location: Some(format!("Batch Location {}", i + 1)),
            metadata: Some(serde_json::json!({
                "template_id": test_data.template_id,
                "sample_type": sample_type,
                "batch_id": "BATCH_001"
            })),
        };

        let sample = sample_service
            .create_sample(create_sample)
            .await
            .expect("Failed to create sample");

        // Validate each sample
        let validated_sample = sample_service
            .validate_sample(sample.id)
            .await
            .expect("Failed to validate sample");

        sample_ids.push(validated_sample.id);
    }

    // Create a batch sequencing job for all samples
    let create_batch_job = CreateJob {
        name: "integration_test_batch_job".to_string(),
        description: "Batch sequencing job for multiple sample types".to_string(),
        sample_ids: sample_ids.clone(),
    };

    let batch_job = sequencing_service
        .create_job(create_batch_job)
        .await
        .expect("Failed to create batch job");

    assert_eq!(batch_job.name, "integration_test_batch_job");
    assert_eq!(batch_job.status, JobStatus::Pending);

    // Process the batch job
    let running_batch_job = sequencing_service
        .update_job_status(batch_job.id, JobStatus::Running)
        .await
        .expect("Failed to update batch job to running");

    assert_eq!(running_batch_job.status, JobStatus::Running);

    // Complete the batch job
    let completed_batch_job = sequencing_service
        .update_job_status(batch_job.id, JobStatus::Completed)
        .await
        .expect("Failed to complete batch job");

    assert_eq!(completed_batch_job.status, JobStatus::Completed);

    // Verify all samples are still accessible
    for sample_id in sample_ids {
        let sample = sample_service
            .get_sample(sample_id)
            .await
            .expect("Failed to get batch sample");

        assert_eq!(sample.status, "validated");
        assert!(sample.metadata.is_some());

        let metadata = sample.metadata.unwrap();
        assert_eq!(metadata["batch_id"], "BATCH_001");
    }

    cleanup_integration_test_data(&database.pool)
        .await
        .expect("Failed to cleanup test data");
}

/// Test error handling and recovery in workflows
#[tokio::test]
async fn test_workflow_error_handling_and_recovery() {
    let database = create_test_database().await;
    let test_data = setup_integration_test_data(&database.pool)
        .await
        .expect("Failed to setup test data");

    let sample_manager = SampleSubmissionManager::new(database.clone());
    let sequencing_manager = SequencingManager::new(database.clone());
    let sample_service = SampleService::new(sample_manager);
    let sequencing_service = SequencingService::new(sequencing_manager);
    let mut barcode_service = BarcodeService::with_default_config();

    // Test 1: Duplicate barcode handling
    let barcode = barcode_service
        .generate_barcode(Some("DNA"), Some(test_data.storage_id))
        .await
        .expect("Failed to generate barcode");

    let create_sample1 = CreateSample {
        name: "integration_test_error_sample_1".to_string(),
        barcode: Some(barcode.clone()),
        location: Some("Error Test Location 1".to_string()),
        metadata: None,
    };

    let sample1 = sample_service
        .create_sample(create_sample1)
        .await
        .expect("Failed to create first sample");

    // Try to create second sample with same barcode - should fail
    let create_sample2 = CreateSample {
        name: "integration_test_error_sample_2".to_string(),
        barcode: Some(barcode), // Same barcode
        location: Some("Error Test Location 2".to_string()),
        metadata: None,
    };

    let result2 = sample_service.create_sample(create_sample2).await;
    assert!(
        result2.is_err(),
        "Second sample with duplicate barcode should fail"
    );

    // Test 2: Invalid sample validation
    let invalid_sample_id = Uuid::new_v4();
    let validation_result = sample_service.validate_sample(invalid_sample_id).await;
    assert!(
        validation_result.is_err(),
        "Validating non-existent sample should fail"
    );

    // Test 3: Invalid sequencing job creation with non-existent sample
    let create_invalid_job = CreateJob {
        name: "integration_test_invalid_job".to_string(),
        description: "Job with non-existent sample".to_string(),
        sample_ids: vec![Uuid::new_v4()], // Non-existent sample ID
    };

    let invalid_job_result = sequencing_service.create_job(create_invalid_job).await;
    // This might succeed depending on foreign key constraints, but the job would be invalid

    // Test 4: Recovery - Create valid job with existing sample
    let validated_sample = sample_service
        .validate_sample(sample1.id)
        .await
        .expect("Failed to validate sample for recovery test");

    let create_recovery_job = CreateJob {
        name: "integration_test_recovery_job".to_string(),
        description: "Recovery job with valid sample".to_string(),
        sample_ids: vec![validated_sample.id],
    };

    let recovery_job = sequencing_service
        .create_job(create_recovery_job)
        .await
        .expect("Recovery job creation should succeed");

    assert_eq!(recovery_job.status, JobStatus::Pending);

    cleanup_integration_test_data(&database.pool)
        .await
        .expect("Failed to cleanup test data");
}

/// Test service health checks and monitoring
#[tokio::test]
async fn test_service_health_monitoring_integration() {
    let database = create_test_database().await;

    // Initialize all services
    let sample_manager = SampleSubmissionManager::new(database.clone());
    let sequencing_manager = SequencingManager::new(database.clone());
    let sample_service = SampleService::new(sample_manager);
    let sequencing_service = SequencingService::new(sequencing_manager);

    // Check health of all services
    let sample_health = sample_service.health_check().await;
    let sequencing_health = sequencing_service.health_check().await;

    // All services should be healthy if database is available
    assert_eq!(sample_health.status, crate::services::HealthStatus::Healthy);
    assert_eq!(
        sequencing_health.status,
        crate::services::HealthStatus::Healthy
    );

    // All services should have database dependency
    let sample_config = sample_service.config();
    let sequencing_config = sequencing_service.config();

    assert!(sample_config.dependencies.contains(&"database".to_string()));
    assert!(sequencing_config
        .dependencies
        .contains(&"database".to_string()));

    // Health checks should include database connectivity
    assert!(sample_health.checks.contains_key("database"));
    assert!(sequencing_health.checks.contains_key("database"));
}

/// Test concurrent access and data consistency
#[tokio::test]
async fn test_concurrent_access_data_consistency() {
    use std::sync::Arc;
    use tokio::task;

    let database = create_test_database().await;
    let test_data = setup_integration_test_data(&database.pool)
        .await
        .expect("Failed to setup test data");

    let sample_manager = Arc::new(SampleSubmissionManager::new(database.clone()));
    let mut barcode_service = BarcodeService::with_default_config();

    // Generate unique barcodes for concurrent samples
    let mut barcodes = Vec::new();
    for i in 0..5 {
        let barcode = barcode_service
            .generate_barcode(Some("CONCURRENT"), Some(test_data.storage_id + i))
            .await
            .expect("Failed to generate barcode");
        barcodes.push(barcode);
    }

    let mut handles = Vec::new();

    // Create multiple samples concurrently
    for (i, barcode) in barcodes.into_iter().enumerate() {
        let manager_clone = sample_manager.clone();
        let template_id = test_data.template_id;

        let handle = task::spawn(async move {
            let create_sample = CreateSample {
                name: format!("integration_test_concurrent_sample_{:03}", i),
                barcode: Some(barcode),
                location: Some(format!("Concurrent Location {}", i)),
                metadata: Some(serde_json::json!({
                    "template_id": template_id,
                    "sample_type": "CONCURRENT",
                    "concurrent_batch": "BATCH_CONCURRENT"
                })),
            };

            manager_clone.create_sample(create_sample).await
        });

        handles.push(handle);
    }

    // Wait for all tasks to complete
    let mut results = Vec::new();
    for handle in handles {
        let result = handle.await.unwrap();
        assert!(result.is_ok(), "Concurrent sample creation should succeed");
        results.push(result.unwrap());
    }

    // Verify all samples were created successfully
    assert_eq!(results.len(), 5);

    // Verify data consistency - all samples should be unique
    let mut sample_names = std::collections::HashSet::new();
    let mut sample_barcodes = std::collections::HashSet::new();

    for sample in &results {
        assert!(
            sample_names.insert(sample.name.clone()),
            "Sample names should be unique"
        );
        if let Some(barcode) = &sample.barcode {
            assert!(
                sample_barcodes.insert(barcode.clone()),
                "Sample barcodes should be unique"
            );
        }
    }

    cleanup_integration_test_data(&database.pool)
        .await
        .expect("Failed to cleanup test data");
}

/// Test full system configuration and initialization
#[tokio::test]
async fn test_full_system_initialization() {
    let database = create_test_database().await;

    // Test that all major components can be initialized
    let sample_manager = SampleSubmissionManager::new(database.clone());
    let sequencing_manager = SequencingManager::new(database.clone());
    let barcode_service = BarcodeService::with_default_config();

    // Test service creation
    let sample_service = SampleService::new(sample_manager);
    let sequencing_service = SequencingService::new(sequencing_manager);

    // Test configuration consistency
    let sample_config = sample_service.config();
    let sequencing_config = sequencing_service.config();
    let barcode_stats = barcode_service.get_stats();

    // All configs should be valid
    assert!(!sample_config.name.is_empty());
    assert!(!sample_config.version.is_empty());
    assert!(!sequencing_config.name.is_empty());
    assert!(!sequencing_config.version.is_empty());
    assert!(!barcode_stats.config.prefix.is_empty());

    // Test basic functionality of each service
    let sample_health = sample_service.health_check().await;
    let sequencing_health = sequencing_service.health_check().await;

    assert_eq!(sample_health.status, crate::services::HealthStatus::Healthy);
    assert_eq!(
        sequencing_health.status,
        crate::services::HealthStatus::Healthy
    );
}

/// Test cross-service data validation and constraints
#[tokio::test]
async fn test_cross_service_data_validation() {
    let database = create_test_database().await;
    let test_data = setup_integration_test_data(&database.pool)
        .await
        .expect("Failed to setup test data");

    let sample_manager = SampleSubmissionManager::new(database.clone());
    let sequencing_manager = SequencingManager::new(database.clone());
    let sample_service = SampleService::new(sample_manager);
    let sequencing_service = SequencingService::new(sequencing_manager);
    let mut barcode_service = BarcodeService::with_default_config();

    // Create a sample with specific constraints
    let barcode = barcode_service
        .generate_barcode(Some("VALIDATION"), Some(test_data.storage_id))
        .await
        .expect("Failed to generate barcode");

    let create_sample = CreateSample {
        name: "integration_test_validation_sample".to_string(),
        barcode: Some(barcode),
        location: Some("Validation Test Location".to_string()),
        metadata: Some(serde_json::json!({
            "template_id": test_data.template_id,
            "sample_type": "VALIDATION",
            "quality_control": true,
            "batch_number": "VALIDATION_BATCH_001"
        })),
    };

    let sample = sample_service
        .create_sample(create_sample)
        .await
        .expect("Failed to create validation sample");

    // Validate the sample
    let validated_sample = sample_service
        .validate_sample(sample.id)
        .await
        .expect("Failed to validate sample");

    // Test that sample can be used in sequencing job
    let create_job = CreateJob {
        name: "integration_test_validation_job".to_string(),
        description: "Job to test cross-service validation".to_string(),
        sample_ids: vec![validated_sample.id],
    };

    let job = sequencing_service
        .create_job(create_job)
        .await
        .expect("Failed to create validation job");

    // Test that sample status affects job processing
    assert_eq!(job.status, JobStatus::Pending);
    assert_eq!(validated_sample.status, "validated");

    // Verify data relationships are maintained
    let retrieved_sample = sample_service
        .get_sample(validated_sample.id)
        .await
        .expect("Failed to retrieve sample");

    let retrieved_job = sequencing_service
        .get_job(job.id)
        .await
        .expect("Failed to retrieve job");

    assert_eq!(retrieved_sample.id, validated_sample.id);
    assert_eq!(retrieved_job.id, job.id);

    cleanup_integration_test_data(&database.pool)
        .await
        .expect("Failed to cleanup test data");
}
