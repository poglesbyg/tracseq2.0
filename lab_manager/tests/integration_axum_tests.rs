use axum_test::TestServer;
use serde_json::{json, Value};
use lab_manager::{create_app, AppState};
use std::sync::Arc;
use tokio::sync::RwLock;

/// Integration tests for Lab Manager using axum-test
/// Tests all major API endpoints and functionality

async fn create_test_server() -> TestServer {
    // Create a test app state
    let app_state = Arc::new(RwLock::new(AppState::new_test().await));
    
    // Create the app with test configuration
    let app = create_app(app_state).await;
    
    TestServer::new(app).unwrap()
}

#[tokio::test]
async fn test_health_endpoint() {
    let server = create_test_server().await;
    
    let response = server.get("/health").await;
    
    assert_eq!(response.status_code(), 200);
    
    let data: Value = response.json();
    assert_eq!(data["status"], "healthy");
    assert!(data["timestamp"].is_string());
    assert_eq!(data["version"], "0.1.0");
    
    println!("✅ Health endpoint test passed");
}

#[tokio::test]
async fn test_dashboard_stats_endpoint() {
    let server = create_test_server().await;
    
    let response = server.get("/api/dashboard/stats").await;
    
    assert_eq!(response.status_code(), 200);
    
    let data: Value = response.json();
    assert!(data["totalTemplates"].is_number());
    assert!(data["totalSamples"].is_number());
    assert!(data["pendingSequencing"].is_number());
    assert!(data["completedSequencing"].is_number());
    
    println!("✅ Dashboard stats endpoint test passed");
}

#[tokio::test]
async fn test_templates_list_endpoint() {
    let server = create_test_server().await;
    
    let response = server.get("/api/templates").await;
    
    assert_eq!(response.status_code(), 200);
    
    let data: Value = response.json();
    assert!(data.is_array());
    
    println!("✅ Templates list endpoint test passed");
}

#[tokio::test]
async fn test_samples_list_endpoint() {
    let server = create_test_server().await;
    
    let response = server.get("/api/samples").await;
    
    assert_eq!(response.status_code(), 200);
    
    let data: Value = response.json();
    assert!(data.is_array());
    
    println!("✅ Samples list endpoint test passed");
}

#[tokio::test]
async fn test_sequencing_jobs_endpoint() {
    let server = create_test_server().await;
    
    let response = server.get("/api/sequencing/jobs").await;
    
    assert_eq!(response.status_code(), 200);
    
    let data: Value = response.json();
    assert!(data.is_array());
    
    println!("✅ Sequencing jobs endpoint test passed");
}

#[tokio::test]
async fn test_template_creation() {
    let server = create_test_server().await;
    
    let new_template = json!({
        "name": "Test Template",
        "description": "A test template for axum-test",
        "fields": [
            {
                "name": "sample_id",
                "field_type": "text",
                "required": true
            },
            {
                "name": "concentration",
                "field_type": "number",
                "required": false
            }
        ]
    });
    
    let response = server
        .post("/api/templates")
        .json(&new_template)
        .await;
    
    assert_eq!(response.status_code(), 201);
    
    let data: Value = response.json();
    assert_eq!(data["name"], "Test Template");
    assert!(data["id"].is_string());
    
    println!("✅ Template creation test passed");
}

#[tokio::test]
async fn test_template_creation_invalid_data() {
    let server = create_test_server().await;
    
    let invalid_template = json!({
        "name": "", // Empty name should fail validation
        "description": "Invalid template"
    });
    
    let response = server
        .post("/api/templates")
        .json(&invalid_template)
        .await;
    
    assert_eq!(response.status_code(), 422); // Unprocessable Entity
    
    println!("✅ Template validation test passed");
}

#[tokio::test]
async fn test_sample_creation() {
    let server = create_test_server().await;
    
    // First create a template
    let template = json!({
        "name": "Sample Template",
        "description": "Template for sample testing",
        "fields": [
            {
                "name": "sample_id",
                "field_type": "text",
                "required": true
            }
        ]
    });
    
    let template_response = server
        .post("/api/templates")
        .json(&template)
        .await;
    
    assert_eq!(template_response.status_code(), 201);
    let template_data: Value = template_response.json();
    let template_id = template_data["id"].as_str().unwrap();
    
    // Create a sample using the template
    let new_sample = json!({
        "template_id": template_id,
        "data": {
            "sample_id": "TEST-001"
        }
    });
    
    let response = server
        .post("/api/samples")
        .json(&new_sample)
        .await;
    
    assert_eq!(response.status_code(), 201);
    
    let data: Value = response.json();
    assert_eq!(data["template_id"], template_id);
    assert!(data["id"].is_string());
    
    println!("✅ Sample creation test passed");
}

#[tokio::test]
async fn test_authentication_endpoints() {
    let server = create_test_server().await;
    
    // Test login without credentials (should fail)
    let response = server
        .post("/api/auth/login")
        .json(&json!({}))
        .await;
    
    assert!(response.status_code() >= 400);
    
    // Test login with invalid credentials
    let invalid_login = json!({
        "email": "invalid@example.com",
        "password": "wrongpassword"
    });
    
    let response = server
        .post("/api/auth/login")
        .json(&invalid_login)
        .await;
    
    assert!(response.status_code() >= 400);
    
    println!("✅ Authentication endpoints test passed");
}

#[tokio::test]
async fn test_cors_headers() {
    let server = create_test_server().await;
    
    let response = server
        .get("/api/dashboard/stats")
        .add_header("Origin", "http://localhost:5173")
        .await;
    
    assert_eq!(response.status_code(), 200);
    
    let headers = response.headers();
    assert!(headers.contains_key("access-control-allow-origin"));
    
    println!("✅ CORS headers test passed");
}

#[tokio::test]
async fn test_error_handling() {
    let server = create_test_server().await;
    
    // Test non-existent endpoint
    let response = server.get("/api/nonexistent").await;
    assert_eq!(response.status_code(), 404);
    
    // Test invalid JSON in POST request
    let response = server
        .post("/api/templates")
        .text("invalid json")
        .await;
    
    assert!(response.status_code() >= 400);
    
    println!("✅ Error handling test passed");
}

#[tokio::test]
async fn test_concurrent_requests() {
    let server = create_test_server().await;
    
    // Make multiple concurrent requests
    let futures = (0..10).map(|_| {
        let server = &server;
        async move {
            server.get("/api/dashboard/stats").await
        }
    });
    
    let responses = futures::future::join_all(futures).await;
    
    // All requests should succeed
    for response in responses {
        assert_eq!(response.status_code(), 200);
    }
    
    println!("✅ Concurrent requests test passed");
}

#[tokio::test]
async fn test_request_validation() {
    let server = create_test_server().await;
    
    // Test various invalid requests
    let test_cases = vec![
        ("/api/templates", json!({"name": ""})), // Empty name
        ("/api/templates", json!({"description": "No name"})), // Missing name
        ("/api/samples", json!({"template_id": "invalid"})), // Invalid template ID
    ];
    
    for (endpoint, payload) in test_cases {
        let response = server
            .post(endpoint)
            .json(&payload)
            .await;
        
        assert!(response.status_code() >= 400, "Expected error for endpoint: {}", endpoint);
    }
    
    println!("✅ Request validation test passed");
}

#[tokio::test]
async fn test_performance_benchmarks() {
    let server = create_test_server().await;
    
    // Test response times
    let start = std::time::Instant::now();
    
    let response = server.get("/api/dashboard/stats").await;
    
    let duration = start.elapsed();
    
    assert_eq!(response.status_code(), 200);
    assert!(duration.as_millis() < 1000, "Response too slow: {:?}", duration);
    
    println!("✅ Performance benchmark test passed ({}ms)", duration.as_millis());
} 