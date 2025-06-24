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
        calibration_data: Some(serde_json::json!({
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

    // Create multiple sensors
    let sensors_to_create = 5;
    for i in 0..sensors_to_create {
        let sensor_request = RegisterSensorRequest {
            sensor_id: format!("SENSOR_{}", i),
            sensor_type: if i % 2 == 0 { "temperature" } else { "humidity" }.to_string(),
            location_id: None,
            calibration_data: Some(serde_json::json!({"test": true})),
        };

        let result = register_sensor(State(app_state.clone()), Json(sensor_request)).await;
        assert!(result.is_ok(), "Sensor registration should succeed");
    }

    let result = list_sensors(State(app_state)).await;

    assert!(result.is_ok(), "List sensors should succeed");
    let response = result.unwrap();
    let api_response = response.0;

    TestAssertions::assert_api_response_success(&api_response);
    let sensors = api_response.data.unwrap();

    assert!(sensors.len() >= sensors_to_create);

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
        calibration_data: None,
    };

    let sensor_result = register_sensor(State(app_state.clone()), Json(sensor_request.clone())).await;
    let sensor = sensor_result.unwrap().0.data.unwrap();

    // Record sensor reading
    let reading_request = SensorReading {
        sensor_id: sensor.sensor_id.clone(),
        readings: vec![
            SensorReadingValue {
                reading_type: "temperature".to_string(),
                value: -20.5,
                unit: "celsius".to_string(),
                quality_score: Some(0.98),
            },
        ],
        timestamp: chrono::Utc::now(),
    };

    let result = record_sensor_reading(
        State(app_state), 
        Path(sensor.id), 
        Json(reading_request)
    ).await;

    assert!(result.is_ok(), "Record sensor reading should succeed");
    let response = result.unwrap();
    let api_response = response.0;

    TestAssertions::assert_api_response_success(&api_response);
    assert!(api_response.data.unwrap().contains("recorded successfully"));

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
        calibration_data: None,
    };

    let sensor_result = register_sensor(State(app_state.clone()), Json(sensor_request.clone())).await;
    let sensor = sensor_result.unwrap().0.data.unwrap();

    // Record multiple readings
    for i in 0..3 {
        let reading_request = SensorReading {
            sensor_id: sensor.sensor_id.clone(),
            readings: vec![
                SensorReadingValue {
                    reading_type: "temperature".to_string(),
                    value: -20.0 + (i as f64 * 0.1),
                    unit: "celsius".to_string(),
                    quality_score: Some(0.98),
                },
            ],
            timestamp: chrono::Utc::now(),
        };

        let result = record_sensor_reading(
            State(app_state.clone()), 
            Path(sensor.id), 
            Json(reading_request)
        ).await;
        assert!(result.is_ok(), "Record sensor reading should succeed");
    }

    // Get sensor data
    let query = SensorDataQuery {
        start_time: Some(chrono::Utc::now() - chrono::Duration::hours(1)),
        end_time: Some(chrono::Utc::now()),
        reading_type: Some("temperature".to_string()),
        limit: Some(10),
    };

    let result = get_sensor_data(State(app_state), Path(sensor.id), Query(query)).await;

    assert!(result.is_ok(), "Get sensor data should succeed");
    let response = result.unwrap();
    let api_response = response.0;

    TestAssertions::assert_api_response_success(&api_response);
    let sensor_data = api_response.data.unwrap();

    assert!(sensor_data.len() >= 3);
    
    for data_point in &sensor_data {
        assert_eq!(data_point.reading_type, "temperature");
        assert_eq!(data_point.unit, "celsius");
        assert!(data_point.value >= -20.5 && data_point.value <= -19.5);
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

    let result = get_alerts(State(app_state)).await;

    assert!(result.is_ok(), "Get alerts should succeed");
    let response = result.unwrap();
    let api_response = response.0;

    TestAssertions::assert_api_response_success(&api_response);
    let alerts = api_response.data.unwrap();

    // Initially should be empty or contain any default alerts
    assert!(alerts.len() >= 0);

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

// Helper structs for test requests (these would normally be in the models)
#[derive(serde::Serialize, serde::Deserialize)]
struct RegisterSensorRequest {
    sensor_id: String,
    sensor_type: String,
    location_id: Option<Uuid>,
    calibration_data: Option<serde_json::Value>,
}

#[derive(serde::Deserialize)]
struct SensorDataQuery {
    start_time: Option<chrono::DateTime<chrono::Utc>>,
    end_time: Option<chrono::DateTime<chrono::Utc>>,
    reading_type: Option<String>,
    limit: Option<i32>,
} 
