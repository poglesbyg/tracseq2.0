use library_details_service::{models::*, Config, LibraryDetailsService};
use axum::{http::StatusCode, Router};
use axum_test::TestServer;
use fake::{Fake, Faker};
use serde_json::Value;
use sqlx::PgPool;
use std::sync::Arc;
use uuid::Uuid;

/// Test database manager for isolated library details testing
pub struct TestDatabase {
    pub pool: PgPool,
    pub cleanup_libraries: Vec<Uuid>,
    pub cleanup_protocols: Vec<Uuid>,
    pub cleanup_metrics: Vec<Uuid>,
}

impl TestDatabase {
    pub async fn new() -> Self {
        let pool = get_test_db().await.clone();
        Self {
            pool,
            cleanup_libraries: Vec::new(),
            cleanup_protocols: Vec::new(),
            cleanup_metrics: Vec::new(),
        }
    }

    pub async fn cleanup(&mut self) {
        for metric_id in &self.cleanup_metrics {
            let _ = sqlx::query("DELETE FROM library_metrics WHERE id = $1")
                .bind(metric_id)
                .execute(&self.pool)
                .await;
        }

        for library_id in &self.cleanup_libraries {
            let _ = sqlx::query("DELETE FROM library_details WHERE id = $1")
                .bind(library_id)
                .execute(&self.pool)
                .await;
        }

        for protocol_id in &self.cleanup_protocols {
            let _ = sqlx::query("DELETE FROM library_protocols WHERE id = $1")
                .bind(protocol_id)
                .execute(&self.pool)
                .await;
        }

        self.cleanup_libraries.clear();
        self.cleanup_protocols.clear();
        self.cleanup_metrics.clear();
    }

    pub fn track_library(&mut self, library_id: Uuid) {
        self.cleanup_libraries.push(library_id);
    }

    pub fn track_protocol(&mut self, protocol_id: Uuid) {
        self.cleanup_protocols.push(protocol_id);
    }

    pub fn track_metric(&mut self, metric_id: Uuid) {
        self.cleanup_metrics.push(metric_id);
    }
}

/// Factory for creating test library entities
pub struct LibraryFactory;

impl LibraryFactory {
    pub fn create_valid_library_request() -> CreateLibraryRequest {
        CreateLibraryRequest {
            sample_id: Uuid::new_v4(),
            library_name: format!("Test Library {}", Faker.fake::<String>()),
            library_type: LibraryType::DNASeq,
            preparation_method: PreparationMethod::TruSeq,
            adapter_sequence: Some("AGATCGGAAGAGC".to_string()),
            index_sequence: Some("ATCGATCG".to_string()),
            target_insert_size: Some(350),
            concentration: Some(25.5),
            volume: Some(30.0),
            quantification_method: Some(QuantificationMethod::Qubit),
            preparation_date: chrono::Utc::now().date_naive(),
            prepared_by: Uuid::new_v4(),
            notes: Some("Test library preparation".to_string()),
        }
    }

    pub fn create_valid_protocol_request() -> CreateProtocolRequest {
        CreateProtocolRequest {
            name: format!("Test Protocol {}", Faker.fake::<String>()),
            version: "1.0.0".to_string(),
            description: Some("Test library preparation protocol".to_string()),
            protocol_type: ProtocolType::Library,
            steps: vec![
                ProtocolStep {
                    step_number: 1,
                    title: "DNA Fragmentation".to_string(),
                    description: "Fragment DNA to target size".to_string(),
                    duration_minutes: 30,
                    temperature: Some(37.0),
                    reagents: vec!["Fragmentation Buffer".to_string()],
                    equipment: vec!["Thermocycler".to_string()],
                },
                ProtocolStep {
                    step_number: 2,
                    title: "End Repair".to_string(),
                    description: "Repair DNA ends and add A-tails".to_string(),
                    duration_minutes: 45,
                    temperature: Some(65.0),
                    reagents: vec!["End Repair Mix".to_string()],
                    equipment: vec!["Thermocycler".to_string()],
                },
            ],
            is_active: true,
        }
    }

    pub fn create_quality_metrics() -> LibraryQualityMetrics {
        LibraryQualityMetrics {
            concentration: 25.5,
            molarity: 15.2,
            fragment_size_bp: 350,
            adapter_dimer_percentage: 2.1,
            gc_content: 45.2,
            quality_score: 8.5,
            pass_fail_status: QualityStatus::Pass,
            measurement_date: chrono::Utc::now(),
            measured_by: Uuid::new_v4(),
        }
    }
}

/// HTTP test client wrapper for library details API testing
pub struct LibraryTestClient {
    pub server: TestServer,
    pub auth_token: Option<String>,
}

impl LibraryTestClient {
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

/// Common assertions for library testing
pub struct LibraryAssertions;

impl LibraryAssertions {
    pub fn assert_successful_creation(response: &Value) {
        assert_eq!(response["success"], true);
        assert!(response["data"]["id"].is_string());
        assert!(response["data"]["created_at"].is_string());
    }

    pub fn assert_library_data(response: &Value, expected_name: &str) {
        assert_eq!(response["success"], true);
        assert_eq!(response["data"]["library_name"], expected_name);
        assert!(response["data"]["sample_id"].is_string());
        assert!(response["data"]["library_type"].is_string());
    }

    pub fn assert_quality_metrics(response: &Value) {
        assert_eq!(response["success"], true);
        assert!(response["data"]["concentration"].is_number());
        assert!(response["data"]["quality_score"].is_number());
        assert!(response["data"]["pass_fail_status"].is_string());
    }

    pub fn assert_validation_error(response: &Value) {
        assert_eq!(response["success"], false);
        assert!(response["error"].is_string());
    }
}

/// Test data generators for library scenarios
pub struct LibraryTestDataGenerator;

impl LibraryTestDataGenerator {
    pub fn library_types() -> Vec<LibraryType> {
        vec![
            LibraryType::DNASeq,
            LibraryType::RNASeq,
            LibraryType::ChIPSeq,
            LibraryType::ATACSeq,
            LibraryType::Amplicon,
        ]
    }

    pub fn preparation_methods() -> Vec<PreparationMethod> {
        vec![
            PreparationMethod::TruSeq,
            PreparationMethod::Nextera,
            PreparationMethod::KAPA,
            PreparationMethod::Custom,
        ]
    }

    pub fn quantification_methods() -> Vec<QuantificationMethod> {
        vec![
            QuantificationMethod::Qubit,
            QuantificationMethod::Nanodrop,
            QuantificationMethod::qPCR,
            QuantificationMethod::Bioanalyzer,
        ]
    }

    pub fn generate_failing_metrics() -> LibraryQualityMetrics {
        LibraryQualityMetrics {
            concentration: 2.5, // Too low
            molarity: 1.2, // Too low
            fragment_size_bp: 150, // Too small
            adapter_dimer_percentage: 25.0, // Too high
            gc_content: 25.0, // Unusual
            quality_score: 3.2, // Poor quality
            pass_fail_status: QualityStatus::Fail,
            measurement_date: chrono::Utc::now(),
            measured_by: Uuid::new_v4(),
        }
    }
}