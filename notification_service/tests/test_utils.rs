use notification_service::{models::*, Config, NotificationService};
use axum::{http::StatusCode, Router};
use axum_test::TestServer;
use fake::{Fake, Faker};
use serde_json::Value;
use sqlx::PgPool;
use std::sync::Arc;
use uuid::Uuid;

/// Test database manager for isolated notification testing
pub struct TestDatabase {
    pub pool: PgPool,
    pub cleanup_notifications: Vec<Uuid>,
    pub cleanup_templates: Vec<Uuid>,
    pub cleanup_subscriptions: Vec<Uuid>,
    pub cleanup_channels: Vec<Uuid>,
}

impl TestDatabase {
    pub async fn new() -> Self {
        let pool = get_test_db().await.clone();
        Self {
            pool,
            cleanup_notifications: Vec::new(),
            cleanup_templates: Vec::new(),
            cleanup_subscriptions: Vec::new(),
            cleanup_channels: Vec::new(),
        }
    }

    pub async fn cleanup(&mut self) {
        // Clean up in reverse dependency order
        for notification_id in &self.cleanup_notifications {
            let _ = sqlx::query("DELETE FROM notification_delivery_attempts WHERE notification_id = $1")
                .bind(notification_id)
                .execute(&self.pool)
                .await;
            let _ = sqlx::query("DELETE FROM notification_logs WHERE notification_id = $1")
                .bind(notification_id)
                .execute(&self.pool)
                .await;
            let _ = sqlx::query("DELETE FROM notifications WHERE id = $1")
                .bind(notification_id)
                .execute(&self.pool)
                .await;
        }

        for subscription_id in &self.cleanup_subscriptions {
            let _ = sqlx::query("DELETE FROM notification_subscriptions WHERE id = $1")
                .bind(subscription_id)
                .execute(&self.pool)
                .await;
        }

        for template_id in &self.cleanup_templates {
            let _ = sqlx::query("DELETE FROM notification_templates WHERE id = $1")
                .bind(template_id)
                .execute(&self.pool)
                .await;
        }

        for channel_id in &self.cleanup_channels {
            let _ = sqlx::query("DELETE FROM notification_channels WHERE id = $1")
                .bind(channel_id)
                .execute(&self.pool)
                .await;
        }

        self.cleanup_notifications.clear();
        self.cleanup_templates.clear();
        self.cleanup_subscriptions.clear();
        self.cleanup_channels.clear();
    }

    pub fn track_notification(&mut self, notification_id: Uuid) {
        self.cleanup_notifications.push(notification_id);
    }

    pub fn track_template(&mut self, template_id: Uuid) {
        self.cleanup_templates.push(template_id);
    }

    pub fn track_subscription(&mut self, subscription_id: Uuid) {
        self.cleanup_subscriptions.push(subscription_id);
    }

    pub fn track_channel(&mut self, channel_id: Uuid) {
        self.cleanup_channels.push(channel_id);
    }
}

/// Factory for creating test notification entities
pub struct NotificationFactory;

impl NotificationFactory {
    pub fn create_valid_notification_request() -> CreateNotificationRequest {
        CreateNotificationRequest {
            channel_type: ChannelType::Email,
            recipient: "test@example.com".to_string(),
            subject: format!("Test Notification {}", Faker.fake::<String>()),
            message: "This is a test notification message".to_string(),
            priority: NotificationPriority::Normal,
            template_id: None,
            template_variables: None,
            scheduled_for: None,
            tags: Some(vec!["test".to_string(), "automated".to_string()]),
            metadata: Some(serde_json::json!({
                "source": "test_suite",
                "test_run_id": Uuid::new_v4()
            })),
        }
    }

    pub fn create_valid_template_request() -> CreateTemplateRequest {
        CreateTemplateRequest {
            name: format!("Test Template {}", Faker.fake::<String>()),
            description: Some("Test template for automated testing".to_string()),
            channel_type: ChannelType::Email,
            subject_template: Some("{{title}} - {{project_name}}".to_string()),
            body_template: "Hello {{user_name}},\n\nYour {{item_type}} has been {{action}}.\n\nBest regards,\nThe Lab Team".to_string(),
            variables: vec![
                TemplateVariable {
                    name: "user_name".to_string(),
                    variable_type: VariableType::String,
                    required: true,
                    default_value: None,
                },
                TemplateVariable {
                    name: "title".to_string(),
                    variable_type: VariableType::String,
                    required: true,
                    default_value: None,
                },
                TemplateVariable {
                    name: "project_name".to_string(),
                    variable_type: VariableType::String,
                    required: false,
                    default_value: Some("Lab Project".to_string()),
                },
                TemplateVariable {
                    name: "item_type".to_string(),
                    variable_type: VariableType::String,
                    required: true,
                    default_value: None,
                },
                TemplateVariable {
                    name: "action".to_string(),
                    variable_type: VariableType::String,
                    required: true,
                    default_value: None,
                },
            ],
            is_active: true,
        }
    }

    pub fn create_valid_subscription_request() -> CreateSubscriptionRequest {
        CreateSubscriptionRequest {
            user_id: Uuid::new_v4(),
            event_type: EventType::SampleStatusChanged,
            channel_type: ChannelType::Email,
            recipient: "subscriber@example.com".to_string(),
            preferences: SubscriptionPreferences {
                frequency: DeliveryFrequency::Immediate,
                quiet_hours: Some(QuietHours {
                    start_time: chrono::NaiveTime::from_hms_opt(22, 0, 0).unwrap(),
                    end_time: chrono::NaiveTime::from_hms_opt(8, 0, 0).unwrap(),
                    timezone: "UTC".to_string(),
                }),
                digest_enabled: false,
                min_priority: NotificationPriority::Normal,
            },
            filters: Some(serde_json::json!({
                "project_ids": [Uuid::new_v4()],
                "sample_types": ["DNA", "RNA"]
            })),
            is_active: true,
        }
    }

    pub fn create_valid_channel_request() -> CreateChannelRequest {
        CreateChannelRequest {
            name: format!("Test Channel {}", Faker.fake::<String>()),
            channel_type: ChannelType::Slack,
            configuration: serde_json::json!({
                "webhook_url": "https://hooks.slack.com/services/test/test/test",
                "default_channel": "#notifications",
                "username": "LabBot"
            }),
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
        }
    }

    pub fn create_bulk_notification_request(count: usize) -> BulkNotificationRequest {
        let notifications = (0..count)
            .map(|i| CreateNotificationRequest {
                recipient: format!("test{}@example.com", i + 1),
                subject: format!("Bulk Notification {}", i + 1),
                ..Self::create_valid_notification_request()
            })
            .collect();

        BulkNotificationRequest { notifications }
    }

    pub async fn create_test_notification(notification_service: &NotificationService) -> Notification {
        let request = Self::create_valid_notification_request();
        notification_service.create_notification(request).await
            .expect("Failed to create test notification")
    }

    pub async fn create_test_template(notification_service: &NotificationService) -> NotificationTemplate {
        let request = Self::create_valid_template_request();
        notification_service.create_template(request).await
            .expect("Failed to create test template")
    }

    pub async fn create_test_subscription(notification_service: &NotificationService) -> NotificationSubscription {
        let request = Self::create_valid_subscription_request();
        notification_service.create_subscription(request).await
            .expect("Failed to create test subscription")
    }
}

/// HTTP test client wrapper for notification API testing
pub struct NotificationTestClient {
    pub server: TestServer,
    pub auth_token: Option<String>,
}

impl NotificationTestClient {
    pub fn new(app: Router) -> Self {
        let server = TestServer::new(app).unwrap();
        Self {
            server,
            auth_token: None,
        }
    }

    pub fn with_auth_token(mut self, token: String) -> Self {
        self.auth_token = Some(token);
        self
    }

    pub async fn post_json<T: serde::Serialize>(&self, path: &str, body: &T) -> axum_test::TestResponse {
        let mut request = self.server.post(path).json(body);
        if let Some(token) = &self.auth_token {
            request = request.add_header("Authorization", format!("Bearer {}", token).parse().unwrap());
        }
        request.await
    }

    pub async fn get(&self, path: &str) -> axum_test::TestResponse {
        let mut request = self.server.get(path);
        if let Some(token) = &self.auth_token {
            request = request.add_header("Authorization", format!("Bearer {}", token).parse().unwrap());
        }
        request.await
    }

    pub async fn put_json<T: serde::Serialize>(&self, path: &str, body: &T) -> axum_test::TestResponse {
        let mut request = self.server.put(path).json(body);
        if let Some(token) = &self.auth_token {
            request = request.add_header("Authorization", format!("Bearer {}", token).parse().unwrap());
        }
        request.await
    }

    pub async fn delete(&self, path: &str) -> axum_test::TestResponse {
        let mut request = self.server.delete(path);
        if let Some(token) = &self.auth_token {
            request = request.add_header("Authorization", format!("Bearer {}", token).parse().unwrap());
        }
        request.await
    }
}

/// Common assertions for notification testing
pub struct NotificationAssertions;

impl NotificationAssertions {
    pub fn assert_successful_creation(response: &Value) {
        assert_eq!(response["success"], true);
        assert!(response["data"]["id"].is_string());
        assert!(response["data"]["created_at"].is_string());
    }

    pub fn assert_notification_data(response: &Value, expected_subject: &str) {
        assert_eq!(response["success"], true);
        assert_eq!(response["data"]["subject"], expected_subject);
        assert!(response["data"]["id"].is_string());
        assert!(response["data"]["recipient"].is_string());
        assert!(response["data"]["status"].is_string());
    }

    pub fn assert_template_data(response: &Value, expected_name: &str) {
        assert_eq!(response["success"], true);
        assert_eq!(response["data"]["name"], expected_name);
        assert!(response["data"]["id"].is_string());
        assert!(response["data"]["variables"].is_array());
    }

    pub fn assert_subscription_data(response: &Value, expected_event_type: &str) {
        assert_eq!(response["success"], true);
        assert_eq!(response["data"]["event_type"], expected_event_type);
        assert!(response["data"]["id"].is_string());
        assert!(response["data"]["user_id"].is_string());
    }

    pub fn assert_delivery_status(response: &Value, expected_status: &str) {
        assert_eq!(response["success"], true);
        assert_eq!(response["data"]["delivery_status"], expected_status);
        if expected_status == "delivered" {
            assert!(response["data"]["delivered_at"].is_string());
        }
        if expected_status == "failed" {
            assert!(response["data"]["error_message"].is_string());
        }
    }

    pub fn assert_bulk_operation(response: &Value, expected_total: usize) {
        assert_eq!(response["success"], true);
        assert_eq!(response["data"]["total_processed"], expected_total);
        assert!(response["data"]["successful_count"].is_number());
        assert!(response["data"]["failed_count"].is_number());
        assert!(response["data"]["results"].is_array());
    }

    pub fn assert_webhook_delivery(response: &Value) {
        assert_eq!(response["success"], true);
        assert!(response["data"]["webhook_id"].is_string());
        assert!(response["data"]["delivered"].is_boolean());
        assert!(response["data"]["response_code"].is_number());
        assert!(response["data"]["response_time_ms"].is_number());
    }

    pub fn assert_validation_error(response: &Value) {
        assert_eq!(response["success"], false);
        assert!(response["error"].is_string());
    }

    pub fn assert_status_code(status: StatusCode, expected: StatusCode) {
        assert_eq!(status, expected);
    }
}

/// Test data generators for various notification scenarios
pub struct NotificationTestDataGenerator;

impl NotificationTestDataGenerator {
    pub fn channel_types() -> Vec<ChannelType> {
        vec![
            ChannelType::Email,
            ChannelType::SMS,
            ChannelType::Slack,
            ChannelType::Discord,
            ChannelType::Teams,
            ChannelType::Push,
            ChannelType::Webhook,
        ]
    }

    pub fn notification_priorities() -> Vec<NotificationPriority> {
        vec![
            NotificationPriority::Low,
            NotificationPriority::Normal,
            NotificationPriority::High,
            NotificationPriority::Critical,
        ]
    }

    pub fn event_types() -> Vec<EventType> {
        vec![
            EventType::SampleCreated,
            EventType::SampleStatusChanged,
            EventType::SequencingCompleted,
            EventType::QualityControlAlert,
            EventType::StorageAlert,
            EventType::UserRegistered,
            EventType::SystemMaintenance,
        ]
    }

    pub fn delivery_frequencies() -> Vec<DeliveryFrequency> {
        vec![
            DeliveryFrequency::Immediate,
            DeliveryFrequency::Hourly,
            DeliveryFrequency::Daily,
            DeliveryFrequency::Weekly,
        ]
    }

    pub fn variable_types() -> Vec<VariableType> {
        vec![
            VariableType::String,
            VariableType::Number,
            VariableType::Boolean,
            VariableType::Date,
            VariableType::Array,
            VariableType::Object,
        ]
    }

    pub fn generate_email_content(recipient_count: usize) -> Vec<CreateNotificationRequest> {
        (0..recipient_count)
            .map(|i| CreateNotificationRequest {
                channel_type: ChannelType::Email,
                recipient: format!("test{}@example.com", i + 1),
                subject: format!("Test Email {}", i + 1),
                message: format!("This is test email content for recipient {}", i + 1),
                priority: Self::notification_priorities()[i % Self::notification_priorities().len()],
                ..NotificationFactory::create_valid_notification_request()
            })
            .collect()
    }

    pub fn generate_slack_message() -> serde_json::Value {
        serde_json::json!({
            "channel": "#notifications",
            "text": "Lab alert notification",
            "attachments": [{
                "color": "warning",
                "title": "Sample Status Update",
                "text": "Sample XYZ123 has completed processing",
                "fields": [
                    {
                        "title": "Sample ID",
                        "value": "XYZ123",
                        "short": true
                    },
                    {
                        "title": "Status",
                        "value": "Completed",
                        "short": true
                    }
                ]
            }]
        })
    }

    pub fn generate_discord_embed() -> serde_json::Value {
        serde_json::json!({
            "embeds": [{
                "title": "Lab Notification",
                "description": "A new sample has been processed",
                "color": 5814783,
                "fields": [
                    {
                        "name": "Sample ID",
                        "value": "ABC456",
                        "inline": true
                    },
                    {
                        "name": "Type",
                        "value": "DNA",
                        "inline": true
                    }
                ],
                "timestamp": chrono::Utc::now().to_rfc3339()
            }]
        })
    }

    pub fn generate_push_notification() -> serde_json::Value {
        serde_json::json!({
            "title": "Lab Alert",
            "body": "Your sample analysis is complete",
            "data": {
                "sample_id": "DEF789",
                "action": "view_results"
            },
            "priority": "high",
            "sound": "default"
        })
    }

    pub fn invalid_email_addresses() -> Vec<String> {
        vec![
            "".to_string(),
            "invalid-email".to_string(),
            "@example.com".to_string(),
            "user@".to_string(),
            "user@.com".to_string(),
            "user name@example.com".to_string(),
        ]
    }

    pub fn invalid_phone_numbers() -> Vec<String> {
        vec![
            "".to_string(),
            "123".to_string(),
            "not-a-number".to_string(),
            "+1-800-INVALID".to_string(),
        ]
    }

    pub fn invalid_webhook_urls() -> Vec<String> {
        vec![
            "".to_string(),
            "not-a-url".to_string(),
            "ftp://invalid.com".to_string(),
            "http://".to_string(),
        ]
    }
}

/// Performance testing utilities for notification operations
pub struct NotificationPerformanceUtils;

impl NotificationPerformanceUtils {
    pub async fn measure_notification_creation_time(
        client: &NotificationTestClient,
        request: &CreateNotificationRequest,
    ) -> std::time::Duration {
        let start = std::time::Instant::now();
        let _ = client.post_json("/api/notifications", request).await;
        start.elapsed()
    }

    pub async fn measure_bulk_notification_time(
        client: &NotificationTestClient,
        bulk_request: &BulkNotificationRequest,
    ) -> std::time::Duration {
        let start = std::time::Instant::now();
        let _ = client.post_json("/api/notifications/bulk", bulk_request).await;
        start.elapsed()
    }

    pub async fn concurrent_notification_sending(
        client: &NotificationTestClient,
        concurrent_count: usize,
    ) -> Vec<StatusCode> {
        let tasks: Vec<_> = (0..concurrent_count)
            .map(|i| {
                let request = CreateNotificationRequest {
                    recipient: format!("concurrent{}@example.com", i),
                    subject: format!("Concurrent Notification {}", i),
                    ..NotificationFactory::create_valid_notification_request()
                };
                async move {
                    client.post_json("/api/notifications", &request).await.status_code()
                }
            })
            .collect();

        futures::future::join_all(tasks).await
    }

    pub async fn template_rendering_performance(
        client: &NotificationTestClient,
        template_id: Uuid,
        variable_sets: Vec<serde_json::Value>,
    ) -> std::time::Duration {
        let start = std::time::Instant::now();
        
        let tasks: Vec<_> = variable_sets.into_iter()
            .map(|variables| {
                let render_request = serde_json::json!({
                    "template_id": template_id,
                    "variables": variables
                });
                async move {
                    client.post_json("/api/templates/render", &render_request).await
                }
            })
            .collect();

        let _ = futures::future::join_all(tasks).await;
        start.elapsed()
    }

    pub async fn webhook_delivery_latency(
        client: &NotificationTestClient,
        webhook_urls: Vec<String>,
        payload: &serde_json::Value,
    ) -> Vec<std::time::Duration> {
        let tasks: Vec<_> = webhook_urls.into_iter()
            .map(|url| {
                let webhook_request = serde_json::json!({
                    "url": url,
                    "payload": payload,
                    "headers": {
                        "Content-Type": "application/json"
                    }
                });
                async move {
                    let start = std::time::Instant::now();
                    let _ = client.post_json("/api/webhooks/send", &webhook_request).await;
                    start.elapsed()
                }
            })
            .collect();

        futures::future::join_all(tasks).await
    }
}

/// Template testing utilities
pub struct TemplateTestUtils;

impl TemplateTestUtils {
    pub fn assert_template_rendering(rendered: &str, variables: &serde_json::Value) {
        // Check if all variables were substituted
        assert!(!rendered.contains("{{"), "Template should not contain unsubstituted variables");
        assert!(!rendered.contains("}}"), "Template should not contain unsubstituted variables");
        
        // Check if expected values are present
        if let Some(user_name) = variables.get("user_name") {
            if let Some(name) = user_name.as_str() {
                assert!(rendered.contains(name), "Rendered template should contain user name");
            }
        }
    }

    pub fn create_template_variables() -> serde_json::Value {
        serde_json::json!({
            "user_name": "Dr. Jane Smith",
            "title": "Sample Analysis Complete",
            "project_name": "COVID-19 Research",
            "item_type": "DNA Sample",
            "action": "completed processing",
            "sample_id": "SAM-2024-001",
            "completion_date": chrono::Utc::now().format("%Y-%m-%d %H:%M:%S").to_string()
        })
    }

    pub fn assert_variable_validation(template: &NotificationTemplate, variables: &serde_json::Value) {
        for template_var in &template.variables {
            if template_var.required {
                assert!(
                    variables.get(&template_var.name).is_some(),
                    "Required variable '{}' is missing",
                    template_var.name
                );
            }
        }
    }

    pub fn generate_stress_test_variables(count: usize) -> Vec<serde_json::Value> {
        (0..count)
            .map(|i| serde_json::json!({
                "user_name": format!("User {}", i),
                "title": format!("Notification {}", i),
                "project_name": format!("Project {}", i),
                "item_type": "Sample",
                "action": "processed",
                "iteration": i
            }))
            .collect()
    }
}

/// Channel integration testing utilities
pub struct ChannelTestUtils;

impl ChannelTestUtils {
    pub async fn test_email_delivery(
        client: &NotificationTestClient,
        email_config: &EmailConfig,
    ) -> axum_test::TestResponse {
        let test_request = serde_json::json!({
            "channel_type": "email",
            "config": email_config,
            "test_message": {
                "to": "test@example.com",
                "subject": "Test Email",
                "body": "This is a test email"
            }
        });

        client.post_json("/api/channels/test", &test_request).await
    }

    pub async fn test_slack_delivery(
        client: &NotificationTestClient,
        webhook_url: &str,
    ) -> axum_test::TestResponse {
        let test_request = serde_json::json!({
            "channel_type": "slack",
            "config": {
                "webhook_url": webhook_url
            },
            "test_message": NotificationTestDataGenerator::generate_slack_message()
        });

        client.post_json("/api/channels/test", &test_request).await
    }

    pub async fn test_discord_delivery(
        client: &NotificationTestClient,
        webhook_url: &str,
    ) -> axum_test::TestResponse {
        let test_request = serde_json::json!({
            "channel_type": "discord",
            "config": {
                "webhook_url": webhook_url
            },
            "test_message": NotificationTestDataGenerator::generate_discord_embed()
        });

        client.post_json("/api/channels/test", &test_request).await
    }

    pub fn assert_rate_limiting(response: &Value, expected_remaining: i32) {
        assert!(response["rate_limit"].is_object());
        assert_eq!(response["rate_limit"]["remaining"], expected_remaining);
        assert!(response["rate_limit"]["reset_time"].is_string());
    }

    pub fn assert_retry_attempt(response: &Value, attempt_number: i32) {
        assert_eq!(response["retry_attempt"], attempt_number);
        assert!(response["next_retry_at"].is_string());
        assert!(response["delay_ms"].is_number());
    }
}

/// Subscription and event testing utilities
pub struct SubscriptionTestUtils;

impl SubscriptionTestUtils {
    pub async fn trigger_test_event(
        client: &NotificationTestClient,
        event_type: EventType,
        event_data: &serde_json::Value,
    ) -> axum_test::TestResponse {
        let event_request = serde_json::json!({
            "event_type": event_type,
            "data": event_data,
            "timestamp": chrono::Utc::now(),
            "source": "test_suite"
        });

        client.post_json("/api/events/trigger", &event_request).await
    }

    pub fn create_sample_event_data() -> serde_json::Value {
        serde_json::json!({
            "sample_id": Uuid::new_v4(),
            "sample_name": "TEST-SAMPLE-001",
            "old_status": "InProgress",
            "new_status": "Completed",
            "project_id": Uuid::new_v4(),
            "updated_by": Uuid::new_v4(),
            "metadata": {
                "processing_time_minutes": 120,
                "quality_score": 98.5
            }
        })
    }

    pub fn assert_subscription_match(subscription: &NotificationSubscription, event_data: &serde_json::Value) {
        // Check if the event matches subscription filters
        if let Some(filters) = &subscription.filters {
            if let Some(project_ids) = filters.get("project_ids") {
                if let Some(event_project_id) = event_data.get("project_id") {
                    assert!(
                        project_ids.as_array().unwrap().contains(event_project_id),
                        "Event project ID should match subscription filter"
                    );
                }
            }
        }
    }

    pub async fn test_digest_generation(
        client: &NotificationTestClient,
        user_id: Uuid,
        frequency: DeliveryFrequency,
    ) -> axum_test::TestResponse {
        let digest_request = serde_json::json!({
            "user_id": user_id,
            "frequency": frequency,
            "start_time": chrono::Utc::now() - chrono::Duration::hours(24),
            "end_time": chrono::Utc::now()
        });

        client.post_json("/api/subscriptions/digest", &digest_request).await
    }
}