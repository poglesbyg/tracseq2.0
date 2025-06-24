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
};

// Global test database pool to avoid connection exhaustion
static TEST_DB_POOL: Lazy<Arc<Mutex<Option<DatabasePool>>>> = 
    Lazy::new(|| Arc::new(Mutex::new(None)));

/// Test database configuration
pub struct TestDatabase {
    pub pool: DatabasePool,
    pub database_name: String,
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
        let ai_platform = Arc::new(enhanced_storage_service::ai::AIPlatform::mock());
        let integration_hub = Arc::new(enhanced_storage_service::integrations::IntegrationHub::mock());

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
        },
        database_url: "postgresql://test:test@localhost:5432/test".to_string(),
        storage: enhanced_storage_service::config::StorageConfig {
            default_temperature_zones: vec![
                "-80C".to_string(),
                "-20C".to_string(), 
                "4C".to_string(),
                "RT".to_string(),
                "37C".to_string(),
            ],
            max_capacity_per_location: 1000,
            enable_automation: true,
        },
        iot: enhanced_storage_service::config::IoTConfig {
            enable_mqtt: false, // Disable for tests
            mqtt_broker_url: "mqtt://localhost:1883".to_string(),
            alert_threshold_temperature: 2.0,
            alert_threshold_humidity: 10.0,
            sensor_health_check_interval: 300,
        },
        blockchain: enhanced_storage_service::config::BlockchainConfig {
            enable_blockchain: true,
            network: "test".to_string(),
            private_key: "test_private_key".to_string(),
        },
        ai: enhanced_storage_service::config::AIConfig {
            enable_ai_predictions: false, // Disable for most tests
            model_endpoint: "http://localhost:8000".to_string(),
            prediction_confidence_threshold: 0.8,
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
        format!("TEST{}", Uuid::new_v4().simple()[..8].to_uppercase())
    }

    /// Generate test sensor ID
    pub fn sensor_id() -> String {
        format!("SENSOR_{}", Uuid::new_v4().simple()[..8].to_uppercase())
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
    let _ = env_logger::builder()
        .filter_level(log::LevelFilter::Debug)
        .is_test(true)
        .try_init();
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
