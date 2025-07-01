//! Integration tests for fault tolerance scenarios

use circuit_breaker_lib::{CircuitBreaker, CircuitBreakerConfig, CircuitBreakerRegistry, CircuitState, HttpClientWithCircuitBreaker};
use crate::test_utils::*;
use std::time::Duration;
use wiremock::MockServer;

#[tokio::test]
async fn test_cascading_failures() {
    let registry = CircuitBreakerRegistry::new();
    
    // Register multiple dependent services
    registry.register("database".to_string(), ConfigFactory::create_fast_config()).await;
    registry.register("cache".to_string(), ConfigFactory::create_fast_config()).await;
    registry.register("api".to_string(), ConfigFactory::create_fast_config()).await;
    
    // Simulate database failure
    if let Some(db_breaker) = registry.get("database").await {
        for _ in 0..2 {
            let _ = db_breaker.call(TestOperations::always_fails::<i32>).await;
        }
        assert_eq!(db_breaker.get_state().await, CircuitState::Open);
    }
    
    // API should detect database is down and fail fast
    if let Some(api_breaker) = registry.get("api").await {
        if let Some(db_breaker) = registry.get("database").await {
            let result = api_breaker.call(async move {
                // Check if database is available
                if db_breaker.get_state().await == CircuitState::Open {
                    Err::<i32, std::io::Error>(std::io::Error::new(
                        std::io::ErrorKind::Other,
                        "Database unavailable"
                    ))
                } else {
                    Ok(42)
                }
            }).await;
            
            assert!(result.is_err());
        }
    }
}

#[tokio::test]
async fn test_progressive_recovery() {
    let breaker = create_test_breaker("recovery_test", ConfigFactory::create_fast_config());
    
    // Open the circuit
    for _ in 0..2 {
        let _ = breaker.call(TestOperations::always_fails::<i32>).await;
    }
    assert_eq!(breaker.get_state().await, CircuitState::Open);
    
    // Wait for recovery timeout
    tokio::time::sleep(Duration::from_millis(150)).await;
    
    // First success moves to half-open
    let result = breaker.call(async { TestOperations::always_succeeds(1).await }).await;
    assert!(result.is_ok());
    
    // After success threshold (1), should be closed
    assert_eq!(breaker.get_state().await, CircuitState::Closed);
    
    // Verify normal operation resumed
    for i in 0..5 {
        let result = breaker.call(async move { TestOperations::always_succeeds(i).await }).await;
        assert!(result.is_ok());
    }
}

#[tokio::test]
async fn test_partial_system_degradation() {
    let registry = CircuitBreakerRegistry::new();
    
    // Register services with different criticality
    registry.register("critical-service".to_string(), ConfigFactory::create_strict_config()).await;
    registry.register("optional-service".to_string(), ConfigFactory::create_lenient_config()).await;
    
    // Fail optional service
    if let Some(optional) = registry.get("optional-service").await {
        for _ in 0..5 {
            let _ = optional.call(TestOperations::always_fails::<i32>).await;
        }
    }
    
    // Critical service should still work
    if let Some(critical) = registry.get("critical-service").await {
        let result = critical.call(async { TestOperations::always_succeeds("critical data").await }).await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "critical data");
    }
    
    // System should handle partial degradation
    let metrics = registry.get_all_metrics().await;
    let open_count = metrics.iter().filter(|m| m.state == CircuitState::Open).count();
    let closed_count = metrics.iter().filter(|m| m.state == CircuitState::Closed).count();
    
    assert!(open_count > 0);
    assert!(closed_count > 0);
}

#[tokio::test]
async fn test_bulkhead_pattern() {
    let config = CircuitBreakerConfig {
        failure_threshold: 5,
        recovery_timeout: Duration::from_millis(200),
        request_timeout: Duration::from_millis(100),
        max_concurrent_requests: 3, // Small bulkhead
        success_threshold: 2,
    };
    
    let breaker = create_test_breaker("bulkhead_test", config);
    let counter = CallCounter::new();
    
    // Try to run 10 concurrent slow operations
    let mut handles = Vec::new();
    for i in 0..10 {
        let breaker_clone = breaker.clone();
        let counter_clone = counter.clone();
        let handle = tokio::spawn(async move {
            let result = breaker_clone.call(async move {
                counter_clone.increment().await;
                TestOperations::slow_operation(Duration::from_millis(50), i).await
            }).await;
            result.is_ok()
        });
        handles.push(handle);
    }
    
    // Wait a bit for operations to start
    tokio::time::sleep(Duration::from_millis(10)).await;
    
    // Check concurrent count - should be limited by bulkhead
    let concurrent = counter.get().await;
    assert!(concurrent <= 3, "Too many concurrent operations: {}", concurrent);
    
    // Wait for all to complete
    let results: Vec<bool> = futures::future::join_all(handles)
        .await
        .into_iter()
        .filter_map(Result::ok)
        .collect();
    
    // Some should succeed (within bulkhead), some should fail
    let successes = results.iter().filter(|&&r| r).count();
    let failures = results.iter().filter(|&&r| !r).count();
    
    assert!(successes > 0);
    assert!(failures > 0);
}

#[tokio::test]
async fn test_real_world_http_scenario() {
    let mock_server = MockServer::start().await;
    
    // Setup endpoints simulating real service behavior
    MockServerSetup::setup_success_endpoint(&mock_server).await;
    MockServerSetup::setup_failure_endpoint(&mock_server).await;
    MockServerSetup::setup_slow_endpoint(&mock_server, Duration::from_millis(150)).await;
    
    let client = HttpClientWithCircuitBreaker::new(
        "real-world-service".to_string(),
        Some(ConfigFactory::create_fast_config()),
    );
    
    // Normal operation
    let url = format!("{}/success", mock_server.uri());
    let result = client.get(&url).await;
    assert!(result.is_ok());
    
    // Some failures
    let failure_url = format!("{}/failure", mock_server.uri());
    let _ = client.get(&failure_url).await;
    let _ = client.get(&failure_url).await;
    
    // Circuit should be open
    let metrics = client.get_metrics().await;
    assert_eq!(metrics.state, CircuitState::Open);
    
    // Success endpoint should also be rejected now
    let result = client.get(&url).await;
    assert!(result.is_err());
    
    // Wait for recovery
    tokio::time::sleep(Duration::from_millis(150)).await;
    
    // Should recover
    let result = client.get(&url).await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_adaptive_timeout_strategy() {
    let breaker = create_test_breaker("adaptive_test", ConfigFactory::create_fast_config());
    
    // Mix of fast and slow operations
    let results = TestScenarioBuilder::new()
        .add_success(1)  // Fast
        .add_success(2)  // Fast
        .execute_with_breaker(&breaker).await;
    
    assert!(results.iter().all(|r| r.is_ok()));
    
    // Now slow operations that timeout
    let slow_result = breaker.call(
        TestOperations::slow_operation(Duration::from_millis(100), 3)
    ).await;
    assert!(slow_result.is_err());
    
    // Circuit should still be functional
    assert_ne!(breaker.get_state().await, CircuitState::Open);
}

#[tokio::test]
async fn test_graceful_degradation_with_fallback() {
    let primary = create_test_breaker("primary", ConfigFactory::create_strict_config());
    let fallback = create_test_breaker("fallback", ConfigFactory::create_lenient_config());
    
    // Primary fails
    let _ = primary.call(TestOperations::always_fails::<String>).await;
    assert_eq!(primary.get_state().await, CircuitState::Open);
    
    // Use fallback
    let result = if primary.get_state().await == CircuitState::Open {
        fallback.call(async { 
            TestOperations::always_succeeds("fallback response".to_string()).await 
        }).await
    } else {
        primary.call(async { 
            TestOperations::always_succeeds("primary response".to_string()).await 
        }).await
    };
    
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), "fallback response");
}

#[tokio::test]
async fn test_monitoring_and_alerting_integration() {
    let registry = CircuitBreakerRegistry::new();
    
    // Register services
    for i in 0..5 {
        registry.register(
            format!("service-{}", i),
            ConfigFactory::create_fast_config(),
        ).await;
    }
    
    // Simulate various states
    if let Some(service0) = registry.get("service-0").await {
        let _ = service0.call(async { TestOperations::always_succeeds(0).await }).await;
    }
    
    if let Some(service1) = registry.get("service-1").await {
        for _ in 0..2 {
            let _ = service1.call(TestOperations::always_fails::<i32>).await;
        }
    }
    
    if let Some(service2) = registry.get("service-2").await {
        service2.force_open().await;
    }
    
    // Collect monitoring data
    let metrics = registry.get_all_metrics().await;
    
    // Alert on open circuits
    let open_services: Vec<&str> = metrics.iter()
        .filter(|m| m.state == CircuitState::Open)
        .map(|m| m.service_name.as_str())
        .collect();
    
    assert!(!open_services.is_empty());
    assert!(open_services.contains(&"service-1"));
    assert!(open_services.contains(&"service-2"));
    
    // Check failure rates
    let high_failure_services: Vec<&str> = metrics.iter()
        .filter(|m| m.failure_count > 0)
        .map(|m| m.service_name.as_str())
        .collect();
    
    assert!(high_failure_services.contains(&"service-1"));
}