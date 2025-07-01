use barcode_service::{
    config::{BarcodeConfig, Config},
    database::DatabasePool,
    models::*,
    service::BarcodeService,
    error::{BarcodeError, Result},
};
use sqlx::PgPool;
use std::sync::Arc;
use uuid::Uuid;

/// Test database manager for isolated test environments
pub struct TestDatabase {
    pub pool: PgPool,
    pub tracked_barcodes: Vec<String>,
}

impl TestDatabase {
    pub async fn new() -> Self {
        // Use test database URL from environment or default
        let database_url = std::env::var("TEST_DATABASE_URL")
            .unwrap_or_else(|_| "postgres://postgres:password@localhost:5432/barcode_test".to_string());
        
        let pool = PgPool::connect(&database_url)
            .await
            .expect("Failed to connect to test database");
        
        // Run migrations
        sqlx::migrate!("./migrations")
            .run(&pool)
            .await
            .expect("Failed to run migrations");
        
        Self {
            pool,
            tracked_barcodes: Vec::new(),
        }
    }

    pub async fn cleanup(&mut self) {
        for barcode in &self.tracked_barcodes {
            let _ = sqlx::query("DELETE FROM barcodes WHERE barcode = $1")
                .bind(barcode)
                .execute(&self.pool)
                .await;
        }
        self.tracked_barcodes.clear();
    }

    pub fn track_barcode(&mut self, barcode: String) {
        self.tracked_barcodes.push(barcode);
    }
}

impl Drop for TestDatabase {
    fn drop(&mut self) {
        let pool = self.pool.clone();
        let barcodes = self.tracked_barcodes.clone();
        tokio::spawn(async move {
            for barcode in barcodes {
                let _ = sqlx::query("DELETE FROM barcodes WHERE barcode = $1")
                    .bind(barcode)
                    .execute(&pool)
                    .await;
            }
        });
    }
}

/// Factory for creating test barcode configurations
pub struct BarcodeConfigFactory;

impl BarcodeConfigFactory {
    pub fn create_default() -> BarcodeConfig {
        BarcodeConfig {
            prefix: "TST".to_string(),
            min_length: 10,
            include_date: true,
            include_sequence: true,
            separator: "-".to_string(),
            validation_pattern: r"^[A-Z0-9\-]+$".to_string(),
        }
    }

    pub fn create_minimal() -> BarcodeConfig {
        BarcodeConfig {
            prefix: "MIN".to_string(),
            min_length: 5,
            include_date: false,
            include_sequence: false,
            separator: "".to_string(),
            validation_pattern: r"^[A-Z0-9]+$".to_string(),
        }
    }

    pub fn create_complex() -> BarcodeConfig {
        BarcodeConfig {
            prefix: "COMPLEX".to_string(),
            min_length: 20,
            include_date: true,
            include_sequence: true,
            separator: "_".to_string(),
            validation_pattern: r"^[A-Z0-9_]+$".to_string(),
        }
    }
}

/// Test data generator for barcode service
pub struct TestDataGenerator;

impl TestDataGenerator {
    pub fn sample_types() -> Vec<&'static str> {
        vec!["DNA", "RNA", "PROTEIN", "CELL", "TISSUE", "PLASMA", "SERUM"]
    }

    pub fn invalid_barcodes() -> Vec<&'static str> {
        vec![
            "",                    // Empty
            "short",              // Too short
            "invalid@chars",      // Invalid characters
            "lower-case",         // Lowercase not allowed
            "space in barcode",   // Spaces not allowed
            "!@#$%^&*()",        // Special characters
            "a".repeat(100).as_str(), // Too long (static lifetime issue, simplified)
        ]
    }

    pub fn valid_barcodes() -> Vec<&'static str> {
        vec![
            "TST-DNA-20240115-L001-1234567",
            "MIN12345",
            "COMPLEX_RNA_20240115_2345678",
            "SIMPLE-001",
            "LAB-SAMPLE-12345",
        ]
    }

    pub fn location_ids() -> Vec<i32> {
        vec![1, 10, 100, 999]
    }

    pub fn template_names() -> Vec<&'static str> {
        vec![
            "Standard DNA Extraction",
            "RNA Sequencing Prep",
            "Protein Purification",
            "Cell Culture Protocol",
        ]
    }
}

/// Assertions for barcode testing
pub struct BarcodeAssertions;

impl BarcodeAssertions {
    pub fn assert_barcode_format(barcode: &str, config: &BarcodeConfig) {
        // Check minimum length
        assert!(
            barcode.len() >= config.min_length,
            "Barcode '{}' is shorter than minimum length {}",
            barcode,
            config.min_length
        );

        // Check prefix
        assert!(
            barcode.starts_with(&config.prefix),
            "Barcode '{}' does not start with prefix '{}'",
            barcode,
            config.prefix
        );

        // Check separator usage
        if !config.separator.is_empty() {
            assert!(
                barcode.contains(&config.separator),
                "Barcode '{}' does not contain separator '{}'",
                barcode,
                config.separator
            );
        }
    }

    pub fn assert_barcode_components(info: &BarcodeInfo, expected_prefix: &str) {
        assert_eq!(
            info.prefix.as_deref(),
            Some(expected_prefix),
            "Prefix mismatch"
        );
        assert!(info.is_valid, "Barcode should be valid");
    }

    pub fn assert_barcode_unique(barcode1: &str, barcode2: &str) {
        assert_ne!(
            barcode1, barcode2,
            "Generated barcodes should be unique"
        );
    }

    pub fn assert_error_type<T>(result: Result<T>, expected_error_type: &str) {
        assert!(result.is_err(), "Expected error but got success");
        let error_str = format!("{:?}", result.unwrap_err());
        assert!(
            error_str.contains(expected_error_type),
            "Expected error containing '{}', got '{}'",
            expected_error_type,
            error_str
        );
    }
}

/// Performance test utilities
pub struct PerformanceTestUtils;

impl PerformanceTestUtils {
    pub async fn measure_generation_time(
        service: &BarcodeService,
        iterations: u32,
    ) -> std::time::Duration {
        let start = std::time::Instant::now();

        for i in 0..iterations {
            let sample_type = TestDataGenerator::sample_types()[i as usize % 7];
            let _ = service.generate_barcode(
                Some(sample_type),
                Some((i % 100) as i32),
                None,
                None,
            ).await;
        }

        start.elapsed()
    }

    pub fn measure_validation_time(
        service: &BarcodeService,
        barcodes: &[&str],
    ) -> std::time::Duration {
        let start = std::time::Instant::now();

        for barcode in barcodes {
            let _ = service.validate_barcode_format(barcode);
        }

        start.elapsed()
    }
}

/// Helper to create test barcode service
pub async fn create_test_barcode_service(config: BarcodeConfig) -> (BarcodeService, TestDatabase) {
    let mut test_db = TestDatabase::new().await;
    let db_pool = DatabasePool::from(test_db.pool.clone());
    let service = BarcodeService::new(db_pool, config)
        .await
        .expect("Failed to create barcode service");
    
    (service, test_db)
}

/// Test macro for barcode service tests
#[macro_export]
macro_rules! test_with_barcode_db {
    ($test_name:ident, $config:expr, $test_body:expr) => {
        #[tokio::test]
        async fn $test_name() {
            let (service, mut test_db) = crate::test_utils::create_test_barcode_service($config).await;
            let result = std::panic::AssertUnwindSafe($test_body(&service, &mut test_db))
                .catch_unwind()
                .await;
            test_db.cleanup().await;
            if let Err(panic) = result {
                std::panic::resume_unwind(panic);
            }
        }
    };
}