// Library re-exports for examples and external use

pub mod assembly;
pub mod config;
pub mod errors;
pub mod events;
pub mod handlers;
pub mod logging;
pub mod middleware;
pub mod models;
pub mod observability;
pub mod plugins;
pub mod repositories;
pub mod router;
pub mod sample_submission;
pub mod sequencing;
pub mod services;
pub mod tests;
pub mod validation;

// Re-export main component types for convenience
pub use assembly::{
    AppComponents, AssemblyError, ComponentBuilder, DatabaseComponent, ObservabilityComponent,
    SampleProcessingComponent, SequencingComponent, StorageComponent,
};
pub use config::{AppConfig, ServerConfig};
pub use observability::{
    HealthChecker, HealthStatus, MetricValue, MetricsCollector, ServiceStatus, TracingService,
};
