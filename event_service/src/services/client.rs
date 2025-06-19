//! Event service client library for TracSeq microservices.

use anyhow::{Context, Result};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use tracing::{debug, error, info};
use uuid::Uuid;

/// Event service client for publishing events
#[derive(Clone)]
pub struct EventServiceClient {
    /// HTTP client for API communication
    client: Client,
    
    /// Event service base URL
    base_url: String,
    
    /// Service name for event attribution
    service_name: String,
}

/// Event publication request for the REST API
#[derive(Debug, Serialize)]
struct PublishEventRequest {
    event_type: String,
    source_service: String,
    payload: serde_json::Value,
    subject: Option<String>,
    priority: Option<u8>,
    correlation_id: Option<Uuid>,
}

/// Event publication response
#[derive(Debug, Deserialize)]
pub struct PublishResponse {
    pub event_id: Uuid,
    pub stream_id: String,
    pub published_at: chrono::DateTime<chrono::Utc>,
}

impl EventServiceClient {
    /// Create a new event service client
    pub fn new(event_service_url: &str, service_name: &str) -> Self {
        let client = Client::new();
        
        Self {
            client,
            base_url: event_service_url.trim_end_matches('/').to_string(),
            service_name: service_name.to_string(),
        }
    }

    /// Publish an event to the event service
    pub async fn publish_event(
        &self,
        event_type: &str,
        payload: serde_json::Value,
    ) -> Result<PublishResponse> {
        self.publish_event_with_options(
            event_type,
            payload,
            None,     // subject
            None,     // priority
            None,     // correlation_id
        ).await
    }

    /// Publish an event with additional options
    pub async fn publish_event_with_options(
        &self,
        event_type: &str,
        payload: serde_json::Value,
        subject: Option<String>,
        priority: Option<u8>,
        correlation_id: Option<Uuid>,
    ) -> Result<PublishResponse> {
        let request = PublishEventRequest {
            event_type: event_type.to_string(),
            source_service: self.service_name.clone(),
            payload,
            subject,
            priority,
            correlation_id,
        };

        let url = format!("{}/api/v1/events/publish", self.base_url);
        
        debug!("Publishing event {} from {}", event_type, self.service_name);

        let response = self.client
            .post(&url)
            .json(&request)
            .send()
            .await
            .context("Failed to send publish request")?;

        if response.status().is_success() {
            let publish_response: PublishResponse = response
                .json()
                .await
                .context("Failed to deserialize publish response")?;
            
            debug!("Successfully published event {}", publish_response.event_id);
            Ok(publish_response)
        } else {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            error!("Failed to publish event: {} - {}", status, body);
            Err(anyhow::anyhow!("Failed to publish event: {}", status))
        }
    }

    /// Publish a sample created event
    pub async fn publish_sample_created(
        &self,
        sample_id: Uuid,
        barcode: &str,
        sample_type: &str,
        submitter_id: Uuid,
        lab_id: Uuid,
    ) -> Result<PublishResponse> {
        let payload = serde_json::json!({
            "sample_id": sample_id,
            "barcode": barcode,
            "sample_type": sample_type,
            "submitter_id": submitter_id,
            "lab_id": lab_id,
            "created_at": chrono::Utc::now()
        });

        self.publish_event("sample.created", payload).await
    }

    /// Publish a sample status changed event
    pub async fn publish_sample_status_changed(
        &self,
        sample_id: Uuid,
        barcode: &str,
        old_status: &str,
        new_status: &str,
        changed_by: Uuid,
    ) -> Result<PublishResponse> {
        let payload = serde_json::json!({
            "sample_id": sample_id,
            "barcode": barcode,
            "old_status": old_status,
            "new_status": new_status,
            "changed_by": changed_by,
            "changed_at": chrono::Utc::now()
        });

        self.publish_event_with_options(
            "sample.status_changed",
            payload,
            Some(format!("sample-{}", sample_id)),
            Some(2), // High priority
            None,
        ).await
    }

    /// Publish a user logged in event
    pub async fn publish_user_logged_in(
        &self,
        user_id: Uuid,
        username: &str,
        session_id: &str,
        ip_address: Option<&str>,
    ) -> Result<PublishResponse> {
        let payload = serde_json::json!({
            "user_id": user_id,
            "username": username,
            "session_id": session_id,
            "ip_address": ip_address,
            "login_at": chrono::Utc::now()
        });

        self.publish_event("auth.user_logged_in", payload).await
    }

    /// Publish a temperature alert event
    pub async fn publish_temperature_alert(
        &self,
        location_id: Uuid,
        zone_name: &str,
        current_temperature: f64,
        target_temperature: f64,
        sensor_id: &str,
    ) -> Result<PublishResponse> {
        let payload = serde_json::json!({
            "location_id": location_id,
            "zone_name": zone_name,
            "current_temperature": current_temperature,
            "target_temperature": target_temperature,
            "sensor_id": sensor_id,
            "alert_at": chrono::Utc::now()
        });

        self.publish_event_with_options(
            "storage.temperature_alert",
            payload,
            Some(format!("location-{}", location_id)),
            Some(1), // Critical priority
            None,
        ).await
    }

    /// Check if the event service is healthy
    pub async fn health_check(&self) -> Result<bool> {
        let url = format!("{}/health", self.base_url);
        
        match self.client.get(&url).send().await {
            Ok(response) => Ok(response.status().is_success()),
            Err(_) => Ok(false),
        }
    }
}
