#[cfg(test)]
use axum::{extract::State, Json};
use serde_json::json;
use sqlx::PgPool;
use uuid::Uuid;

use crate::config::database::Database;
use crate::handlers::reports::{
    execute_report, get_report_templates, get_schema, ReportQuery, ReportResult, ReportTemplate,
};
use crate::assembly::AppComponents;

/// Test helper to create app components with test database
async fn create_test_app_components() -> AppComponents {
    let database_url = std::env::var("TEST_DATABASE_URL")
        .unwrap_or_else(|_| "postgres://postgres:password@localhost/lab_manager_test".to_string());

    let database = Database::new(&database_url)
        .await
        .expect("Failed to connect to test database");

    AppComponents {
        database,
        rag_service_url: "http://localhost:8000".to_string(),
    }
}

/// Setup test data for reports tests
async fn setup_test_data(pool: &PgPool) -> Result<(), sqlx::Error> {
    // Clean up existing test data
    cleanup_test_data(pool).await?;

    // Create test templates
    for i in 1..=3 {
        sqlx::query(
            "INSERT INTO templates (id, name, description, fields, created_at, updated_at) 
             VALUES ($1, $2, $3, $4, NOW(), NOW())",
        )
        .bind(Uuid::new_v4())
        .bind(format!("test_report_template_{}", i))
        .bind(format!("Test report template {}", i))
        .bind(json!([{"name": "field1", "type": "text"}]))
        .execute(pool)
        .await?;
    }

    // Create test samples with different statuses
    let statuses = ["pending", "validated", "in_storage"];
    for (i, status) in statuses.iter().enumerate() {
        sqlx::query(
            "INSERT INTO samples (id, name, barcode, location, status, metadata, created_at, updated_at)
             VALUES ($1, $2, $3, $4, $5, $6, NOW() - INTERVAL '%d days', NOW())"
        )
        .bind(Uuid::new_v4())
        .bind(format!("test_report_sample_{}", i))
        .bind(format!("TEST-REPORT-{:03}", i))
        .bind(format!("Test Location {}", i))
        .bind(*status)
        .bind(json!({"template_name": "test_report_template_1"}))
        .execute(pool)
        .await?;
    }

    Ok(())
}

/// Cleanup test data
async fn cleanup_test_data(pool: &PgPool) -> Result<(), sqlx::Error> {
    sqlx::query("DELETE FROM samples WHERE name LIKE 'test_report_%'")
        .execute(pool)
        .await?;
    sqlx::query("DELETE FROM templates WHERE name LIKE 'test_report_%'")
        .execute(pool)
        .await?;
    Ok(())
}

#[tokio::test]
async fn test_get_schema_returns_valid_structure() {
    let app_components = create_test_app_components().await;

    let result = get_schema(State(app_components)).await;

    assert!(result.is_ok());
    let Json(schema) = result.unwrap();

    // Should have at least the core tables
    assert!(!schema.tables.is_empty());

    // Look for essential tables
    let table_names: Vec<&String> = schema.tables.iter().map(|t| &t.name).collect();
    assert!(
        table_names.contains(&&"templates".to_string()),
        "Should include templates table"
    );
    assert!(
        table_names.contains(&&"samples".to_string()),
        "Should include samples table"
    );

    // Check that each table has columns
    for table in &schema.tables {
        assert!(
            !table.columns.is_empty(),
            "Table {} should have columns",
            table.name
        );

        // Check column structure
        for column in &table.columns {
            assert!(!column.name.is_empty(), "Column name should not be empty");
            assert!(
                !column.data_type.is_empty(),
                "Column data type should not be empty"
            );
        }
    }
}

#[tokio::test]
async fn test_get_report_templates_returns_expected_templates() {
    let app_components = create_test_app_components().await;

    let result = get_report_templates(State(app_components)).await;

    assert!(result.is_ok());
    let Json(templates) = result.unwrap();

    assert!(!templates.is_empty(), "Should return predefined templates");

    // Check for expected template IDs
    let template_ids: Vec<&String> = templates.iter().map(|t| &t.id).collect();
    assert!(template_ids.contains(&&"samples_by_status".to_string()));
    assert!(template_ids.contains(&&"recent_samples".to_string()));
    assert!(template_ids.contains(&&"templates_usage".to_string()));
    assert!(template_ids.contains(&&"sample_locations".to_string()));

    // Verify template structure
    for template in &templates {
        assert!(!template.id.is_empty());
        assert!(!template.name.is_empty());
        assert!(!template.description.is_empty());
        assert!(!template.sql.is_empty());
        assert!(!template.category.is_empty());
        assert!(template.sql.to_lowercase().starts_with("select"));
    }
}

#[tokio::test]
async fn test_execute_report_valid_select_query() {
    let app_components = create_test_app_components().await;

    setup_test_data(&app_components.database.pool)
        .await
        .expect("Failed to setup test data");

    let query = ReportQuery {
        sql: "SELECT COUNT(*) as total FROM templates WHERE name LIKE 'test_report_%'".to_string(),
        export_format: None,
    };

    let result = execute_report(State(app_components.clone()), Json(query)).await;

    assert!(result.is_ok());
    let Json(report_result) = result.unwrap();

    assert_eq!(report_result.columns, vec!["total"]);
    assert_eq!(report_result.rows.len(), 1);
    assert!(report_result.execution_time_ms > 0);
    assert!(!report_result.query.is_empty());

    // Check that the count is correct
    if let Some(row) = report_result.rows.first() {
        if let Some(total) = row.get("total") {
            assert!(total.as_i64().unwrap_or(0) >= 3);
        }
    }

    cleanup_test_data(&app_components.database.pool)
        .await
        .expect("Failed to cleanup test data");
}

#[tokio::test]
async fn test_execute_report_complex_query() {
    let app_components = create_test_app_components().await;

    setup_test_data(&app_components.database.pool)
        .await
        .expect("Failed to setup test data");

    let query = ReportQuery {
        sql: "SELECT status, COUNT(*) as count FROM samples WHERE name LIKE 'test_report_%' GROUP BY status ORDER BY count DESC".to_string(),
        export_format: None,
    };

    let result = execute_report(State(app_components.clone()), Json(query)).await;

    assert!(result.is_ok());
    let Json(report_result) = result.unwrap();

    assert_eq!(report_result.columns, vec!["status", "count"]);
    assert!(report_result.rows.len() > 0);
    assert!(report_result.execution_time_ms > 0);

    // Verify data structure
    for row in &report_result.rows {
        assert!(row.contains_key("status"));
        assert!(row.contains_key("count"));
        assert!(row.get("count").unwrap().as_i64().unwrap_or(0) > 0);
    }

    cleanup_test_data(&app_components.database.pool)
        .await
        .expect("Failed to cleanup test data");
}

#[tokio::test]
async fn test_execute_report_empty_result() {
    let app_components = create_test_app_components().await;

    let query = ReportQuery {
        sql: "SELECT * FROM samples WHERE name = 'nonexistent_sample'".to_string(),
        export_format: None,
    };

    let result = execute_report(State(app_components), Json(query)).await;

    assert!(result.is_ok());
    let Json(report_result) = result.unwrap();

    assert_eq!(report_result.rows.len(), 0);
    assert_eq!(report_result.row_count, 0);
    assert!(report_result.execution_time_ms >= 0);
    assert_eq!(report_result.columns.len(), 0);
}

#[tokio::test]
async fn test_execute_report_security_sql_injection_protection() {
    let app_components = create_test_app_components().await;

    // Test various SQL injection attempts
    let malicious_queries = vec![
        "SELECT * FROM samples; DROP TABLE samples; --",
        "INSERT INTO samples (name) VALUES ('malicious')",
        "UPDATE samples SET name = 'hacked'",
        "DELETE FROM samples",
        "CREATE TABLE malicious (id INT)",
        "ALTER TABLE samples ADD COLUMN malicious TEXT",
        "GRANT ALL ON samples TO PUBLIC",
        "EXEC sp_executesql 'malicious code'",
        "SELECT * FROM samples WHERE 1=1; EXEC xp_cmdshell('dir')",
    ];

    for malicious_sql in malicious_queries {
        let query = ReportQuery {
            sql: malicious_sql.to_string(),
            export_format: None,
        };

        let result = execute_report(State(app_components.clone()), Json(query)).await;

        assert!(
            result.is_err(),
            "Malicious query should be rejected: {}",
            malicious_sql
        );

        if let Err((status, message)) = result {
            assert_eq!(status, axum::http::StatusCode::BAD_REQUEST);
            assert!(message.contains("Only SELECT queries are allowed"));
        }
    }
}

#[tokio::test]
async fn test_execute_report_comment_injection_protection() {
    let app_components = create_test_app_components().await;

    let queries_with_comments = vec![
        "SELECT * FROM samples -- comment",
        "SELECT * FROM samples /* comment */",
        "SELECT * FROM samples WHERE /* malicious */ 1=1",
    ];

    for query_with_comment in queries_with_comments {
        let query = ReportQuery {
            sql: query_with_comment.to_string(),
            export_format: None,
        };

        let result = execute_report(State(app_components.clone()), Json(query)).await;

        assert!(
            result.is_err(),
            "Query with comments should be rejected: {}",
            query_with_comment
        );
    }
}

#[tokio::test]
async fn test_execute_report_multiple_statement_protection() {
    let app_components = create_test_app_components().await;

    let query = ReportQuery {
        sql: "SELECT * FROM samples; SELECT * FROM templates;".to_string(),
        export_format: None,
    };

    let result = execute_report(State(app_components), Json(query)).await;

    assert!(result.is_err(), "Multiple statements should be rejected");
}

#[tokio::test]
async fn test_execute_report_invalid_sql_syntax() {
    let app_components = create_test_app_components().await;

    let query = ReportQuery {
        sql: "INVALID SQL SYNTAX".to_string(),
        export_format: None,
    };

    let result = execute_report(State(app_components), Json(query)).await;

    assert!(result.is_err());
    if let Err((status, message)) = result {
        assert_eq!(status, axum::http::StatusCode::BAD_REQUEST);
        assert!(message.contains("SQL Error"));
    }
}

#[tokio::test]
async fn test_execute_report_with_different_data_types() {
    let app_components = create_test_app_components().await;

    // Create a sample with various data types
    let sample_id = Uuid::new_v4();
    sqlx::query(
        "INSERT INTO samples (id, name, barcode, location, status, metadata, created_at, updated_at)
         VALUES ($1, $2, $3, $4, $5, $6, $7, $8)"
    )
    .bind(sample_id)
    .bind("test_datatypes_sample")
    .bind("TEST-DATATYPES-001")
    .bind("Test Location")
    .bind("validated")
    .bind(json!({"key": "value", "number": 42, "boolean": true}))
    .bind(chrono::Utc::now())
    .bind(chrono::Utc::now())
    .execute(&app_components.database.pool)
    .await
    .expect("Failed to create test sample");

    let query = ReportQuery {
        sql: "SELECT id, name, metadata, created_at FROM samples WHERE name = 'test_datatypes_sample'".to_string(),
        export_format: None,
    };

    let result = execute_report(State(app_components.clone()), Json(query)).await;

    assert!(result.is_ok());
    let Json(report_result) = result.unwrap();

    assert_eq!(report_result.rows.len(), 1);
    let row = &report_result.rows[0];

    // Check UUID conversion
    assert!(row.contains_key("id"));
    if let Some(id_value) = row.get("id") {
        assert!(id_value.is_string());
        assert!(Uuid::parse_str(id_value.as_str().unwrap()).is_ok());
    }

    // Check JSON conversion
    assert!(row.contains_key("metadata"));
    if let Some(metadata_value) = row.get("metadata") {
        assert!(metadata_value.is_object());
    }

    // Check timestamp conversion
    assert!(row.contains_key("created_at"));
    if let Some(timestamp_value) = row.get("created_at") {
        assert!(timestamp_value.is_string());
        assert!(chrono::DateTime::parse_from_rfc3339(timestamp_value.as_str().unwrap()).is_ok());
    }

    // Cleanup
    sqlx::query("DELETE FROM samples WHERE name = 'test_datatypes_sample'")
        .execute(&app_components.database.pool)
        .await
        .expect("Failed to cleanup test sample");
}

#[tokio::test]
async fn test_report_template_serialization() {
    let template = ReportTemplate {
        id: "test_template".to_string(),
        name: "Test Template".to_string(),
        description: "A test template".to_string(),
        sql: "SELECT * FROM samples".to_string(),
        category: "Test".to_string(),
    };

    let serialized = serde_json::to_string(&template);
    assert!(serialized.is_ok());

    let json_str = serialized.unwrap();
    assert!(json_str.contains("test_template"));
    assert!(json_str.contains("Test Template"));
    assert!(json_str.contains("A test template"));
    assert!(json_str.contains("SELECT * FROM samples"));
    assert!(json_str.contains("Test"));

    // Test deserialization
    let deserialized: Result<ReportTemplate, _> = serde_json::from_str(&json_str);
    assert!(deserialized.is_ok());

    let parsed_template = deserialized.unwrap();
    assert_eq!(parsed_template.id, "test_template");
    assert_eq!(parsed_template.name, "Test Template");
}

#[tokio::test]
async fn test_report_result_serialization() {
    let mut test_row = std::collections::HashMap::new();
    test_row.insert("id".to_string(), json!(1));
    test_row.insert("name".to_string(), json!("test"));

    let report_result = ReportResult {
        columns: vec!["id".to_string(), "name".to_string()],
        rows: vec![test_row],
        row_count: 1,
        execution_time_ms: 150,
        query: "SELECT id, name FROM test".to_string(),
    };

    let serialized = serde_json::to_string(&report_result);
    assert!(serialized.is_ok());

    let json_str = serialized.unwrap();
    assert!(json_str.contains("columns"));
    assert!(json_str.contains("rows"));
    assert!(json_str.contains("row_count"));
    assert!(json_str.contains("execution_time_ms"));
    assert!(json_str.contains("query"));

    // Test deserialization
    let deserialized: Result<ReportResult, _> = serde_json::from_str(&json_str);
    assert!(deserialized.is_ok());

    let parsed_result = deserialized.unwrap();
    assert_eq!(parsed_result.columns.len(), 2);
    assert_eq!(parsed_result.rows.len(), 1);
    assert_eq!(parsed_result.row_count, 1);
    assert_eq!(parsed_result.execution_time_ms, 150);
}

#[tokio::test]
async fn test_execute_report_performance_tracking() {
    let app_components = create_test_app_components().await;

    // Execute a simple query multiple times to check performance tracking
    let query = ReportQuery {
        sql: "SELECT 1 as test_value".to_string(),
        export_format: None,
    };

    let mut execution_times = Vec::new();

    for _ in 0..5 {
        let result = execute_report(State(app_components.clone()), Json(query.clone())).await;
        assert!(result.is_ok());

        let Json(report_result) = result.unwrap();
        assert!(report_result.execution_time_ms >= 0);
        execution_times.push(report_result.execution_time_ms);
    }

    // All executions should have recorded some execution time
    assert!(execution_times.iter().all(|&time| time >= 0));

    // Performance should be relatively consistent for simple queries
    let max_time = execution_times.iter().max().unwrap();
    let min_time = execution_times.iter().min().unwrap();

    // The difference shouldn't be more than 1000ms for a simple query
    assert!(
        max_time - min_time < 1000,
        "Performance should be consistent for simple queries"
    );
}

/// Integration test for the full reports workflow
#[tokio::test]
async fn test_reports_integration_workflow() {
    let app_components = create_test_app_components().await;

    setup_test_data(&app_components.database.pool)
        .await
        .expect("Failed to setup test data");

    // 1. Get schema to understand database structure
    let schema_result = get_schema(State(app_components.clone())).await;
    assert!(schema_result.is_ok());
    let Json(schema) = schema_result.unwrap();
    assert!(!schema.tables.is_empty());

    // 2. Get available report templates
    let templates_result = get_report_templates(State(app_components.clone())).await;
    assert!(templates_result.is_ok());
    let Json(templates) = templates_result.unwrap();
    assert!(!templates.is_empty());

    // 3. Execute one of the predefined templates
    let samples_by_status_template = templates
        .iter()
        .find(|t| t.id == "samples_by_status")
        .expect("Should find samples_by_status template");

    let query = ReportQuery {
        sql: samples_by_status_template.sql.clone(),
        export_format: None,
    };

    let report_result = execute_report(State(app_components.clone()), Json(query)).await;
    assert!(report_result.is_ok());

    let Json(report) = report_result.unwrap();
    assert!(report.execution_time_ms > 0);
    assert!(!report.columns.is_empty());

    cleanup_test_data(&app_components.database.pool)
        .await
        .expect("Failed to cleanup test data");
}
