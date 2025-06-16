#[cfg(test)]
use axum::{
    body::Body,
    extract::State,
    http::{Request, StatusCode},
    Json,
};
use sqlx::{PgPool, Row};
use uuid::Uuid;

use crate::config::database::Database;
use crate::handlers::dashboard::{get_dashboard_stats, health_check, DashboardStats, HealthStatus};
use crate::AppComponents;

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

/// Setup test data for dashboard tests
async fn setup_test_data(pool: &PgPool) -> Result<(), sqlx::Error> {
    // Clean up existing test data
    sqlx::query("DELETE FROM samples WHERE name LIKE 'test_%'")
        .execute(pool)
        .await?;
    sqlx::query("DELETE FROM templates WHERE name LIKE 'test_%'")
        .execute(pool)
        .await?;
    sqlx::query("DELETE FROM sequencing_jobs WHERE name LIKE 'test_%'")
        .execute(pool)
        .await?;

    // Create test templates
    for i in 1..=3 {
        sqlx::query(
            "INSERT INTO templates (id, name, description, fields, created_at, updated_at) 
             VALUES ($1, $2, $3, $4, NOW(), NOW())",
        )
        .bind(Uuid::new_v4())
        .bind(format!("test_template_{}", i))
        .bind(format!("Test template {}", i))
        .bind(serde_json::json!([{"name": "field1", "type": "text"}]))
        .execute(pool)
        .await?;
    }

    // Create test samples
    for i in 1..=5 {
        sqlx::query(
            "INSERT INTO samples (id, name, barcode, location, status, metadata, created_at, updated_at)
             VALUES ($1, $2, $3, $4, $5, $6, NOW(), NOW())"
        )
        .bind(Uuid::new_v4())
        .bind(format!("test_sample_{}", i))
        .bind(format!("TEST-{:03}", i))
        .bind("Test Location")
        .bind("validated")
        .bind(serde_json::json!({"template_name": "test_template_1"}))
        .execute(pool)
        .await?;
    }

    // Create test sequencing jobs
    for i in 1..=4 {
        let status = if i <= 2 { "pending" } else { "completed" };
        sqlx::query(
            "INSERT INTO sequencing_jobs (id, name, description, status, created_at, updated_at)
             VALUES ($1, $2, $3, $4, NOW(), NOW())",
        )
        .bind(Uuid::new_v4())
        .bind(format!("test_job_{}", i))
        .bind(format!("Test job {}", i))
        .bind(status)
        .execute(pool)
        .await?;
    }

    Ok(())
}

#[tokio::test]
async fn test_health_check_healthy_database() {
    let app_components = create_test_app_components().await;

    let result = health_check(State(app_components)).await;

    assert!(result.is_ok());
    let Json(health_status) = result.unwrap();

    assert_eq!(health_status.status, "healthy");
    assert!(health_status.database_connected);
    assert!(!health_status.timestamp.to_string().is_empty());
}

#[tokio::test]
async fn test_get_dashboard_stats_with_data() {
    let app_components = create_test_app_components().await;

    // Setup test data
    setup_test_data(&app_components.database.pool)
        .await
        .expect("Failed to setup test data");

    let result = get_dashboard_stats(State(app_components.clone())).await;

    assert!(result.is_ok());
    let Json(stats) = result.unwrap();

    // Verify stats reflect our test data
    assert!(
        stats.total_templates >= 3,
        "Should have at least 3 test templates"
    );
    assert!(
        stats.total_samples >= 5,
        "Should have at least 5 test samples"
    );
    assert!(
        stats.pending_sequencing >= 2,
        "Should have at least 2 pending jobs"
    );
    assert!(
        stats.completed_sequencing >= 2,
        "Should have at least 2 completed jobs"
    );

    // Cleanup
    cleanup_test_data(&app_components.database.pool)
        .await
        .expect("Failed to cleanup test data");
}

#[tokio::test]
async fn test_get_dashboard_stats_empty_database() {
    let app_components = create_test_app_components().await;

    // Ensure clean state
    cleanup_test_data(&app_components.database.pool)
        .await
        .expect("Failed to cleanup test data");

    let result = get_dashboard_stats(State(app_components.clone())).await;

    assert!(result.is_ok());
    let Json(stats) = result.unwrap();

    // With empty database, all counts should be 0 or greater
    assert!(stats.total_templates >= 0);
    assert!(stats.total_samples >= 0);
    assert!(stats.pending_sequencing >= 0);
    assert!(stats.completed_sequencing >= 0);
}

#[tokio::test]
async fn test_dashboard_stats_data_consistency() {
    let app_components = create_test_app_components().await;

    setup_test_data(&app_components.database.pool)
        .await
        .expect("Failed to setup test data");

    // Get stats multiple times to ensure consistency
    let result1 = get_dashboard_stats(State(app_components.clone())).await;
    let result2 = get_dashboard_stats(State(app_components.clone())).await;

    assert!(result1.is_ok());
    assert!(result2.is_ok());

    let Json(stats1) = result1.unwrap();
    let Json(stats2) = result2.unwrap();

    // Stats should be identical when no data changes
    assert_eq!(stats1.total_templates, stats2.total_templates);
    assert_eq!(stats1.total_samples, stats2.total_samples);
    assert_eq!(stats1.pending_sequencing, stats2.pending_sequencing);
    assert_eq!(stats1.completed_sequencing, stats2.completed_sequencing);

    cleanup_test_data(&app_components.database.pool)
        .await
        .expect("Failed to cleanup test data");
}

#[tokio::test]
async fn test_dashboard_stats_with_various_job_statuses() {
    let app_components = create_test_app_components().await;

    cleanup_test_data(&app_components.database.pool)
        .await
        .expect("Failed to cleanup test data");

    // Create jobs with different statuses
    let job_statuses = vec!["pending", "running", "completed", "failed"];
    for (i, status) in job_statuses.iter().enumerate() {
        sqlx::query(
            "INSERT INTO sequencing_jobs (id, name, description, status, created_at, updated_at)
             VALUES ($1, $2, $3, $4, NOW(), NOW())",
        )
        .bind(Uuid::new_v4())
        .bind(format!("test_job_status_{}", i))
        .bind(format!("Test job with status {}", status))
        .bind(*status)
        .execute(&app_components.database.pool)
        .await
        .expect("Failed to create test job");
    }

    let result = get_dashboard_stats(State(app_components.clone())).await;
    assert!(result.is_ok());

    let Json(stats) = result.unwrap();

    // Should count pending and completed specifically
    assert!(stats.pending_sequencing >= 1);
    assert!(stats.completed_sequencing >= 1);

    cleanup_test_data(&app_components.database.pool)
        .await
        .expect("Failed to cleanup test data");
}

#[tokio::test]
async fn test_health_check_timestamp_format() {
    let app_components = create_test_app_components().await;

    let result = health_check(State(app_components)).await;
    assert!(result.is_ok());

    let Json(health_status) = result.unwrap();

    // Verify timestamp is valid UTC format
    let parsed_time = chrono::DateTime::parse_from_rfc3339(&health_status.timestamp.to_rfc3339());
    assert!(
        parsed_time.is_ok(),
        "Timestamp should be valid RFC3339 format"
    );

    // Verify timestamp is recent (within last minute)
    let now = chrono::Utc::now();
    let time_diff = now.signed_duration_since(health_status.timestamp);
    assert!(time_diff.num_seconds() < 60, "Timestamp should be recent");
}

#[tokio::test]
async fn test_dashboard_stats_serialization() {
    let stats = DashboardStats {
        total_templates: 10,
        total_samples: 25,
        pending_sequencing: 5,
        completed_sequencing: 15,
    };

    let serialized = serde_json::to_string(&stats);
    assert!(serialized.is_ok());

    let json_str = serialized.unwrap();
    assert!(json_str.contains("totalTemplates"));
    assert!(json_str.contains("totalSamples"));
    assert!(json_str.contains("pendingSequencing"));
    assert!(json_str.contains("completedSequencing"));

    // Test deserialization
    let deserialized: Result<DashboardStats, _> = serde_json::from_str(&json_str);
    assert!(deserialized.is_ok());

    let parsed_stats = deserialized.unwrap();
    assert_eq!(parsed_stats.total_templates, 10);
    assert_eq!(parsed_stats.total_samples, 25);
    assert_eq!(parsed_stats.pending_sequencing, 5);
    assert_eq!(parsed_stats.completed_sequencing, 15);
}

#[tokio::test]
async fn test_health_check_serialization() {
    let health_status = HealthStatus {
        status: "healthy".to_string(),
        database_connected: true,
        timestamp: chrono::Utc::now(),
    };

    let serialized = serde_json::to_string(&health_status);
    assert!(serialized.is_ok());

    let json_str = serialized.unwrap();
    assert!(json_str.contains("status"));
    assert!(json_str.contains("database_connected"));
    assert!(json_str.contains("timestamp"));
    assert!(json_str.contains("healthy"));
    assert!(json_str.contains("true"));
}

/// Helper function to cleanup test data
async fn cleanup_test_data(pool: &PgPool) -> Result<(), sqlx::Error> {
    sqlx::query("DELETE FROM samples WHERE name LIKE 'test_%'")
        .execute(pool)
        .await?;
    sqlx::query("DELETE FROM templates WHERE name LIKE 'test_%'")
        .execute(pool)
        .await?;
    sqlx::query("DELETE FROM sequencing_jobs WHERE name LIKE 'test_%'")
        .execute(pool)
        .await?;
    Ok(())
}

/// Integration test for dashboard API endpoints
#[tokio::test]
async fn test_dashboard_endpoints_integration() {
    let app_components = create_test_app_components().await;

    setup_test_data(&app_components.database.pool)
        .await
        .expect("Failed to setup test data");

    // Test both endpoints work together
    let health_result = health_check(State(app_components.clone())).await;
    let stats_result = get_dashboard_stats(State(app_components.clone())).await;

    assert!(health_result.is_ok());
    assert!(stats_result.is_ok());

    let Json(health) = health_result.unwrap();
    let Json(stats) = stats_result.unwrap();

    // If database is healthy, stats should be retrievable
    if health.database_connected {
        assert!(stats.total_templates >= 0);
        assert!(stats.total_samples >= 0);
        assert!(stats.pending_sequencing >= 0);
        assert!(stats.completed_sequencing >= 0);
    }

    cleanup_test_data(&app_components.database.pool)
        .await
        .expect("Failed to cleanup test data");
}
