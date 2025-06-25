// Minimal validation test for axum-test integration
use axum::{response::Json, routing::get, Router};
use axum_test::TestServer;
use serde_json::json;

async fn hello_handler() -> Json<serde_json::Value> {
    Json(json!({
        "message": "Hello from notification service!",
        "status": "running",
        "timestamp": chrono::Utc::now()
    }))
}

fn create_minimal_app() -> Router {
    Router::new().route("/health", get(hello_handler))
}

#[tokio::test]
async fn test_axum_test_integration() {
    // Create a minimal axum app
    let app = create_minimal_app();
    
    // Create test server
    let server = TestServer::new(app).unwrap();
    
    // Test GET request
    let response = server.get("/health").await;
    
    // Validate response
    assert_eq!(response.status_code(), axum::http::StatusCode::OK);
    
    let data: serde_json::Value = response.json();
    assert_eq!(data["message"], "Hello from notification service!");
    assert_eq!(data["status"], "running");
    assert!(data["timestamp"].is_string());
    
    println!("✅ Axum-test integration validated successfully!");
}

#[tokio::test]
async fn test_json_post_request() {
    let app = Router::new().route("/echo", axum::routing::post(|payload: Json<serde_json::Value>| async move {
        Json(json!({
            "received": payload.0,
            "echo": true
        }))
    }));
    
    let server = TestServer::new(app).unwrap();
    
    let test_payload = json!({
        "test": "data",
        "number": 42,
        "array": [1, 2, 3]
    });
    
    let response = server.post("/echo").json(&test_payload).await;
    
    assert_eq!(response.status_code(), axum::http::StatusCode::OK);
    
    let data: serde_json::Value = response.json();
    assert_eq!(data["echo"], true);
    assert_eq!(data["received"], test_payload);
    
    println!("✅ JSON POST request validation successful!");
}

#[tokio::test]
async fn test_uuid_handling() {
    use uuid::Uuid;
    
    let app = Router::new().route("/uuid/:id", axum::routing::get(|axum::extract::Path(id): axum::extract::Path<Uuid>| async move {
        Json(json!({
            "received_uuid": id,
            "is_valid": true
        }))
    }));
    
    let server = TestServer::new(app).unwrap();
    let test_uuid = Uuid::new_v4();
    
    let response = server.get(&format!("/uuid/{}", test_uuid)).await;
    
    assert_eq!(response.status_code(), axum::http::StatusCode::OK);
    
    let data: serde_json::Value = response.json();
    assert_eq!(data["received_uuid"], test_uuid.to_string());
    assert_eq!(data["is_valid"], true);
    
    println!("✅ UUID handling validation successful!");
}

#[cfg(test)]
mod validation_tests {
    use super::*;
    
    #[test]
    fn test_basic_dependencies() {
        // Test that our core dependencies are available
        let _uuid = uuid::Uuid::new_v4();
        let _now = chrono::Utc::now();
        let _json = serde_json::json!({"test": true});
        
        println!("✅ Core dependencies available!");
    }
}