use reqwest::Client;
use serde_json::json;
use std::time::Duration;

use crate::error::{SampleResult, SampleServiceError};

#[derive(Debug, Clone)]
pub struct AuthClient {
    client: Client,
    base_url: String,
}

impl AuthClient {
    pub fn new(base_url: String) -> Self {
        let client = Client::builder()
            .timeout(Duration::from_secs(30))
            .build()
            .expect("Failed to create HTTP client");

        Self { client, base_url }
    }

    /// Health check for auth service
    pub async fn health_check(&self) -> SampleResult<()> {
        let url = format!("{}/health", self.base_url);
        
        let response = self.client
            .get(&url)
            .send()
            .await
            .map_err(|e| SampleServiceError::ExternalService {
                service: "auth".to_string(),
                message: e.to_string(),
            })?;

        if response.status().is_success() {
            Ok(())
        } else {
            Err(SampleServiceError::ExternalService {
                service: "auth".to_string(),
                message: format!("Health check failed: {}", response.status()),
            })
        }
    }

    /// Validate JWT token
    pub async fn validate_token(&self, token: &str) -> SampleResult<bool> {
        let url = format!("{}/api/v1/validate/token", self.base_url);
        
        let response = self.client
            .post(&url)
            .json(&json!({
                "token": token
            }))
            .send()
            .await
            .map_err(|e| SampleServiceError::ExternalService {
                service: "auth".to_string(),
                message: e.to_string(),
            })?;

        if response.status().is_success() {
            let result: serde_json::Value = response.json().await.map_err(|e| {
                SampleServiceError::ExternalService {
                    service: "auth".to_string(),
                    message: format!("Failed to parse response: {}", e),
                }
            })?;

            Ok(result["data"]["valid"].as_bool().unwrap_or(false))
        } else {
            Ok(false)
        }
    }

    /// Get user information from token
    pub async fn get_user_from_token(&self, token: &str) -> SampleResult<Option<serde_json::Value>> {
        let url = format!("{}/api/v1/validate/token", self.base_url);
        
        let response = self.client
            .post(&url)
            .json(&json!({
                "token": token
            }))
            .send()
            .await
            .map_err(|e| SampleServiceError::ExternalService {
                service: "auth".to_string(),
                message: e.to_string(),
            })?;

        if response.status().is_success() {
            let result: serde_json::Value = response.json().await.map_err(|e| {
                SampleServiceError::ExternalService {
                    service: "auth".to_string(),
                    message: format!("Failed to parse response: {}", e),
                }
            })?;

            if result["data"]["valid"].as_bool().unwrap_or(false) {
                Ok(Some(result["data"].clone()))
            } else {
                Ok(None)
            }
        } else {
            Ok(None)
        }
    }

    /// Validate user permissions
    pub async fn validate_permissions(&self, token: &str, required_role: &str) -> SampleResult<bool> {
        let url = format!("{}/api/v1/validate/permissions", self.base_url);
        
        let response = self.client
            .post(&url)
            .json(&json!({
                "token": token,
                "required_role": required_role
            }))
            .send()
            .await
            .map_err(|e| SampleServiceError::ExternalService {
                service: "auth".to_string(),
                message: e.to_string(),
            })?;

        if response.status().is_success() {
            let result: serde_json::Value = response.json().await.map_err(|e| {
                SampleServiceError::ExternalService {
                    service: "auth".to_string(),
                    message: format!("Failed to parse response: {}", e),
                }
            })?;

            Ok(result["data"]["authorized"].as_bool().unwrap_or(false))
        } else {
            Ok(false)
        }
    }
} 
