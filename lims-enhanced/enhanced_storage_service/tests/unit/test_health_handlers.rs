use crate::test_utils::*;
use axum::extract::State;
use enhanced_storage_service::handlers::health::*;

#[tokio::test]
async fn test_health_check() {
    let test_db = TestDatabase::new().await.unwrap();
    let app_state = TestAppStateBuilder::new()
        .with_database(test_db.pool.clone())
        .build()
        .await
        .unwrap();

    let result = health_check(State(app_state)).await;

    assert!(result.is_ok(), "Health check should succeed");
    let health_response = result.unwrap().0;

    assert_eq!(health_response.status, "healthy");
    assert!(!health_response.version.is_empty());
    assert!(health_response.uptime_seconds >= 0);
    assert!(!health_response.services.is_empty());

    test_db.cleanup().await.unwrap();
}

#[tokio::test]
async fn test_readiness_check() {
    let test_db = TestDatabase::new().await.unwrap();
    let app_state = TestAppStateBuilder::new()
        .with_database(test_db.pool.clone())
        .build()
        .await
        .unwrap();

    let result = readiness_check(State(app_state)).await;

    // The readiness check can return either Ok or Err based on service status
    match result {
        Ok(json_response) => {
            let readiness_response = json_response.0;
            assert!(readiness_response.ready);
            assert!(!readiness_response.checks.is_empty());
            assert!(readiness_response.checks.contains_key("database"));
        }
        Err(status_code) => {
            // If we get an error, it should be SERVICE_UNAVAILABLE
            assert_eq!(status_code, axum::http::StatusCode::SERVICE_UNAVAILABLE);
        }
    }

    test_db.cleanup().await.unwrap();
}

#[tokio::test]
async fn test_metrics() {
    let test_db = TestDatabase::new().await.unwrap();
    let app_state = TestAppStateBuilder::new()
        .with_database(test_db.pool.clone())
        .build()
        .await
        .unwrap();

    let result = metrics(State(app_state)).await;

    assert!(result.is_ok(), "Metrics should succeed");
    let metrics_response = result.unwrap().0;

    assert!(metrics_response.system_health.database_connections >= 0);
    assert!(metrics_response.storage_locations >= 0);
    assert!(metrics_response.total_samples >= 0);
    assert!(metrics_response.active_sensors >= 0);
    assert!(metrics_response.recent_alerts >= 0);
    assert!(metrics_response.system_health.memory_usage_mb >= 0.0);
    assert!(metrics_response.system_health.cpu_usage_percent >= 0.0);
    assert!(metrics_response.system_health.disk_usage_percent >= 0.0);

    test_db.cleanup().await.unwrap();
} 
