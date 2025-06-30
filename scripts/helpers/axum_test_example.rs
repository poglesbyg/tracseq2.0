// Simple axum-test example
use axum::{response::Json, routing::get, Router};
use axum_test::TestServer;
use serde_json::json;

async fn hello_handler() -> Json<serde_json::Value> {
    Json(json!({
        "message": "Hello from TracSeq 2.0!",
        "status": "running",
        "timestamp": chrono::Utc::now()
    }))
}

async fn echo_handler(Json(payload): Json<serde_json::Value>) -> Json<serde_json::Value> {
    Json(json!({
        "received": payload,
        "echo": true
    }))
}

fn create_test_app() -> Router {
    Router::new()
        .route("/health", get(hello_handler))
        .route("/echo", axum::routing::post(echo_handler))
}

#[tokio::test]
async fn test_health_endpoint() {
    // Create test server with your axum app
    let server = TestServer::new(create_test_app()).unwrap();
    
    // Test GET request
    let response = server.get("/health").await;
    
    // Assert response
    assert_eq!(response.status_code(), axum::http::StatusCode::OK);
    
    let data: serde_json::Value = response.json();
    assert_eq!(data["message"], "Hello from TracSeq 2.0!");
    assert_eq!(data["status"], "running");
    
    println!("✅ Health endpoint test passed!");
}

#[tokio::test]
async fn test_echo_endpoint() {
    let server = TestServer::new(create_test_app()).unwrap();
    
    let test_payload = json!({
        "sample_id": "TRAC-001",
        "status": "processing",
        "priority": "high"
    });
    
    // Test POST request with JSON
    let response = server
        .post("/echo")
        .json(&test_payload)
        .await;
    
    assert_eq!(response.status_code(), axum::http::StatusCode::OK);
    
    let data: serde_json::Value = response.json();
    assert_eq!(data["echo"], true);
    assert_eq!(data["received"]["sample_id"], "TRAC-001");
    
    println!("✅ Echo endpoint test passed!");
}

#[tokio::test]
async fn test_not_found() {
    let server = TestServer::new(create_test_app()).unwrap();
    
    let response = server.get("/nonexistent").await;
    
    assert_eq!(response.status_code(), axum::http::StatusCode::NOT_FOUND);
    
    println!("✅ 404 test passed!");
} 
