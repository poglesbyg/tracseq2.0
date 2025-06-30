use reqwest::Client;
use serde_json::json;
use std::time::Duration;
use uuid::Uuid;

use crate::error::{SampleResult, SampleServiceError};

#[derive(Debug, Clone)]
pub struct StorageClient {
    client: Client,
    base_url: String,
}

impl StorageClient {
    pub fn new(base_url: String) -> Self {
        let client = Client::builder()
            .timeout(Duration::from_secs(30))
            .build()
            .expect("Failed to create HTTP client");

        Self { client, base_url }
    }

    /// Health check for storage service
    pub async fn health_check(&self) -> SampleResult<()> {
        let url = format!("{}/health", self.base_url);
        
        let response = self.client
            .get(&url)
            .send()
            .await
            .map_err(|e| SampleServiceError::ExternalService {
                service: "storage".to_string(),
                message: e.to_string(),
            })?;

        if response.status().is_success() {
            Ok(())
        } else {
            Err(SampleServiceError::ExternalService {
                service: "storage".to_string(),
                message: format!("Health check failed: {}", response.status()),
            })
        }
    }

    /// Reserve storage location for sample
    pub async fn reserve_storage_location(
        &self,
        sample_id: Uuid,
        sample_type: &str,
        temperature_requirement: Option<&str>,
    ) -> SampleResult<serde_json::Value> {
        let url = format!("{}/api/v1/storage/reserve", self.base_url);
        
        let request_body = json!({
            "sample_id": sample_id,
            "sample_type": sample_type,
            "temperature_requirement": temperature_requirement,
            "priority": "normal"
        });

        let response = self.client
            .post(&url)
            .json(&request_body)
            .send()
            .await
            .map_err(|e| SampleServiceError::ExternalService {
                service: "storage".to_string(),
                message: e.to_string(),
            })?;

        if response.status().is_success() {
            let result: serde_json::Value = response.json().await.map_err(|e| {
                SampleServiceError::ExternalService {
                    service: "storage".to_string(),
                    message: format!("Failed to parse response: {}", e),
                }
            })?;

            Ok(result["data"].clone())
        } else {
            let error_text = response.text().await.unwrap_or_default();
            Err(SampleServiceError::ExternalService {
                service: "storage".to_string(),
                message: format!("Failed to reserve storage: {}", error_text),
            })
        }
    }

    /// Update sample location in storage
    pub async fn update_sample_location(
        &self,
        sample_id: Uuid,
        location_id: &str,
        status: &str,
    ) -> SampleResult<()> {
        let url = format!("{}/api/v1/storage/samples/{}/location", self.base_url, sample_id);
        
        let request_body = json!({
            "location_id": location_id,
            "status": status,
            "updated_at": chrono::Utc::now()
        });

        let response = self.client
            .put(&url)
            .json(&request_body)
            .send()
            .await
            .map_err(|e| SampleServiceError::ExternalService {
                service: "storage".to_string(),
                message: e.to_string(),
            })?;

        if response.status().is_success() {
            Ok(())
        } else {
            let error_text = response.text().await.unwrap_or_default();
            Err(SampleServiceError::ExternalService {
                service: "storage".to_string(),
                message: format!("Failed to update location: {}", error_text),
            })
        }
    }

    /// Get sample storage information
    pub async fn get_sample_storage_info(&self, sample_id: Uuid) -> SampleResult<Option<serde_json::Value>> {
        let url = format!("{}/api/v1/storage/samples/{}", self.base_url, sample_id);
        
        let response = self.client
            .get(&url)
            .send()
            .await
            .map_err(|e| SampleServiceError::ExternalService {
                service: "storage".to_string(),
                message: e.to_string(),
            })?;

        match response.status().as_u16() {
            200 => {
                let result: serde_json::Value = response.json().await.map_err(|e| {
                    SampleServiceError::ExternalService {
                        service: "storage".to_string(),
                        message: format!("Failed to parse response: {}", e),
                    }
                })?;
                Ok(Some(result["data"].clone()))
            }
            404 => Ok(None),
            _ => {
                let error_text = response.text().await.unwrap_or_default();
                Err(SampleServiceError::ExternalService {
                    service: "storage".to_string(),
                    message: format!("Failed to get storage info: {}", error_text),
                })
            }
        }
    }

    /// Release storage location
    pub async fn release_storage_location(&self, sample_id: Uuid) -> SampleResult<()> {
        let url = format!("{}/api/v1/storage/samples/{}/release", self.base_url, sample_id);
        
        let response = self.client
            .post(&url)
            .json(&json!({
                "released_at": chrono::Utc::now(),
                "reason": "sample_completed"
            }))
            .send()
            .await
            .map_err(|e| SampleServiceError::ExternalService {
                service: "storage".to_string(),
                message: e.to_string(),
            })?;

        if response.status().is_success() {
            Ok(())
        } else {
            let error_text = response.text().await.unwrap_or_default();
            Err(SampleServiceError::ExternalService {
                service: "storage".to_string(),
                message: format!("Failed to release storage: {}", error_text),
            })
        }
    }

    /// Check storage capacity
    pub async fn check_storage_capacity(&self, temperature_zone: &str) -> SampleResult<serde_json::Value> {
        let url = format!("{}/api/v1/storage/capacity", self.base_url);
        
        let response = self.client
            .get(&url)
            .query(&[("zone", temperature_zone)])
            .send()
            .await
            .map_err(|e| SampleServiceError::ExternalService {
                service: "storage".to_string(),
                message: e.to_string(),
            })?;

        if response.status().is_success() {
            let result: serde_json::Value = response.json().await.map_err(|e| {
                SampleServiceError::ExternalService {
                    service: "storage".to_string(),
                    message: format!("Failed to parse response: {}", e),
                }
            })?;

            Ok(result["data"].clone())
        } else {
            let error_text = response.text().await.unwrap_or_default();
            Err(SampleServiceError::ExternalService {
                service: "storage".to_string(),
                message: format!("Failed to check capacity: {}", error_text),
            })
        }
    }

    /// Track sample movement
    pub async fn track_sample_movement(
        &self,
        sample_id: Uuid,
        from_location: Option<&str>,
        to_location: &str,
        moved_by: &str,
    ) -> SampleResult<()> {
        let url = format!("{}/api/v1/storage/movements", self.base_url);
        
        let request_body = json!({
            "sample_id": sample_id,
            "from_location": from_location,
            "to_location": to_location,
            "moved_by": moved_by,
            "moved_at": chrono::Utc::now(),
            "movement_type": "sample_transfer"
        });

        let response = self.client
            .post(&url)
            .json(&request_body)
            .send()
            .await
            .map_err(|e| SampleServiceError::ExternalService {
                service: "storage".to_string(),
                message: e.to_string(),
            })?;

        if response.status().is_success() {
            Ok(())
        } else {
            let error_text = response.text().await.unwrap_or_default();
            Err(SampleServiceError::ExternalService {
                service: "storage".to_string(),
                message: format!("Failed to track movement: {}", error_text),
            })
        }
    }
} 
