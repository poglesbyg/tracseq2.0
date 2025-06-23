use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::any::Any;
use std::collections::HashMap;
use std::time::{Duration, Instant};

use super::super::traits::{
    Component, ComponentError, Configurable, ServiceProvider, ServiceRegistry,
};

/// Configuration for the monitoring component
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MonitoringConfig {
    /// Interval between health checks in seconds
    pub health_check_interval: u64,
    /// Interval between metric collection in seconds
    pub metrics_collection_interval: u64,
    /// Maximum number of metrics to store in memory
    pub max_metrics_history: usize,
    /// Enable performance monitoring
    pub enable_performance_monitoring: bool,
    /// Enable alert generation
    pub enable_alerts: bool,
    /// Alert thresholds
    pub alert_thresholds: AlertThresholds,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AlertThresholds {
    /// CPU usage threshold (percentage)
    pub cpu_usage_threshold: f64,
    /// Memory usage threshold (percentage)  
    pub memory_usage_threshold: f64,
    /// Response time threshold (milliseconds)
    pub response_time_threshold: u64,
    /// Error rate threshold (percentage)
    pub error_rate_threshold: f64,
}

impl Default for MonitoringConfig {
    fn default() -> Self {
        Self {
            health_check_interval: 30,
            metrics_collection_interval: 60,
            max_metrics_history: 1000,
            enable_performance_monitoring: true,
            enable_alerts: true,
            alert_thresholds: AlertThresholds {
                cpu_usage_threshold: 80.0,
                memory_usage_threshold: 85.0,
                response_time_threshold: 5000,
                error_rate_threshold: 5.0,
            },
        }
    }
}

/// System metrics snapshot
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemMetrics {
    pub timestamp: chrono::DateTime<chrono::Utc>,
    pub cpu_usage: f64,
    pub memory_usage: f64,
    pub disk_usage: f64,
    pub network_io: NetworkIO,
    pub component_metrics: HashMap<String, ComponentMetrics>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkIO {
    pub bytes_sent: u64,
    pub bytes_received: u64,
    pub packets_sent: u64,
    pub packets_received: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComponentMetrics {
    pub component_id: String,
    pub component_name: String,
    pub status: HealthStatus,
    pub response_time: Duration,
    pub request_count: u64,
    pub error_count: u64,
    pub custom_metrics: HashMap<String, f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum HealthStatus {
    Healthy,
    Warning,
    Critical,
    Unknown,
}

/// Alert severity levels
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum AlertSeverity {
    Info,
    Warning,
    Critical,
    Emergency,
}

/// Alert type categorization
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum AlertType {
    ComponentFailure,
    StorageCapacity,
    SecurityEvent,
    PerformanceIssue,
    SystemEvent,
}

/// System alert representation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Alert {
    pub id: String,
    pub alert_type: AlertType,
    pub severity: AlertSeverity,
    pub message: String,
    pub component: Option<String>,
    pub timestamp: chrono::DateTime<chrono::Utc>,
    pub resolved: bool,
}

impl Alert {
    /// Create a new alert
    pub fn new(
        severity: AlertSeverity,
        message: String,
        source: String,
        component: Option<String>,
    ) -> Self {
        let id = format!("alert_{}", uuid::Uuid::new_v4());

        Self {
            id,
            alert_type: AlertType::SystemEvent,
            severity,
            message,
            component,
            timestamp: chrono::Utc::now(),
            resolved: false,
        }
    }
}

/// Monitoring component that provides system observability
pub struct MonitoringComponent {
    config: MonitoringConfig,
    metrics_history: Vec<SystemMetrics>,
    active_alerts: HashMap<String, Alert>,
    service_registry: Option<std::sync::Arc<ServiceRegistry>>,
    monitoring_start_time: Option<Instant>,
    is_initialized: bool,
    component_health: HashMap<String, bool>,
}

impl MonitoringComponent {
    pub fn new(config: MonitoringConfig) -> Self {
        Self {
            config,
            metrics_history: Vec::new(),
            active_alerts: HashMap::new(),
            service_registry: None,
            monitoring_start_time: None,
            is_initialized: false,
            component_health: HashMap::new(),
        }
    }

    /// Collect current system metrics
    pub async fn collect_metrics(&mut self) -> Result<SystemMetrics, ComponentError> {
        if !self.is_initialized {
            return Err(ComponentError::InitializationFailed(
                "Component not initialized".to_string(),
            ));
        }

        let metrics = SystemMetrics {
            timestamp: chrono::Utc::now(),
            cpu_usage: self.get_cpu_usage().await,
            memory_usage: self.get_memory_usage().await,
            disk_usage: self.get_disk_usage().await,
            network_io: self.get_network_io().await,
            component_metrics: self.collect_component_metrics().await?,
        };

        // Store in history
        self.store_metrics(metrics.clone());

        // Check for alerts
        if self.config.enable_alerts {
            self.check_alerts(&metrics).await;
        }

        Ok(metrics)
    }

    /// Perform health check on all registered components
    pub async fn perform_health_check(
        &self,
    ) -> Result<HashMap<String, HealthStatus>, ComponentError> {
        if !self.is_initialized {
            return Err(ComponentError::InitializationFailed(
                "Component not initialized".to_string(),
            ));
        }

        let mut health_results = HashMap::new();

        if let Some(registry) = &self.service_registry {
            let health_check_results = registry.health_check_all().await?;

            // Update component health status
            for health_result in health_check_results {
                // Parse the health status string (format: "component_id: Healthy")
                if let Some(idx) = health_result.find(':') {
                    let component_id = health_result[..idx].trim().to_string();
                    let is_healthy = health_result[idx + 1..].trim().contains("Healthy");

                    // Update metrics
                    let component_status = if is_healthy { "healthy" } else { "unhealthy" };

                    self.component_health
                        .insert(component_id.clone(), is_healthy);

                    // Add to alert if unhealthy
                    if !is_healthy {
                        let alert = Alert {
                            id: format!("component_failure_{}", component_id),
                            alert_type: AlertType::ComponentFailure,
                            severity: AlertSeverity::Critical,
                            message: format!("Component {} is in critical state", component_id),
                            component: Some(component_id.clone()),
                            timestamp: chrono::Utc::now(),
                            resolved: false,
                        };
                        self.active_alerts.insert(alert.id.clone(), alert);
                    }
                }
            }

            for (component_id, is_healthy) in self.component_health.iter() {
                let status = if *is_healthy {
                    HealthStatus::Healthy
                } else {
                    HealthStatus::Critical
                };
                health_results.insert(component_id.clone(), status);
            }
        }

        Ok(health_results)
    }

    /// Get current alerts
    pub fn get_active_alerts(&self) -> Vec<Alert> {
        self.active_alerts.values().cloned().collect()
    }

    /// Get metrics history
    pub fn get_metrics_history(&self, limit: Option<usize>) -> Vec<SystemMetrics> {
        match limit {
            Some(n) => self.metrics_history.iter().rev().take(n).cloned().collect(),
            None => self.metrics_history.clone(),
        }
    }

    /// Generate monitoring report
    pub fn generate_report(&self) -> MonitoringReport {
        let uptime = self
            .monitoring_start_time
            .map(|start| start.elapsed())
            .unwrap_or(Duration::ZERO);

        let total_alerts = self.active_alerts.len();
        let critical_alerts = self
            .active_alerts
            .values()
            .filter(|alert| alert.severity == AlertSeverity::Critical)
            .count();

        let latest_metrics = self.metrics_history.last();
        let avg_cpu = self.calculate_average_cpu();
        let avg_memory = self.calculate_average_memory();

        MonitoringReport {
            uptime,
            total_alerts,
            critical_alerts,
            latest_metrics: latest_metrics.cloned(),
            avg_cpu_usage: avg_cpu,
            avg_memory_usage: avg_memory,
            metrics_collected: self.metrics_history.len(),
            report_timestamp: chrono::Utc::now(),
        }
    }

    /// Clear resolved alerts
    pub fn clear_resolved_alerts(&mut self) {
        self.active_alerts.retain(|_, alert| !alert.resolved);
    }

    // Private helper methods

    async fn get_cpu_usage(&self) -> f64 {
        // Mock CPU usage - in real implementation would use system APIs
        use fastrand;
        50.0 + fastrand::f64() * 30.0 // 50-80% range
    }

    async fn get_memory_usage(&self) -> f64 {
        // Mock memory usage
        use fastrand;
        40.0 + fastrand::f64() * 40.0 // 40-80% range
    }

    async fn get_disk_usage(&self) -> f64 {
        // Mock disk usage
        use fastrand;
        30.0 + fastrand::f64() * 50.0 // 30-80% range
    }

    async fn get_network_io(&self) -> NetworkIO {
        // Mock network I/O
        use fastrand;
        NetworkIO {
            bytes_sent: fastrand::u64(1000000..10000000),
            bytes_received: fastrand::u64(1000000..10000000),
            packets_sent: fastrand::u64(1000..10000),
            packets_received: fastrand::u64(1000..10000),
        }
    }

    async fn collect_component_metrics(
        &self,
    ) -> Result<HashMap<String, ComponentMetrics>, ComponentError> {
        let mut component_metrics = HashMap::new();

        if let Some(registry) = &self.service_registry {
            let health_results = registry.health_check_all().await?;

            for (component_id, is_healthy) in health_results {
                let status = if is_healthy {
                    HealthStatus::Healthy
                } else {
                    HealthStatus::Critical
                };

                let metrics = ComponentMetrics {
                    component_id: component_id.clone(),
                    component_name: component_id.clone(), // In real implementation, get actual name
                    status,
                    response_time: Duration::from_millis(fastrand::u64(10..500)),
                    request_count: fastrand::u64(100..10000),
                    error_count: fastrand::u64(0..100),
                    custom_metrics: HashMap::new(),
                };

                component_metrics.insert(component_id, metrics);
            }
        }

        Ok(component_metrics)
    }

    fn store_metrics(&mut self, metrics: SystemMetrics) {
        self.metrics_history.push(metrics);

        // Trim history if it exceeds max size
        while self.metrics_history.len() > self.config.max_metrics_history {
            self.metrics_history.remove(0);
        }
    }

    async fn check_alerts(&mut self, metrics: &SystemMetrics) {
        // Check CPU usage
        if metrics.cpu_usage > self.config.alert_thresholds.cpu_usage_threshold {
            let alert = Alert {
                id: "high_cpu_usage".to_string(),
                alert_type: AlertType::PerformanceIssue,
                severity: AlertSeverity::Warning,
                message: format!("High CPU usage: {:.1}%", metrics.cpu_usage),
                component: None,
                timestamp: chrono::Utc::now(),
                resolved: false,
            };
            self.active_alerts.insert(alert.id.clone(), alert);
        }

        // Check memory usage
        if metrics.memory_usage > self.config.alert_thresholds.memory_usage_threshold {
            let alert = Alert {
                id: "high_memory_usage".to_string(),
                alert_type: AlertType::PerformanceIssue,
                severity: AlertSeverity::Warning,
                message: format!("High memory usage: {:.1}%", metrics.memory_usage),
                component: None,
                timestamp: chrono::Utc::now(),
                resolved: false,
            };
            self.active_alerts.insert(alert.id.clone(), alert);
        }

        // Check component health
        for (component_id, component_metrics) in &metrics.component_metrics {
            if component_metrics.status == HealthStatus::Critical {
                let alert = Alert {
                    id: format!("component_failure_{}", component_id),
                    alert_type: AlertType::ComponentFailure,
                    severity: AlertSeverity::Critical,
                    message: format!("Component {} is in critical state", component_id),
                    component: Some(component_id.clone()),
                    timestamp: chrono::Utc::now(),
                    resolved: false,
                };
                self.active_alerts.insert(alert.id.clone(), alert);
            }

            // Check response time
            if component_metrics.response_time.as_millis()
                > self.config.alert_thresholds.response_time_threshold as u128
            {
                let alert = Alert {
                    id: format!("high_response_time_{}", component_id),
                    alert_type: AlertType::PerformanceIssue,
                    severity: AlertSeverity::Warning,
                    message: format!(
                        "High response time for {}: {}ms",
                        component_id,
                        component_metrics.response_time.as_millis()
                    ),
                    component: Some(component_id.clone()),
                    timestamp: chrono::Utc::now(),
                    resolved: false,
                };
                self.active_alerts.insert(alert.id.clone(), alert);
            }
        }
    }

    fn calculate_average_cpu(&self) -> f64 {
        if self.metrics_history.is_empty() {
            return 0.0;
        }

        let sum: f64 = self.metrics_history.iter().map(|m| m.cpu_usage).sum();

        sum / self.metrics_history.len() as f64
    }

    fn calculate_average_memory(&self) -> f64 {
        if self.metrics_history.is_empty() {
            return 0.0;
        }

        let sum: f64 = self.metrics_history.iter().map(|m| m.memory_usage).sum();

        sum / self.metrics_history.len() as f64
    }

    /// Generate health alerts based on registry components
    async fn generate_health_alerts(&mut self, registry: &ServiceRegistry) -> Vec<Alert> {
        let mut alerts = Vec::new();

        // Run health checks
        if let Ok(health_results) = registry.health_check_all().await {
            // Update component health status
            for health_result in health_results {
                // Parse the health status string (format: "component_id: Healthy")
                if let Some(idx) = health_result.find(':') {
                    let component_id = health_result[..idx].trim().to_string();
                    let is_healthy = health_result[idx + 1..].trim().contains("Healthy");

                    // Update component health status
                    self.component_health
                        .insert(component_id.clone(), is_healthy);

                    // Add to alert if unhealthy
                    if !is_healthy {
                        alerts.push(Alert::new(
                            AlertSeverity::Critical,
                            format!("Component {} is unhealthy", component_id),
                            "monitoring".to_string(),
                            Some(component_id),
                        ));
                    }
                }
            }
        }

        alerts
    }

    /// Update component health status based on health check results
    async fn update_component_health(
        &mut self,
        registry: &ServiceRegistry,
    ) -> Result<(), ComponentError> {
        if let Ok(health_check_results) = registry.health_check_all().await {
            // Update component health status
            for health_result in health_check_results {
                // Parse the health status string (format: "component_id: Healthy")
                if let Some(idx) = health_result.find(':') {
                    let component_id = health_result[..idx].trim().to_string();
                    let is_healthy = health_result[idx + 1..].trim().contains("Healthy");

                    // Update metrics
                    let component_status = if is_healthy { "healthy" } else { "unhealthy" };

                    self.component_health
                        .insert(component_id.clone(), is_healthy);

                    // Add to alert if unhealthy
                    if !is_healthy {
                        let alert = Alert {
                            id: format!("component_failure_{}", component_id),
                            alert_type: AlertType::ComponentFailure,
                            severity: AlertSeverity::Critical,
                            message: format!("Component {} is in critical state", component_id),
                            component: Some(component_id.clone()),
                            timestamp: chrono::Utc::now(),
                            resolved: false,
                        };
                        self.active_alerts.insert(alert.id.clone(), alert);
                    }
                }
            }
        }
        Ok(())
    }

    /// Check component health status and return results
    async fn check_health(&self) -> HashMap<String, HealthStatus> {
        let mut health_results = HashMap::new();

        // If we have a registry, check component health
        if let Some(registry) = &self.service_registry {
            if let Ok(health_check_results) = registry.health_check_all().await {
                // Parse health check results
                for health_result in health_check_results {
                    // Parse the health status string (format: "component_id: Healthy")
                    if let Some(idx) = health_result.find(':') {
                        let component_id = health_result[..idx].trim().to_string();
                        let is_healthy = health_result[idx + 1..].trim().contains("Healthy");

                        let status = if is_healthy {
                            HealthStatus::Healthy
                        } else {
                            HealthStatus::Critical
                        };

                        health_results.insert(component_id, status);
                    }
                }
            }
        }

        // Add our own health status
        health_results.insert("monitoring".to_string(), HealthStatus::Healthy);

        health_results
    }

    /// Process health check results and update internal state
    async fn process_health_check(&mut self) -> Result<(), ComponentError> {
        let mut alerts = Vec::new();

        // Check component health
        if let Some(registry) = &self.service_registry {
            if let Ok(health_results) = registry.health_check_all().await {
                // Process each health result string
                for health_result in health_results {
                    // Parse the health status string (format: "component_id: Healthy")
                    if let Some(idx) = health_result.find(':') {
                        let component_id = health_result[..idx].trim().to_string();
                        let is_healthy = health_result[idx + 1..].trim().contains("Healthy");

                        // Update component health status
                        self.component_health
                            .insert(component_id.clone(), is_healthy);

                        // Add to alert if unhealthy
                        if !is_healthy {
                            alerts.push(Alert::new(
                                AlertSeverity::Critical,
                                format!("Component {} is unhealthy", component_id),
                                "monitoring".to_string(),
                                Some(component_id),
                            ));
                        }
                    }
                }
            }
        }

        // Process alerts
        for alert in alerts {
            self.active_alerts.insert(alert.id.clone(), alert);
        }

        Ok(())
    }

    /// Process health check results from the health_results HashMap
    async fn process_health_results(
        &mut self,
        health_results: &HashMap<String, String>,
    ) -> Result<(), ComponentError> {
        // Process each health result
        for (component_id, health_status) in health_results {
            let is_healthy = health_status.contains("Healthy");

            // Update metrics
            let status = if is_healthy {
                HealthStatus::Healthy
            } else {
                HealthStatus::Critical
            };

            self.component_health
                .insert(component_id.clone(), is_healthy);

            // Add to alert if unhealthy
            if !is_healthy {
                let alert = Alert::new(
                    AlertSeverity::Critical,
                    format!("Component {} is unhealthy", component_id),
                    "monitoring".to_string(),
                    Some(component_id.clone()),
                );
                self.active_alerts.insert(alert.id.clone(), alert);
            }
        }

        Ok(())
    }
}

#[async_trait]
impl Component for MonitoringComponent {
    fn component_id(&self) -> &'static str {
        "monitoring"
    }

    fn component_name(&self) -> &'static str {
        "System Monitoring & Observability"
    }

    async fn initialize(&mut self, context: &ServiceRegistry) -> Result<(), ComponentError> {
        if self.is_initialized {
            return Ok(());
        }

        tracing::info!("Initializing monitoring component");

        // Store reference to service registry for health checks
        // Note: This is a simplified approach - in production, we'd need proper Arc handling

        self.monitoring_start_time = Some(Instant::now());
        self.is_initialized = true;

        tracing::info!("Monitoring component initialized successfully");
        Ok(())
    }

    async fn health_check(&self) -> Result<(), ComponentError> {
        if !self.is_initialized {
            return Err(ComponentError::InitializationFailed(
                "Component not initialized".to_string(),
            ));
        }

        // Check if monitoring is functioning properly
        if self.monitoring_start_time.is_none() {
            return Err(ComponentError::ServiceUnavailable(
                "Monitoring start time not set".to_string(),
            ));
        }

        Ok(())
    }

    async fn shutdown(&mut self) -> Result<(), ComponentError> {
        tracing::info!("Shutting down monitoring component");

        // Generate final report
        let final_report = self.generate_report();
        tracing::info!(
            "Final monitoring report: uptime={:?}, alerts={}, metrics={}",
            final_report.uptime,
            final_report.total_alerts,
            final_report.metrics_collected
        );

        // Clear all data
        self.metrics_history.clear();
        self.active_alerts.clear();
        self.service_registry = None;
        self.monitoring_start_time = None;
        self.is_initialized = false;

        Ok(())
    }

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
}

#[async_trait]
impl ServiceProvider for MonitoringComponent {
    fn provided_services(&self) -> Vec<&'static str> {
        vec![
            "monitoring",
            "metrics_collection",
            "health_monitoring",
            "alert_management",
            "system_observability",
        ]
    }
}

impl Configurable for MonitoringComponent {
    type Config = MonitoringConfig;

    fn configure(&mut self, config: Self::Config) -> Result<(), ComponentError> {
        if self.is_initialized {
            return Err(ComponentError::ConfigurationError(
                "Cannot reconfigure initialized component".to_string(),
            ));
        }

        // Validate configuration
        if config.health_check_interval == 0 {
            return Err(ComponentError::ConfigurationError(
                "Health check interval must be greater than 0".to_string(),
            ));
        }

        if config.max_metrics_history == 0 {
            return Err(ComponentError::ConfigurationError(
                "Metrics history size must be greater than 0".to_string(),
            ));
        }

        self.config = config;
        Ok(())
    }

    fn get_config(&self) -> &Self::Config {
        &self.config
    }
}

/// Monitoring report structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MonitoringReport {
    pub uptime: Duration,
    pub total_alerts: usize,
    pub critical_alerts: usize,
    pub latest_metrics: Option<SystemMetrics>,
    pub avg_cpu_usage: f64,
    pub avg_memory_usage: f64,
    pub metrics_collected: usize,
    pub report_timestamp: chrono::DateTime<chrono::Utc>,
}

/// Builder for monitoring components
pub struct MonitoringBuilder {
    config: MonitoringConfig,
}

impl MonitoringBuilder {
    pub fn new() -> Self {
        Self {
            config: MonitoringConfig::default(),
        }
    }

    pub fn with_health_check_interval(mut self, seconds: u64) -> Self {
        self.config.health_check_interval = seconds;
        self
    }

    pub fn with_metrics_interval(mut self, seconds: u64) -> Self {
        self.config.metrics_collection_interval = seconds;
        self
    }

    pub fn with_max_history(mut self, size: usize) -> Self {
        self.config.max_metrics_history = size;
        self
    }

    pub fn with_alerts(mut self, enabled: bool) -> Self {
        self.config.enable_alerts = enabled;
        self
    }

    pub fn with_performance_monitoring(mut self, enabled: bool) -> Self {
        self.config.enable_performance_monitoring = enabled;
        self
    }

    pub fn for_production(mut self) -> Self {
        self.config.health_check_interval = 60;
        self.config.metrics_collection_interval = 300;
        self.config.max_metrics_history = 2880; // 24 hours at 5-minute intervals
        self.config.enable_alerts = true;
        self.config.enable_performance_monitoring = true;
        self
    }

    pub fn for_development(mut self) -> Self {
        self.config.health_check_interval = 10;
        self.config.metrics_collection_interval = 30;
        self.config.max_metrics_history = 100;
        self.config.enable_alerts = false;
        self.config.enable_performance_monitoring = false;
        self
    }

    pub fn build(self) -> MonitoringComponent {
        MonitoringComponent::new(self.config)
    }
}

impl Default for MonitoringBuilder {
    fn default() -> Self {
        Self::new()
    }
}
