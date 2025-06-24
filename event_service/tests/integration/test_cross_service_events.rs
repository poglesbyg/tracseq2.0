use event_service::*;
use crate::test_utils::*;
use std::time::Duration;
use tokio::time::timeout;

#[test_with_event_cleanup]
async fn test_laboratory_sample_workflow_events(test_env: &mut TestEventEnvironment) {
    // Create cross-service subscription
    let subscription = SubscriptionFactory::create_cross_service_subscription();
    let subscription_name = subscription.name.clone();
    test_env.track_subscription(subscription_name.clone());

    // Set up event handler to capture all events
    let handler = TestEventHandler::new(
        "workflow-handler".to_string(),
        vec!["*".to_string()],
    );

    // Subscribe to cross-service events
    let result = test_env.event_service.subscribe(subscription).await;
    assert!(result.is_ok(), "Should subscribe successfully");

    // Simulate complete laboratory workflow
    
    // 1. User authentication event
    let auth_event = EventFactory::create_auth_event();
    test_env.track_event(auth_event.id);
    let auth_result = test_env.event_service.publish_event(auth_event.clone()).await;
    EventAssertions::assert_event_published(&auth_result.map_err(|e| e.to_string()));

    // 2. Sample creation event
    let sample_event = EventFactory::create_sample_event();
    test_env.track_event(sample_event.id);
    let sample_result = test_env.event_service.publish_event(sample_event.clone()).await;
    EventAssertions::assert_event_published(&sample_result.map_err(|e| e.to_string()));

    // 3. Storage movement event
    let storage_event = EventFactory::create_storage_event();
    test_env.track_event(storage_event.id);
    let storage_result = test_env.event_service.publish_event(storage_event.clone()).await;
    EventAssertions::assert_event_published(&storage_result.map_err(|e| e.to_string()));

    // 4. Transaction processing event
    let transaction_event = EventFactory::create_transaction_event();
    test_env.track_event(transaction_event.id);
    let transaction_result = test_env.event_service.publish_event(transaction_event.clone()).await;
    EventAssertions::assert_event_published(&transaction_result.map_err(|e| e.to_string()));

    // Wait for event propagation
    tokio::time::sleep(Duration::from_millis(100)).await;

    // Verify events were processed in sequence
    let processed_events = handler.get_processed_events().await;
    EventAssertions::assert_event_count(&processed_events, 4);

    // Verify event types in workflow
    let event_types: Vec<_> = processed_events.iter()
        .map(|ctx| &ctx.event.event_type)
        .collect();
    
    assert!(event_types.contains(&&"user.login".to_string()));
    assert!(event_types.contains(&&"sample.created".to_string()));
    assert!(event_types.contains(&&"storage.sample_moved".to_string()));
    assert!(event_types.contains(&&"transaction.started".to_string()));
}

#[test_with_event_cleanup]
async fn test_rag_document_processing_workflow(test_env: &mut TestEventEnvironment) {
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

    let handler = TestEventHandler::new(
        "rag-handler".to_string(),
        vec!["document.*".to_string(), "sample.*".to_string()],
    );

    // Subscribe to RAG workflow events
    let result = test_env.event_service.subscribe(rag_subscription).await;
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
    
    let upload_result = test_env.event_service.publish_event(document_upload_event).await;
    EventAssertions::assert_event_published(&upload_result.map_err(|e| e.to_string()));

    // 2. Document processed event (with laboratory-specific extraction)
    let rag_event = EventFactory::create_rag_event();
    test_env.track_event(rag_event.id);
    
    let rag_result = test_env.event_service.publish_event(rag_event).await;
    EventAssertions::assert_event_published(&rag_result.map_err(|e| e.to_string()));

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
        
        let sample_result = test_env.event_service.publish_event(extracted_sample_event).await;
        EventAssertions::assert_event_published(&sample_result.map_err(|e| e.to_string()));
    }

    // Wait for processing
    tokio::time::sleep(Duration::from_millis(150)).await;

    // Verify RAG workflow events
    let processed_events = handler.get_processed_events().await;
    EventAssertions::assert_event_count(&processed_events, 5); // 1 upload + 1 process + 3 extracted samples

    // Verify RAG-specific event content
    let rag_processed_events: Vec<_> = processed_events.iter()
        .filter(|ctx| ctx.event.source_service == "rag-service")
        .collect();
    
    assert!(rag_processed_events.len() >= 4, "Should have RAG service events");
}

#[test_with_event_cleanup]
async fn test_critical_alert_priority_handling(test_env: &mut TestEventEnvironment) {
    // Create high-priority subscription
    let priority_subscription = SubscriptionFactory::create_high_priority_subscription();
    test_env.track_subscription(priority_subscription.name.clone());

    let handler = TestEventHandler::new(
        "priority-handler".to_string(),
        vec!["alert.*".to_string(), "transaction.failed".to_string()],
    );

    // Subscribe to high-priority events
    let result = test_env.event_service.subscribe(priority_subscription).await;
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
        let _ = test_env.event_service.publish_event(low_priority_event).await;
    }

    // High priority critical alert
    let critical_event = EventFactory::create_high_priority_event();
    test_env.track_event(critical_event.id);
    let critical_result = test_env.event_service.publish_event(critical_event.clone()).await;
    EventAssertions::assert_event_published(&critical_result.map_err(|e| e.to_string()));

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
    let failure_result = test_env.event_service.publish_event(failure_event).await;
    EventAssertions::assert_event_published(&failure_result.map_err(|e| e.to_string()));

    // Wait for processing
    tokio::time::sleep(Duration::from_millis(100)).await;

    // Verify only high-priority events were processed
    let processed_events = handler.get_processed_events().await;
    EventAssertions::assert_event_count(&processed_events, 2); // Only critical alert and transaction failure

    // Verify priority handling
    for event_ctx in &processed_events {
        assert!(
            event_ctx.event.priority <= 2,
            "Only high-priority events should be processed"
        );
    }
}

#[test_with_event_cleanup]
async fn test_event_correlation_across_services(test_env: &mut TestEventEnvironment) {
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

    let handler = TestEventHandler::new(
        "correlation-handler".to_string(),
        vec!["*".to_string()],
    );

    // Subscribe
    let result = test_env.event_service.subscribe(correlation_subscription).await;
    assert!(result.is_ok(), "Should subscribe for correlation testing");

    // Publish correlated events across services
    let services_and_events = vec![
        ("auth-service", "user.authenticated"),
        ("sample-service", "sample.validated"),
        ("storage-service", "sample.stored"),
        ("transaction-service", "transaction.completed"),
        ("rag-service", "document.indexed"),
    ];

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
        let result = test_env.event_service.publish_event(correlated_event).await;
        EventAssertions::assert_event_published(&result.map_err(|e| e.to_string()));
    }

    // Wait for processing
    tokio::time::sleep(Duration::from_millis(150)).await;

    // Verify correlation
    let processed_events = handler.get_processed_events().await;
    EventAssertions::assert_event_count(&processed_events, 5);

    // Verify all events have the same correlation ID
    for event_ctx in &processed_events {
        assert_eq!(
            event_ctx.event.correlation_id,
            Some(correlation_id),
            "All events should have the same correlation ID"
        );
    }

    // Verify events from different services
    let services: std::collections::HashSet<_> = processed_events.iter()
        .map(|ctx| &ctx.event.source_service)
        .collect();
    
    assert_eq!(services.len(), 5, "Should have events from 5 different services");
}

#[test_with_event_cleanup]
async fn test_event_throughput_under_load(test_env: &mut TestEventEnvironment) {
    let subscription = SubscriptionFactory::create_all_events_subscription();
    test_env.track_subscription(subscription.name.clone());

    let handler = TestEventHandler::new(
        "load-test-handler".to_string(),
        vec!["*".to_string()],
    );

    // Subscribe
    let result = test_env.event_service.subscribe(subscription).await;
    assert!(result.is_ok(), "Should subscribe for load testing");

    // Generate large batch of events
    let event_count = 1000;
    let batch_events = EventFactory::create_batch_events(event_count, "load.test");
    
    for event in &batch_events {
        test_env.track_event(event.id);
    }

    // Measure throughput
    let start_time = std::time::Instant::now();
    
    // Publish events concurrently
    let publication_results = EventPerformanceUtils::concurrent_event_publication(
        &test_env.event_service,
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
    assert!(
        events_per_second >= 100.0,
        "Should achieve at least 100 events/second, got {:.2}",
        events_per_second
    );

    // Wait for event processing
    let timeout_duration = Duration::from_secs(10);
    let processing_result = timeout(timeout_duration, async {
        loop {
            let processed_count = handler.get_processed_events().await.len();
            if processed_count >= event_count {
                break;
            }
            tokio::time::sleep(Duration::from_millis(50)).await;
        }
    }).await;

    assert!(
        processing_result.is_ok(),
        "Should process all events within timeout"
    );

    let processed_events = handler.get_processed_events().await;
    EventAssertions::assert_event_count(&processed_events, event_count);
} 
