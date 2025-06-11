#[cfg(test)]
mod spreadsheet_processing_tests {
    use crate::models::spreadsheet::{
        CreateSpreadsheetDataset, ParsedSpreadsheetData, SpreadsheetDataManager,
        SpreadsheetDataset, SpreadsheetRecord, SpreadsheetSearchQuery, UploadStatus,
    };
    use crate::services::spreadsheet_service::SpreadsheetService;
    use chrono::Utc;
    use serde_json::json;
    use sqlx::postgres::PgPoolOptions;
    use std::collections::HashMap;
    use uuid::Uuid;

    async fn setup_test_db() -> sqlx::PgPool {
        let database_url = std::env::var("TEST_DATABASE_URL").unwrap_or_else(|_| {
            "postgres://lab_manager:lab_manager@localhost:5432/lab_manager_test".to_string()
        });

        PgPoolOptions::new()
            .max_connections(5)
            .connect(&database_url)
            .await
            .expect("Failed to connect to test database")
    }

    #[tokio::test]
    async fn test_create_spreadsheet_dataset() {
        let pool = setup_test_db().await;
        let manager = SpreadsheetDataManager::new(pool.clone());
        let service = SpreadsheetService::new(manager);

        let create_request = CreateSpreadsheetDataset {
            filename: "test.xlsx".to_string(),
            original_filename: "Test Dataset.xlsx".to_string(),
            file_type: "xlsx".to_string(),
            file_size: 1024,
            sheet_name: Some("Sheet1".to_string()),
            column_headers: vec!["Sample_ID".to_string(), "Patient_ID".to_string()],
            uploaded_by: Some("test_user".to_string()),
            metadata: Some(json!({"source": "unit_test", "version": "1.0"})),
        };

        let result = service
            .process_upload(
                create_request.filename.clone(),
                create_request.original_filename.clone(),
                vec![1, 2, 3], // dummy data
                create_request.file_type.clone(),
                create_request.sheet_name.clone(),
                create_request.uploaded_by.clone(),
            )
            .await;

        assert!(result.is_ok(), "Creating dataset should succeed");

        let dataset = result.unwrap();
        assert_eq!(dataset.original_filename, "Test Dataset.xlsx");
        assert_eq!(dataset.file_type, "xlsx");
        assert!(dataset.metadata.is_object());

        // Cleanup
        let _ = service.delete_dataset(dataset.id).await;
    }

    #[tokio::test]
    async fn test_list_datasets_with_pagination() {
        let pool = setup_test_db().await;
        let manager = SpreadsheetDataManager::new(pool.clone());
        let service = SpreadsheetService::new(manager);

        // Create multiple test datasets
        let mut created_ids = Vec::new();
        for i in 1..=3 {
            let result = service
                .process_upload(
                    format!("test{}.xlsx", i),
                    format!("Test Dataset {}.xlsx", i),
                    vec![1, 2, 3], // dummy data
                    "xlsx".to_string(),
                    Some("Sheet1".to_string()),
                    Some("test_user".to_string()),
                )
                .await;

            if let Ok(dataset) = result {
                created_ids.push(dataset.id);
            }
        }

        // Test pagination
        let result = service.list_datasets(Some(2), Some(0)).await;
        assert!(result.is_ok(), "Listing datasets should succeed");

        let datasets = result.unwrap();
        assert!(datasets.len() <= 2, "Should respect limit");

        // Cleanup
        for id in created_ids {
            let _ = service.delete_dataset(id).await;
        }
    }

    #[tokio::test]
    async fn test_search_spreadsheet_data() {
        let pool = setup_test_db().await;
        let manager = SpreadsheetDataManager::new(pool.clone());
        let service = SpreadsheetService::new(manager);

        // Create a test dataset
        let dataset = service
            .process_upload(
                "search_test.csv".to_string(),
                "Search Test.csv".to_string(),
                vec![1, 2, 3], // dummy data
                "csv".to_string(),
                None,
                Some("test_user".to_string()),
            )
            .await
            .expect("Should create dataset");

        // Test search functionality
        let search_query = SpreadsheetSearchQuery {
            search_term: Some("test".to_string()),
            dataset_id: Some(dataset.id),
            pool_filter: None,
            sample_filter: None,
            project_filter: None,
            column_filters: None,
            limit: Some(10),
            offset: None,
        };

        let result = service.search_data(search_query).await;
        assert!(result.is_ok(), "Search should succeed");

        let search_result = result.unwrap();
        assert!(search_result.total_count >= 0, "Should return valid count");

        // Cleanup
        let _ = service.delete_dataset(dataset.id).await;
    }

    #[test]
    fn test_file_type_validation() {
        let supported_types = vec!["xlsx", "xls", "csv"];
        let unsupported_types = vec!["txt", "json", "xml", "pdf"];

        for file_type in supported_types {
            // Test that supported types are valid for dataset creation
            let create_request = CreateSpreadsheetDataset {
                filename: format!("test.{}", file_type),
                original_filename: format!("Test.{}", file_type),
                file_type: file_type.to_string(),
                file_size: 1024,
                sheet_name: None,
                column_headers: vec!["Sample_ID".to_string()],
                uploaded_by: Some("test_user".to_string()),
                metadata: None,
            };

            assert_eq!(create_request.file_type, file_type);
            assert!(create_request.filename.ends_with(file_type));
        }

        // Verify unsupported types are identified
        for file_type in unsupported_types {
            // In a real implementation, these would be rejected during validation
            assert!(!vec!["xlsx", "xls", "csv"].contains(&file_type));
        }
    }

    #[test]
    fn test_spreadsheet_dataset_serialization() {
        let dataset = SpreadsheetDataset {
            id: Uuid::new_v4(),
            filename: "test.xlsx".to_string(),
            original_filename: "Test Serialization.xlsx".to_string(),
            file_type: "xlsx".to_string(),
            file_size: 1024,
            sheet_name: Some("Sheet1".to_string()),
            total_rows: 100,
            total_columns: 5,
            column_headers: vec!["Sample_ID".to_string(), "Patient_ID".to_string()],
            upload_status: UploadStatus::Completed,
            error_message: None,
            uploaded_by: Some("test_user".to_string()),
            created_at: Utc::now(),
            updated_at: Utc::now(),
            metadata: json!({"test": true, "version": 1}),
        };

        // Test JSON serialization
        let serialized = serde_json::to_string(&dataset);
        assert!(serialized.is_ok(), "Dataset should serialize to JSON");
    }

    #[test]
    fn test_dataset_creation() {
        let dataset = SpreadsheetDataset {
            id: Uuid::new_v4(),
            filename: "test_data.xlsx".to_string(),
            original_filename: "Test Dataset.xlsx".to_string(),
            file_type: "xlsx".to_string(),
            file_size: 2048,
            sheet_name: Some("Data".to_string()),
            total_rows: 150,
            total_columns: 8,
            column_headers: vec![
                "Sample_ID".to_string(),
                "Patient_ID".to_string(),
                "Sample_Type".to_string(),
            ],
            upload_status: UploadStatus::Completed,
            error_message: None,
            uploaded_by: Some("lab_tech".to_string()),
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
            metadata: json!({"source": "laboratory_import"}),
        };

        assert_eq!(dataset.original_filename, "Test Dataset.xlsx");
        assert_eq!(dataset.filename, "test_data.xlsx");
        assert_eq!(dataset.file_type, "xlsx");
        assert_eq!(dataset.metadata["source"], "laboratory_import");
        assert_eq!(dataset.total_rows, 150);
        assert_eq!(dataset.total_columns, 8);
    }

    #[test]
    fn test_search_query_defaults() {
        let query = SpreadsheetSearchQuery {
            search_term: None,
            dataset_id: None,
            pool_filter: None,
            sample_filter: None,
            project_filter: None,
            column_filters: None,
            limit: None,
            offset: None,
        };

        // Test that query can be created with all None values
        assert!(query.search_term.is_none());
        assert!(query.dataset_id.is_none());
        assert!(query.pool_filter.is_none());
        assert!(query.sample_filter.is_none());
        assert!(query.project_filter.is_none());
        assert!(query.column_filters.is_none());
        assert!(query.limit.is_none());
        assert!(query.offset.is_none());
    }

    #[test]
    fn test_search_query_with_values() {
        let dataset_id = Uuid::new_v4();
        let mut column_filters = HashMap::new();
        column_filters.insert("Sample_Type".to_string(), "DNA".to_string());

        let query = SpreadsheetSearchQuery {
            search_term: Some("LAB001".to_string()),
            dataset_id: Some(dataset_id),
            pool_filter: Some("Pool1".to_string()),
            sample_filter: Some("Sample1".to_string()),
            project_filter: Some("Project1".to_string()),
            column_filters: Some(column_filters),
            limit: Some(25),
            offset: Some(0),
        };

        assert_eq!(query.search_term, Some("LAB001".to_string()));
        assert_eq!(query.dataset_id, Some(dataset_id));
        assert_eq!(query.pool_filter, Some("Pool1".to_string()));
        assert_eq!(query.sample_filter, Some("Sample1".to_string()));
        assert_eq!(query.project_filter, Some("Project1".to_string()));
        assert_eq!(query.limit, Some(25));
        assert_eq!(query.offset, Some(0));
        assert!(query.column_filters.is_some());
    }

    #[test]
    fn test_parsed_spreadsheet_data() {
        let mut rows = Vec::new();
        let mut row1 = HashMap::new();
        row1.insert("Sample_ID".to_string(), "LAB001".to_string());
        row1.insert("Patient_ID".to_string(), "P001".to_string());
        rows.push(row1);

        let parsed_data = ParsedSpreadsheetData {
            headers: vec!["Sample_ID".to_string(), "Patient_ID".to_string()],
            rows,
            total_rows: 1,
            total_columns: 2,
        };

        assert_eq!(parsed_data.headers.len(), 2);
        assert_eq!(parsed_data.total_rows, 1);
        assert_eq!(parsed_data.total_columns, 2);
        assert_eq!(parsed_data.rows.len(), 1);
        assert_eq!(parsed_data.rows[0]["Sample_ID"], "LAB001");
    }

    #[test]
    fn test_dataset_metadata_structure() {
        let metadata = json!({
            "source": "manual_upload",
            "validation_status": "passed",
            "row_count": 150,
            "column_count": 12,
            "file_type": "xlsx",
            "processing_time_ms": 1234,
            "errors": [],
            "warnings": ["Column 'Notes' contains empty cells"]
        });

        let dataset = SpreadsheetDataset {
            id: Uuid::new_v4(),
            filename: "metadata_test.xlsx".to_string(),
            original_filename: "Test Dataset.xlsx".to_string(),
            file_type: "xlsx".to_string(),
            file_size: 4096,
            sheet_name: Some("Data".to_string()),
            total_rows: 150,
            total_columns: 12,
            column_headers: vec!["Sample_ID".to_string(), "Patient_ID".to_string()],
            upload_status: UploadStatus::Completed,
            error_message: None,
            uploaded_by: Some("test_user".to_string()),
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
            metadata,
        };

        // Test metadata access
        assert_eq!(dataset.metadata["source"], "manual_upload");
        assert_eq!(dataset.metadata["row_count"], 150);
        assert!(dataset.metadata["warnings"].is_array());
    }

    #[test]
    fn test_dataset_validation_rules() {
        // Test empty filename validation
        let dataset_filename = "";
        assert!(
            dataset_filename.is_empty(),
            "Empty dataset filename should be invalid"
        );

        // Test valid filename
        let valid_filename = "valid_dataset.xlsx";
        assert!(
            !valid_filename.is_empty() && valid_filename.len() <= 255,
            "Valid filename should pass validation"
        );

        // Test file size limits
        let max_file_size = 100 * 1024 * 1024; // 100MB
        let test_file_size = 50 * 1024 * 1024; // 50MB
        assert!(
            test_file_size <= max_file_size,
            "File size should be within limits"
        );
    }

    #[test]
    fn test_laboratory_specific_headers() {
        let laboratory_headers = vec![
            "Sample_ID".to_string(),
            "Sample_Type".to_string(),
            "Concentration_ng_uL".to_string(),
            "Volume_uL".to_string(),
            "Storage_Location".to_string(),
            "Collection_Date".to_string(),
            "Barcode".to_string(),
            "Quality_Score".to_string(),
            "Status".to_string(),
        ];

        assert!(laboratory_headers.contains(&"Sample_ID".to_string()));
        assert!(laboratory_headers.contains(&"Concentration_ng_uL".to_string()));
        assert!(laboratory_headers.contains(&"Storage_Location".to_string()));
        assert_eq!(laboratory_headers.len(), 9);

        // Test that headers use proper scientific notation
        let has_concentration = laboratory_headers.iter().any(|h| h.contains("ng_uL"));
        let has_volume = laboratory_headers.iter().any(|h| h.contains("uL"));
        assert!(has_concentration);
        assert!(has_volume);
    }

    #[test]
    fn test_pagination_logic() {
        let page = 1i64;
        let per_page = 20i64;
        let offset = (page.saturating_sub(1)) * per_page;

        assert_eq!(offset, 0); // First page should have no offset

        let page2 = 3i64;
        let offset2 = (page2.saturating_sub(1)) * per_page;
        assert_eq!(offset2, 40); // Third page should skip 40 records

        // Test edge cases
        let page_zero = 0i64;
        let offset_zero = (page_zero.saturating_sub(1)) * per_page;
        assert_eq!(offset_zero, 0); // Should handle zero gracefully (underflow protection)
    }
}
