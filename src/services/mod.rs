pub mod sample_service;
pub mod sequencing_service;
pub mod storage_service;
pub mod template_service;

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

/// Base trait for all services in the modular architecture
#[async_trait]
pub trait Service: Send + Sync {
    /// Get the service name for logging and monitoring
    fn name(&self) -> &'static str;

    /// Check if the service is healthy
    async fn health_check(&self) -> ServiceHealth;

    /// Get service configuration
    fn config(&self) -> ServiceConfig;
}

/// Service health status
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceHealth {
    pub status: HealthStatus,
    pub message: Option<String>,
    pub checks: HashMap<String, HealthCheck>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum HealthStatus {
    Healthy,
    Degraded,
    Unhealthy,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthCheck {
    pub status: HealthStatus,
    pub duration_ms: u64,
    pub details: Option<String>,
}

/// Service configuration trait
#[derive(Debug, Clone)]
pub struct ServiceConfig {
    pub name: String,
    pub version: String,
    pub dependencies: Vec<String>,
    pub settings: HashMap<String, String>,
}

/// Service registry for managing multiple service instances
#[derive(Default)]
pub struct ServiceRegistry {
    services: HashMap<String, Box<dyn Service>>,
}

impl ServiceRegistry {
    pub fn new() -> Self {
        Self {
            services: HashMap::new(),
        }
    }

    pub fn register<S: Service + 'static>(&mut self, service: S) {
        self.services
            .insert(service.name().to_string(), Box::new(service));
    }

    pub fn get(&self, name: &str) -> Option<&dyn Service> {
        self.services.get(name).map(|s| s.as_ref())
    }

    pub async fn health_check_all(&self) -> HashMap<String, ServiceHealth> {
        let mut results = HashMap::new();

        for (name, service) in &self.services {
            let health = service.health_check().await;
            results.insert(name.clone(), health);
        }

        results
    }

    pub fn list_services(&self) -> Vec<ServiceConfig> {
        self.services.values().map(|s| s.config()).collect()
    }
}

/// Paginated result type for service operations
#[derive(Debug, Serialize, Deserialize)]
pub struct PaginatedResult<T> {
    pub items: Vec<T>,
    pub total: u64,
    pub page: u32,
    pub per_page: u32,
    pub has_next: bool,
    pub has_prev: bool,
}

impl<T> PaginatedResult<T> {
    pub fn new(items: Vec<T>, total: u64, page: u32, per_page: u32) -> Self {
        let has_next = ((page * per_page) as u64) < total;
        let has_prev = page > 1;

        Self {
            items,
            total,
            page,
            per_page,
            has_next,
            has_prev,
        }
    }
}

/// Query parameters for service operations
#[derive(Debug, Deserialize)]
pub struct QueryParams {
    pub page: Option<u32>,
    pub per_page: Option<u32>,
    pub sort_by: Option<String>,
    pub sort_order: Option<SortOrder>,
    pub filters: HashMap<String, String>,
}

impl Default for QueryParams {
    fn default() -> Self {
        Self {
            page: Some(1),
            per_page: Some(20),
            sort_by: None,
            sort_order: Some(SortOrder::Asc),
            filters: HashMap::new(),
        }
    }
}

#[derive(Debug, Deserialize)]
pub enum SortOrder {
    Asc,
    Desc,
}

/// Service metrics for monitoring
#[derive(Debug, Serialize)]
pub struct ServiceMetrics {
    pub service_name: String,
    pub total_requests: u64,
    pub successful_requests: u64,
    pub failed_requests: u64,
    pub average_response_time_ms: f64,
    pub uptime_seconds: u64,
}

/// Service event for inter-service communication
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceEvent {
    pub id: Uuid,
    pub event_type: String,
    pub source_service: String,
    pub target_service: Option<String>,
    pub payload: serde_json::Value,
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

impl ServiceEvent {
    pub fn new(event_type: String, source_service: String, payload: serde_json::Value) -> Self {
        Self {
            id: Uuid::new_v4(),
            event_type,
            source_service,
            target_service: None,
            payload,
            timestamp: chrono::Utc::now(),
        }
    }

    pub fn with_target(mut self, target_service: String) -> Self {
        self.target_service = Some(target_service);
        self
    }
}
