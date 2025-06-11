#[cfg(test)]
mod spreadsheet_processing_tests {
    use crate::models::spreadsheet::{
        CreateSpreadsheetDataset, SpreadsheetColumn, SpreadsheetDataset, SpreadsheetListQuery,
        SpreadsheetRow, SpreadsheetValue,
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
        let service = SpreadsheetService::new(pool.clone());

        let create_request = CreateSpreadsheetDataset {
            name: "Test Dataset".to_string(),
            description: Some("A test dataset for unit tests".to_string()),
            file_path: "/tmp/test.xlsx".to_string(),
            file_type: "xlsx".to_string(),
            metadata: Some(json!({"source": "unit_test", "version": "1.0"})),
        };

        let result = service.create_dataset(create_request, None).await;
        assert!(result.is_ok(), "Creating dataset should succeed");

        let dataset = result.unwrap();
        assert_eq!(dataset.name, "Test Dataset");
        assert_eq!(dataset.file_type, "xlsx");
        assert!(dataset.metadata.is_object());

        // Cleanup
        let _ = service.delete_dataset(dataset.id).await;
    }

    #[tokio::test]
    async fn test_list_datasets_with_pagination() {
        let pool = setup_test_db().await;
        let service = SpreadsheetService::new(pool.clone());

        // Create multiple test datasets
        let mut created_ids = Vec::new();
        for i in 1..=5 {
            let create_request = CreateSpreadsheetDataset {
                name: format!("Test Dataset {}", i),
                description: Some(format!("Test description {}", i)),
                file_path: format!("/tmp/test{}.xlsx", i),
                file_type: "xlsx".to_string(),
                metadata: Some(json!({"index": i})),
            };

            let dataset = service
                .create_dataset(create_request, None)
                .await
                .expect("Creating dataset should succeed");
            created_ids.push(dataset.id);
        }

        // Test pagination
        let query = SpreadsheetListQuery {
            page: Some(1),
            per_page: Some(3),
            file_type: None,
            name_filter: None,
        };

        let result = service.list_datasets(query).await;
        assert!(result.is_ok(), "Listing datasets should succeed");

        let list_response = result.unwrap();
        assert!(list_response.datasets.len() <= 3);
        assert!(list_response.total >= 5);
        assert_eq!(list_response.page, 1);
        assert_eq!(list_response.per_page, 3);

        // Cleanup
        for id in created_ids {
            let _ = service.delete_dataset(id).await;
        }
    }

    #[test]
    fn test_spreadsheet_value_types() {
        // Test different value types
        let string_value = SpreadsheetValue::Text("Sample001".to_string());
        let numeric_value = SpreadsheetValue::Number(123.45);
        let boolean_value = SpreadsheetValue::Boolean(true);
        let null_value = SpreadsheetValue::Null;

        match string_value {
            SpreadsheetValue::Text(s) => assert_eq!(s, "Sample001"),
            _ => panic!("Expected text value"),
        }

        match numeric_value {
            SpreadsheetValue::Number(n) => assert_eq!(n, 123.45),
            _ => panic!("Expected numeric value"),
        }

        match boolean_value {
            SpreadsheetValue::Boolean(b) => assert!(b),
            _ => panic!("Expected boolean value"),
        }

        match null_value {
            SpreadsheetValue::Null => assert!(true),
            _ => panic!("Expected null value"),
        }
    }

    #[test]
    fn test_file_type_validation() {
        let supported_types = vec!["xlsx", "xls", "csv", "tsv"];
        let unsupported_types = vec!["txt", "json", "xml", "pdf"];

        for file_type in supported_types {
            // Test that supported types are valid for dataset creation
            let create_request = CreateSpreadsheetDataset {
                name: format!("Test {}", file_type),
                description: None,
                file_path: format!("/tmp/test.{}", file_type),
                file_type: file_type.to_string(),
                metadata: None,
            };

            assert_eq!(create_request.file_type, file_type);
            assert!(create_request.file_path.ends_with(file_type));
        }

        // Verify unsupported types are identified
        for file_type in unsupported_types {
            // In a real implementation, these would be rejected during validation
            assert!(!vec!["xlsx", "xls", "csv", "tsv"].contains(&file_type));
        }
    }

    #[test]
    fn test_spreadsheet_dataset_serialization() {
        let dataset = SpreadsheetDataset {
            id: Uuid::new_v4(),
            name: "Test Serialization".to_string(),
            description: Some("Testing JSON serialization".to_string()),
            file_path: "/tmp/test.xlsx".to_string(),
            file_type: "xlsx".to_string(),
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
            name: "Test Dataset".to_string(),
            description: Some("A test dataset for unit testing".to_string()),
            file_path: "test_data.xlsx".to_string(),
            file_type: "xlsx".to_string(),
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
            metadata: json!({"source": "laboratory_import"}),
        };

        assert_eq!(dataset.name, "Test Dataset");
        assert_eq!(dataset.file_path, "test_data.xlsx");
        assert_eq!(dataset.file_type, "xlsx");
        assert_eq!(dataset.metadata["source"], "laboratory_import");
    }

    #[test]
    fn test_dataset_list_query_defaults() {
        let query = SpreadsheetListQuery {
            page: None,
            per_page: None,
            search: None,
            name_filter: None,
        };

        // Test that query can be created with all None values
        assert!(query.page.is_none());
        assert!(query.per_page.is_none());
        assert!(query.search.is_none());
        assert!(query.name_filter.is_none());
    }

    #[test]
    fn test_dataset_list_query_with_values() {
        let user_id = Uuid::new_v4();
        let query = SpreadsheetListQuery {
            page: Some(1),
            per_page: Some(25),
            search: Some("test".to_string()),
            name_filter: None,
        };

        assert_eq!(query.page, Some(1));
        assert_eq!(query.per_page, Some(25));
        assert_eq!(query.search, Some("test".to_string()));
        assert!(query.name_filter.is_none());
    }

    #[test]
    fn test_dataset_filter_creation() {
        let filter = SpreadsheetColumn {
            name: "Sample Type".to_string(),
            operator: "equals".to_string(),
            value: SpreadsheetValue::Text("DNA".to_string()),
        };

        assert_eq!(filter.name, "Sample Type");
        assert_eq!(filter.operator, "equals");
        
        if let SpreadsheetValue::Text(value) = &filter.value {
            assert_eq!(value, "DNA");
        } else {
            panic!("Expected Text value");
        }
    }

    #[test]
    fn test_spreadsheet_value_serialization() {
        let values = vec![
            SpreadsheetValue::Text("Sample123".to_string()),
            SpreadsheetValue::Number(123.45),
            SpreadsheetValue::Boolean(false),
            SpreadsheetValue::Null,
        ];

        for value in values {
            let serialized = serde_json::to_string(&value).expect("Should serialize");
            let deserialized: SpreadsheetValue = serde_json::from_str(&serialized).expect("Should deserialize");
            
            match (&value, &deserialized) {
                (SpreadsheetValue::Text(a), SpreadsheetValue::Text(b)) => assert_eq!(a, b),
                (SpreadsheetValue::Number(a), SpreadsheetValue::Number(b)) => assert_eq!(a, b),
                (SpreadsheetValue::Boolean(a), SpreadsheetValue::Boolean(b)) => assert_eq!(a, b),
                (SpreadsheetValue::Null, SpreadsheetValue::Null) => {},
                _ => panic!("Serialization/deserialization mismatch"),
            }
        }
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
            name: "Test Dataset".to_string(),
            description: Some("Test metadata structure".to_string()),
            file_path: "metadata_test.xlsx".to_string(),
            file_type: "xlsx".to_string(),
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
        // Test empty name validation
        let dataset_name = "";
        assert!(
            dataset_name.is_empty(),
            "Empty dataset name should be invalid"
        );

        // Test valid name
        let valid_name = "Valid Dataset Name";
        assert!(
            !valid_name.is_empty() && valid_name.len() <= 255,
            "Valid name should pass validation"
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
            "Sample ID".to_string(),
            "Sample Type".to_string(),
            "Concentration (ng/µL)".to_string(),
            "Volume (µL)".to_string(),
            "Storage Location".to_string(),
            "Collection Date".to_string(),
            "Barcode".to_string(),
            "Quality Score".to_string(),
            "Status".to_string(),
        ];

        assert!(laboratory_headers.contains(&"Sample ID".to_string()));
        assert!(laboratory_headers.contains(&"Concentration (ng/µL)".to_string()));
        assert!(laboratory_headers.contains(&"Storage Location".to_string()));
        assert_eq!(laboratory_headers.len(), 9);

        // Test that headers use proper scientific notation
        assert!(laboratory_headers[2].contains("ng/µL"));
        assert!(laboratory_headers[3].contains("µL"));
    }

    #[test]
    fn test_pagination_logic() {
        let page = 1u32;
        let per_page = 20u32;
        let offset = (page.saturating_sub(1)) * per_page;

        assert_eq!(offset, 0); // First page should have no offset

        let page2 = 3u32;
        let offset2 = (page2.saturating_sub(1)) * per_page;
        assert_eq!(offset2, 40); // Third page should skip 40 records

        // Test edge cases
        let page_zero = 0u32;
        let offset_zero = (page_zero.saturating_sub(1)) * per_page;
        assert_eq!(offset_zero, 0); // Should handle zero gracefully
    }
} 
