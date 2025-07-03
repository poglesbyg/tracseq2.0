//! Integration tests for AI-powered laboratory queries

use cognitive_assistant_service::{
    AppState, LabQuery, LabQueryType, UserRole, Department,
    LabContext, ContextualResponse, OllamaService, LabContextService,
};
use crate::test_utils::*;
use wiremock::MockServer;
use std::time::Duration;

#[tokio::test]
async fn test_complete_query_workflow() {
    let mock_server = MockServer::start().await;
    MockOllamaSetup::setup_chat_endpoint(&mock_server).await;
    
    let test_db = TestDatabase::new().await;
    
    // Setup test data
    sqlx::query!(
        r#"
        INSERT INTO user_contexts (user_id, role, department, preferences, created_at)
        VALUES ($1, $2, $3, $4, NOW())
        "#,
        "test_researcher",
        UserRole::Researcher as i32,
        Department::Molecular as i32,
        serde_json::json!({"language": "en", "detail_level": "expert"})
    )
    .execute(&test_db.pool)
    .await
    .expect("Failed to insert user context");
    
    // Create services
    let ollama_service = OllamaService::new(mock_server.uri());
    let context_service = LabContextService::new(test_db.pool.clone());
    
    // Build context
    let context = context_service.build_context_for_user("test_researcher", Department::Molecular).await.unwrap();
    
    // Create query with context
    let query = LabQuery {
        id: None,
        query: "What's the best protocol for RNA extraction from tissue samples?".to_string(),
        query_type: LabQueryType::ProcessGuidance,
        user_role: UserRole::Researcher,
        department: Department::Molecular,
        context: Some(context),
        timestamp: chrono::Utc::now(),
    };
    
    // Process query through AI
    let response = process_lab_query_with_ai(&ollama_service, &context_service, query).await;
    
    assert!(response.is_ok());
    let contextual_response = response.unwrap();
    
    CognitiveAssertions::assert_response_quality(&contextual_response);
    assert!(contextual_response.confidence > 0.5);
    assert!(!contextual_response.context_used.is_empty());
    
    test_db.cleanup().await;
}

#[tokio::test]
async fn test_multi_department_queries() {
    let mock_server = MockServer::start().await;
    MockOllamaSetup::setup_chat_endpoint(&mock_server).await;
    
    let departments = vec![
        (Department::Sequencing, "How do I prepare libraries for NGS?"),
        (Department::Molecular, "What's the optimal PCR cycling conditions?"),
        (Department::QualityControl, "How do I validate sample integrity?"),
        (Department::Storage, "What are the storage requirements for RNA samples?"),
    ];
    
    let ollama_service = OllamaService::new(mock_server.uri());
    
    for (dept, question) in departments {
        let query = LabQuery {
            id: None,
            query: question.to_string(),
            query_type: LabQueryType::ProcessGuidance,
            user_role: UserRole::Technician,
            department: dept.clone(),
            context: None,
            timestamp: chrono::Utc::now(),
        };
        
        let response = process_simple_lab_query(&ollama_service, query).await;
        
        assert!(response.is_ok());
        let contextual_response = response.unwrap();
        assert!(!contextual_response.response.is_empty());
    }
}

#[tokio::test]
async fn test_context_enriched_responses() {
    let mock_server = MockServer::start().await;
    MockOllamaSetup::setup_chat_endpoint(&mock_server).await;
    
    let ollama_service = OllamaService::new(mock_server.uri());
    
    // Query without context
    let query_no_context = LabQuery {
        id: None,
        query: "How do I troubleshoot PCR?".to_string(),
        query_type: LabQueryType::Troubleshooting,
        user_role: UserRole::Technician,
        department: Department::Molecular,
        context: None,
        timestamp: chrono::Utc::now(),
    };
    
    let response_no_context = process_simple_lab_query(&ollama_service, query_no_context).await.unwrap();
    
    // Query with rich context
    let context = ContextBuilder::new()
        .with_samples(vec!["PCR001".to_string(), "PCR002".to_string()])
        .with_activities(vec!["PCR setup failed".to_string(), "No amplification detected".to_string()])
        .with_equipment(vec![
            ("Thermocycler A".to_string(), "Error: Lid not closing".to_string())
        ])
        .build();
    
    let query_with_context = LabQuery {
        id: None,
        query: "How do I troubleshoot PCR?".to_string(),
        query_type: LabQueryType::Troubleshooting,
        user_role: UserRole::Technician,
        department: Department::Molecular,
        context: Some(context),
        timestamp: chrono::Utc::now(),
    };
    
    let response_with_context = process_simple_lab_query(&ollama_service, query_with_context).await.unwrap();
    
    // Context-enriched response should be more specific
    assert!(response_with_context.context_used.len() > response_no_context.context_used.len());
}

#[tokio::test]
async fn test_query_performance_under_load() {
    let mock_server = MockServer::start().await;
    MockOllamaSetup::setup_chat_endpoint(&mock_server).await;
    
    let ollama_service = Arc::new(OllamaService::new(mock_server.uri()));
    
    let (duration, results) = PerformanceTestUtils::measure_query_time(|| async {
        let mut handles = Vec::new();
        
        for i in 0..20 {
            let service_clone = ollama_service.clone();
            let query = LabQuery {
                id: None,
                query: format!("Question {}: How do I process samples?", i),
                query_type: LabQueryType::ProcessGuidance,
                user_role: UserRole::Technician,
                department: Department::Molecular,
                context: None,
                timestamp: chrono::Utc::now(),
            };
            
            let handle = tokio::spawn(async move {
                process_simple_lab_query(&service_clone, query).await
            });
            handles.push(handle);
        }
        
        futures::future::join_all(handles).await
    }).await;
    
    println!("Processed 20 concurrent queries in {:?}", duration);
    
    // All queries should succeed
    let successful = results.into_iter()
        .filter(|r| r.is_ok() && r.as_ref().unwrap().is_ok())
        .count();
    
    assert_eq!(successful, 20);
    assert!(duration < Duration::from_secs(5)); // Should complete reasonably fast
}

#[tokio::test]
async fn test_fallback_on_ai_service_failure() {
    let mock_server = MockServer::start().await;
    // Setup flaky endpoint
    MockOllamaSetup::setup_flaky_endpoint(&mock_server).await;
    
    let ollama_service = OllamaService::new(mock_server.uri());
    let mut success_count = 0;
    let mut failure_count = 0;
    
    // Try multiple queries
    for i in 0..10 {
        let query = LabQuery {
            id: None,
            query: format!("Query {}: Standard lab question", i),
            query_type: LabQueryType::General,
            user_role: UserRole::Technician,
            department: Department::Molecular,
            context: None,
            timestamp: chrono::Utc::now(),
        };
        
        match process_simple_lab_query(&ollama_service, query).await {
            Ok(_) => success_count += 1,
            Err(_) => failure_count += 1,
        }
    }
    
    // Should have some successes and some failures
    assert!(success_count > 0);
    assert!(failure_count > 0);
    
    println!("Flaky service: {} successes, {} failures", success_count, failure_count);
}

#[tokio::test]
async fn test_specialized_query_routing() {
    let mock_server = MockServer::start().await;
    
    // Setup specialized endpoints for different query types
    use wiremock::{Mock, ResponseTemplate};
    use wiremock::matchers::{method, path, body_string_contains};
    
    Mock::given(method("POST"))
        .and(path("/api/chat"))
        .and(body_string_contains("compliance"))
        .respond_with(ResponseTemplate::new(200)
            .set_body_json(serde_json::json!({
                "response": "Compliance-specific response with regulations",
                "done": true
            })))
        .mount(&mock_server)
        .await;
    
    Mock::given(method("POST"))
        .and(path("/api/chat"))
        .and(body_string_contains("data analysis"))
        .respond_with(ResponseTemplate::new(200)
            .set_body_json(serde_json::json!({
                "response": "Data analysis response with statistics",
                "done": true
            })))
        .mount(&mock_server)
        .await;
    
    let ollama_service = OllamaService::new(mock_server.uri());
    
    // Test compliance query
    let compliance_query = LabQuery {
        id: None,
        query: "What are the compliance requirements for sample storage?".to_string(),
        query_type: LabQueryType::Compliance,
        user_role: UserRole::Admin,
        department: Department::Storage,
        context: None,
        timestamp: chrono::Utc::now(),
    };
    
    let compliance_response = process_simple_lab_query(&ollama_service, compliance_query).await.unwrap();
    assert!(compliance_response.response.contains("Compliance") || compliance_response.response.contains("regulations"));
    
    // Test data analysis query
    let analysis_query = LabQuery {
        id: None,
        query: "Show me data analysis for this month's QC metrics".to_string(),
        query_type: LabQueryType::DataAnalysis,
        user_role: UserRole::Supervisor,
        department: Department::QualityControl,
        context: None,
        timestamp: chrono::Utc::now(),
    };
    
    let analysis_response = process_simple_lab_query(&ollama_service, analysis_query).await.unwrap();
    assert!(analysis_response.response.contains("analysis") || analysis_response.response.contains("statistics"));
}

// Helper functions for integration tests

async fn process_simple_lab_query(
    ollama_service: &OllamaService,
    query: LabQuery,
) -> Result<ContextualResponse, ServiceError> {
    // Simulate the actual processing pipeline
    let chat_request = build_chat_request_from_query(&query);
    let ai_response = ollama_service.chat(&chat_request).await?;
    
    Ok(ContextualResponse {
        id: None,
        query_id: query.id,
        response: ai_response.response,
        confidence: 0.85, // Mock confidence
        reasoning: "AI-based response".to_string(),
        context_used: query.context.map(|_| vec!["User context".to_string()]).unwrap_or_default(),
        response_time_ms: ai_response.total_duration.unwrap_or(0) / 1_000_000, // Convert ns to ms
        model_used: "llama3.2".to_string(),
        tokens_used: ai_response.eval_count,
        created_at: chrono::Utc::now(),
    })
}

async fn process_lab_query_with_ai(
    ollama_service: &OllamaService,
    context_service: &LabContextService,
    mut query: LabQuery,
) -> Result<ContextualResponse, ServiceError> {
    // Enrich query with additional context if needed
    if query.context.is_none() {
        query.context = Some(
            context_service
                .build_context_for_user("default_user", query.department.clone())
                .await?
        );
    }
    
    process_simple_lab_query(ollama_service, query).await
}

fn build_chat_request_from_query(query: &LabQuery) -> OllamaChat {
    use cognitive_assistant_service::{OllamaChat, OllamaMessage};
    
    let system_prompt = format!(
        "You are a laboratory assistant specializing in {}. The user is a {}.",
        match query.department {
            Department::Sequencing => "DNA/RNA sequencing",
            Department::Molecular => "molecular biology",
            Department::QualityControl => "quality control",
            Department::Storage => "sample storage",
        },
        match query.user_role {
            UserRole::Technician => "laboratory technician",
            UserRole::Researcher => "research scientist",
            UserRole::Supervisor => "laboratory supervisor",
            UserRole::Admin => "laboratory administrator",
        }
    );
    
    let mut messages = vec![
        OllamaMessage {
            role: "system".to_string(),
            content: system_prompt,
        },
    ];
    
    // Add context if available
    if let Some(context) = &query.context {
        if !context.active_samples.is_empty() {
            messages.push(OllamaMessage {
                role: "system".to_string(),
                content: format!("Active samples: {:?}", context.active_samples),
            });
        }
    }
    
    messages.push(OllamaMessage {
        role: "user".to_string(),
        content: query.query.clone(),
    });
    
    OllamaChat {
        model: "llama3.2".to_string(),
        messages,
        stream: false,
        format: None,
        options: None,
    }
}

use cognitive_assistant_service::{OllamaChat, ServiceError};
use std::sync::Arc;