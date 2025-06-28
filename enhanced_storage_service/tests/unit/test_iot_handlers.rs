use crate::{test_utils::*, fixtures::*};
use axum::{extract::{Path, Query, State}, Json};
use enhanced_storage_service::{
    handlers::iot::*,
    models::*,
    error::StorageError,
};
use uuid::Uuid;

#[tokio::test]
async fn test_register_sensor_success() {
    let test_db = TestDatabase::new().await.unwrap();
    let app_state = TestAppStateBuilder::new()
        .with_database(test_db.pool.clone())
        .build()
        .await
        .unwrap();

    // Create a location first
    let location_request = StorageLocationFixtures::create_location_request();
    let location_result = enhanced_storage_service::handlers::storage::create_location(
        State(app_state.clone()), 
        Json(location_request)
    ).await;
    let location = location_result.unwrap().0.data.unwrap();

    let sensor_request = RegisterSensorRequest {
        sensor_id: TestDataFactory::sensor_id(),
        sensor_type: "temperature".to_string(),
        location_id: Some(location.id),
        battery_level: Some(90),
        signal_strength: Some(85),
        firmware_version: Some("1.2.3".to_string()),
        configuration: Some(serde_json::json!({
            "accuracy": 0.95,
            "offset": 0.1
        })),
    };

    let result = register_sensor(State(app_state), Json(sensor_request.clone())).await;

    assert!(result.is_ok(), "Register sensor should succeed");
    let response = result.unwrap();
    let api_response = response.0;

    TestAssertions::assert_api_response_success(&api_response);
    let sensor = api_response.data.unwrap();

    assert_eq!(sensor.sensor_id, sensor_request.sensor_id);
    assert_eq!(sensor.sensor_type, "temperature");
    assert_eq!(sensor.location_id, Some(location.id));
    assert_eq!(sensor.status, "active");
    TestAssertions::assert_timestamp_recent(sensor.created_at, 5);

    test_db.cleanup().await.unwrap();
}

#[tokio::test]
async fn test_list_sensors() {
    let test_db = TestDatabase::new().await.unwrap();
    let app_state = TestAppStateBuilder::new()
        .with_database(test_db.pool.clone())
        .build()
        .await
        .unwrap();

    // Create multiple sensors with unique IDs
    let sensors_to_create = 5;
    let mut created_sensor_ids = Vec::new();
    let test_run_id = Uuid::new_v4().to_string().chars().take(8).collect::<String>();
    
    for i in 0..sensors_to_create {
        let sensor_request = RegisterSensorRequest {
            sensor_id: format!("TEST_SENSOR_{}_{}", test_run_id, i),
            sensor_type: if i % 2 == 0 { "temperature" } else { "humidity" }.to_string(),
            location_id: None,
            battery_level: Some(95),
            signal_strength: Some(80),
            firmware_version: Some("1.0.0".to_string()),
            configuration: Some(serde_json::json!({"test": true})),
        };

        let result = register_sensor(State(app_state.clone()), Json(sensor_request.clone())).await;
        match &result {
            Ok(response) => {
                if let Some(sensor) = &response.0.data {
                    created_sensor_ids.push(sensor.sensor_id.clone());
                    println!("Created sensor: {}", sensor.sensor_id);
                }
            }
            Err(e) => {
                panic!("Sensor registration failed for sensor {}: {:?}", i, e);
            }
        }
    }

    println!("Total sensors created: {}", created_sensor_ids.len());

    let query = SensorListQuery {
        page: Some(1),
        per_page: Some(100), // Increase page size to ensure we get all sensors
        sensor_type: None,
        status: Some("active".to_string()), // Filter by active status
        location_id: None,
    };

    let result = list_sensors(State(app_state), Query(query)).await;

    assert!(result.is_ok(), "List sensors should succeed");
    let response = result.unwrap();
    let api_response = response.0;

    TestAssertions::assert_api_response_success(&api_response);
    let sensors = api_response.data.unwrap();

    println!("Total sensors returned: {}", sensors.data.len());
    for sensor in &sensors.data {
        println!("Found sensor: {} (type: {}, status: {})", sensor.sensor_id, sensor.sensor_type, sensor.status);
    }

    // Count how many of our test sensors are in the response
    let our_sensors_count = sensors.data.iter()
        .filter(|s| created_sensor_ids.contains(&s.sensor_id))
        .count();
    
    println!("Our sensors found: {}", our_sensors_count);
    
    assert_eq!(our_sensors_count, sensors_to_create, 
        "Should find all {} created test sensors, but found {}", 
        sensors_to_create, our_sensors_count);

    test_db.cleanup().await.unwrap();
}

#[tokio::test]
async fn test_record_sensor_reading_success() {
    let test_db = TestDatabase::new().await.unwrap();
    let app_state = TestAppStateBuilder::new()
        .with_database(test_db.pool.clone())
        .build()
        .await
        .unwrap();

    // Register a sensor first
    let sensor_request = RegisterSensorRequest {
        sensor_id: TestDataFactory::sensor_id(),
        sensor_type: "temperature".to_string(),
        location_id: None,
        battery_level: Some(90),
        signal_strength: Some(85),
        firmware_version: None,
        configuration: None,
    };

    let sensor_result = register_sensor(State(app_state.clone()), Json(sensor_request.clone())).await;
    let sensor = sensor_result.unwrap().0.data.unwrap();

    // Record sensor reading using the correct request type
    let reading_request = RecordReadingRequest {
        value: -20.5,
        unit: Some("celsius".to_string()),
        timestamp: Some(chrono::Utc::now()),
        metadata: Some(serde_json::json!({"quality": "good"})),
    };

    let result = record_sensor_reading(
        State(app_state), 
        Path(sensor.sensor_id.to_string()), 
        Json(reading_request)
    ).await;

    assert!(result.is_ok(), "Record sensor reading should succeed");
    let response = result.unwrap();
    let api_response = response.0;

    TestAssertions::assert_api_response_success(&api_response);
    let reading = api_response.data.unwrap();
    assert_eq!(reading.value, -20.5);
    assert_eq!(reading.unit, "celsius");

    test_db.cleanup().await.unwrap();
}

#[tokio::test]
async fn test_get_sensor_data() {
    let test_db = TestDatabase::new().await.unwrap();
    let app_state = TestAppStateBuilder::new()
        .with_database(test_db.pool.clone())
        .build()
        .await
        .unwrap();

    // Register a sensor and record some data
    let sensor_request = RegisterSensorRequest {
        sensor_id: TestDataFactory::sensor_id(),
        sensor_type: "temperature".to_string(),
        location_id: None,
        battery_level: Some(90),
        signal_strength: Some(85),
        firmware_version: None,
        configuration: None,
    };

    let sensor_result = register_sensor(State(app_state.clone()), Json(sensor_request.clone())).await;
    let sensor = sensor_result.unwrap().0.data.unwrap();

    // Record multiple readings
    for i in 0..3 {
        let reading_request = RecordReadingRequest {
            value: -20.0 + (i as f64 * 0.1),
            unit: Some("celsius".to_string()),
            timestamp: Some(chrono::Utc::now()),
            metadata: Some(serde_json::json!({"sequence": i})),
        };

        let result = record_sensor_reading(
            State(app_state.clone()), 
            Path(sensor.sensor_id.to_string()), 
            Json(reading_request)
        ).await;
        assert!(result.is_ok(), "Record sensor reading should succeed");
    }

    // Get sensor data using correct query structure
    let query = SensorDataQuery {
        hours_back: Some(24),
        limit: Some(10),
    };

    let result = get_sensor_data(State(app_state), Path(sensor.sensor_id.to_string()), Query(query)).await;

    assert!(result.is_ok(), "Get sensor data should succeed");
    let response = result.unwrap();
    let api_response = response.0;

    TestAssertions::assert_api_response_success(&api_response);
    let sensor_data = api_response.data.unwrap();

    assert!(sensor_data.readings.len() >= 3);
    
    for reading in &sensor_data.readings {
        assert_eq!(reading.unit, "celsius");
        assert!(reading.value >= -20.5 && reading.value <= -19.5);
    }

    test_db.cleanup().await.unwrap();
}

#[tokio::test]
async fn test_get_alerts() {
    let test_db = TestDatabase::new().await.unwrap();
    let app_state = TestAppStateBuilder::new()
        .with_database(test_db.pool.clone())
        .build()
        .await
        .unwrap();

    let query = AlertQuery {
        page: Some(1),
        per_page: Some(10),
        sensor_id: None,
        severity: None,
        resolved: Some(false),
    };

    let result = get_alerts(State(app_state), Query(query)).await;

    assert!(result.is_ok(), "Get alerts should succeed");
    let response = result.unwrap();
    let api_response = response.0;

    TestAssertions::assert_api_response_success(&api_response);
    let alerts = api_response.data.unwrap();

    // Initially should be empty or contain any default alerts
    assert!(alerts.data.len() >= 0);

    test_db.cleanup().await.unwrap();
}

#[tokio::test]
async fn test_sensor_health_check() {
    let test_db = TestDatabase::new().await.unwrap();
    let app_state = TestAppStateBuilder::new()
        .with_database(test_db.pool.clone())
        .build()
        .await
        .unwrap();

    let result = get_sensor_health(State(app_state)).await;

    assert!(result.is_ok(), "Sensor health check should succeed");
    let response = result.unwrap();
    let api_response = response.0;

    TestAssertions::assert_api_response_success(&api_response);
    let health_status = api_response.data.unwrap();

    assert!(health_status.total_sensors >= 0);
    assert!(health_status.active_sensors >= 0);
    assert!(health_status.active_sensors <= health_status.total_sensors);

    test_db.cleanup().await.unwrap();
} 
