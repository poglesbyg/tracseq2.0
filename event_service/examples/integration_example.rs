//! Integration examples for TracSeq Event Service
//!
//! This example demonstrates how to integrate the event service
//! with other TracSeq microservices.

use tracseq_event_service::{
    events::{EventHandler, EventContext, SubscriptionConfig},
    services::{
        event_bus::RedisEventBus,
        client::EventServiceClient,
    },
};
use anyhow::Result;
use async_trait::async_trait;
use std::sync::Arc;
use uuid::Uuid;
use tokio::time::{sleep, Duration};

/// Example event handler for processing sample events
struct SampleEventHandler {
    service_name: String,
}

#[async_trait]
impl EventHandler for SampleEventHandler {
    async fn handle(&self, context: EventContext) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        match context.event.event_type.as_str() {
            "sample.created" => {
                println!("📝 Sample created: Processing sample validation...");
                // Simulate processing time
                sleep(Duration::from_millis(100)).await;
                println!("✅ Sample validation completed");
            }
            "sample.status_changed" => {
                if let Some(payload) = context.event.payload.as_object() {
                    if let (Some(old_status), Some(new_status)) = (
                        payload.get("old_status").and_then(|v| v.as_str()),
                        payload.get("new_status").and_then(|v| v.as_str())
                    ) {
                        println!("🔄 Sample status changed: {} → {}", old_status, new_status);
                    }
                }
            }
            _ => {
                println!("📨 Received event: {}", context.event.event_type);
            }
        }
        Ok(())
    }

    fn event_types(&self) -> Vec<String> {
        vec!["sample.*".to_string()]
    }

    fn name(&self) -> String {
        self.service_name.clone()
    }
}

/// Example event handler for storage alerts
struct StorageEventHandler {
    service_name: String,
}

#[async_trait]
impl EventHandler for StorageEventHandler {
    async fn handle(&self, context: EventContext) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        match context.event.event_type.as_str() {
            "storage.temperature_alert" => {
                println!("🚨 TEMPERATURE ALERT!");
                if let Some(payload) = context.event.payload.as_object() {
                    if let (Some(zone), Some(temp)) = (
                        payload.get("zone_name").and_then(|v| v.as_str()),
                        payload.get("current_temperature").and_then(|v| v.as_f64())
                    ) {
                        println!("❄️  Zone: {}, Temperature: {:.1}°C", zone, temp);
                        // Here you would typically send notifications, trigger alerts, etc.
                    }
                }
            }
            "storage.capacity_warning" => {
                println!("⚠️  Storage capacity warning received");
            }
            _ => {
                println!("🏠 Storage event: {}", context.event.event_type);
            }
        }
        Ok(())
    }

    fn event_types(&self) -> Vec<String> {
        vec!["storage.*".to_string()]
    }

    fn name(&self) -> String {
        self.service_name.clone()
    }
}

/// Example: How a sample service would integrate with events
async fn sample_service_example() -> Result<()> {
    println!("🧬 === Sample Service Integration Example ===");

    // Create event client
    let client = EventServiceClient::new("http://localhost:8087", "sample-service");

    // Check if event service is available
    if !client.health_check().await.unwrap_or(false) {
        println!("❌ Event service is not available. Please start the event service first.");
        return Ok(());
    }

    println!("✅ Connected to Event Service");

    // Simulate creating a sample
    let sample_id = Uuid::new_v4();
    let submitter_id = Uuid::new_v4();
    let lab_id = Uuid::new_v4();

    println!("📝 Creating sample: {}", sample_id);

    // Publish sample created event
    let sample_data = serde_json::json!({
        "sample_id": sample_id,
        "barcode": "SAM-20240101-001",
        "sample_type": "DNA",
        "submitter_id": submitter_id,
        "lab_id": lab_id
    });
    let result = client.publish_sample_created(
        &sample_id.to_string(),
        sample_data,
    ).await?;

    println!("✅ Published sample.created event: {}", result.event_id);

    // Simulate sample status changes
    sleep(Duration::from_secs(1)).await;

    println!("🔄 Updating sample status: Pending → Validated");
    client.publish_sample_status_changed(
        &sample_id.to_string(),
        "Pending",
        "Validated",
        None,
    ).await?;

    sleep(Duration::from_secs(1)).await;

    println!("🔄 Updating sample status: Validated → InStorage");
    client.publish_sample_status_changed(
        &sample_id.to_string(),
        "Validated",
        "InStorage",
        None,
    ).await?;

    Ok(())
}

/// Example: How a storage service would integrate with events
async fn storage_service_example() -> Result<()> {
    println!("🏠 === Storage Service Integration Example ===");

    let client = EventServiceClient::new("http://localhost:8087", "storage-service");

    if !client.health_check().await.unwrap_or(false) {
        println!("❌ Event service is not available");
        return Ok(());
    }

    println!("✅ Connected to Event Service");

    // Simulate temperature monitoring
    let location_id = Uuid::new_v4();

    println!("🌡️  Monitoring temperature for location: {}", location_id);

    // Simulate normal temperature
    sleep(Duration::from_secs(1)).await;
    println!("📊 Temperature check: Normal (within range)");

    // Simulate temperature alert
    sleep(Duration::from_secs(2)).await;
    println!("🚨 Temperature threshold exceeded!");

    client.publish_temperature_alert(
        "freezer-zone-1",
        -75.5,  // Current temperature
        -80.0,  // Target temperature
        "critical",
    ).await?;

    println!("✅ Published storage.temperature_alert event");

    Ok(())
}

/// Example: Event subscriber with direct Redis connection
async fn event_subscriber_example() -> Result<()> {
    println!("📡 === Event Subscriber Example ===");

    let redis_url = std::env::var("REDIS_URL")
        .unwrap_or_else(|_| "redis://localhost:6379".to_string());

    // Create event bus
    let event_bus = RedisEventBus::new(&redis_url).await?;

    // Register handlers
    let sample_handler = Arc::new(SampleEventHandler {
        service_name: "sample-processor".to_string(),
    });

    let storage_handler = Arc::new(StorageEventHandler {
        service_name: "storage-monitor".to_string(),
    });

    event_bus.register_handler(sample_handler).await?;
    event_bus.register_handler(storage_handler).await?;

    println!("✅ Registered event handlers");

    // Create subscriptions
    let sample_subscription = SubscriptionConfig {
        name: "sample-processor-subscription".to_string(),
        event_types: vec!["sample.*".to_string()],
        consumer_group: "sample-processors".to_string(),
        consumer_name: "sample-processor-001".to_string(),
        batch_size: 10,
        timeout_ms: 5000,
        auto_ack: true,
        read_latest: false,
    };

    let storage_subscription = SubscriptionConfig {
        name: "storage-monitor-subscription".to_string(),
        event_types: vec!["storage.*".to_string()],
        consumer_group: "storage-monitors".to_string(),
        consumer_name: "storage-monitor-001".to_string(),
        batch_size: 5,
        timeout_ms: 1000,
        auto_ack: true,
        read_latest: false,
    };

    event_bus.subscribe(sample_subscription).await?;
    event_bus.subscribe(storage_subscription).await?;

    println!("✅ Created event subscriptions");
    println!("🎧 Listening for events... (Press Ctrl+C to stop)");

    // Keep the program running to process events
    loop {
        sleep(Duration::from_secs(5)).await;
        let stats = event_bus.get_stats().await;
        println!("📊 Stats - Published: {}, Consumed: {}, Failed: {}", 
                 stats.events_published, stats.events_consumed, stats.events_failed);
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize logging
    tracing_subscriber::fmt::init();

    println!("🚀 TracSeq Event Service Integration Examples");
    println!("==============================================");

    let args: Vec<String> = std::env::args().collect();
    let example = args.get(1).map(|s| s.as_str()).unwrap_or("sample");

    match example {
        "sample" => {
            sample_service_example().await?;
        }
        "storage" => {
            storage_service_example().await?;
        }
        "subscriber" => {
            event_subscriber_example().await?;
        }
        "all" => {
            // Run sample and storage examples
            sample_service_example().await?;
            println!();
            storage_service_example().await?;
        }
        _ => {
            println!("Usage: cargo run --example integration_example [sample|storage|subscriber|all]");
            println!();
            println!("Examples:");
            println!("  sample     - Sample service integration");
            println!("  storage    - Storage service integration");
            println!("  subscriber - Event subscription example");
            println!("  all        - Run sample and storage examples");
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_sample_handler() {
        let handler = SampleEventHandler {
            service_name: "test-handler".to_string(),
        };

        assert_eq!(handler.name(), "test-handler");
        assert_eq!(handler.event_types(), vec!["sample.*"]);
    }

    #[tokio::test]
    async fn test_storage_handler() {
        let handler = StorageEventHandler {
            service_name: "test-storage".to_string(),
        };

        assert_eq!(handler.name(), "test-storage");
        assert_eq!(handler.event_types(), vec!["storage.*"]);
    }
} 
