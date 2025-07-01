//! MCP Bridge for Rust Services
//! 
//! This library provides a Rust interface to communicate with MCP (Model Context Protocol) servers,
//! enabling Rust microservices to leverage AI capabilities and coordinate with other MCP services.

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;
use std::sync::Arc;
use thiserror::Error;
use tracing::{debug, error, info};

#[derive(Error, Debug)]
pub enum MCPError {
    #[error("Network error: {0}")]
    Network(#[from] reqwest::Error),
    
    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),
    
    #[error("Service not found: {0}")]
    ServiceNotFound(String),
    
    #[error("Tool not found: {0}")]
    ToolNotFound(String),
    
    #[error("MCP proxy error: {0}")]
    ProxyError(String),
    
    #[error("Timeout error")]
    Timeout,
}

pub type Result<T> = std::result::Result<T, MCPError>;

/// MCP service configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MCPConfig {
    /// MCP proxy server URL
    pub proxy_url: String,
    /// Default timeout in seconds
    pub timeout_secs: u64,
    /// Service authentication token (if required)
    pub auth_token: Option<String>,
}

impl Default for MCPConfig {
    fn default() -> Self {
        Self {
            proxy_url: "http://localhost:8000".to_string(),
            timeout_secs: 30,
            auth_token: None,
        }
    }
}

/// Request to invoke an MCP service tool
#[derive(Debug, Serialize, Deserialize)]
pub struct ServiceRequest {
    pub service: String,
    pub tool: String,
    pub params: Value,
    pub timeout: Option<u64>,
}

/// Response from an MCP service
#[derive(Debug, Serialize, Deserialize)]
pub struct ServiceResponse {
    pub success: bool,
    pub service: String,
    pub tool: String,
    pub result: Option<Value>,
    pub error: Option<String>,
    pub response_time_ms: u64,
}

/// Workflow execution request
#[derive(Debug, Serialize, Deserialize)]
pub struct WorkflowRequest {
    pub workflow_name: String,
    pub steps: Vec<WorkflowStep>,
    pub transaction: bool,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct WorkflowStep {
    pub name: String,
    pub service: String,
    pub tool: String,
    pub params: Value,
    pub parallel: Option<bool>,
    pub tasks: Option<Vec<WorkflowTask>>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct WorkflowTask {
    pub service: String,
    pub tool: String,
    pub params: Value,
}

/// Main MCP bridge client
#[derive(Clone)]
pub struct MCPBridge {
    config: MCPConfig,
    client: reqwest::Client,
}

impl MCPBridge {
    /// Create a new MCP bridge instance
    pub fn new(config: MCPConfig) -> Self {
        let client = reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(config.timeout_secs))
            .build()
            .expect("Failed to create HTTP client");
        
        Self { config, client }
    }
    
    /// Call an MCP service tool
    pub async fn call_tool(
        &self,
        service: &str,
        tool: &str,
        params: Value,
    ) -> Result<Value> {
        let request = ServiceRequest {
            service: service.to_string(),
            tool: tool.to_string(),
            params,
            timeout: Some(self.config.timeout_secs),
        };
        
        debug!("Calling MCP service: {}.{}", service, tool);
        
        let response = self.client
            .post(format!("{}/mcp/tools/invoke_service_tool", self.config.proxy_url))
            .json(&request)
            .send()
            .await?;
        
        let service_response: ServiceResponse = response.json().await?;
        
        if service_response.success {
            Ok(service_response.result.unwrap_or(Value::Null))
        } else {
            Err(MCPError::ProxyError(
                service_response.error.unwrap_or_else(|| "Unknown error".to_string())
            ))
        }
    }
    
    /// Execute a workflow
    pub async fn execute_workflow(
        &self,
        workflow_name: &str,
        steps: Vec<WorkflowStep>,
        transaction: bool,
    ) -> Result<Value> {
        let request = WorkflowRequest {
            workflow_name: workflow_name.to_string(),
            steps,
            transaction,
        };
        
        info!("Executing MCP workflow: {}", workflow_name);
        
        let response = self.client
            .post(format!("{}/mcp/tools/execute_workflow", self.config.proxy_url))
            .json(&request)
            .send()
            .await?;
        
        let result: Value = response.json().await?;
        
        if result.get("success").and_then(|v| v.as_bool()).unwrap_or(false) {
            Ok(result)
        } else {
            let error = result.get("error")
                .and_then(|e| e.as_str())
                .unwrap_or("Workflow execution failed");
            Err(MCPError::ProxyError(error.to_string()))
        }
    }
    
    /// Get the status of all MCP services
    pub async fn get_services_status(&self) -> Result<Value> {
        let response = self.client
            .get(format!("{}/mcp/resources/proxy://services", self.config.proxy_url))
            .send()
            .await?;
        
        Ok(response.json().await?)
    }
    
    /// Get proxy metrics
    pub async fn get_metrics(&self) -> Result<Value> {
        let response = self.client
            .get(format!("{}/mcp/resources/proxy://metrics", self.config.proxy_url))
            .send()
            .await?;
        
        Ok(response.json().await?)
    }
}

/// Trait for services that can use MCP
#[async_trait]
pub trait MCPEnabled {
    /// Get the MCP bridge instance
    fn mcp(&self) -> &MCPBridge;
    
    /// Call the cognitive assistant
    async fn ask_lab_question(&self, query: &str, context: Option<&str>) -> Result<String> {
        let params = serde_json::json!({
            "query": query,
            "context": context,
        });
        
        let result = self.mcp().call_tool("cognitive_assistant", "ask_laboratory_question", params).await?;
        
        result.get("response")
            .and_then(|r| r.as_str())
            .map(|s| s.to_string())
            .ok_or_else(|| MCPError::ProxyError("Invalid response format".to_string()))
    }
    
    /// Process a document with RAG
    async fn process_document(&self, document_path: &str) -> Result<Value> {
        let params = serde_json::json!({
            "document_path": document_path,
            "extraction_type": "comprehensive"
        });
        
        self.mcp().call_tool("rag_service", "extract_laboratory_data", params).await
    }
    
    /// Get storage recommendations
    async fn optimize_storage(&self, samples: Value) -> Result<Value> {
        let params = serde_json::json!({
            "samples": samples,
            "optimization_goals": ["efficiency", "accessibility", "compliance"]
        });
        
        self.mcp().call_tool("storage_optimizer", "optimize_storage", params).await
    }
}

/// Helper macro to implement MCPEnabled for a struct
#[macro_export]
macro_rules! impl_mcp_enabled {
    ($struct_name:ident) => {
        #[async_trait::async_trait]
        impl MCPEnabled for $struct_name {
            fn mcp(&self) -> &MCPBridge {
                &self.mcp_bridge
            }
        }
    };
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_config_default() {
        let config = MCPConfig::default();
        assert_eq!(config.proxy_url, "http://localhost:8000");
        assert_eq!(config.timeout_secs, 30);
        assert!(config.auth_token.is_none());
    }
    
    #[test]
    fn test_service_request_serialization() {
        let request = ServiceRequest {
            service: "test_service".to_string(),
            tool: "test_tool".to_string(),
            params: serde_json::json!({"key": "value"}),
            timeout: Some(60),
        };
        
        let json = serde_json::to_string(&request).unwrap();
        let deserialized: ServiceRequest = serde_json::from_str(&json).unwrap();
        
        assert_eq!(deserialized.service, request.service);
        assert_eq!(deserialized.tool, request.tool);
    }
} 