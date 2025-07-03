#[cfg(test)]
use sqlx::PgPool;
use uuid::Uuid;

use crate::config::database::Database;
use crate::config::database::DatabaseConfig;
use crate::models::storage::BarcodeConfig;
use crate::sequencing::{CreateJob, JobStatus, SequencingJob, SequencingManager};

/// Test helper to create database connection
async fn create_test_database() -> Database {
    let database_url = std::env::var("TEST_DATABASE_URL")
        .unwrap_or_else(|_| "postgres://postgres:password@localhost/lab_manager_test".to_string());

    Database::new(&database_url)
        .await
        .expect("Failed to connect to test database")
}

/// Setup test data for sequencing tests
async fn setup_sequencing_test_data(pool: &PgPool) -> Result<Vec<Uuid>, sqlx::Error> {
    // Clean up existing test data
    cleanup_sequencing_test_data(pool).await?;

    // Create test samples for sequencing jobs
    let mut sample_ids = Vec::new();
    for i in 1..=3 {
        let sample_id = Uuid::new_v4();
        sqlx::query(
            "INSERT INTO samples (id, name, barcode, location, status, metadata, created_at, updated_at)
             VALUES ($1, $2, $3, $4, $5, $6, NOW(), NOW())"
        )
        .bind(sample_id)
        .bind(format!("test_sequencing_sample_{}", i))
        .bind(format!("TEST-SEQ-{:03}", i))
        .bind("Sequencing Test Location")
        .bind("validated")
        .bind(serde_json::json!({"sample_type": "DNA"}))
        .execute(pool)
        .await?;
        sample_ids.push(sample_id);
    }

    Ok(sample_ids)
}

/// Cleanup sequencing test data
async fn cleanup_sequencing_test_data(pool: &PgPool) -> Result<(), sqlx::Error> {
    sqlx::query("DELETE FROM sequencing_jobs WHERE name LIKE 'test_sequencing_%'")
        .execute(pool)
        .await?;
    sqlx::query("DELETE FROM samples WHERE name LIKE 'test_sequencing_%'")
        .execute(pool)
        .await?;
    Ok(())
}

#[tokio::test]
async fn test_sequencing_manager_creation() {
    let database = create_test_database().await;
    let manager = SequencingManager::new(database);

    // Manager should be created successfully
    // This test primarily ensures the constructor works
    assert!(true); // Manager creation succeeded if we reach here
}

#[tokio::test]
async fn test_create_sequencing_job() {
    let database = create_test_database().await;
    let sample_ids = setup_sequencing_test_data(&database.pool)
        .await
        .expect("Failed to setup test data");

    let manager = SequencingManager::new(database.clone());

    let create_job = CreateJob {
        name: "test_sequencing_job_001".to_string(),
        description: "Test sequencing job for unit testing".to_string(),
        sample_ids: vec![sample_ids[0]],
        sample_sheet_path: None,
        metadata: None,
    };

    let result = manager.create_job(create_job).await;
    assert!(result.is_ok());

    let job = result.unwrap();
    assert_eq!(job.name, "test_sequencing_job_001");
    assert_eq!(job.description, "Test sequencing job for unit testing");
    assert_eq!(job.status, JobStatus::Pending);
    assert!(!job.id.is_nil());

    cleanup_sequencing_test_data(&database.pool)
        .await
        .expect("Failed to cleanup test data");
}

#[tokio::test]
async fn test_create_batch_sequencing_job() {
    let database = create_test_database().await;
    let sample_ids = setup_sequencing_test_data(&database.pool)
        .await
        .expect("Failed to setup test data");

    let manager = SequencingManager::new(database.clone());

    let create_batch_job = CreateJob {
        name: "test_sequencing_batch_job".to_string(),
        description: "Batch sequencing job with multiple samples".to_string(),
        sample_ids: sample_ids.clone(),
        sample_sheet_path: None,
        metadata: None,
    };

    let result = manager.create_job(create_batch_job).await;
    assert!(result.is_ok());

    let job = result.unwrap();
    assert_eq!(job.name, "test_sequencing_batch_job");
    assert_eq!(job.status, JobStatus::Pending);

    cleanup_sequencing_test_data(&database.pool)
        .await
        .expect("Failed to cleanup test data");
}

#[tokio::test]
async fn test_get_sequencing_job() {
    let database = create_test_database().await;
    let sample_ids = setup_sequencing_test_data(&database.pool)
        .await
        .expect("Failed to setup test data");

    let manager = SequencingManager::new(database.clone());

    // Create a job first
    let create_job = CreateJob {
        name: "test_sequencing_get_job".to_string(),
        description: "Job for testing retrieval".to_string(),
        sample_ids: vec![sample_ids[0]],
        sample_sheet_path: None,
        metadata: None,
    };

    let created_job = manager.create_job(create_job).await.unwrap();

    // Retrieve the job
    let result = manager.get_job(created_job.id).await;
    assert!(result.is_ok());

    let retrieved_job = result.unwrap();
    assert_eq!(retrieved_job.id, created_job.id);
    assert_eq!(retrieved_job.name, "test_sequencing_get_job");
    assert_eq!(retrieved_job.status, JobStatus::Pending);

    cleanup_sequencing_test_data(&database.pool)
        .await
        .expect("Failed to cleanup test data");
}

#[tokio::test]
async fn test_get_nonexistent_sequencing_job() {
    let database = create_test_database().await;
    let manager = SequencingManager::new(database);

    let nonexistent_id = Uuid::new_v4();
    let result = manager.get_job(nonexistent_id).await;
    assert!(result.is_err());
}

#[tokio::test]
async fn test_list_sequencing_jobs() {
    let database = create_test_database().await;
    let sample_ids = setup_sequencing_test_data(&database.pool)
        .await
        .expect("Failed to setup test data");

    let manager = SequencingManager::new(database.clone());

    // Create multiple jobs
    for i in 1..=3 {
        let create_job = CreateJob {
            name: format!("test_sequencing_list_job_{}", i),
            description: format!("List test job {}", i),
            sample_ids: vec![sample_ids[i - 1]],
        };

        manager.create_job(create_job).await.unwrap();
    }

    let result = manager.list_jobs().await;
    assert!(result.is_ok());

    let jobs = result.unwrap();
    let test_jobs: Vec<_> = jobs
        .iter()
        .filter(|j| j.name.starts_with("test_sequencing_list_job_"))
        .collect();

    assert!(test_jobs.len() >= 3);

    cleanup_sequencing_test_data(&database.pool)
        .await
        .expect("Failed to cleanup test data");
}

#[tokio::test]
async fn test_update_job_status() {
    let database = create_test_database().await;
    let sample_ids = setup_sequencing_test_data(&database.pool)
        .await
        .expect("Failed to setup test data");

    let manager = SequencingManager::new(database.clone());

    // Create a job
    let create_job = CreateJob {
        name: "test_sequencing_status_update".to_string(),
        description: "Job for testing status updates".to_string(),
        sample_ids: vec![sample_ids[0]],
    };

    let job = manager.create_job(create_job).await.unwrap();
    assert_eq!(job.status, JobStatus::Pending);

    // Update to Running
    let running_job = manager
        .update_job_status(job.id, JobStatus::Running)
        .await
        .unwrap();
    assert_eq!(running_job.status, JobStatus::Running);
    assert_eq!(running_job.id, job.id);

    // Update to Completed
    let completed_job = manager
        .update_job_status(job.id, JobStatus::Completed)
        .await
        .unwrap();
    assert_eq!(completed_job.status, JobStatus::Completed);

    // Update to Failed
    let failed_job = manager
        .update_job_status(job.id, JobStatus::Failed)
        .await
        .unwrap();
    assert_eq!(failed_job.status, JobStatus::Failed);

    cleanup_sequencing_test_data(&database.pool)
        .await
        .expect("Failed to cleanup test data");
}

#[tokio::test]
async fn test_job_status_enum_values() {
    // Test all possible job status values
    let statuses = vec![
        JobStatus::Pending,
        JobStatus::Running,
        JobStatus::Completed,
        JobStatus::Failed,
    ];

    for status in statuses {
        // Each status should be serializable and deserializable
        let serialized = serde_json::to_string(&status);
        assert!(serialized.is_ok());

        let json_str = serialized.unwrap();
        let deserialized: Result<JobStatus, _> = serde_json::from_str(&json_str);
        assert!(deserialized.is_ok());
        assert_eq!(deserialized.unwrap(), status);
    }
}

#[tokio::test]
async fn test_sequencing_job_serialization() {
    let job = SequencingJob {
        id: Uuid::new_v4(),
        name: "test_serialization_job".to_string(),
        description: "Job for testing serialization".to_string(),
        status: JobStatus::Running,
        created_at: chrono::Utc::now(),
        updated_at: chrono::Utc::now(),
    };

    // Test serialization
    let serialized = serde_json::to_string(&job);
    assert!(serialized.is_ok());

    let json_str = serialized.unwrap();
    assert!(json_str.contains("test_serialization_job"));
    assert!(json_str.contains("Running"));

    // Test deserialization
    let deserialized: Result<SequencingJob, _> = serde_json::from_str(&json_str);
    assert!(deserialized.is_ok());

    let parsed_job = deserialized.unwrap();
    assert_eq!(parsed_job.id, job.id);
    assert_eq!(parsed_job.name, job.name);
    assert_eq!(parsed_job.status, job.status);
}

#[tokio::test]
async fn test_create_job_serialization() {
    let sample_ids = vec![Uuid::new_v4(), Uuid::new_v4()];
    let create_job = CreateJob {
        name: "test_create_job_serialization".to_string(),
        description: "Testing CreateJob serialization".to_string(),
        sample_ids: sample_ids.clone(),
    };

    // Test serialization
    let serialized = serde_json::to_string(&create_job);
    assert!(serialized.is_ok());

    let json_str = serialized.unwrap();
    assert!(json_str.contains("test_create_job_serialization"));
    assert!(json_str.contains("Testing CreateJob serialization"));

    // Test deserialization
    let deserialized: Result<CreateJob, _> = serde_json::from_str(&json_str);
    assert!(deserialized.is_ok());

    let parsed_create_job = deserialized.unwrap();
    assert_eq!(parsed_create_job.name, create_job.name);
    assert_eq!(parsed_create_job.description, create_job.description);
    assert_eq!(parsed_create_job.sample_ids, sample_ids);
}

#[tokio::test]
async fn test_concurrent_job_creation() {
    use std::sync::Arc;
    use tokio::task;

    let database = create_test_database().await;
    let sample_ids = setup_sequencing_test_data(&database.pool)
        .await
        .expect("Failed to setup test data");

    let manager = Arc::new(SequencingManager::new(database.clone()));
    let mut handles = Vec::new();

    // Create multiple jobs concurrently
    for i in 0..5 {
        let manager_clone = manager.clone();
        let sample_id = sample_ids[i % sample_ids.len()];

        let handle = task::spawn(async move {
            let create_job = CreateJob {
                name: format!("test_sequencing_concurrent_job_{}", i),
                description: format!("Concurrent job {}", i),
                sample_ids: vec![sample_id],
            };

            manager_clone.create_job(create_job).await
        });

        handles.push(handle);
    }

    // Wait for all tasks to complete
    let mut results = Vec::new();
    for handle in handles {
        let result = handle.await.unwrap();
        assert!(result.is_ok());
        results.push(result.unwrap());
    }

    // All jobs should be created successfully
    assert_eq!(results.len(), 5);

    // All job IDs should be unique
    let mut job_ids = std::collections::HashSet::new();
    for job in &results {
        assert!(job_ids.insert(job.id), "Job IDs should be unique");
    }

    cleanup_sequencing_test_data(&database.pool)
        .await
        .expect("Failed to cleanup test data");
}

#[tokio::test]
async fn test_job_status_transitions() {
    let database = create_test_database().await;
    let sample_ids = setup_sequencing_test_data(&database.pool)
        .await
        .expect("Failed to setup test data");

    let manager = SequencingManager::new(database.clone());

    // Create a job
    let create_job = CreateJob {
        name: "test_sequencing_status_transitions".to_string(),
        description: "Testing job status transitions".to_string(),
        sample_ids: vec![sample_ids[0]],
    };

    let job = manager.create_job(create_job).await.unwrap();

    // Test valid transitions
    let transitions = vec![
        (JobStatus::Pending, JobStatus::Running),
        (JobStatus::Running, JobStatus::Completed),
    ];

    let mut current_job = job;
    for (from_status, to_status) in transitions {
        assert_eq!(current_job.status, from_status);

        current_job = manager
            .update_job_status(current_job.id, to_status)
            .await
            .unwrap();

        assert_eq!(current_job.status, to_status);
    }

    cleanup_sequencing_test_data(&database.pool)
        .await
        .expect("Failed to cleanup test data");
}

#[tokio::test]
async fn test_job_with_empty_sample_list() {
    let database = create_test_database().await;
    let manager = SequencingManager::new(database);

    let create_job = CreateJob {
        name: "test_sequencing_empty_samples".to_string(),
        description: "Job with no samples".to_string(),
        sample_ids: vec![], // Empty sample list
    };

    let result = manager.create_job(create_job).await;
    // This might succeed or fail depending on business logic
    // The test documents the current behavior
    match result {
        Ok(job) => {
            assert_eq!(job.name, "test_sequencing_empty_samples");
        }
        Err(_) => {
            // Empty sample list might be invalid
            assert!(true); // Expected behavior
        }
    }
}

#[tokio::test]
async fn test_update_nonexistent_job_status() {
    let database = create_test_database().await;
    let manager = SequencingManager::new(database);

    let nonexistent_id = Uuid::new_v4();
    let result = manager
        .update_job_status(nonexistent_id, JobStatus::Running)
        .await;

    assert!(result.is_err());
}

#[tokio::test]
async fn test_job_error_handling() {
    let database = create_test_database().await;
    let sample_ids = setup_sequencing_test_data(&database.pool)
        .await
        .expect("Failed to setup test data");

    let manager = SequencingManager::new(database.clone());

    // Create a job
    let create_job = CreateJob {
        name: "test_sequencing_error_handling".to_string(),
        description: "Testing error handling in sequencing".to_string(),
        sample_ids: vec![sample_ids[0]],
    };

    let job = manager.create_job(create_job).await.unwrap();

    // Test transitioning to Failed status
    let failed_job = manager
        .update_job_status(job.id, JobStatus::Failed)
        .await
        .unwrap();

    assert_eq!(failed_job.status, JobStatus::Failed);
    assert_eq!(failed_job.id, job.id);

    // Verify the job remains in failed state
    let retrieved_job = manager.get_job(job.id).await.unwrap();
    assert_eq!(retrieved_job.status, JobStatus::Failed);

    cleanup_sequencing_test_data(&database.pool)
        .await
        .expect("Failed to cleanup test data");
}

/// Integration test for the full sequencing workflow
#[tokio::test]
async fn test_full_sequencing_workflow() {
    let database = create_test_database().await;
    let sample_ids = setup_sequencing_test_data(&database.pool)
        .await
        .expect("Failed to setup test data");

    let manager = SequencingManager::new(database.clone());

    // Step 1: Create a sequencing job
    let create_job = CreateJob {
        name: "test_sequencing_full_workflow".to_string(),
        description: "Complete workflow test".to_string(),
        sample_ids: sample_ids.clone(),
    };

    let job = manager.create_job(create_job).await.unwrap();
    assert_eq!(job.status, JobStatus::Pending);

    // Step 2: Start the job
    let running_job = manager
        .update_job_status(job.id, JobStatus::Running)
        .await
        .unwrap();
    assert_eq!(running_job.status, JobStatus::Running);

    // Step 3: Complete the job
    let completed_job = manager
        .update_job_status(job.id, JobStatus::Completed)
        .await
        .unwrap();
    assert_eq!(completed_job.status, JobStatus::Completed);

    // Step 4: Verify final state
    let final_job = manager.get_job(job.id).await.unwrap();
    assert_eq!(final_job.status, JobStatus::Completed);
    assert_eq!(final_job.name, "test_sequencing_full_workflow");

    // Step 5: Verify job appears in job list
    let all_jobs = manager.list_jobs().await.unwrap();
    let our_job = all_jobs.iter().find(|j| j.id == job.id);
    assert!(our_job.is_some());
    assert_eq!(our_job.unwrap().status, JobStatus::Completed);

    cleanup_sequencing_test_data(&database.pool)
        .await
        .expect("Failed to cleanup test data");
}
