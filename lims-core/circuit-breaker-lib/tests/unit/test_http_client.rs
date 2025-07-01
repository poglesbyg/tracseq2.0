//! Unit tests for HTTP client with circuit breaker

use circuit_breaker_lib::{HttpClientWithCircuitBreaker, CircuitBreakerConfig, CircuitState};
use crate::test_utils::*;
use wiremock::MockServer;
use std::time::Duration;
use std::sync::Arc;

#[tokio::test]
async fn test_http_client_creation() {
    let client = HttpClientWithCircuitBreaker::new(
        "test-service".to_string(),
        Some(ConfigFactory::create_fast_config()),
    );
    
    let metrics = client.get_metrics().await;
    assert_eq!(metrics.service_name, "test-service");
    assert_eq!(metrics.state, CircuitState::Closed);
}

#[tokio::test]
async fn test_http_get_success() {
    let mock_server = MockServer::start().await;
    MockServerSetup::setup_success_endpoint(&mock_server).await;
    
    let client = HttpClientWithCircuitBreaker::new(
        "test-http-get".to_string(),
        Some(ConfigFactory::create_fast_config()),
    );
    
    let url = format!("{}/success", mock_server.uri());
    let result = client.get(&url).await;
    
    assert!(result.is_ok());
    let response = result.unwrap();
    assert_eq!(response.status(), 200);
    
    let body: serde_json::Value = response.json().await.unwrap();
    assert_eq!(body["status"], "ok");
}

#[tokio::test]
async fn test_http_get_failure() {
    let mock_server = MockServer::start().await;
    MockServerSetup::setup_failure_endpoint(&mock_server).await;
    
    let client = HttpClientWithCircuitBreaker::new(
        "test-http-failure".to_string(),
        Some(ConfigFactory::create_fast_config()),
    );
    
    let url = format!("{}/failure", mock_server.uri());
    
    // First failure
    let result1 = client.get(&url).await;
    assert!(result1.is_err());
    
    // Second failure should open circuit
    let result2 = client.get(&url).await;
    assert!(result2.is_err());
    
    let metrics = client.get_metrics().await;
    assert_eq!(metrics.state, CircuitState::Open);
}

#[tokio::test]
async fn test_http_timeout() {
    let mock_server = MockServer::start().await;
    MockServerSetup::setup_slow_endpoint(&mock_server, Duration::from_millis(200)).await;
    
    let config = CircuitBreakerConfig {
        failure_threshold: 2,
        recovery_timeout: Duration::from_millis(100),
        request_timeout: Duration::from_millis(50), // Short timeout
        max_concurrent_requests: 10,
        success_threshold: 1,
    };
    
    let client = HttpClientWithCircuitBreaker::new(
        "test-http-timeout".to_string(),
        Some(config),
    );
    
    let url = format!("{}/slow", mock_server.uri());
    let result = client.get(&url).await;
    
    assert!(result.is_err());
    if let Err(error) = result {
        CircuitBreakerAssertions::assert_error_contains(&error, "timed out");
    }
}

#[tokio::test]
async fn test_http_post() {
    let mock_server = MockServer::start().await;
    
    use wiremock::{Mock, ResponseTemplate};
    use wiremock::matchers::{method, path, body_json};
    
    Mock::given(method("POST"))
        .and(path("/api/data"))
        .and(body_json(serde_json::json!({"key": "value"})))
        .respond_with(ResponseTemplate::new(201)
            .set_body_json(serde_json::json!({"id": 123, "created": true})))
        .mount(&mock_server)
        .await;
    
    let client = HttpClientWithCircuitBreaker::new(
        "test-http-post".to_string(),
        Some(ConfigFactory::create_fast_config()),
    );
    
    let url = format!("{}/api/data", mock_server.uri());
    let payload = serde_json::json!({"key": "value"});
    let result = client.post(&url, &payload).await;
    
    assert!(result.is_ok());
    let response = result.unwrap();
    assert_eq!(response.status(), 201);
    
    let body: serde_json::Value = response.json().await.unwrap();
    assert_eq!(body["id"], 123);
    assert_eq!(body["created"], true);
}

#[tokio::test]
async fn test_http_put() {
    let mock_server = MockServer::start().await;
    
    use wiremock::{Mock, ResponseTemplate};
    use wiremock::matchers::{method, path};
    
    Mock::given(method("PUT"))
        .and(path("/api/data/123"))
        .respond_with(ResponseTemplate::new(200)
            .set_body_json(serde_json::json!({"updated": true})))
        .mount(&mock_server)
        .await;
    
    let client = HttpClientWithCircuitBreaker::new(
        "test-http-put".to_string(),
        Some(ConfigFactory::create_fast_config()),
    );
    
    let url = format!("{}/api/data/123", mock_server.uri());
    let payload = serde_json::json!({"key": "updated_value"});
    let result = client.put(&url, &payload).await;
    
    assert!(result.is_ok());
    let response = result.unwrap();
    assert_eq!(response.status(), 200);
}

#[tokio::test]
async fn test_http_delete() {
    let mock_server = MockServer::start().await;
    
    use wiremock::{Mock, ResponseTemplate};
    use wiremock::matchers::{method, path};
    
    Mock::given(method("DELETE"))
        .and(path("/api/data/123"))
        .respond_with(ResponseTemplate::new(204))
        .mount(&mock_server)
        .await;
    
    let client = HttpClientWithCircuitBreaker::new(
        "test-http-delete".to_string(),
        Some(ConfigFactory::create_fast_config()),
    );
    
    let url = format!("{}/api/data/123", mock_server.uri());
    let result = client.delete(&url).await;
    
    assert!(result.is_ok());
    let response = result.unwrap();
    assert_eq!(response.status(), 204);
}

#[tokio::test]
async fn test_http_client_reset() {
    let mock_server = MockServer::start().await;
    MockServerSetup::setup_failure_endpoint(&mock_server).await;
    
    let client = HttpClientWithCircuitBreaker::new(
        "test-http-reset".to_string(),
        Some(ConfigFactory::create_fast_config()),
    );
    
    let url = format!("{}/failure", mock_server.uri());
    
    // Open the circuit
    for _ in 0..2 {
        let _ = client.get(&url).await;
    }
    
    let metrics = client.get_metrics().await;
    assert_eq!(metrics.state, CircuitState::Open);
    
    // Reset
    client.reset().await;
    
    let metrics = client.get_metrics().await;
    assert_eq!(metrics.state, CircuitState::Closed);
    assert_eq!(metrics.failure_count, 0);
}

#[tokio::test]
async fn test_http_flaky_endpoint() {
    let mock_server = MockServer::start().await;
    MockServerSetup::setup_flaky_endpoint(&mock_server).await;
    
    let client = HttpClientWithCircuitBreaker::new(
        "test-http-flaky".to_string(),
        Some(ConfigFactory::create_lenient_config()),
    );
    
    let url = format!("{}/flaky", mock_server.uri());
    let mut successes = 0;
    let mut failures = 0;
    
    // Make several requests
    for _ in 0..10 {
        match client.get(&url).await {
            Ok(_) => successes += 1,
            Err(_) => failures += 1,
        }
    }
    
    // Should have some of each
    assert!(successes > 0);
    assert!(failures > 0);
}

#[tokio::test]
async fn test_http_concurrent_requests() {
    let mock_server = MockServer::start().await;
    MockServerSetup::setup_success_endpoint(&mock_server).await;
    
    let client = Arc::new(HttpClientWithCircuitBreaker::new(
        "test-http-concurrent".to_string(),
        Some(ConfigFactory::create_fast_config()),
    ));
    
    let url = format!("{}/success", mock_server.uri());
    let mut handles = Vec::new();
    
    // Make concurrent requests
    for _ in 0..20 {
        let client_clone = client.clone();
        let url_clone = url.clone();
        let handle = tokio::spawn(async move {
            client_clone.get(&url_clone).await
        });
        handles.push(handle);
    }
    
    let results: Vec<_> = futures::future::join_all(handles).await;
    let successes = results.iter().filter(|r| r.is_ok()).count();
    
    assert!(successes > 0);
}

#[tokio::test]
async fn test_http_client_with_different_endpoints() {
    let mock_server = MockServer::start().await;
    MockServerSetup::setup_success_endpoint(&mock_server).await;
    MockServerSetup::setup_failure_endpoint(&mock_server).await;
    
    let client = HttpClientWithCircuitBreaker::new(
        "test-multi-endpoint".to_string(),
        Some(ConfigFactory::create_lenient_config()),
    );
    
    // Success endpoint
    let success_url = format!("{}/success", mock_server.uri());
    let success_result = client.get(&success_url).await;
    assert!(success_result.is_ok());
    
    // Failure endpoint
    let failure_url = format!("{}/failure", mock_server.uri());
    let failure_result = client.get(&failure_url).await;
    assert!(failure_result.is_err());
    
    // Circuit should still be functional with lenient config
    let metrics = client.get_metrics().await;
    assert_ne!(metrics.state, CircuitState::Open);
}