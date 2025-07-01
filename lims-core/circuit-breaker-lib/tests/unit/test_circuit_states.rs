//! Unit tests for circuit breaker state transitions

use circuit_breaker_lib::{CircuitBreaker, CircuitState};
use crate::test_utils::*;
use std::time::Duration;

test_with_circuit_breaker!(
    test_initial_state_is_closed,
    ConfigFactory::create_fast_config(),
    |breaker: &CircuitBreaker| async move {
        let state = breaker.get_state().await;
        CircuitBreakerAssertions::assert_state(&state, CircuitState::Closed);
    }
);

test_with_circuit_breaker!(
    test_transition_to_open_on_failures,
    ConfigFactory::create_fast_config(),
    |breaker: &CircuitBreaker| async move {
        // Fail twice to trigger open state (threshold is 2)
        let _ = breaker.call(TestOperations::always_fails::<i32>).await;
        let _ = breaker.call(TestOperations::always_fails::<i32>).await;
        
        let state = breaker.get_state().await;
        CircuitBreakerAssertions::assert_state(&state, CircuitState::Open);
    }
);

test_with_circuit_breaker!(
    test_stays_closed_below_threshold,
    ConfigFactory::create_fast_config(),
    |breaker: &CircuitBreaker| async move {
        // One failure (below threshold of 2)
        let _ = breaker.call(TestOperations::always_fails::<i32>).await;
        
        let state = breaker.get_state().await;
        CircuitBreakerAssertions::assert_state(&state, CircuitState::Closed);
        
        // Success should reset failure count
        let _ = breaker.call(async { TestOperations::always_succeeds(42).await }).await;
        let _ = breaker.call(TestOperations::always_fails::<i32>).await;
        
        let state = breaker.get_state().await;
        CircuitBreakerAssertions::assert_state(&state, CircuitState::Closed);
    }
);

test_with_circuit_breaker!(
    test_transition_to_half_open_after_timeout,
    ConfigFactory::create_fast_config(),
    |breaker: &CircuitBreaker| async move {
        // Open the circuit
        let _ = breaker.call(TestOperations::always_fails::<i32>).await;
        let _ = breaker.call(TestOperations::always_fails::<i32>).await;
        
        assert_eq!(breaker.get_state().await, CircuitState::Open);
        
        // Wait for recovery timeout (100ms)
        tokio::time::sleep(Duration::from_millis(150)).await;
        
        // Next call should transition to half-open
        let _ = breaker.call(async { TestOperations::always_succeeds(42).await }).await;
        
        // After one success (success_threshold is 1), should be closed
        let state = breaker.get_state().await;
        CircuitBreakerAssertions::assert_state(&state, CircuitState::Closed);
    }
);

test_with_circuit_breaker!(
    test_half_open_to_open_on_failure,
    ConfigFactory::create_strict_config(),
    |breaker: &CircuitBreaker| async move {
        // Open the circuit (threshold is 1 for strict config)
        let _ = breaker.call(TestOperations::always_fails::<i32>).await;
        assert_eq!(breaker.get_state().await, CircuitState::Open);
        
        // Wait for recovery timeout
        tokio::time::sleep(Duration::from_millis(600)).await;
        
        // Next failure should go back to open
        let _ = breaker.call(TestOperations::always_fails::<i32>).await;
        
        let state = breaker.get_state().await;
        CircuitBreakerAssertions::assert_state(&state, CircuitState::Open);
    }
);

test_with_circuit_breaker!(
    test_half_open_to_closed_on_success,
    ConfigFactory::create_strict_config(),
    |breaker: &CircuitBreaker| async move {
        // Open the circuit
        let _ = breaker.call(TestOperations::always_fails::<i32>).await;
        assert_eq!(breaker.get_state().await, CircuitState::Open);
        
        // Wait for recovery timeout
        tokio::time::sleep(Duration::from_millis(600)).await;
        
        // Need 3 successes for strict config
        for _ in 0..3 {
            let _ = breaker.call(async { TestOperations::always_succeeds(42).await }).await;
        }
        
        let state = breaker.get_state().await;
        CircuitBreakerAssertions::assert_state(&state, CircuitState::Closed);
    }
);

test_with_circuit_breaker!(
    test_reset_functionality,
    ConfigFactory::create_fast_config(),
    |breaker: &CircuitBreaker| async move {
        // Open the circuit
        let _ = breaker.call(TestOperations::always_fails::<i32>).await;
        let _ = breaker.call(TestOperations::always_fails::<i32>).await;
        assert_eq!(breaker.get_state().await, CircuitState::Open);
        
        // Reset the circuit
        breaker.reset().await;
        
        // Should be closed again
        let state = breaker.get_state().await;
        CircuitBreakerAssertions::assert_state(&state, CircuitState::Closed);
        
        // Metrics should be reset
        let metrics = breaker.get_metrics().await;
        CircuitBreakerAssertions::assert_failure_count(&metrics, 0);
        CircuitBreakerAssertions::assert_success_count(&metrics, 0);
    }
);

test_with_circuit_breaker!(
    test_force_open_functionality,
    ConfigFactory::create_fast_config(),
    |breaker: &CircuitBreaker| async move {
        // Initially closed
        assert_eq!(breaker.get_state().await, CircuitState::Closed);
        
        // Force open
        breaker.force_open().await;
        
        let state = breaker.get_state().await;
        CircuitBreakerAssertions::assert_state(&state, CircuitState::Open);
        
        // Should reject calls
        let result = breaker.call(async { TestOperations::always_succeeds(42).await }).await;
        assert!(result.is_err());
    }
);

#[tokio::test]
async fn test_complex_state_transitions() {
    let breaker = create_test_breaker("complex_test", ConfigFactory::create_fast_config());
    
    // Closed -> Open
    for _ in 0..2 {
        let _ = breaker.call(TestOperations::always_fails::<i32>).await;
    }
    assert_eq!(breaker.get_state().await, CircuitState::Open);
    
    // Wait for recovery
    tokio::time::sleep(Duration::from_millis(150)).await;
    
    // Half-open -> Closed
    let result = breaker.call(async { TestOperations::always_succeeds(42).await }).await;
    assert!(result.is_ok());
    assert_eq!(breaker.get_state().await, CircuitState::Closed);
    
    // Back to Open
    for _ in 0..2 {
        let _ = breaker.call(TestOperations::always_fails::<i32>).await;
    }
    assert_eq!(breaker.get_state().await, CircuitState::Open);
}

#[tokio::test]
async fn test_state_transitions_with_mixed_results() {
    let scenario = TestScenarioBuilder::new()
        .add_success(1)
        .add_failure()
        .add_success(2)
        .add_failure()
        .add_failure() // This should open the circuit
        .add_success(3); // This should be rejected
    
    let breaker = create_test_breaker("mixed_test", ConfigFactory::create_fast_config());
    let results = scenario.execute_with_breaker(&breaker).await;
    
    // First 4 should execute (2 successes, 2 failures)
    assert!(results[0].is_ok());
    assert!(results[1].is_err());
    assert!(results[2].is_ok());
    assert!(results[3].is_err());
    assert!(results[4].is_err()); // This triggers open state
    assert!(results[5].is_err()); // This is rejected due to open state
    
    assert_eq!(breaker.get_state().await, CircuitState::Open);
}

#[tokio::test]
async fn test_concurrent_state_checks() {
    let breaker = create_test_breaker("concurrent_state", ConfigFactory::create_fast_config());
    
    // Open the circuit
    for _ in 0..2 {
        let _ = breaker.call(TestOperations::always_fails::<i32>).await;
    }
    
    // Check state concurrently
    let mut handles = Vec::new();
    for _ in 0..10 {
        let breaker_clone = breaker.clone();
        let handle = tokio::spawn(async move {
            breaker_clone.get_state().await
        });
        handles.push(handle);
    }
    
    let states: Vec<CircuitState> = futures::future::join_all(handles)
        .await
        .into_iter()
        .filter_map(Result::ok)
        .collect();
    
    // All should report Open state
    assert!(states.iter().all(|s| *s == CircuitState::Open));
}