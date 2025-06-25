use crate::test_utils::*;
use enhanced_storage_service::{
    models::*,
    handlers::*,
    services::*,
    create_app,
};
use axum_test::TestServer;
use serde_json::json;
use uuid::Uuid;

/// Integration tests for complete storage workflows
#[tokio::test]
async fn test_complete_sample_storage_lifecycle() {
    let mut test_db = TestDatabase::new().await;
    let app = create_app().await;
    let client = StorageTestClient::new(app);

    // Phase 1: Create storage location
    let location_request = StorageFactory::create_valid_location_request();
    let location_name = location_request.name.clone();
    
    let response = client.post_json("/api/storage/locations", &location_request).await;
    StorageAssertions::assert_status_code(response.status_code(), axum::http::StatusCode::CREATED);
    
    let location_data: serde_json::Value = response.json();
    StorageAssertions::assert_location_data(&location_data, &location_name);
    
    let location_id = Uuid::parse_str(location_data["data"]["id"].as_str().unwrap()).unwrap();
    test_db.track_location(location_id);

    // Phase 2: Create container in location
    let mut container_request = StorageFactory::create_valid_container_request();
    container_request.location_id = location_id;
    let container_barcode = container_request.barcode.clone();
    
    let response = client.post_json("/api/storage/containers", &container_request).await;
    StorageAssertions::assert_status_code(response.status_code(), axum::http::StatusCode::CREATED);
    
    let container_data: serde_json::Value = response.json();
    StorageAssertions::assert_container_data(&container_data, &container_barcode);
    
    let container_id = Uuid::parse_str(container_data["data"]["id"].as_str().unwrap()).unwrap();
    test_db.track_container(container_id);

    // Phase 3: Install IoT sensor
    let mut sensor_request = StorageFactory::create_valid_sensor_request();
    sensor_request.location_id = location_id;
    let sensor_identifier = sensor_request.identifier.clone();
    
    let response = client.post_json("/api/storage/sensors", &sensor_request).await;
    StorageAssertions::assert_status_code(response.status_code(), axum::http::StatusCode::CREATED);
    
    let sensor_data: serde_json::Value = response.json();
    StorageAssertions::assert_sensor_data(&sensor_data, "Temperature");
    
    let sensor_id = Uuid::parse_str(sensor_data["data"]["id"].as_str().unwrap()).unwrap();
    test_db.track_sensor(sensor_id);

    // Phase 4: Simulate sensor readings over time
    let readings = vec![-80.1, -80.0, -79.9, -80.2, -80.1];
    for (i, temp) in readings.iter().enumerate() {
        let reading_request = json!({
            "sensor_id": sensor_id,
            "value": temp,
            "unit": "°C",
            "timestamp": chrono::Utc::now() + chrono::Duration::minutes(i as i64),
            "quality": "Good"
        });
        
        let response = client.post_json("/api/storage/sensor-readings", &reading_request).await;
        StorageAssertions::assert_status_code(response.status_code(), axum::http::StatusCode::CREATED);
    }

    // Phase 5: Move container (blockchain event)
    let move_request = json!({
        "container_id": container_id,
        "new_location_id": location_id,
        "new_position": "B2",
        "moved_by": Uuid::new_v4(),
        "reason": "Routine reorganization"
    });
    
    let response = client.post_json("/api/storage/containers/move", &move_request).await;
    StorageAssertions::assert_status_code(response.status_code(), axum::http::StatusCode::OK);
    
    let move_data: serde_json::Value = response.json();
    StorageAssertions::assert_blockchain_transaction(&move_data);

    // Phase 6: Query complete audit trail
    let response = client.get(&format!("/api/storage/containers/{}/audit", container_id)).await;
    StorageAssertions::assert_status_code(response.status_code(), axum::http::StatusCode::OK);
    
    let audit_data: serde_json::Value = response.json();
    assert_eq!(audit_data["success"], true);
    assert!(audit_data["data"]["audit_trail"].is_array());
    
    let audit_trail = audit_data["data"]["audit_trail"].as_array().unwrap();
    assert!(!audit_trail.is_empty(), "Audit trail should contain events");

    // Cleanup
    test_db.cleanup().await;
}

#[tokio::test]
async fn test_iot_alert_workflow() {
    let mut test_db = TestDatabase::new().await;
    let app = create_app().await;
    let client = StorageTestClient::new(app);

    // Setup: Create location and sensor
    let location_request = StorageFactory::create_valid_location_request();
    let response = client.post_json("/api/storage/locations", &location_request).await;
    let location_data: serde_json::Value = response.json();
    let location_id = Uuid::parse_str(location_data["data"]["id"].as_str().unwrap()).unwrap();
    test_db.track_location(location_id);

    let mut sensor_request = StorageFactory::create_valid_sensor_request();
    sensor_request.location_id = location_id;
    sensor_request.alert_threshold_min = Some(-82.0);
    sensor_request.alert_threshold_max = Some(-78.0);
    
    let response = client.post_json("/api/storage/sensors", &sensor_request).await;
    let sensor_data: serde_json::Value = response.json();
    let sensor_id = Uuid::parse_str(sensor_data["data"]["id"].as_str().unwrap()).unwrap();
    test_db.track_sensor(sensor_id);

    // Test: Send reading that triggers alert (temperature too high)
    let alert_reading = json!({
        "sensor_id": sensor_id,
        "value": -75.0, // Above threshold
        "unit": "°C",
        "timestamp": chrono::Utc::now(),
        "quality": "Good"
    });
    
    let response = client.post_json("/api/storage/sensor-readings", &alert_reading).await;
    StorageAssertions::assert_status_code(response.status_code(), axum::http::StatusCode::CREATED);
    
    let reading_data: serde_json::Value = response.json();
    assert_eq!(reading_data["data"]["alert_triggered"], true);
    assert_eq!(reading_data["data"]["alert_severity"], "Critical");

    // Verify alert was created
    let response = client.get(&format!("/api/storage/locations/{}/alerts", location_id)).await;
    StorageAssertions::assert_status_code(response.status_code(), axum::http::StatusCode::OK);
    
    let alerts_data: serde_json::Value = response.json();
    assert_eq!(alerts_data["success"], true);
    
    let alerts = alerts_data["data"]["alerts"].as_array().unwrap();
    assert!(!alerts.is_empty(), "Should have generated an alert");
    assert_eq!(alerts[0]["alert_type"], "TemperatureThreshold");
    assert_eq!(alerts[0]["severity"], "Critical");

    test_db.cleanup().await;
}

#[tokio::test]
async fn test_digital_twin_temperature_prediction() {
    let mut test_db = TestDatabase::new().await;
    let app = create_app().await;
    let client = StorageTestClient::new(app);

    // Setup location with sensor
    let location_request = StorageFactory::create_valid_location_request();
    let response = client.post_json("/api/storage/locations", &location_request).await;
    let location_data: serde_json::Value = response.json();
    let location_id = Uuid::parse_str(location_data["data"]["id"].as_str().unwrap()).unwrap();
    test_db.track_location(location_id);

    let mut sensor_request = StorageFactory::create_valid_sensor_request();
    sensor_request.location_id = location_id;
    let response = client.post_json("/api/storage/sensors", &sensor_request).await;
    let sensor_data: serde_json::Value = response.json();
    let sensor_id = Uuid::parse_str(sensor_data["data"]["id"].as_str().unwrap()).unwrap();
    test_db.track_sensor(sensor_id);

    // Send historical temperature data to train the digital twin
    let historical_temps = vec![-80.0, -80.1, -79.9, -80.2, -80.0, -79.8, -80.1];
    for (i, temp) in historical_temps.iter().enumerate() {
        let reading = json!({
            "sensor_id": sensor_id,
            "value": temp,
            "unit": "°C",
            "timestamp": chrono::Utc::now() - chrono::Duration::hours(24 - i as i64),
            "quality": "Good"
        });
        
        let _ = client.post_json("/api/storage/sensor-readings", &reading).await;
    }

    // Request digital twin prediction
    let prediction_request = json!({
        "location_id": location_id,
        "prediction_horizon_hours": 4,
        "include_confidence": true,
        "include_recommendations": true
    });
    
    let response = client.post_json("/api/storage/digital-twin/predict", &prediction_request).await;
    StorageAssertions::assert_status_code(response.status_code(), axum::http::StatusCode::OK);
    
    let prediction_data: serde_json::Value = response.json();
    DigitalTwinTestUtils::assert_twin_prediction(&prediction_data, 0.85);
    
    assert!(prediction_data["data"]["predictions"].is_array());
    assert!(prediction_data["data"]["recommendations"].is_array());
    assert!(prediction_data["data"]["model_accuracy"].is_number());

    test_db.cleanup().await;
}

#[tokio::test]
async fn test_blockchain_container_chain_of_custody() {
    let mut test_db = TestDatabase::new().await;
    let app = create_app().await;
    let client = StorageTestClient::new(app);

    // Setup multiple locations
    let location1 = StorageFactory::create_valid_location_request();
    let response = client.post_json("/api/storage/locations", &location1).await;
    let location1_data: serde_json::Value = response.json();
    let location1_id = Uuid::parse_str(location1_data["data"]["id"].as_str().unwrap()).unwrap();
    test_db.track_location(location1_id);

    let location2 = StorageFactory::create_valid_location_request();
    let response = client.post_json("/api/storage/locations", &location2).await;
    let location2_data: serde_json::Value = response.json();
    let location2_id = Uuid::parse_str(location2_data["data"]["id"].as_str().unwrap()).unwrap();
    test_db.track_location(location2_id);

    // Create container in first location
    let mut container_request = StorageFactory::create_valid_container_request();
    container_request.location_id = location1_id;
    let response = client.post_json("/api/storage/containers", &container_request).await;
    let container_data: serde_json::Value = response.json();
    let container_id = Uuid::parse_str(container_data["data"]["id"].as_str().unwrap()).unwrap();
    test_db.track_container(container_id);

    // Perform series of moves to create chain of custody
    let moves = vec![
        (location2_id, "A1", "Moved to secondary storage"),
        (location1_id, "B3", "Returned to primary storage"),
        (location2_id, "C2", "Final relocation"),
    ];

    for (i, (new_location, position, reason)) in moves.iter().enumerate() {
        let move_request = json!({
            "container_id": container_id,
            "new_location_id": new_location,
            "new_position": position,
            "moved_by": Uuid::new_v4(),
            "reason": reason
        });
        
        let response = client.post_json("/api/storage/containers/move", &move_request).await;
        StorageAssertions::assert_status_code(response.status_code(), axum::http::StatusCode::OK);
        
        let move_data: serde_json::Value = response.json();
        StorageAssertions::assert_blockchain_transaction(&move_data);
        
        let tx_hash = move_data["data"]["transaction_hash"].as_str().unwrap();
        test_db.track_transaction(tx_hash.to_string());
    }

    // Verify complete blockchain chain of custody
    let response = client.get(&format!("/api/storage/containers/{}/blockchain-history", container_id)).await;
    StorageAssertions::assert_status_code(response.status_code(), axum::http::StatusCode::OK);
    
    let blockchain_data: serde_json::Value = response.json();
    assert_eq!(blockchain_data["success"], true);
    
    let transactions = blockchain_data["data"]["transactions"].as_array().unwrap();
    assert_eq!(transactions.len(), 4); // Initial creation + 3 moves
    
    // Verify chronological order and data integrity
    for (i, transaction) in transactions.iter().enumerate() {
        assert!(transaction["block_number"].is_number());
        assert!(transaction["transaction_hash"].is_string());
        assert_eq!(transaction["validated"], true);
        
        if i > 0 {
            let prev_block = transactions[i-1]["block_number"].as_u64().unwrap();
            let curr_block = transaction["block_number"].as_u64().unwrap();
            assert!(curr_block >= prev_block, "Blocks should be in chronological order");
        }
    }

    test_db.cleanup().await;
}

#[tokio::test]
async fn test_mobile_app_qr_code_workflow() {
    let mut test_db = TestDatabase::new().await;
    let app = create_app().await;
    let client = StorageTestClient::new(app);

    // Setup
    let location_request = StorageFactory::create_valid_location_request();
    let response = client.post_json("/api/storage/locations", &location_request).await;
    let location_data: serde_json::Value = response.json();
    let location_id = Uuid::parse_str(location_data["data"]["id"].as_str().unwrap()).unwrap();
    test_db.track_location(location_id);

    let mut container_request = StorageFactory::create_valid_container_request();
    container_request.location_id = location_id;
    let response = client.post_json("/api/storage/containers", &container_request).await;
    let container_data: serde_json::Value = response.json();
    let container_id = Uuid::parse_str(container_data["data"]["id"].as_str().unwrap()).unwrap();
    let container_barcode = container_data["data"]["barcode"].as_str().unwrap();
    test_db.track_container(container_id);

    // Test: Mobile app scans QR code
    let mobile_request = MobileTestUtils::create_mobile_request(
        Uuid::new_v4(),
        "mobile-device-token-123".to_string(),
    );
    
    let response = client.post_json("/api/storage/mobile/request-access", &mobile_request).await;
    StorageAssertions::assert_status_code(response.status_code(), axum::http::StatusCode::OK);
    
    let access_data: serde_json::Value = response.json();
    MobileTestUtils::assert_mobile_response(&access_data);

    // Test: Scan container QR code
    let qr_data = format!("CONTAINER:{}", container_barcode);
    let response = MobileTestUtils::test_qr_code_scan(&client, &qr_data).await;
    StorageAssertions::assert_status_code(response.status_code(), axum::http::StatusCode::OK);
    
    let scan_data: serde_json::Value = response.json();
    assert_eq!(scan_data["success"], true);
    assert_eq!(scan_data["data"]["container_id"], container_id.to_string());
    assert!(scan_data["data"]["location_info"].is_object());
    assert!(scan_data["data"]["access_permissions"].is_object());

    test_db.cleanup().await;
}

#[tokio::test]
async fn test_storage_capacity_management() {
    let mut test_db = TestDatabase::new().await;
    let app = create_app().await;
    let client = StorageTestClient::new(app);

    // Create location with limited capacity
    let mut location_request = StorageFactory::create_valid_location_request();
    location_request.capacity = 100; // Small capacity for testing
    
    let response = client.post_json("/api/storage/locations", &location_request).await;
    let location_data: serde_json::Value = response.json();
    let location_id = Uuid::parse_str(location_data["data"]["id"].as_str().unwrap()).unwrap();
    test_db.track_location(location_id);

    // Fill location to near capacity
    let mut container_ids = Vec::new();
    for i in 0..8 { // 80% capacity
        let mut container_request = StorageFactory::create_valid_container_request();
        container_request.location_id = location_id;
        container_request.capacity = 10; // Each container takes 10 units
        container_request.barcode = format!("CONT-{:03}", i);
        
        let response = client.post_json("/api/storage/containers", &container_request).await;
        let container_data: serde_json::Value = response.json();
        let container_id = Uuid::parse_str(container_data["data"]["id"].as_str().unwrap()).unwrap();
        container_ids.push(container_id);
        test_db.track_container(container_id);
    }

    // Check capacity utilization
    let response = client.get(&format!("/api/storage/locations/{}/capacity", location_id)).await;
    StorageAssertions::assert_status_code(response.status_code(), axum::http::StatusCode::OK);
    
    let capacity_data: serde_json::Value = response.json();
    assert_eq!(capacity_data["success"], true);
    assert_eq!(capacity_data["data"]["total_capacity"], 100);
    assert_eq!(capacity_data["data"]["used_capacity"], 80);
    assert_eq!(capacity_data["data"]["utilization_percentage"], 80.0);
    assert_eq!(capacity_data["data"]["status"], "NearFull");

    // Try to add container that would exceed capacity
    let mut over_capacity_request = StorageFactory::create_valid_container_request();
    over_capacity_request.location_id = location_id;
    over_capacity_request.capacity = 25; // Would exceed remaining 20 units
    
    let response = client.post_json("/api/storage/containers", &over_capacity_request).await;
    StorageAssertions::assert_status_code(response.status_code(), axum::http::StatusCode::BAD_REQUEST);
    
    let error_data: serde_json::Value = response.json();
    StorageAssertions::assert_validation_error(&error_data);
    assert!(error_data["error"].as_str().unwrap().contains("capacity"));

    test_db.cleanup().await;
}

#[tokio::test] 
async fn test_concurrent_operations_stress_test() {
    let mut test_db = TestDatabase::new().await;
    let app = create_app().await;
    let client = StorageTestClient::new(app);

    // Setup base location
    let location_request = StorageFactory::create_valid_location_request();
    let response = client.post_json("/api/storage/locations", &location_request).await;
    let location_data: serde_json::Value = response.json();
    let location_id = Uuid::parse_str(location_data["data"]["id"].as_str().unwrap()).unwrap();
    test_db.track_location(location_id);

    // Setup sensor for concurrent readings
    let mut sensor_request = StorageFactory::create_valid_sensor_request();
    sensor_request.location_id = location_id;
    let response = client.post_json("/api/storage/sensors", &sensor_request).await;
    let sensor_data: serde_json::Value = response.json();
    let sensor_id = Uuid::parse_str(sensor_data["data"]["id"].as_str().unwrap()).unwrap();
    test_db.track_sensor(sensor_id);

    // Test: Concurrent sensor readings
    let concurrent_readings = StoragePerformanceUtils::concurrent_sensor_readings(
        &client,
        sensor_id,
        20,
    ).await;
    
    let successful_readings = concurrent_readings.iter()
        .filter(|&status| *status == axum::http::StatusCode::CREATED)
        .count();
    
    assert!(successful_readings >= 18, "At least 90% of concurrent readings should succeed");

    // Test: Concurrent container operations
    let concurrent_containers = (0..10)
        .map(|i| {
            let mut request = StorageFactory::create_valid_container_request();
            request.location_id = location_id;
            request.barcode = format!("CONCURRENT-{:03}", i);
            async move {
                let response = client.post_json("/api/storage/containers", &request).await;
                if response.status_code() == axum::http::StatusCode::CREATED {
                    let data: serde_json::Value = response.json();
                    let container_id = Uuid::parse_str(data["data"]["id"].as_str().unwrap()).unwrap();
                    Some(container_id)
                } else {
                    None
                }
            }
        });

    let container_results = futures::future::join_all(concurrent_containers).await;
    let successful_containers: Vec<_> = container_results.into_iter().flatten().collect();
    
    for container_id in &successful_containers {
        test_db.track_container(*container_id);
    }
    
    assert!(successful_containers.len() >= 8, "At least 80% of concurrent container creation should succeed");

    test_db.cleanup().await;
}