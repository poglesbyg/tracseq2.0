//! Integration tests for concurrent operations

use circuit_breaker_lib::{CircuitBreaker, CircuitBreakerConfig, CircuitBreakerRegistry, CircuitState};
use crate::test_utils::*;
use std::time::Duration;
use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};

#[tokio::test]
async fn test_high_concurrency_stress() {
    let config = CircuitBreakerConfig {
        failure_threshold: 10,
        recovery_timeout: Duration::from_millis(500),
        request_timeout: Duration::from_millis(100),
        max_concurrent_requests: 50,
        success_threshold: 5,
    };
    
    let breaker = Arc::new(create_test_breaker("stress_test", config));
    let success_count = Arc::new(AtomicU64::new(0));
    let failure_count = Arc::new(AtomicU64::new(0));
    
    let mut handles = Vec::new();
    
    // Spawn 100 concurrent operations
    for i in 0..100 {
        let breaker_clone = breaker.clone();
        let success_count_clone = success_count.clone();
        let failure_count_clone = failure_count.clone();
        
        let handle = tokio::spawn(async move {
            // Mix of successful and failing operations
            let result = if i % 3 == 0 {
                breaker_clone.call(TestOperations::always_fails::<i32>).await
            } else {
                breaker_clone.call(async move { TestOperations::always_succeeds(i).await }).await
            };
            
            if result.is_ok() {
                success_count_clone.fetch_add(1, Ordering::SeqCst);
            } else {
                failure_count_clone.fetch_add(1, Ordering::SeqCst);
            }
        });
        
        handles.push(handle);
    }
    
    // Wait for all operations to complete
    futures::future::join_all(handles).await;
    
    let successes = success_count.load(Ordering::SeqCst);
    let failures = failure_count.load(Ordering::SeqCst);
    
    println!("Stress test results: {} successes, {} failures", successes, failures);
    
    assert!(successes > 0);
    assert!(failures > 0);
    assert_eq!(successes + failures, 100);
}

#[tokio::test]
async fn test_race_conditions() {
    let breaker = Arc::new(create_test_breaker("race_test", ConfigFactory::create_fast_config()));
    
    // Multiple threads trying to transition states simultaneously
    let mut handles = Vec::new();
    
    // Thread 1: Try to open the circuit
    let breaker1 = breaker.clone();
    let handle1 = tokio::spawn(async move {
        for _ in 0..2 {
            let _ = breaker1.call(TestOperations::always_fails::<i32>).await;
        }
    });
    handles.push(handle1);
    
    // Thread 2: Try to reset the circuit
    let breaker2 = breaker.clone();
    let handle2 = tokio::spawn(async move {
        tokio::time::sleep(Duration::from_millis(10)).await;
        breaker2.reset().await;
    });
    handles.push(handle2);
    
    // Thread 3: Try to force open
    let breaker3 = breaker.clone();
    let handle3 = tokio::spawn(async move {
        tokio::time::sleep(Duration::from_millis(20)).await;
        breaker3.force_open().await;
    });
    handles.push(handle3);
    
    // Thread 4: Monitor state changes
    let breaker4 = breaker.clone();
    let handle4 = tokio::spawn(async move {
        let mut states = Vec::new();
        for _ in 0..10 {
            states.push(breaker4.get_state().await);
            tokio::time::sleep(Duration::from_millis(5)).await;
        }
        states
    });
    handles.push(handle4);
    
    let results = futures::future::join_all(handles).await;
    
    // Extract state monitoring results
    if let Ok(states) = &results[3] {
        // Should have observed state changes
        let unique_states: std::collections::HashSet<_> = states.iter().collect();
        assert!(unique_states.len() > 1, "Expected to see state changes");
    }
    
    // Final state should be consistent
    let final_state = breaker.get_state().await;
    assert!(matches!(final_state, CircuitState::Open | CircuitState::Closed));
}

#[tokio::test]
async fn test_concurrent_registry_operations() {
    let registry = Arc::new(CircuitBreakerRegistry::new());
    
    let mut handles = Vec::new();
    
    // Multiple threads registering services
    for i in 0..20 {
        let registry_clone = registry.clone();
        let handle = tokio::spawn(async move {
            registry_clone.register(
                format!("concurrent-service-{}", i),
                if i % 2 == 0 {
                    ConfigFactory::create_fast_config()
                } else {
                    ConfigFactory::create_strict_config()
                },
            ).await;
        });
        handles.push(handle);
    }
    
    // Wait for registration
    futures::future::join_all(handles).await;
    
    // Verify all services were registered
    let metrics = registry.get_all_metrics().await;
    assert_eq!(metrics.len(), 20);
    
    // Concurrent operations on registered services
    let mut operation_handles = Vec::new();
    
    for i in 0..20 {
        let registry_clone = registry.clone();
        let handle = tokio::spawn(async move {
            if let Some(breaker) = registry_clone.get(&format!("concurrent-service-{}", i)).await {
                let result = breaker.call(async move {
                    if i % 3 == 0 {
                        TestOperations::always_fails::<i32>().await
                    } else {
                        TestOperations::always_succeeds(i).await
                    }
                }).await;
                result.is_ok()
            } else {
                false
            }
        });
        operation_handles.push(handle);
    }
    
    let results: Vec<bool> = futures::future::join_all(operation_handles)
        .await
        .into_iter()
        .filter_map(Result::ok)
        .collect();
    
    let successes = results.iter().filter(|&&r| r).count();
    let failures = results.iter().filter(|&&r| !r).count();
    
    assert!(successes > 0);
    assert!(failures > 0);
}

#[tokio::test]
async fn test_thundering_herd_prevention() {
    let config = CircuitBreakerConfig {
        failure_threshold: 2,
        recovery_timeout: Duration::from_millis(100),
        request_timeout: Duration::from_millis(50),
        max_concurrent_requests: 100,
        success_threshold: 1,
    };
    
    let breaker = Arc::new(create_test_breaker("thundering_herd", config));
    
    // Open the circuit
    for _ in 0..2 {
        let _ = breaker.call(TestOperations::always_fails::<i32>).await;
    }
    assert_eq!(breaker.get_state().await, CircuitState::Open);
    
    // Wait for recovery timeout
    tokio::time::sleep(Duration::from_millis(150)).await;
    
    // Now simulate thundering herd - many requests at once after recovery
    let mut handles = Vec::new();
    let first_success = Arc::new(AtomicU64::new(0));
    
    for i in 0..50 {
        let breaker_clone = breaker.clone();
        let first_success_clone = first_success.clone();
        
        let handle = tokio::spawn(async move {
            let result = breaker_clone.call(async move {
                TestOperations::always_succeeds(i).await
            }).await;
            
            if result.is_ok() {
                first_success_clone.fetch_add(1, Ordering::SeqCst);
            }
            
            result.is_ok()
        });
        
        handles.push(handle);
    }
    
    let results: Vec<bool> = futures::future::join_all(handles)
        .await
        .into_iter()
        .filter_map(Result::ok)
        .collect();
    
    // Should have prevented thundering herd - only limited requests during half-open
    let initial_successes = first_success.load(Ordering::SeqCst);
    assert!(initial_successes < 50, "Too many requests allowed during recovery");
    
    // Eventually should recover
    assert_eq!(breaker.get_state().await, CircuitState::Closed);
}

#[tokio::test]
async fn test_performance_under_load() {
    let breaker = Arc::new(create_test_breaker("performance", ConfigFactory::create_lenient_config()));
    let operation_count = 1000;
    
    let start = std::time::Instant::now();
    
    let results = PerformanceTestUtils::run_concurrent_operations(
        &breaker,
        operation_count,
        || Ok(42),
    ).await;
    
    let duration = start.elapsed();
    
    assert_eq!(results.len(), operation_count);
    let success_rate = results.iter().filter(|r| r.is_ok()).count() as f64 / operation_count as f64;
    
    println!(
        "Performance test: {} operations in {:?}, success rate: {:.2}%",
        operation_count,
        duration,
        success_rate * 100.0
    );
    
    // Should complete reasonably fast
    assert!(duration.as_secs() < 5);
    // Should have high success rate with lenient config
    assert!(success_rate > 0.9);
}

#[tokio::test]
async fn test_memory_leak_prevention() {
    let registry = Arc::new(CircuitBreakerRegistry::new());
    
    // Register and unregister services repeatedly
    for iteration in 0..10 {
        // Register 100 services
        for i in 0..100 {
            registry.register(
                format!("temp-service-{}-{}", iteration, i),
                ConfigFactory::create_fast_config(),
            ).await;
        }
        
        // Use them
        for i in 0..100 {
            if let Some(breaker) = registry.get(&format!("temp-service-{}-{}", iteration, i)).await {
                let _ = breaker.call(async { TestOperations::always_succeeds(i).await }).await;
            }
        }
        
        // Reset all (simulate cleanup)
        registry.reset_all().await;
    }
    
    // Final check - registry should still be functional
    registry.register("final-service".to_string(), ConfigFactory::create_fast_config()).await;
    
    if let Some(breaker) = registry.get("final-service").await {
        let result = breaker.call(async { TestOperations::always_succeeds(999).await }).await;
        assert!(result.is_ok());
    }
}