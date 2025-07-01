//! Unit tests for cognitive handlers

use cognitive_assistant_service::{
    handlers::cognitive_handler::*,
    AppState, LabQuery, LabQueryType, UserRole, Department,
    ContextualResponse, ProactiveSuggestion, ServiceError,
};
use crate::test_utils::*;
use axum::{
    extract::{State, Json},
    http::StatusCode,
};
use wiremock::MockServer;

#[tokio::test]
async fn test_handle_lab_query_success() {
    let mock_server = MockServer::start().await;
    MockOllamaSetup::setup_chat_endpoint(&mock_server).await;
    
    let app_state = create_test_app_state(mock_server.uri());
    let query = QueryFactory::sample_processing_query();
    
    let result = handle_lab_query(
        State(app_state),
        Json(query.clone())
    ).await;
    
    assert!(result.is_ok());
    let Json(response) = result.unwrap();
    
    CognitiveAssertions::assert_response_quality(&response);
    assert!(response.response.contains("AI response to:"));
}

#[tokio::test]
async fn test_handle_lab_query_with_context() {
    let mock_server = MockServer::start().await;
    MockOllamaSetup::setup_chat_endpoint(&mock_server).await;
    
    let app_state = create_test_app_state(mock_server.uri());
    let mut query = QueryFactory::troubleshooting_query();
    query.context = Some(TestDataGenerator::lab_context());
    
    let result = handle_lab_query(
        State(app_state),
        Json(query.clone())
    ).await;
    
    assert!(result.is_ok());
    let Json(response) = result.unwrap();
    
    // Should use context in response
    assert!(!response.context_used.is_empty());
}

#[tokio::test]
async fn test_handle_different_query_types() {
    let mock_server = MockServer::start().await;
    MockOllamaSetup::setup_chat_endpoint(&mock_server).await;
    
    let app_state = create_test_app_state(mock_server.uri());
    
    let queries = vec![
        QueryFactory::sample_processing_query(),
        QueryFactory::troubleshooting_query(),
        QueryFactory::data_analysis_query(),
        QueryFactory::compliance_query(),
    ];
    
    for query in queries {
        let result = handle_lab_query(
            State(app_state.clone()),
            Json(query.clone())
        ).await;
        
        assert!(result.is_ok());
        let Json(response) = result.unwrap();
        assert!(!response.response.is_empty());
    }
}

#[tokio::test]
async fn test_handle_lab_query_ollama_error() {
    let mock_server = MockServer::start().await;
    MockOllamaSetup::setup_failing_endpoint(&mock_server).await;
    
    let app_state = create_test_app_state(mock_server.uri());
    let query = QueryFactory::sample_processing_query();
    
    let result = handle_lab_query(
        State(app_state),
        Json(query)
    ).await;
    
    assert!(result.is_err());
    assert_eq!(result.unwrap_err(), StatusCode::INTERNAL_SERVER_ERROR);
}

#[tokio::test]
async fn test_generate_proactive_suggestions() {
    let mock_server = MockServer::start().await;
    MockOllamaSetup::setup_chat_endpoint(&mock_server).await;
    
    let app_state = create_test_app_state(mock_server.uri());
    
    let result = generate_proactive_suggestions(
        State(app_state),
        Json(Department::Molecular)
    ).await;
    
    assert!(result.is_ok());
    let Json(suggestions) = result.unwrap();
    
    assert!(!suggestions.is_empty());
    for suggestion in &suggestions {
        CognitiveAssertions::assert_suggestion_validity(suggestion);
    }
}

#[tokio::test]
async fn test_analyze_lab_context() {
    let mock_server = MockServer::start().await;
    MockOllamaSetup::setup_chat_endpoint(&mock_server).await;
    
    let app_state = create_test_app_state(mock_server.uri());
    let context = TestDataGenerator::lab_context();
    
    let result = analyze_lab_context(
        State(app_state),
        Json(context.clone())
    ).await;
    
    assert!(result.is_ok());
    let Json(analysis) = result.unwrap();
    
    // Should analyze equipment status
    assert!(analysis.equipment_issues.is_some());
    
    // Should provide recommendations
    assert!(!analysis.recommendations.is_empty());
}

#[tokio::test]
async fn test_get_contextual_help() {
    let mock_server = MockServer::start().await;
    MockOllamaSetup::setup_chat_endpoint(&mock_server).await;
    
    let app_state = create_test_app_state(mock_server.uri());
    
    #[derive(serde::Deserialize)]
    struct HelpRequest {
        topic: String,
        user_role: UserRole,
        context: Option<String>,
    }
    
    let help_request = Json(HelpRequest {
        topic: "PCR troubleshooting".to_string(),
        user_role: UserRole::Technician,
        context: Some("Multiple failed reactions".to_string()),
    });
    
    let result = get_contextual_help(
        State(app_state),
        help_request
    ).await;
    
    assert!(result.is_ok());
    let Json(help_response) = result.unwrap();
    
    assert!(!help_response.content.is_empty());
    assert!(!help_response.related_topics.is_empty());
}

#[tokio::test]
async fn test_query_response_time_tracking() {
    let mock_server = MockServer::start().await;
    MockOllamaSetup::setup_chat_endpoint(&mock_server).await;
    
    let app_state = create_test_app_state(mock_server.uri());
    let query = QueryFactory::sample_processing_query();
    
    let start = std::time::Instant::now();
    let result = handle_lab_query(
        State(app_state),
        Json(query)
    ).await;
    let duration = start.elapsed();
    
    assert!(result.is_ok());
    let Json(response) = result.unwrap();
    
    // Response time should be tracked
    assert!(response.response_time_ms > 0);
    assert!(response.response_time_ms <= duration.as_millis() as u64);
}

#[tokio::test]
async fn test_user_role_based_responses() {
    let mock_server = MockServer::start().await;
    MockOllamaSetup::setup_chat_endpoint(&mock_server).await;
    
    let app_state = create_test_app_state(mock_server.uri());
    
    let roles = vec![
        UserRole::Technician,
        UserRole::Researcher,
        UserRole::Supervisor,
        UserRole::Admin,
    ];
    
    for role in roles {
        let mut query = QueryFactory::sample_processing_query();
        query.user_role = role;
        
        let result = handle_lab_query(
            State(app_state.clone()),
            Json(query)
        ).await;
        
        assert!(result.is_ok());
        // Different roles should get appropriate responses
    }
}

#[tokio::test]
async fn test_concurrent_query_handling() {
    let mock_server = MockServer::start().await;
    MockOllamaSetup::setup_chat_endpoint(&mock_server).await;
    
    let app_state = create_test_app_state(mock_server.uri());
    let mut handles = Vec::new();
    
    // Send multiple queries concurrently
    for i in 0..5 {
        let state_clone = app_state.clone();
        let mut query = QueryFactory::sample_processing_query();
        query.query = format!("Query {}: How do I process samples?", i);
        
        let handle = tokio::spawn(async move {
            handle_lab_query(
                State(state_clone),
                Json(query)
            ).await
        });
        handles.push(handle);
    }
    
    let results = futures::future::join_all(handles).await;
    
    // All should succeed
    assert_eq!(results.len(), 5);
    for result in results {
        assert!(result.is_ok());
        assert!(result.unwrap().is_ok());
    }
}

#[tokio::test]
async fn test_error_handling_graceful_degradation() {
    let mock_server = MockServer::start().await;
    // Don't set up any endpoints - simulate service unavailable
    
    let app_state = create_test_app_state(mock_server.uri());
    let query = QueryFactory::sample_processing_query();
    
    let result = handle_lab_query(
        State(app_state),
        Json(query)
    ).await;
    
    // Should return error status code, not panic
    assert!(result.is_err());
    assert_eq!(result.unwrap_err(), StatusCode::INTERNAL_SERVER_ERROR);
}

#[tokio::test]
async fn test_model_switching_based_on_query() {
    let mock_server = MockServer::start().await;
    
    use wiremock::{Mock, ResponseTemplate};
    use wiremock::matchers::{method, path, body_partial_json};
    
    // Setup different model endpoint
    Mock::given(method("POST"))
        .and(path("/api/chat"))
        .and(body_partial_json(serde_json::json!({
            "model": "codellama"
        })))
        .respond_with(ResponseTemplate::new(200)
            .set_body_json(serde_json::json!({
                "response": "Code-specific laboratory response",
                "done": true
            })))
        .mount(&mock_server)
        .await;
    
    let app_state = create_test_app_state(mock_server.uri());
    let mut query = QueryFactory::data_analysis_query();
    query.query = "Write a Python script to analyze qPCR data".to_string();
    
    let result = handle_lab_query(
        State(app_state),
        Json(query)
    ).await;
    
    assert!(result.is_ok());
    let Json(response) = result.unwrap();
    assert!(response.model_used.contains("llama") || response.model_used.contains("codellama"));
}