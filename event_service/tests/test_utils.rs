use event_service::{events::*, test_utils::*, services::*, Config};
use fake::{Fake, Faker};
use serde_json::Value;
use std::collections::HashMap;
use std::sync::Arc;
use uuid::Uuid;
use chrono::Utc;

/// Test environment for event service testing
pub struct TestEventEnvironment {
    pub event_service: Arc<EventService>,
    pub config: Arc<Config>,
    pub created_events: Vec<Uuid>,
    pub created_subscriptions: Vec<String>,
}

impl TestEventEnvironment {
    pub async fn new() -> Self {
        let event_service = get_test_event_service().await;
        let config = Config::test_config();
        
        Self {
            event_service: Arc::new(event_service.clone()),
            config: Arc::new(config),
            created_events: Vec::new(),
            created_subscriptions: Vec::new(),
        }
    }

    pub async fn cleanup(&mut self) {
        // Cleanup subscriptions
        for subscription_name in &self.created_subscriptions {
            let _ = self.event_service.unsubscribe(subscription_name).await;
        }
        
        // Cleanup Redis test database
        cleanup_test_events().await;
        
        self.created_events.clear();
        self.created_subscriptions.clear();
    }

    pub fn track_event(&mut self, event_id: Uuid) {
        self.created_events.push(event_id);
    }

    pub fn track_subscription(&mut self, subscription_name: String) {
        self.created_subscriptions.push(subscription_name);
    }
}

/// Factory for creating test events
pub struct EventFactory;

impl EventFactory {
    pub fn create_sample_event() -> Event {
        Event::new(
            "sample.created".to_string(),
            "sample-service".to_string(),
            serde_json::json!({
                "sample_id": Uuid::new_v4(),
                "sample_type": "DNA",
                "barcode": "TEST-001",
                "status": "Pending"
            }),
        )
        .with_subject(format!("sample-{}", Uuid::new_v4()))
        .with_priority(2)
    }

    pub fn create_auth_event() -> Event {
        Event::new(
            "user.login".to_string(),
            "auth-service".to_string(),
            serde_json::json!({
                "user_id": Uuid::new_v4(),
                "email": "test@example.com",
                "login_time": Utc::now(),
                "ip_address": "192.168.1.100"
            }),
        )
        .with_subject("user-authentication".to_string())
        .with_priority(1)
    }

    pub fn create_storage_event() -> Event {
        Event::new(
            "storage.sample_moved".to_string(),
            "storage-service".to_string(),
            serde_json::json!({
                "sample_id": Uuid::new_v4(),
                "from_location": "A1-01",
                "to_location": "B2-05",
                "temperature": -80,
                "moved_by": "robot-01"
            }),
        )
        .with_subject("sample-movement".to_string())
        .with_priority(3)
    }

    pub fn create_transaction_event() -> Event {
        Event::new(
            "transaction.started".to_string(),
            "transaction-service".to_string(),
            serde_json::json!({
                "transaction_id": Uuid::new_v4(),
                "saga_id": Uuid::new_v4(),
                "transaction_type": "sample_processing",
                "user_id": Uuid::new_v4()
            }),
        )
        .with_subject("transaction-processing".to_string())
        .with_priority(2)
    }

    pub fn create_rag_event() -> Event {
        Event::new(
            "document.processed".to_string(),
            "rag-service".to_string(),
            serde_json::json!({
                "document_id": Uuid::new_v4(),
                "document_type": "lab_submission",
                "extraction_confidence": 0.92,
                "extracted_samples": 5,
                "processing_time_ms": 2500
            }),
        )
        .with_subject("document-processing".to_string())
        .with_priority(2)
    }

    pub fn create_batch_events(count: usize, event_type: &str) -> Vec<Event> {
        (0..count)
            .map(|i| {
                Event::new(
                    format!("{}.batch_{}", event_type, i),
                    "test-service".to_string(),
                    serde_json::json!({
                        "batch_index": i,
                        "total_batch_size": count,
                        "timestamp": Utc::now()
                    }),
                )
                .with_subject(format!("batch-{}", i))
                .with_priority(3)
            })
            .collect()
    }

    pub fn create_high_priority_event() -> Event {
        Event::new(
            "alert.critical".to_string(),
            "monitoring-service".to_string(),
            serde_json::json!({
                "alert_type": "temperature_breach",
                "location": "freezer-A1",
                "current_temp": -75.2,
                "threshold_temp": -78.0,
                "severity": "critical"
            }),
        )
        .with_subject("critical-alert".to_string())
        .with_priority(1) // Highest priority
    }
}

/// Factory for creating subscription configurations
pub struct SubscriptionFactory;

impl SubscriptionFactory {
    pub fn create_sample_subscription() -> SubscriptionConfig {
        SubscriptionConfig {
            name: format!("sample-subscriber-{}", Uuid::new_v4()),
            event_types: vec!["sample.*".to_string()],
            consumer_group: "sample-processors".to_string(),
            consumer_name: "test-consumer".to_string(),
            batch_size: 10,
            timeout_ms: 5000,
            auto_ack: true,
            read_latest: false,
        }
    }

    pub fn create_all_events_subscription() -> SubscriptionConfig {
        SubscriptionConfig {
            name: format!("all-events-{}", Uuid::new_v4()),
            event_types: vec!["*".to_string()],
            consumer_group: "monitor-group".to_string(),
            consumer_name: "monitor-consumer".to_string(),
            batch_size: 50,
            timeout_ms: 10000,
            auto_ack: false,
            read_latest: true,
        }
    }

    pub fn create_high_priority_subscription() -> SubscriptionConfig {
        SubscriptionConfig {
            name: format!("high-priority-{}", Uuid::new_v4()),
            event_types: vec!["alert.*".to_string(), "transaction.failed".to_string()],
            consumer_group: "priority-handlers".to_string(),
            consumer_name: "priority-consumer".to_string(),
            batch_size: 5,
            timeout_ms: 1000,
            auto_ack: true,
            read_latest: true,
        }
    }

    pub fn create_cross_service_subscription() -> SubscriptionConfig {
        SubscriptionConfig {
            name: format!("cross-service-{}", Uuid::new_v4()),
            event_types: vec![
                "sample.*".to_string(),
                "user.*".to_string(),
                "storage.*".to_string(),
                "transaction.*".to_string()
            ],
            consumer_group: "integration-tests".to_string(),
            consumer_name: "integration-consumer".to_string(),
            batch_size: 20,
            timeout_ms: 15000,
            auto_ack: false,
            read_latest: false,
        }
    }
}

/// Test event handler for capturing processed events
pub struct TestEventHandler {
    pub name: String,
    pub event_types: Vec<String>,
    pub processed_events: Arc<tokio::sync::Mutex<Vec<EventContext>>>,
    pub should_fail: bool,
}

impl TestEventHandler {
    pub fn new(name: String, event_types: Vec<String>) -> Self {
        Self {
            name,
            event_types,
            processed_events: Arc::new(tokio::sync::Mutex::new(Vec::new())),
            should_fail: false,
        }
    }

    pub fn new_failing(name: String, event_types: Vec<String>) -> Self {
        Self {
            name,
            event_types,
            processed_events: Arc::new(tokio::sync::Mutex::new(Vec::new())),
            should_fail: true,
        }
    }

    pub async fn get_processed_events(&self) -> Vec<EventContext> {
        let events = self.processed_events.lock().await;
        events.clone()
    }

    pub async fn clear_processed_events(&self) {
        let mut events = self.processed_events.lock().await;
        events.clear();
    }
}

#[async_trait::async_trait]
impl EventHandler for TestEventHandler {
    async fn handle(&self, context: EventContext) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        if self.should_fail {
            return Err("Simulated handler failure".into());
        }

        let mut events = self.processed_events.lock().await;
        events.push(context);
        Ok(())
    }

    fn event_types(&self) -> Vec<String> {
        self.event_types.clone()
    }

    fn name(&self) -> String {
        self.name.clone()
    }
}

/// Event filter testing utilities
pub struct FilterTestUtils;

impl FilterTestUtils {
    pub fn create_sample_filter() -> EventFilter {
        EventFilter {
            event_types: vec!["sample.*".to_string()],
            source_services: vec!["sample-service".to_string()],
            metadata_filters: HashMap::new(),
            subject_patterns: vec!["sample-*".to_string()],
            priority_range: Some((1, 3)),
        }
    }

    pub fn create_priority_filter(min_priority: u8, max_priority: u8) -> EventFilter {
        EventFilter {
            event_types: vec!["*".to_string()],
            source_services: vec![],
            metadata_filters: HashMap::new(),
            subject_patterns: vec![],
            priority_range: Some((min_priority, max_priority)),
        }
    }

    pub fn create_service_filter(services: Vec<String>) -> EventFilter {
        EventFilter {
            event_types: vec!["*".to_string()],
            source_services: services,
            metadata_filters: HashMap::new(),
            subject_patterns: vec![],
            priority_range: None,
        }
    }
}

/// Performance testing utilities
pub struct EventPerformanceUtils;

impl EventPerformanceUtils {
    pub async fn measure_event_publication_time(
        event_service: &EventService,
        event: &Event,
    ) -> std::time::Duration {
        let start = std::time::Instant::now();
        let _ = event_service.publish_event(event.clone()).await;
        start.elapsed()
    }

    pub async fn measure_batch_publication_time(
        event_service: &EventService,
        events: &[Event],
    ) -> std::time::Duration {
        let start = std::time::Instant::now();
        for event in events {
            let _ = event_service.publish_event(event.clone()).await;
        }
        start.elapsed()
    }

    pub async fn concurrent_event_publication(
        event_service: &EventService,
        events: Vec<Event>,
    ) -> Vec<Result<EventPublicationResult, String>> {
        let tasks: Vec<_> = events
            .into_iter()
            .map(|event| {
                let service = event_service.clone();
                tokio::spawn(async move {
                    service.publish_event(event).await
                        .map_err(|e| e.to_string())
                })
            })
            .collect();

        futures::future::join_all(tasks)
            .await
            .into_iter()
            .map(|result| result.unwrap_or_else(|e| Err(e.to_string())))
            .collect()
    }

    pub async fn measure_subscription_processing_time(
        event_service: &EventService,
        subscription: &SubscriptionConfig,
        event_count: usize,
    ) -> std::time::Duration {
        let start = std::time::Instant::now();
        
        // Subscribe to events
        let _ = event_service.subscribe(subscription.clone()).await;
        
        // Publish test events
        for i in 0..event_count {
            let event = Event::new(
                "performance.test".to_string(),
                "performance-service".to_string(),
                serde_json::json!({ "index": i }),
            );
            let _ = event_service.publish_event(event).await;
        }
        
        // Simulate processing time
        tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
        
        start.elapsed()
    }
}

/// Assertions for event testing
pub struct EventAssertions;

impl EventAssertions {
    pub fn assert_event_published(result: &Result<EventPublicationResult, String>) {
        assert!(result.is_ok(), "Event should be published successfully");
    }

    pub fn assert_event_received(events: &[EventContext], expected_event_type: &str) {
        assert!(
            events.iter().any(|ctx| ctx.event.event_type == expected_event_type),
            "Expected event type '{}' should be received",
            expected_event_type
        );
    }

    pub fn assert_event_count(events: &[EventContext], expected_count: usize) {
        assert_eq!(
            events.len(),
            expected_count,
            "Expected {} events, but got {}",
            expected_count,
            events.len()
        );
    }

    pub fn assert_event_order(events: &[EventContext]) {
        for window in events.windows(2) {
            assert!(
                window[0].event.timestamp <= window[1].event.timestamp,
                "Events should be in chronological order"
            );
        }
    }

    pub fn assert_filter_matches(filter: &EventFilter, event: &Event, should_match: bool) {
        assert_eq!(
            filter.matches(event),
            should_match,
            "Filter matching failed for event type: {}",
            event.event_type
        );
    }

    pub fn assert_publication_performance(duration: std::time::Duration, max_ms: u64) {
        assert!(
            duration.as_millis() <= max_ms as u128,
            "Event publication took {}ms, expected <= {}ms",
            duration.as_millis(),
            max_ms
        );
    }
} 
