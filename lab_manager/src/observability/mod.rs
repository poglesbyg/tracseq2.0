use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::RwLock;
use tracing::{debug, error, info, warn};
use uuid::Uuid;

/// Metrics collector for application performance monitoring
pub struct MetricsCollector {
    metrics: Arc<RwLock<HashMap<String, MetricValue>>>,
    counters: Arc<RwLock<HashMap<String, u64>>>,
    histograms: Arc<RwLock<HashMap<String, Vec<f64>>>>,
}

#[derive(Debug, Clone, Serialize)]
pub enum MetricValue {
    Counter(u64),
    Gauge(f64),
    Histogram(Vec<f64>),
    Timer(Duration),
}

impl MetricsCollector {
    pub fn new() -> Self {
        Self {
            metrics: Arc::new(RwLock::new(HashMap::new())),
            counters: Arc::new(RwLock::new(HashMap::new())),
            histograms: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    pub async fn increment_counter(&self, name: &str, value: u64) {
        let mut counters = self.counters.write().await;
        *counters.entry(name.to_string()).or_insert(0) += value;
    }

    pub async fn set_gauge(&self, name: &str, value: f64) {
        let mut metrics = self.metrics.write().await;
        metrics.insert(name.to_string(), MetricValue::Gauge(value));
    }

    pub async fn record_histogram(&self, name: &str, value: f64) {
        let mut histograms = self.histograms.write().await;
        histograms
            .entry(name.to_string())
            .or_insert_with(Vec::new)
            .push(value);
    }

    pub async fn record_duration(&self, name: &str, duration: Duration) {
        let mut metrics = self.metrics.write().await;
        metrics.insert(name.to_string(), MetricValue::Timer(duration));
    }

    pub async fn get_all_metrics(&self) -> HashMap<String, MetricValue> {
        let metrics = self.metrics.read().await;
        let counters = self.counters.read().await;
        let histograms = self.histograms.read().await;

        let mut all_metrics = metrics.clone();

        for (name, value) in counters.iter() {
            all_metrics.insert(name.clone(), MetricValue::Counter(*value));
        }

        for (name, values) in histograms.iter() {
            all_metrics.insert(name.clone(), MetricValue::Histogram(values.clone()));
        }

        all_metrics
    }
}

/// Distributed tracing service
pub struct TracingService {
    active_spans: Arc<RwLock<HashMap<String, TraceSpan>>>,
}

#[derive(Debug, Clone, Serialize)]
pub struct TraceSpan {
    pub span_id: String,
    pub trace_id: String,
    pub parent_span_id: Option<String>,
    pub operation_name: String,
    pub start_time: DateTime<Utc>,
    pub end_time: Option<DateTime<Utc>>,
    pub duration: Option<Duration>,
    pub tags: HashMap<String, String>,
    pub logs: Vec<SpanLog>,
}

#[derive(Debug, Clone, Serialize)]
pub struct SpanLog {
    pub timestamp: DateTime<Utc>,
    pub level: LogLevel,
    pub message: String,
    pub fields: HashMap<String, String>,
}

#[derive(Debug, Clone, Serialize)]
pub enum LogLevel {
    Debug,
    Info,
    Warn,
    Error,
}

impl TracingService {
    pub fn new() -> Self {
        Self {
            active_spans: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    pub async fn start_span(
        &self,
        operation_name: &str,
        trace_id: Option<String>,
        parent_span_id: Option<String>,
    ) -> String {
        let span_id = Uuid::new_v4().to_string();
        let trace_id = trace_id.unwrap_or_else(|| Uuid::new_v4().to_string());

        let span = TraceSpan {
            span_id: span_id.clone(),
            trace_id,
            parent_span_id,
            operation_name: operation_name.to_string(),
            start_time: Utc::now(),
            end_time: None,
            duration: None,
            tags: HashMap::new(),
            logs: Vec::new(),
        };

        let mut spans = self.active_spans.write().await;
        spans.insert(span_id.clone(), span);

        debug!(
            "Started span: {} for operation: {}",
            span_id, operation_name
        );
        span_id
    }

    pub async fn finish_span(&self, span_id: &str) {
        let mut spans = self.active_spans.write().await;
        if let Some(mut span) = spans.remove(span_id) {
            let end_time = Utc::now();
            span.end_time = Some(end_time);
            span.duration = Some(
                end_time
                    .signed_duration_since(span.start_time)
                    .to_std()
                    .unwrap_or(Duration::from_secs(0)),
            );

            debug!(
                "Finished span: {} with duration: {:?}",
                span_id, span.duration
            );

            // Here you would typically export the span to a tracing backend like Jaeger or Zipkin
            self.export_span(&span).await;
        }
    }

    pub async fn add_span_tag(&self, span_id: &str, key: &str, value: &str) {
        let mut spans = self.active_spans.write().await;
        if let Some(span) = spans.get_mut(span_id) {
            span.tags.insert(key.to_string(), value.to_string());
        }
    }

    pub async fn add_span_log(
        &self,
        span_id: &str,
        level: LogLevel,
        message: &str,
        fields: HashMap<String, String>,
    ) {
        let mut spans = self.active_spans.write().await;
        if let Some(span) = spans.get_mut(span_id) {
            span.logs.push(SpanLog {
                timestamp: Utc::now(),
                level,
                message: message.to_string(),
                fields,
            });
        }
    }

    async fn export_span(&self, span: &TraceSpan) {
        // Implementation would export to tracing backend
        info!("Exporting span: {} to tracing backend", span.span_id);
    }
}

/// Health checking service
pub struct HealthChecker {
    checks: Arc<RwLock<HashMap<String, Box<dyn HealthCheck + Send + Sync>>>>,
}

#[async_trait::async_trait]
pub trait HealthCheck {
    async fn check(&self) -> HealthStatus;
    fn name(&self) -> &str;
}

#[derive(Debug, Clone, Serialize)]
pub struct HealthStatus {
    pub is_healthy: bool,
    pub status: ServiceStatus,
    pub message: Option<String>,
    pub response_time_ms: u64,
    pub last_checked: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize)]
pub enum ServiceStatus {
    Healthy,
    Degraded,
    Unhealthy,
    Unknown,
}

impl HealthChecker {
    pub fn new() -> Self {
        Self {
            checks: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    pub async fn register_check<T: HealthCheck + Send + Sync + 'static>(&self, check: T) {
        let mut checks = self.checks.write().await;
        checks.insert(check.name().to_string(), Box::new(check));
    }

    pub async fn check_all(&self) -> HashMap<String, HealthStatus> {
        let checks = self.checks.read().await;
        let mut results = HashMap::new();

        for (name, check) in checks.iter() {
            let start = Instant::now();
            let status = check.check().await;
            let duration = start.elapsed();

            let health_status = HealthStatus {
                is_healthy: status.is_healthy,
                status: status.status,
                message: status.message,
                response_time_ms: duration.as_millis() as u64,
                last_checked: Utc::now(),
            };

            results.insert(name.clone(), health_status);
        }

        results
    }

    pub async fn check_single(&self, name: &str) -> Option<HealthStatus> {
        let checks = self.checks.read().await;
        if let Some(check) = checks.get(name) {
            let start = Instant::now();
            let status = check.check().await;
            let duration = start.elapsed();

            Some(HealthStatus {
                is_healthy: status.is_healthy,
                status: status.status,
                message: status.message,
                response_time_ms: duration.as_millis() as u64,
                last_checked: Utc::now(),
            })
        } else {
            None
        }
    }
}

/// Database health check implementation
pub struct DatabaseHealthCheck {
    pool: sqlx::PgPool,
}

impl DatabaseHealthCheck {
    pub fn new(pool: sqlx::PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait::async_trait]
impl HealthCheck for DatabaseHealthCheck {
    async fn check(&self) -> HealthStatus {
        match sqlx::query("SELECT 1").fetch_one(&self.pool).await {
            Ok(_) => HealthStatus {
                is_healthy: true,
                status: ServiceStatus::Healthy,
                message: Some("Database connection successful".to_string()),
                response_time_ms: 0, // Will be filled by HealthChecker
                last_checked: Utc::now(),
            },
            Err(e) => HealthStatus {
                is_healthy: false,
                status: ServiceStatus::Unhealthy,
                message: Some(format!("Database connection failed: {}", e)),
                response_time_ms: 0,
                last_checked: Utc::now(),
            },
        }
    }

    fn name(&self) -> &str {
        "database"
    }
}

/// RAG Service health check implementation
pub struct RagServiceHealthCheck {
    base_url: String,
    client: reqwest::Client,
}

impl RagServiceHealthCheck {
    pub fn new(base_url: String) -> Self {
        Self {
            base_url,
            client: reqwest::Client::new(),
        }
    }
}

#[async_trait::async_trait]
impl HealthCheck for RagServiceHealthCheck {
    async fn check(&self) -> HealthStatus {
        let health_url = format!("{}/health", self.base_url);

        match self
            .client
            .get(&health_url)
            .timeout(Duration::from_secs(5))
            .send()
            .await
        {
            Ok(response) if response.status().is_success() => HealthStatus {
                is_healthy: true,
                status: ServiceStatus::Healthy,
                message: Some("RAG service is responding".to_string()),
                response_time_ms: 0,
                last_checked: Utc::now(),
            },
            Ok(response) => HealthStatus {
                is_healthy: false,
                status: ServiceStatus::Degraded,
                message: Some(format!(
                    "RAG service returned status: {}",
                    response.status()
                )),
                response_time_ms: 0,
                last_checked: Utc::now(),
            },
            Err(e) => HealthStatus {
                is_healthy: false,
                status: ServiceStatus::Unhealthy,
                message: Some(format!("RAG service connection failed: {}", e)),
                response_time_ms: 0,
                last_checked: Utc::now(),
            },
        }
    }

    fn name(&self) -> &str {
        "rag_service"
    }
}
