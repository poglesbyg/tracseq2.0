use crate::test_utils::*;
use notification_service::{
    models::*,
    handlers::*,
    services::*,
    create_app,
};
use axum_test::TestServer;
use serde_json::json;
use uuid::Uuid;

/// Integration tests for complete notification workflows
#[tokio::test]
async fn test_complete_notification_lifecycle() {
    let mut test_db = TestDatabase::new().await;
    let app = create_app().await;
    let client = NotificationTestClient::new(app);

    // Phase 1: Create notification template
    let template_request = NotificationFactory::create_valid_template_request();
    let template_name = template_request.name.clone();
    
    let response = client.post_json("/api/notifications/templates", &template_request).await;
    NotificationAssertions::assert_status_code(response.status_code(), axum::http::StatusCode::CREATED);
    
    let template_data: serde_json::Value = response.json();
    NotificationAssertions::assert_template_data(&template_data, &template_name);
    
    let template_id = Uuid::parse_str(template_data["data"]["id"].as_str().unwrap()).unwrap();
    test_db.track_template(template_id);

    // Phase 2: Create notification channel
    let channel_request = NotificationFactory::create_valid_channel_request();
    let channel_name = channel_request.name.clone();
    
    let response = client.post_json("/api/notifications/channels", &channel_request).await;
    NotificationAssertions::assert_status_code(response.status_code(), axum::http::StatusCode::CREATED);
    
    let channel_data: serde_json::Value = response.json();
    assert_eq!(channel_data["data"]["name"], channel_name);
    
    let channel_id = Uuid::parse_str(channel_data["data"]["id"].as_str().unwrap()).unwrap();
    test_db.track_channel(channel_id);

    // Phase 3: Create subscription
    let subscription_request = NotificationFactory::create_valid_subscription_request();
    let event_type = subscription_request.event_type.clone();
    
    let response = client.post_json("/api/notifications/subscriptions", &subscription_request).await;
    NotificationAssertions::assert_status_code(response.status_code(), axum::http::StatusCode::CREATED);
    
    let subscription_data: serde_json::Value = response.json();
    NotificationAssertions::assert_subscription_data(&subscription_data, &format!("{:?}", event_type));
    
    let subscription_id = Uuid::parse_str(subscription_data["data"]["id"].as_str().unwrap()).unwrap();
    test_db.track_subscription(subscription_id);

    // Phase 4: Send templated notification
    let mut notification_request = NotificationFactory::create_valid_notification_request();
    notification_request.template_id = Some(template_id);
    notification_request.template_variables = Some(TemplateTestUtils::create_template_variables());
    
    let response = client.post_json("/api/notifications", &notification_request).await;
    NotificationAssertions::assert_status_code(response.status_code(), axum::http::StatusCode::CREATED);
    
    let notification_data: serde_json::Value = response.json();
    NotificationAssertions::assert_notification_data(&notification_data, &notification_request.subject);
    
    let notification_id = Uuid::parse_str(notification_data["data"]["id"].as_str().unwrap()).unwrap();
    test_db.track_notification(notification_id);

    // Phase 5: Check delivery status
    tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;
    
    let response = client.get(&format!("/api/notifications/{}/status", notification_id)).await;
    NotificationAssertions::assert_status_code(response.status_code(), axum::http::StatusCode::OK);
    
    let status_data: serde_json::Value = response.json();
    NotificationAssertions::assert_delivery_status(&status_data, "delivered");

    // Phase 6: Trigger event-based notification
    let event_data = SubscriptionTestUtils::create_sample_event_data();
    let response = SubscriptionTestUtils::trigger_test_event(&client, event_type, &event_data).await;
    NotificationAssertions::assert_status_code(response.status_code(), axum::http::StatusCode::OK);

    // Verify event triggered notification
    tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
    
    let response = client.get(&format!("/api/notifications/subscriptions/{}/notifications", subscription_id)).await;
    NotificationAssertions::assert_status_code(response.status_code(), axum::http::StatusCode::OK);
    
    let triggered_notifications: serde_json::Value = response.json();
    assert_eq!(triggered_notifications["success"], true);
    assert!(triggered_notifications["data"]["notifications"].is_array());

    test_db.cleanup().await;
}

#[tokio::test]
async fn test_multi_channel_notification_delivery() {
    let mut test_db = TestDatabase::new().await;
    let app = create_app().await;
    let client = NotificationTestClient::new(app);

    // Create channels for different platforms
    let channels = vec![
        ("Email Channel", ChannelType::Email, json!({
            "smtp_host": "smtp.example.com",
            "smtp_port": 587,
            "username": "test@example.com",
            "password": "password",
            "use_tls": true
        })),
        ("Slack Channel", ChannelType::Slack, json!({
            "webhook_url": "https://hooks.slack.com/services/test/test/test",
            "default_channel": "#notifications",
            "username": "LabBot"
        })),
        ("Discord Channel", ChannelType::Discord, json!({
            "webhook_url": "https://discord.com/api/webhooks/test/test",
            "username": "Lab Notifications"
        })),
    ];

    let mut channel_ids = Vec::new();
    
    for (name, channel_type, config) in channels {
        let channel_request = CreateChannelRequest {
            name: name.to_string(),
            channel_type,
            configuration: config,
            is_active: true,
            rate_limit: Some(RateLimit {
                max_messages_per_minute: 60,
                max_messages_per_hour: 1000,
                burst_limit: 10,
            }),
            retry_policy: Some(RetryPolicy {
                max_retries: 3,
                initial_delay_ms: 1000,
                max_delay_ms: 30000,
                backoff_multiplier: 2.0,
            }),
        };
        
        let response = client.post_json("/api/notifications/channels", &channel_request).await;
        NotificationAssertions::assert_status_code(response.status_code(), axum::http::StatusCode::CREATED);
        
        let channel_data: serde_json::Value = response.json();
        let channel_id = Uuid::parse_str(channel_data["data"]["id"].as_str().unwrap()).unwrap();
        channel_ids.push(channel_id);
        test_db.track_channel(channel_id);
    }

    // Test channel connectivity
    for channel_id in &channel_ids {
        let test_request = json!({
            "channel_id": channel_id,
            "test_message": "Test connectivity"
        });
        
        let response = client.post_json("/api/notifications/channels/test", &test_request).await;
        NotificationAssertions::assert_status_code(response.status_code(), axum::http::StatusCode::OK);
        
        let test_result: serde_json::Value = response.json();
        assert_eq!(test_result["success"], true);
    }

    // Send notification to all channels
    let broadcast_request = json!({
        "subject": "Critical Lab Alert",
        "message": "Temperature alarm triggered in Freezer Unit A",
        "priority": "Critical",
        "channels": channel_ids,
        "metadata": {
            "alert_type": "temperature",
            "location": "Freezer Unit A",
            "temperature": -75.0,
            "threshold": -80.0
        }
    });
    
    let response = client.post_json("/api/notifications/broadcast", &broadcast_request).await;
    NotificationAssertions::assert_status_code(response.status_code(), axum::http::StatusCode::CREATED);
    
    let broadcast_data: serde_json::Value = response.json();
    assert_eq!(broadcast_data["success"], true);
    assert_eq!(broadcast_data["data"]["channels_count"], 3);
    assert_eq!(broadcast_data["data"]["status"], "Processing");

    test_db.cleanup().await;
}

#[tokio::test]
async fn test_bulk_notification_processing() {
    let mut test_db = TestDatabase::new().await;
    let app = create_app().await;
    let client = NotificationTestClient::new(app);

    // Create bulk notification request
    let bulk_count = 50;
    let bulk_request = NotificationFactory::create_bulk_notification_request(bulk_count);
    
    let start_time = std::time::Instant::now();
    let response = client.post_json("/api/notifications/bulk", &bulk_request).await;
    let processing_time = start_time.elapsed();
    
    NotificationAssertions::assert_status_code(response.status_code(), axum::http::StatusCode::CREATED);
    
    let bulk_data: serde_json::Value = response.json();
    NotificationAssertions::assert_bulk_operation(&bulk_data, bulk_count);
    
    let batch_id = bulk_data["data"]["batch_id"].as_str().unwrap();
    
    // Monitor bulk processing progress
    let mut attempts = 0;
    let max_attempts = 30;
    
    loop {
        tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
        
        let response = client.get(&format!("/api/notifications/bulk/{}/status", batch_id)).await;
        let status_data: serde_json::Value = response.json();
        
        let status = status_data["data"]["status"].as_str().unwrap();
        let processed = status_data["data"]["processed_count"].as_u64().unwrap();
        
        if status == "Completed" || processed == bulk_count as u64 {
            assert_eq!(status, "Completed");
            assert_eq!(processed, bulk_count as u64);
            break;
        }
        
        attempts += 1;
        if attempts >= max_attempts {
            panic!("Bulk processing did not complete within timeout");
        }
    }
    
    // Verify performance metrics
    assert!(processing_time.as_millis() < 5000, "Bulk processing should complete quickly");
    
    // Get detailed results
    let response = client.get(&format!("/api/notifications/bulk/{}/results", batch_id)).await;
    let results_data: serde_json::Value = response.json();
    
    assert_eq!(results_data["success"], true);
    assert!(results_data["data"]["successful_count"].as_u64().unwrap() >= (bulk_count as u64 * 90 / 100)); // 90% success rate

    test_db.cleanup().await;
}

#[tokio::test]
async fn test_template_rendering_and_validation() {
    let mut test_db = TestDatabase::new().await;
    let app = create_app().await;
    let client = NotificationTestClient::new(app);

    // Create template with complex variables
    let template_request = CreateTemplateRequest {
        name: "Sample Processing Complete".to_string(),
        description: Some("Template for sample processing completion".to_string()),
        channel_type: ChannelType::Email,
        subject_template: Some("Sample {{sample.id}} - {{status}}".to_string()),
        body_template: r#"
Dear {{user.name}},

Your sample {{sample.id}} ({{sample.name}}) has been processed with status: {{status}}.

Processing Details:
{{#each processing_steps}}
- {{name}}: {{description}} ({{duration}} minutes)
{{/each}}

Results Summary:
{{#each results}}
- {{metric}}: {{value}} {{unit}} ({{status}})
{{/each}}

{{#if status_is_pass}}
✅ All quality checks passed.
{{else}}
❌ Some quality checks failed. Please review.
{{/if}}

Next Steps:
{{#if next_steps}}
{{#each next_steps}}
{{step_number}}. {{description}}
{{/each}}
{{else}}
No further action required.
{{/if}}

Best regards,
The Lab Team
        "#.trim().to_string(),
        variables: vec![
            TemplateVariable {
                name: "user".to_string(),
                variable_type: VariableType::Object,
                required: true,
                default_value: None,
            },
            TemplateVariable {
                name: "sample".to_string(),
                variable_type: VariableType::Object,
                required: true,
                default_value: None,
            },
            TemplateVariable {
                name: "status".to_string(),
                variable_type: VariableType::String,
                required: true,
                default_value: None,
            },
            TemplateVariable {
                name: "processing_steps".to_string(),
                variable_type: VariableType::Array,
                required: false,
                default_value: Some("[]".to_string()),
            },
            TemplateVariable {
                name: "results".to_string(),
                variable_type: VariableType::Array,
                required: false,
                default_value: Some("[]".to_string()),
            },
        ],
        is_active: true,
    };
    
    let response = client.post_json("/api/notifications/templates", &template_request).await;
    let template_data: serde_json::Value = response.json();
    let template_id = Uuid::parse_str(template_data["data"]["id"].as_str().unwrap()).unwrap();
    test_db.track_template(template_id);

    // Test template rendering with complete data
    let template_variables = json!({
        "user": {
            "name": "Dr. Jane Smith",
            "email": "jane.smith@lab.com"
        },
        "sample": {
            "id": "SAM-2024-001",
            "name": "Test Sample Alpha"
        },
        "status": "Completed Successfully",
        "status_is_pass": true,
        "processing_steps": [
            {
                "name": "DNA Extraction",
                "description": "High-quality DNA extracted",
                "duration": 45
            },
            {
                "name": "Quality Control",
                "description": "All QC metrics passed",
                "duration": 30
            }
        ],
        "results": [
            {
                "metric": "Concentration",
                "value": 150.5,
                "unit": "ng/µL",
                "status": "Pass"
            },
            {
                "metric": "Purity (260/280)",
                "value": 1.85,
                "unit": "ratio",
                "status": "Pass"
            }
        ]
    });
    
    let render_request = json!({
        "template_id": template_id,
        "variables": template_variables
    });
    
    let response = client.post_json("/api/notifications/templates/render", &render_request).await;
    NotificationAssertions::assert_status_code(response.status_code(), axum::http::StatusCode::OK);
    
    let render_data: serde_json::Value = response.json();
    assert_eq!(render_data["success"], true);
    
    let rendered_subject = render_data["data"]["subject"].as_str().unwrap();
    let rendered_body = render_data["data"]["body"].as_str().unwrap();
    
    // Validate template rendering
    TemplateTestUtils::assert_template_rendering(rendered_subject, &template_variables);
    TemplateTestUtils::assert_template_rendering(rendered_body, &template_variables);
    
    assert!(rendered_subject.contains("SAM-2024-001"));
    assert!(rendered_subject.contains("Completed Successfully"));
    assert!(rendered_body.contains("Dr. Jane Smith"));
    assert!(rendered_body.contains("DNA Extraction"));
    assert!(rendered_body.contains("✅ All quality checks passed"));

    // Test template validation with missing required variables
    let invalid_variables = json!({
        "user": {"name": "Test User"},
        // Missing required "sample" and "status"
    });
    
    let invalid_render_request = json!({
        "template_id": template_id,
        "variables": invalid_variables
    });
    
    let response = client.post_json("/api/notifications/templates/render", &invalid_render_request).await;
    NotificationAssertions::assert_status_code(response.status_code(), axum::http::StatusCode::BAD_REQUEST);
    
    let error_data: serde_json::Value = response.json();
    NotificationAssertions::assert_validation_error(&error_data);
    assert!(error_data["error"].as_str().unwrap().contains("required"));

    test_db.cleanup().await;
}

#[tokio::test]
async fn test_notification_rate_limiting_and_retry() {
    let mut test_db = TestDatabase::new().await;
    let app = create_app().await;
    let client = NotificationTestClient::new(app);

    // Create channel with strict rate limits
    let channel_request = CreateChannelRequest {
        name: "Rate Limited Channel".to_string(),
        channel_type: ChannelType::Email,
        configuration: json!({
            "smtp_host": "smtp.example.com",
            "smtp_port": 587
        }),
        is_active: true,
        rate_limit: Some(RateLimit {
            max_messages_per_minute: 5,  // Very low limit for testing
            max_messages_per_hour: 20,
            burst_limit: 2,
        }),
        retry_policy: Some(RetryPolicy {
            max_retries: 3,
            initial_delay_ms: 500,
            max_delay_ms: 5000,
            backoff_multiplier: 2.0,
        }),
    };
    
    let response = client.post_json("/api/notifications/channels", &channel_request).await;
    let channel_data: serde_json::Value = response.json();
    let channel_id = Uuid::parse_str(channel_data["data"]["id"].as_str().unwrap()).unwrap();
    test_db.track_channel(channel_id);

    // Send burst of notifications to trigger rate limiting
    let notification_requests: Vec<_> = (0..10)
        .map(|i| {
            let mut request = NotificationFactory::create_valid_notification_request();
            request.subject = format!("Test Notification {}", i + 1);
            request
        })
        .collect();

    let mut responses = Vec::new();
    for request in notification_requests {
        let response = client.post_json("/api/notifications", &request).await;
        responses.push((response.status_code(), response.json::<serde_json::Value>()));
    }

    // Verify rate limiting behavior
    let successful_immediate = responses.iter()
        .filter(|(status, _)| *status == axum::http::StatusCode::CREATED)
        .count();
    
    let rate_limited = responses.iter()
        .filter(|(status, _)| *status == axum::http::StatusCode::TOO_MANY_REQUESTS)
        .count();

    assert!(successful_immediate <= 2, "Should only allow burst limit through immediately");
    assert!(rate_limited >= 5, "Should rate limit excess requests");

    // Check rate limit status
    let response = client.get(&format!("/api/notifications/channels/{}/rate-limit", channel_id)).await;
    let rate_limit_data: serde_json::Value = response.json();
    
    ChannelTestUtils::assert_rate_limiting(&rate_limit_data, 0); // Should be at limit

    // Test retry mechanism - send notification that will fail initially
    let retry_request = json!({
        "recipient": "fail-initially@example.com", // Simulated failing recipient
        "subject": "Retry Test",
        "message": "This should be retried",
        "priority": "High",
        "channel_id": channel_id
    });
    
    let response = client.post_json("/api/notifications/with-retry", &retry_request).await;
    let retry_data: serde_json::Value = response.json();
    
    if response.status_code() == axum::http::StatusCode::ACCEPTED {
        let notification_id = retry_data["data"]["id"].as_str().unwrap();
        
        // Wait for retry attempts
        tokio::time::sleep(tokio::time::Duration::from_secs(3)).await;
        
        let response = client.get(&format!("/api/notifications/{}/retry-history", notification_id)).await;
        let retry_history: serde_json::Value = response.json();
        
        assert!(retry_history["data"]["retry_attempts"].is_array());
        let attempts = retry_history["data"]["retry_attempts"].as_array().unwrap();
        assert!(!attempts.is_empty(), "Should have retry attempts");
        
        for attempt in attempts {
            ChannelTestUtils::assert_retry_attempt(attempt, attempt["attempt_number"].as_i64().unwrap() as i32);
        }
    }

    test_db.cleanup().await;
}

#[tokio::test]
async fn test_notification_subscription_workflow() {
    let mut test_db = TestDatabase::new().await;
    let app = create_app().await;
    let client = NotificationTestClient::new(app);

    // Create multiple subscriptions for different event types
    let event_types = NotificationTestDataGenerator::event_types();
    let mut subscription_ids = Vec::new();
    
    for event_type in &event_types {
        let mut subscription_request = NotificationFactory::create_valid_subscription_request();
        subscription_request.event_type = event_type.clone();
        subscription_request.recipient = format!("user-{}@example.com", format!("{:?}", event_type).to_lowercase());
        
        let response = client.post_json("/api/notifications/subscriptions", &subscription_request).await;
        let subscription_data: serde_json::Value = response.json();
        let subscription_id = Uuid::parse_str(subscription_data["data"]["id"].as_str().unwrap()).unwrap();
        subscription_ids.push((subscription_id, event_type.clone()));
        test_db.track_subscription(subscription_id);
    }

    // Test event triggering and notification generation
    for (subscription_id, event_type) in &subscription_ids {
        let event_data = match event_type {
            EventType::SampleCreated => json!({
                "sample_id": Uuid::new_v4(),
                "sample_name": "TEST-SAMPLE-001",
                "sample_type": "DNA",
                "created_by": Uuid::new_v4()
            }),
            EventType::SampleStatusChanged => SubscriptionTestUtils::create_sample_event_data(),
            EventType::SequencingCompleted => json!({
                "run_id": Uuid::new_v4(),
                "sample_count": 24,
                "completion_time": chrono::Utc::now(),
                "success_rate": 96.5
            }),
            _ => json!({"event_id": Uuid::new_v4(), "timestamp": chrono::Utc::now()}),
        };
        
        let response = SubscriptionTestUtils::trigger_test_event(&client, event_type.clone(), &event_data).await;
        NotificationAssertions::assert_status_code(response.status_code(), axum::http::StatusCode::OK);
        
        // Wait for event processing
        tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;
        
        // Verify notification was generated
        let response = client.get(&format!("/api/notifications/subscriptions/{}/notifications", subscription_id)).await;
        let notifications: serde_json::Value = response.json();
        
        assert_eq!(notifications["success"], true);
        let notification_list = notifications["data"]["notifications"].as_array().unwrap();
        assert!(!notification_list.is_empty(), "Event should trigger notification");
    }

    // Test digest generation
    let user_id = Uuid::new_v4();
    let response = SubscriptionTestUtils::test_digest_generation(&client, user_id, DeliveryFrequency::Daily).await;
    NotificationAssertions::assert_status_code(response.status_code(), axum::http::StatusCode::OK);
    
    let digest_data: serde_json::Value = response.json();
    assert_eq!(digest_data["success"], true);
    assert!(digest_data["data"]["digest_content"].is_string());
    assert!(digest_data["data"]["notification_count"].is_number());

    test_db.cleanup().await;
}

#[tokio::test]
async fn test_concurrent_notification_operations() {
    let mut test_db = TestDatabase::new().await;
    let app = create_app().await;
    let client = NotificationTestClient::new(app);

    // Test concurrent notification sending
    let concurrent_count = 20;
    let concurrent_results = NotificationPerformanceUtils::concurrent_notification_sending(
        &client,
        concurrent_count,
    ).await;
    
    let successful_notifications = concurrent_results.iter()
        .filter(|&status| *status == axum::http::StatusCode::CREATED)
        .count();
    
    assert!(successful_notifications >= (concurrent_count * 90 / 100), "At least 90% of concurrent notifications should succeed");

    // Test template rendering performance
    let template_request = NotificationFactory::create_valid_template_request();
    let response = client.post_json("/api/notifications/templates", &template_request).await;
    let template_data: serde_json::Value = response.json();
    let template_id = Uuid::parse_str(template_data["data"]["id"].as_str().unwrap()).unwrap();
    test_db.track_template(template_id);

    let variable_sets = TemplateTestUtils::generate_stress_test_variables(50);
    let rendering_duration = NotificationPerformanceUtils::template_rendering_performance(
        &client,
        template_id,
        variable_sets,
    ).await;
    
    assert!(rendering_duration.as_secs() < 10, "Template rendering should complete within 10 seconds");

    // Test webhook delivery latency
    let webhook_urls = vec![
        "https://httpbin.org/post".to_string(),
        "https://webhook.site/unique-id".to_string(),
        "https://postman-echo.com/post".to_string(),
    ];
    
    let test_payload = json!({
        "event": "test_webhook",
        "timestamp": chrono::Utc::now(),
        "data": {"test": true}
    });
    
    let webhook_latencies = NotificationPerformanceUtils::webhook_delivery_latency(
        &client,
        webhook_urls,
        &test_payload,
    ).await;
    
    for latency in webhook_latencies {
        assert!(latency.as_secs() < 30, "Webhook delivery should complete within 30 seconds");
    }

    test_db.cleanup().await;
}