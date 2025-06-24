#[cfg(test)]
mod tests {
    use super::*;
    use crate::{test_utils::*, fixtures::*};
    use axum::{
        body::Body,
        extract::{Path, Query, State},
        http::{Request, StatusCode},
        Json,
    };
    use enhanced_storage_service::{
        handlers::storage::*,
        models::*,
        error::StorageError,
    };
    use serde_json;
    use uuid::Uuid;

    #[tokio::test]
    #[test_log::test]
    async fn test_create_location_success() {
        let test_db = TestDatabase::new().await.unwrap();
        let app_state = TestAppStateBuilder::new()
            .with_database(test_db.pool.clone())
            .build()
            .await
            .unwrap();

        let request = StorageLocationFixtures::create_location_request();
        let location_name = request.name.clone();

        let result = create_location(State(app_state), Json(request)).await;

        assert!(result.is_ok(), "Create location should succeed");
        let response = result.unwrap();
        let api_response = response.0;

        TestAssertions::assert_api_response_success(&api_response);
        let location = api_response.data.unwrap();

        assert_eq!(location.name, location_name);
        assert_eq!(location.temperature_zone, "-20C");
        assert_eq!(location.max_capacity, 100);
        assert_eq!(location.current_capacity, 0);
        assert_eq!(location.status, "active");
        TestAssertions::assert_timestamp_recent(location.created_at, 5);

        test_db.cleanup().await.unwrap();
    }

    #[tokio::test]
    #[test_log::test]
    async fn test_create_location_validation_failure() {
        let test_db = TestDatabase::new().await.unwrap();
        let app_state = TestAppStateBuilder::new()
            .with_database(test_db.pool.clone())
            .build()
            .await
            .unwrap();

        let mut request = StorageLocationFixtures::create_location_request();
        request.name = "".to_string(); // Invalid empty name
        request.max_capacity = 0; // Invalid capacity

        let result = create_location(State(app_state), Json(request)).await;

        assert!(result.is_err(), "Create location should fail with validation error");
        let error = result.unwrap_err();
        assert!(matches!(error, StorageError::Validation(_)));

        test_db.cleanup().await.unwrap();
    }

    #[tokio::test]
    #[test_log::test]
    async fn test_create_location_duplicate_name() {
        let test_db = TestDatabase::new().await.unwrap();
        let app_state = TestAppStateBuilder::new()
            .with_database(test_db.pool.clone())
            .build()
            .await
            .unwrap();

        let request = StorageLocationFixtures::create_location_request();

        // Create first location
        let result1 = create_location(State(app_state.clone()), Json(request.clone())).await;
        assert!(result1.is_ok(), "First location creation should succeed");

        // Try to create second location with same name
        let result2 = create_location(State(app_state), Json(request)).await;
        assert!(result2.is_err(), "Second location creation should fail");

        test_db.cleanup().await.unwrap();
    }

    #[tokio::test]
    #[test_log::test]
    async fn test_list_locations_empty() {
        let test_db = TestDatabase::new().await.unwrap();
        let app_state = TestAppStateBuilder::new()
            .with_database(test_db.pool.clone())
            .build()
            .await
            .unwrap();

        // Clear default locations that might be created
        test_db.clear_data().await.unwrap();

        let query = LocationQuery {
            page: Some(1),
            per_page: Some(10),
            temperature_zone: None,
            location_type: None,
        };

        let result = list_locations(State(app_state), Query(query)).await;

        assert!(result.is_ok(), "List locations should succeed even when empty");
        let response = result.unwrap();
        let api_response = response.0;

        TestAssertions::assert_api_response_success(&api_response);
        let paginated_response = api_response.data.unwrap();

        assert_eq!(paginated_response.data.len(), 0);
        assert_eq!(paginated_response.pagination.total_items, 0);
        assert_eq!(paginated_response.pagination.page, 1);
        assert!(!paginated_response.pagination.has_next);
        assert!(!paginated_response.pagination.has_prev);

        test_db.cleanup().await.unwrap();
    }

    #[tokio::test]
    #[test_log::test]
    async fn test_list_locations_with_data() {
        let test_db = TestDatabase::new().await.unwrap();
        let app_state = TestAppStateBuilder::new()
            .with_database(test_db.pool.clone())
            .build()
            .await
            .unwrap();

        // Create multiple locations
        let locations_to_create = 15;
        for i in 0..locations_to_create {
            let mut request = StorageLocationFixtures::create_location_request();
            request.name = format!("Test Location {}", i);
            
            let result = create_location(State(app_state.clone()), Json(request)).await;
            assert!(result.is_ok(), "Location creation should succeed");
        }

        // Test pagination
        let query = LocationQuery {
            page: Some(1),
            per_page: Some(10),
            temperature_zone: None,
            location_type: None,
        };

        let result = list_locations(State(app_state), Query(query)).await;

        assert!(result.is_ok(), "List locations should succeed");
        let response = result.unwrap();
        let api_response = response.0;

        TestAssertions::assert_api_response_success(&api_response);
        let paginated_response = api_response.data.unwrap();

        assert_eq!(paginated_response.data.len(), 10);
        assert!(paginated_response.pagination.total_items >= locations_to_create);
        assert_eq!(paginated_response.pagination.page, 1);
        assert!(paginated_response.pagination.has_next);
        assert!(!paginated_response.pagination.has_prev);

        TestAssertions::assert_pagination_valid(
            1, 10, 
            paginated_response.pagination.total_items,
            paginated_response.data.len(),
            paginated_response.pagination.has_next,
            paginated_response.pagination.has_prev,
        );

        test_db.cleanup().await.unwrap();
    }

    #[tokio::test]
    #[test_log::test]
    async fn test_get_location_success() {
        let test_db = TestDatabase::new().await.unwrap();
        let app_state = TestAppStateBuilder::new()
            .with_database(test_db.pool.clone())
            .build()
            .await
            .unwrap();

        // Create a location first
        let request = StorageLocationFixtures::create_location_request();
        let create_result = create_location(State(app_state.clone()), Json(request)).await;
        let created_location = create_result.unwrap().0.data.unwrap();

        // Get the location
        let result = get_location(State(app_state), Path(created_location.id)).await;

        assert!(result.is_ok(), "Get location should succeed");
        let response = result.unwrap();
        let api_response = response.0;

        TestAssertions::assert_api_response_success(&api_response);
        let location = api_response.data.unwrap();

        assert_eq!(location.id, created_location.id);
        assert_eq!(location.name, created_location.name);
        assert_eq!(location.temperature_zone, created_location.temperature_zone);

        test_db.cleanup().await.unwrap();
    }

    #[tokio::test]
    #[test_log::test]
    async fn test_get_location_not_found() {
        let test_db = TestDatabase::new().await.unwrap();
        let app_state = TestAppStateBuilder::new()
            .with_database(test_db.pool.clone())
            .build()
            .await
            .unwrap();

        let non_existent_id = TestDataFactory::uuid();

        let result = get_location(State(app_state), Path(non_existent_id)).await;

        assert!(result.is_err(), "Get location should fail for non-existent ID");
        let error = result.unwrap_err();
        assert!(matches!(error, StorageError::LocationNotFound(_)));

        test_db.cleanup().await.unwrap();
    }

    #[tokio::test]
    #[test_log::test]
    async fn test_update_location_success() {
        let test_db = TestDatabase::new().await.unwrap();
        let app_state = TestAppStateBuilder::new()
            .with_database(test_db.pool.clone())
            .build()
            .await
            .unwrap();

        // Create a location first
        let request = StorageLocationFixtures::create_location_request();
        let create_result = create_location(State(app_state.clone()), Json(request)).await;
        let created_location = create_result.unwrap().0.data.unwrap();

        // Update the location
        let update_request = StorageLocationFixtures::update_location_request();
        let new_name = update_request.name.as_ref().unwrap().clone();

        let result = update_location(
            State(app_state), 
            Path(created_location.id), 
            Json(update_request)
        ).await;

        assert!(result.is_ok(), "Update location should succeed");
        let response = result.unwrap();
        let api_response = response.0;

        TestAssertions::assert_api_response_success(&api_response);
        let updated_location = api_response.data.unwrap();

        assert_eq!(updated_location.id, created_location.id);
        assert_eq!(updated_location.name, new_name);
        assert_eq!(updated_location.temperature_zone, "-80C");
        assert_eq!(updated_location.max_capacity, 200);
        assert_eq!(updated_location.status, "maintenance");
        assert!(updated_location.updated_at > created_location.updated_at);

        test_db.cleanup().await.unwrap();
    }

    #[tokio::test]
    #[test_log::test]
    async fn test_delete_location_success() {
        let test_db = TestDatabase::new().await.unwrap();
        let app_state = TestAppStateBuilder::new()
            .with_database(test_db.pool.clone())
            .build()
            .await
            .unwrap();

        // Create a location first
        let request = StorageLocationFixtures::create_location_request();
        let create_result = create_location(State(app_state.clone()), Json(request)).await;
        let created_location = create_result.unwrap().0.data.unwrap();

        // Delete the location
        let result = delete_location(State(app_state.clone()), Path(created_location.id)).await;

        assert!(result.is_ok(), "Delete location should succeed");
        let response = result.unwrap();
        let api_response = response.0;

        TestAssertions::assert_api_response_success(&api_response);
        assert!(api_response.data.unwrap().contains("deleted successfully"));

        // Verify location is deleted
        let get_result = get_location(State(app_state), Path(created_location.id)).await;
        assert!(get_result.is_err(), "Getting deleted location should fail");

        test_db.cleanup().await.unwrap();
    }

    #[tokio::test]
    #[test_log::test]
    async fn test_delete_location_with_samples() {
        let test_db = TestDatabase::new().await.unwrap();
        let app_state = TestAppStateBuilder::new()
            .with_database(test_db.pool.clone())
            .build()
            .await
            .unwrap();

        // Create a location
        let location_request = StorageLocationFixtures::create_location_request();
        let create_result = create_location(State(app_state.clone()), Json(location_request)).await;
        let created_location = create_result.unwrap().0.data.unwrap();

        // Store a sample in the location
        let sample_request = SampleFixtures::store_sample_request(created_location.id);
        let store_result = store_sample(State(app_state.clone()), Json(sample_request)).await;
        assert!(store_result.is_ok(), "Sample storage should succeed");

        // Try to delete the location
        let result = delete_location(State(app_state), Path(created_location.id)).await;

        assert!(result.is_err(), "Delete location should fail when samples exist");
        let error = result.unwrap_err();
        assert!(matches!(error, StorageError::Validation(_)));

        test_db.cleanup().await.unwrap();
    }

    #[tokio::test]
    #[test_log::test]
    async fn test_get_capacity() {
        let test_db = TestDatabase::new().await.unwrap();
        let app_state = TestAppStateBuilder::new()
            .with_database(test_db.pool.clone())
            .build()
            .await
            .unwrap();

        // Create a location
        let location_request = StorageLocationFixtures::create_location_request();
        let create_result = create_location(State(app_state.clone()), Json(location_request)).await;
        let created_location = create_result.unwrap().0.data.unwrap();

        // Get capacity information
        let result = get_capacity(State(app_state), Path(created_location.id)).await;

        assert!(result.is_ok(), "Get capacity should succeed");
        let response = result.unwrap();
        let api_response = response.0;

        TestAssertions::assert_api_response_success(&api_response);
        let capacity_info = api_response.data.unwrap();

        assert_eq!(capacity_info.location_id, created_location.id);
        assert_eq!(capacity_info.max_capacity, created_location.max_capacity);
        assert_eq!(capacity_info.current_capacity, created_location.current_capacity);
        assert_eq!(capacity_info.utilization_percentage, 0.0);
        assert_eq!(capacity_info.available_capacity, created_location.max_capacity);
        assert_eq!(capacity_info.status, "normal");

        test_db.cleanup().await.unwrap();
    }

    #[tokio::test]
    #[test_log::test]
    async fn test_store_sample_success() {
        let test_db = TestDatabase::new().await.unwrap();
        let app_state = TestAppStateBuilder::new()
            .with_database(test_db.pool.clone())
            .build()
            .await
            .unwrap();

        // Create a location first
        let location_request = StorageLocationFixtures::create_location_request();
        let create_result = create_location(State(app_state.clone()), Json(location_request)).await;
        let created_location = create_result.unwrap().0.data.unwrap();

        // Store a sample
        let sample_request = SampleFixtures::store_sample_request(created_location.id);
        let sample_barcode = sample_request.barcode.clone();

        let result = store_sample(State(app_state), Json(sample_request)).await;

        assert!(result.is_ok(), "Store sample should succeed");
        let response = result.unwrap();
        let api_response = response.0;

        TestAssertions::assert_api_response_success(&api_response);
        let sample = api_response.data.unwrap();

        assert_eq!(sample.barcode, sample_barcode);
        assert_eq!(sample.sample_type, "blood");
        assert_eq!(sample.storage_location_id, Some(created_location.id));
        assert_eq!(sample.status, "stored");
        assert!(sample.stored_at.is_some());
        TestAssertions::assert_timestamp_recent(sample.created_at, 5);

        test_db.cleanup().await.unwrap();
    }

    #[tokio::test]
    #[test_log::test]
    async fn test_store_sample_capacity_exceeded() {
        let test_db = TestDatabase::new().await.unwrap();
        let app_state = TestAppStateBuilder::new()
            .with_database(test_db.pool.clone())
            .build()
            .await
            .unwrap();

        // Create a location with capacity 1
        let mut location_request = StorageLocationFixtures::create_location_request();
        location_request.max_capacity = 1;
        
        let create_result = create_location(State(app_state.clone()), Json(location_request)).await;
        let created_location = create_result.unwrap().0.data.unwrap();

        // Store first sample (should succeed)
        let sample_request1 = SampleFixtures::store_sample_request(created_location.id);
        let result1 = store_sample(State(app_state.clone()), Json(sample_request1)).await;
        assert!(result1.is_ok(), "First sample storage should succeed");

        // Try to store second sample (should fail due to capacity)
        let sample_request2 = SampleFixtures::store_sample_request(created_location.id);
        let result2 = store_sample(State(app_state), Json(sample_request2)).await;

        assert!(result2.is_err(), "Second sample storage should fail due to capacity");
        let error = result2.unwrap_err();
        assert!(matches!(error, StorageError::CapacityExceeded));

        test_db.cleanup().await.unwrap();
    }

    #[tokio::test]
    #[test_log::test]
    async fn test_store_sample_temperature_violation() {
        let test_db = TestDatabase::new().await.unwrap();
        let app_state = TestAppStateBuilder::new()
            .with_database(test_db.pool.clone())
            .build()
            .await
            .unwrap();

        // Create a -80C location
        let location_request = StorageLocationFixtures::create_location_request_with_zone("-80C");
        let create_result = create_location(State(app_state.clone()), Json(location_request)).await;
        let created_location = create_result.unwrap().0.data.unwrap();

        // Try to store sample with -20C requirements in -80C location
        let mut sample_request = SampleFixtures::store_sample_request(created_location.id);
        sample_request.temperature_requirements = Some("-20C".to_string());

        let result = store_sample(State(app_state), Json(sample_request)).await;

        assert!(result.is_err(), "Store sample should fail with temperature violation");
        let error = result.unwrap_err();
        assert!(matches!(error, StorageError::TemperatureViolation(_)));

        test_db.cleanup().await.unwrap();
    }

    #[tokio::test]
    #[test_log::test]
    async fn test_move_sample_success() {
        let test_db = TestDatabase::new().await.unwrap();
        let app_state = TestAppStateBuilder::new()
            .with_database(test_db.pool.clone())
            .build()
            .await
            .unwrap();

        // Create two locations
        let location1_request = StorageLocationFixtures::create_location_request();
        let location1_result = create_location(State(app_state.clone()), Json(location1_request)).await;
        let location1 = location1_result.unwrap().0.data.unwrap();

        let mut location2_request = StorageLocationFixtures::create_location_request();
        location2_request.name = "Target Location".to_string();
        let location2_result = create_location(State(app_state.clone()), Json(location2_request)).await;
        let location2 = location2_result.unwrap().0.data.unwrap();

        // Store sample in first location
        let sample_request = SampleFixtures::store_sample_request(location1.id);
        let store_result = store_sample(State(app_state.clone()), Json(sample_request)).await;
        let sample = store_result.unwrap().0.data.unwrap();

        // Move sample to second location
        let move_request = SampleFixtures::move_sample_request(location2.id);
        let result = move_sample(State(app_state), Path(sample.id), Json(move_request)).await;

        assert!(result.is_ok(), "Move sample should succeed");
        let response = result.unwrap();
        let api_response = response.0;

        TestAssertions::assert_api_response_success(&api_response);
        let moved_sample = api_response.data.unwrap();

        assert_eq!(moved_sample.id, sample.id);
        assert_eq!(moved_sample.storage_location_id, Some(location2.id));
        assert!(moved_sample.updated_at > sample.updated_at);

        // Verify chain of custody was updated
        let chain_of_custody = moved_sample.chain_of_custody.as_array().unwrap();
        assert!(chain_of_custody.len() >= 2);
        let latest_entry = &chain_of_custody[chain_of_custody.len() - 1];
        assert_eq!(latest_entry["action"], "moved");

        test_db.cleanup().await.unwrap();
    }

    #[tokio::test]
    #[test_log::test]
    async fn test_retrieve_sample_success() {
        let test_db = TestDatabase::new().await.unwrap();
        let app_state = TestAppStateBuilder::new()
            .with_database(test_db.pool.clone())
            .build()
            .await
            .unwrap();

        // Create a location and store a sample
        let location_request = StorageLocationFixtures::create_location_request();
        let location_result = create_location(State(app_state.clone()), Json(location_request)).await;
        let location = location_result.unwrap().0.data.unwrap();

        let sample_request = SampleFixtures::store_sample_request(location.id);
        let store_result = store_sample(State(app_state.clone()), Json(sample_request)).await;
        let sample = store_result.unwrap().0.data.unwrap();

        // Retrieve the sample
        let result = retrieve_sample(State(app_state), Path(sample.id)).await;

        assert!(result.is_ok(), "Retrieve sample should succeed");
        let response = result.unwrap();
        let api_response = response.0;

        TestAssertions::assert_api_response_success(&api_response);
        let retrieved_sample = api_response.data.unwrap();

        assert_eq!(retrieved_sample.id, sample.id);
        assert_eq!(retrieved_sample.status, "retrieved");
        assert!(retrieved_sample.storage_location_id.is_none());
        assert!(retrieved_sample.position.is_none());

        // Verify chain of custody was updated
        let chain_of_custody = retrieved_sample.chain_of_custody.as_array().unwrap();
        assert!(chain_of_custody.len() >= 2);
        let latest_entry = &chain_of_custody[chain_of_custody.len() - 1];
        assert_eq!(latest_entry["action"], "retrieved");

        test_db.cleanup().await.unwrap();
    }

    #[tokio::test]
    #[test_log::test]
    async fn test_retrieve_sample_not_found() {
        let test_db = TestDatabase::new().await.unwrap();
        let app_state = TestAppStateBuilder::new()
            .with_database(test_db.pool.clone())
            .build()
            .await
            .unwrap();

        let non_existent_id = TestDataFactory::uuid();

        let result = retrieve_sample(State(app_state), Path(non_existent_id)).await;

        assert!(result.is_err(), "Retrieve sample should fail for non-existent ID");
        let error = result.unwrap_err();
        assert!(matches!(error, StorageError::SampleNotFound(_)));

        test_db.cleanup().await.unwrap();
    }

    #[tokio::test]
    #[test_log::test]
    async fn test_get_sample_location() {
        let test_db = TestDatabase::new().await.unwrap();
        let app_state = TestAppStateBuilder::new()
            .with_database(test_db.pool.clone())
            .build()
            .await
            .unwrap();

        // Create a location and store a sample
        let location_request = StorageLocationFixtures::create_location_request();
        let location_result = create_location(State(app_state.clone()), Json(location_request)).await;
        let location = location_result.unwrap().0.data.unwrap();

        let sample_request = SampleFixtures::store_sample_request(location.id);
        let store_result = store_sample(State(app_state.clone()), Json(sample_request)).await;
        let sample = store_result.unwrap().0.data.unwrap();

        // Get sample location
        let result = get_sample_location(State(app_state), Path(sample.id)).await;

        assert!(result.is_ok(), "Get sample location should succeed");
        let response = result.unwrap();
        let api_response = response.0;

        TestAssertions::assert_api_response_success(&api_response);
        let sample_location = api_response.data.unwrap();

        assert!(sample_location.is_some());
        let location_data = sample_location.unwrap();
        assert_eq!(location_data.id, location.id);
        assert_eq!(location_data.name, location.name);

        test_db.cleanup().await.unwrap();
    }
} 
