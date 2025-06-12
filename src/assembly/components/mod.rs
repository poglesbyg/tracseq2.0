/// Modular component implementations for the IKEA-like architecture
///
/// This module contains different component implementations that can be mixed and matched
/// to create different application configurations. Each component implements the core
/// traits defined in the parent traits module.
pub mod database;
pub mod event_system;
pub mod monitoring;
pub mod sample_processing;
pub mod storage;
pub mod template_processing;

// Re-export key types for easier usage
pub use database::{DatabaseComponent, DatabaseComponentBuilder};
pub use event_system::{EventSystemBuilder, EventSystemComponent, EventSystemConfig, LabEvent};
pub use monitoring::{
    Alert, MonitoringBuilder, MonitoringComponent, MonitoringConfig, SystemMetrics,
};
pub use sample_processing::{
    ProcessingResult, ProcessingStage, SampleProcessingBuilder, SampleProcessingComponent,
    SampleProcessingConfig,
};
pub use storage::{StorageBackend, StorageComponent, StorageComponentBuilder};
pub use template_processing::{
    TemplateProcessingBuilder, TemplateProcessingComponent, TemplateProcessingConfig,
    TemplateResult, TemplateStage,
};
