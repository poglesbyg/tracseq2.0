use axum::{
    body::{Body, Bytes},
    http::{HeaderMap, Method, StatusCode},
    response::{IntoResponse, Response},
    Json,
};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::RwLock;
use tracing::{error, info, warn};

/// Configuration for a microservice endpoint
#[derive(Debug, Clone, Deserialize)]
pub struct ServiceConfig {
    pub name: String,
    pub url: String,
    pub health_check_path: String,
    pub timeout_seconds: u64,
    pub retry_attempts: u32,
}

/// Health status of a service
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ServiceHealth {
    Healthy,
    Unhealthy,
    Unknown,
}

/// Circuit breaker states
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum CircuitState {
    Closed,
    Open,
    HalfOpen,
}

/// Circuit breaker for a service
#[derive(Debug)]
pub struct CircuitBreaker {
    state: RwLock<CircuitState>,
    failure_count: RwLock<u32>,
    last_failure_time: RwLock<Option<std::time::Instant>>,
    failure_threshold: u32,
    recovery_timeout: Duration,
}

impl CircuitBreaker {
    pub fn new(failure_threshold: u32, recovery_timeout_seconds: u64) -> Self {
        Self {
            state: RwLock::new(CircuitState::Closed),
            failure_count: RwLock::new(0),
            last_failure_time: RwLock::new(None),
            failure_threshold,
            recovery_timeout: Duration::from_secs(recovery_timeout_seconds),
        }
    }

    pub async fn is_available(&self) -> bool {
        let state = self.state.read().await;
        match *state {
            CircuitState::Closed => true,
            CircuitState::Open => {
                // Check if recovery timeout has passed
                if let Some(last_failure) = *self.last_failure_time.read().await {
                    if last_failure.elapsed() > self.recovery_timeout {
                        drop(state);
                        *self.state.write().await = CircuitState::HalfOpen;
                        true
                    } else {
                        false
                    }
                } else {
                    false
                }
            }
            CircuitState::HalfOpen => true,
        }
    }

    pub async fn record_success(&self) {
        *self.failure_count.write().await = 0;
        *self.state.write().await = CircuitState::Closed;
    }

    pub async fn record_failure(&self) {
        let mut failure_count = self.failure_count.write().await;
        *failure_count += 1;
        *self.last_failure_time.write().await = Some(std::time::Instant::now());

        if *failure_count >= self.failure_threshold {
            *self.state.write().await = CircuitState::Open;
            warn!("Circuit breaker opened after {} failures", failure_count);
        }
    }
}

/// Service proxy for routing requests to microservices
pub struct ServiceProxy {
    client: Client,
    services: HashMap<String, ServiceConfig>,
    circuit_breakers: Arc<RwLock<HashMap<String, Arc<CircuitBreaker>>>>,
}

impl ServiceProxy {
    pub fn new() -> Self {
        let client = Client::builder()
            .timeout(Duration::from_secs(30))
            .build()
            .expect("Failed to create HTTP client");

        let mut services = HashMap::new();

        // Configure microservices
        services.insert(
            "auth".to_string(),
            ServiceConfig {
                name: "auth_service".to_string(),
                url: std::env::var("AUTH_SERVICE_URL")
                    .unwrap_or_else(|_| "http://auth-service:3010".to_string()),
                health_check_path: "/health".to_string(),
                timeout_seconds: 30,
                retry_attempts: 3,
            },
        );

        services.insert(
            "sample".to_string(),
            ServiceConfig {
                name: "sample_service".to_string(),
                url: std::env::var("SAMPLE_SERVICE_URL")
                    .unwrap_or_else(|_| "http://sample-service:3011".to_string()),
                health_check_path: "/health".to_string(),
                timeout_seconds: 30,
                retry_attempts: 3,
            },
        );

        services.insert(
            "sequencing".to_string(),
            ServiceConfig {
                name: "sequencing_service".to_string(),
                url: std::env::var("SEQUENCING_SERVICE_URL")
                    .unwrap_or_else(|_| "http://lims-sequencing:8000".to_string()),
                health_check_path: "/health".to_string(),
                timeout_seconds: 30,
                retry_attempts: 3,
            },
        );

        services.insert(
            "template".to_string(),
            ServiceConfig {
                name: "template_service".to_string(),
                url: std::env::var("TEMPLATE_SERVICE_URL")
                    .unwrap_or_else(|_| "http://lims-templates:8083".to_string()),
                health_check_path: "/health".to_string(),
                timeout_seconds: 30,
                retry_attempts: 3,
            },
        );

        services.insert(
            "storage".to_string(),
            ServiceConfig {
                name: "enhanced_storage_service".to_string(),
                url: std::env::var("STORAGE_SERVICE_URL")
                    .unwrap_or_else(|_| "http://enhanced-storage-service:3014".to_string()),
                health_check_path: "/health".to_string(),
                timeout_seconds: 30,
                retry_attempts: 3,
            },
        );

        services.insert(
            "spreadsheet".to_string(),
            ServiceConfig {
                name: "spreadsheet_versioning_service".to_string(),
                url: std::env::var("SPREADSHEET_SERVICE_URL")
                    .unwrap_or_else(|_| "http://spreadsheet-versioning-service:3015".to_string()),
                health_check_path: "/health".to_string(),
                timeout_seconds: 30,
                retry_attempts: 3,
            },
        );

        // Enhanced services
        services.insert(
            "notifications".to_string(),
            ServiceConfig {
                name: "notification_service".to_string(),
                url: std::env::var("NOTIFICATION_SERVICE_URL")
                    .unwrap_or_else(|_| "http://lims-notification:8000".to_string()),
                health_check_path: "/health".to_string(),
                timeout_seconds: 30,
                retry_attempts: 3,
            },
        );

        services.insert(
            "events".to_string(),
            ServiceConfig {
                name: "event_service".to_string(),
                url: std::env::var("EVENT_SERVICE_URL")
                    .unwrap_or_else(|_| "http://lims-events:8087".to_string()),
                health_check_path: "/health".to_string(),
                timeout_seconds: 30,
                retry_attempts: 3,
            },
        );

        services.insert(
            "transactions".to_string(),
            ServiceConfig {
                name: "transaction_service".to_string(),
                url: std::env::var("TRANSACTION_SERVICE_URL")
                    .unwrap_or_else(|_| "http://lims-transactions:8000".to_string()),
                health_check_path: "/health".to_string(),
                timeout_seconds: 30,
                retry_attempts: 3,
            },
        );

        services.insert(
            "qaqc".to_string(),
            ServiceConfig {
                name: "qaqc_service".to_string(),
                url: std::env::var("QAQC_SERVICE_URL")
                    .unwrap_or_else(|_| "http://lims-qaqc:8089".to_string()),
                health_check_path: "/health".to_string(),
                timeout_seconds: 30,
                retry_attempts: 3,
            },
        );

        let circuit_breakers = Arc::new(RwLock::new(HashMap::new()));

        Self {
            client,
            services,
            circuit_breakers,
        }
    }

    /// Initialize circuit breakers for all services
    pub async fn initialize_circuit_breakers(&self) {
        let mut breakers = self.circuit_breakers.write().await;
        for (key, _) in &self.services {
            breakers.insert(
                key.clone(),
                Arc::new(CircuitBreaker::new(5, 60)), // 5 failures, 60s recovery
            );
        }
    }

    /// Check health of a specific service
    pub async fn check_service_health(&self, service_key: &str) -> ServiceHealth {
        if let Some(config) = self.services.get(service_key) {
            let url = format!("{}{}", config.url, config.health_check_path);
            match self.client.get(&url).send().await {
                Ok(response) if response.status().is_success() => ServiceHealth::Healthy,
                Ok(_) => ServiceHealth::Unhealthy,
                Err(e) => {
                    error!("Health check failed for {}: {}", service_key, e);
                    ServiceHealth::Unhealthy
                }
            }
        } else {
            ServiceHealth::Unknown
        }
    }

    /// Check health of all services
    pub async fn check_all_services_health(&self) -> HashMap<String, ServiceHealth> {
        let mut health_map = HashMap::new();
        for (key, _) in &self.services {
            let health = self.check_service_health(key).await;
            health_map.insert(key.clone(), health);
        }
        health_map
    }

    /// Proxy a request to a microservice
    pub async fn proxy_request(
        &self,
        service_key: &str,
        method: Method,
        path: &str,
        headers: HeaderMap,
        body: Option<Bytes>,
    ) -> Result<Response, ProxyError> {
        let config = self
            .services
            .get(service_key)
            .ok_or_else(|| ProxyError::ServiceNotFound(service_key.to_string()))?;

        // Check circuit breaker
        let breakers = self.circuit_breakers.read().await;
        if let Some(breaker) = breakers.get(service_key) {
            if !breaker.is_available().await {
                return Err(ProxyError::CircuitBreakerOpen(service_key.to_string()));
            }
        }
        drop(breakers);

        let url = format!("{}{}", config.url, path);
        info!("Proxying {} request to {}", method, url);

        // Build request
        let mut request = self.client.request(method.clone(), &url);
        
        // Forward headers (excluding host header)
        for (key, value) in headers.iter() {
            if key != "host" {
                request = request.header(key, value);
            }
        }

        // Add body if present
        if let Some(body_bytes) = body {
            request = request.body(body_bytes);
        }

        // Set timeout
        request = request.timeout(Duration::from_secs(config.timeout_seconds));

        // Execute request with retries
        let mut last_error = None;
        for attempt in 0..config.retry_attempts {
            if attempt > 0 {
                info!("Retry attempt {} for {}", attempt, url);
                tokio::time::sleep(Duration::from_millis(100 * attempt as u64)).await;
            }

            match request.try_clone().unwrap().send().await {
                Ok(response) => {
                    // Record success to circuit breaker
                    let breakers = self.circuit_breakers.read().await;
                    if let Some(breaker) = breakers.get(service_key) {
                        breaker.record_success().await;
                    }

                    // Convert response
                    let status = response.status();
                    let headers = response.headers().clone();
                    let body = response.bytes().await.map_err(ProxyError::ResponseBody)?;

                    let mut response_builder = Response::builder().status(status);
                    for (key, value) in headers.iter() {
                        response_builder = response_builder.header(key, value);
                    }

                    return response_builder
                        .body(Body::from(body))
                        .map_err(|e| ProxyError::ResponseBuild(e.to_string()));
                }
                Err(e) => {
                    last_error = Some(e);
                }
            }
        }

        // Record failure to circuit breaker
        let breakers = self.circuit_breakers.read().await;
        if let Some(breaker) = breakers.get(service_key) {
            breaker.record_failure().await;
        }

        Err(ProxyError::RequestFailed(
            last_error.unwrap().to_string(),
        ))
    }

    /// Get service configuration
    pub fn get_service_config(&self, service_key: &str) -> Option<&ServiceConfig> {
        self.services.get(service_key)
    }

    /// List all configured services
    pub fn list_services(&self) -> Vec<String> {
        self.services.keys().cloned().collect()
    }
}

/// Proxy error types
#[derive(Debug, thiserror::Error)]
pub enum ProxyError {
    #[error("Service not found: {0}")]
    ServiceNotFound(String),

    #[error("Circuit breaker is open for service: {0}")]
    CircuitBreakerOpen(String),

    #[error("Request failed: {0}")]
    RequestFailed(String),

    #[error("Failed to read response body: {0}")]
    ResponseBody(reqwest::Error),

    #[error("Failed to build response: {0}")]
    ResponseBuild(String),
}

impl IntoResponse for ProxyError {
    fn into_response(self) -> Response {
        let (status, message) = match self {
            ProxyError::ServiceNotFound(_) => (StatusCode::NOT_FOUND, self.to_string()),
            ProxyError::CircuitBreakerOpen(_) => (StatusCode::SERVICE_UNAVAILABLE, self.to_string()),
            ProxyError::RequestFailed(_) => (StatusCode::BAD_GATEWAY, self.to_string()),
            ProxyError::ResponseBody(_) => (StatusCode::BAD_GATEWAY, self.to_string()),
            ProxyError::ResponseBuild(_) => (StatusCode::INTERNAL_SERVER_ERROR, self.to_string()),
        };

        (status, Json(serde_json::json!({ "error": message }))).into_response()
    }
}

/// Service registry response
#[derive(Debug, Serialize)]
pub struct ServiceRegistryResponse {
    pub services: Vec<ServiceInfo>,
}

#[derive(Debug, Serialize)]
pub struct ServiceInfo {
    pub key: String,
    pub name: String,
    pub url: String,
    pub health: ServiceHealth,
}

impl Serialize for ServiceHealth {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        match self {
            ServiceHealth::Healthy => serializer.serialize_str("healthy"),
            ServiceHealth::Unhealthy => serializer.serialize_str("unhealthy"),
            ServiceHealth::Unknown => serializer.serialize_str("unknown"),
        }
    }
}

// Global service proxy instance
lazy_static::lazy_static! {
    pub static ref SERVICE_PROXY: ServiceProxy = ServiceProxy::new();
}