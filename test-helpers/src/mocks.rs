//! Mock services and utilities for testing

use axum::{
    routing::{get, post},
    Router, Json,
    extract::{Path, State},
    http::StatusCode,
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::Mutex;
use uuid::Uuid;

/// Mock service state for tracking calls
#[derive(Clone)]
pub struct MockServiceState {
    pub calls: Arc<Mutex<Vec<MockCall>>>,
    pub responses: Arc<Mutex<Vec<MockResponse>>>,
}

/// Record of a mock service call
#[derive(Debug, Clone, Serialize)]
pub struct MockCall {
    pub method: String,
    pub path: String,
    pub body: Option<serde_json::Value>,
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

/// Configured mock response
#[derive(Debug, Clone)]
pub struct MockResponse {
    pub path_pattern: String,
    pub method: String,
    pub status: StatusCode,
    pub body: serde_json::Value,
}

impl MockServiceState {
    /// Create new mock service state
    pub fn new() -> Self {
        Self {
            calls: Arc::new(Mutex::new(Vec::new())),
            responses: Arc::new(Mutex::new(Vec::new())),
        }
    }
    
    /// Record a call
    pub async fn record_call(&self, method: &str, path: &str, body: Option<serde_json::Value>) {
        let mut calls = self.calls.lock().await;
        calls.push(MockCall {
            method: method.to_string(),
            path: path.to_string(),
            body,
            timestamp: chrono::Utc::now(),
        });
    }
    
    /// Add a mock response
    pub async fn add_response(
        &self,
        path_pattern: &str,
        method: &str,
        status: StatusCode,
        body: serde_json::Value,
    ) {
        let mut responses = self.responses.lock().await;
        responses.push(MockResponse {
            path_pattern: path_pattern.to_string(),
            method: method.to_string(),
            status,
            body,
        });
    }
    
    /// Get matching response
    pub async fn get_response(&self, method: &str, path: &str) -> Option<MockResponse> {
        let responses = self.responses.lock().await;
        responses
            .iter()
            .find(|r| r.method == method && path.contains(&r.path_pattern))
            .cloned()
    }
    
    /// Get all calls
    pub async fn get_calls(&self) -> Vec<MockCall> {
        self.calls.lock().await.clone()
    }
    
    /// Clear all calls
    pub async fn clear_calls(&self) {
        self.calls.lock().await.clear();
    }
    
    /// Assert a call was made
    pub async fn assert_called(&self, method: &str, path_contains: &str) {
        let calls = self.calls.lock().await;
        let found = calls
            .iter()
            .any(|c| c.method == method && c.path.contains(path_contains));
        
        assert!(
            found,
            "Expected call {} {} not found. Actual calls: {:?}",
            method,
            path_contains,
            calls.iter().map(|c| format!("{} {}", c.method, c.path)).collect::<Vec<_>>()
        );
    }
}

/// Create a mock auth service
pub fn create_mock_auth_service() -> (Router, MockServiceState) {
    let state = MockServiceState::new();
    
    let app = Router::new()
        .route("/health", get(|| async { Json(serde_json::json!({ "status": "healthy" })) }))
        .route("/validate/token", post({
            let state = state.clone();
            move |State(s): State<MockServiceState>, Json(body): Json<serde_json::Value>| {
                let state = s.clone();
                async move {
                    state.record_call("POST", "/validate/token", Some(body.clone())).await;
                    
                    if let Some(response) = state.get_response("POST", "/validate/token").await {
                        return (response.status, Json(response.body));
                    }
                    
                    // Default response
                    let token = body["token"].as_str().unwrap_or("");
                    if token.starts_with("valid_") {
                        (StatusCode::OK, Json(serde_json::json!({
                            "valid": true,
                            "user_id": "test-user-id",
                            "email": "test@example.com",
                            "role": "technician"
                        })))
                    } else {
                        (StatusCode::OK, Json(serde_json::json!({
                            "valid": false
                        })))
                    }
                }
            }
        }))
        .with_state(state.clone());
    
    (app, state)
}

/// Create a mock sample service
pub fn create_mock_sample_service() -> (Router, MockServiceState) {
    let state = MockServiceState::new();
    
    let app = Router::new()
        .route("/health", get(|| async { Json(serde_json::json!({ "status": "healthy" })) }))
        .route("/api/v1/samples", post({
            let state = state.clone();
            move |State(s): State<MockServiceState>, Json(body): Json<serde_json::Value>| {
                let state = s.clone();
                async move {
                    state.record_call("POST", "/api/v1/samples", Some(body.clone())).await;
                    
                    if let Some(response) = state.get_response("POST", "/api/v1/samples").await {
                        return (response.status, Json(response.body));
                    }
                    
                    // Default response
                    (StatusCode::CREATED, Json(serde_json::json!({
                        "id": Uuid::new_v4(),
                        "sample_id": format!("SMP-{}", Uuid::new_v4().to_string().split('-').next().unwrap()),
                        "name": body["name"],
                        "status": "created",
                        "created_at": chrono::Utc::now()
                    })))
                }
            }
        }))
        .route("/api/v1/samples/:id", get({
            let state = state.clone();
            move |State(s): State<MockServiceState>, Path(id): Path<String>| {
                let state = s.clone();
                async move {
                    state.record_call("GET", &format!("/api/v1/samples/{}", id), None).await;
                    
                    if let Some(response) = state.get_response("GET", &format!("/api/v1/samples/{}", id)).await {
                        return (response.status, Json(response.body));
                    }
                    
                    // Default response
                    (StatusCode::OK, Json(serde_json::json!({
                        "id": id,
                        "sample_id": format!("SMP-TEST"),
                        "name": "Test Sample",
                        "status": "active",
                        "created_at": chrono::Utc::now()
                    })))
                }
            }
        }))
        .with_state(state.clone());
    
    (app, state)
}

/// Create a mock notification service
pub fn create_mock_notification_service() -> (Router, MockServiceState) {
    let state = MockServiceState::new();
    
    let app = Router::new()
        .route("/health", get(|| async { Json(serde_json::json!({ "status": "healthy" })) }))
        .route("/api/v1/notifications/send", post({
            let state = state.clone();
            move |State(s): State<MockServiceState>, Json(body): Json<serde_json::Value>| {
                let state = s.clone();
                async move {
                    state.record_call("POST", "/api/v1/notifications/send", Some(body.clone())).await;
                    
                    if let Some(response) = state.get_response("POST", "/api/v1/notifications/send").await {
                        return (response.status, Json(response.body));
                    }
                    
                    // Default response
                    (StatusCode::OK, Json(serde_json::json!({
                        "notification_id": Uuid::new_v4(),
                        "status": "sent",
                        "sent_at": chrono::Utc::now()
                    })))
                }
            }
        }))
        .with_state(state.clone());
    
    (app, state)
}

/// Create a mock event service
pub fn create_mock_event_service() -> (Router, MockServiceState) {
    let state = MockServiceState::new();
    
    let app = Router::new()
        .route("/health", get(|| async { Json(serde_json::json!({ "status": "healthy" })) }))
        .route("/api/v1/events", post({
            let state = state.clone();
            move |State(s): State<MockServiceState>, Json(body): Json<serde_json::Value>| {
                let state = s.clone();
                async move {
                    state.record_call("POST", "/api/v1/events", Some(body.clone())).await;
                    
                    if let Some(response) = state.get_response("POST", "/api/v1/events").await {
                        return (response.status, Json(response.body));
                    }
                    
                    // Default response
                    (StatusCode::CREATED, Json(serde_json::json!({
                        "event_id": Uuid::new_v4(),
                        "status": "published",
                        "published_at": chrono::Utc::now()
                    })))
                }
            }
        }))
        .with_state(state.clone());
    
    (app, state)
}

/// Mock service collection for integration tests
pub struct MockServices {
    pub auth: (Router, MockServiceState),
    pub sample: (Router, MockServiceState),
    pub notification: (Router, MockServiceState),
    pub event: (Router, MockServiceState),
}

impl MockServices {
    /// Create all mock services
    pub fn new() -> Self {
        Self {
            auth: create_mock_auth_service(),
            sample: create_mock_sample_service(),
            notification: create_mock_notification_service(),
            event: create_mock_event_service(),
        }
    }
    
    /// Create a combined router for all services
    pub fn combined_router(&self) -> Router {
        Router::new()
            .nest("/auth", self.auth.0.clone())
            .nest("/sample", self.sample.0.clone())
            .nest("/notification", self.notification.0.clone())
            .nest("/event", self.event.0.clone())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    async fn test_mock_service_state() {
        let state = MockServiceState::new();
        
        // Record a call
        state.record_call("GET", "/test", None).await;
        
        // Check it was recorded
        let calls = state.get_calls().await;
        assert_eq!(calls.len(), 1);
        assert_eq!(calls[0].method, "GET");
        assert_eq!(calls[0].path, "/test");
    }
    
    #[tokio::test]
    async fn test_mock_response() {
        let state = MockServiceState::new();
        
        // Add a mock response
        state.add_response(
            "/test",
            "GET",
            StatusCode::OK,
            serde_json::json!({ "test": "response" })
        ).await;
        
        // Get the response
        let response = state.get_response("GET", "/test").await;
        assert!(response.is_some());
        
        let response = response.unwrap();
        assert_eq!(response.status, StatusCode::OK);
        assert_eq!(response.body["test"], "response");
    }
}