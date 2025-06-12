use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::any::Any;
use std::collections::{HashMap, VecDeque};
use std::sync::{Arc, Mutex};
use tokio::sync::broadcast;

use super::super::traits::{
    Component, ComponentError, Configurable, ServiceConsumer, ServiceProvider, ServiceRegistry,
};

/// Configuration for the event system component
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EventSystemConfig {
    /// Maximum number of events to keep in memory
    pub max_event_history: usize,
    /// Enable event persistence to storage
    pub enable_persistence: bool,
    /// Event processing batch size
    pub batch_size: usize,
    /// Enable event metrics collection
    pub enable_metrics: bool,
}

impl Default for EventSystemConfig {
    fn default() -> Self {
        Self {
            max_event_history: 10000,
            enable_persistence: false,
            batch_size: 100,
            enable_metrics: true,
        }
    }
}

/// Laboratory event types that flow through the system
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum LabEvent {
    /// Sample lifecycle events
    SampleCreated {
        sample_id: String,
        sample_type: String,
        patient_id: String,
        created_at: chrono::DateTime<chrono::Utc>,
    },
    SampleStateChanged {
        sample_id: String,
        from_state: String,
        to_state: String,
        changed_at: chrono::DateTime<chrono::Utc>,
        changed_by: String,
    },

    /// Storage events
    SampleStored {
        sample_id: String,
        storage_location: String,
        temperature_zone: String,
        stored_at: chrono::DateTime<chrono::Utc>,
    },
    SampleMoved {
        sample_id: String,
        from_location: String,
        to_location: String,
        moved_at: chrono::DateTime<chrono::Utc>,
        moved_by: String,
    },

    /// Capacity and monitoring events
    StorageCapacityAlert {
        location: String,
        current_utilization: f64,
        threshold_type: AlertThreshold,
        alert_at: chrono::DateTime<chrono::Utc>,
    },
    TemperatureAlert {
        location: String,
        current_temperature: f64,
        expected_temperature: f64,
        severity: AlertSeverity,
        alert_at: chrono::DateTime<chrono::Utc>,
    },

    /// Processing events
    DocumentProcessed {
        document_id: String,
        processing_result: ProcessingStatus,
        confidence_score: f64,
        processed_at: chrono::DateTime<chrono::Utc>,
    },

    /// Sequencing events
    SequencingJobStarted {
        job_id: String,
        sample_ids: Vec<String>,
        job_type: String,
        started_at: chrono::DateTime<chrono::Utc>,
    },
    SequencingJobCompleted {
        job_id: String,
        result_status: String,
        completed_at: chrono::DateTime<chrono::Utc>,
    },

    /// System events
    SystemHealthCheck {
        component: String,
        status: HealthStatus,
        checked_at: chrono::DateTime<chrono::Utc>,
    },
    AuditTrailEntry {
        entity_type: String,
        entity_id: String,
        action: String,
        actor: String,
        timestamp: chrono::DateTime<chrono::Utc>,
        metadata: HashMap<String, String>,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum AlertThreshold {
    Warning,  // >80%
    Critical, // >95%
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum AlertSeverity {
    Low,
    Medium,
    High,
    Critical,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ProcessingStatus {
    Success,
    Failed,
    RequiresReview,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum HealthStatus {
    Healthy,
    Warning,
    Critical,
    Unknown,
}

/// Event with metadata for processing
#[derive(Debug, Clone)]
pub struct EventEnvelope {
    pub event_id: String,
    pub event: LabEvent,
    pub source_component: String,
    pub correlation_id: Option<String>,
    pub retry_count: u32,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

/// Event processing result
#[derive(Debug, Clone)]
pub struct EventProcessingResult {
    pub event_id: String,
    pub success: bool,
    pub message: Option<String>,
    pub processing_duration: std::time::Duration,
    pub triggered_events: Vec<LabEvent>,
}

/// Event system component that manages laboratory event flows
pub struct EventSystemComponent {
    config: EventSystemConfig,
    event_history: Arc<Mutex<VecDeque<EventEnvelope>>>,
    event_sender: Option<broadcast::Sender<EventEnvelope>>,
    event_handlers: Arc<Mutex<HashMap<String, Box<dyn EventHandler + Send + Sync>>>>,
    metrics: Arc<Mutex<EventMetrics>>,
    event_count: u64,
    is_initialized: bool,
}

/// Trait for handling specific event types
pub trait EventHandler {
    fn can_handle(&self, event: &LabEvent) -> bool;
    fn handle(&self, envelope: &EventEnvelope) -> Result<Vec<LabEvent>, String>;
    fn handler_name(&self) -> &'static str;
}

/// Event processing metrics
#[derive(Debug, Default)]
pub struct EventMetrics {
    pub total_events_processed: u64,
    pub events_by_type: HashMap<String, u64>,
    pub processing_errors: u64,
    pub average_processing_time: std::time::Duration,
    pub last_processed: Option<chrono::DateTime<chrono::Utc>>,
}

impl EventSystemComponent {
    pub fn new(config: EventSystemConfig) -> Self {
        Self {
            config,
            event_history: Arc::new(Mutex::new(VecDeque::new())),
            event_sender: None,
            event_handlers: Arc::new(Mutex::new(HashMap::new())),
            metrics: Arc::new(Mutex::new(EventMetrics::default())),
            event_count: 0,
            is_initialized: false,
        }
    }

    /// Publish an event to the system
    pub async fn publish_event(
        &self,
        event: LabEvent,
        source_component: &str,
    ) -> Result<String, ComponentError> {
        if !self.is_initialized {
            return Err(ComponentError::InitializationFailed(
                "Event system not initialized".to_string(),
            ));
        }

        let event_id = self.generate_event_id();
        let envelope = EventEnvelope {
            event_id: event_id.clone(),
            event,
            source_component: source_component.to_string(),
            correlation_id: None,
            retry_count: 0,
            created_at: chrono::Utc::now(),
        };

        // Add to history
        self.add_to_history(envelope.clone()).await;

        // Broadcast to subscribers
        if let Some(sender) = &self.event_sender {
            if let Err(e) = sender.send(envelope) {
                tracing::warn!("Failed to broadcast event: {}", e);
            }
        }

        Ok(event_id)
    }

    /// Subscribe to events with a receiver
    pub fn subscribe(&self) -> Option<broadcast::Receiver<EventEnvelope>> {
        self.event_sender.as_ref().map(|sender| sender.subscribe())
    }

    /// Register an event handler
    pub fn register_handler<H>(&self, handler: H) -> Result<(), ComponentError>
    where
        H: EventHandler + Send + Sync + 'static,
    {
        let handler_name = handler.handler_name().to_string();
        let mut handlers = self.event_handlers.lock().unwrap();

        if handlers.contains_key(&handler_name) {
            return Err(ComponentError::AlreadyRegistered(format!(
                "Event handler: {}",
                handler_name
            )));
        }

        handlers.insert(handler_name.clone(), Box::new(handler));
        tracing::info!("Registered event handler: {}", handler_name);

        Ok(())
    }

    /// Process events with registered handlers
    pub async fn process_pending_events(
        &self,
    ) -> Result<Vec<EventProcessingResult>, ComponentError> {
        let mut results = Vec::new();
        let mut events_to_process = Vec::new();

        // Get events from history for processing
        {
            let mut history = self.event_history.lock().unwrap();
            let batch_size = std::cmp::min(self.config.batch_size, history.len());

            for _ in 0..batch_size {
                if let Some(envelope) = history.pop_front() {
                    events_to_process.push(envelope);
                }
            }
        }

        // Process each event
        for envelope in events_to_process {
            let start_time = std::time::Instant::now();
            let mut triggered_events = Vec::new();
            let mut success = true;
            let mut message = None;

            // Find and execute handlers
            {
                let handlers = self.event_handlers.lock().unwrap();
                for (handler_name, handler) in handlers.iter() {
                    if handler.can_handle(&envelope.event) {
                        match handler.handle(&envelope) {
                            Ok(mut new_events) => {
                                triggered_events.append(&mut new_events);
                                tracing::debug!(
                                    "Event {} processed by handler {}",
                                    envelope.event_id,
                                    handler_name
                                );
                            }
                            Err(e) => {
                                success = false;
                                message = Some(format!("Handler {} failed: {}", handler_name, e));
                                tracing::error!(
                                    "Handler {} failed to process event {}: {}",
                                    handler_name,
                                    envelope.event_id,
                                    e
                                );
                            }
                        }
                    }
                }
            }

            let processing_duration = start_time.elapsed();

            // Update metrics
            self.update_metrics(&envelope.event, processing_duration, success)
                .await;

            // Publish triggered events
            for triggered_event in &triggered_events {
                if let Err(e) = self
                    .publish_event(triggered_event.clone(), "event_system")
                    .await
                {
                    tracing::warn!("Failed to publish triggered event: {}", e);
                }
            }

            results.push(EventProcessingResult {
                event_id: envelope.event_id,
                success,
                message,
                processing_duration,
                triggered_events,
            });
        }

        Ok(results)
    }

    /// Get event processing metrics
    pub fn get_metrics(&self) -> EventMetrics {
        self.metrics.lock().unwrap().clone()
    }

    /// Get recent event history
    pub fn get_recent_events(&self, limit: usize) -> Vec<EventEnvelope> {
        let history = self.event_history.lock().unwrap();
        history.iter().rev().take(limit).cloned().collect()
    }

    /// Search events by criteria
    pub fn search_events(
        &self,
        event_type: Option<&str>,
        sample_id: Option<&str>,
    ) -> Vec<EventEnvelope> {
        let history = self.event_history.lock().unwrap();

        history
            .iter()
            .filter(|envelope| {
                // Filter by event type if specified
                if let Some(event_type_filter) = event_type {
                    let event_type_name = self.get_event_type_name(&envelope.event);
                    if !event_type_name.contains(event_type_filter) {
                        return false;
                    }
                }

                // Filter by sample ID if specified
                if let Some(sample_id_filter) = sample_id {
                    if !self.event_contains_sample_id(&envelope.event, sample_id_filter) {
                        return false;
                    }
                }

                true
            })
            .cloned()
            .collect()
    }

    // Private helper methods

    async fn add_to_history(&self, envelope: EventEnvelope) {
        let mut history = self.event_history.lock().unwrap();

        // Add to history
        history.push_back(envelope);

        // Trim history if it exceeds max size
        while history.len() > self.config.max_event_history {
            history.pop_front();
        }
    }

    async fn update_metrics(&self, event: &LabEvent, duration: std::time::Duration, success: bool) {
        if !self.config.enable_metrics {
            return;
        }

        let mut metrics = self.metrics.lock().unwrap();

        metrics.total_events_processed += 1;

        if !success {
            metrics.processing_errors += 1;
        }

        // Update event type counter
        let event_type = self.get_event_type_name(event);
        *metrics.events_by_type.entry(event_type).or_insert(0) += 1;

        // Update average processing time
        let total_events = metrics.total_events_processed as f64;
        let current_avg_ms = metrics.average_processing_time.as_millis() as f64;
        let new_duration_ms = duration.as_millis() as f64;

        let new_avg_ms = (current_avg_ms * (total_events - 1.0) + new_duration_ms) / total_events;
        metrics.average_processing_time = std::time::Duration::from_millis(new_avg_ms as u64);

        metrics.last_processed = Some(chrono::Utc::now());
    }

    fn generate_event_id(&self) -> String {
        use fastrand;
        let timestamp = chrono::Utc::now().timestamp_millis();
        let random = fastrand::u32(..);
        format!("evt_{}_{:08x}", timestamp, random)
    }

    fn get_event_type_name(&self, event: &LabEvent) -> String {
        match event {
            LabEvent::SampleCreated { .. } => "SampleCreated".to_string(),
            LabEvent::SampleStateChanged { .. } => "SampleStateChanged".to_string(),
            LabEvent::SampleStored { .. } => "SampleStored".to_string(),
            LabEvent::SampleMoved { .. } => "SampleMoved".to_string(),
            LabEvent::StorageCapacityAlert { .. } => "StorageCapacityAlert".to_string(),
            LabEvent::TemperatureAlert { .. } => "TemperatureAlert".to_string(),
            LabEvent::DocumentProcessed { .. } => "DocumentProcessed".to_string(),
            LabEvent::SequencingJobStarted { .. } => "SequencingJobStarted".to_string(),
            LabEvent::SequencingJobCompleted { .. } => "SequencingJobCompleted".to_string(),
            LabEvent::SystemHealthCheck { .. } => "SystemHealthCheck".to_string(),
            LabEvent::AuditTrailEntry { .. } => "AuditTrailEntry".to_string(),
        }
    }

    fn event_contains_sample_id(&self, event: &LabEvent, sample_id: &str) -> bool {
        match event {
            LabEvent::SampleCreated { sample_id: id, .. }
            | LabEvent::SampleStateChanged { sample_id: id, .. }
            | LabEvent::SampleStored { sample_id: id, .. }
            | LabEvent::SampleMoved { sample_id: id, .. } => id == sample_id,
            LabEvent::SequencingJobStarted { sample_ids, .. } => {
                sample_ids.contains(&sample_id.to_string())
            }
            _ => false,
        }
    }
}

#[async_trait]
impl Component for EventSystemComponent {
    fn component_id(&self) -> &'static str {
        "event_system"
    }

    fn component_name(&self) -> &'static str {
        "Laboratory Event System"
    }

    async fn initialize(&mut self, _context: &ServiceRegistry) -> Result<(), ComponentError> {
        if self.is_initialized {
            return Ok(());
        }

        tracing::info!("Initializing event system component");

        // Create broadcast channel for event distribution
        let (sender, _) = broadcast::channel(1000);
        self.event_sender = Some(sender);

        // Register default event handlers
        self.register_default_handlers()?;

        self.is_initialized = true;
        tracing::info!("Event system component initialized successfully");

        Ok(())
    }

    async fn health_check(&self) -> Result<(), ComponentError> {
        if !self.is_initialized {
            return Err(ComponentError::InitializationFailed(
                "Component not initialized".to_string(),
            ));
        }

        // Check if event channel is working
        if self.event_sender.is_none() {
            return Err(ComponentError::ServiceUnavailable(
                "Event broadcast channel not available".to_string(),
            ));
        }

        // Check metrics
        let metrics = self.get_metrics();
        if self.config.enable_metrics
            && metrics.processing_errors > metrics.total_events_processed / 2
        {
            return Err(ComponentError::ServiceUnavailable(
                "High error rate in event processing".to_string(),
            ));
        }

        Ok(())
    }

    async fn shutdown(&mut self) -> Result<(), ComponentError> {
        tracing::info!("Shutting down event system component");

        // Final metrics report
        let metrics = self.get_metrics();
        tracing::info!("Final event metrics: {:?}", metrics);

        // Clear resources
        self.event_sender = None;
        self.event_handlers.lock().unwrap().clear();
        self.event_history.lock().unwrap().clear();

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
impl ServiceProvider for EventSystemComponent {
    fn provided_services(&self) -> Vec<&'static str> {
        vec![
            "event_system",
            "event_publishing",
            "event_subscription",
            "audit_trail",
        ]
    }
}

impl Configurable for EventSystemComponent {
    type Config = EventSystemConfig;

    fn configure(&mut self, config: Self::Config) -> Result<(), ComponentError> {
        if self.is_initialized {
            return Err(ComponentError::ConfigurationError(
                "Cannot reconfigure initialized component".to_string(),
            ));
        }

        self.config = config;
        Ok(())
    }

    fn get_config(&self) -> &Self::Config {
        &self.config
    }
}

impl EventSystemComponent {
    fn register_default_handlers(&self) -> Result<(), ComponentError> {
        // Register default audit trail handler
        self.register_handler(AuditTrailHandler)?;

        // Register storage monitoring handler
        self.register_handler(StorageMonitoringHandler)?;

        // Register sample lifecycle handler
        self.register_handler(SampleLifecycleHandler)?;

        Ok(())
    }
}

/// Default event handlers

/// Handles audit trail generation for all events
struct AuditTrailHandler;

impl EventHandler for AuditTrailHandler {
    fn can_handle(&self, _event: &LabEvent) -> bool {
        true // Handle all events for audit trail
    }

    fn handle(&self, envelope: &EventEnvelope) -> Result<Vec<LabEvent>, String> {
        // Generate audit trail entry for the event
        let audit_entry = LabEvent::AuditTrailEntry {
            entity_type: "Event".to_string(),
            entity_id: envelope.event_id.clone(),
            action: format!(
                "Event processed: {}",
                std::mem::discriminant(&envelope.event)
            ),
            actor: envelope.source_component.clone(),
            timestamp: chrono::Utc::now(),
            metadata: HashMap::new(),
        };

        Ok(vec![audit_entry])
    }

    fn handler_name(&self) -> &'static str {
        "audit_trail_handler"
    }
}

/// Handles storage-related events and monitoring
struct StorageMonitoringHandler;

impl EventHandler for StorageMonitoringHandler {
    fn can_handle(&self, event: &LabEvent) -> bool {
        matches!(
            event,
            LabEvent::SampleStored { .. }
                | LabEvent::SampleMoved { .. }
                | LabEvent::StorageCapacityAlert { .. }
                | LabEvent::TemperatureAlert { .. }
        )
    }

    fn handle(&self, envelope: &EventEnvelope) -> Result<Vec<LabEvent>, String> {
        match &envelope.event {
            LabEvent::SampleStored {
                storage_location, ..
            } => {
                // Check capacity after storing sample
                // This would trigger capacity monitoring in a real implementation
                tracing::info!("Storage monitoring: Sample stored in {}", storage_location);
            }
            LabEvent::StorageCapacityAlert {
                location,
                current_utilization,
                threshold_type,
                ..
            } => {
                // Handle capacity alerts
                match threshold_type {
                    AlertThreshold::Critical => {
                        tracing::error!(
                            "CRITICAL: Storage {} at {:.1}% capacity",
                            location,
                            current_utilization
                        );
                    }
                    AlertThreshold::Warning => {
                        tracing::warn!(
                            "WARNING: Storage {} at {:.1}% capacity",
                            location,
                            current_utilization
                        );
                    }
                }
            }
            _ => {}
        }

        Ok(vec![]) // No additional events triggered by default
    }

    fn handler_name(&self) -> &'static str {
        "storage_monitoring_handler"
    }
}

/// Handles sample lifecycle events
struct SampleLifecycleHandler;

impl EventHandler for SampleLifecycleHandler {
    fn can_handle(&self, event: &LabEvent) -> bool {
        matches!(
            event,
            LabEvent::SampleCreated { .. } | LabEvent::SampleStateChanged { .. }
        )
    }

    fn handle(&self, envelope: &EventEnvelope) -> Result<Vec<LabEvent>, String> {
        match &envelope.event {
            LabEvent::SampleStateChanged {
                sample_id,
                to_state,
                ..
            } => {
                // Trigger appropriate downstream events based on state change
                match to_state.as_str() {
                    "InStorage" => {
                        // Sample is now in storage - could trigger capacity check
                        tracing::info!(
                            "Sample {} moved to storage, triggering capacity check",
                            sample_id
                        );
                    }
                    "InSequencing" => {
                        // Sample moved to sequencing - could trigger sequencing workflow
                        tracing::info!("Sample {} moved to sequencing workflow", sample_id);
                    }
                    "Completed" => {
                        // Sample processing completed - could trigger result notification
                        tracing::info!("Sample {} processing completed", sample_id);
                    }
                    _ => {}
                }
            }
            _ => {}
        }

        Ok(vec![])
    }

    fn handler_name(&self) -> &'static str {
        "sample_lifecycle_handler"
    }
}

/// Builder for event system components
pub struct EventSystemBuilder {
    config: EventSystemConfig,
}

impl EventSystemBuilder {
    pub fn new() -> Self {
        Self {
            config: EventSystemConfig::default(),
        }
    }

    pub fn with_history_size(mut self, size: usize) -> Self {
        self.config.max_event_history = size;
        self
    }

    pub fn with_persistence(mut self, enabled: bool) -> Self {
        self.config.enable_persistence = enabled;
        self
    }

    pub fn with_batch_size(mut self, size: usize) -> Self {
        self.config.batch_size = size;
        self
    }

    pub fn with_metrics(mut self, enabled: bool) -> Self {
        self.config.enable_metrics = enabled;
        self
    }

    pub fn for_high_throughput(mut self) -> Self {
        self.config.max_event_history = 50000;
        self.config.batch_size = 500;
        self.config.processing_timeout_ms = 1000;
        self
    }

    pub fn build(self) -> EventSystemComponent {
        EventSystemComponent::new(self.config)
    }
}

impl Default for EventSystemBuilder {
    fn default() -> Self {
        Self::new()
    }
}
