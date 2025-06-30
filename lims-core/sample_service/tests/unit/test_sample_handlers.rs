use sample_service::{handlers::samples::*, models::*, test_utils::*};
use crate::test_utils::*;
use axum::{extract::{Path, Query}, Json};
use serial_test::serial;
use uuid::Uuid;

#[test_with_sample_db]
async fn test_create_sample_success(test_db: &mut TestDatabase) {
    let app_state = create_test_app_state().await;
    let request = SampleFactory::create_valid_sample_request();
    let sample_name = request.name.clone();

    let result = create_sample(axum::extract::State(app_state), Json(request)).await;

    assert!(result.is_ok());
    let response = result.unwrap();
    SampleAssertions::assert_successful_creation(&response.0);

    // Verify sample exists in database
    let sample = sqlx::query_as::<_, Sample>("SELECT * FROM samples WHERE name = $1")
        .bind(&sample_name)
        .fetch_one(&test_db.pool)
        .await
        .expect("Sample should exist");

    test_db.track_sample(sample.id);
    assert_eq!(sample.name, sample_name);
    assert_eq!(sample.status, SampleStatus::Pending);
}

#[test_with_sample_db]
async fn test_create_sample_validation_failure(test_db: &mut TestDatabase) {
    let app_state = create_test_app_state().await;
    let request = SampleFactory::create_invalid_sample_request();

    let result = create_sample(axum::extract::State(app_state), Json(request)).await;

    assert!(result.is_err());
    match result.unwrap_err() {
        SampleServiceError::Validation(_) => {}, // Expected
        other => panic!("Expected validation error, got: {:?}", other),
    }
}

#[test_with_sample_db]
async fn test_get_sample_success(test_db: &mut TestDatabase) {
    let app_state = create_test_app_state().await;
    
    // Create test sample
    let sample = SampleFactory::create_test_sample(&app_state.sample_service).await;
    test_db.track_sample(sample.id);

    let result = get_sample(axum::extract::State(app_state), Path(sample.id)).await;

    assert!(result.is_ok());
    let response = result.unwrap();
    SampleAssertions::assert_sample_data(&response.0, &sample.name);
}

#[test_with_sample_db]
async fn test_get_sample_not_found(test_db: &mut TestDatabase) {
    let app_state = create_test_app_state().await;
    let non_existent_id = Uuid::new_v4();

    let result = get_sample(axum::extract::State(app_state), Path(non_existent_id)).await;

    assert!(result.is_err());
    match result.unwrap_err() {
        SampleServiceError::NotFound(_) => {}, // Expected
        other => panic!("Expected NotFound error, got: {:?}", other),
    }
}

#[test_with_sample_db]
async fn test_get_sample_by_barcode_success(test_db: &mut TestDatabase) {
    let app_state = create_test_app_state().await;
    
    // Create test sample
    let sample = SampleFactory::create_test_sample(&app_state.sample_service).await;
    test_db.track_sample(sample.id);

    let result = get_sample_by_barcode(
        axum::extract::State(app_state), 
        Path(sample.barcode.clone())
    ).await;

    assert!(result.is_ok());
    let response = result.unwrap();
    SampleAssertions::assert_sample_data(&response.0, &sample.name);
}

#[test_with_sample_db]
async fn test_list_samples_success(test_db: &mut TestDatabase) {
    let app_state = create_test_app_state().await;
    
    // Create multiple test samples
    let samples = SampleFactory::create_test_samples(&app_state.sample_service, 3).await;
    for sample in &samples {
        test_db.track_sample(sample.id);
    }

    let query = ListSamplesQuery {
        limit: Some(10),
        offset: Some(0),
        status: None,
        sample_type: None,
        template_id: None,
        created_after: None,
        created_before: None,
        created_by: None,
        barcode_prefix: None,
        search: None,
        page: Some(1),
        page_size: Some(10),
        sort_by: Some("created_at".to_string()),
    };

    let result = list_samples(axum::extract::State(app_state), Query(query)).await;

    assert!(result.is_ok());
    let response = result.unwrap();
    SampleAssertions::assert_sample_list(&response.0, 3);
}

#[test_with_sample_db]
async fn test_update_sample_success(test_db: &mut TestDatabase) {
    let app_state = create_test_app_state().await;
    
    // Create test sample
    let sample = SampleFactory::create_test_sample(&app_state.sample_service).await;
    test_db.track_sample(sample.id);

    let update_request = SampleFactory::create_update_request();
    let new_name = update_request.name.clone().unwrap();

    let result = update_sample(
        axum::extract::State(app_state),
        Path(sample.id),
        Json(update_request),
    ).await;

    assert!(result.is_ok());
    let response = result.unwrap();
    assert_eq!(response.0["success"], true);
    assert_eq!(response.0["data"]["name"], new_name);
}

#[test_with_sample_db]
async fn test_update_sample_status_success(test_db: &mut TestDatabase) {
    let app_state = create_test_app_state().await;
    
    // Create test sample
    let sample = SampleFactory::create_test_sample(&app_state.sample_service).await;
    test_db.track_sample(sample.id);

    let status_request = UpdateSampleStatusRequest {
        new_status: SampleStatus::Validated,
        reason: Some("Automated testing".to_string()),
    };

    let result = update_status(
        axum::extract::State(app_state),
        Path(sample.id),
        Json(status_request),
    ).await;

    assert!(result.is_ok());
    let response = result.unwrap();
    assert_eq!(response.0["success"], true);
    assert_eq!(response.0["data"]["status"], "Validated");
}

#[test_with_sample_db]
async fn test_validate_sample_success(test_db: &mut TestDatabase) {
    let app_state = create_test_app_state().await;
    
    // Create test sample
    let sample = SampleFactory::create_test_sample(&app_state.sample_service).await;
    test_db.track_sample(sample.id);

    let result = validate_sample(axum::extract::State(app_state), Path(sample.id)).await;

    assert!(result.is_ok());
    let response = result.unwrap();
    assert_eq!(response.0["success"], true);
    assert!(response.0["data"]["is_valid"].is_boolean());
}

#[test_with_sample_db]
async fn test_delete_sample_success(test_db: &mut TestDatabase) {
    let app_state = create_test_app_state().await;
    
    // Create test sample
    let sample = SampleFactory::create_test_sample(&app_state.sample_service).await;
    test_db.track_sample(sample.id);

    let result = delete_sample(axum::extract::State(app_state), Path(sample.id)).await;

    assert!(result.is_ok());
    let response = result.unwrap();
    assert_eq!(response.0["success"], true);

    // Verify sample is soft deleted (if implemented) or actually deleted
    let deleted_sample = sqlx::query_as::<_, Sample>("SELECT * FROM samples WHERE id = $1")
        .bind(sample.id)
        .fetch_optional(&test_db.pool)
        .await
        .expect("Database query should succeed");

    // Either deleted or marked as deleted
    assert!(deleted_sample.is_none() || deleted_sample.unwrap().status == SampleStatus::Deleted);
}

#[test_with_sample_db]
async fn test_create_batch_samples_success(test_db: &mut TestDatabase) {
    let app_state = create_test_app_state().await;
    let batch_request = SampleFactory::create_batch_request(5);

    let result = create_batch_samples(axum::extract::State(app_state), Json(batch_request)).await;

    assert!(result.is_ok());
    let response = result.unwrap();
    SampleAssertions::assert_batch_response(&response.0, 5, 0);

    // Verify samples were created
    let created_samples = sqlx::query_as::<_, Sample>(
        "SELECT * FROM samples WHERE name LIKE 'Batch Sample%' ORDER BY created_at DESC LIMIT 5"
    )
    .fetch_all(&test_db.pool)
    .await
    .expect("Should find created samples");

    assert_eq!(created_samples.len(), 5);
    for sample in created_samples {
        test_db.track_sample(sample.id);
    }
}

#[test_with_sample_db]
async fn test_create_batch_samples_partial_failure(test_db: &mut TestDatabase) {
    let app_state = create_test_app_state().await;
    
    // Create batch with some invalid samples
    let mut batch_request = SampleFactory::create_batch_request(3);
    batch_request.samples[1] = SampleFactory::create_invalid_sample_request(); // Make one invalid

    let result = create_batch_samples(axum::extract::State(app_state), Json(batch_request)).await;

    assert!(result.is_ok());
    let response = result.unwrap();
    SampleAssertions::assert_batch_response(&response.0, 2, 1); // 2 success, 1 failure

    // Clean up successful samples
    let created_samples = sqlx::query_as::<_, Sample>(
        "SELECT * FROM samples WHERE name LIKE 'Batch Sample%' ORDER BY created_at DESC LIMIT 3"
    )
    .fetch_all(&test_db.pool)
    .await
    .expect("Should find created samples");

    for sample in created_samples {
        test_db.track_sample(sample.id);
    }
}

#[test_with_sample_db]
async fn test_validate_batch_success(test_db: &mut TestDatabase) {
    let app_state = create_test_app_state().await;
    
    // Create test samples
    let samples = SampleFactory::create_test_samples(&app_state.sample_service, 3).await;
    for sample in &samples {
        test_db.track_sample(sample.id);
    }

    let batch_validate_request = BatchValidateRequest {
        sample_ids: samples.iter().map(|s| s.id).collect(),
    };

    let result = validate_batch(
        axum::extract::State(app_state),
        Json(batch_validate_request),
    ).await;

    assert!(result.is_ok());
    let response = result.unwrap();
    assert_eq!(response.0["success"], true);
    assert_eq!(response.0["data"]["total_samples"], 3);
}

#[test_with_sample_db]
async fn test_get_sample_history_success(test_db: &mut TestDatabase) {
    let app_state = create_test_app_state().await;
    
    // Create test sample
    let sample = SampleFactory::create_test_sample(&app_state.sample_service).await;
    test_db.track_sample(sample.id);

    let query = SampleHistoryQuery {
        page: Some(1),
        page_size: Some(10),
    };

    let result = get_sample_history(
        axum::extract::State(app_state),
        Path(sample.id),
        Query(query),
    ).await;

    assert!(result.is_ok());
    let response = result.unwrap();
    assert_eq!(response.0["success"], true);
    assert_eq!(response.0["data"]["sample_id"], sample.id.to_string());
    assert!(response.0["data"]["status_history"].is_array());
    assert!(response.0["data"]["audit_log"].is_array());
}

#[test_with_sample_db]
async fn test_export_samples_csv(test_db: &mut TestDatabase) {
    let app_state = create_test_app_state().await;
    
    // Create test samples
    let samples = SampleFactory::create_test_samples(&app_state.sample_service, 3).await;
    for sample in &samples {
        test_db.track_sample(sample.id);
    }

    let query = ExportSamplesQuery {
        status: None,
        sample_type: None,
        template_id: None,
        created_after: None,
        created_before: None,
        search: None,
    };

    let result = export_samples(axum::extract::State(app_state), Query(query)).await;

    assert!(result.is_ok());
    let (headers, csv_content) = result.unwrap();
    
    // Verify CSV headers
    assert_eq!(headers[0].0, "Content-Type");
    assert_eq!(headers[0].1, "text/csv");
    
    // Verify CSV content
    CsvTestUtils::assert_csv_headers(&csv_content, &["ID", "Name", "Barcode", "Type", "Status"]);
    
    // Verify sample data is in CSV
    for sample in samples {
        CsvTestUtils::assert_csv_sample_data(&csv_content, &sample);
    }
}

#[test_with_sample_db]
async fn test_search_samples_success(test_db: &mut TestDatabase) {
    let app_state = create_test_app_state().await;
    
    // Create test samples with searchable names
    let mut sample1_req = SampleFactory::create_valid_sample_request();
    sample1_req.name = "Searchable DNA Sample".to_string();
    let sample1 = app_state.sample_service.create_sample(sample1_req).await.unwrap();
    test_db.track_sample(sample1.id);

    let search_request = SampleSearchRequest {
        query: Some("Searchable".to_string()),
        filters: SampleSearchFilters {
            status: None,
            sample_type: Some(SampleType::DNA),
            template_id: None,
            created_after: None,
            created_before: None,
            created_by: None,
            barcode_prefix: None,
        },
        page: Some(1),
        page_size: Some(10),
        sort_by: Some("created_at".to_string()),
    };

    let result = search_samples(axum::extract::State(app_state), Json(search_request)).await;

    assert!(result.is_ok());
    let response = result.unwrap();
    assert_eq!(response.0["success"], true);
    assert!(response.0["data"]["samples"].is_array());
    assert_eq!(response.0["search_query"], "Searchable");
}

#[test_with_sample_db]
async fn test_get_sample_statistics(test_db: &mut TestDatabase) {
    let app_state = create_test_app_state().await;
    
    // Create test samples with different types
    let samples = SampleFactory::create_test_samples(&app_state.sample_service, 5).await;
    for sample in &samples {
        test_db.track_sample(sample.id);
    }

    let query = StatisticsQuery {
        period_days: Some(30),
    };

    let result = get_sample_statistics(axum::extract::State(app_state), Query(query)).await;

    assert!(result.is_ok());
    let response = result.unwrap();
    assert_eq!(response.0["success"], true);
    assert!(response.0["data"]["overview"]["total_samples"].is_number());
    assert!(response.0["data"]["status_distribution"].is_object());
    assert!(response.0["data"]["type_distribution"].is_object());
    assert!(response.0["data"]["daily_trends"].is_array());
} 
