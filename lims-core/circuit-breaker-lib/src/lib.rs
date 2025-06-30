use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::RwLock;
use tracing::{info, warn, error};
use serde::{Serialize, Deserialize};

#[derive(Debug, Clone, PartialEq)]
pub enum CircuitState {
    Closed,
    Open,
    HalfOpen,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CircuitBreakerConfig {
    pub failure_threshold: u64,
    pub recovery_timeout: Duration,
    pub request_timeout: Duration,
    pub max_concurrent_requests: u64,
    pub success_threshold: u64,
}

impl Default for CircuitBreakerConfig {
    fn default() -> Self {
        Self {
            failure_threshold: 5,
            recovery_timeout: Duration::from_secs(30),
            request_timeout: Duration::from_secs(10),
            max_concurrent_requests: 100,
            success_threshold: 3,
        }
    }
}

#[derive(Debug)]
struct CircuitBreakerState {
    state: CircuitState,
    failure_count: u64,
    success_count: u64,
    last_failure_time: Option<Instant>,
    concurrent_requests: u64,
}

impl Default for CircuitBreakerState {
    fn default() -> Self {
        Self {
            state: CircuitState::Closed,
            failure_count: 0,
            success_count: 0,
            last_failure_time: None,
            concurrent_requests: 0,
        }
    }
}

pub struct CircuitBreaker {
    config: CircuitBreakerConfig,
    state: Arc<RwLock<CircuitBreakerState>>,
    service_name: String,
}

#[derive(Debug, Clone)]
pub struct CircuitBreakerError {
    pub message: String,
    pub circuit_state: CircuitState,
}

impl std::fmt::Display for CircuitBreakerError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Circuit breaker error: {} (state: {:?})", self.message, self.circuit_state)
    }
}

impl std::error::Error for CircuitBreakerError {}

pub type CircuitBreakerResult<T> = Result<T, CircuitBreakerError>;

impl CircuitBreaker {
    pub fn new(service_name: String, config: CircuitBreakerConfig) -> Self {
        info!("Creating circuit breaker for service: {}", service_name);
        
        Self {
            config,
            state: Arc::new(RwLock::new(CircuitBreakerState::default())),
            service_name,
        }
    }

    pub async fn call<F, T, E>(&self, operation: F) -> CircuitBreakerResult<T>
    where
        F: std::future::Future<Output = Result<T, E>>,
        E: std::error::Error + Send + Sync + 'static,
    {
        // Check if we can proceed with the request
        if !self.can_proceed().await? {
            return Err(CircuitBreakerError {
                message: "Circuit breaker is open".to_string(),
                circuit_state: CircuitState::Open,
            });
        }

        // Increment concurrent requests
        self.increment_concurrent_requests().await;

        // Execute the operation with timeout
        let result = tokio::time::timeout(self.config.request_timeout, operation).await;

        // Handle the result
        match result {
            Ok(Ok(success)) => {
                self.on_success().await;
                Ok(success)
            }
            Ok(Err(error)) => {
                self.on_failure().await;
                Err(CircuitBreakerError {
                    message: format!("Operation failed: {}", error),
                    circuit_state: self.get_state().await,
                })
            }
            Err(_) => {
                self.on_failure().await;
                Err(CircuitBreakerError {
                    message: "Operation timed out".to_string(),
                    circuit_state: self.get_state().await,
                })
            }
        }
    }

    async fn can_proceed(&self) -> CircuitBreakerResult<bool> {
        let mut state = self.state.write().await;
        
        match state.state {
            CircuitState::Closed => {
                // Check if we have too many concurrent requests
                if state.concurrent_requests >= self.config.max_concurrent_requests {
                    return Ok(false);
                }
                Ok(true)
            }
            CircuitState::Open => {
                // Check if recovery timeout has passed
                if let Some(last_failure) = state.last_failure_time {
                    if last_failure.elapsed() >= self.config.recovery_timeout {
                        info!("Circuit breaker {} transitioning to half-open", self.service_name);
                        state.state = CircuitState::HalfOpen;
                        state.success_count = 0;
                        Ok(true)
                    } else {
                        Ok(false)
                    }
                } else {
                    Ok(false)
                }
            }
            CircuitState::HalfOpen => {
                // Allow limited requests to test if service is recovered
                Ok(state.concurrent_requests < 1)
            }
        }
    }

    async fn increment_concurrent_requests(&self) {
        let mut state = self.state.write().await;
        state.concurrent_requests += 1;
    }

    async fn decrement_concurrent_requests(&self) {
        let mut state = self.state.write().await;
        if state.concurrent_requests > 0 {
            state.concurrent_requests -= 1;
        }
    }

    async fn on_success(&self) {
        let mut state = self.state.write().await;
        
        // Decrement concurrent requests within the same lock scope
        if state.concurrent_requests > 0 {
            state.concurrent_requests -= 1;
        }
        
        match state.state {
            CircuitState::Closed => {
                // Reset failure count on success
                state.failure_count = 0;
            }
            CircuitState::HalfOpen => {
                state.success_count += 1;
                if state.success_count >= self.config.success_threshold {
                    info!("Circuit breaker {} transitioning to closed (recovered)", self.service_name);
                    state.state = CircuitState::Closed;
                    state.failure_count = 0;
                    state.success_count = 0;
                    state.last_failure_time = None;
                }
            }
            CircuitState::Open => {
                // This shouldn't happen, but reset if it does
                warn!("Unexpected success in open state for circuit breaker {}", self.service_name);
            }
        }
    }

    async fn on_failure(&self) {
        let mut state = self.state.write().await;
        
        // Decrement concurrent requests within the same lock scope
        if state.concurrent_requests > 0 {
            state.concurrent_requests -= 1;
        }
        
        state.failure_count += 1;
        state.last_failure_time = Some(Instant::now());

        match state.state {
            CircuitState::Closed => {
                if state.failure_count >= self.config.failure_threshold {
                    warn!("Circuit breaker {} transitioning to open (failure threshold exceeded)", self.service_name);
                    state.state = CircuitState::Open;
                }
            }
            CircuitState::HalfOpen => {
                warn!("Circuit breaker {} transitioning back to open (test failed)", self.service_name);
                state.state = CircuitState::Open;
            }
            CircuitState::Open => {
                // Already open, just record the failure
            }
        }
    }

    pub async fn get_state(&self) -> CircuitState {
        self.state.read().await.state.clone()
    }

    pub async fn get_metrics(&self) -> CircuitBreakerMetrics {
        let state = self.state.read().await;
        CircuitBreakerMetrics {
            service_name: self.service_name.clone(),
            state: state.state.clone(),
            failure_count: state.failure_count,
            success_count: state.success_count,
            concurrent_requests: state.concurrent_requests,
            last_failure_time: state.last_failure_time,
            config: self.config.clone(),
        }
    }

    pub async fn reset(&self) {
        let mut state = self.state.write().await;
        info!("Resetting circuit breaker for service: {}", self.service_name);
        
        state.state = CircuitState::Closed;
        state.failure_count = 0;
        state.success_count = 0;
        state.last_failure_time = None;
        state.concurrent_requests = 0;
    }

    pub async fn force_open(&self) {
        let mut state = self.state.write().await;
        warn!("Forcing circuit breaker open for service: {}", self.service_name);
        
        state.state = CircuitState::Open;
        state.last_failure_time = Some(Instant::now());
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct CircuitBreakerMetrics {
    pub service_name: String,
    pub state: CircuitState,
    pub failure_count: u64,
    pub success_count: u64,
    pub concurrent_requests: u64,
    pub last_failure_time: Option<Instant>,
    pub config: CircuitBreakerConfig,
}

// HTTP Client with Circuit Breaker
pub struct HttpClientWithCircuitBreaker {
    client: reqwest::Client,
    circuit_breaker: CircuitBreaker,
}

impl HttpClientWithCircuitBreaker {
    pub fn new(service_name: String, config: Option<CircuitBreakerConfig>) -> Self {
        let client = reqwest::Client::builder()
            .timeout(config.as_ref().unwrap_or(&CircuitBreakerConfig::default()).request_timeout)
            .build()
            .expect("Failed to create HTTP client");

        let circuit_breaker = CircuitBreaker::new(
            service_name,
            config.unwrap_or_default(),
        );

        Self {
            client,
            circuit_breaker,
        }
    }

    pub async fn get(&self, url: &str) -> CircuitBreakerResult<reqwest::Response> {
        self.circuit_breaker
            .call(async {
                self.client
                    .get(url)
                    .send()
                    .await
                    .map_err(|e| Box::new(e) as Box<dyn std::error::Error + Send + Sync>)
            })
            .await
    }

    pub async fn post<T: serde::Serialize>(&self, url: &str, json: &T) -> CircuitBreakerResult<reqwest::Response> {
        self.circuit_breaker
            .call(async {
                self.client
                    .post(url)
                    .json(json)
                    .send()
                    .await
                    .map_err(|e| Box::new(e) as Box<dyn std::error::Error + Send + Sync>)
            })
            .await
    }

    pub async fn put<T: serde::Serialize>(&self, url: &str, json: &T) -> CircuitBreakerResult<reqwest::Response> {
        self.circuit_breaker
            .call(async {
                self.client
                    .put(url)
                    .json(json)
                    .send()
                    .await
                    .map_err(|e| Box::new(e) as Box<dyn std::error::Error + Send + Sync>)
            })
            .await
    }

    pub async fn delete(&self, url: &str) -> CircuitBreakerResult<reqwest::Response> {
        self.circuit_breaker
            .call(async {
                self.client
                    .delete(url)
                    .send()
                    .await
                    .map_err(|e| Box::new(e) as Box<dyn std::error::Error + Send + Sync>)
            })
            .await
    }

    pub async fn get_metrics(&self) -> CircuitBreakerMetrics {
        self.circuit_breaker.get_metrics().await
    }

    pub async fn reset(&self) {
        self.circuit_breaker.reset().await
    }
}

// Service Registry for managing multiple circuit breakers
#[derive(Default)]
pub struct CircuitBreakerRegistry {
    breakers: Arc<RwLock<std::collections::HashMap<String, CircuitBreaker>>>,
}

impl CircuitBreakerRegistry {
    pub fn new() -> Self {
        Self {
            breakers: Arc::new(RwLock::new(std::collections::HashMap::new())),
        }
    }

    pub async fn register(&self, service_name: String, config: CircuitBreakerConfig) {
        let breaker = CircuitBreaker::new(service_name.clone(), config);
        let mut breakers = self.breakers.write().await;
        breakers.insert(service_name, breaker);
    }

    pub async fn get(&self, service_name: &str) -> Option<CircuitBreaker> {
        let breakers = self.breakers.read().await;
        breakers.get(service_name).cloned()
    }

    pub async fn get_all_metrics(&self) -> Vec<CircuitBreakerMetrics> {
        let breakers = self.breakers.read().await;
        let mut metrics = Vec::new();
        
        for breaker in breakers.values() {
            metrics.push(breaker.get_metrics().await);
        }
        
        metrics
    }

    pub async fn reset_all(&self) {
        let breakers = self.breakers.read().await;
        for breaker in breakers.values() {
            breaker.reset().await;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;

    #[tokio::test]
    async fn test_circuit_breaker_basic_functionality() {
        let config = CircuitBreakerConfig {
            failure_threshold: 2,
            recovery_timeout: Duration::from_millis(100),
            request_timeout: Duration::from_millis(50),
            max_concurrent_requests: 10,
            success_threshold: 1,
        };

        let breaker = CircuitBreaker::new("test-service".to_string(), config);

        // Test successful operation
        let result = breaker.call(async { Ok::<i32, std::io::Error>(42) }).await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 42);

        // Test failure leading to open state
        for _ in 0..2 {
            let _ = breaker.call(async { 
                Err::<i32, std::io::Error>(std::io::Error::new(std::io::ErrorKind::Other, "test error"))
            }).await;
        }

        assert_eq!(breaker.get_state().await, CircuitState::Open);

        // Test that requests are rejected when open
        let result = breaker.call(async { Ok::<i32, std::io::Error>(42) }).await;
        assert!(result.is_err());

        // Wait for recovery timeout
        tokio::time::sleep(Duration::from_millis(150)).await;

        // Test transition to half-open and recovery
        let result = breaker.call(async { Ok::<i32, std::io::Error>(42) }).await;
        assert!(result.is_ok());
        assert_eq!(breaker.get_state().await, CircuitState::Closed);
    }

    #[tokio::test]
    async fn test_http_client_with_circuit_breaker() {
        let client = HttpClientWithCircuitBreaker::new(
            "test-http-service".to_string(),
            Some(CircuitBreakerConfig::default()),
        );

        // This will fail because we're not running a real server
        let result = client.get("http://localhost:99999/test").await;
        assert!(result.is_err());

        let metrics = client.get_metrics().await;
        assert_eq!(metrics.service_name, "test-http-service");
        assert!(metrics.failure_count > 0);
    }
}