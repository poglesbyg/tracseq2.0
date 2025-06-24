use crate::test_utils::*;
use axum::{extract::State, Json};
use enhanced_storage_service::handlers::health::*;

#[tokio::test]
async fn test_health_check_success() {
    let test_db = TestDatabase::new().await.unwrap();
    let app_state = TestAppStateBuilder::new()
        .with_database(test_db.pool.clone())
        .build()
        .await
        .unwrap();

    let result = health_check(State(app_state)).await;

    assert!(result.is_ok(), "Health check should succeed");
    let response = result.unwrap();
    let health_response = response.0;

    assert_eq!(health_response.status, "healthy");
    assert_eq!(health_response.service, "Enhanced Storage Service");
    TestAssertions::assert_timestamp_recent(health_response.timestamp, 5);

    test_db.cleanup().await.unwrap();
}

#[tokio::test]
async fn test_readiness_check_success() {
    let test_db = TestDatabase::new().await.unwrap();
    let app_state = TestAppStateBuilder::new()
        .with_database(test_db.pool.clone())
        .build()
        .await
        .unwrap();

    let result = readiness_check(State(app_state)).await;

    assert!(result.is_ok(), "Readiness check should succeed");
    let response = result.unwrap();
    let readiness_response = response.0;

    assert_eq!(readiness_response.status, "ready");
    assert!(readiness_response.database_connected);
    assert!(readiness_response.services_available);

    test_db.cleanup().await.unwrap();
}

#[tokio::test]
async fn test_metrics_endpoint() {
    let test_db = TestDatabase::new().await.unwrap();
    let app_state = TestAppStateBuilder::new()
        .with_database(test_db.pool.clone())
        .build()
        .await
        .unwrap();

    let result = metrics(State(app_state)).await;

    assert!(result.is_ok(), "Metrics endpoint should succeed");
    let response = result.unwrap();
    let metrics_response = response.0;

    // Verify basic metrics structure
    assert!(metrics_response.database_connections >= 0);
    assert!(metrics_response.total_requests >= 0);
    assert!(metrics_response.uptime_seconds >= 0);
    assert!(!metrics_response.version.is_empty());

    test_db.cleanup().await.unwrap();
} 
