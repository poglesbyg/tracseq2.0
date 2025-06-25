use anyhow::Result;
use once_cell::sync::Lazy;
use std::sync::Arc;
use sqlx::{PgPool, Row};
use tokio::sync::Mutex;
use uuid::Uuid;
use enhanced_storage_service::{
    config::Config,
    database::DatabasePool,
    services::EnhancedStorageService,
    AppState,
    models::*,
    handlers::iot::RecordReadingRequest,
    // test_utils::*,
};
use axum::{http::StatusCode, Router};
use axum_test::TestServer;
use fake::{Fake, Faker};
use serde_json::Value;

// Global test database pool to avoid connection exhaustion
static TEST_DB_POOL: Lazy<Arc<Mutex<Option<DatabasePool>>>> = 
    Lazy::new(|| Arc::new(Mutex::new(None)));

/// Test database configuration
pub struct TestDatabase {
    pub pool: DatabasePool,
    pub database_name: String,
    pub cleanup_containers: Vec<Uuid>,
    pub cleanup_locations: Vec<Uuid>,
    pub cleanup_sensors: Vec<Uuid>,
    pub cleanup_transactions: Vec<String>,
}

impl TestDatabase {
    /// Create a new test database with unique name
    pub async fn new() -> Result<Self> {
        let database_name = format!("test_enhanced_storage_{}", Uuid::new_v4().simple());
        let database_url = std::env::var("TEST_DATABASE_URL")
            .unwrap_or_else(|_| "postgresql://postgres:password@localhost:5432/postgres".to_string());

        // Connect to postgres database to create test database
        let admin_pool = sqlx::PgPool::connect(&database_url).await?;
        
        // Create test database
        sqlx::query(&format!("CREATE DATABASE \"{}\"", database_name))
            .execute(&admin_pool)
            .await?;

        admin_pool.close().await;

        // Connect to test database
        let test_url = database_url.replace("/postgres", &format!("/{}", database_name));
        let pool = DatabasePool::new(&test_url).await?;
        
        // Run migrations
        pool.migrate().await?;

        Ok(Self {
            pool,
            database_name,
            cleanup_containers: Vec::new(),
            cleanup_locations: Vec::new(),
            cleanup_sensors: Vec::new(),
            cleanup_transactions: Vec::new(),
        })
    }

    /// Clean up test database
    pub async fn cleanup(self) -> Result<()> {
        let database_url = std::env::var("TEST_DATABASE_URL")
            .unwrap_or_else(|_| "postgresql://postgres:password@localhost:5432/postgres".to_string());

        // Close test pool
        self.pool.pool.close().await;

        // Connect to postgres database to drop test database
        let admin_pool = sqlx::PgPool::connect(&database_url).await?;
        
        // Terminate active connections to test database
        sqlx::query(&format!(
            "SELECT pg_terminate_backend(pid) FROM pg_stat_activity WHERE datname = '{}'",
            self.database_name
        ))
        .execute(&admin_pool)
        .await?;

        // Drop test database
        sqlx::query(&format!("DROP DATABASE IF EXISTS \"{}\"", self.database_name))
            .execute(&admin_pool)
            .await?;

        admin_pool.close().await;
        Ok(())
    }

    /// Clear all data from tables (faster than recreating database)
    pub async fn clear_data(&self) -> Result<()> {
        let tables = vec![
            "compliance_events",
            "energy_consumption", 
            "automation_tasks",
            "blockchain_transactions",
            "predictions",
            "analytics_models",
            "alerts",
            "sensor_data",
            "iot_sensors",
            "samples",
            "storage_locations",
        ];

        for table in tables {
            sqlx::query(&format!("TRUNCATE TABLE {} CASCADE", table))
                .execute(&self.pool.pool)
                .await?;
        }

        Ok(())
    }

    pub fn track_container(&mut self, container_id: Uuid) {
        self.cleanup_containers.push(container_id);
    }

    pub fn track_location(&mut self, location_id: Uuid) {
        self.cleanup_locations.push(location_id);
    }

    pub fn track_sensor(&mut self, sensor_id: Uuid) {
        self.cleanup_sensors.push(sensor_id);
    }

    pub fn track_transaction(&mut self, transaction_hash: String) {
        self.cleanup_transactions.push(transaction_hash);
    }
}

/// Test application state factory
pub struct TestAppStateBuilder {
    pub db: Option<DatabasePool>,
    pub config: Option<Config>,
}

impl TestAppStateBuilder {
    pub fn new() -> Self {
        Self {
            db: None,
            config: None,
        }
    }

    pub fn with_database(mut self, db: DatabasePool) -> Self {
        self.db = Some(db);
        self
    }

    pub fn with_config(mut self, config: Config) -> Self {
        self.config = Some(config);
        self
    }

    pub async fn build(self) -> Result<AppState> {
        let config = self.config.unwrap_or_else(|| test_config());
        let db_pool = self.db.unwrap_or_else(|| {
            panic!("Database must be provided for test app state")
        });

        let storage_service = EnhancedStorageService::new(db_pool.clone(), config.clone()).await?;

        // Mock AI platform and integration hub for testing
        let ai_config = enhanced_storage_service::ai::AIConfig {
            model_storage_path: config.analytics.prediction_models_path.clone(),
            training_data_path: "./test_training_data".to_string(),
            inference_timeout_seconds: 30,
            model_update_interval_hours: 24,
            enable_real_time_training: false,
            enable_anomaly_detection: false,
            enable_predictive_maintenance: false,
            confidence_threshold: 0.85,
        };
        let ai_platform = Arc::new(enhanced_storage_service::ai::AIPlatform::new(ai_config));
        
        let integration_config = enhanced_storage_service::integrations::IntegrationConfig::default();
        let integration_hub = Arc::new(enhanced_storage_service::integrations::IntegrationHub::new(integration_config));

        Ok(AppState {
            storage_service,
            config,
            db_pool,
            ai_platform,
            integration_hub,
        })
    }
}

impl Default for TestAppStateBuilder {
    fn default() -> Self {
        Self::new()
    }
}

/// Create test configuration
pub fn test_config() -> Config {
    Config {
        server: enhanced_storage_service::config::ServerConfig {
            host: "127.0.0.1".to_string(),
            port: 0, // Use random port for tests
            max_connections: 100,
            timeout_seconds: 30,
            workers: 4,
        },
        database_url: "postgresql://test:test@localhost:5432/test".to_string(),
        auth_service_url: "http://localhost:8001".to_string(),
        storage: enhanced_storage_service::config::StorageConfig {
            default_temperature_zones: vec![
                "-80C".to_string(),
                "-20C".to_string(), 
                "4C".to_string(),
                "RT".to_string(),
                "37C".to_string(),
            ],
            capacity_warning_threshold: 80.0,
            capacity_critical_threshold: 95.0,
            auto_organization: true,
            barcode_generation: true,
            location_tracking: true,
        },
        iot: enhanced_storage_service::config::IoTConfig {
            enabled: false, // Disable for tests
            mqtt_broker_url: "mqtt://localhost:1883".to_string(),
            mqtt_username: "test".to_string(),
            mqtt_password: "test".to_string(),
            alert_threshold_temperature: 2.0,
            alert_threshold_humidity: 10.0,
            sensor_polling_interval_seconds: 60,
            calibration_enabled: true,
            real_time_monitoring: false,
        },
        blockchain: enhanced_storage_service::config::BlockchainConfig {
            enabled: true,
            chain_id: "test".to_string(),
            private_key: "test_private_key".to_string(),
            block_size_limit: 1000,
            mining_difficulty: 1,
            consensus_algorithm: "proof_of_authority".to_string(),
            immutable_records: true,
        },
        analytics: enhanced_storage_service::config::AnalyticsConfig {
            enabled: false,
            prediction_models_path: "./test_models".to_string(),
            training_data_retention_days: 365,
            prediction_horizon_days: 30,
            anomaly_detection_enabled: false,
            machine_learning_enabled: false,
            time_series_analysis: false,
            model_retraining_interval_hours: 24,
        },
        digital_twin: enhanced_storage_service::config::DigitalTwinConfig {
            enabled: false,
            simulation_engine: "basic".to_string(),
            update_interval_minutes: 300,
            physics_simulation: false,
            thermal_modeling: false,
            capacity_modeling: false,
            optimization_enabled: false,
        },
        automation: enhanced_storage_service::config::AutomationConfig {
            enabled: false,
            robot_integration: false,
            automated_placement: false,
            automated_retrieval: false,
            scheduling_enabled: false,
            max_concurrent_tasks: 10,
            safety_checks: true,
        },
        energy: enhanced_storage_service::config::EnergyConfig {
            optimization_enabled: false,
            smart_scheduling: false,
            energy_monitoring: false,
            cost_optimization: false,
            renewable_integration: false,
            efficiency_targets: 0.85,
        },
        mobile: enhanced_storage_service::config::MobileConfig {
            enabled: false,
            jwt_secret: "test_secret".to_string(),
            barcode_scanning: false,
            geolocation_tracking: false,
            offline_support: false,
            push_notifications: false,
        },
        compliance: enhanced_storage_service::config::ComplianceConfig {
            enabled: true,
            regulatory_standards: vec!["basic".to_string()],
            audit_logging: true,
            chain_of_custody: true,
            data_integrity: true,
            access_controls: true,
        },
    }
}

/// Common test assertions
pub struct TestAssertions;

impl TestAssertions {
    /// Assert that two timestamps are within acceptable range
    pub fn assert_timestamp_recent(timestamp: chrono::DateTime<chrono::Utc>, tolerance_seconds: i64) {
        let now = chrono::Utc::now();
        let diff = (now - timestamp).num_seconds().abs();
        assert!(
            diff <= tolerance_seconds,
            "Timestamp {} is not within {} seconds of now ({}). Difference: {} seconds",
            timestamp, tolerance_seconds, now, diff
        );
    }

    /// Assert pagination response structure
    pub fn assert_pagination_valid(
        page: i32,
        per_page: i32,
        total_items: i64,
        returned_items: usize,
        has_next: bool,
        has_prev: bool,
    ) {
        let total_pages = (total_items as i32 + per_page - 1) / per_page;
        
        assert!(page >= 1, "Page should be >= 1");
        assert!(per_page >= 1, "Per page should be >= 1");
        assert!(returned_items <= per_page as usize, "Returned items should not exceed per_page");
        assert_eq!(has_next, page < total_pages, "has_next calculation incorrect");
        assert_eq!(has_prev, page > 1, "has_prev calculation incorrect");
    }

    /// Assert API response structure
    pub fn assert_api_response_success<T>(response: &enhanced_storage_service::models::ApiResponse<T>) {
        assert!(response.success, "API response should be successful");
        assert!(response.data.is_some(), "API response should contain data");
        assert!(response.message.is_none(), "Successful response should not have error message");
        Self::assert_timestamp_recent(response.timestamp, 5);
    }

    /// Assert API error response structure
    pub fn assert_api_response_error<T>(response: &enhanced_storage_service::models::ApiResponse<T>) {
        assert!(!response.success, "API response should not be successful");
        assert!(response.data.is_none(), "Error response should not contain data");
        assert!(response.message.is_some(), "Error response should have error message");
        Self::assert_timestamp_recent(response.timestamp, 5);
    }
}

/// Test data factory for creating consistent test objects
pub struct TestDataFactory;

impl TestDataFactory {
    /// Generate test UUID
    pub fn uuid() -> Uuid {
        Uuid::new_v4()
    }

    /// Generate test barcode
    pub fn barcode() -> String {
        let uuid_str = Uuid::new_v4().simple().to_string();
        format!("TEST{}", &uuid_str[..8].to_uppercase())
    }

    /// Generate test sensor ID
    pub fn sensor_id() -> String {
        let uuid_str = Uuid::new_v4().simple().to_string();
        format!("SENSOR_{}", &uuid_str[..8].to_uppercase())
    }

    /// Generate test temperature value
    pub fn temperature(zone: &str) -> f64 {
        match zone {
            "-80C" => -80.0 + (fastrand::f64() * 4.0 - 2.0), // -82 to -78
            "-20C" => -20.0 + (fastrand::f64() * 4.0 - 2.0), // -22 to -18
            "4C" => 4.0 + (fastrand::f64() * 2.0 - 1.0),     // 3 to 5
            "RT" => 22.0 + (fastrand::f64() * 6.0 - 3.0),    // 19 to 25
            "37C" => 37.0 + (fastrand::f64() * 2.0 - 1.0),   // 36 to 38
            _ => 20.0,
        }
    }

    /// Generate test coordinates
    pub fn coordinates() -> serde_json::Value {
        serde_json::json!({
            "x": fastrand::f64() * 100.0,
            "y": fastrand::f64() * 100.0,
            "z": fastrand::f64() * 10.0,
            "rack": format!("R{}", fastrand::u32(1..=20)),
            "shelf": fastrand::u32(1..=10),
            "position": fastrand::u32(1..=96)
        })
    }
}

/// HTTP test client wrapper
pub struct TestClient {
    client: reqwest::Client,
    base_url: String,
}

impl TestClient {
    pub fn new(base_url: String) -> Self {
        Self {
            client: reqwest::Client::new(),
            base_url,
        }
    }

    pub async fn get(&self, path: &str) -> Result<reqwest::Response> {
        let url = format!("{}{}", self.base_url, path);
        Ok(self.client.get(&url).send().await?)
    }

    pub async fn post<T: serde::Serialize>(&self, path: &str, json: &T) -> Result<reqwest::Response> {
        let url = format!("{}{}", self.base_url, path);
        Ok(self.client.post(&url).json(json).send().await?)
    }

    pub async fn put<T: serde::Serialize>(&self, path: &str, json: &T) -> Result<reqwest::Response> {
        let url = format!("{}{}", self.base_url, path);
        Ok(self.client.put(&url).json(json).send().await?)
    }

    pub async fn delete(&self, path: &str) -> Result<reqwest::Response> {
        let url = format!("{}{}", self.base_url, path);
        Ok(self.client.delete(&url).send().await?)
    }
}

/// Test logging setup
pub fn setup_test_logging() {
    // Simple test logging setup without external dependencies
    unsafe {
        std::env::set_var("RUST_LOG", "debug");
    }
}

/// Macro for running tests with database cleanup
#[macro_export]
macro_rules! test_with_db {
    ($test_name:ident, $test_fn:expr) => {
        #[tokio::test]
        #[test_log::test]
        async fn $test_name() {
            use $crate::test_utils::{TestDatabase, setup_test_logging};
            
            setup_test_logging();
            
            let test_db = TestDatabase::new().await
                .expect("Failed to create test database");
            
            let result = $test_fn(test_db.pool.clone()).await;
            
            // Clean up database
            test_db.cleanup().await
                .expect("Failed to cleanup test database");
            
            // Propagate test result
            result.expect("Test failed");
        }
    };
}

/// Factory for creating test storage entities with realistic laboratory data
pub struct StorageFactory;

impl StorageFactory {
    pub fn create_valid_location_request() -> CreateStorageLocationRequest {
        let uuid_str = Uuid::new_v4().simple().to_string();
        CreateStorageLocationRequest {
            name: format!("Test Location {}", &uuid_str[..8]),
            description: Some("Test storage location for automated testing".to_string()),
            location_type: "freezer".to_string(),
            temperature_zone: "-80C".to_string(),
            max_capacity: 1000,
            coordinates: Some(serde_json::json!({
                "building": "Lab Building A",
                "room": "Room 101",
                "zone": "Zone A1"
            })),
            metadata: Some(serde_json::json!({
                "purpose": "testing",
                "created_by": "test_framework"
            })),
        }
    }

    pub fn create_valid_container_request() -> StoreSampleRequest {
        StoreSampleRequest {
            barcode: Self::generate_container_barcode(),
            sample_type: "blood".to_string(),
            storage_location_id: Uuid::new_v4(), // Will be replaced with actual location ID in tests
            position: Some(serde_json::json!({"rack": "A1", "position": 42})),
            temperature_requirements: Some("-80C".to_string()),
            metadata: Some(serde_json::json!({
                "collection_date": chrono::Utc::now(),
                "patient_id": "TEST_PATIENT_001",
                "test_type": "genetic_analysis"
            })),
        }
    }

    pub fn create_valid_sensor_request() -> enhanced_storage_service::handlers::iot::RegisterSensorRequest {
        enhanced_storage_service::handlers::iot::RegisterSensorRequest {
            sensor_id: Self::generate_sensor_id(),
            sensor_type: "temperature".to_string(),
            location_id: Some(Uuid::new_v4()), // Will be replaced with actual location ID in tests
            battery_level: Some(95),
            signal_strength: Some(85),
            firmware_version: Some("v1.2.3".to_string()),
            configuration: Some(serde_json::json!({
                "max_temperature": 5.0,
                "min_temperature": -85.0,
                "alert_interval_minutes": 5
            })),
        }
    }

    pub fn create_blockchain_transaction() -> BlockchainTransaction {
        BlockchainTransaction {
            id: Uuid::new_v4(),
            transaction_hash: Self::generate_transaction_hash(),
            block_number: Some(fastrand::i64(1000000..9999999)),
            transaction_type: "storage_event".to_string(),
            data_hash: "sample_data_hash".to_string(),
            previous_hash: Some("previous_block_hash".to_string()),
            timestamp: chrono::Utc::now(),
            signature: "test_signature".to_string(),
            metadata: serde_json::json!({
                "event_type": "container_move",
                "container_id": Uuid::new_v4(),
                "from_location": "A1",
                "to_location": "B2",
                "timestamp": chrono::Utc::now()
            }),
            created_at: chrono::Utc::now(),
        }
    }

    pub fn generate_container_barcode() -> String {
        format!("CONT-{:08}-{:06}", 
                chrono::Utc::now().format("%Y%m%d"),
                fastrand::u32(100000..999999))
    }

    pub fn generate_sensor_id() -> String {
        format!("SENS-{:06}", fastrand::u32(100000..999999))
    }

    pub fn generate_transaction_hash() -> String {
        format!("0x{:064x}", fastrand::u128(..))
    }

    pub fn create_sample_request(location_id: Uuid) -> StoreSampleRequest {
        StoreSampleRequest {
            barcode: Self::generate_container_barcode(),
            sample_type: "blood".to_string(),
            storage_location_id: location_id,
            position: Some(serde_json::json!({"rack": "A1", "position": 42})),
            temperature_requirements: Some("-80C".to_string()),
            metadata: Some(serde_json::json!({
                "collection_date": chrono::Utc::now(),
                "patient_id": "TEST_PATIENT_001",
                "test_type": "genetic_analysis"
            })),
        }
    }
}

/// HTTP test client wrapper for storage API testing
pub struct StorageTestClient {
    pub server: TestServer,
    pub auth_token: Option<String>,
}

impl StorageTestClient {
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
            request = request.add_header("Authorization", format!("Bearer {}", token));
        }
        request.await
    }

    pub async fn get(&self, path: &str) -> axum_test::TestResponse {
        let mut request = self.server.get(path);
        if let Some(token) = &self.auth_token {
            request = request.add_header("Authorization", format!("Bearer {}", token));
        }
        request.await
    }

    pub async fn put_json<T: serde::Serialize>(&self, path: &str, body: &T) -> axum_test::TestResponse {
        let mut request = self.server.put(path).json(body);
        if let Some(token) = &self.auth_token {
            request = request.add_header("Authorization", format!("Bearer {}", token));
        }
        request.await
    }

    pub async fn delete(&self, path: &str) -> axum_test::TestResponse {
        let mut request = self.server.delete(path);
        if let Some(token) = &self.auth_token {
            request = request.add_header("Authorization", format!("Bearer {}", token));
        }
        request.await
    }
}

/// Common assertions for storage testing
pub struct StorageAssertions;

impl StorageAssertions {
    pub fn assert_successful_creation(response: &Value) {
        assert_eq!(response["success"], true);
        assert!(response["data"]["id"].is_string());
        assert!(response["data"]["created_at"].is_string());
    }

    pub fn assert_location_data(response: &Value, expected_name: &str) {
        assert_eq!(response["success"], true);
        assert_eq!(response["data"]["name"], expected_name);
        assert!(response["data"]["id"].is_string());
        assert!(response["data"]["temperature_range"].is_object());
    }

    pub fn assert_container_data(response: &Value, expected_barcode: &str) {
        assert_eq!(response["success"], true);
        assert_eq!(response["data"]["barcode"], expected_barcode);
        assert!(response["data"]["id"].is_string());
        assert!(response["data"]["location_id"].is_string());
    }

    pub fn assert_sensor_data(response: &Value, expected_type: &str) {
        assert_eq!(response["success"], true);
        assert_eq!(response["data"]["sensor_type"], expected_type);
        assert!(response["data"]["id"].is_string());
        assert!(response["data"]["identifier"].is_string());
    }

    pub fn assert_blockchain_transaction(response: &Value) {
        assert_eq!(response["success"], true);
        assert!(response["data"]["transaction_hash"].is_string());
        assert!(response["data"]["block_number"].is_number());
        assert_eq!(response["data"]["validated"], true);
    }

    pub fn assert_iot_reading(response: &Value, expected_sensor_id: &str) {
        assert_eq!(response["success"], true);
        assert_eq!(response["data"]["sensor_id"], expected_sensor_id);
        assert!(response["data"]["value"].is_number());
        assert!(response["data"]["timestamp"].is_string());
    }

    pub fn assert_validation_error(response: &Value) {
        assert_eq!(response["success"], false);
        assert!(response["error"].is_string());
    }

    pub fn assert_status_code(status: StatusCode, expected: StatusCode) {
        assert_eq!(status, expected);
    }
}

/// Test data generators for various storage scenarios
pub struct StorageTestDataGenerator;

impl StorageTestDataGenerator {
    pub fn location_types() -> Vec<String> {
        vec![
            "freezer".to_string(),
            "refrigerator".to_string(),
            "room_temperature".to_string(),
            "incubator".to_string(),
            "liquid_nitrogen".to_string(),
        ]
    }

    pub fn temperature_zones() -> Vec<String> {
        vec![
            "-80C".to_string(),
            "-20C".to_string(),
            "4C".to_string(),
            "RT".to_string(),
            "37C".to_string(),
        ]
    }

    pub fn sample_types() -> Vec<String> {
        vec![
            "blood".to_string(),
            "plasma".to_string(),
            "serum".to_string(),
            "tissue".to_string(),
            "dna".to_string(),
        ]
    }

    pub fn generate_sensor_readings(sensor_id: String, count: usize) -> Vec<SensorReading> {
        (0..count)
            .map(|i| SensorReading {
                sensor_id: sensor_id.clone(),
                readings: vec![SensorReadingValue {
                    reading_type: "temperature".to_string(),
                    value: -80.0 + (i as f64 * 0.1),
                    unit: "°C".to_string(),
                    quality_score: Some(0.95),
                }],
                timestamp: chrono::Utc::now() - chrono::Duration::minutes(i as i64),
            })
            .collect()
    }

    pub fn invalid_temperature_zones() -> Vec<String> {
        vec![
            "INVALID".to_string(),
            "999C".to_string(),
            "-999C".to_string(),
            "".to_string(),
        ]
    }

    pub fn invalid_capacities() -> Vec<i32> {
        vec![-1, 0, -100]
    }
}

/// Performance testing utilities for storage operations
pub struct StoragePerformanceUtils;

impl StoragePerformanceUtils {
    pub async fn measure_location_creation_time(
        client: &StorageTestClient,
        request: &CreateStorageLocationRequest,
    ) -> std::time::Duration {
        let start = std::time::Instant::now();
        let _ = client.post_json("/api/storage/locations", request).await;
        start.elapsed()
    }

    pub async fn measure_sample_storage_time(
        client: &StorageTestClient,
        request: &StoreSampleRequest,
    ) -> std::time::Duration {
        let start = std::time::Instant::now();
        let _ = client.post_json("/api/storage/samples", request).await;
        start.elapsed()
    }

    pub async fn concurrent_sensor_readings(
        client: &StorageTestClient,
        sensor_id: String,
        concurrent_count: usize,
    ) -> Vec<StatusCode> {
        let tasks: Vec<_> = (0..concurrent_count)
            .map(|i| {
                let sensor_id = sensor_id.clone(); // Clone to avoid move
                let reading = RecordReadingRequest {
                    value: -80.0 + (i as f64 * 0.1),
                    unit: Some("°C".to_string()),
                    timestamp: Some(chrono::Utc::now()),
                    metadata: Some(serde_json::json!({"sequence": i})),
                };
                async move {
                    client.post_json(&format!("/api/iot/sensors/{}/readings", sensor_id), &reading).await.status_code()
                }
            })
            .collect();

        futures::future::join_all(tasks).await
    }

    pub async fn blockchain_transaction_throughput(
        client: &StorageTestClient,
        operation_count: usize,
    ) -> std::time::Duration {
        let start = std::time::Instant::now();
        
        let tasks: Vec<_> = (0..operation_count)
            .map(|i| {
                let transaction = StorageFactory::create_blockchain_transaction();
                async move {
                    client.post_json("/api/storage/blockchain/record", &transaction).await
                }
            })
            .collect();

        let _ = futures::future::join_all(tasks).await;
        start.elapsed()
    }
}

/// Digital Twin testing utilities
pub struct DigitalTwinTestUtils;

impl DigitalTwinTestUtils {
    pub fn create_twin_state(location_id: Uuid) -> DigitalTwinState {
        DigitalTwinState {
            twin_id: Uuid::new_v4(),
            physical_entity_id: location_id,
            entity_type: "storage_location".to_string(),
            current_state: serde_json::json!({
                "temperature": -80.0,
                "capacity_utilization": 0.75,
                "energy_consumption": {
                    "current_kw": 2.5,
                    "daily_kwh": 60.0
                }
            }),
            predicted_state: Some(serde_json::json!({
                "temperature_next_hour": -80.1,
                "confidence": 0.95
            })),
            simulation_parameters: serde_json::json!({
                "thermal_mass": 1000.0,
                "insulation_factor": 0.95,
                "ambient_temperature": 22.0
            }),
            last_sync: chrono::Utc::now(),
            sync_status: "synchronized".to_string(),
        }
    }

    pub fn assert_twin_prediction(response: &Value, expected_confidence: f64) {
        assert_eq!(response["success"], true);
        assert!(response["data"]["prediction"].is_object());
        assert!(response["data"]["prediction"]["confidence"].as_f64().unwrap() >= expected_confidence);
    }

    pub async fn simulate_temperature_drift(
        client: &StorageTestClient,
        sensor_id: String,
        duration_minutes: i64,
    ) -> Vec<f64> {
        let mut temperatures = Vec::new();
        
        for minute in 0..duration_minutes {
            let temp = -80.0 + (minute as f64 * 0.01); // Slight drift
            let reading = RecordReadingRequest {
                value: temp,
                unit: Some("°C".to_string()),
                timestamp: Some(chrono::Utc::now() + chrono::Duration::minutes(minute)),
                metadata: Some(serde_json::json!({"drift_simulation": true})),
            };
            
            let _ = client.post_json(&format!("/api/iot/sensors/{}/readings", sensor_id), &reading).await;
            temperatures.push(temp);
        }
        
        temperatures
    }
}

/// Mobile integration testing utilities
pub struct MobileTestUtils;

impl MobileTestUtils {
    pub fn create_mobile_task(user_id: Uuid, device_token: String) -> MobileTaskAssignment {
        MobileTaskAssignment {
            task_id: Uuid::new_v4(),
            user_id,
            task_type: "sample_retrieval".to_string(),
            description: "Retrieve sample for testing".to_string(),
            location: Some(serde_json::json!({
                "latitude": 40.7128,
                "longitude": -74.0060,
                "accuracy": 5.0
            })),
            priority: "normal".to_string(),
            estimated_duration: Some(30),
            assigned_at: chrono::Utc::now(),
            due_date: Some(chrono::Utc::now() + chrono::Duration::hours(2)),
        }
    }

    pub fn create_mobile_request(user_id: Uuid, device_token: String) -> MobileTaskAssignment {
        Self::create_mobile_task(user_id, device_token)
    }

    pub fn assert_mobile_response(response: &Value) {
        assert_eq!(response["success"], true);
        assert!(response["data"]["access_granted"].is_boolean());
        assert!(response["data"]["session_id"].is_string());
        if response["data"]["access_granted"].as_bool().unwrap() {
            assert!(response["data"]["access_token"].is_string());
            assert!(response["data"]["expires_at"].is_string());
        }
    }

    pub async fn test_qr_code_scan(
        client: &StorageTestClient,
        qr_data: &str,
    ) -> axum_test::TestResponse {
        let scan_request = serde_json::json!({
            "qr_data": qr_data,
            "scan_timestamp": chrono::Utc::now(),
            "device_id": "test-device-123"
        });

        client.post_json("/api/storage/mobile/scan", &scan_request).await
    }
} 
