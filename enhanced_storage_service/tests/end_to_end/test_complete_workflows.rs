use crate::{test_utils::*, fixtures::*};
use axum::{
    body::{Body, to_bytes},
    http::{Request, StatusCode},
};
use enhanced_storage_service::create_app;
use tower::ServiceExt;
use uuid::Uuid;

#[tokio::test]
async fn test_complete_sample_lifecycle() {
    let test_db = TestDatabase::new().await.unwrap();
    let app_state = TestAppStateBuilder::new()
        .with_database(test_db.pool.clone())
        .build()
        .await
        .unwrap();
    
    let app = create_app(app_state);

    // 1. Create storage location
    let location_request = StorageLocationFixtures::create_location_request();
    let create_location_request = Request::builder()
        .method("POST")
        .uri("/storage/locations")
        .header("content-type", "application/json")
        .body(Body::from(serde_json::to_string(&location_request).unwrap()))
        .unwrap();

    let response = app.clone().oneshot(create_location_request).await.unwrap();
    assert_eq!(response.status(), StatusCode::OK);

    let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
    let location_response: enhanced_storage_service::models::ApiResponse<enhanced_storage_service::models::StorageLocation> 
        = serde_json::from_slice(&body).unwrap();
    let location = location_response.data.unwrap();

    // 2. Store sample in location
    let sample_request = SampleFixtures::store_sample_request(location.id);
    let store_sample_request = Request::builder()
        .method("POST")
        .uri("/storage/samples")
        .header("content-type", "application/json")
        .body(Body::from(serde_json::to_string(&sample_request).unwrap()))
        .unwrap();

    let response = app.clone().oneshot(store_sample_request).await.unwrap();
    assert_eq!(response.status(), StatusCode::OK);

    let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
    let sample_response: enhanced_storage_service::models::ApiResponse<enhanced_storage_service::models::Sample> 
        = serde_json::from_slice(&body).unwrap();
    let sample = sample_response.data.unwrap();

    // 3. Get sample location
    let get_location_request = Request::builder()
        .method("GET")
        .uri(&format!("/storage/samples/{}/location", sample.id))
        .body(Body::empty())
        .unwrap();

    let response = app.clone().oneshot(get_location_request).await.unwrap();
    assert_eq!(response.status(), StatusCode::OK);

    // 4. Create second location
    let mut location2_request = StorageLocationFixtures::create_location_request();
    location2_request.name = "Target Location".to_string();
    let create_location2_request = Request::builder()
        .method("POST")
        .uri("/storage/locations")
        .header("content-type", "application/json")
        .body(Body::from(serde_json::to_string(&location2_request).unwrap()))
        .unwrap();

    let response = app.clone().oneshot(create_location2_request).await.unwrap();
    assert_eq!(response.status(), StatusCode::OK);

    let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
    let location2_response: enhanced_storage_service::models::ApiResponse<enhanced_storage_service::models::StorageLocation> 
        = serde_json::from_slice(&body).unwrap();
    let location2 = location2_response.data.unwrap();

    // 5. Move sample to new location
    let move_request = SampleFixtures::move_sample_request(location2.id);
    let move_sample_request = Request::builder()
        .method("POST")
        .uri(&format!("/storage/samples/{}/move", sample.id))
        .header("content-type", "application/json")
        .body(Body::from(serde_json::to_string(&move_request).unwrap()))
        .unwrap();

    let response = app.clone().oneshot(move_sample_request).await.unwrap();
    assert_eq!(response.status(), StatusCode::OK);

    // 6. Retrieve sample
    let retrieve_request = Request::builder()
        .method("POST")
        .uri(&format!("/storage/samples/{}/retrieve", sample.id))
        .header("content-type", "application/json")
        .body(Body::empty())
        .unwrap();

    let response = app.clone().oneshot(retrieve_request).await.unwrap();
    assert_eq!(response.status(), StatusCode::OK);

    let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
    let retrieved_response: enhanced_storage_service::models::ApiResponse<enhanced_storage_service::models::Sample> 
        = serde_json::from_slice(&body).unwrap();
    let retrieved_sample = retrieved_response.data.unwrap();

    assert_eq!(retrieved_sample.status, "retrieved");
    assert!(retrieved_sample.storage_location_id.is_none());

    test_db.cleanup().await.unwrap();
}

#[tokio::test]
async fn test_iot_monitoring_workflow() {
    let test_db = TestDatabase::new().await.unwrap();
    let app_state = TestAppStateBuilder::new()
        .with_database(test_db.pool.clone())
        .build()
        .await
        .unwrap();
    
    let app = create_app(app_state);

    // 1. Create storage location
    let location_request = StorageLocationFixtures::create_location_request();
    let create_location_request = Request::builder()
        .method("POST")
        .uri("/storage/locations")
        .header("content-type", "application/json")
        .body(Body::from(serde_json::to_string(&location_request).unwrap()))
        .unwrap();

    let response = app.clone().oneshot(create_location_request).await.unwrap();
    assert_eq!(response.status(), StatusCode::OK);

    let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
    let location_response: enhanced_storage_service::models::ApiResponse<enhanced_storage_service::models::StorageLocation> 
        = serde_json::from_slice(&body).unwrap();
    let location = location_response.data.unwrap();

    // 2. Register IoT sensor for location
    let sensor_request = serde_json::json!({
        "sensor_id": TestDataFactory::sensor_id(),
        "sensor_type": "temperature",
        "location_id": location.id,
        "calibration_data": {
            "accuracy": 0.95,
            "offset": 0.1
        }
    });

    let register_sensor_request = Request::builder()
        .method("POST")
        .uri("/iot/sensors")
        .header("content-type", "application/json")
        .body(Body::from(serde_json::to_string(&sensor_request).unwrap()))
        .unwrap();

    let response = app.clone().oneshot(register_sensor_request).await.unwrap();
    assert_eq!(response.status(), StatusCode::OK);

    // 3. List sensors
    let list_sensors_request = Request::builder()
        .method("GET")
        .uri("/iot/sensors")
        .body(Body::empty())
        .unwrap();

    let response = app.clone().oneshot(list_sensors_request).await.unwrap();
    assert_eq!(response.status(), StatusCode::OK);

    // 4. Check IoT health
    let iot_health_request = Request::builder()
        .method("GET")
        .uri("/iot/health")
        .body(Body::empty())
        .unwrap();

    let response = app.clone().oneshot(iot_health_request).await.unwrap();
    assert_eq!(response.status(), StatusCode::OK);

    test_db.cleanup().await.unwrap();
}

#[tokio::test]
async fn test_capacity_management_workflow() {
    let test_db = TestDatabase::new().await.unwrap();
    let app_state = TestAppStateBuilder::new()
        .with_database(test_db.pool.clone())
        .build()
        .await
        .unwrap();
    
    let app = create_app(app_state);

    // 1. Create location with limited capacity
    let mut location_request = StorageLocationFixtures::create_location_request();
    location_request.max_capacity = 2;

    let create_location_request = Request::builder()
        .method("POST")
        .uri("/storage/locations")
        .header("content-type", "application/json")
        .body(Body::from(serde_json::to_string(&location_request).unwrap()))
        .unwrap();

    let response = app.clone().oneshot(create_location_request).await.unwrap();
    assert_eq!(response.status(), StatusCode::OK);

    let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
    let location_response: enhanced_storage_service::models::ApiResponse<enhanced_storage_service::models::StorageLocation> 
        = serde_json::from_slice(&body).unwrap();
    let location = location_response.data.unwrap();

    // 2. Check initial capacity
    let capacity_request = Request::builder()
        .method("GET")
        .uri(&format!("/storage/locations/{}/capacity", location.id))
        .body(Body::empty())
        .unwrap();

    let response = app.clone().oneshot(capacity_request).await.unwrap();
    assert_eq!(response.status(), StatusCode::OK);

    // 3. Store first sample
    let sample1_request = SampleFixtures::store_sample_request(location.id);
    let store_sample1_request = Request::builder()
        .method("POST")
        .uri("/storage/samples")
        .header("content-type", "application/json")
        .body(Body::from(serde_json::to_string(&sample1_request).unwrap()))
        .unwrap();

    let response = app.clone().oneshot(store_sample1_request).await.unwrap();
    assert_eq!(response.status(), StatusCode::OK);

    // 4. Store second sample
    let sample2_request = SampleFixtures::store_sample_request(location.id);
    let store_sample2_request = Request::builder()
        .method("POST")
        .uri("/storage/samples")
        .header("content-type", "application/json")
        .body(Body::from(serde_json::to_string(&sample2_request).unwrap()))
        .unwrap();

    let response = app.clone().oneshot(store_sample2_request).await.unwrap();
    assert_eq!(response.status(), StatusCode::OK);

    // 5. Try to store third sample (should fail)
    let sample3_request = SampleFixtures::store_sample_request(location.id);
    let store_sample3_request = Request::builder()
        .method("POST")
        .uri("/storage/samples")
        .header("content-type", "application/json")
        .body(Body::from(serde_json::to_string(&sample3_request).unwrap()))
        .unwrap();

    let response = app.clone().oneshot(store_sample3_request).await.unwrap();
    assert_eq!(response.status(), StatusCode::BAD_REQUEST);

    // 6. Check final capacity (should show critical status)
    let final_capacity_request = Request::builder()
        .method("GET")
        .uri(&format!("/storage/locations/{}/capacity", location.id))
        .body(Body::empty())
        .unwrap();

    let response = app.clone().oneshot(final_capacity_request).await.unwrap();
    assert_eq!(response.status(), StatusCode::OK);

    let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
    let capacity_response: enhanced_storage_service::models::ApiResponse<enhanced_storage_service::handlers::storage::CapacityInfo> 
        = serde_json::from_slice(&body).unwrap();
    let capacity_info = capacity_response.data.unwrap();

    assert_eq!(capacity_info.current_capacity, 2);
    assert_eq!(capacity_info.max_capacity, 2);
    assert_eq!(capacity_info.utilization_percentage, 100.0);
    assert_eq!(capacity_info.status, "critical");

    test_db.cleanup().await.unwrap();
}

#[tokio::test]
async fn test_error_handling_workflow() {
    let test_db = TestDatabase::new().await.unwrap();
    let app_state = TestAppStateBuilder::new()
        .with_database(test_db.pool.clone())
        .build()
        .await
        .unwrap();
    
    let app = create_app(app_state);

    // 1. Try to get non-existent location
    let non_existent_id = Uuid::new_v4();
    let get_location_request = Request::builder()
        .method("GET")
        .uri(&format!("/storage/locations/{}", non_existent_id))
        .body(Body::empty())
        .unwrap();

    let response = app.clone().oneshot(get_location_request).await.unwrap();
    assert_eq!(response.status(), StatusCode::NOT_FOUND);

    // 2. Try to create location with invalid data
    let invalid_location = serde_json::json!({
        "name": "",  // Invalid empty name
        "location_type": "rack",
        "temperature_zone": "-20C",
        "max_capacity": 0  // Invalid capacity
    });

    let create_invalid_request = Request::builder()
        .method("POST")
        .uri("/storage/locations")
        .header("content-type", "application/json")
        .body(Body::from(serde_json::to_string(&invalid_location).unwrap()))
        .unwrap();

    let response = app.clone().oneshot(create_invalid_request).await.unwrap();
    assert_eq!(response.status(), StatusCode::BAD_REQUEST);

    // 3. Try to store sample in non-existent location
    let sample_request = serde_json::json!({
        "barcode": "TEST123",
        "sample_type": "blood",
        "storage_location_id": non_existent_id,
        "temperature_requirements": "-20C"
    });

    let store_sample_request = Request::builder()
        .method("POST")
        .uri("/storage/samples")
        .header("content-type", "application/json")
        .body(Body::from(serde_json::to_string(&sample_request).unwrap()))
        .unwrap();

    let response = app.clone().oneshot(store_sample_request).await.unwrap();
    assert_eq!(response.status(), StatusCode::NOT_FOUND);

    test_db.cleanup().await.unwrap();
}

#[tokio::test]
async fn test_health_endpoints() {
    let test_db = TestDatabase::new().await.unwrap();
    let app_state = TestAppStateBuilder::new()
        .with_database(test_db.pool.clone())
        .build()
        .await
        .unwrap();
    
    let app = create_app(app_state);

    // 1. Test health check
    let health_request = Request::builder()
        .method("GET")
        .uri("/health")
        .body(Body::empty())
        .unwrap();

    let response = app.clone().oneshot(health_request).await.unwrap();
    assert_eq!(response.status(), StatusCode::OK);

    // 2. Test readiness check
    let ready_request = Request::builder()
        .method("GET")
        .uri("/health/ready")
        .body(Body::empty())
        .unwrap();

    let response = app.clone().oneshot(ready_request).await.unwrap();
    assert_eq!(response.status(), StatusCode::OK);

    // 3. Test metrics endpoint
    let metrics_request = Request::builder()
        .method("GET")
        .uri("/health/metrics")
        .body(Body::empty())
        .unwrap();

    let response = app.clone().oneshot(metrics_request).await.unwrap();
    assert_eq!(response.status(), StatusCode::OK);

    test_db.cleanup().await.unwrap();
} 
