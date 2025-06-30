use sample_service::{models::*, test_utils::*, Config, SampleService};
use axum::{http::StatusCode, Router};
use axum_test::TestServer;
use fake::{Fake, Faker};
use serde_json::Value;
use sqlx::PgPool;
use std::sync::Arc;
use uuid::Uuid;

/// Test database manager for isolated sample testing
pub struct TestDatabase {
    pub pool: PgPool,
    pub cleanup_samples: Vec<Uuid>,
    pub cleanup_batches: Vec<Uuid>,
}

impl TestDatabase {
    pub async fn new() -> Self {
        let pool = get_test_db().await.clone();
        Self {
            pool,
            cleanup_samples: Vec::new(),
            cleanup_batches: Vec::new(),
        }
    }

    pub async fn cleanup(&mut self) {
        // Clean up samples
        for sample_id in &self.cleanup_samples {
            let _ = sqlx::query("DELETE FROM sample_audit_log WHERE sample_id = $1")
                .bind(sample_id)
                .execute(&self.pool)
                .await;
            let _ = sqlx::query("DELETE FROM sample_status_history WHERE sample_id = $1")
                .bind(sample_id)
                .execute(&self.pool)
                .await;
            let _ = sqlx::query("DELETE FROM samples WHERE id = $1")
                .bind(sample_id)
                .execute(&self.pool)
                .await;
        }
        
        // Clean up batches
        for batch_id in &self.cleanup_batches {
            let _ = sqlx::query("DELETE FROM sample_batches WHERE id = $1")
                .bind(batch_id)
                .execute(&self.pool)
                .await;
        }
        
        self.cleanup_samples.clear();
        self.cleanup_batches.clear();
    }

    pub fn track_sample(&mut self, sample_id: Uuid) {
        self.cleanup_samples.push(sample_id);
    }

    pub fn track_batch(&mut self, batch_id: Uuid) {
        self.cleanup_batches.push(batch_id);
    }
}

/// Factory for creating test samples with realistic laboratory data
pub struct SampleFactory;

impl SampleFactory {
    pub fn create_valid_sample_request() -> CreateSampleRequest {
        CreateSampleRequest {
            name: format!("Test Sample {}", Faker.fake::<String>()),
            sample_type: SampleType::DNA,
            barcode: Self::generate_barcode(),
            volume_ml: Some(10.5),
            concentration_ng_ul: Some(250.0),
            storage_temperature: Some(StorageTemperature::Minus80),
            container_type: Some(ContainerType::Tube),
            submitter_name: Some("Test Researcher".to_string()),
            project_name: Some("Test Project".to_string()),
            notes: Some("Test sample for automated testing".to_string()),
            priority: Some(SamplePriority::Normal),
            collection_date: Some(chrono::Utc::now().date_naive()),
            expiration_date: Some(chrono::Utc::now().date_naive() + chrono::Duration::days(365)),
        }
    }

    pub fn create_invalid_sample_request() -> CreateSampleRequest {
        CreateSampleRequest {
            name: "".to_string(), // Invalid: empty name
            sample_type: SampleType::DNA,
            barcode: "INVALID".to_string(), // Invalid: doesn't match pattern
            volume_ml: Some(-1.0), // Invalid: negative volume
            concentration_ng_ul: Some(-10.0), // Invalid: negative concentration
            storage_temperature: Some(StorageTemperature::Minus80),
            container_type: Some(ContainerType::Tube),
            submitter_name: None,
            project_name: None,
            notes: None,
            priority: Some(SamplePriority::Normal),
            collection_date: None,
            expiration_date: None,
        }
    }

    pub fn create_batch_request(count: usize) -> BatchCreateSampleRequest {
        let samples = (0..count)
            .map(|i| CreateSampleRequest {
                name: format!("Batch Sample {}", i + 1),
                barcode: format!("BATCH-{:06}", i + 1),
                ..Self::create_valid_sample_request()
            })
            .collect();

        BatchCreateSampleRequest { samples }
    }

    pub fn create_update_request() -> UpdateSampleRequest {
        UpdateSampleRequest {
            name: Some("Updated Sample Name".to_string()),
            volume_ml: Some(15.0),
            concentration_ng_ul: Some(300.0),
            notes: Some("Updated notes".to_string()),
            priority: Some(SamplePriority::High),
            expiration_date: Some(chrono::Utc::now().date_naive() + chrono::Duration::days(730)),
        }
    }

    pub fn generate_barcode() -> String {
        format!("TEST-{:08}-{:06}", 
                chrono::Utc::now().format("%Y%m%d"),
                fastrand::u32(100000..999999))
    }

    pub async fn create_test_sample(sample_service: &SampleService) -> Sample {
        let request = Self::create_valid_sample_request();
        sample_service.create_sample(request).await
            .expect("Failed to create test sample")
    }

    pub async fn create_test_samples(sample_service: &SampleService, count: usize) -> Vec<Sample> {
        let mut samples = Vec::new();
        for i in 0..count {
            let mut request = Self::create_valid_sample_request();
            request.name = format!("Test Sample {}", i + 1);
            request.barcode = format!("TEST-{:06}", i + 1);
            
            let sample = sample_service.create_sample(request).await
                .expect("Failed to create test sample");
            samples.push(sample);
        }
        samples
    }
}

/// HTTP test client wrapper for sample API testing
pub struct SampleTestClient {
    pub server: TestServer,
    pub auth_token: Option<String>,
}

impl SampleTestClient {
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
}

/// Common assertions for sample testing
pub struct SampleAssertions;

impl SampleAssertions {
    pub fn assert_successful_creation(response: &Value) {
        assert_eq!(response["success"], true);
        assert!(response["data"]["id"].is_string());
        assert!(response["data"]["barcode"].is_string());
        assert_eq!(response["data"]["status"], "Pending");
    }

    pub fn assert_sample_data(response: &Value, expected_name: &str) {
        assert_eq!(response["success"], true);
        assert_eq!(response["data"]["name"], expected_name);
        assert!(response["data"]["id"].is_string());
        assert!(response["data"]["created_at"].is_string());
    }

    pub fn assert_validation_error(response: &Value) {
        assert_eq!(response["success"], false);
        assert!(response["error"].is_string());
    }

    pub fn assert_batch_response(response: &Value, expected_created: usize, expected_failed: usize) {
        assert_eq!(response["success"], true);
        assert_eq!(response["data"]["total_created"], expected_created);
        assert_eq!(response["data"]["total_failed"], expected_failed);
        assert!(response["data"]["results"].is_array());
    }

    pub fn assert_sample_list(response: &Value, min_count: usize) {
        assert_eq!(response["success"], true);
        assert!(response["data"]["samples"].is_array());
        let samples = response["data"]["samples"].as_array().unwrap();
        assert!(samples.len() >= min_count);
        assert!(response["data"]["pagination"].is_object());
    }

    pub fn assert_status_code(status: StatusCode, expected: StatusCode) {
        assert_eq!(status, expected);
    }
}

/// Test data generators for various scenarios
pub struct SampleTestDataGenerator;

impl SampleTestDataGenerator {
    pub fn sample_types() -> Vec<SampleType> {
        vec![SampleType::DNA, SampleType::RNA, SampleType::Protein, SampleType::Blood, SampleType::Tissue]
    }

    pub fn storage_temperatures() -> Vec<StorageTemperature> {
        vec![
            StorageTemperature::Minus80,
            StorageTemperature::Minus20,
            StorageTemperature::Four,
            StorageTemperature::RoomTemp,
            StorageTemperature::ThirtySeven,
        ]
    }

    pub fn container_types() -> Vec<ContainerType> {
        vec![ContainerType::Tube, ContainerType::Plate, ContainerType::Vial, ContainerType::Bag]
    }

    pub fn sample_statuses() -> Vec<SampleStatus> {
        vec![
            SampleStatus::Pending,
            SampleStatus::Validated,
            SampleStatus::InStorage,
            SampleStatus::InSequencing,
            SampleStatus::Completed,
        ]
    }

    pub fn invalid_barcodes() -> Vec<String> {
        vec![
            "".to_string(),
            "short".to_string(),
            "INVALID-FORMAT".to_string(),
            "123456789012345678901234567890123456789012345".to_string(), // Too long
            "special!@#$%".to_string(),
        ]
    }

    pub fn invalid_volumes() -> Vec<f64> {
        vec![-1.0, 0.0, -100.5, f64::NEG_INFINITY, f64::NAN]
    }

    pub fn invalid_concentrations() -> Vec<f64> {
        vec![-1.0, -100.0, f64::NEG_INFINITY, f64::NAN]
    }

    pub fn generate_large_batch(size: usize) -> BatchCreateSampleRequest {
        let samples = (0..size)
            .map(|i| CreateSampleRequest {
                name: format!("Large Batch Sample {}", i + 1),
                barcode: format!("LARGE-{:08}", i + 1),
                sample_type: Self::sample_types()[i % Self::sample_types().len()],
                storage_temperature: Some(Self::storage_temperatures()[i % Self::storage_temperatures().len()]),
                container_type: Some(Self::container_types()[i % Self::container_types().len()]),
                volume_ml: Some(10.0 + (i as f64 * 0.1)),
                concentration_ng_ul: Some(100.0 + (i as f64 * 5.0)),
                ..SampleFactory::create_valid_sample_request()
            })
            .collect();

        BatchCreateSampleRequest { samples }
    }
}

/// Performance testing utilities
pub struct SamplePerformanceUtils;

impl SamplePerformanceUtils {
    pub async fn measure_creation_time(
        client: &SampleTestClient,
        request: &CreateSampleRequest,
    ) -> std::time::Duration {
        let start = std::time::Instant::now();
        let _ = client.post_json("/api/samples", request).await;
        start.elapsed()
    }

    pub async fn measure_batch_creation_time(
        client: &SampleTestClient,
        batch_request: &BatchCreateSampleRequest,
    ) -> std::time::Duration {
        let start = std::time::Instant::now();
        let _ = client.post_json("/api/samples/batch", batch_request).await;
        start.elapsed()
    }

    pub async fn concurrent_sample_creation(
        client: &SampleTestClient,
        concurrent_count: usize,
    ) -> Vec<StatusCode> {
        let tasks: Vec<_> = (0..concurrent_count)
            .map(|i| {
                let request = CreateSampleRequest {
                    name: format!("Concurrent Sample {}", i),
                    barcode: format!("CONC-{:06}", i),
                    ..SampleFactory::create_valid_sample_request()
                };
                async move {
                    client.post_json("/api/samples", &request).await.status_code()
                }
            })
            .collect();

        futures::future::join_all(tasks).await
    }
}

/// CSV testing utilities for export functionality
pub struct CsvTestUtils;

impl CsvTestUtils {
    pub fn parse_csv_response(csv_content: &str) -> Result<Vec<Vec<String>>, csv::Error> {
        let mut reader = csv::Reader::from_reader(csv_content.as_bytes());
        let mut records = Vec::new();
        
        for result in reader.records() {
            let record = result?;
            records.push(record.iter().map(|s| s.to_string()).collect());
        }
        
        Ok(records)
    }

    pub fn assert_csv_headers(csv_content: &str, expected_headers: &[&str]) {
        let records = Self::parse_csv_response(csv_content).expect("Failed to parse CSV");
        assert!(!records.is_empty(), "CSV should have headers");
        
        let headers = &records[0];
        for expected_header in expected_headers {
            assert!(
                headers.contains(&expected_header.to_string()),
                "CSV should contain header: {}",
                expected_header
            );
        }
    }

    pub fn assert_csv_sample_data(csv_content: &str, sample: &Sample) {
        let records = Self::parse_csv_response(csv_content).expect("Failed to parse CSV");
        assert!(records.len() > 1, "CSV should have data rows");
        
        // Find the sample row by ID or barcode
        let sample_row = records.iter()
            .skip(1) // Skip header
            .find(|row| {
                row.iter().any(|cell| 
                    cell == &sample.id.to_string() || 
                    cell == &sample.barcode
                )
            });
            
        assert!(sample_row.is_some(), "CSV should contain sample data");
    }
} 
