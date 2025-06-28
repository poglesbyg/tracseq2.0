//! HTTP test utilities for Axum applications

use anyhow::Result;
use axum::{body::Body, Router};
use axum_test::{TestServer as AxumTestServer, TestServerConfig};
use hyper::{Method, StatusCode};
use serde::{de::DeserializeOwned, Serialize};
use std::fmt::Debug;

/// Test server wrapper with additional utilities
pub struct TestServer {
    inner: AxumTestServer,
    base_url: String,
}

impl TestServer {
    /// Create a new test server
    pub async fn new(app: Router) -> Result<Self> {
        let config = TestServerConfig::builder()
            .save_cookies()
            .build();
            
        let server = AxumTestServer::new_with_config(app, config)?;
        let base_url = server.server_address()
            .map(|url| url.to_string())
            .unwrap_or_else(|| "http://localhost".to_string());
        
        Ok(Self {
            inner: server,
            base_url,
        })
    }
    
    /// Get the base URL of the test server
    pub fn base_url(&self) -> &str {
        &self.base_url
    }
    
    /// Create a GET request
    pub fn get(&self, path: &str) -> TestRequest {
        TestRequest::new(self, Method::GET, path)
    }
    
    /// Create a POST request
    pub fn post(&self, path: &str) -> TestRequest {
        TestRequest::new(self, Method::POST, path)
    }
    
    /// Create a PUT request
    pub fn put(&self, path: &str) -> TestRequest {
        TestRequest::new(self, Method::PUT, path)
    }
    
    /// Create a DELETE request
    pub fn delete(&self, path: &str) -> TestRequest {
        TestRequest::new(self, Method::DELETE, path)
    }
    
    /// Create a PATCH request
    pub fn patch(&self, path: &str) -> TestRequest {
        TestRequest::new(self, Method::PATCH, path)
    }
    
    /// Get the inner test server for direct access
    pub fn inner(&self) -> &AxumTestServer {
        &self.inner
    }
}

/// Test request builder
pub struct TestRequest<'a> {
    server: &'a TestServer,
    method: Method,
    path: String,
    headers: Vec<(String, String)>,
    body: Option<Body>,
}

impl<'a> TestRequest<'a> {
    fn new(server: &'a TestServer, method: Method, path: &str) -> Self {
        Self {
            server,
            method,
            path: path.to_string(),
            headers: Vec::new(),
            body: None,
        }
    }
    
    /// Add a header to the request
    pub fn header(mut self, key: &str, value: &str) -> Self {
        self.headers.push((key.to_string(), value.to_string()));
        self
    }
    
    /// Add an authorization header
    pub fn auth(self, token: &str) -> Self {
        self.header("Authorization", &format!("Bearer {}", token))
    }
    
    /// Set JSON body
    pub fn json<T: Serialize>(mut self, body: &T) -> Self {
        let json_body = serde_json::to_string(body).expect("Failed to serialize body");
        self.body = Some(Body::from(json_body));
        self.header("Content-Type", "application/json")
    }
    
    /// Set raw body
    pub fn body(mut self, body: Body) -> Self {
        self.body = Some(body);
        self
    }
    
    /// Send the request and get the response
    pub async fn send(self) -> TestResponse {
        let mut req = match self.method {
            Method::GET => self.server.inner.get(&self.path),
            Method::POST => self.server.inner.post(&self.path),
            Method::PUT => self.server.inner.put(&self.path),
            Method::DELETE => self.server.inner.delete(&self.path),
            Method::PATCH => self.server.inner.patch(&self.path),
            _ => panic!("Unsupported method: {}", self.method),
        };
        
        for (key, value) in self.headers {
            req = req.add_header(
                key.parse::<hyper::header::HeaderName>().unwrap(), 
                value.parse::<hyper::header::HeaderValue>().unwrap()
            );
        }
        
        if let Some(body) = self.body {
            req = req.bytes(axum::body::to_bytes(body, usize::MAX).await.unwrap());
        }
        
        let response = req.await;
        
        TestResponse {
            inner: response,
        }
    }
}

/// Test response wrapper
pub struct TestResponse {
    inner: axum_test::TestResponse,
}

impl TestResponse {
    /// Get the status code
    pub fn status(&self) -> StatusCode {
        self.inner.status_code()
    }
    
    /// Check if the response was successful (2xx)
    pub fn is_success(&self) -> bool {
        self.status().is_success()
    }
    
    /// Get response body as JSON
    pub async fn json<T: DeserializeOwned>(&self) -> T {
        self.inner.json()
    }
    
    /// Get response body as text
    pub async fn text(&self) -> String {
        self.inner.text()
    }
    
    /// Get response body as bytes
    pub async fn bytes(&self) -> Vec<u8> {
        self.inner.as_bytes().to_vec()
    }
    
    /// Get a header value
    pub fn header(&self, key: &str) -> Option<String> {
        self.inner
            .headers()
            .get(key)
            .and_then(|v| v.to_str().ok())
            .map(|s| s.to_string())
    }
    
    /// Assert status code
    pub fn assert_status(&self, expected: StatusCode) {
        assert_eq!(
            self.status(),
            expected,
            "Expected status {}, got {}",
            expected,
            self.status()
        );
    }
    
    /// Assert response is successful
    pub fn assert_success(&self) {
        assert!(
            self.is_success(),
            "Expected successful response, got {}",
            self.status()
        );
    }
    
    /// Assert JSON response matches expected value
    pub async fn assert_json<T>(&self, expected: &T)
    where
        T: DeserializeOwned + PartialEq + Debug,
    {
        let actual: T = self.json().await;
        assert_eq!(actual, *expected);
    }
}

/// Common test assertions
pub mod assertions {
    use super::*;
    
    /// Assert that a response has a specific error message
    pub async fn assert_error_message(response: &TestResponse, expected_message: &str) {
        let body: serde_json::Value = response.json().await;
        let message = body["error"].as_str().or(body["message"].as_str());
        
        assert!(
            message.is_some(),
            "Expected error message in response, got: {:?}",
            body
        );
        
        assert!(
            message.unwrap().contains(expected_message),
            "Expected error message to contain '{}', got: '{}'",
            expected_message,
            message.unwrap()
        );
    }
    
    /// Assert that a response has a specific success field
    pub async fn assert_success_response(response: &TestResponse) {
        response.assert_success();
        let body: serde_json::Value = response.json().await;
        
        assert_eq!(
            body["success"].as_bool(),
            Some(true),
            "Expected success: true in response, got: {:?}",
            body
        );
    }
}

/// Create a test router with common middleware
pub fn create_test_router() -> Router {
    Router::new()
        .layer(tower_http::trace::TraceLayer::new_for_http())
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::routing::get;
    
    #[tokio::test]
    async fn test_server_creation() {
        let app = Router::new().route("/health", get(|| async { "OK" }));
        let server = TestServer::new(app).await.unwrap();
        
        let response = server.get("/health").send().await;
        
        response.assert_success();
        assert_eq!(response.text().await, "OK");
    }
    
    #[tokio::test]
    async fn test_json_request_response() {
        use serde_json::json;
        
        let app = Router::new().route(
            "/echo",
            axum::routing::post(|axum::Json(body): axum::Json<serde_json::Value>| async move {
                axum::Json(json!({
                    "received": body,
                    "success": true
                }))
            }),
        );
        
        let server = TestServer::new(app).await.unwrap();
        let test_data = json!({"test": "data"});
        
        let response = server
            .post("/echo")
            .json(&test_data)
            .send()
            .await;
        
        response.assert_success();
        
        let body: serde_json::Value = response.json().await;
        assert_eq!(body["received"], test_data);
        assert_eq!(body["success"], true);
    }
}