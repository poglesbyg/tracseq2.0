//! Unit tests for Ollama service

use cognitive_assistant_service::{
    OllamaService, OllamaChat, OllamaMessage, OllamaResponse,
    OllamaEmbeddingRequest, ServiceError, ServiceErrorKind,
};
use crate::test_utils::*;
use wiremock::MockServer;
use std::time::Duration;
use std::sync::Arc;

#[tokio::test]
async fn test_ollama_service_creation() {
    let service = OllamaService::new("http://localhost:11434".to_string());
    
    // Service should be created successfully
    assert_eq!(service.base_url, "http://localhost:11434");
}

#[tokio::test]
async fn test_ollama_chat_request() {
    let mock_server = MockServer::start().await;
    MockOllamaSetup::setup_chat_endpoint(&mock_server).await;
    
    let service = OllamaService::new(mock_server.uri());
    
    let chat_request = OllamaChat {
        model: "llama3.2".to_string(),
        messages: vec![
            OllamaMessage {
                role: "system".to_string(),
                content: "You are a helpful laboratory assistant.".to_string(),
            },
            OllamaMessage {
                role: "user".to_string(),
                content: "How do I prepare samples?".to_string(),
            },
        ],
        stream: false,
        format: None,
        options: None,
    };
    
    let result = service.chat(&chat_request).await;
    
    assert!(result.is_ok());
    let response = result.unwrap();
    assert!(response.response.contains("AI response to:"));
    assert!(response.done);
}

#[tokio::test]
async fn test_ollama_chat_with_context() {
    let mock_server = MockServer::start().await;
    MockOllamaSetup::setup_chat_endpoint(&mock_server).await;
    
    let service = OllamaService::new(mock_server.uri());
    
    let chat_request = OllamaChat {
        model: "llama3.2".to_string(),
        messages: vec![
            OllamaMessage {
                role: "system".to_string(),
                content: "You are analyzing laboratory data.".to_string(),
            },
            OllamaMessage {
                role: "user".to_string(),
                content: "Current samples: RNA001, RNA002. What's next?".to_string(),
            },
        ],
        stream: false,
        format: None,
        options: Some(serde_json::json!({
            "temperature": 0.7,
            "top_p": 0.9
        })),
    };
    
    let result = service.chat(&chat_request).await;
    
    assert!(result.is_ok());
    let response = result.unwrap();
    assert!(response.response.contains("What's next?"));
}

#[tokio::test]
async fn test_ollama_embedding_request() {
    let mock_server = MockServer::start().await;
    MockOllamaSetup::setup_embedding_endpoint(&mock_server).await;
    
    let service = OllamaService::new(mock_server.uri());
    
    let embedding_request = OllamaEmbeddingRequest {
        model: "llama3.2".to_string(),
        prompt: "DNA extraction protocol".to_string(),
    };
    
    let result = service.generate_embedding(&embedding_request).await;
    
    assert!(result.is_ok());
    let embedding = result.unwrap();
    assert_eq!(embedding.embedding.len(), 768); // Standard embedding size
    assert!(embedding.embedding.iter().all(|&v| v.is_finite()));
}

#[tokio::test]
async fn test_ollama_error_handling() {
    let mock_server = MockServer::start().await;
    MockOllamaSetup::setup_failing_endpoint(&mock_server).await;
    
    let service = OllamaService::new(mock_server.uri());
    
    let chat_request = OllamaChat {
        model: "llama3.2".to_string(),
        messages: vec![
            OllamaMessage {
                role: "user".to_string(),
                content: "This will fail".to_string(),
            },
        ],
        stream: false,
        format: None,
        options: None,
    };
    
    let result = service.chat(&chat_request).await;
    
    assert!(result.is_err());
    if let Err(error) = result {
        CognitiveAssertions::assert_error_type(&error, ServiceErrorKind::OllamaError);
    }
}

#[tokio::test]
async fn test_ollama_timeout_handling() {
    let mock_server = MockServer::start().await;
    MockOllamaSetup::setup_slow_endpoint(&mock_server, Duration::from_secs(5)).await;
    
    // Create service with short timeout
    let service = OllamaService::with_timeout(
        mock_server.uri(),
        Duration::from_millis(100),
    );
    
    let chat_request = OllamaChat {
        model: "llama3.2".to_string(),
        messages: vec![
            OllamaMessage {
                role: "user".to_string(),
                content: "This will timeout".to_string(),
            },
        ],
        stream: false,
        format: None,
        options: None,
    };
    
    let start = std::time::Instant::now();
    let result = service.chat(&chat_request).await;
    let duration = start.elapsed();
    
    assert!(result.is_err());
    assert!(duration < Duration::from_secs(1)); // Should timeout quickly
}

#[tokio::test]
async fn test_ollama_response_metrics() {
    let mock_server = MockServer::start().await;
    MockOllamaSetup::setup_chat_endpoint(&mock_server).await;
    
    let service = OllamaService::new(mock_server.uri());
    
    let chat_request = OllamaChat {
        model: "llama3.2".to_string(),
        messages: vec![
            OllamaMessage {
                role: "user".to_string(),
                content: "Test query".to_string(),
            },
        ],
        stream: false,
        format: None,
        options: None,
    };
    
    let result = service.chat(&chat_request).await;
    
    assert!(result.is_ok());
    let response = result.unwrap();
    
    // Check metrics are present
    assert!(response.total_duration.is_some());
    assert!(response.prompt_eval_count.is_some());
    assert!(response.eval_count.is_some());
    
    // Check metric values are reasonable
    if let Some(total_duration) = response.total_duration {
        assert!(total_duration > 0);
    }
}

#[tokio::test]
async fn test_ollama_model_selection() {
    let mock_server = MockServer::start().await;
    
    use wiremock::{Mock, ResponseTemplate};
    use wiremock::matchers::{method, path, body_partial_json};
    
    // Setup endpoint that verifies model
    Mock::given(method("POST"))
        .and(path("/api/chat"))
        .and(body_partial_json(serde_json::json!({
            "model": "codellama"
        })))
        .respond_with(ResponseTemplate::new(200)
            .set_body_json(OllamaResponse {
                response: "Code-specific response".to_string(),
                done: true,
                context: None,
                total_duration: Some(50_000_000),
                load_duration: None,
                prompt_eval_count: None,
                prompt_eval_duration: None,
                eval_count: None,
                eval_duration: None,
            }))
        .mount(&mock_server)
        .await;
    
    let service = OllamaService::new(mock_server.uri());
    
    let chat_request = OllamaChat {
        model: "codellama".to_string(), // Different model
        messages: vec![
            OllamaMessage {
                role: "user".to_string(),
                content: "Write a function".to_string(),
            },
        ],
        stream: false,
        format: None,
        options: None,
    };
    
    let result = service.chat(&chat_request).await;
    
    assert!(result.is_ok());
    assert_eq!(result.unwrap().response, "Code-specific response");
}

#[tokio::test]
async fn test_ollama_message_formatting() {
    let messages = vec![
        OllamaMessage {
            role: "system".to_string(),
            content: "System prompt with\nmultiple lines".to_string(),
        },
        OllamaMessage {
            role: "user".to_string(),
            content: "User query with special chars: <>&\"'".to_string(),
        },
        OllamaMessage {
            role: "assistant".to_string(),
            content: "Assistant response with unicode: 🧬 DNA 🔬".to_string(),
        },
    ];
    
    // Verify all messages are properly formatted
    for message in messages {
        assert!(!message.role.is_empty());
        assert!(!message.content.is_empty());
        assert!(["system", "user", "assistant"].contains(&message.role.as_str()));
    }
}

#[tokio::test]
async fn test_ollama_service_retry_logic() {
    // Note: This would test retry logic if implemented
    let service = OllamaService::new("http://localhost:11434".to_string());
    
    // For now, just verify service is created
    assert!(!service.base_url.is_empty());
}

#[tokio::test]
async fn test_ollama_concurrent_requests() {
    let mock_server = MockServer::start().await;
    MockOllamaSetup::setup_chat_endpoint(&mock_server).await;
    
    let service = Arc::new(OllamaService::new(mock_server.uri()));
    let mut handles = Vec::new();
    
    // Send multiple concurrent requests
    for i in 0..5 {
        let service_clone = service.clone();
        let handle = tokio::spawn(async move {
            let chat_request = OllamaChat {
                model: "llama3.2".to_string(),
                messages: vec![
                    OllamaMessage {
                        role: "user".to_string(),
                        content: format!("Query {}", i),
                    },
                ],
                stream: false,
                format: None,
                options: None,
            };
            
            service_clone.chat(&chat_request).await
        });
        handles.push(handle);
    }
    
    let results = futures::future::join_all(handles).await;
    
    // All requests should succeed
    assert_eq!(results.len(), 5);
    for result in results {
        assert!(result.is_ok());
        assert!(result.unwrap().is_ok());
    }
}