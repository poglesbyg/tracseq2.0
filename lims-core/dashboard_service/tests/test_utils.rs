use dashboard_service::{
    AppState, DashboardData, Settings, ServiceUrls,
    config,
};
use sqlx::PgPool;
use std::sync::Arc;
use std::time::Duration;
use wiremock::{MockServer, Mock, ResponseTemplate};
use wiremock::matchers::{method, path};

/// Test database manager for isolated test environments
pub struct TestDatabase {
    pub pool: PgPool,
}

impl TestDatabase {
    pub async fn new() -> Self {
        // Use test database URL from environment or default
        let database_url = std::env::var("TEST_DATABASE_URL")
            .unwrap_or_else(|_| "postgres://postgres:password@localhost:5432/dashboard_test".to_string());
        
        let pool = PgPool::connect(&database_url)
            .await
            .expect("Failed to connect to test database");
        
        // Run migrations
        sqlx::migrate!("./migrations")
            .run(&pool)
            .await
            .expect("Failed to run migrations");
        
        // Clean up any existing test data
        sqlx::query("TRUNCATE TABLE dashboard_widgets CASCADE")
            .execute(&pool)
            .await
            .ok();
        
        Self { pool }
    }

    pub async fn cleanup(&self) {
        // Clean up test data
        sqlx::query("TRUNCATE TABLE dashboard_widgets CASCADE")
            .execute(&self.pool)
            .await
            .ok();
    }
}

/// Factory for creating test settings
pub struct SettingsFactory;

impl SettingsFactory {
    pub fn create_test_settings(mock_server_url: &str) -> Settings {
        Settings {
            database_url: "postgres://postgres:password@localhost:5432/dashboard_test".to_string(),
            port: 3025,
            service_urls: ServiceUrls {
                auth_service: format!("{}/auth", mock_server_url),
                sample_service: format!("{}/sample", mock_server_url),
                storage_service: format!("{}/storage", mock_server_url),
                sequencing_service: format!("{}/sequencing", mock_server_url),
                notification_service: format!("{}/notification", mock_server_url),
                rag_service: format!("{}/rag", mock_server_url),
                barcode_service: format!("{}/barcode", mock_server_url),
                qaqc_service: format!("{}/qaqc", mock_server_url),
                library_service: format!("{}/library", mock_server_url),
                event_service: format!("{}/event", mock_server_url),
                transaction_service: format!("{}/transaction", mock_server_url),
                spreadsheet_service: format!("{}/spreadsheet", mock_server_url),
            },
        }
    }
}

/// Mock service responses
pub struct MockResponses;

impl MockResponses {
    pub fn sample_metrics() -> serde_json::Value {
        serde_json::json!({
            "total_samples": 1234,
            "samples_today": 45,
            "samples_this_week": 289,
            "samples_this_month": 1122,
            "sample_types": {
                "DNA": 456,
                "RNA": 321,
                "PROTEIN": 234,
                "CELL": 123,
                "TISSUE": 100
            }
        })
    }

    pub fn storage_metrics() -> serde_json::Value {
        serde_json::json!({
            "total_capacity": 10000,
            "used_capacity": 7850,
            "utilization_percent": 78.5,
            "temperature_zones": {
                "-80C": {"capacity": 2000, "used": 1800},
                "-20C": {"capacity": 2000, "used": 1600},
                "4C": {"capacity": 3000, "used": 2450},
                "RT": {"capacity": 3000, "used": 2000}
            }
        })
    }

    pub fn sequencing_metrics() -> serde_json::Value {
        serde_json::json!({
            "active_runs": 5,
            "completed_today": 2,
            "average_runtime_hours": 48.5,
            "success_rate": 97.8,
            "queue_depth": 23
        })
    }

    pub fn service_health() -> serde_json::Value {
        serde_json::json!({
            "status": "healthy",
            "uptime_seconds": 86400,
            "last_check": chrono::Utc::now(),
            "memory_usage_mb": 256,
            "cpu_usage_percent": 15.5
        })
    }
}

/// Mock server setup helpers
pub struct MockServerSetup;

impl MockServerSetup {
    pub async fn setup_healthy_services(mock_server: &MockServer) {
        // Mock health endpoints
        Mock::given(method("GET"))
            .and(path("/auth/health"))
            .respond_with(ResponseTemplate::new(200).set_body_json(MockResponses::service_health()))
            .mount(mock_server)
            .await;

        Mock::given(method("GET"))
            .and(path("/sample/health"))
            .respond_with(ResponseTemplate::new(200).set_body_json(MockResponses::service_health()))
            .mount(mock_server)
            .await;

        Mock::given(method("GET"))
            .and(path("/storage/health"))
            .respond_with(ResponseTemplate::new(200).set_body_json(MockResponses::service_health()))
            .mount(mock_server)
            .await;

        Mock::given(method("GET"))
            .and(path("/sequencing/health"))
            .respond_with(ResponseTemplate::new(200).set_body_json(MockResponses::service_health()))
            .mount(mock_server)
            .await;
    }

    pub async fn setup_sample_service_mocks(mock_server: &MockServer) {
        Mock::given(method("GET"))
            .and(path("/sample/metrics"))
            .respond_with(ResponseTemplate::new(200).set_body_json(MockResponses::sample_metrics()))
            .mount(mock_server)
            .await;
    }

    pub async fn setup_storage_service_mocks(mock_server: &MockServer) {
        Mock::given(method("GET"))
            .and(path("/storage/metrics"))
            .respond_with(ResponseTemplate::new(200).set_body_json(MockResponses::storage_metrics()))
            .mount(mock_server)
            .await;
    }

    pub async fn setup_sequencing_service_mocks(mock_server: &MockServer) {
        Mock::given(method("GET"))
            .and(path("/sequencing/metrics"))
            .respond_with(ResponseTemplate::new(200).set_body_json(MockResponses::sequencing_metrics()))
            .mount(mock_server)
            .await;
    }
}

/// Test data generators
pub struct TestDataGenerator;

impl TestDataGenerator {
    pub fn sample_dashboard_data() -> DashboardData {
        DashboardData {
            timestamp: chrono::Utc::now(),
            data: serde_json::json!({
                "widgets": [
                    {
                        "id": "sample-count",
                        "type": "metric",
                        "title": "Total Samples",
                        "value": 1234
                    },
                    {
                        "id": "storage-gauge",
                        "type": "gauge",
                        "title": "Storage Utilization",
                        "value": 78.5,
                        "max": 100
                    }
                ]
            }),
            ttl_seconds: 300,
        }
    }

    pub fn custom_dashboard_config() -> serde_json::Value {
        serde_json::json!({
            "name": "Lab Performance Dashboard",
            "description": "Custom dashboard for lab performance metrics",
            "widgets": [
                {
                    "type": "line_chart",
                    "data_source": "sample_throughput",
                    "position": {"x": 0, "y": 0, "w": 6, "h": 4}
                },
                {
                    "type": "pie_chart",
                    "data_source": "sample_types",
                    "position": {"x": 6, "y": 0, "w": 6, "h": 4}
                }
            ],
            "refresh_interval_seconds": 60
        })
    }
}

/// Cache test helpers
pub struct CacheTestUtils;

impl CacheTestUtils {
    pub fn create_test_cache() -> moka::future::Cache<String, DashboardData> {
        moka::future::Cache::builder()
            .time_to_live(Duration::from_secs(60))
            .max_capacity(100)
            .build()
    }

    pub async fn populate_cache(cache: &moka::future::Cache<String, DashboardData>) {
        cache.insert(
            "test_key_1".to_string(),
            TestDataGenerator::sample_dashboard_data(),
        ).await;

        cache.insert(
            "test_key_2".to_string(),
            DashboardData {
                timestamp: chrono::Utc::now(),
                data: serde_json::json!({"test": "data"}),
                ttl_seconds: 120,
            },
        ).await;
    }
}

/// Helper to create test app state
pub async fn create_test_app_state(mock_server_url: &str) -> AppState {
    let test_db = TestDatabase::new().await;
    let settings = SettingsFactory::create_test_settings(mock_server_url);
    let http_client = reqwest::Client::builder()
        .timeout(Duration::from_secs(5))
        .build()
        .unwrap();
    let cache = CacheTestUtils::create_test_cache();

    AppState {
        pool: test_db.pool,
        http_client,
        settings,
        cache,
    }
}

/// Performance test utilities
pub struct PerformanceTestUtils;

impl PerformanceTestUtils {
    pub async fn measure_aggregation_time<F, Fut>(operation: F) -> Duration
    where
        F: FnOnce() -> Fut,
        Fut: std::future::Future,
    {
        let start = std::time::Instant::now();
        operation().await;
        start.elapsed()
    }
}

/// Dashboard assertions
pub struct DashboardAssertions;

impl DashboardAssertions {
    pub fn assert_valid_dashboard_response(response: &serde_json::Value) {
        assert!(response.is_object());
        assert!(response.get("timestamp").is_some());
        assert!(response["timestamp"].is_string());
    }

    pub fn assert_metrics_format(metrics: &serde_json::Value) {
        assert!(metrics.is_object());
        // Add specific metric format validations
    }

    pub fn assert_cache_hit(cache_stats: &moka::future::CacheStats) {
        assert!(cache_stats.hits() > 0);
    }

    pub fn assert_service_aggregation_complete(dashboard_data: &serde_json::Value) {
        // Verify all expected service data is present
        assert!(dashboard_data.get("sample_metrics").is_some());
        assert!(dashboard_data.get("storage_metrics").is_some());
        assert!(dashboard_data.get("sequencing_metrics").is_some());
    }
}

/// Test macro for dashboard tests
#[macro_export]
macro_rules! test_with_mock_services {
    ($test_name:ident, $test_body:expr) => {
        #[tokio::test]
        async fn $test_name() {
            let mock_server = MockServer::start().await;
            let app_state = crate::test_utils::create_test_app_state(&mock_server.uri()).await;
            
            let result = std::panic::AssertUnwindSafe($test_body(&app_state, &mock_server))
                .catch_unwind()
                .await;
            
            if let Err(panic) = result {
                std::panic::resume_unwind(panic);
            }
        }
    };
}