//! RAG/LLM Integration for Enhanced Laboratory Workflows

use anyhow::Result;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::workflows::RiskLevel;

/// RAG service client for workflow intelligence
#[derive(Clone)]
#[derive(Debug)]
pub struct RagServiceClient {
    base_url: String,
    client: Client,
    timeout_seconds: u64,
}

impl RagServiceClient {
    /// Create a new RAG service client
    pub async fn new(base_url: &str) -> Result<Self> {
        let client = Client::builder()
            .timeout(std::time::Duration::from_secs(30))
            .build()?;

        Ok(Self {
            base_url: base_url.to_string(),
            client,
            timeout_seconds: 30,
        })
    }

    /// Enrich sample data with AI analysis
    pub async fn enrich_sample_data(
        &self,
        sample_type: &str,
        properties: &HashMap<String, serde_json::Value>,
    ) -> Result<SampleEnrichment> {
        let request = SampleEnrichmentRequest {
            sample_type: sample_type.to_string(),
            properties: properties.clone(),
            analysis_depth: AnalysisDepth::Standard,
        };

        let response = self
            .client
            .post(&format!("{}/api/v1/analyze/sample", self.base_url))
            .json(&request)
            .send()
            .await?;

        let enrichment: SampleEnrichment = response.json().await?;
        Ok(enrichment)
    }
}

/// Sample enrichment request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SampleEnrichmentRequest {
    pub sample_type: String,
    pub properties: HashMap<String, serde_json::Value>,
    pub analysis_depth: AnalysisDepth,
}

/// Analysis depth levels
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AnalysisDepth {
    Basic,
    Standard,
    Comprehensive,
    Expert,
}

/// Sample enrichment response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SampleEnrichment {
    pub additional_properties: HashMap<String, serde_json::Value>,
    pub suggested_processing: Vec<String>,
    pub risk_factors: Vec<RiskFactor>,
    pub quality_considerations: Vec<QualityConsideration>,
    pub confidence: f64,
}

/// Risk factor information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RiskFactor {
    pub risk_type: String,
    pub description: String,
    pub severity: RiskLevel,
    pub mitigation: String,
}

/// Quality consideration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QualityConsideration {
    pub aspect: String,
    pub importance: QualityImportance,
    pub recommendation: String,
}

/// Quality importance levels
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum QualityImportance {
    Low,
    Medium,
    High,
    Critical,
}
