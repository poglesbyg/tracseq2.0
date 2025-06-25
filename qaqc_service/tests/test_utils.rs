use qaqc_service::{models::*, Config, QAQCService};
use axum::{http::StatusCode, Router};
use axum_test::TestServer;
use fake::{Fake, Faker};
use serde_json::Value;
use sqlx::PgPool;
use std::sync::Arc;
use uuid::Uuid;

/// Test database manager for isolated QAQC testing
pub struct TestDatabase {
    pub pool: PgPool,
    pub cleanup_checks: Vec<Uuid>,
    pub cleanup_rules: Vec<Uuid>,
    pub cleanup_results: Vec<Uuid>,
}

impl TestDatabase {
    pub async fn new() -> Self {
        let pool = get_test_db().await.clone();
        Self {
            pool,
            cleanup_checks: Vec::new(),
            cleanup_rules: Vec::new(),
            cleanup_results: Vec::new(),
        }
    }

    pub async fn cleanup(&mut self) {
        for result_id in &self.cleanup_results {
            let _ = sqlx::query("DELETE FROM qc_results WHERE id = $1")
                .bind(result_id)
                .execute(&self.pool)
                .await;
        }

        for check_id in &self.cleanup_checks {
            let _ = sqlx::query("DELETE FROM qc_checks WHERE id = $1")
                .bind(check_id)
                .execute(&self.pool)
                .await;
        }

        for rule_id in &self.cleanup_rules {
            let _ = sqlx::query("DELETE FROM qc_rules WHERE id = $1")
                .bind(rule_id)
                .execute(&self.pool)
                .await;
        }

        self.cleanup_checks.clear();
        self.cleanup_rules.clear();
        self.cleanup_results.clear();
    }

    pub fn track_check(&mut self, check_id: Uuid) {
        self.cleanup_checks.push(check_id);
    }

    pub fn track_rule(&mut self, rule_id: Uuid) {
        self.cleanup_rules.push(rule_id);
    }

    pub fn track_result(&mut self, result_id: Uuid) {
        self.cleanup_results.push(result_id);
    }
}

/// Factory for creating test QAQC entities
pub struct QAQCFactory;

impl QAQCFactory {
    pub fn create_valid_qc_rule_request() -> CreateQCRuleRequest {
        CreateQCRuleRequest {
            name: format!("Test QC Rule {}", Faker.fake::<String>()),
            description: Some("Test quality control rule".to_string()),
            rule_type: QCRuleType::Threshold,
            category: QCCategory::SampleQuality,
            parameters: serde_json::json!({
                "metric": "concentration",
                "min_value": 10.0,
                "max_value": 1000.0,
                "unit": "ng/µL"
            }),
            severity: QCSeverity::Warning,
            is_active: true,
            auto_apply: true,
        }
    }

    pub fn create_valid_qc_check_request() -> CreateQCCheckRequest {
        CreateQCCheckRequest {
            name: format!("Test QC Check {}", Faker.fake::<String>()),
            description: Some("Automated QC check".to_string()),
            check_type: QCCheckType::Automated,
            target_entity: QCTargetEntity::Sample,
            rules: vec![Uuid::new_v4()],
            schedule: Some(QCSchedule {
                frequency: QCFrequency::OnDemand,
                time_of_day: None,
                days_of_week: None,
            }),
            is_active: true,
        }
    }

    pub fn create_sample_data_for_qc() -> serde_json::Value {
        serde_json::json!({
            "sample_id": Uuid::new_v4(),
            "concentration": 150.5,
            "purity_260_280": 1.85,
            "purity_260_230": 2.1,
            "volume": 50.0,
            "quality_score": 98.5,
            "contamination_level": 0.02
        })
    }
}

/// HTTP test client wrapper for QAQC API testing
pub struct QAQCTestClient {
    pub server: TestServer,
    pub auth_token: Option<String>,
}

impl QAQCTestClient {
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

/// Common assertions for QAQC testing
pub struct QAQCAssertions;

impl QAQCAssertions {
    pub fn assert_successful_creation(response: &Value) {
        assert_eq!(response["success"], true);
        assert!(response["data"]["id"].is_string());
        assert!(response["data"]["created_at"].is_string());
    }

    pub fn assert_qc_rule_data(response: &Value, expected_name: &str) {
        assert_eq!(response["success"], true);
        assert_eq!(response["data"]["name"], expected_name);
        assert!(response["data"]["rule_type"].is_string());
        assert!(response["data"]["parameters"].is_object());
    }

    pub fn assert_qc_result(response: &Value, expected_status: &str) {
        assert_eq!(response["success"], true);
        assert_eq!(response["data"]["status"], expected_status);
        assert!(response["data"]["score"].is_number());
        assert!(response["data"]["violations"].is_array());
    }

    pub fn assert_validation_error(response: &Value) {
        assert_eq!(response["success"], false);
        assert!(response["error"].is_string());
    }
}

/// Test data generators for QAQC scenarios
pub struct QAQCTestDataGenerator;

impl QAQCTestDataGenerator {
    pub fn qc_rule_types() -> Vec<QCRuleType> {
        vec![
            QCRuleType::Threshold,
            QCRuleType::Range,
            QCRuleType::Pattern,
            QCRuleType::Statistical,
            QCRuleType::Custom,
        ]
    }

    pub fn qc_categories() -> Vec<QCCategory> {
        vec![
            QCCategory::SampleQuality,
            QCCategory::DataIntegrity,
            QCCategory::ProcessCompliance,
            QCCategory::Equipment,
            QCCategory::Environmental,
        ]
    }

    pub fn qc_severities() -> Vec<QCSeverity> {
        vec![
            QCSeverity::Info,
            QCSeverity::Warning,
            QCSeverity::Error,
            QCSeverity::Critical,
        ]
    }

    pub fn generate_failing_sample_data() -> serde_json::Value {
        serde_json::json!({
            "sample_id": Uuid::new_v4(),
            "concentration": 5.0, // Below threshold
            "purity_260_280": 1.2, // Below acceptable range
            "purity_260_230": 0.8, // Below acceptable range
            "volume": 10.0, // Low volume
            "quality_score": 45.0, // Low quality
            "contamination_level": 0.15 // High contamination
        })
    }
}

/// Performance testing utilities for QAQC operations
pub struct QAQCPerformanceUtils;

impl QAQCPerformanceUtils {
    pub async fn measure_qc_check_time(
        client: &QAQCTestClient,
        check_id: Uuid,
        data: &serde_json::Value,
    ) -> std::time::Duration {
        let start = std::time::Instant::now();
        let _ = client.post_json(&format!("/api/qc/checks/{}/run", check_id), data).await;
        start.elapsed()
    }

    pub async fn concurrent_qc_checks(
        client: &QAQCTestClient,
        check_id: Uuid,
        concurrent_count: usize,
    ) -> Vec<StatusCode> {
        let tasks: Vec<_> = (0..concurrent_count)
            .map(|_| {
                let data = QAQCFactory::create_sample_data_for_qc();
                async move {
                    client.post_json(&format!("/api/qc/checks/{}/run", check_id), &data).await.status_code()
                }
            })
            .collect();

        futures::future::join_all(tasks).await
    }
}