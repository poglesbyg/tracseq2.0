use spreadsheet_versioning_service::{models::*, Config, SpreadsheetVersioningService};
use axum::{http::StatusCode, Router};
use axum_test::TestServer;
use fake::{Fake, Faker};
use serde_json::Value;
use sqlx::PgPool;
use std::sync::Arc;
use uuid::Uuid;

/// Test database manager for isolated spreadsheet versioning testing
pub struct TestDatabase {
    pub pool: PgPool,
    pub cleanup_spreadsheets: Vec<Uuid>,
    pub cleanup_versions: Vec<Uuid>,
    pub cleanup_changes: Vec<Uuid>,
}

impl TestDatabase {
    pub async fn new() -> Self {
        let pool = get_test_db().await.clone();
        Self {
            pool,
            cleanup_spreadsheets: Vec::new(),
            cleanup_versions: Vec::new(),
            cleanup_changes: Vec::new(),
        }
    }

    pub async fn cleanup(&mut self) {
        for change_id in &self.cleanup_changes {
            let _ = sqlx::query("DELETE FROM spreadsheet_changes WHERE id = $1")
                .bind(change_id)
                .execute(&self.pool)
                .await;
        }

        for version_id in &self.cleanup_versions {
            let _ = sqlx::query("DELETE FROM spreadsheet_versions WHERE id = $1")
                .bind(version_id)
                .execute(&self.pool)
                .await;
        }

        for spreadsheet_id in &self.cleanup_spreadsheets {
            let _ = sqlx::query("DELETE FROM spreadsheets WHERE id = $1")
                .bind(spreadsheet_id)
                .execute(&self.pool)
                .await;
        }

        self.cleanup_spreadsheets.clear();
        self.cleanup_versions.clear();
        self.cleanup_changes.clear();
    }

    pub fn track_spreadsheet(&mut self, spreadsheet_id: Uuid) {
        self.cleanup_spreadsheets.push(spreadsheet_id);
    }

    pub fn track_version(&mut self, version_id: Uuid) {
        self.cleanup_versions.push(version_id);
    }

    pub fn track_change(&mut self, change_id: Uuid) {
        self.cleanup_changes.push(change_id);
    }
}

/// Factory for creating test spreadsheet entities
pub struct SpreadsheetFactory;

impl SpreadsheetFactory {
    pub fn create_valid_spreadsheet_request() -> CreateSpreadsheetRequest {
        CreateSpreadsheetRequest {
            name: format!("Test Spreadsheet {}", Faker.fake::<String>()),
            description: Some("Test spreadsheet for versioning".to_string()),
            spreadsheet_type: SpreadsheetType::SampleTracking,
            initial_data: Self::sample_spreadsheet_data(),
            schema: Some(Self::sample_spreadsheet_schema()),
            permissions: SpreadsheetPermissions {
                owner: Uuid::new_v4(),
                editors: vec![Uuid::new_v4()],
                viewers: vec![Uuid::new_v4()],
                public_read: false,
            },
        }
    }

    pub fn create_change_request() -> CreateChangeRequest {
        CreateChangeRequest {
            spreadsheet_id: Uuid::new_v4(),
            change_type: ChangeType::CellUpdate,
            changes: vec![
                CellChange {
                    row: 1,
                    column: 1,
                    old_value: Some("Old Value".to_string()),
                    new_value: "New Value".to_string(),
                    data_type: CellDataType::Text,
                },
            ],
            author: Uuid::new_v4(),
            comment: Some("Test change".to_string()),
        }
    }

    pub fn sample_spreadsheet_data() -> serde_json::Value {
        serde_json::json!({
            "sheets": [
                {
                    "name": "Sample Data",
                    "data": [
                        ["Sample ID", "Name", "Type", "Status", "Date"],
                        ["SAM-001", "Sample 1", "DNA", "Active", "2024-01-15"],
                        ["SAM-002", "Sample 2", "RNA", "Processing", "2024-01-16"],
                        ["SAM-003", "Sample 3", "DNA", "Completed", "2024-01-17"]
                    ]
                }
            ]
        })
    }

    pub fn sample_spreadsheet_schema() -> serde_json::Value {
        serde_json::json!({
            "columns": [
                {
                    "name": "Sample ID",
                    "type": "string",
                    "required": true,
                    "unique": true
                },
                {
                    "name": "Name",
                    "type": "string",
                    "required": true
                },
                {
                    "name": "Type",
                    "type": "enum",
                    "options": ["DNA", "RNA", "Protein"]
                },
                {
                    "name": "Status",
                    "type": "enum",
                    "options": ["Active", "Processing", "Completed", "Failed"]
                },
                {
                    "name": "Date",
                    "type": "date"
                }
            ]
        })
    }
}

/// HTTP test client wrapper for spreadsheet API testing
pub struct SpreadsheetTestClient {
    pub server: TestServer,
    pub auth_token: Option<String>,
}

impl SpreadsheetTestClient {
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

/// Common assertions for spreadsheet testing
pub struct SpreadsheetAssertions;

impl SpreadsheetAssertions {
    pub fn assert_successful_creation(response: &Value) {
        assert_eq!(response["success"], true);
        assert!(response["data"]["id"].is_string());
        assert!(response["data"]["version"].is_number());
        assert!(response["data"]["created_at"].is_string());
    }

    pub fn assert_spreadsheet_data(response: &Value, expected_name: &str) {
        assert_eq!(response["success"], true);
        assert_eq!(response["data"]["name"], expected_name);
        assert!(response["data"]["id"].is_string());
        assert!(response["data"]["spreadsheet_type"].is_string());
    }

    pub fn assert_version_history(response: &Value) {
        assert_eq!(response["success"], true);
        assert!(response["data"]["versions"].is_array());
        let versions = response["data"]["versions"].as_array().unwrap();
        assert!(!versions.is_empty());
        
        for version in versions {
            assert!(version["version_number"].is_number());
            assert!(version["created_at"].is_string());
            assert!(version["author"].is_string());
        }
    }

    pub fn assert_change_tracking(response: &Value) {
        assert_eq!(response["success"], true);
        assert!(response["data"]["changes"].is_array());
        let changes = response["data"]["changes"].as_array().unwrap();
        
        for change in changes {
            assert!(change["change_type"].is_string());
            assert!(change["author"].is_string());
            assert!(change["timestamp"].is_string());
        }
    }

    pub fn assert_validation_error(response: &Value) {
        assert_eq!(response["success"], false);
        assert!(response["error"].is_string());
    }
}

/// Test data generators for spreadsheet scenarios
pub struct SpreadsheetTestDataGenerator;

impl SpreadsheetTestDataGenerator {
    pub fn spreadsheet_types() -> Vec<SpreadsheetType> {
        vec![
            SpreadsheetType::SampleTracking,
            SpreadsheetType::ExperimentData,
            SpreadsheetType::QualityControl,
            SpreadsheetType::Inventory,
            SpreadsheetType::Report,
        ]
    }

    pub fn change_types() -> Vec<ChangeType> {
        vec![
            ChangeType::CellUpdate,
            ChangeType::RowInsert,
            ChangeType::RowDelete,
            ChangeType::ColumnInsert,
            ChangeType::ColumnDelete,
            ChangeType::FormatChange,
        ]
    }

    pub fn cell_data_types() -> Vec<CellDataType> {
        vec![
            CellDataType::Text,
            CellDataType::Number,
            CellDataType::Date,
            CellDataType::Boolean,
            CellDataType::Formula,
        ]
    }

    pub fn generate_bulk_changes(count: usize) -> Vec<CellChange> {
        (0..count)
            .map(|i| CellChange {
                row: i as u32 + 1,
                column: 1,
                old_value: Some(format!("Old Value {}", i)),
                new_value: format!("New Value {}", i),
                data_type: CellDataType::Text,
            })
            .collect()
    }

    pub fn generate_large_spreadsheet_data(rows: usize, cols: usize) -> serde_json::Value {
        let mut data = Vec::new();
        
        // Header row
        let header: Vec<String> = (0..cols).map(|i| format!("Column {}", i + 1)).collect();
        data.push(header);
        
        // Data rows
        for row in 0..rows {
            let row_data: Vec<String> = (0..cols).map(|col| format!("Cell({},{})", row + 1, col + 1)).collect();
            data.push(row_data);
        }
        
        serde_json::json!({
            "sheets": [
                {
                    "name": "Large Dataset",
                    "data": data
                }
            ]
        })
    }
}

/// Performance testing utilities for spreadsheet operations
pub struct SpreadsheetPerformanceUtils;

impl SpreadsheetPerformanceUtils {
    pub async fn measure_bulk_update_time(
        client: &SpreadsheetTestClient,
        spreadsheet_id: Uuid,
        changes: &[CellChange],
    ) -> std::time::Duration {
        let start = std::time::Instant::now();
        let update_request = serde_json::json!({
            "changes": changes
        });
        let _ = client.put_json(&format!("/api/spreadsheets/{}/bulk-update", spreadsheet_id), &update_request).await;
        start.elapsed()
    }

    pub async fn concurrent_edit_test(
        client: &SpreadsheetTestClient,
        spreadsheet_id: Uuid,
        concurrent_count: usize,
    ) -> Vec<StatusCode> {
        let tasks: Vec<_> = (0..concurrent_count)
            .map(|i| {
                let change = CellChange {
                    row: i as u32 + 1,
                    column: 1,
                    old_value: None,
                    new_value: format!("Concurrent Edit {}", i),
                    data_type: CellDataType::Text,
                };
                let update_request = serde_json::json!({
                    "changes": [change]
                });
                async move {
                    client.put_json(&format!("/api/spreadsheets/{}/update", spreadsheet_id), &update_request).await.status_code()
                }
            })
            .collect();

        futures::future::join_all(tasks).await
    }
}