//! Unit tests for service aggregation functionality

use dashboard_service::{AppState, DashboardData};
use crate::test_utils::*;
use wiremock::matchers::{method, path};
use wiremock::{Mock, ResponseTemplate};

test_with_mock_services!(
    test_aggregate_all_services,
    |app_state: &AppState, mock_server: &MockServer| async move {
        // Set up all service mocks
        MockServerSetup::setup_healthy_services(mock_server).await;
        MockServerSetup::setup_sample_service_mocks(mock_server).await;
        MockServerSetup::setup_storage_service_mocks(mock_server).await;
        MockServerSetup::setup_sequencing_service_mocks(mock_server).await;
        
        // Simulate aggregation (would be done by actual service logic)
        let sample_response = app_state.http_client
            .get(format!("{}/sample/metrics", app_state.settings.service_urls.sample_service))
            .send()
            .await
            .unwrap();
        
        assert_eq!(sample_response.status(), 200);
        
        let sample_data: serde_json::Value = sample_response.json().await.unwrap();
        assert_eq!(sample_data["total_samples"], 1234);
    }
);

test_with_mock_services!(
    test_partial_service_failure,
    |app_state: &AppState, mock_server: &MockServer| async move {
        // Set up some services as healthy
        MockServerSetup::setup_sample_service_mocks(mock_server).await;
        
        // Set up some services as failing
        Mock::given(method("GET"))
            .and(path("/storage/metrics"))
            .respond_with(ResponseTemplate::new(500))
            .mount(mock_server)
            .await;
        
        // Test aggregation handles partial failures
        let sample_response = app_state.http_client
            .get(format!("{}/sample/metrics", app_state.settings.service_urls.sample_service))
            .send()
            .await
            .unwrap();
        
        let storage_response = app_state.http_client
            .get(format!("{}/storage/metrics", app_state.settings.service_urls.storage_service))
            .send()
            .await
            .unwrap();
        
        assert_eq!(sample_response.status(), 200);
        assert_eq!(storage_response.status(), 500);
    }
);

test_with_mock_services!(
    test_service_timeout_handling,
    |app_state: &AppState, mock_server: &MockServer| async move {
        // Set up a service that delays response
        Mock::given(method("GET"))
            .and(path("/slow/metrics"))
            .respond_with(ResponseTemplate::new(200)
                .set_delay(std::time::Duration::from_secs(10)))
            .mount(mock_server)
            .await;
        
        // HTTP client should timeout before the delay
        let result = app_state.http_client
            .get(format!("{}/slow/metrics", mock_server.uri()))
            .timeout(std::time::Duration::from_secs(1))
            .send()
            .await;
        
        assert!(result.is_err());
    }
);

test_with_mock_services!(
    test_concurrent_service_calls,
    |app_state: &AppState, mock_server: &MockServer| async move {
        // Set up all service mocks
        MockServerSetup::setup_sample_service_mocks(mock_server).await;
        MockServerSetup::setup_storage_service_mocks(mock_server).await;
        MockServerSetup::setup_sequencing_service_mocks(mock_server).await;
        
        // Make concurrent calls
        let sample_future = app_state.http_client
            .get(format!("{}/sample/metrics", app_state.settings.service_urls.sample_service))
            .send();
            
        let storage_future = app_state.http_client
            .get(format!("{}/storage/metrics", app_state.settings.service_urls.storage_service))
            .send();
            
        let sequencing_future = app_state.http_client
            .get(format!("{}/sequencing/metrics", app_state.settings.service_urls.sequencing_service))
            .send();
        
        // Wait for all futures
        let (sample_result, storage_result, sequencing_result) = 
            tokio::join!(sample_future, storage_future, sequencing_future);
        
        assert!(sample_result.is_ok());
        assert!(storage_result.is_ok());
        assert!(sequencing_result.is_ok());
        
        assert_eq!(sample_result.unwrap().status(), 200);
        assert_eq!(storage_result.unwrap().status(), 200);
        assert_eq!(sequencing_result.unwrap().status(), 200);
    }
);

test_with_mock_services!(
    test_service_response_parsing,
    |app_state: &AppState, mock_server: &MockServer| async move {
        // Set up mock with specific response
        Mock::given(method("GET"))
            .and(path("/custom/metrics"))
            .respond_with(ResponseTemplate::new(200)
                .set_body_json(serde_json::json!({
                    "metrics": {
                        "custom_value": 42,
                        "nested": {
                            "data": [1, 2, 3]
                        }
                    }
                })))
            .mount(mock_server)
            .await;
        
        let response = app_state.http_client
            .get(format!("{}/custom/metrics", mock_server.uri()))
            .send()
            .await
            .unwrap();
        
        let data: serde_json::Value = response.json().await.unwrap();
        assert_eq!(data["metrics"]["custom_value"], 42);
        assert_eq!(data["metrics"]["nested"]["data"][0], 1);
    }
);

test_with_mock_services!(
    test_empty_response_handling,
    |app_state: &AppState, mock_server: &MockServer| async move {
        // Set up service returning empty response
        Mock::given(method("GET"))
            .and(path("/empty/metrics"))
            .respond_with(ResponseTemplate::new(200)
                .set_body_json(serde_json::json!({})))
            .mount(mock_server)
            .await;
        
        let response = app_state.http_client
            .get(format!("{}/empty/metrics", mock_server.uri()))
            .send()
            .await
            .unwrap();
        
        let data: serde_json::Value = response.json().await.unwrap();
        assert!(data.is_object());
        assert_eq!(data.as_object().unwrap().len(), 0);
    }
);

test_with_mock_services!(
    test_malformed_response_handling,
    |app_state: &AppState, mock_server: &MockServer| async move {
        // Set up service returning malformed JSON
        Mock::given(method("GET"))
            .and(path("/malformed/metrics"))
            .respond_with(ResponseTemplate::new(200)
                .set_body_string("not valid json"))
            .mount(mock_server)
            .await;
        
        let response = app_state.http_client
            .get(format!("{}/malformed/metrics", mock_server.uri()))
            .send()
            .await
            .unwrap();
        
        // Attempting to parse as JSON should fail
        let json_result = response.json::<serde_json::Value>().await;
        assert!(json_result.is_err());
    }
);

test_with_mock_services!(
    test_service_aggregation_performance,
    |app_state: &AppState, mock_server: &MockServer| async move {
        // Set up fast responding services
        MockServerSetup::setup_sample_service_mocks(mock_server).await;
        MockServerSetup::setup_storage_service_mocks(mock_server).await;
        MockServerSetup::setup_sequencing_service_mocks(mock_server).await;
        
        let start = std::time::Instant::now();
        
        // Simulate parallel aggregation
        let futures = vec![
            app_state.http_client.get(format!("{}/sample/metrics", app_state.settings.service_urls.sample_service)).send(),
            app_state.http_client.get(format!("{}/storage/metrics", app_state.settings.service_urls.storage_service)).send(),
            app_state.http_client.get(format!("{}/sequencing/metrics", app_state.settings.service_urls.sequencing_service)).send(),
        ];
        
        let results = futures::future::join_all(futures).await;
        let duration = start.elapsed();
        
        // All should succeed
        assert!(results.iter().all(|r| r.is_ok()));
        
        // Parallel calls should be fast
        assert!(
            duration.as_millis() < 500,
            "Service aggregation took {:?}, which is too slow",
            duration
        );
    }
);

test_with_mock_services!(
    test_service_retry_logic,
    |app_state: &AppState, mock_server: &MockServer| async move {
        let mut call_count = 0;
        
        // First call fails, subsequent calls succeed
        Mock::given(method("GET"))
            .and(path("/flaky/metrics"))
            .respond_with(move |_| {
                call_count += 1;
                if call_count == 1 {
                    ResponseTemplate::new(500)
                } else {
                    ResponseTemplate::new(200)
                        .set_body_json(serde_json::json!({"status": "ok"}))
                }
            })
            .mount(mock_server)
            .await;
        
        // First attempt
        let first_response = app_state.http_client
            .get(format!("{}/flaky/metrics", mock_server.uri()))
            .send()
            .await
            .unwrap();
        
        assert_eq!(first_response.status(), 500);
        
        // Retry should succeed
        let retry_response = app_state.http_client
            .get(format!("{}/flaky/metrics", mock_server.uri()))
            .send()
            .await
            .unwrap();
        
        assert_eq!(retry_response.status(), 200);
    }
);

#[tokio::test]
async fn test_aggregate_data_structure() {
    let sample_data = MockResponses::sample_metrics();
    let storage_data = MockResponses::storage_metrics();
    let sequencing_data = MockResponses::sequencing_metrics();
    
    // Simulate aggregation
    let aggregated = serde_json::json!({
        "sample_metrics": sample_data,
        "storage_metrics": storage_data,
        "sequencing_metrics": sequencing_data,
        "timestamp": chrono::Utc::now(),
    });
    
    DashboardAssertions::assert_service_aggregation_complete(&aggregated);
}

test_with_mock_services!(
    test_cache_with_aggregation,
    |app_state: &AppState, mock_server: &MockServer| async move {
        MockServerSetup::setup_sample_service_mocks(mock_server).await;
        
        let cache_key = "aggregated_data";
        
        // First call - should hit service
        let response = app_state.http_client
            .get(format!("{}/sample/metrics", app_state.settings.service_urls.sample_service))
            .send()
            .await
            .unwrap();
        
        let data: serde_json::Value = response.json().await.unwrap();
        
        // Cache the result
        let dashboard_data = DashboardData {
            timestamp: chrono::Utc::now(),
            data: data.clone(),
            ttl_seconds: 300,
        };
        app_state.cache.insert(cache_key.to_string(), dashboard_data).await;
        
        // Second call - should use cache
        let cached = app_state.cache.get(&cache_key.to_string()).await;
        assert!(cached.is_some());
        assert_eq!(cached.unwrap().data, data);
    }
);