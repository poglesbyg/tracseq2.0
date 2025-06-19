//! Enhanced Laboratory Workflows with RAG/LLM Integration

pub mod laboratory;
pub mod orchestrator;
pub mod rag_integration;
pub mod templates;

use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

pub use laboratory::*;
pub use orchestrator::*;
pub use rag_integration::*;
pub use templates::*;

/// Enhanced workflow service configuration
#[derive(Debug, Clone)]
pub struct WorkflowConfig {
    /// RAG service URL
    pub rag_service_url: String,

    /// Enable AI-powered decision making
    pub enable_ai_decisions: bool,

    /// Maximum workflow complexity (number of steps)
    pub max_workflow_steps: usize,

    /// Default timeout for AI operations
    pub ai_timeout_seconds: u64,

    /// Laboratory manager service URL
    pub lab_manager_url: String,

    /// Quality threshold for AI confidence
    pub ai_confidence_threshold: f64,
}

impl Default for WorkflowConfig {
    fn default() -> Self {
        Self {
            rag_service_url: "http://localhost:8086".to_string(),
            enable_ai_decisions: true,
            max_workflow_steps: 50,
            ai_timeout_seconds: 30,
            lab_manager_url: "http://localhost:3000".to_string(),
            ai_confidence_threshold: 0.8,
        }
    }
}

/// Workflow priority levels
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum WorkflowPriority {
    Low,
    Normal,
    High,
    Critical,
    Emergency,
}

/// Risk level enumeration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RiskLevel {
    Low,
    Medium,
    High,
    Critical,
}
