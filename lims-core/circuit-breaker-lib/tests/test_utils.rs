use circuit_breaker_lib::{
    CircuitBreaker, CircuitBreakerConfig, CircuitBreakerError, CircuitBreakerResult,
    CircuitState, CircuitBreakerMetrics, HttpClientWithCircuitBreaker, CircuitBreakerRegistry,
};
use std::time::Duration;
use std::sync::Arc;
use tokio::sync::RwLock;
use wiremock::{MockServer, Mock, ResponseTemplate};
use wiremock::matchers::{method, path};

/// Factory for creating test circuit breaker configurations
pub struct ConfigFactory;

impl ConfigFactory {
    pub fn create_fast_config() -> CircuitBreakerConfig {
        CircuitBreakerConfig {
            failure_threshold: 2,
            recovery_timeout: Duration::from_millis(100),
            request_timeout: Duration::from_millis(50),
            max_concurrent_requests: 10,
            success_threshold: 1,
        }
    }

    pub fn create_strict_config() -> CircuitBreakerConfig {
        CircuitBreakerConfig {
            failure_threshold: 1,
            recovery_timeout: Duration::from_millis(500),
            request_timeout: Duration::from_millis(100),
            max_concurrent_requests: 5,
            success_threshold: 3,
        }
    }

    pub fn create_lenient_config() -> CircuitBreakerConfig {
        CircuitBreakerConfig {
            failure_threshold: 10,
            recovery_timeout: Duration::from_millis(50),
            request_timeout: Duration::from_secs(1),
            max_concurrent_requests: 100,
            success_threshold: 1,
        }
    }
}

/// Test operations for simulating various scenarios
pub struct TestOperations;

impl TestOperations {
    pub async fn always_succeeds<T: Clone + Send>(value: T) -> Result<T, std::io::Error> {
        Ok(value)
    }

    pub async fn always_fails<T>() -> Result<T, std::io::Error> {
        Err(std::io::Error::new(std::io::ErrorKind::Other, "Test failure"))
    }

    pub async fn slow_operation<T: Clone + Send>(duration: Duration, value: T) -> Result<T, std::io::Error> {
        tokio::time::sleep(duration).await;
        Ok(value)
    }

    pub async fn flaky_operation<T: Clone + Send>(success_rate: f64, value: T) -> Result<T, std::io::Error> {
        if rand::random::<f64>() < success_rate {
            Ok(value)
        } else {
            Err(std::io::Error::new(std::io::ErrorKind::Other, "Random failure"))
        }
    }
}

/// Counter for tracking operation calls
#[derive(Clone)]
pub struct CallCounter {
    count: Arc<RwLock<u64>>,
}

impl CallCounter {
    pub fn new() -> Self {
        Self {
            count: Arc::new(RwLock::new(0)),
        }
    }

    pub async fn increment(&self) {
        let mut count = self.count.write().await;
        *count += 1;
    }

    pub async fn get(&self) -> u64 {
        *self.count.read().await
    }

    pub async fn reset(&self) {
        let mut count = self.count.write().await;
        *count = 0;
    }
}

/// Mock server setup for HTTP client tests
pub struct MockServerSetup;

impl MockServerSetup {
    pub async fn setup_success_endpoint(mock_server: &MockServer) {
        Mock::given(method("GET"))
            .and(path("/success"))
            .respond_with(ResponseTemplate::new(200)
                .set_body_json(serde_json::json!({"status": "ok"})))
            .mount(mock_server)
            .await;
    }

    pub async fn setup_failure_endpoint(mock_server: &MockServer) {
        Mock::given(method("GET"))
            .and(path("/failure"))
            .respond_with(ResponseTemplate::new(500)
                .set_body_string("Internal server error"))
            .mount(mock_server)
            .await;
    }

    pub async fn setup_slow_endpoint(mock_server: &MockServer, delay: Duration) {
        Mock::given(method("GET"))
            .and(path("/slow"))
            .respond_with(ResponseTemplate::new(200)
                .set_delay(delay)
                .set_body_json(serde_json::json!({"status": "slow but ok"})))
            .mount(mock_server)
            .await;
    }

    pub async fn setup_flaky_endpoint(mock_server: &MockServer) {
        // This endpoint alternates between success and failure
        Mock::given(method("GET"))
            .and(path("/flaky"))
            .respond_with(|req: &wiremock::Request| {
                static COUNTER: std::sync::atomic::AtomicU32 = std::sync::atomic::AtomicU32::new(0);
                let count = COUNTER.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
                
                if count % 2 == 0 {
                    ResponseTemplate::new(200)
                        .set_body_json(serde_json::json!({"status": "ok"}))
                } else {
                    ResponseTemplate::new(500)
                        .set_body_string("Flaky failure")
                }
            })
            .mount(mock_server)
            .await;
    }
}

/// Assertions for circuit breaker tests
pub struct CircuitBreakerAssertions;

impl CircuitBreakerAssertions {
    pub fn assert_state(actual: &CircuitState, expected: CircuitState) {
        assert_eq!(
            actual, &expected,
            "Expected circuit state {:?}, but got {:?}",
            expected, actual
        );
    }

    pub fn assert_metrics(metrics: &CircuitBreakerMetrics, expected_state: CircuitState) {
        assert_eq!(
            metrics.state, expected_state,
            "Expected state {:?} in metrics, but got {:?}",
            expected_state, metrics.state
        );
    }

    pub fn assert_error_contains(error: &CircuitBreakerError, substring: &str) {
        assert!(
            error.message.contains(substring),
            "Expected error message to contain '{}', but got '{}'",
            substring, error.message
        );
    }

    pub fn assert_failure_count(metrics: &CircuitBreakerMetrics, expected: u64) {
        assert_eq!(
            metrics.failure_count, expected,
            "Expected {} failures, but got {}",
            expected, metrics.failure_count
        );
    }

    pub fn assert_success_count(metrics: &CircuitBreakerMetrics, expected: u64) {
        assert_eq!(
            metrics.success_count, expected,
            "Expected {} successes, but got {}",
            expected, metrics.success_count
        );
    }
}

/// Performance test utilities
pub struct PerformanceTestUtils;

impl PerformanceTestUtils {
    pub async fn measure_operation_time<F, Fut, T>(operation: F) -> (Duration, CircuitBreakerResult<T>)
    where
        F: FnOnce() -> Fut,
        Fut: std::future::Future<Output = CircuitBreakerResult<T>>,
    {
        let start = std::time::Instant::now();
        let result = operation().await;
        (start.elapsed(), result)
    }

    pub async fn run_concurrent_operations<F, T>(
        breaker: &CircuitBreaker,
        operation_count: usize,
        operation: F,
    ) -> Vec<CircuitBreakerResult<T>>
    where
        F: Fn() -> Result<T, std::io::Error> + Clone + Send + 'static,
        T: Send + 'static,
    {
        let mut handles = Vec::new();
        
        for _ in 0..operation_count {
            let breaker_clone = breaker.clone();
            let op = operation.clone();
            let handle = tokio::spawn(async move {
                breaker_clone.call(async move { op() }).await
            });
            handles.push(handle);
        }
        
        let mut results = Vec::new();
        for handle in handles {
            if let Ok(result) = handle.await {
                results.push(result);
            }
        }
        
        results
    }
}

/// Helper to create test circuit breaker
pub fn create_test_breaker(name: &str, config: CircuitBreakerConfig) -> CircuitBreaker {
    CircuitBreaker::new(name.to_string(), config)
}

/// Test scenario builder for complex test cases
pub struct TestScenarioBuilder {
    operations: Vec<Box<dyn Fn() -> Result<i32, std::io::Error> + Send + Sync>>,
}

impl TestScenarioBuilder {
    pub fn new() -> Self {
        Self {
            operations: Vec::new(),
        }
    }

    pub fn add_success(mut self, value: i32) -> Self {
        self.operations.push(Box::new(move || Ok(value)));
        self
    }

    pub fn add_failure(mut self) -> Self {
        self.operations.push(Box::new(|| {
            Err(std::io::Error::new(std::io::ErrorKind::Other, "Scenario failure"))
        }));
        self
    }

    pub fn add_n_successes(mut self, n: usize, value: i32) -> Self {
        for _ in 0..n {
            self = self.add_success(value);
        }
        self
    }

    pub fn add_n_failures(mut self, n: usize) -> Self {
        for _ in 0..n {
            self = self.add_failure();
        }
        self
    }

    pub async fn execute_with_breaker(self, breaker: &CircuitBreaker) -> Vec<CircuitBreakerResult<i32>> {
        let mut results = Vec::new();
        
        for operation in self.operations {
            let result = breaker.call(async move { operation() }).await;
            results.push(result);
        }
        
        results
    }
}

/// Test macro for circuit breaker tests
#[macro_export]
macro_rules! test_with_circuit_breaker {
    ($test_name:ident, $config:expr, $test_body:expr) => {
        #[tokio::test]
        async fn $test_name() {
            let breaker = crate::test_utils::create_test_breaker(
                stringify!($test_name), 
                $config
            );
            
            let result = std::panic::AssertUnwindSafe($test_body(&breaker))
                .catch_unwind()
                .await;
            
            if let Err(panic) = result {
                std::panic::resume_unwind(panic);
            }
        }
    };
}