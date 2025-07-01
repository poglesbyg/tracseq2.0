//! Test utilities for Reports Service

use reports_service::{AppState, Settings, ServiceUrls, StorageConfig, create_router};
use sqlx::{postgres::PgPoolOptions, PgPool};
use std::sync::Arc;
use uuid::Uuid;
use wiremock::{Mock, MockServer, ResponseTemplate};
use wiremock::matchers::{method, path, header};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Test database management
pub struct TestDatabase {
    pub pool: PgPool,
    database_name: String,
}

impl TestDatabase {
    pub async fn new() -> Self {
        let database_url = std::env::var("TEST_DATABASE_URL")
            .unwrap_or_else(|_| "postgres://reports_user:reports_pass@localhost/postgres".to_string());
        
        let database_name = format!("test_reports_{}", Uuid::new_v4().to_string().replace("-", ""));
        
        // Create database
        let conn = PgPoolOptions::new()
            .max_connections(1)
            .connect(&database_url)
            .await
            .expect("Failed to connect to Postgres");
        
        sqlx::query(&format!("CREATE DATABASE {}", database_name))
            .execute(&conn)
            .await
            .expect("Failed to create test database");
        
        // Connect to the new database
        let test_database_url = database_url.replace("/postgres", &format!("/{}", database_name));
        let pool = PgPoolOptions::new()
            .max_connections(5)
            .connect(&test_database_url)
            .await
            .expect("Failed to connect to test database");
        
        // Run migrations
        sqlx::migrate!("./migrations")
            .run(&pool)
            .await
            .expect("Failed to run migrations");
        
        Self { pool, database_name }
    }
    
    pub async fn cleanup(self) {
        // Drop all connections
        self.pool.close().await;
        
        // Drop the database
        let database_url = std::env::var("TEST_DATABASE_URL")
            .unwrap_or_else(|_| "postgres://reports_user:reports_pass@localhost/postgres".to_string());
        
        let conn = PgPoolOptions::new()
            .max_connections(1)
            .connect(&database_url)
            .await
            .expect("Failed to connect to Postgres");
        
        sqlx::query(&format!("DROP DATABASE IF EXISTS {}", self.database_name))
            .execute(&conn)
            .await
            .expect("Failed to drop test database");
    }
}

/// Test application setup
pub async fn create_test_app() -> TestApp {
    let test_db = TestDatabase::new().await;
    let mock_server = MockServer::start().await;
    
    let settings = Settings {
        database_url: "unused".to_string(), // We'll use the test DB pool directly
        port: 0, // Random port
        service_urls: ServiceUrls {
            auth_service: mock_server.uri(),
            sample_service: mock_server.uri(),
            storage_service: mock_server.uri(),
            sequencing_service: mock_server.uri(),
            dashboard_service: mock_server.uri(),
        },
        storage: StorageConfig {
            reports_path: "/tmp/test_reports".to_string(),
            templates_path: "/tmp/test_templates".to_string(),
            retention_days: 7,
        },
    };
    
    // Create test template engine
    let mut template_engine = tera::Tera::default();
    template_engine.add_raw_template("test.html", "Test template: {{ content }}").unwrap();
    
    // Create test scheduler
    let scheduler = tokio_cron_scheduler::JobScheduler::new()
        .await
        .expect("Failed to create scheduler");
    
    let app_state = Arc::new(AppState {
        pool: test_db.pool.clone(),
        http_client: reqwest::Client::new(),
        settings: settings.clone(),
        template_engine,
        scheduler: Arc::new(scheduler),
    });
    
    let router = create_router(app_state);
    
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0")
        .await
        .expect("Failed to bind to port");
    let addr = listener.local_addr().unwrap();
    
    tokio::spawn(async move {
        axum::Server::from_tcp(listener)
            .unwrap()
            .serve(router.into_make_service())
            .await
            .unwrap();
    });
    
    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(10))
        .build()
        .unwrap();
    
    TestApp {
        address: format!("http://{}", addr),
        test_db,
        mock_server,
        client,
        settings,
    }
}

pub struct TestApp {
    pub address: String,
    pub test_db: TestDatabase,
    pub mock_server: MockServer,
    pub client: reqwest::Client,
    pub settings: Settings,
}

impl TestApp {
    pub fn url(&self, path: &str) -> String {
        format!("{}{}", self.address, path)
    }
    
    pub async fn get(&self, path: &str) -> reqwest::Response {
        self.client
            .get(&self.url(path))
            .send()
            .await
            .expect("Failed to send request")
    }
    
    pub async fn post(&self, path: &str, json: &serde_json::Value) -> reqwest::Response {
        self.client
            .post(&self.url(path))
            .json(json)
            .send()
            .await
            .expect("Failed to send request")
    }
    
    pub async fn put(&self, path: &str, json: &serde_json::Value) -> reqwest::Response {
        self.client
            .put(&self.url(path))
            .json(json)
            .send()
            .await
            .expect("Failed to send request")
    }
    
    pub async fn delete(&self, path: &str) -> reqwest::Response {
        self.client
            .delete(&self.url(path))
            .send()
            .await
            .expect("Failed to send request")
    }
}

/// Factory functions for creating test data
pub struct ReportFactory;

impl ReportFactory {
    pub fn create_report_request() -> serde_json::Value {
        serde_json::json!({
            "template_id": "sample-summary",
            "title": "Sample Processing Report",
            "parameters": {
                "start_date": "2024-01-01",
                "end_date": "2024-01-31",
                "department": "molecular"
            },
            "format": "pdf"
        })
    }
    
    pub fn create_sample_analytics_report() -> serde_json::Value {
        serde_json::json!({
            "total_samples": 1500,
            "processed_samples": 1450,
            "failed_samples": 50,
            "processing_rate": 96.67,
            "by_type": {
                "dna": 800,
                "rna": 500,
                "protein": 200
            },
            "by_status": {
                "received": 100,
                "processing": 250,
                "completed": 1100,
                "failed": 50
            },
            "time_metrics": {
                "average_processing_time_hours": 48.5,
                "min_processing_time_hours": 24,
                "max_processing_time_hours": 120
            }
        })
    }
    
    pub fn create_schedule_request() -> serde_json::Value {
        serde_json::json!({
            "name": "Weekly Sample Report",
            "template_id": "sample-summary",
            "cron_expression": "0 0 * * MON",
            "parameters": {
                "period": "weekly",
                "recipients": ["lab-manager@example.com"]
            },
            "enabled": true
        })
    }
    
    pub fn create_custom_query() -> serde_json::Value {
        serde_json::json!({
            "name": "Active Samples Query",
            "sql": "SELECT * FROM samples WHERE status = 'processing' ORDER BY created_at DESC",
            "parameters": {},
            "description": "Get all samples currently being processed"
        })
    }
}

/// Mock service responses
pub struct MockServiceResponses;

impl MockServiceResponses {
    pub async fn setup_sample_service_mocks(mock_server: &MockServer) {
        Mock::given(method("GET"))
            .and(path("/api/samples/stats"))
            .respond_with(ResponseTemplate::new(200)
                .set_body_json(serde_json::json!({
                    "total": 1500,
                    "active": 350,
                    "completed": 1100,
                    "failed": 50
                })))
            .mount(mock_server)
            .await;
        
        Mock::given(method("GET"))
            .and(path("/api/samples"))
            .respond_with(ResponseTemplate::new(200)
                .set_body_json(serde_json::json!({
                    "samples": [
                        {
                            "id": "sample-001",
                            "type": "dna",
                            "status": "processing",
                            "created_at": "2024-01-15T10:00:00Z"
                        }
                    ],
                    "total": 1
                })))
            .mount(mock_server)
            .await;
    }
    
    pub async fn setup_storage_service_mocks(mock_server: &MockServer) {
        Mock::given(method("GET"))
            .and(path("/api/storage/stats"))
            .respond_with(ResponseTemplate::new(200)
                .set_body_json(serde_json::json!({
                    "total_capacity": 10000,
                    "used_capacity": 7500,
                    "utilization_percent": 75.0,
                    "by_temperature": {
                        "-80": {"used": 3000, "total": 4000},
                        "-20": {"used": 2000, "total": 3000},
                        "4": {"used": 1500, "total": 2000},
                        "RT": {"used": 1000, "total": 1000}
                    }
                })))
            .mount(mock_server)
            .await;
    }
    
    pub async fn setup_sequencing_service_mocks(mock_server: &MockServer) {
        Mock::given(method("GET"))
            .and(path("/api/sequencing/runs/stats"))
            .respond_with(ResponseTemplate::new(200)
                .set_body_json(serde_json::json!({
                    "total_runs": 250,
                    "successful_runs": 240,
                    "failed_runs": 10,
                    "average_quality_score": 35.5,
                    "total_bases": 1500000000,
                    "by_platform": {
                        "illumina": 200,
                        "nanopore": 50
                    }
                })))
            .mount(mock_server)
            .await;
    }
}

/// Test data generators
pub struct TestDataGenerator;

impl TestDataGenerator {
    pub fn generate_report_data(count: usize) -> Vec<serde_json::Value> {
        (0..count)
            .map(|i| serde_json::json!({
                "id": format!("report-{:03}", i),
                "title": format!("Test Report {}", i),
                "template_id": "sample-summary",
                "created_at": Utc::now() - chrono::Duration::days(i as i64),
                "status": if i % 3 == 0 { "completed" } else { "generating" },
                "format": if i % 2 == 0 { "pdf" } else { "excel" },
                "size_bytes": 1024 * (i + 1),
                "parameters": {
                    "start_date": "2024-01-01",
                    "end_date": "2024-01-31"
                }
            }))
            .collect()
    }
    
    pub fn generate_template_data() -> Vec<serde_json::Value> {
        vec![
            serde_json::json!({
                "id": "sample-summary",
                "name": "Sample Summary Report",
                "description": "Comprehensive sample processing summary",
                "category": "operational",
                "fields": ["sample_id", "type", "status", "created_at"],
                "parameters": {
                    "date_range": "required",
                    "department": "optional"
                }
            }),
            serde_json::json!({
                "id": "sequencing-metrics",
                "name": "Sequencing Metrics Report",
                "description": "Detailed sequencing performance analysis",
                "category": "technical",
                "fields": ["run_id", "quality_score", "read_count", "error_rate"],
                "parameters": {
                    "platform": "optional",
                    "quality_threshold": "optional"
                }
            }),
            serde_json::json!({
                "id": "financial-summary",
                "name": "Financial Summary Report",
                "description": "Cost analysis and billing summary",
                "category": "financial",
                "fields": ["project_id", "cost", "revenue", "profit_margin"],
                "parameters": {
                    "fiscal_period": "required",
                    "cost_center": "optional"
                }
            }),
        ]
    }
    
    pub fn generate_analytics_data() -> HashMap<String, serde_json::Value> {
        let mut data = HashMap::new();
        
        data.insert("sample_trends".to_string(), serde_json::json!({
            "daily": [
                {"date": "2024-01-01", "count": 45},
                {"date": "2024-01-02", "count": 52},
                {"date": "2024-01-03", "count": 48},
            ],
            "weekly": [
                {"week": "2024-W01", "count": 320},
                {"week": "2024-W02", "count": 345},
            ],
            "monthly": [
                {"month": "2024-01", "count": 1450},
            ]
        }));
        
        data.insert("performance_metrics".to_string(), serde_json::json!({
            "turnaround_time": {
                "average_hours": 48.5,
                "p50_hours": 45.0,
                "p90_hours": 72.0,
                "p99_hours": 120.0
            },
            "success_rate": {
                "overall": 96.67,
                "by_type": {
                    "dna": 97.5,
                    "rna": 95.0,
                    "protein": 98.0
                }
            },
            "throughput": {
                "samples_per_day": 48.33,
                "peak_day": 75,
                "capacity_utilization": 0.85
            }
        }));
        
        data
    }
}

/// Assertion helpers
pub struct ReportAssertions;

impl ReportAssertions {
    pub fn assert_report_structure(report: &serde_json::Value) {
        assert!(report.get("id").is_some());
        assert!(report.get("title").is_some());
        assert!(report.get("created_at").is_some());
        assert!(report.get("status").is_some());
        
        let status = report["status"].as_str().unwrap();
        assert!(["generating", "completed", "failed", "scheduled"].contains(&status));
    }
    
    pub fn assert_template_structure(template: &serde_json::Value) {
        assert!(template.get("id").is_some());
        assert!(template.get("name").is_some());
        assert!(template.get("description").is_some());
        assert!(template.get("category").is_some());
        assert!(template.get("fields").is_some());
    }
    
    pub fn assert_schedule_structure(schedule: &serde_json::Value) {
        assert!(schedule.get("id").is_some());
        assert!(schedule.get("name").is_some());
        assert!(schedule.get("template_id").is_some());
        assert!(schedule.get("cron_expression").is_some());
        assert!(schedule.get("enabled").is_some());
        assert!(schedule.get("next_run").is_some());
    }
    
    pub fn assert_analytics_structure(analytics: &serde_json::Value) {
        // Check for common analytics fields
        assert!(analytics.is_object());
        let obj = analytics.as_object().unwrap();
        assert!(!obj.is_empty());
    }
}

/// Performance test utilities
pub struct PerformanceTestUtils;

impl PerformanceTestUtils {
    pub async fn measure_report_generation_time<F, Fut>(f: F) -> (std::time::Duration, serde_json::Value)
    where
        F: FnOnce() -> Fut,
        Fut: std::future::Future<Output = serde_json::Value>,
    {
        let start = std::time::Instant::now();
        let result = f().await;
        let duration = start.elapsed();
        (duration, result)
    }
    
    pub async fn concurrent_report_generation(
        app: &TestApp,
        count: usize,
    ) -> Vec<(std::time::Duration, reqwest::StatusCode)> {
        let mut handles = Vec::new();
        
        for i in 0..count {
            let client = app.client.clone();
            let url = app.url("/api/reports/generate");
            let request = ReportFactory::create_report_request();
            
            let handle = tokio::spawn(async move {
                let start = std::time::Instant::now();
                let response = client
                    .post(&url)
                    .json(&request)
                    .send()
                    .await
                    .expect("Failed to send request");
                let duration = start.elapsed();
                (duration, response.status())
            });
            
            handles.push(handle);
        }
        
        futures::future::join_all(handles)
            .await
            .into_iter()
            .map(|r| r.unwrap())
            .collect()
    }
}

/// Schedule test helpers
pub struct ScheduleTestHelpers;

impl ScheduleTestHelpers {
    pub fn parse_cron(expression: &str) -> Result<(), String> {
        // Basic cron validation
        let parts: Vec<&str> = expression.split_whitespace().collect();
        if parts.len() != 5 {
            return Err("Cron expression must have 5 parts".to_string());
        }
        Ok(())
    }
    
    pub fn next_execution_time(cron_expression: &str) -> Option<DateTime<Utc>> {
        // Simplified - would use a proper cron parser in real implementation
        Some(Utc::now() + chrono::Duration::hours(1))
    }
}

/// Export test helpers
pub struct ExportTestHelpers;

impl ExportTestHelpers {
    pub fn create_pdf_export_request() -> serde_json::Value {
        serde_json::json!({
            "report_id": "report-001",
            "options": {
                "page_size": "A4",
                "orientation": "portrait",
                "include_charts": true,
                "include_summary": true
            }
        })
    }
    
    pub fn create_excel_export_request() -> serde_json::Value {
        serde_json::json!({
            "report_id": "report-001",
            "options": {
                "include_formulas": true,
                "include_charts": true,
                "separate_sheets": true
            }
        })
    }
    
    pub fn create_csv_export_request() -> serde_json::Value {
        serde_json::json!({
            "data": [
                {"sample_id": "S001", "type": "DNA", "status": "completed"},
                {"sample_id": "S002", "type": "RNA", "status": "processing"},
            ],
            "options": {
                "delimiter": ",",
                "include_headers": true
            }
        })
    }
    
    pub fn verify_pdf_response(response: &reqwest::Response) {
        assert_eq!(response.status(), reqwest::StatusCode::OK);
        let content_type = response.headers().get("content-type").unwrap();
        assert_eq!(content_type, "application/pdf");
    }
    
    pub fn verify_excel_response(response: &reqwest::Response) {
        assert_eq!(response.status(), reqwest::StatusCode::OK);
        let content_type = response.headers().get("content-type").unwrap();
        assert_eq!(content_type, "application/vnd.openxmlformats-officedocument.spreadsheetml.sheet");
    }
    
    pub fn verify_csv_response(response: &reqwest::Response) {
        assert_eq!(response.status(), reqwest::StatusCode::OK);
        let content_type = response.headers().get("content-type").unwrap();
        assert_eq!(content_type, "text/csv");
    }
}