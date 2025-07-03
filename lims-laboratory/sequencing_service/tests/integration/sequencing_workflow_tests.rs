use crate::test_utils::*;
use sequencing_service::{
    models::*,
    handlers::*,
    services::*,
    create_app,
};
use axum_test::TestServer;
use serde_json::json;
use uuid::Uuid;

/// Integration tests for complete sequencing workflows
#[tokio::test]
async fn test_complete_sequencing_pipeline() {
    let mut test_db = TestDatabase::new().await;
    let app = create_app().await;
    let client = SequencingTestClient::new(app);

    // Phase 1: Create sequencing workflow
    let workflow_request = SequencingFactory::create_valid_workflow_request();
    let workflow_name = workflow_request.name.clone();
    
    let response = client.post_json("/api/sequencing/workflows", &workflow_request).await;
    SequencingAssertions::assert_status_code(response.status_code(), axum::http::StatusCode::CREATED);
    
    let workflow_data: serde_json::Value = response.json();
    SequencingAssertions::assert_workflow_data(&workflow_data, &workflow_name);
    
    let workflow_id = Uuid::parse_str(workflow_data["data"]["id"].as_str().unwrap()).unwrap();
    test_db.track_workflow(workflow_id);

    // Phase 2: Create sequencing job
    let mut job_request = SequencingFactory::create_valid_job_request();
    job_request.workflow_id = workflow_id;
    let job_name = job_request.name.clone();
    
    let response = client.post_json("/api/sequencing/jobs", &job_request).await;
    SequencingAssertions::assert_status_code(response.status_code(), axum::http::StatusCode::CREATED);
    
    let job_data: serde_json::Value = response.json();
    SequencingAssertions::assert_job_data(&job_data, "Pending");
    
    let job_id = Uuid::parse_str(job_data["data"]["id"].as_str().unwrap()).unwrap();
    test_db.track_job(job_id);

    // Phase 3: Upload input files
    let (temp_dir, fastq_path) = FileTestUtils::create_temp_fastq_file(1000);
    let fastq_content = std::fs::read(&fastq_path).unwrap();
    
    let upload_files = vec![("input.fastq", fastq_content)];
    let response = client.post_multipart("/api/sequencing/files/upload", upload_files).await;
    SequencingAssertions::assert_status_code(response.status_code(), axum::http::StatusCode::CREATED);
    
    let upload_data: serde_json::Value = response.json();
    SequencingAssertions::assert_file_upload(&upload_data);

    // Phase 4: Start job execution
    let start_request = json!({
        "job_id": job_id,
        "input_files": ["input.fastq"],
        "parameters": {
            "quality_threshold": 30,
            "output_format": "BAM"
        }
    });
    
    let response = client.post_json("/api/sequencing/jobs/start", &start_request).await;
    SequencingAssertions::assert_status_code(response.status_code(), axum::http::StatusCode::OK);
    
    let start_data: serde_json::Value = response.json();
    assert_eq!(start_data["success"], true);
    assert_eq!(start_data["data"]["status"], "Running");

    // Phase 5: Monitor job progress
    let mut attempts = 0;
    let max_attempts = 30; // 30 seconds timeout
    
    loop {
        tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
        
        let response = client.get(&format!("/api/sequencing/jobs/{}", job_id)).await;
        let job_status: serde_json::Value = response.json();
        
        let status = job_status["data"]["status"].as_str().unwrap();
        
        if status == "Completed" || status == "Failed" {
            assert_eq!(status, "Completed", "Job should complete successfully");
            break;
        }
        
        attempts += 1;
        if attempts >= max_attempts {
            panic!("Job did not complete within timeout");
        }
    }

    // Phase 6: Verify output files
    let response = client.get(&format!("/api/sequencing/jobs/{}/results", job_id)).await;
    SequencingAssertions::assert_status_code(response.status_code(), axum::http::StatusCode::OK);
    
    let results_data: serde_json::Value = response.json();
    assert_eq!(results_data["success"], true);
    assert!(results_data["data"]["output_files"].is_array());
    
    let output_files = results_data["data"]["output_files"].as_array().unwrap();
    assert!(!output_files.is_empty(), "Should have output files");

    // Phase 7: Quality control analysis
    let response = client.get(&format!("/api/sequencing/jobs/{}/quality-metrics", job_id)).await;
    SequencingAssertions::assert_status_code(response.status_code(), axum::http::StatusCode::OK);
    
    let qc_data: serde_json::Value = response.json();
    SequencingAssertions::assert_quality_metrics(&qc_data);

    // Cleanup
    drop(temp_dir);
    test_db.cleanup().await;
}

#[tokio::test]
async fn test_sequencing_run_management() {
    let mut test_db = TestDatabase::new().await;
    let app = create_app().await;
    let client = SequencingTestClient::new(app);

    // Create sequencing run
    let run_request = SequencingFactory::create_valid_run_request();
    let run_name = run_request.run_name.clone();
    
    let response = client.post_json("/api/sequencing/runs", &run_request).await;
    SequencingAssertions::assert_status_code(response.status_code(), axum::http::StatusCode::CREATED);
    
    let run_data: serde_json::Value = response.json();
    SequencingAssertions::assert_run_data(&run_data, "NovaSeq6000");
    
    let run_id = Uuid::parse_str(run_data["data"]["id"].as_str().unwrap()).unwrap();
    test_db.track_run(run_id);

    // Start the run
    let start_request = json!({
        "run_id": run_id,
        "operator_notes": "Starting automated test run"
    });
    
    let response = client.post_json("/api/sequencing/runs/start", &start_request).await;
    SequencingAssertions::assert_status_code(response.status_code(), axum::http::StatusCode::OK);

    // Check run status
    let response = client.get(&format!("/api/sequencing/runs/{}", run_id)).await;
    let status_data: serde_json::Value = response.json();
    assert_eq!(status_data["data"]["status"], "Running");

    // Simulate run completion
    let complete_request = json!({
        "run_id": run_id,
        "status": "Completed",
        "completion_notes": "Run completed successfully",
        "metrics": {
            "total_reads": 50000000,
            "quality_score": 35.2,
            "cluster_density": 850000
        }
    });
    
    let response = client.post_json("/api/sequencing/runs/complete", &complete_request).await;
    SequencingAssertions::assert_status_code(response.status_code(), axum::http::StatusCode::OK);

    // Verify final status
    let response = client.get(&format!("/api/sequencing/runs/{}", run_id)).await;
    let final_data: serde_json::Value = response.json();
    assert_eq!(final_data["data"]["status"], "Completed");
    assert!(final_data["data"]["completion_time"].is_string());

    test_db.cleanup().await;
}

#[tokio::test]
async fn test_bioinformatics_analysis_pipeline() {
    let mut test_db = TestDatabase::new().await;
    let app = create_app().await;
    let client = SequencingTestClient::new(app);

    // Create analysis workflow
    let mut workflow_request = SequencingFactory::create_valid_workflow_request();
    workflow_request.workflow_type = WorkflowType::WholeGenome;
    workflow_request.steps = vec![
        WorkflowStep {
            step_number: 1,
            name: "Quality Control".to_string(),
            command: "fastqc --threads 4 input.fastq".to_string(),
            input_files: vec!["input.fastq".to_string()],
            output_files: vec!["qc_report.html".to_string()],
            parameters: json!({"quality_threshold": 30}),
            estimated_duration_minutes: 15,
        },
        WorkflowStep {
            step_number: 2,
            name: "Alignment".to_string(),
            command: "bwa mem -t 8 reference.fa input.fastq | samtools sort -o aligned.bam".to_string(),
            input_files: vec!["input.fastq".to_string(), "reference.fa".to_string()],
            output_files: vec!["aligned.bam".to_string()],
            parameters: json!({"min_mapq": 20}),
            estimated_duration_minutes: 120,
        },
        WorkflowStep {
            step_number: 3,
            name: "Variant Calling".to_string(),
            command: "gatk HaplotypeCaller -R reference.fa -I aligned.bam -O variants.vcf".to_string(),
            input_files: vec!["aligned.bam".to_string(), "reference.fa".to_string()],
            output_files: vec!["variants.vcf".to_string()],
            parameters: json!({"min_base_quality": 20}),
            estimated_duration_minutes: 90,
        },
    ];
    
    let response = client.post_json("/api/sequencing/workflows", &workflow_request).await;
    let workflow_data: serde_json::Value = response.json();
    let workflow_id = Uuid::parse_str(workflow_data["data"]["id"].as_str().unwrap()).unwrap();
    test_db.track_workflow(workflow_id);

    // Create and start analysis job
    let mut job_request = SequencingFactory::create_valid_job_request();
    job_request.workflow_id = workflow_id;
    job_request.priority = JobPriority::High;
    
    let response = client.post_json("/api/sequencing/jobs", &job_request).await;
    let job_data: serde_json::Value = response.json();
    let job_id = Uuid::parse_str(job_data["data"]["id"].as_str().unwrap()).unwrap();
    test_db.track_job(job_id);

    // Upload input files
    let (temp_dir, fastq_path) = FileTestUtils::create_temp_fastq_file(500);
    let fastq_content = std::fs::read(&fastq_path).unwrap();
    
    // Validate FASTQ quality
    let fastq_string = String::from_utf8(fastq_content.clone()).unwrap();
    FileTestUtils::assert_fastq_quality(&fastq_string, 20);
    
    let upload_files = vec![("input.fastq", fastq_content)];
    let response = client.post_multipart("/api/sequencing/files/upload", upload_files).await;
    SequencingAssertions::assert_file_upload(&response.json());

    // Start job with step-by-step execution
    let start_request = json!({
        "job_id": job_id,
        "execution_mode": "step_by_step",
        "input_files": ["input.fastq"],
        "parameters": {
            "quality_threshold": 30,
            "min_mapq": 20,
            "min_base_quality": 20
        }
    });
    
    let response = client.post_json("/api/sequencing/jobs/start", &start_request).await;
    SequencingAssertions::assert_status_code(response.status_code(), axum::http::StatusCode::OK);

    // Monitor each step completion
    for step_number in 1..=3 {
        // Wait for step to complete
        loop {
            tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;
            
            let response = client.get(&format!("/api/sequencing/jobs/{}/steps/{}", job_id, step_number)).await;
            let step_data: serde_json::Value = response.json();
            
            let status = step_data["data"]["status"].as_str().unwrap();
            if status == "Completed" || status == "Failed" {
                assert_eq!(status, "Completed", "Step {} should complete successfully", step_number);
                break;
            }
        }
        
        // Verify step output
        let response = client.get(&format!("/api/sequencing/jobs/{}/steps/{}/output", job_id, step_number)).await;
        SequencingAssertions::assert_status_code(response.status_code(), axum::http::StatusCode::OK);
        
        let output_data: serde_json::Value = response.json();
        assert!(output_data["data"]["output_files"].is_array());
        assert!(!output_data["data"]["output_files"].as_array().unwrap().is_empty());
    }

    // Verify final results and analysis
    let response = client.get(&format!("/api/sequencing/jobs/{}/analysis", job_id)).await;
    SequencingAssertions::assert_status_code(response.status_code(), axum::http::StatusCode::OK);
    
    let analysis_data: serde_json::Value = response.json();
    BioinformaticsTestUtils::assert_alignment_statistics(&analysis_data["data"]["alignment_stats"]);
    BioinformaticsTestUtils::assert_variant_calling_results(&analysis_data["data"]["variant_stats"]);

    // Cleanup
    drop(temp_dir);
    test_db.cleanup().await;
}

#[tokio::test]
async fn test_concurrent_sequencing_jobs() {
    let mut test_db = TestDatabase::new().await;
    let app = create_app().await;
    let client = SequencingTestClient::new(app);

    // Create workflow
    let workflow_request = SequencingFactory::create_valid_workflow_request();
    let response = client.post_json("/api/sequencing/workflows", &workflow_request).await;
    let workflow_data: serde_json::Value = response.json();
    let workflow_id = Uuid::parse_str(workflow_data["data"]["id"].as_str().unwrap()).unwrap();
    test_db.track_workflow(workflow_id);

    // Test concurrent job submissions
    let concurrent_results = SequencingPerformanceUtils::concurrent_job_submissions(
        &client,
        workflow_id,
        10,
    ).await;
    
    let successful_jobs = concurrent_results.iter()
        .filter(|&status| *status == axum::http::StatusCode::CREATED)
        .count();
    
    assert!(successful_jobs >= 8, "At least 80% of concurrent job submissions should succeed");

    // Test file upload throughput
    let upload_duration = SequencingPerformanceUtils::file_upload_throughput(
        &client,
        5,  // 5 files
        100, // 100KB each
    ).await;
    
    assert!(upload_duration.as_secs() < 30, "File uploads should complete within 30 seconds");

    test_db.cleanup().await;
}

#[tokio::test]
async fn test_sequencing_data_validation() {
    let mut test_db = TestDatabase::new().await;
    let app = create_app().await;
    let client = SequencingTestClient::new(app);

    // Create workflow with strict validation
    let mut workflow_request = SequencingFactory::create_valid_workflow_request();
    workflow_request.steps[0].parameters = json!({
        "quality_threshold": 35, // High quality threshold
        "min_read_length": 50,
        "max_n_content": 0.05
    });
    
    let response = client.post_json("/api/sequencing/workflows", &workflow_request).await;
    let workflow_data: serde_json::Value = response.json();
    let workflow_id = Uuid::parse_str(workflow_data["data"]["id"].as_str().unwrap()).unwrap();
    test_db.track_workflow(workflow_id);

    // Test with high-quality FASTQ data
    let (temp_dir1, fastq_path1) = FileTestUtils::create_temp_fastq_file(100);
    let high_quality_content = std::fs::read(&fastq_path1).unwrap();
    
    let upload_files = vec![("high_quality.fastq", high_quality_content)];
    let response = client.post_multipart("/api/sequencing/files/upload", upload_files).await;
    SequencingAssertions::assert_status_code(response.status_code(), axum::http::StatusCode::CREATED);

    // Test with low-quality data (should fail validation)
    let low_quality_fastq = SequencingTestDataGenerator::generate_fastq_content(50);
    let low_quality_bytes = low_quality_fastq.replace('I', '!').into_bytes(); // Replace high quality with low quality
    
    let upload_files = vec![("low_quality.fastq", low_quality_bytes)];
    let response = client.post_multipart("/api/sequencing/files/upload", upload_files).await;
    // Note: Upload might succeed, but validation during job execution should fail
    
    // Create job with high-quality data
    let mut job_request = SequencingFactory::create_valid_job_request();
    job_request.workflow_id = workflow_id;
    
    let response = client.post_json("/api/sequencing/jobs", &job_request).await;
    let job_data: serde_json::Value = response.json();
    let job_id = Uuid::parse_str(job_data["data"]["id"].as_str().unwrap()).unwrap();
    test_db.track_job(job_id);

    // Start job with validation
    let start_request = json!({
        "job_id": job_id,
        "input_files": ["high_quality.fastq"],
        "validate_inputs": true
    });
    
    let response = client.post_json("/api/sequencing/jobs/start", &start_request).await;
    SequencingAssertions::assert_status_code(response.status_code(), axum::http::StatusCode::OK);

    // Wait for validation to complete
    tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;
    
    let response = client.get(&format!("/api/sequencing/jobs/{}/validation", job_id)).await;
    SequencingAssertions::assert_status_code(response.status_code(), axum::http::StatusCode::OK);
    
    let validation_data: serde_json::Value = response.json();
    assert_eq!(validation_data["data"]["validation_passed"], true);
    assert!(validation_data["data"]["quality_metrics"].is_object());

    // Cleanup
    drop(temp_dir1);
    test_db.cleanup().await;
}