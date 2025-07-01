//! MCP-Enhanced Cognitive Assistant Implementation
//! 
//! This module shows how to integrate MCP into the Cognitive Assistant Service,
//! replacing basic HTTP calls with full MCP capabilities.

use axum::{
    extract::State,
    http::StatusCode,
    response::Json,
};
use mcp_bridge::{MCPBridge, MCPConfig, MCPEnabled, impl_mcp_enabled};
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::sync::Arc;
use tracing::{error, info};

/// Enhanced application state with MCP support
#[derive(Clone)]
pub struct EnhancedAppState {
    pub mcp_bridge: MCPBridge,
    pub database_url: String,
}

// Implement MCPEnabled trait for our state
impl_mcp_enabled!(EnhancedAppState);

impl EnhancedAppState {
    pub fn new(database_url: String, mcp_proxy_url: Option<String>) -> Self {
        let config = MCPConfig {
            proxy_url: mcp_proxy_url.unwrap_or_else(|| "http://localhost:8000".to_string()),
            timeout_secs: 30,
            auth_token: None,
        };
        
        Self {
            mcp_bridge: MCPBridge::new(config),
            database_url,
        }
    }
}

#[derive(Deserialize)]
pub struct EnhancedLabQueryRequest {
    query: String,
    user_role: Option<String>,
    context: Option<String>,
    conversation_id: Option<String>,
}

#[derive(Serialize)]
pub struct EnhancedLabQueryResponse {
    response: String,
    confidence: f64,
    reasoning: String,
    response_time_ms: u64,
    sources: Vec<String>,
    conversation_id: Option<String>,
}

/// Enhanced intelligent query handler using MCP
pub async fn handle_mcp_intelligent_query(
    State(state): State<EnhancedAppState>,
    Json(request): Json<EnhancedLabQueryRequest>,
) -> Result<Json<EnhancedLabQueryResponse>, StatusCode> {
    info!("🧠 Processing MCP-enhanced query: {}", request.query);
    
    let start_time = std::time::Instant::now();
    
    // Prepare MCP parameters
    let params = json!({
        "query": request.query,
        "user_role": request.user_role.unwrap_or_else(|| "researcher".to_string()),
        "context": request.context,
        "conversation_id": request.conversation_id,
    });
    
    // Call MCP cognitive assistant
    match state.mcp_bridge.call_tool("cognitive_assistant", "ask_laboratory_question", params).await {
        Ok(result) => {
            // Extract response from MCP result
            let response = result.get("response")
                .and_then(|r| r.as_str())
                .unwrap_or("Unable to process query")
                .to_string();
            
            let confidence = result.get("confidence")
                .and_then(|c| c.as_f64())
                .unwrap_or(0.0);
            
            let reasoning = result.get("reasoning")
                .and_then(|r| r.as_str())
                .unwrap_or("MCP-based reasoning")
                .to_string();
            
            let sources = result.get("sources")
                .and_then(|s| s.as_array())
                .map(|arr| {
                    arr.iter()
                        .filter_map(|v| v.as_str().map(String::from))
                        .collect()
                })
                .unwrap_or_else(Vec::new);
            
            let conversation_id = result.get("conversation_id")
                .and_then(|c| c.as_str())
                .map(String::from);
            
            let response_time_ms = start_time.elapsed().as_millis() as u64;
            
            info!("✅ MCP query processed successfully in {}ms", response_time_ms);
            
            Ok(Json(EnhancedLabQueryResponse {
                response,
                confidence,
                reasoning,
                response_time_ms,
                sources,
                conversation_id,
            }))
        }
        Err(e) => {
            error!("MCP query failed: {}", e);
            
            // Fallback response
            Ok(Json(EnhancedLabQueryResponse {
                response: "I apologize, but I'm having trouble processing your query. Please try again.".to_string(),
                confidence: 0.0,
                reasoning: format!("MCP error: {}", e),
                response_time_ms: start_time.elapsed().as_millis() as u64,
                sources: vec![],
                conversation_id: None,
            }))
        }
    }
}

#[derive(Serialize)]
pub struct ProactiveSuggestion {
    suggestion_type: String,
    priority: String,
    suggestion: String,
    action: String,
    impact: String,
}

#[derive(Deserialize)]
pub struct SuggestionRequest {
    context_type: Option<String>,
    user_role: Option<String>,
    recent_activities: Option<Vec<String>>,
}

/// Enhanced proactive suggestions using MCP
pub async fn handle_mcp_proactive_suggestions(
    State(state): State<EnhancedAppState>,
    Json(request): Json<SuggestionRequest>,
) -> Result<Json<Vec<ProactiveSuggestion>>, StatusCode> {
    info!("🔮 Generating MCP-enhanced proactive suggestions");
    
    let params = json!({
        "context_type": request.context_type.unwrap_or_else(|| "general".to_string()),
        "user_role": request.user_role.unwrap_or_else(|| "researcher".to_string()),
        "recent_activities": request.recent_activities.unwrap_or_default(),
    });
    
    match state.mcp_bridge.call_tool("cognitive_assistant", "get_proactive_suggestions", params).await {
        Ok(result) => {
            // Parse suggestions from MCP result
            if let Some(suggestions_array) = result.as_array() {
                let suggestions: Vec<ProactiveSuggestion> = suggestions_array
                    .iter()
                    .filter_map(|s| {
                        Some(ProactiveSuggestion {
                            suggestion_type: s.get("type")?.as_str()?.to_string(),
                            priority: s.get("priority")?.as_str()?.to_string(),
                            suggestion: s.get("suggestion")?.as_str()?.to_string(),
                            action: s.get("action")?.as_str()?.to_string(),
                            impact: s.get("impact")?.as_str()?.to_string(),
                        })
                    })
                    .collect();
                
                info!("✅ Generated {} MCP-enhanced suggestions", suggestions.len());
                Ok(Json(suggestions))
            } else {
                Ok(Json(vec![]))
            }
        }
        Err(e) => {
            error!("Failed to get MCP suggestions: {}", e);
            
            // Fallback suggestions
            Ok(Json(vec![
                ProactiveSuggestion {
                    suggestion_type: "maintenance".to_string(),
                    priority: "medium".to_string(),
                    suggestion: "Check MCP service connectivity".to_string(),
                    action: "Verify all AI services are online".to_string(),
                    impact: "Ensure optimal system performance".to_string(),
                }
            ]))
        }
    }
}

/// Example of using MCP for complex workflows
pub async fn handle_sample_analysis_workflow(
    State(state): State<EnhancedAppState>,
    Json(submission_id): Json<String>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    info!("🔬 Starting MCP sample analysis workflow for: {}", submission_id);
    
    // Define workflow steps
    let steps = vec![
        json!({
            "name": "extract_submission",
            "service": "rag_service",
            "tool": "extract_laboratory_data",
            "params": {
                "document_path": format!("/submissions/{}.pdf", submission_id)
            }
        }),
        json!({
            "name": "validate_and_analyze",
            "parallel": true,
            "tasks": [
                {
                    "service": "cognitive_assistant",
                    "tool": "ask_laboratory_question",
                    "params": {
                        "query": "Analyze the extracted sample data and identify any quality concerns"
                    }
                },
                {
                    "service": "storage_optimizer",
                    "tool": "predict_capacity",
                    "params": {
                        "sample_count": 10
                    }
                }
            ]
        }),
        json!({
            "name": "generate_report",
            "service": "cognitive_assistant",
            "tool": "ask_laboratory_question",
            "params": {
                "query": "Generate a comprehensive analysis report for the submission"
            }
        })
    ];
    
    // Execute workflow via MCP
    match state.mcp_bridge.execute_workflow(
        "sample_analysis",
        serde_json::from_value(serde_json::Value::Array(steps)).unwrap(),
        false, // Not a transaction
    ).await {
        Ok(result) => {
            info!("✅ MCP workflow completed successfully");
            Ok(Json(result))
        }
        Err(e) => {
            error!("MCP workflow failed: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_enhanced_state_creation() {
        let state = EnhancedAppState::new(
            "postgres://test".to_string(),
            Some("http://localhost:8000".to_string())
        );
        
        assert_eq!(state.database_url, "postgres://test");
    }
} 