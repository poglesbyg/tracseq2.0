//! Event Service Client for publishing events to the TracSeq Event Service.

use anyhow::Result;
use chrono::{DateTime, Utc};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use tracing::{debug, error};
use uuid::Uuid;

/// Client for interacting with the TracSeq Event Service
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct EventServiceClient {
    client: Client,
    event_service_url: String,
    service_name: String,
}

/// Event publication request
#[derive(Serialize)]
#[allow(dead_code)]
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
#[allow(dead_code)]
pub struct PublishResponse {
    pub event_id: Uuid,
    pub stream_id: String,
    pub published_at: chrono::DateTime<chrono::Utc>,
}

impl EventServiceClient {
    /// Create a new event service client
    #[allow(dead_code)]
    pub fn new(event_service_url: &str, service_name: &str) -> Self {
        Self {
            client: Client::new(),
            event_service_url: event_service_url.to_string(),
            service_name: service_name.to_string(),
        }
    }

    /// Publish an event with minimal information
    #[allow(dead_code)]
    pub async fn publish_event(
        &self,
        event_type: &str,
        payload: serde_json::Value,
    ) -> Result<PublishResponse> {
        self.publish_event_with_options(event_type, payload, None, None, None)
            .await
    }

    /// Publish an event with additional options
    #[allow(dead_code)]
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

        let url = format!("{}/api/v1/events/publish", self.event_service_url);
        let response = self
            .client
            .post(&url)
            .json(&request)
            .send()
            .await?;

        if response.status().is_success() {
            let publish_response: PublishResponse = response.json().await?;
            debug!(
                "Successfully published event {} of type {}",
                publish_response.event_id, event_type
            );
            Ok(publish_response)
        } else {
            let status = response.status();
            let error_text = response.text().await.unwrap_or_default();
            error!(
                "Failed to publish event of type {}: HTTP {} - {}",
                event_type,
                status,
                error_text
            );
            Err(anyhow::anyhow!(
                "Failed to publish event: HTTP {} - {}",
                status,
                error_text
            ))
        }
    }

    /// Publish a sample created event
    #[allow(dead_code)]
    pub async fn publish_sample_created(
        &self,
        sample_id: &str,
        sample_data: serde_json::Value,
    ) -> Result<PublishResponse> {
        self.publish_event_with_options(
            "sample.created",
            sample_data,
            Some(format!("sample-{}", sample_id)),
            Some(2), // High priority
            None,
        )
        .await
    }

    /// Publish a sample status changed event
    #[allow(dead_code)]
    pub async fn publish_sample_status_changed(
        &self,
        sample_id: &str,
        old_status: &str,
        new_status: &str,
        metadata: Option<serde_json::Value>,
    ) -> Result<PublishResponse> {
        let payload = serde_json::json!({
            "sample_id": sample_id,
            "old_status": old_status,
            "new_status": new_status,
            "metadata": metadata,
            "timestamp": Utc::now()
        });

        self.publish_event_with_options(
            "sample.status_changed",
            payload,
            Some(format!("sample-{}", sample_id)),
            Some(2), // High priority
            None,
        )
        .await
    }

    /// Publish a user logged in event
    #[allow(dead_code)]
    pub async fn publish_user_logged_in(
        &self,
        user_id: &str,
        username: &str,
        login_time: DateTime<Utc>,
    ) -> Result<PublishResponse> {
        let payload = serde_json::json!({
            "user_id": user_id,
            "username": username,
            "login_time": login_time,
            "source_ip": "unknown" // Would be filled in by actual implementation
        });

        self.publish_event_with_options(
            "auth.user_logged_in",
            payload,
            Some(format!("user-{}", user_id)),
            Some(3), // Normal priority
            None,
        )
        .await
    }

    /// Publish a temperature alert event
    #[allow(dead_code)]
    pub async fn publish_temperature_alert(
        &self,
        storage_location: &str,
        current_temperature: f64,
        threshold_temperature: f64,
        alert_level: &str,
    ) -> Result<PublishResponse> {
        let payload = serde_json::json!({
            "storage_location": storage_location,
            "current_temperature": current_temperature,
            "threshold_temperature": threshold_temperature,
            "alert_level": alert_level,
            "timestamp": Utc::now()
        });

        self.publish_event_with_options(
            "storage.temperature_alert",
            payload,
            Some(format!("storage-{}", storage_location)),
            Some(1), // Critical priority
            None,
        )
        .await
    }

    /// Check if the event service is healthy
    #[allow(dead_code)]
    pub async fn health_check(&self) -> Result<bool> {
        let url = format!("{}/health", self.event_service_url);
        match self.client.get(&url).send().await {
            Ok(response) => Ok(response.status().is_success()),
            Err(_) => Ok(false),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_client_creation() {
        let client = EventServiceClient::new("http://localhost:8087", "test-service");
        assert_eq!(client.service_name, "test-service");
        assert_eq!(client.event_service_url, "http://localhost:8087");
    }
}
