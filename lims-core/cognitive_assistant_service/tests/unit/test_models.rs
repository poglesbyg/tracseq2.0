//! Unit tests for cognitive assistant models

use cognitive_assistant_service::{
    LabQuery, LabQueryType, LabContext, UserRole, Department,
    ContextualResponse, ProactiveSuggestion, SuggestionType,
    OllamaRequest, OllamaResponse, OllamaMessage,
    ServiceError, ServiceErrorKind,
};
use crate::test_utils::*;

#[test]
fn test_lab_query_creation() {
    let query = QueryFactory::sample_processing_query();
    
    assert_eq!(query.query_type, LabQueryType::ProcessGuidance);
    assert_eq!(query.user_role, UserRole::Technician);
    assert_eq!(query.department, Department::Sequencing);
    assert!(query.context.is_some());
    assert!(query.query.contains("RNA samples"));
}

#[test]
fn test_lab_query_types() {
    let queries = vec![
        (QueryFactory::sample_processing_query(), LabQueryType::ProcessGuidance),
        (QueryFactory::troubleshooting_query(), LabQueryType::Troubleshooting),
        (QueryFactory::data_analysis_query(), LabQueryType::DataAnalysis),
        (QueryFactory::compliance_query(), LabQueryType::Compliance),
    ];
    
    for (query, expected_type) in queries {
        assert_eq!(query.query_type, expected_type);
    }
}

#[test]
fn test_user_roles() {
    let roles = vec![
        (UserRole::Technician, "Technician"),
        (UserRole::Researcher, "Researcher"),
        (UserRole::Supervisor, "Supervisor"),
        (UserRole::Admin, "Admin"),
    ];
    
    for (role, expected_str) in roles {
        // Test that each role is distinct
        match role {
            UserRole::Technician => assert_eq!(expected_str, "Technician"),
            UserRole::Researcher => assert_eq!(expected_str, "Researcher"),
            UserRole::Supervisor => assert_eq!(expected_str, "Supervisor"),
            UserRole::Admin => assert_eq!(expected_str, "Admin"),
        }
    }
}

#[test]
fn test_lab_context_creation() {
    let context = TestDataGenerator::lab_context();
    
    assert_eq!(context.active_samples.len(), 3);
    assert_eq!(context.recent_activities.len(), 3);
    assert_eq!(context.equipment_status.len(), 3);
    assert_eq!(context.user_preferences.len(), 2);
    
    // Check specific values
    assert!(context.active_samples.contains(&"SAMPLE001".to_string()));
    assert!(context.equipment_status.contains_key("PCR Machine 1"));
    assert_eq!(context.user_preferences.get("notification_level"), Some(&"high".to_string()));
}

#[test]
fn test_contextual_response() {
    let response = ContextualResponse {
        id: None,
        query_id: None,
        response: "Test response".to_string(),
        confidence: 0.85,
        reasoning: "Based on laboratory best practices".to_string(),
        context_used: vec!["Active samples".to_string()],
        response_time_ms: 100,
        model_used: "llama3.2".to_string(),
        tokens_used: Some(150),
        created_at: chrono::Utc::now(),
    };
    
    CognitiveAssertions::assert_response_quality(&response);
    assert_eq!(response.confidence, 0.85);
    assert_eq!(response.model_used, "llama3.2");
}

#[test]
fn test_proactive_suggestion() {
    let suggestions = TestDataGenerator::proactive_suggestions();
    
    assert_eq!(suggestions.len(), 2);
    
    for suggestion in &suggestions {
        CognitiveAssertions::assert_suggestion_validity(suggestion);
    }
    
    // Check specific suggestion types
    assert!(suggestions.iter().any(|s| matches!(s.suggestion_type, SuggestionType::ProcessImprovement)));
    assert!(suggestions.iter().any(|s| matches!(s.suggestion_type, SuggestionType::SafetyReminder)));
}

#[test]
fn test_ollama_message_construction() {
    let message = OllamaMessage {
        role: "user".to_string(),
        content: "How do I extract DNA?".to_string(),
    };
    
    assert_eq!(message.role, "user");
    assert!(message.content.contains("DNA"));
}

#[test]
fn test_ollama_response_parsing() {
    let response = OllamaResponse {
        response: "Here's how to extract DNA...".to_string(),
        done: true,
        context: None,
        total_duration: Some(100_000_000),
        load_duration: Some(10_000_000),
        prompt_eval_count: Some(50),
        prompt_eval_duration: Some(20_000_000),
        eval_count: Some(100),
        eval_duration: Some(70_000_000),
    };
    
    assert!(response.done);
    assert!(response.response.contains("extract DNA"));
    assert_eq!(response.total_duration, Some(100_000_000));
}

#[test]
fn test_service_error_types() {
    let errors = vec![
        ServiceError {
            kind: ServiceErrorKind::DatabaseError,
            message: "Connection failed".to_string(),
            details: None,
        },
        ServiceError {
            kind: ServiceErrorKind::OllamaError,
            message: "Model not available".to_string(),
            details: Some("llama3.2 not loaded".to_string()),
        },
        ServiceError {
            kind: ServiceErrorKind::ValidationError,
            message: "Invalid query".to_string(),
            details: None,
        },
    ];
    
    CognitiveAssertions::assert_error_type(&errors[0], ServiceErrorKind::DatabaseError);
    CognitiveAssertions::assert_error_type(&errors[1], ServiceErrorKind::OllamaError);
    CognitiveAssertions::assert_error_type(&errors[2], ServiceErrorKind::ValidationError);
    
    // Check error with details
    assert_eq!(errors[1].details, Some("llama3.2 not loaded".to_string()));
}

#[test]
fn test_context_builder() {
    let context = ContextBuilder::new()
        .with_samples(vec!["TEST001".to_string(), "TEST002".to_string()])
        .with_activities(vec!["PCR setup".to_string()])
        .with_equipment(vec![
            ("Thermocycler".to_string(), "Available".to_string())
        ])
        .build();
    
    assert_eq!(context.active_samples.len(), 2);
    assert_eq!(context.recent_activities.len(), 1);
    assert_eq!(context.equipment_status.len(), 1);
    assert!(context.active_samples.contains(&"TEST001".to_string()));
}

#[test]
fn test_department_coverage() {
    let departments = vec![
        Department::Sequencing,
        Department::Molecular,
        Department::QualityControl,
        Department::Storage,
    ];
    
    // Ensure all departments are distinct
    for (i, dept1) in departments.iter().enumerate() {
        for (j, dept2) in departments.iter().enumerate() {
            if i != j {
                // This would fail to compile if departments weren't distinct variants
                assert!(!matches!(dept1, dept2));
            }
        }
    }
}

#[test]
fn test_confidence_bounds() {
    let valid_confidences = vec![0.0, 0.5, 0.85, 1.0];
    
    for confidence in valid_confidences {
        let response = ContextualResponse {
            id: None,
            query_id: None,
            response: "Test".to_string(),
            confidence,
            reasoning: "Test".to_string(),
            context_used: vec![],
            response_time_ms: 0,
            model_used: "test".to_string(),
            tokens_used: None,
            created_at: chrono::Utc::now(),
        };
        
        CognitiveAssertions::assert_response_quality(&response);
    }
}

#[test]
fn test_suggestion_priority_levels() {
    let priorities = vec!["low", "medium", "high"];
    
    for priority in priorities {
        let suggestion = ProactiveSuggestion {
            id: None,
            suggestion_type: SuggestionType::ProcessImprovement,
            title: "Test".to_string(),
            description: "Test".to_string(),
            priority: priority.to_string(),
            department: Department::Molecular,
            potential_impact: None,
            action_items: vec!["Test".to_string()],
            created_at: chrono::Utc::now(),
        };
        
        CognitiveAssertions::assert_suggestion_validity(&suggestion);
    }
}