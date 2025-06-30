use crate::models::{
    LabQuery, IntelligentResponse, OllamaRequest, OllamaResponse, OllamaOptions,
    SuggestedAction, DataPoint, Prediction, DataSource, SourceType, ActionType, Priority,
    PredictionType, Impact, UserRole, QueryType, LabContext
};
use anyhow::{Result, anyhow};
use chrono::Utc;
use reqwest::Client;
use serde_json::json;
use std::time::Instant;
use tracing::{info, warn, error};

pub struct OllamaService {
    client: Client,
    base_url: String,
    default_model: String,
}

impl OllamaService {
    pub fn new(base_url: &str) -> Self {
        Self {
            client: Client::new(),
            base_url: base_url.to_string(),
            default_model: "llama3.2:3b".to_string(),
        }
    }

    pub async fn check_connection(&self) -> Result<()> {
        let url = format!("{}/api/version", self.base_url);
        
        match self.client.get(&url).send().await {
            Ok(response) => {
                if response.status().is_success() {
                    info!("✅ Ollama connection successful");
                    Ok(())
                } else {
                    Err(anyhow!("Ollama returned status: {}", response.status()))
                }
            }
            Err(e) => {
                warn!("❌ Failed to connect to Ollama: {}", e);
                Err(anyhow!("Connection failed: {}", e))
            }
        }
    }

    pub async fn process_lab_query(&self, query: &LabQuery) -> Result<IntelligentResponse> {
        let start_time = Instant::now();
        
        // Build laboratory-specific system prompt
        let system_prompt = self.build_lab_system_prompt(&query.user_role, &query.context);
        
        // Build user prompt with context
        let user_prompt = self.build_contextual_prompt(query);
        
        // Call Ollama for AI reasoning
        let ai_response = self.call_ollama(&user_prompt, Some(&system_prompt)).await?;
        
        // Parse and enhance the response
        let enhanced_response = self.enhance_response(ai_response, query).await?;
        
        let response_time = start_time.elapsed().as_millis() as u64;
        
        Ok(IntelligentResponse {
            response: enhanced_response.response,
            confidence: enhanced_response.confidence,
            reasoning: enhanced_response.reasoning,
            suggested_actions: enhanced_response.suggested_actions,
            relevant_data: enhanced_response.relevant_data,
            predictions: enhanced_response.predictions,
            follow_up_questions: enhanced_response.follow_up_questions,
            response_time_ms: response_time,
            sources: vec![DataSource {
                source_type: SourceType::AIInference,
                source_id: "ollama_llama3.2".to_string(),
                confidence: enhanced_response.confidence,
                last_updated: Utc::now(),
            }],
        })
    }

    async fn call_ollama(&self, prompt: &str, system: Option<&str>) -> Result<String> {
        let url = format!("{}/api/generate", self.base_url);
        
        let request = OllamaRequest {
            model: self.default_model.clone(),
            prompt: prompt.to_string(),
            system: system.map(|s| s.to_string()),
            stream: false,
            options: Some(OllamaOptions {
                temperature: Some(0.3), // Lower temperature for more consistent lab responses
                top_p: Some(0.9),
                top_k: Some(40),
                repeat_penalty: Some(1.1),
            }),
        };

        info!("🤖 Sending request to Ollama: {}", &self.default_model);
        
        let response = self.client
            .post(&url)
            .json(&request)
            .send()
            .await?;

        if !response.status().is_success() {
            return Err(anyhow!("Ollama API error: {}", response.status()));
        }

        let ollama_response: OllamaResponse = response.json().await?;
        
        info!("✅ Received response from Ollama ({} chars)", ollama_response.response.len());
        
        Ok(ollama_response.response)
    }

    fn build_lab_system_prompt(&self, user_role: &UserRole, context: &LabContext) -> String {
        format!(
            r#"You are an advanced AI laboratory assistant specialized in laboratory management, sample tracking, and scientific workflows. You have deep expertise in:

LABORATORY DOMAIN KNOWLEDGE:
- Sample management (DNA, RNA, Protein, Tissue samples)
- Storage systems (temperature zones: -80°C, -20°C, 4°C, RT, 37°C)
- Laboratory workflows (submission, processing, QC, sequencing)
- Quality control and compliance (HIPAA, FDA, GLP standards)
- Equipment management and automation
- Data analysis and scientific interpretation

USER CONTEXT:
- Role: {:?}
- Department: {}
- Lab Location: {}
- Current Projects: {}

RESPONSE GUIDELINES:
1. Provide accurate, scientific, and laboratory-specific advice
2. Consider safety, compliance, and best practices
3. Suggest specific actions when appropriate
4. Include confidence scores and reasoning
5. Be proactive in identifying potential issues
6. Use laboratory terminology appropriately
7. Consider the user's role and permissions

Always respond with structured JSON containing:
- Clear, actionable response
- Confidence score (0.0-1.0)
- Reasoning for your response
- Suggested actions if applicable
- Relevant data points
- Predictions or warnings if relevant
- Follow-up questions to gather more context

Focus on being helpful, accurate, and laboratory-domain aware."#,
            user_role,
            context.user_department,
            context.lab_location,
            context.current_projects.join(", ")
        )
    }

    fn build_contextual_prompt(&self, query: &LabQuery) -> String {
        format!(
            r#"LABORATORY QUERY:
Query Type: {:?}
Question: {}

CURRENT LABORATORY CONTEXT:
- Active Samples: {}
- Current Workflows: {}
- Recent Activities: {}

Please provide a comprehensive laboratory-focused response in JSON format with the following structure:
{{
    "response": "Your detailed response here",
    "confidence": 0.95,
    "reasoning": "Explanation of your reasoning",
    "suggested_actions": [
        {{
            "action_type": "CreateSample",
            "description": "Action description",
            "priority": "High",
            "estimated_time": "5 minutes",
            "required_resources": ["Resource list"]
        }}
    ],
    "relevant_data": [
        {{
            "data_type": "sample_count",
            "value": 42,
            "source": "database",
            "relevance_score": 0.9
        }}
    ],
    "predictions": [
        {{
            "prediction_type": "CapacityAlert",
            "description": "Prediction description",
            "confidence": 0.85,
            "timeframe": "24 hours",
            "impact": "Moderate"
        }}
    ],
    "follow_up_questions": ["Question 1", "Question 2"]
}}"#,
            query.query_type,
            query.query,
            query.context.current_samples.join(", "),
            query.context.active_workflows.join(", "),
            query.context.recent_activities.join(", ")
        )
    }

    async fn enhance_response(&self, ai_response: String, query: &LabQuery) -> Result<IntelligentResponse> {
        // Try to parse the AI response as JSON first
        if let Ok(structured_response) = serde_json::from_str::<serde_json::Value>(&ai_response) {
            return Ok(self.parse_structured_response(structured_response)?);
        }

        // If not JSON, create a structured response from the text
        warn!("AI response not in JSON format, creating structured response");
        
        Ok(IntelligentResponse {
            response: ai_response.clone(),
            confidence: 0.7, // Lower confidence for unstructured responses
            reasoning: "AI provided unstructured response, parsed as text".to_string(),
            suggested_actions: self.extract_actions_from_text(&ai_response),
            relevant_data: vec![],
            predictions: vec![],
            follow_up_questions: vec![],
            response_time_ms: 0, // Will be set by caller
            sources: vec![],
        })
    }

    fn parse_structured_response(&self, response: serde_json::Value) -> Result<IntelligentResponse> {
        let confidence = response.get("confidence")
            .and_then(|v| v.as_f64())
            .unwrap_or(0.7);

        let response_text = response.get("response")
            .and_then(|v| v.as_str())
            .unwrap_or("AI response parsing error")
            .to_string();

        let reasoning = response.get("reasoning")
            .and_then(|v| v.as_str())
            .unwrap_or("No reasoning provided")
            .to_string();

        // Parse suggested actions
        let suggested_actions = response.get("suggested_actions")
            .and_then(|v| v.as_array())
            .map(|actions| {
                actions.iter()
                    .filter_map(|action| self.parse_suggested_action(action))
                    .collect()
            })
            .unwrap_or_default();

        // Parse follow-up questions
        let follow_up_questions = response.get("follow_up_questions")
            .and_then(|v| v.as_array())
            .map(|questions| {
                questions.iter()
                    .filter_map(|q| q.as_str())
                    .map(|s| s.to_string())
                    .collect()
            })
            .unwrap_or_default();

        Ok(IntelligentResponse {
            response: response_text,
            confidence,
            reasoning,
            suggested_actions,
            relevant_data: vec![], // Will be populated by lab context service
            predictions: vec![],   // Will be populated by predictive service
            follow_up_questions,
            response_time_ms: 0,   // Will be set by caller
            sources: vec![],       // Will be set by caller
        })
    }

    fn parse_suggested_action(&self, action: &serde_json::Value) -> Option<SuggestedAction> {
        Some(SuggestedAction {
            action_type: self.parse_action_type(action.get("action_type")?)?,
            description: action.get("description")?.as_str()?.to_string(),
            priority: self.parse_priority(action.get("priority")?),
            estimated_time: action.get("estimated_time")?.as_str()?.to_string(),
            required_resources: action.get("required_resources")
                .and_then(|v| v.as_array())
                .map(|arr| {
                    arr.iter()
                        .filter_map(|v| v.as_str())
                        .map(|s| s.to_string())
                        .collect()
                })
                .unwrap_or_default(),
            endpoint: action.get("endpoint").and_then(|v| v.as_str()).map(|s| s.to_string()),
            payload: action.get("payload").cloned(),
        })
    }

    fn parse_action_type(&self, value: &serde_json::Value) -> Option<ActionType> {
        match value.as_str()? {
            "CreateSample" => Some(ActionType::CreateSample),
            "UpdateSample" => Some(ActionType::UpdateSample),
            "AssignStorage" => Some(ActionType::AssignStorage),
            "RunAnalysis" => Some(ActionType::RunAnalysis),
            "GenerateReport" => Some(ActionType::GenerateReport),
            "ScheduleTask" => Some(ActionType::ScheduleTask),
            "NotifyUser" => Some(ActionType::NotifyUser),
            "OptimizeWorkflow" => Some(ActionType::OptimizeWorkflow),
            _ => Some(ActionType::NotifyUser), // Default fallback
        }
    }

    fn parse_priority(&self, value: &serde_json::Value) -> Priority {
        match value.as_str().unwrap_or("Medium") {
            "Low" => Priority::Low,
            "High" => Priority::High,
            "Critical" => Priority::Critical,
            _ => Priority::Medium,
        }
    }

    fn extract_actions_from_text(&self, text: &str) -> Vec<SuggestedAction> {
        // Simple heuristic to extract actions from unstructured text
        let mut actions = Vec::new();
        
        if text.to_lowercase().contains("create") && text.to_lowercase().contains("sample") {
            actions.push(SuggestedAction {
                action_type: ActionType::CreateSample,
                description: "Consider creating a new sample based on the discussion".to_string(),
                priority: Priority::Medium,
                estimated_time: "5-10 minutes".to_string(),
                required_resources: vec!["Sample information".to_string()],
                endpoint: Some("/samples".to_string()),
                payload: None,
            });
        }

        actions
    }

    pub async fn generate_proactive_suggestions(&self, context: &LabContext, user_role: &UserRole) -> Result<Vec<String>> {
        let prompt = format!(
            r#"Based on the current laboratory context, generate 3-5 proactive suggestions that would be helpful for a {} in this situation:

Current Context:
- Active Samples: {}
- Current Workflows: {}
- Recent Activities: {}

Provide practical, actionable suggestions that could improve efficiency, prevent issues, or optimize workflows. Return as a simple list."#,
            format!("{:?}", user_role),
            context.current_samples.join(", "),
            context.active_workflows.join(", "),
            context.recent_activities.join(", ")
        );

        let response = self.call_ollama(&prompt, None).await?;
        
        // Parse suggestions from response
        let suggestions: Vec<String> = response
            .lines()
            .filter(|line| !line.trim().is_empty())
            .take(5)
            .map(|line| line.trim().to_string())
            .collect();

        Ok(suggestions)
    }
} 