use crate::{test_utils::*, fixtures::*};
use enhanced_storage_service::{models::*, services::EnhancedStorageService};
use sqlx::Row;
use uuid::Uuid;

#[tokio::test]
async fn test_database_connection_health() {
    let test_db = TestDatabase::new().await.unwrap();
    
    let health_result = test_db.pool.health_check().await;
    assert!(health_result.is_ok(), "Database health check should pass");
    assert!(health_result.unwrap(), "Database should be healthy");

    test_db.cleanup().await.unwrap();
}

#[tokio::test]
async fn test_create_and_retrieve_location() {
    let test_db = TestDatabase::new().await.unwrap();
    let config = test_config();
    let service = EnhancedStorageService::new(test_db.pool.clone(), config).await.unwrap();

    let request = StorageLocationFixtures::create_location_request();
    let created_location = service.create_storage_location(request).await.unwrap();

    let retrieved_location = service.get_storage_location(created_location.id).await.unwrap();

    assert_eq!(created_location.id, retrieved_location.id);
    assert_eq!(created_location.name, retrieved_location.name);
    assert_eq!(created_location.temperature_zone, retrieved_location.temperature_zone);

    test_db.cleanup().await.unwrap();
}

#[tokio::test]
async fn test_sample_storage_workflow() {
    let test_db = TestDatabase::new().await.unwrap();
    let config = test_config();
    let service = EnhancedStorageService::new(test_db.pool.clone(), config).await.unwrap();

    // Create location
    let location_request = StorageLocationFixtures::create_location_request();
    let location = service.create_storage_location(location_request).await.unwrap();

    // Store sample
    let sample_request = SampleFixtures::store_sample_request(location.id);
    let sample = service.store_sample(sample_request).await.unwrap();

    // Verify sample is stored
    assert_eq!(sample.storage_location_id, Some(location.id));
    assert_eq!(sample.status, "stored");

    // Verify location capacity updated
    let updated_location = service.get_storage_location(location.id).await.unwrap();
    assert_eq!(updated_location.current_capacity, 1);

    test_db.cleanup().await.unwrap();
}

#[tokio::test]
async fn test_sample_movement_workflow() {
    let test_db = TestDatabase::new().await.unwrap();
    let config = test_config();
    let service = EnhancedStorageService::new(test_db.pool.clone(), config).await.unwrap();

    // Create two locations
    let location1_request = StorageLocationFixtures::create_location_request();
    let location1 = service.create_storage_location(location1_request).await.unwrap();

    let mut location2_request = StorageLocationFixtures::create_location_request();
    location2_request.name = "Target Location".to_string();
    let location2 = service.create_storage_location(location2_request).await.unwrap();

    // Store sample in first location
    let sample_request = SampleFixtures::store_sample_request(location1.id);
    let sample = service.store_sample(sample_request).await.unwrap();

    // Move sample to second location
    let move_request = SampleFixtures::move_sample_request(location2.id);
    let moved_sample = service.move_sample(sample.id, move_request).await.unwrap();

    // Verify sample moved
    assert_eq!(moved_sample.storage_location_id, Some(location2.id));

    // Verify location capacities updated
    let updated_location1 = service.get_storage_location(location1.id).await.unwrap();
    let updated_location2 = service.get_storage_location(location2.id).await.unwrap();

    assert_eq!(updated_location1.current_capacity, 0);
    assert_eq!(updated_location2.current_capacity, 1);

    test_db.cleanup().await.unwrap();
}

#[tokio::test]
async fn test_iot_sensor_registration_and_data() {
    let test_db = TestDatabase::new().await.unwrap();
    let config = test_config();
    let service = EnhancedStorageService::new(test_db.pool.clone(), config).await.unwrap();

    let sensor_id = TestDataFactory::sensor_id();
    let sensor = service.register_sensor(
        sensor_id.clone(),
        "temperature".to_string(),
        None
    ).await.unwrap();

    assert_eq!(sensor.sensor_id, sensor_id);
    assert_eq!(sensor.sensor_type, "temperature");
    assert_eq!(sensor.status, "active");

    // Record sensor data
    let reading = SensorReading {
        sensor_id: sensor_id.clone(),
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

    let result = service.record_sensor_data(reading).await;
    assert!(result.is_ok(), "Recording sensor data should succeed");

    test_db.cleanup().await.unwrap();
}

#[tokio::test]
async fn test_transaction_rollback_on_error() {
    let test_db = TestDatabase::new().await.unwrap();
    let config = test_config();
    let service = EnhancedStorageService::new(test_db.pool.clone(), config).await.unwrap();

    // Create location with capacity 1
    let mut location_request = StorageLocationFixtures::create_location_request();
    location_request.max_capacity = 1;
    let location = service.create_storage_location(location_request).await.unwrap();

    // Store first sample (should succeed)
    let sample_request1 = SampleFixtures::store_sample_request(location.id);
    let result1 = service.store_sample(sample_request1).await;
    assert!(result1.is_ok(), "First sample should be stored successfully");

    // Try to store second sample (should fail due to capacity)
    let sample_request2 = SampleFixtures::store_sample_request(location.id);
    let result2 = service.store_sample(sample_request2).await;
    assert!(result2.is_err(), "Second sample should fail due to capacity");

    // Verify location capacity is still 1 (transaction rolled back)
    let updated_location = service.get_storage_location(location.id).await.unwrap();
    assert_eq!(updated_location.current_capacity, 1);

    test_db.cleanup().await.unwrap();
}

#[tokio::test]
async fn test_concurrent_location_creation() {
    let test_db = TestDatabase::new().await.unwrap();
    let config = test_config();
    
    // Create multiple services (simulating concurrent access)
    let service1 = EnhancedStorageService::new(test_db.pool.clone(), config.clone()).await.unwrap();
    let service2 = EnhancedStorageService::new(test_db.pool.clone(), config.clone()).await.unwrap();
    let service3 = EnhancedStorageService::new(test_db.pool.clone(), config).await.unwrap();

    // Create locations concurrently
    let handles = vec![
        tokio::spawn(async move {
            let mut request = StorageLocationFixtures::create_location_request();
            request.name = "Concurrent Location 1".to_string();
            service1.create_storage_location(request).await
        }),
        tokio::spawn(async move {
            let mut request = StorageLocationFixtures::create_location_request();
            request.name = "Concurrent Location 2".to_string();
            service2.create_storage_location(request).await
        }),
        tokio::spawn(async move {
            let mut request = StorageLocationFixtures::create_location_request();
            request.name = "Concurrent Location 3".to_string();
            service3.create_storage_location(request).await
        }),
    ];

    let results = futures::future::join_all(handles).await;
    
    for result in results {
        let location_result = result.unwrap();
        assert!(location_result.is_ok(), "Concurrent location creation should succeed");
    }

    test_db.cleanup().await.unwrap();
}

#[tokio::test]
async fn test_database_constraints() {
    let test_db = TestDatabase::new().await.unwrap();

    // Test foreign key constraints
    let non_existent_location_id = Uuid::new_v4();
    
    let result = sqlx::query(
        "INSERT INTO samples (barcode, sample_type, storage_location_id) VALUES ($1, $2, $3)"
    )
    .bind("TEST123")
    .bind("blood")
    .bind(non_existent_location_id)
    .execute(&test_db.pool.pool)
    .await;

    assert!(result.is_err(), "Foreign key constraint should prevent invalid location reference");

    test_db.cleanup().await.unwrap();
}

#[tokio::test]
async fn test_pagination_edge_cases() {
    let test_db = TestDatabase::new().await.unwrap();
    let config = test_config();
    let service = EnhancedStorageService::new(test_db.pool.clone(), config).await.unwrap();

    // Test pagination with no data
    let result = service.list_storage_locations(Some(1), Some(10)).await.unwrap();
    assert_eq!(result.data.len(), 0);
    assert_eq!(result.pagination.total_items, 0);
    assert!(!result.pagination.has_next);
    assert!(!result.pagination.has_prev);

    // Create some locations
    for i in 0..25 {
        let mut request = StorageLocationFixtures::create_location_request();
        request.name = format!("Location {}", i);
        service.create_storage_location(request).await.unwrap();
    }

    // Test various pagination scenarios
    let page1 = service.list_storage_locations(Some(1), Some(10)).await.unwrap();
    assert_eq!(page1.data.len(), 10);
    assert!(page1.pagination.has_next);
    assert!(!page1.pagination.has_prev);

    let page2 = service.list_storage_locations(Some(2), Some(10)).await.unwrap();
    assert_eq!(page2.data.len(), 10);
    assert!(page2.pagination.has_next);
    assert!(page2.pagination.has_prev);

    let page3 = service.list_storage_locations(Some(3), Some(10)).await.unwrap();
    assert!(page3.data.len() >= 5); // Should have remaining items
    assert!(!page3.pagination.has_next);
    assert!(page3.pagination.has_prev);

    test_db.cleanup().await.unwrap();
} 
