//! Unit tests for circuit breaker registry

use circuit_breaker_lib::{CircuitBreakerRegistry, CircuitState};
use crate::test_utils::*;
use std::sync::Arc;

#[tokio::test]
async fn test_registry_creation() {
    let registry = CircuitBreakerRegistry::new();
    let metrics = registry.get_all_metrics().await;
    assert!(metrics.is_empty());
}

#[tokio::test]
async fn test_registry_register_and_get() {
    let registry = CircuitBreakerRegistry::new();
    
    // Register a circuit breaker
    registry.register(
        "service-1".to_string(),
        ConfigFactory::create_fast_config(),
    ).await;
    
    // Get the registered breaker
    let breaker = registry.get("service-1").await;
    assert!(breaker.is_some());
    
    let breaker = breaker.unwrap();
    assert_eq!(breaker.get_state().await, CircuitState::Closed);
}

#[tokio::test]
async fn test_registry_multiple_services() {
    let registry = CircuitBreakerRegistry::new();
    
    // Register multiple services
    registry.register("service-1".to_string(), ConfigFactory::create_fast_config()).await;
    registry.register("service-2".to_string(), ConfigFactory::create_strict_config()).await;
    registry.register("service-3".to_string(), ConfigFactory::create_lenient_config()).await;
    
    // Get all metrics
    let metrics = registry.get_all_metrics().await;
    assert_eq!(metrics.len(), 3);
    
    // Verify service names
    let service_names: Vec<String> = metrics.iter()
        .map(|m| m.service_name.clone())
        .collect();
    assert!(service_names.contains(&"service-1".to_string()));
    assert!(service_names.contains(&"service-2".to_string()));
    assert!(service_names.contains(&"service-3".to_string()));
}

#[tokio::test]
async fn test_registry_get_nonexistent() {
    let registry = CircuitBreakerRegistry::new();
    
    let breaker = registry.get("nonexistent").await;
    assert!(breaker.is_none());
}

#[tokio::test]
async fn test_registry_reset_all() {
    let registry = CircuitBreakerRegistry::new();
    
    // Register services
    registry.register("service-1".to_string(), ConfigFactory::create_fast_config()).await;
    registry.register("service-2".to_string(), ConfigFactory::create_fast_config()).await;
    
    // Open the circuits
    if let Some(breaker1) = registry.get("service-1").await {
        breaker1.force_open().await;
    }
    if let Some(breaker2) = registry.get("service-2").await {
        breaker2.force_open().await;
    }
    
    // Verify they are open
    let metrics = registry.get_all_metrics().await;
    assert!(metrics.iter().all(|m| m.state == CircuitState::Open));
    
    // Reset all
    registry.reset_all().await;
    
    // Verify they are closed
    let metrics = registry.get_all_metrics().await;
    assert!(metrics.iter().all(|m| m.state == CircuitState::Closed));
}

#[tokio::test]
async fn test_registry_concurrent_access() {
    let registry = Arc::new(CircuitBreakerRegistry::new());
    
    // Register a service
    registry.register("shared-service".to_string(), ConfigFactory::create_fast_config()).await;
    
    let mut handles = Vec::new();
    
    // Spawn multiple tasks accessing the same breaker
    for i in 0..10 {
        let registry_clone = registry.clone();
        let handle = tokio::spawn(async move {
            if let Some(breaker) = registry_clone.get("shared-service").await {
                // Perform an operation
                let result = breaker.call(async move {
                    TestOperations::always_succeeds(i).await
                }).await;
                result.is_ok()
            } else {
                false
            }
        });
        handles.push(handle);
    }
    
    let results: Vec<bool> = futures::future::join_all(handles)
        .await
        .into_iter()
        .filter_map(Result::ok)
        .collect();
    
    // All should succeed
    assert!(results.iter().all(|&r| r));
    assert_eq!(results.len(), 10);
}

#[tokio::test]
async fn test_registry_overwrite_service() {
    let registry = CircuitBreakerRegistry::new();
    
    // Register a service
    registry.register("service".to_string(), ConfigFactory::create_fast_config()).await;
    
    // Open the circuit
    if let Some(breaker) = registry.get("service").await {
        breaker.force_open().await;
        assert_eq!(breaker.get_state().await, CircuitState::Open);
    }
    
    // Register again with different config (overwrites)
    registry.register("service".to_string(), ConfigFactory::create_strict_config()).await;
    
    // New breaker should be closed
    if let Some(breaker) = registry.get("service").await {
        assert_eq!(breaker.get_state().await, CircuitState::Closed);
        
        // Verify it has the new config
        let metrics = breaker.get_metrics().await;
        assert_eq!(metrics.config.failure_threshold, 1); // Strict config has threshold 1
    }
}

#[tokio::test]
async fn test_registry_metrics_aggregation() {
    let registry = CircuitBreakerRegistry::new();
    
    // Register multiple services
    registry.register("service-1".to_string(), ConfigFactory::create_fast_config()).await;
    registry.register("service-2".to_string(), ConfigFactory::create_fast_config()).await;
    registry.register("service-3".to_string(), ConfigFactory::create_fast_config()).await;
    
    // Create different states
    if let Some(breaker1) = registry.get("service-1").await {
        // Keep closed
        let _ = breaker1.call(async { TestOperations::always_succeeds(1).await }).await;
    }
    
    if let Some(breaker2) = registry.get("service-2").await {
        // Open it
        breaker2.force_open().await;
    }
    
    if let Some(breaker3) = registry.get("service-3").await {
        // Generate some failures
        let _ = breaker3.call(TestOperations::always_fails::<i32>).await;
    }
    
    // Get all metrics
    let metrics = registry.get_all_metrics().await;
    
    // Verify different states
    let states: Vec<CircuitState> = metrics.iter().map(|m| m.state.clone()).collect();
    assert!(states.contains(&CircuitState::Closed));
    assert!(states.contains(&CircuitState::Open));
    
    // Verify different counts
    let total_failures: u64 = metrics.iter().map(|m| m.failure_count).sum();
    let total_successes: u64 = metrics.iter().map(|m| m.success_count).sum();
    
    assert!(total_failures > 0);
    assert!(total_successes > 0);
}

#[tokio::test]
async fn test_registry_isolation() {
    let registry = CircuitBreakerRegistry::new();
    
    // Register two services
    registry.register("isolated-1".to_string(), ConfigFactory::create_fast_config()).await;
    registry.register("isolated-2".to_string(), ConfigFactory::create_fast_config()).await;
    
    // Fail one service
    if let Some(breaker1) = registry.get("isolated-1").await {
        for _ in 0..2 {
            let _ = breaker1.call(TestOperations::always_fails::<i32>).await;
        }
        assert_eq!(breaker1.get_state().await, CircuitState::Open);
    }
    
    // Other service should be unaffected
    if let Some(breaker2) = registry.get("isolated-2").await {
        assert_eq!(breaker2.get_state().await, CircuitState::Closed);
        
        // Should still work
        let result = breaker2.call(async { TestOperations::always_succeeds(42).await }).await;
        assert!(result.is_ok());
    }
}

#[tokio::test]
async fn test_registry_performance() {
    let registry = Arc::new(CircuitBreakerRegistry::new());
    
    // Register many services
    for i in 0..100 {
        registry.register(
            format!("service-{}", i),
            ConfigFactory::create_fast_config(),
        ).await;
    }
    
    let start = std::time::Instant::now();
    
    // Get all metrics
    let metrics = registry.get_all_metrics().await;
    
    let duration = start.elapsed();
    
    assert_eq!(metrics.len(), 100);
    assert!(
        duration.as_millis() < 100,
        "Getting metrics for 100 services took {:?}, which is too slow",
        duration
    );
}