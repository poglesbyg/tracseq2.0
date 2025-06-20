/// LIMS (Laboratory Information Management System) Integration
/// 
/// This module provides comprehensive integration with laboratory information
/// management systems including sample tracking, workflow management, and
/// data synchronization capabilities.

use super::{Integration, IntegrationError, IntegrationStatus, ConnectionTest, HealthStatus, ConnectionStatus};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use chrono::{DateTime, Utc};
use uuid::Uuid;
use async_trait::async_trait;
use reqwest::Client;
use tokio::time::{timeout, Duration};

/// LIMS Integration implementation
#[derive(Debug)]
pub struct LIMSIntegration {
    config: LIMSConfig,
    client: Client,
    auth_token: Option<String>,
    last_health_check: Option<DateTime<Utc>>,
}

impl LIMSIntegration {
    pub fn new(config: &LIMSConfig) -> Self {
        Self {
            config: config.clone(),
            client: Client::new(),
            auth_token: None,
            last_health_check: None,
        }
    }

    /// Authenticate with LIMS system
    async fn authenticate(&mut self) -> Result<(), IntegrationError> {
        let auth_request = LIMSAuthRequest {
            username: self.config.username.clone(),
            password: self.config.password.clone(),
            client_id: self.config.client_id.clone(),
        };

        let response = timeout(
            Duration::from_secs(self.config.timeout_seconds),
            self.client
                .post(&format!("{}/auth/token", self.config.base_url))
                .json(&auth_request)
                .send()
        ).await
        .map_err(|_| IntegrationError::TimeoutError("Authentication timeout".to_string()))?
        .map_err(|e| IntegrationError::NetworkError(e.to_string()))?;

        if !response.status().is_success() {
            return Err(IntegrationError::AuthenticationFailed(
                format!("HTTP {}: {}", response.status(), response.status().canonical_reason().unwrap_or("Unknown"))
            ));
        }

        let auth_response: LIMSAuthResponse = response
            .json()
            .await
            .map_err(|e| IntegrationError::InternalError(format!("Failed to parse auth response: {}", e)))?;

        self.auth_token = Some(auth_response.access_token);
        Ok(())
    }

    /// Get samples from LIMS
    pub async fn get_samples(&self, query: &LIMSSampleQuery) -> Result<Vec<LIMSSample>, IntegrationError> {
        let token = self.auth_token.as_ref()
            .ok_or_else(|| IntegrationError::AuthenticationFailed("Not authenticated".to_string()))?;

        let mut url = format!("{}/api/v1/samples", self.config.base_url);
        let mut params = Vec::new();

        if let Some(project_id) = &query.project_id {
            params.push(format!("project_id={}", project_id));
        }
        if let Some(status) = &query.status {
            params.push(format!("status={}", status));
        }
        if let Some(limit) = query.limit {
            params.push(format!("limit={}", limit));
        }
        if let Some(offset) = query.offset {
            params.push(format!("offset={}", offset));
        }

        if !params.is_empty() {
            url.push('?');
            url.push_str(&params.join("&"));
        }

        let response = timeout(
            Duration::from_secs(self.config.timeout_seconds),
            self.client
                .get(&url)
                .bearer_auth(token)
                .send()
        ).await
        .map_err(|_| IntegrationError::TimeoutError("Sample query timeout".to_string()))?
        .map_err(|e| IntegrationError::NetworkError(e.to_string()))?;

        if !response.status().is_success() {
            return Err(IntegrationError::InternalError(
                format!("Failed to fetch samples: HTTP {}", response.status())
            ));
        }

        let samples_response: LIMSSamplesResponse = response
            .json()
            .await
            .map_err(|e| IntegrationError::InternalError(format!("Failed to parse samples response: {}", e)))?;

        Ok(samples_response.samples)
    }

    /// Sync sample data to LIMS
    pub async fn sync_sample(&self, sample: &LIMSSampleSync) -> Result<LIMSSyncResult, IntegrationError> {
        let token = self.auth_token.as_ref()
            .ok_or_else(|| IntegrationError::AuthenticationFailed("Not authenticated".to_string()))?;

        let url = if sample.lims_id.is_some() {
            format!("{}/api/v1/samples/{}", self.config.base_url, sample.lims_id.as_ref().unwrap())
        } else {
            format!("{}/api/v1/samples", self.config.base_url)
        };

        let method = if sample.lims_id.is_some() { "PUT" } else { "POST" };

        let response = timeout(
            Duration::from_secs(self.config.timeout_seconds),
            match method {
                "PUT" => self.client.put(&url),
                _ => self.client.post(&url),
            }
            .bearer_auth(token)
            .json(sample)
            .send()
        ).await
        .map_err(|_| IntegrationError::TimeoutError("Sample sync timeout".to_string()))?
        .map_err(|e| IntegrationError::NetworkError(e.to_string()))?;

        if !response.status().is_success() {
            return Err(IntegrationError::InternalError(
                format!("Failed to sync sample: HTTP {}", response.status())
            ));
        }

        let sync_response: LIMSSample = response
            .json()
            .await
            .map_err(|e| IntegrationError::InternalError(format!("Failed to parse sync response: {}", e)))?;

        Ok(LIMSSyncResult {
            success: true,
            lims_id: sync_response.id,
            updated_at: sync_response.updated_at,
        })
    }

    /// Get workflow status from LIMS
    pub async fn get_workflow_status(&self, workflow_id: &str) -> Result<LIMSWorkflowStatus, IntegrationError> {
        let token = self.auth_token.as_ref()
            .ok_or_else(|| IntegrationError::AuthenticationFailed("Not authenticated".to_string()))?;

        let url = format!("{}/api/v1/workflows/{}", self.config.base_url, workflow_id);

        let response = timeout(
            Duration::from_secs(self.config.timeout_seconds),
            self.client
                .get(&url)
                .bearer_auth(token)
                .send()
        ).await
        .map_err(|_| IntegrationError::TimeoutError("Workflow query timeout".to_string()))?
        .map_err(|e| IntegrationError::NetworkError(e.to_string()))?;

        if !response.status().is_success() {
            return Err(IntegrationError::InternalError(
                format!("Failed to fetch workflow: HTTP {}", response.status())
            ));
        }

        let workflow: LIMSWorkflowStatus = response
            .json()
            .await
            .map_err(|e| IntegrationError::InternalError(format!("Failed to parse workflow response: {}", e)))?;

        Ok(workflow)
    }

    /// Submit workflow to LIMS
    pub async fn submit_workflow(&self, workflow: &LIMSWorkflowSubmission) -> Result<LIMSWorkflowResult, IntegrationError> {
        let token = self.auth_token.as_ref()
            .ok_or_else(|| IntegrationError::AuthenticationFailed("Not authenticated".to_string()))?;

        let url = format!("{}/api/v1/workflows/{}/submit", self.config.base_url, workflow.workflow_id);

        let response = timeout(
            Duration::from_secs(self.config.timeout_seconds),
            self.client
                .post(&url)
                .bearer_auth(token)
                .json(workflow)
                .send()
        ).await
        .map_err(|_| IntegrationError::TimeoutError("Workflow submission timeout".to_string()))?
        .map_err(|e| IntegrationError::NetworkError(e.to_string()))?;

        if !response.status().is_success() {
            return Err(IntegrationError::InternalError(
                format!("Failed to submit workflow: HTTP {}", response.status())
            ));
        }

        let result: LIMSWorkflowResult = response
            .json()
            .await
            .map_err(|e| IntegrationError::InternalError(format!("Failed to parse workflow result: {}", e)))?;

        Ok(result)
    }

    /// Get LIMS system information
    pub async fn get_system_info(&self) -> Result<LIMSSystemInfo, IntegrationError> {
        let url = format!("{}/api/v1/system/info", self.config.base_url);

        let response = timeout(
            Duration::from_secs(self.config.timeout_seconds),
            self.client
                .get(&url)
                .send()
        ).await
        .map_err(|_| IntegrationError::TimeoutError("System info timeout".to_string()))?
        .map_err(|e| IntegrationError::NetworkError(e.to_string()))?;

        if !response.status().is_success() {
            return Err(IntegrationError::InternalError(
                format!("Failed to get system info: HTTP {}", response.status())
            ));
        }

        let system_info: LIMSSystemInfo = response
            .json()
            .await
            .map_err(|e| IntegrationError::InternalError(format!("Failed to parse system info: {}", e)))?;

        Ok(system_info)
    }

    /// Perform health check
    async fn health_check(&mut self) -> Result<(), IntegrationError> {
        let start_time = std::time::Instant::now();
        
        // Test basic connectivity
        let system_info = self.get_system_info().await?;
        
        // Verify authentication if we have a token
        if self.auth_token.is_some() {
            // Try a simple API call that requires auth
            let query = LIMSSampleQuery {
                project_id: None,
                status: None,
                limit: Some(1),
                offset: None,
            };
            self.get_samples(&query).await?;
        }

        self.last_health_check = Some(Utc::now());
        Ok(())
    }
}

#[async_trait]
impl Integration for LIMSIntegration {
    async fn initialize(&self) -> Result<(), IntegrationError> {
        // Test basic connectivity
        let _system_info = self.get_system_info().await?;
        Ok(())
    }

    async fn get_status(&self) -> Result<IntegrationStatus, IntegrationError> {
        let connection_status = if self.auth_token.is_some() {
            ConnectionStatus::Connected
        } else {
            ConnectionStatus::Disconnected
        };

        let health = match self.last_health_check {
            Some(last_check) if Utc::now().signed_duration_since(last_check).num_minutes() < 5 => {
                HealthStatus::Healthy
            }
            Some(_) => HealthStatus::Warning,
            None => HealthStatus::Critical,
        };

        Ok(IntegrationStatus {
            name: "LIMS Integration".to_string(),
            health,
            last_sync: self.last_health_check,
            connection_status,
        })
    }

    async fn test_connection(&self) -> Result<ConnectionTest, IntegrationError> {
        let start_time = std::time::Instant::now();
        
        match self.get_system_info().await {
            Ok(system_info) => {
                let response_time = start_time.elapsed().as_millis() as u64;
                Ok(ConnectionTest {
                    success: true,
                    response_time_ms: response_time,
                    error_message: None,
                })
            }
            Err(e) => {
                let response_time = start_time.elapsed().as_millis() as u64;
                Ok(ConnectionTest {
                    success: false,
                    response_time_ms: response_time,
                    error_message: Some(e.to_string()),
                })
            }
        }
    }
}

// Configuration and data structures

/// LIMS integration configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LIMSConfig {
    pub base_url: String,
    pub username: String,
    pub password: String,
    pub client_id: String,
    pub timeout_seconds: u64,
    pub enable_webhooks: bool,
    pub webhook_secret: Option<String>,
    pub sync_interval_minutes: u32,
    pub batch_size: usize,
}

impl Default for LIMSConfig {
    fn default() -> Self {
        Self {
            base_url: "https://lims.example.com".to_string(),
            username: "api_user".to_string(),
            password: "api_password".to_string(),
            client_id: "tracseq_integration".to_string(),
            timeout_seconds: 30,
            enable_webhooks: false,
            webhook_secret: None,
            sync_interval_minutes: 15,
            batch_size: 100,
        }
    }
}

/// LIMS authentication request
#[derive(Debug, Serialize)]
struct LIMSAuthRequest {
    username: String,
    password: String,
    client_id: String,
}

/// LIMS authentication response
#[derive(Debug, Deserialize)]
struct LIMSAuthResponse {
    access_token: String,
    token_type: String,
    expires_in: u64,
}

/// Sample query parameters
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LIMSSampleQuery {
    pub project_id: Option<String>,
    pub status: Option<String>,
    pub limit: Option<usize>,
    pub offset: Option<usize>,
}

/// LIMS sample data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LIMSSample {
    pub id: String,
    pub barcode: String,
    pub sample_type: String,
    pub project_id: String,
    pub status: String,
    pub location: Option<String>,
    pub volume_ml: Option<f64>,
    pub concentration: Option<f64>,
    pub quality_score: Option<f64>,
    pub collected_at: DateTime<Utc>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub metadata: HashMap<String, serde_json::Value>,
}

/// Sample synchronization data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LIMSSampleSync {
    pub lims_id: Option<String>,
    pub tracseq_id: Uuid,
    pub barcode: String,
    pub sample_type: String,
    pub project_id: String,
    pub status: String,
    pub location: Option<String>,
    pub volume_ml: Option<f64>,
    pub temperature: Option<f64>,
    pub storage_conditions: Option<String>,
    pub metadata: HashMap<String, serde_json::Value>,
}

/// Sample sync result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LIMSSyncResult {
    pub success: bool,
    pub lims_id: String,
    pub updated_at: DateTime<Utc>,
}

/// LIMS samples response
#[derive(Debug, Deserialize)]
struct LIMSSamplesResponse {
    samples: Vec<LIMSSample>,
    total_count: usize,
    page: usize,
    per_page: usize,
}

/// Workflow status
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LIMSWorkflowStatus {
    pub id: String,
    pub name: String,
    pub status: String,
    pub progress: f64,
    pub started_at: DateTime<Utc>,
    pub completed_at: Option<DateTime<Utc>>,
    pub steps: Vec<LIMSWorkflowStep>,
    pub metadata: HashMap<String, serde_json::Value>,
}

/// Workflow step
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LIMSWorkflowStep {
    pub id: String,
    pub name: String,
    pub status: String,
    pub started_at: Option<DateTime<Utc>>,
    pub completed_at: Option<DateTime<Utc>>,
    pub results: Option<serde_json::Value>,
}

/// Workflow submission data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LIMSWorkflowSubmission {
    pub workflow_id: String,
    pub sample_ids: Vec<String>,
    pub parameters: HashMap<String, serde_json::Value>,
    pub priority: Option<String>,
    pub requested_completion: Option<DateTime<Utc>>,
}

/// Workflow submission result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LIMSWorkflowResult {
    pub submission_id: String,
    pub workflow_id: String,
    pub status: String,
    pub estimated_completion: Option<DateTime<Utc>>,
    pub queue_position: Option<usize>,
}

/// LIMS system information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LIMSSystemInfo {
    pub name: String,
    pub version: String,
    pub api_version: String,
    pub status: String,
    pub uptime_seconds: u64,
    pub capabilities: Vec<String>,
    pub supported_sample_types: Vec<String>,
    pub max_batch_size: usize,
}

/// LIMS webhook event
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LIMSWebhookEvent {
    pub event_type: String,
    pub timestamp: DateTime<Utc>,
    pub sample_id: Option<String>,
    pub workflow_id: Option<String>,
    pub data: serde_json::Value,
}

/// LIMS integration metrics
#[derive(Debug, Clone, Serialize)]
pub struct LIMSMetrics {
    pub total_samples_synced: u64,
    pub successful_syncs: u64,
    pub failed_syncs: u64,
    pub average_sync_time_ms: f64,
    pub last_sync_time: Option<DateTime<Utc>>,
    pub active_workflows: u32,
    pub completed_workflows: u64,
    pub api_calls_per_hour: f64,
    pub error_rate: f64,
}

/// LIMS data quality check result
#[derive(Debug, Clone, Serialize)]
pub struct LIMSDataQuality {
    pub samples_checked: usize,
    pub quality_score: f64,
    pub issues_found: Vec<DataQualityIssue>,
    pub recommendations: Vec<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct DataQualityIssue {
    pub sample_id: String,
    pub issue_type: String,
    pub severity: String,
    pub description: String,
    pub suggested_fix: Option<String>,
}

/// Create LIMS integration instance
pub fn create_lims_integration(config: LIMSConfig) -> LIMSIntegration {
    LIMSIntegration::new(&config)
} 
