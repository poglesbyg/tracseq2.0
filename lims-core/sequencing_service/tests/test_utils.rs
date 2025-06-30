use sequencing_service::{models::*, Config, SequencingService};
use axum::{http::StatusCode, Router};
use axum_test::TestServer;
use fake::{Fake, Faker};
use serde_json::Value;
use sqlx::PgPool;
use std::sync::Arc;
use uuid::Uuid;
use tempfile::TempDir;
use std::path::PathBuf;

/// Test database manager for isolated sequencing testing
pub struct TestDatabase {
    pub pool: PgPool,
    pub cleanup_workflows: Vec<Uuid>,
    pub cleanup_jobs: Vec<Uuid>,
    pub cleanup_runs: Vec<Uuid>,
    pub cleanup_samples: Vec<Uuid>,
}

impl TestDatabase {
    pub async fn new() -> Self {
        let pool = get_test_db().await.clone();
        Self {
            pool,
            cleanup_workflows: Vec::new(),
            cleanup_jobs: Vec::new(),
            cleanup_runs: Vec::new(),
            cleanup_samples: Vec::new(),
        }
    }

    pub async fn cleanup(&mut self) {
        // Clean up in reverse dependency order
        for run_id in &self.cleanup_runs {
            let _ = sqlx::query("DELETE FROM sequencing_run_files WHERE run_id = $1")
                .bind(run_id)
                .execute(&self.pool)
                .await;
            let _ = sqlx::query("DELETE FROM sequencing_runs WHERE id = $1")
                .bind(run_id)
                .execute(&self.pool)
                .await;
        }

        for job_id in &self.cleanup_jobs {
            let _ = sqlx::query("DELETE FROM job_logs WHERE job_id = $1")
                .bind(job_id)
                .execute(&self.pool)
                .await;
            let _ = sqlx::query("DELETE FROM sequencing_jobs WHERE id = $1")
                .bind(job_id)
                .execute(&self.pool)
                .await;
        }

        for workflow_id in &self.cleanup_workflows {
            let _ = sqlx::query("DELETE FROM workflow_steps WHERE workflow_id = $1")
                .bind(workflow_id)
                .execute(&self.pool)
                .await;
            let _ = sqlx::query("DELETE FROM sequencing_workflows WHERE id = $1")
                .bind(workflow_id)
                .execute(&self.pool)
                .await;
        }

        for sample_id in &self.cleanup_samples {
            let _ = sqlx::query("DELETE FROM sample_sequencing_requests WHERE sample_id = $1")
                .bind(sample_id)
                .execute(&self.pool)
                .await;
        }

        self.cleanup_workflows.clear();
        self.cleanup_jobs.clear();
        self.cleanup_runs.clear();
        self.cleanup_samples.clear();
    }

    pub fn track_workflow(&mut self, workflow_id: Uuid) {
        self.cleanup_workflows.push(workflow_id);
    }

    pub fn track_job(&mut self, job_id: Uuid) {
        self.cleanup_jobs.push(job_id);
    }

    pub fn track_run(&mut self, run_id: Uuid) {
        self.cleanup_runs.push(run_id);
    }

    pub fn track_sample(&mut self, sample_id: Uuid) {
        self.cleanup_samples.push(sample_id);
    }
}

/// Factory for creating test sequencing entities
pub struct SequencingFactory;

impl SequencingFactory {
    pub fn create_valid_workflow_request() -> CreateWorkflowRequest {
        CreateWorkflowRequest {
            name: format!("Test Workflow {}", Faker.fake::<String>()),
            description: Some("Automated test workflow for sequencing".to_string()),
            workflow_type: WorkflowType::DNASeq,
            protocol: SequencingProtocol::Illumina,
            steps: vec![
                WorkflowStep {
                    step_number: 1,
                    name: "Quality Control".to_string(),
                    command: "fastqc --outdir results/ input.fastq".to_string(),
                    input_files: vec!["input.fastq".to_string()],
                    output_files: vec!["results/fastqc_report.html".to_string()],
                    parameters: serde_json::json!({
                        "quality_threshold": 30,
                        "adapter_trimming": true
                    }),
                    estimated_duration_minutes: 30,
                },
                WorkflowStep {
                    step_number: 2,
                    name: "Alignment".to_string(),
                    command: "bwa mem reference.fa input.fastq > aligned.sam".to_string(),
                    input_files: vec!["input.fastq".to_string(), "reference.fa".to_string()],
                    output_files: vec!["aligned.sam".to_string()],
                    parameters: serde_json::json!({
                        "min_seed_length": 19,
                        "match_score": 1,
                        "mismatch_penalty": 4
                    }),
                    estimated_duration_minutes: 120,
                },
            ],
            resource_requirements: ResourceRequirements {
                cpu_cores: 4,
                memory_gb: 16,
                storage_gb: 100,
                gpu_required: false,
            },
            is_active: true,
        }
    }

    pub fn create_valid_job_request() -> CreateJobRequest {
        CreateJobRequest {
            workflow_id: Uuid::new_v4(), // Will be replaced with real workflow
            sample_ids: vec![Uuid::new_v4()], // Will be replaced with real samples
            name: format!("Test Job {}", Faker.fake::<String>()),
            priority: JobPriority::Normal,
            parameters: serde_json::json!({
                "quality_threshold": 30,
                "output_format": "BAM",
                "compression_level": 6
            }),
            expected_completion: Some(chrono::Utc::now() + chrono::Duration::hours(6)),
            requester_id: Uuid::new_v4(),
            notes: Some("Automated test job".to_string()),
        }
    }

    pub fn create_valid_run_request() -> CreateRunRequest {
        CreateRunRequest {
            run_name: format!("Run_{}", chrono::Utc::now().format("%Y%m%d_%H%M%S")),
            instrument: Instrument::NovaSeq6000,
            flow_cell_id: Self::generate_flow_cell_id(),
            lane_count: 4,
            read_length: ReadLength::PE150,
            chemistry: Chemistry::V1_5,
            planned_samples: vec![
                PlannedSample {
                    sample_id: Uuid::new_v4(),
                    lane: 1,
                    index_sequence: "ATCGATCG".to_string(),
                    library_prep: LibraryPrep::TruSeq,
                    target_coverage: Some(30.0),
                },
            ],
            estimated_start: chrono::Utc::now() + chrono::Duration::hours(1),
            estimated_duration_hours: 24,
            operator_id: Uuid::new_v4(),
        }
    }

    pub fn create_sequencing_request() -> SequencingRequest {
        SequencingRequest {
            sample_id: Uuid::new_v4(),
            sequencing_type: SequencingType::WholeGenome,
            read_length: ReadLength::PE150,
            target_coverage: 30.0,
            library_prep: LibraryPrep::TruSeq,
            index_sequence: Some("ATCGATCG".to_string()),
            priority: RequestPriority::Standard,
            special_instructions: Some("Handle with care".to_string()),
            requested_by: Uuid::new_v4(),
            project_code: Some("PROJ-2024-001".to_string()),
            billing_code: Some("BILL-LAB-001".to_string()),
        }
    }

    pub fn generate_flow_cell_id() -> String {
        format!("FC{:08}", fastrand::u32(10000000..99999999))
    }

    pub fn generate_run_id() -> String {
        format!("RUN_{:06}", fastrand::u32(100000..999999))
    }

    pub async fn create_test_workflow(sequencing_service: &SequencingService) -> Workflow {
        let request = Self::create_valid_workflow_request();
        sequencing_service.create_workflow(request).await
            .expect("Failed to create test workflow")
    }

    pub async fn create_test_job(sequencing_service: &SequencingService, workflow_id: Uuid) -> SequencingJob {
        let mut request = Self::create_valid_job_request();
        request.workflow_id = workflow_id;
        sequencing_service.create_job(request).await
            .expect("Failed to create test job")
    }

    pub async fn create_test_run(sequencing_service: &SequencingService) -> SequencingRun {
        let request = Self::create_valid_run_request();
        sequencing_service.create_run(request).await
            .expect("Failed to create test run")
    }
}

/// HTTP test client wrapper for sequencing API testing
pub struct SequencingTestClient {
    pub server: TestServer,
    pub auth_token: Option<String>,
}

impl SequencingTestClient {
    pub fn new(app: Router) -> Self {
        let server = TestServer::new(app).unwrap();
        Self {
            server,
            auth_token: None,
        }
    }

    pub fn with_auth_token(mut self, token: String) -> Self {
        self.auth_token = Some(token);
        self
    }

    pub async fn post_json<T: serde::Serialize>(&self, path: &str, body: &T) -> axum_test::TestResponse {
        let mut request = self.server.post(path).json(body);
        if let Some(token) = &self.auth_token {
            request = request.add_header("Authorization", format!("Bearer {}", token).parse().unwrap());
        }
        request.await
    }

    pub async fn get(&self, path: &str) -> axum_test::TestResponse {
        let mut request = self.server.get(path);
        if let Some(token) = &self.auth_token {
            request = request.add_header("Authorization", format!("Bearer {}", token).parse().unwrap());
        }
        request.await
    }

    pub async fn put_json<T: serde::Serialize>(&self, path: &str, body: &T) -> axum_test::TestResponse {
        let mut request = self.server.put(path).json(body);
        if let Some(token) = &self.auth_token {
            request = request.add_header("Authorization", format!("Bearer {}", token).parse().unwrap());
        }
        request.await
    }

    pub async fn delete(&self, path: &str) -> axum_test::TestResponse {
        let mut request = self.server.delete(path);
        if let Some(token) = &self.auth_token {
            request = request.add_header("Authorization", format!("Bearer {}", token).parse().unwrap());
        }
        request.await
    }

    pub async fn post_multipart(&self, path: &str, files: Vec<(&str, Vec<u8>)>) -> axum_test::TestResponse {
        let mut request = self.server.post(path);
        
        // Add multipart data
        for (filename, content) in files {
            request = request.add_part(filename, content);
        }

        if let Some(token) = &self.auth_token {
            request = request.add_header("Authorization", format!("Bearer {}", token).parse().unwrap());
        }
        request.await
    }
}

/// Common assertions for sequencing testing
pub struct SequencingAssertions;

impl SequencingAssertions {
    pub fn assert_successful_creation(response: &Value) {
        assert_eq!(response["success"], true);
        assert!(response["data"]["id"].is_string());
        assert!(response["data"]["created_at"].is_string());
    }

    pub fn assert_workflow_data(response: &Value, expected_name: &str) {
        assert_eq!(response["success"], true);
        assert_eq!(response["data"]["name"], expected_name);
        assert!(response["data"]["id"].is_string());
        assert!(response["data"]["steps"].is_array());
        assert!(response["data"]["resource_requirements"].is_object());
    }

    pub fn assert_job_data(response: &Value, expected_status: &str) {
        assert_eq!(response["success"], true);
        assert_eq!(response["data"]["status"], expected_status);
        assert!(response["data"]["id"].is_string());
        assert!(response["data"]["workflow_id"].is_string());
    }

    pub fn assert_run_data(response: &Value, expected_instrument: &str) {
        assert_eq!(response["success"], true);
        assert_eq!(response["data"]["instrument"], expected_instrument);
        assert!(response["data"]["id"].is_string());
        assert!(response["data"]["flow_cell_id"].is_string());
    }

    pub fn assert_file_upload(response: &Value) {
        assert_eq!(response["success"], true);
        assert!(response["data"]["file_id"].is_string());
        assert!(response["data"]["file_path"].is_string());
        assert!(response["data"]["file_size"].is_number());
        assert!(response["data"]["checksum"].is_string());
    }

    pub fn assert_quality_metrics(response: &Value) {
        assert_eq!(response["success"], true);
        assert!(response["data"]["quality_score"].is_number());
        assert!(response["data"]["read_count"].is_number());
        assert!(response["data"]["base_count"].is_number());
        assert!(response["data"]["metrics"].is_object());
    }

    pub fn assert_validation_error(response: &Value) {
        assert_eq!(response["success"], false);
        assert!(response["error"].is_string());
    }

    pub fn assert_status_code(status: StatusCode, expected: StatusCode) {
        assert_eq!(status, expected);
    }
}

/// Test data generators for various sequencing scenarios
pub struct SequencingTestDataGenerator;

impl SequencingTestDataGenerator {
    pub fn workflow_types() -> Vec<WorkflowType> {
        vec![
            WorkflowType::DNASeq,
            WorkflowType::RNASeq,
            WorkflowType::ChIPSeq,
            WorkflowType::ATACSeq,
            WorkflowType::WholeGenome,
        ]
    }

    pub fn sequencing_protocols() -> Vec<SequencingProtocol> {
        vec![
            SequencingProtocol::Illumina,
            SequencingProtocol::PacBio,
            SequencingProtocol::Nanopore,
            SequencingProtocol::IonTorrent,
        ]
    }

    pub fn instruments() -> Vec<Instrument> {
        vec![
            Instrument::NovaSeq6000,
            Instrument::NextSeq2000,
            Instrument::MiSeq,
            Instrument::HiSeq4000,
            Instrument::iSeq100,
        ]
    }

    pub fn read_lengths() -> Vec<ReadLength> {
        vec![
            ReadLength::SR50,
            ReadLength::SR75,
            ReadLength::PE100,
            ReadLength::PE150,
            ReadLength::PE250,
        ]
    }

    pub fn library_preps() -> Vec<LibraryPrep> {
        vec![
            LibraryPrep::TruSeq,
            LibraryPrep::Nextera,
            LibraryPrep::KAPA,
            LibraryPrep::Custom,
        ]
    }

    pub fn job_priorities() -> Vec<JobPriority> {
        vec![
            JobPriority::Low,
            JobPriority::Normal,
            JobPriority::High,
            JobPriority::Urgent,
        ]
    }

    pub fn generate_fastq_content(read_count: usize) -> String {
        let mut content = String::new();
        for i in 0..read_count {
            content.push_str(&format!("@READ_{}\n", i + 1));
            content.push_str("ATCGATCGATCGATCGATCGATCGATCGATCG\n");
            content.push_str("+\n");
            content.push_str("IIIIIIIIIIIIIIIIIIIIIIIIIIIIIIII\n");
        }
        content
    }

    pub fn generate_sam_content(alignment_count: usize) -> String {
        let mut content = String::new();
        content.push_str("@HD\tVN:1.0\tSO:coordinate\n");
        content.push_str("@SQ\tSN:chr1\tLN:248956422\n");
        for i in 0..alignment_count {
            content.push_str(&format!(
                "READ_{}\t0\tchr1\t{}\t60\t32M\t*\t0\t0\tATCGATCGATCGATCGATCGATCGATCGATCG\tIIIIIIIIIIIIIIIIIIIIIIIIIIIIIIII\n",
                i + 1, (i + 1) * 100
            ));
        }
        content
    }

    pub fn invalid_workflow_steps() -> Vec<WorkflowStep> {
        vec![
            WorkflowStep {
                step_number: 0, // Invalid: step number should be >= 1
                name: "".to_string(), // Invalid: empty name
                command: "".to_string(), // Invalid: empty command
                input_files: vec![],
                output_files: vec![],
                parameters: serde_json::json!({}),
                estimated_duration_minutes: -1, // Invalid: negative duration
            },
        ]
    }

    pub fn invalid_resource_requirements() -> Vec<ResourceRequirements> {
        vec![
            ResourceRequirements {
                cpu_cores: 0, // Invalid: zero cores
                memory_gb: -1, // Invalid: negative memory
                storage_gb: 0, // Invalid: zero storage
                gpu_required: false,
            },
            ResourceRequirements {
                cpu_cores: 1000, // Invalid: too many cores
                memory_gb: 10000, // Invalid: too much memory
                storage_gb: 1000000, // Invalid: too much storage
                gpu_required: false,
            },
        ]
    }
}

/// Performance testing utilities for sequencing operations
pub struct SequencingPerformanceUtils;

impl SequencingPerformanceUtils {
    pub async fn measure_workflow_creation_time(
        client: &SequencingTestClient,
        request: &CreateWorkflowRequest,
    ) -> std::time::Duration {
        let start = std::time::Instant::now();
        let _ = client.post_json("/api/sequencing/workflows", request).await;
        start.elapsed()
    }

    pub async fn measure_job_execution_time(
        client: &SequencingTestClient,
        job_id: Uuid,
    ) -> std::time::Duration {
        let start = std::time::Instant::now();
        
        // Start job
        let _ = client.post_json(&format!("/api/sequencing/jobs/{}/start", job_id), &serde_json::json!({})).await;
        
        // Poll until completion
        loop {
            let response = client.get(&format!("/api/sequencing/jobs/{}", job_id)).await;
            let data: Value = response.json();
            
            if data["data"]["status"] == "Completed" || data["data"]["status"] == "Failed" {
                break;
            }
            
            tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
        }
        
        start.elapsed()
    }

    pub async fn concurrent_job_submissions(
        client: &SequencingTestClient,
        workflow_id: Uuid,
        concurrent_count: usize,
    ) -> Vec<StatusCode> {
        let tasks: Vec<_> = (0..concurrent_count)
            .map(|i| {
                let mut request = SequencingFactory::create_valid_job_request();
                request.workflow_id = workflow_id;
                request.name = format!("Concurrent Job {}", i);
                async move {
                    client.post_json("/api/sequencing/jobs", &request).await.status_code()
                }
            })
            .collect();

        futures::future::join_all(tasks).await
    }

    pub async fn file_upload_throughput(
        client: &SequencingTestClient,
        file_count: usize,
        file_size_kb: usize,
    ) -> std::time::Duration {
        let start = std::time::Instant::now();
        
        let content = vec![b'A'; file_size_kb * 1024];
        let tasks: Vec<_> = (0..file_count)
            .map(|i| {
                let content = content.clone();
                async move {
                    let files = vec![(format!("test_file_{}.fastq", i).as_str(), content)];
                    client.post_multipart("/api/sequencing/files/upload", files).await
                }
            })
            .collect();

        let _ = futures::future::join_all(tasks).await;
        start.elapsed()
    }
}

/// File handling testing utilities
pub struct FileTestUtils;

impl FileTestUtils {
    pub fn create_temp_fastq_file(read_count: usize) -> (TempDir, PathBuf) {
        let temp_dir = tempfile::tempdir().expect("Failed to create temp dir");
        let file_path = temp_dir.path().join("test.fastq");
        
        let content = SequencingTestDataGenerator::generate_fastq_content(read_count);
        std::fs::write(&file_path, content).expect("Failed to write test file");
        
        (temp_dir, file_path)
    }

    pub fn create_temp_sam_file(alignment_count: usize) -> (TempDir, PathBuf) {
        let temp_dir = tempfile::tempdir().expect("Failed to create temp dir");
        let file_path = temp_dir.path().join("test.sam");
        
        let content = SequencingTestDataGenerator::generate_sam_content(alignment_count);
        std::fs::write(&file_path, content).expect("Failed to write test file");
        
        (temp_dir, file_path)
    }

    pub fn assert_fastq_quality(content: &str, min_quality_score: u8) {
        let lines: Vec<&str> = content.lines().collect();
        assert_eq!(lines.len() % 4, 0, "FASTQ file should have multiple of 4 lines");
        
        for chunk in lines.chunks(4) {
            assert!(chunk[0].starts_with('@'), "Header line should start with @");
            assert!(chunk[2].starts_with('+'), "Plus line should start with +");
            
            let quality_line = chunk[3];
            for quality_char in quality_line.chars() {
                let quality_score = quality_char as u8 - 33; // Phred+33 encoding
                assert!(quality_score >= min_quality_score, 
                    "Quality score {} below minimum {}", quality_score, min_quality_score);
            }
        }
    }

    pub fn assert_sam_alignment(content: &str) {
        let lines: Vec<&str> = content.lines().collect();
        let mut header_lines = 0;
        let mut alignment_lines = 0;
        
        for line in lines {
            if line.starts_with('@') {
                header_lines += 1;
            } else if !line.trim().is_empty() {
                alignment_lines += 1;
                let fields: Vec<&str> = line.split('\t').collect();
                assert!(fields.len() >= 11, "SAM alignment should have at least 11 fields");
            }
        }
        
        assert!(header_lines > 0, "SAM file should have header lines");
        assert!(alignment_lines > 0, "SAM file should have alignment lines");
    }

    pub fn calculate_file_checksum(content: &[u8]) -> String {
        use sha2::{Sha256, Digest};
        let mut hasher = Sha256::new();
        hasher.update(content);
        format!("{:x}", hasher.finalize())
    }
}

/// Bioinformatics utilities for testing analysis results
pub struct BioinformaticsTestUtils;

impl BioinformaticsTestUtils {
    pub fn assert_quality_metrics(metrics: &Value) {
        assert!(metrics["total_reads"].is_number());
        assert!(metrics["total_bases"].is_number());
        assert!(metrics["average_quality"].is_number());
        assert!(metrics["gc_content"].is_number());
        
        let gc_content = metrics["gc_content"].as_f64().unwrap();
        assert!(gc_content >= 0.0 && gc_content <= 100.0, "GC content should be between 0 and 100");
        
        let avg_quality = metrics["average_quality"].as_f64().unwrap();
        assert!(avg_quality >= 0.0 && avg_quality <= 60.0, "Average quality should be between 0 and 60");
    }

    pub fn assert_alignment_statistics(stats: &Value) {
        assert!(stats["mapped_reads"].is_number());
        assert!(stats["unmapped_reads"].is_number());
        assert!(stats["mapping_rate"].is_number());
        assert!(stats["duplicate_rate"].is_number());
        
        let mapping_rate = stats["mapping_rate"].as_f64().unwrap();
        assert!(mapping_rate >= 0.0 && mapping_rate <= 100.0, "Mapping rate should be between 0 and 100");
        
        let duplicate_rate = stats["duplicate_rate"].as_f64().unwrap();
        assert!(duplicate_rate >= 0.0 && duplicate_rate <= 100.0, "Duplicate rate should be between 0 and 100");
    }

    pub fn generate_mock_vcf_content(variant_count: usize) -> String {
        let mut content = String::new();
        content.push_str("##fileformat=VCFv4.2\n");
        content.push_str("##contig=<ID=chr1,length=248956422>\n");
        content.push_str("#CHROM\tPOS\tID\tREF\tALT\tQUAL\tFILTER\tINFO\tFORMAT\tSAMPLE1\n");
        
        for i in 0..variant_count {
            content.push_str(&format!(
                "chr1\t{}\trs{}\tA\tT\t60\tPASS\tDP=30\tGT:DP\t0/1:30\n",
                (i + 1) * 1000, i + 1
            ));
        }
        
        content
    }

    pub fn assert_variant_calling_results(results: &Value) {
        assert!(results["total_variants"].is_number());
        assert!(results["snvs"].is_number());
        assert!(results["indels"].is_number());
        assert!(results["quality_distribution"].is_object());
        
        let total = results["total_variants"].as_u64().unwrap();
        let snvs = results["snvs"].as_u64().unwrap();
        let indels = results["indels"].as_u64().unwrap();
        
        assert_eq!(total, snvs + indels, "Total variants should equal SNVs + indels");
    }
}