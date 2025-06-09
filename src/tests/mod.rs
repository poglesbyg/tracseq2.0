use crate::sequencing::{CreateJob, JobStatus, SequencingManager};
use sqlx::postgres::PgPoolOptions;

#[cfg(test)]
mod template_tests;

#[cfg(test)]
mod rag_integration_tests;

#[cfg(test)]
mod repository_tests;

#[cfg(test)]
mod rag_integration_workflow_tests;

pub mod modular_assembly_test;

pub async fn test_sequencing_crud() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize database connection
    let pool = PgPoolOptions::new()
        .max_connections(5)
        .connect("postgres://postgres:postgres@localhost:5432/lab_manager")
        .await?;

    let manager = SequencingManager::new(pool);

    println!("Testing Sequencing Job CRUD operations...\n");

    // Test CREATE
    println!("1. Testing CREATE operation...");
    let new_job = CreateJob {
        name: "Test Sequencing Run".to_string(),
        sample_sheet_path: "/path/to/sample_sheet.csv".to_string(),
        metadata: Some(serde_json::json!({
            "instrument": "NovaSeq",
            "read_length": "2x150",
            "project": "Test Project"
        })),
    };

    let created_job = manager.create_job(new_job).await?;
    println!("Created job: {:?}\n", created_job);

    // Test READ (get single job)
    println!("2. Testing READ operation (get single job)...");
    let retrieved_job = manager.get_job(created_job.id).await?;
    println!("Retrieved job: {:?}\n", retrieved_job);

    // Test UPDATE
    println!("3. Testing UPDATE operation...");
    let updated_job = manager
        .update_job_status(created_job.id, JobStatus::InProgress)
        .await?;
    println!("Updated job status: {:?}\n", updated_job);

    // Test READ (list all jobs)
    println!("4. Testing READ operation (list all jobs)...");
    let all_jobs = manager.list_jobs().await?;
    println!("All jobs:");
    for job in &all_jobs {
        println!("- {:?}", job);
    }

    println!("\nAll CRUD operations completed successfully!");
    Ok(())
}
