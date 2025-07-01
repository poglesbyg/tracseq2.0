//! Unit tests for circuit breaker operations

use circuit_breaker_lib::{CircuitBreaker, CircuitState, CircuitBreakerError};
use crate::test_utils::*;
use std::time::Duration;

test_with_circuit_breaker!(
    test_successful_operation,
    ConfigFactory::create_fast_config(),
    |breaker: &CircuitBreaker| async move {
        let result = breaker.call(async { TestOperations::always_succeeds(42).await }).await;
        
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 42);
        
        let metrics = breaker.get_metrics().await;
        CircuitBreakerAssertions::assert_success_count(&metrics, 1);
        CircuitBreakerAssertions::assert_failure_count(&metrics, 0);
    }
);

test_with_circuit_breaker!(
    test_failed_operation,
    ConfigFactory::create_fast_config(),
    |breaker: &CircuitBreaker| async move {
        let result = breaker.call(TestOperations::always_fails::<i32>).await;
        
        assert!(result.is_err());
        if let Err(error) = result {
            CircuitBreakerAssertions::assert_error_contains(&error, "Operation failed");
        }
        
        let metrics = breaker.get_metrics().await;
        CircuitBreakerAssertions::assert_success_count(&metrics, 0);
        CircuitBreakerAssertions::assert_failure_count(&metrics, 1);
    }
);

test_with_circuit_breaker!(
    test_timeout_operation,
    ConfigFactory::create_fast_config(),
    |breaker: &CircuitBreaker| async move {
        // Operation takes 100ms, timeout is 50ms
        let result = breaker.call(
            TestOperations::slow_operation(Duration::from_millis(100), 42)
        ).await;
        
        assert!(result.is_err());
        if let Err(error) = result {
            CircuitBreakerAssertions::assert_error_contains(&error, "timed out");
        }
        
        let metrics = breaker.get_metrics().await;
        CircuitBreakerAssertions::assert_failure_count(&metrics, 1);
    }
);

test_with_circuit_breaker!(
    test_concurrent_request_limit,
    ConfigFactory::create_fast_config(),
    |breaker: &CircuitBreaker| async move {
        // max_concurrent_requests is 10
        let mut handles = Vec::new();
        
        // Start 15 concurrent slow operations
        for i in 0..15 {
            let breaker_clone = breaker.clone();
            let handle = tokio::spawn(async move {
                breaker_clone.call(
                    TestOperations::slow_operation(Duration::from_millis(200), i)
                ).await
            });
            handles.push(handle);
        }
        
        // Give operations time to start
        tokio::time::sleep(Duration::from_millis(50)).await;
        
        let results: Vec<_> = futures::future::join_all(handles).await;
        
        // Some should succeed, some should be rejected due to concurrent limit
        let successes = results.iter().filter(|r| r.is_ok()).count();
        let failures = results.iter().filter(|r| r.is_err()).count();
        
        assert!(successes > 0);
        assert!(failures > 0);
    }
);

test_with_circuit_breaker!(
    test_operation_rejected_when_open,
    ConfigFactory::create_fast_config(),
    |breaker: &CircuitBreaker| async move {
        // Open the circuit
        breaker.force_open().await;
        
        // Try to call operation
        let result = breaker.call(async { TestOperations::always_succeeds(42).await }).await;
        
        assert!(result.is_err());
        if let Err(error) = result {
            assert_eq!(error.circuit_state, CircuitState::Open);
            CircuitBreakerAssertions::assert_error_contains(&error, "Circuit breaker is open");
        }
    }
);

test_with_circuit_breaker!(
    test_metrics_tracking,
    ConfigFactory::create_fast_config(),
    |breaker: &CircuitBreaker| async move {
        // Mix of successes and failures
        let _ = breaker.call(async { TestOperations::always_succeeds(1).await }).await;
        let _ = breaker.call(TestOperations::always_fails::<i32>).await;
        let _ = breaker.call(async { TestOperations::always_succeeds(2).await }).await;
        let _ = breaker.call(TestOperations::always_fails::<i32>).await;
        
        let metrics = breaker.get_metrics().await;
        assert_eq!(metrics.service_name, "test_metrics_tracking");
        CircuitBreakerAssertions::assert_success_count(&metrics, 2);
        CircuitBreakerAssertions::assert_failure_count(&metrics, 2);
        assert_eq!(metrics.state, CircuitState::Open); // 2 failures = threshold
    }
);

#[tokio::test]
async fn test_operation_with_different_error_types() {
    let breaker = create_test_breaker("error_types", ConfigFactory::create_fast_config());
    
    // Test with IO error
    let io_error_result = breaker.call(async {
        Err::<i32, std::io::Error>(std::io::Error::new(
            std::io::ErrorKind::NotFound,
            "File not found"
        ))
    }).await;
    
    assert!(io_error_result.is_err());
    
    // Test with custom error type
    #[derive(Debug)]
    struct CustomError(&'static str);
    impl std::fmt::Display for CustomError {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            write!(f, "{}", self.0)
        }
    }
    impl std::error::Error for CustomError {}
    
    let custom_error_result = breaker.call(async {
        Err::<i32, CustomError>(CustomError("Custom error"))
    }).await;
    
    assert!(custom_error_result.is_err());
}

#[tokio::test]
async fn test_operation_performance() {
    let breaker = create_test_breaker("performance", ConfigFactory::create_lenient_config());
    let counter = CallCounter::new();
    
    let start = std::time::Instant::now();
    
    // Run 100 fast operations
    for _ in 0..100 {
        let counter_clone = counter.clone();
        let _ = breaker.call(async move {
            counter_clone.increment().await;
            TestOperations::always_succeeds(1).await
        }).await;
    }
    
    let duration = start.elapsed();
    let count = counter.get().await;
    
    assert_eq!(count, 100);
    assert!(
        duration.as_millis() < 1000,
        "100 operations took {:?}, which is too slow",
        duration
    );
    
    println!("Performed {} operations in {:?}", count, duration);
}

#[tokio::test]
async fn test_flaky_operation_handling() {
    let breaker = create_test_breaker("flaky", ConfigFactory::create_lenient_config());
    let mut successes = 0;
    let mut failures = 0;
    
    // Run flaky operation multiple times
    for _ in 0..20 {
        let result = breaker.call(
            TestOperations::flaky_operation(0.5, 42) // 50% success rate
        ).await;
        
        if result.is_ok() {
            successes += 1;
        } else {
            failures += 1;
        }
    }
    
    // Should have some of each
    assert!(successes > 0);
    assert!(failures > 0);
    
    // Circuit should still be functional (lenient config)
    assert_ne!(breaker.get_state().await, CircuitState::Open);
}

test_with_circuit_breaker!(
    test_operation_result_types,
    ConfigFactory::create_fast_config(),
    |breaker: &CircuitBreaker| async move {
        // Test with different result types
        let string_result = breaker.call(async {
            TestOperations::always_succeeds("Hello".to_string()).await
        }).await;
        assert_eq!(string_result.unwrap(), "Hello");
        
        let vec_result = breaker.call(async {
            TestOperations::always_succeeds(vec![1, 2, 3]).await
        }).await;
        assert_eq!(vec_result.unwrap(), vec![1, 2, 3]);
        
        let option_result = breaker.call(async {
            TestOperations::always_succeeds(Some(42)).await
        }).await;
        assert_eq!(option_result.unwrap(), Some(42));
    }
);

#[tokio::test]
async fn test_operation_chaining() {
    let breaker = create_test_breaker("chaining", ConfigFactory::create_fast_config());
    
    let result = breaker.call(async {
        Ok::<i32, std::io::Error>(10)
    }).await
    .and_then(|value| {
        if value > 5 {
            Ok(value * 2)
        } else {
            Err(CircuitBreakerError {
                message: "Value too small".to_string(),
                circuit_state: CircuitState::Closed,
            })
        }
    });
    
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), 20);
}