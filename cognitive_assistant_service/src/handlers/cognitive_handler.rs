use axum::{
    extract::State,
    http::StatusCode,
    response::Json,
};
use tracing::{info, error};
use uuid::Uuid;

use crate::{
    AppState,
    models::{
        LabQuery, IntelligentResponse, ChatMessage, ChatResponse, AnalysisRequest, AnalysisResult,
        ProactiveSuggestionRequest, ProactiveSuggestion, SuggestionType, Priority,
    },
};

pub async fn handle_intelligent_query(
    State(state): State<AppState>,
    Json(query): Json<LabQuery>,
) -> Result<Json<IntelligentResponse>, StatusCode> {
    info!("🧠 Processing intelligent query: {}", query.query);

    // Enrich context with laboratory data
    let enriched_context = match state.lab_context_service.get_enriched_context(&query.context).await {
        Ok(context) => context,
        Err(e) => {
            error!("Failed to enrich context: {}", e);
            query.context // Use original context as fallback
        }
    };

    // Create enriched query
    let enriched_query = LabQuery {
        context: enriched_context,
        ..query
    };

    // Process with Ollama
    let mut response = match state.ollama_service.process_lab_query(&enriched_query).await {
        Ok(response) => response,
        Err(e) => {
            error!("Ollama processing failed: {}", e);
            return Err(StatusCode::INTERNAL_SERVER_ERROR);
        }
    };

    // Add relevant data points
    match state.lab_context_service.get_relevant_data_points(&enriched_query.query, &enriched_query.context).await {
        Ok(data_points) => {
            response.relevant_data = data_points;
        }
        Err(e) => {
            error!("Failed to get relevant data points: {}", e);
        }
    }

    // Store query history (fire and forget)
    let user_id = Uuid::new_v4(); // In production, get from authentication
    let _ = state.lab_context_service.store_query_history(
        user_id,
        &enriched_query.query,
        &response.response,
        response.confidence,
        response.response_time_ms,
    ).await;

    info!("✅ Query processed successfully with confidence: {:.2}", response.confidence);
    Ok(Json(response))
}

pub async fn handle_proactive_suggestions(
    State(state): State<AppState>,
    Json(request): Json<ProactiveSuggestionRequest>,
) -> Result<Json<Vec<ProactiveSuggestion>>, StatusCode> {
    info!("🔮 Generating proactive suggestions for user role: {:?}", request.user_role);

    // Get AI-generated suggestions
    let ai_suggestions = match state.ollama_service.generate_proactive_suggestions(&request.user_context, &request.user_role).await {
        Ok(suggestions) => suggestions,
        Err(e) => {
            error!("Failed to generate AI suggestions: {}", e);
            return Err(StatusCode::INTERNAL_SERVER_ERROR);
        }
    };

    // Convert to structured suggestions
    let structured_suggestions: Vec<ProactiveSuggestion> = ai_suggestions
        .into_iter()
        .enumerate()
        .map(|(i, suggestion)| ProactiveSuggestion {
            suggestion_type: match i % 4 {
                0 => SuggestionType::Optimization,
                1 => SuggestionType::QualityAlert,
                2 => SuggestionType::ResourceManagement,
                _ => SuggestionType::WorkflowImprovement,
            },
            title: format!("Suggestion {}", i + 1),
            description: suggestion,
            urgency: if suggestion.to_lowercase().contains("critical") || suggestion.to_lowercase().contains("urgent") {
                Priority::High
            } else {
                Priority::Medium
            },
            potential_benefit: "Improved laboratory efficiency and quality".to_string(),
            action: None, // Could be enhanced with specific actions
        })
        .collect();

    info!("✅ Generated {} proactive suggestions", structured_suggestions.len());
    Ok(Json(structured_suggestions))
}

pub async fn handle_context_analysis(
    State(state): State<AppState>,
    Json(request): Json<AnalysisRequest>,
) -> Result<Json<AnalysisResult>, StatusCode> {
    info!("📊 Performing context analysis for data type: {}", request.data_type);

    // For now, return a simplified analysis result
    // In a full implementation, this would use specialized analytics engines
    let analysis_result = AnalysisResult {
        insights: vec![],
        visualizations: vec![],
        recommendations: vec![],
        confidence_score: 0.85,
        processing_time_ms: 250,
    };

    info!("✅ Context analysis completed");
    Ok(Json(analysis_result))
}

pub async fn handle_predictive_insights(
    State(state): State<AppState>,
    Json(query): Json<LabQuery>,
) -> Result<Json<Vec<crate::models::Prediction>>, StatusCode> {
    info!("🔮 Generating predictive insights");

    // Placeholder for predictive analytics
    // In a full implementation, this would use ML models and historical data
    let predictions = vec![];

    info!("✅ Predictive insights generated");
    Ok(Json(predictions))
}

pub async fn handle_lab_chat(
    State(state): State<AppState>,
    Json(message): Json<ChatMessage>,
) -> Result<Json<ChatResponse>, StatusCode> {
    info!("💬 Processing lab chat message from user: {}", message.user_id);

    // Create a lab query from the chat message
    let lab_query = LabQuery {
        query: message.message.clone(),
        context: message.context.unwrap_or_else(|| crate::models::LabContext {
            current_samples: vec![],
            active_workflows: vec![],
            user_department: "General".to_string(),
            lab_location: "Main Lab".to_string(),
            current_projects: vec![],
            recent_activities: vec![],
        }),
        user_role: crate::models::UserRole::ResearchScientist, // Default role
        query_type: crate::models::QueryType::Question,
        session_id: message.session_id,
    };

    // Process with the intelligent query handler
    let response = match state.ollama_service.process_lab_query(&lab_query).await {
        Ok(response) => response,
        Err(e) => {
            error!("Failed to process chat message: {}", e);
            return Err(StatusCode::INTERNAL_SERVER_ERROR);
        }
    };

    let chat_response = ChatResponse {
        response: response.response,
        session_id: message.session_id.unwrap_or_else(|| Uuid::new_v4()),
        suggestions: response.follow_up_questions,
        clarification_needed: response.confidence < 0.7,
        clarification_questions: if response.confidence < 0.7 {
            vec!["Could you provide more context about your request?".to_string()]
        } else {
            vec![]
        },
    };

    info!("✅ Lab chat response generated");
    Ok(Json(chat_response))
} 