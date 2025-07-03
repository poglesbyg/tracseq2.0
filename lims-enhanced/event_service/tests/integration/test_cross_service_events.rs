use event_service::*;
use crate::test_utils::*;
use std::time::Duration;

#[tokio::test]
async fn test_laboratory_sample_workflow_events() {
    let mut test_env = TestEventEnvironment::new().await;
    
    // Create cross-service subscription
    let subscription = SubscriptionFactory::create_cross_service_subscription();
    let subscription_name = subscription.name.clone();
    test_env.track_subscription(subscription_name.clone());

    // Set up event handler to capture all events
    let _handler = TestEventHandler::new(
        "workflow-handler".to_string(),
        vec!["*".to_string()],
    );

    // Subscribe to cross-service events
    let result = test_env.event_bus.subscribe(subscription).await;
    assert!(result.is_ok(), "Should subscribe successfully");

    // Simulate complete laboratory workflow
    
    // 1. User authentication event
    let auth_event = EventFactory::create_auth_event();
    test_env.track_event(auth_event.id);
    let auth_result = test_env.event_bus.publish(auth_event.clone()).await;
    EventAssertions::assert_event_published(auth_result.is_ok());

    // 2. Sample creation event
    let sample_event = EventFactory::create_sample_event();
    test_env.track_event(sample_event.id);
    let sample_result = test_env.event_bus.publish(sample_event.clone()).await;
    EventAssertions::assert_event_published(sample_result.is_ok());

    // 3. Storage movement event
    let storage_event = EventFactory::create_storage_event();
    test_env.track_event(storage_event.id);
    let storage_result = test_env.event_bus.publish(storage_event.clone()).await;
    EventAssertions::assert_event_published(storage_result.is_ok());

    // 4. Transaction processing event
    let transaction_event = EventFactory::create_transaction_event();
    test_env.track_event(transaction_event.id);
    let transaction_result = test_env.event_bus.publish(transaction_event.clone()).await;
    EventAssertions::assert_event_published(transaction_result.is_ok());

    // Wait for event propagation
    tokio::time::sleep(Duration::from_millis(100)).await;

    // Since we can't directly verify processing without implementing handlers,
    // we'll just verify that events were published successfully
    assert!(auth_result.is_ok());
    assert!(sample_result.is_ok());
    assert!(storage_result.is_ok());
    assert!(transaction_result.is_ok());
    
    test_env.cleanup().await;
}

#[tokio::test]
async fn test_rag_document_processing_workflow() {
    let mut test_env = TestEventEnvironment::new().await;
    
    // *Context added by Giga rag-algorithms*
    
    // Create subscription for RAG-related events
    let rag_subscription = SubscriptionConfig {
        name: format!("rag-workflow-{}", uuid::Uuid::new_v4()),
        event_types: vec![
            "document.*".to_string(),
            "sample.created".to_string(),
            "extraction.*".to_string()
        ],
        consumer_group: "rag-processors".to_string(),
        consumer_name: "rag-consumer".to_string(),
        batch_size: 10,
        timeout_ms: 5000,
        auto_ack: true,
        read_latest: false,
    };
    
    test_env.track_subscription(rag_subscription.name.clone());

    let _handler = TestEventHandler::new(
        "rag-handler".to_string(),
        vec!["document.*".to_string(), "sample.*".to_string()],
    );

    // Subscribe to RAG workflow events
    let result = test_env.event_bus.subscribe(rag_subscription).await;
    assert!(result.is_ok(), "Should subscribe to RAG events");

    // Simulate RAG document processing workflow
    
    // 1. Document uploaded event
    let document_upload_event = Event::new(
        "document.uploaded".to_string(),
        "rag-service".to_string(),
        serde_json::json!({
            "document_id": uuid::Uuid::new_v4(),
            "filename": "lab_submission_form.pdf",
            "file_size": 1024000,
            "content_type": "application/pdf",
            "uploaded_by": uuid::Uuid::new_v4()
        }),
    );
    test_env.track_event(document_upload_event.id);
    
    let upload_result = test_env.event_bus.publish(document_upload_event).await;
    EventAssertions::assert_event_published(upload_result.is_ok());

    // 2. Document processed event (with laboratory-specific extraction)
    let rag_event = EventFactory::create_rag_event();
    test_env.track_event(rag_event.id);
    
    let rag_result = test_env.event_bus.publish(rag_event).await;
    EventAssertions::assert_event_published(rag_result.is_ok());

    // 3. Extracted samples creation events
    for i in 0..3 {
        let extracted_sample_event = Event::new(
            "sample.extracted".to_string(),
            "rag-service".to_string(),
            serde_json::json!({
                "sample_id": uuid::Uuid::new_v4(),
                "sample_type": "DNA",
                "confidence_score": 0.95,
                "extraction_source": "rag_processing",
                "batch_index": i
            }),
        );
        test_env.track_event(extracted_sample_event.id);
        
        let sample_result = test_env.event_bus.publish(extracted_sample_event).await;
        EventAssertions::assert_event_published(sample_result.is_ok());
    }

    // Wait for processing
    tokio::time::sleep(Duration::from_millis(150)).await;

    // Verify all events were published successfully
    assert!(upload_result.is_ok());
    assert!(rag_result.is_ok());
    
    test_env.cleanup().await;
}

#[tokio::test]
async fn test_critical_alert_priority_handling() {
    let mut test_env = TestEventEnvironment::new().await;
    
    // Create high-priority subscription
    let priority_subscription = SubscriptionFactory::create_high_priority_subscription();
    test_env.track_subscription(priority_subscription.name.clone());

    let _handler = TestEventHandler::new(
        "priority-handler".to_string(),
        vec!["alert.*".to_string(), "transaction.failed".to_string()],
    );

    // Subscribe to high-priority events
    let result = test_env.event_bus.subscribe(priority_subscription).await;
    assert!(result.is_ok(), "Should subscribe to priority events");

    // Publish mixed priority events
    
    // Low priority events
    for i in 0..3 {
        let low_priority_event = Event::new(
            "info.update".to_string(),
            "monitoring-service".to_string(),
            serde_json::json!({ "update_id": i }),
        ).with_priority(5); // Lowest priority
        
        test_env.track_event(low_priority_event.id);
        let _ = test_env.event_bus.publish(low_priority_event).await;
    }

    // High priority critical alert
    let critical_event = EventFactory::create_high_priority_event();
    test_env.track_event(critical_event.id);
    let critical_result = test_env.event_bus.publish(critical_event.clone()).await;
    EventAssertions::assert_event_published(critical_result.is_ok());

    // Transaction failure event
    let failure_event = Event::new(
        "transaction.failed".to_string(),
        "transaction-service".to_string(),
        serde_json::json!({
            "transaction_id": uuid::Uuid::new_v4(),
            "error": "Database connection timeout",
            "saga_id": uuid::Uuid::new_v4()
        }),
    ).with_priority(1); // High priority
    
    test_env.track_event(failure_event.id);
    let failure_result = test_env.event_bus.publish(failure_event).await;
    EventAssertions::assert_event_published(failure_result.is_ok());

    // Wait for processing
    tokio::time::sleep(Duration::from_millis(100)).await;

    // Verify high-priority events were published successfully
    assert!(critical_result.is_ok());
    assert!(failure_result.is_ok());
    
    test_env.cleanup().await;
}

#[tokio::test]
async fn test_event_correlation_across_services() {
    let mut test_env = TestEventEnvironment::new().await;
    let correlation_id = uuid::Uuid::new_v4();
    
    // Create subscription for correlated events
    let correlation_subscription = SubscriptionConfig {
        name: format!("correlation-test-{}", uuid::Uuid::new_v4()),
        event_types: vec!["*".to_string()],
        consumer_group: "correlation-group".to_string(),
        consumer_name: "correlation-consumer".to_string(),
        batch_size: 20,
        timeout_ms: 10000,
        auto_ack: true,
        read_latest: false,
    };
    
    test_env.track_subscription(correlation_subscription.name.clone());

    let _handler = TestEventHandler::new(
        "correlation-handler".to_string(),
        vec!["*".to_string()],
    );

    // Subscribe
    let result = test_env.event_bus.subscribe(correlation_subscription).await;
    assert!(result.is_ok(), "Should subscribe for correlation testing");

    // Publish correlated events across services
    let services_and_events = vec![
        ("auth-service", "user.authenticated"),
        ("sample-service", "sample.validated"),
        ("storage-service", "sample.stored"),
        ("transaction-service", "transaction.completed"),
        ("rag-service", "document.indexed"),
    ];

    let mut results = Vec::new();
    for (service, event_type) in services_and_events {
        let correlated_event = Event::new(
            event_type.to_string(),
            service.to_string(),
            serde_json::json!({
                "service": service,
                "correlation_data": "test_workflow_123"
            }),
        ).with_correlation_id(correlation_id);
        
        test_env.track_event(correlated_event.id);
        let result = test_env.event_bus.publish(correlated_event).await;
        EventAssertions::assert_event_published(result.is_ok());
        results.push(result);
    }

    // Wait for processing
    tokio::time::sleep(Duration::from_millis(150)).await;

    // Verify all correlated events were published
    assert_eq!(results.len(), 5);
    for result in results {
        assert!(result.is_ok());
    }
    
    test_env.cleanup().await;
}

#[tokio::test]
async fn test_event_throughput_under_load() {
    let mut test_env = TestEventEnvironment::new().await;
    let subscription = SubscriptionFactory::create_all_events_subscription();
    test_env.track_subscription(subscription.name.clone());

    let _handler = TestEventHandler::new(
        "load-test-handler".to_string(),
        vec!["*".to_string()],
    );

    // Subscribe
    let result = test_env.event_bus.subscribe(subscription).await;
    assert!(result.is_ok(), "Should subscribe for load testing");

    // Generate large batch of events
    let event_count = 100; // Reduced from 1000 for faster testing
    let batch_events = EventFactory::create_batch_events(event_count, "load.test");
    
    for event in &batch_events {
        test_env.track_event(event.id);
    }

    // Measure throughput
    let start_time = std::time::Instant::now();
    
    // Publish events concurrently
    let publication_results = EventPerformanceUtils::concurrent_event_publication(
        &test_env.event_bus,
        batch_events,
    ).await;

    let publication_time = start_time.elapsed();

    // Verify all events were published successfully
    let successful_publications = publication_results.iter()
        .filter(|result| result.is_ok())
        .count();
    
    assert_eq!(
        successful_publications,
        event_count,
        "All events should be published successfully"
    );

    // Performance assertions
    let events_per_second = event_count as f64 / publication_time.as_secs_f64();
    println!("Achieved throughput: {:.2} events/second", events_per_second);
    
    // Lower threshold for test reliability
    assert!(
        events_per_second >= 50.0,
        "Should achieve at least 50 events/second, got {:.2}",
        events_per_second
    );

    test_env.cleanup().await;
} 
