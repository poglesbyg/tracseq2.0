use auth_service::{AppState, AuthError, Config, DatabasePool, models::*};
use auth_service::services::AuthServiceImpl;
use axum::{
    Router,
    body::Body,
    http::{Request, StatusCode},
};
use axum_test::TestServer;
use fake::{Fake, Faker};
use once_cell::sync::Lazy;
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use std::sync::Arc;
use uuid::Uuid;

/// Test database manager for isolated test environments
pub struct TestDatabase {
    pub pool: PgPool,
    pub tracked_users: Vec<Uuid>,
}

impl TestDatabase {
    pub async fn new() -> Self {
        let pool = auth_service::test_utils::get_test_db().await.clone();
        Self {
            pool,
            tracked_users: Vec::new(),
        }
    }

    pub async fn cleanup(&mut self) {
        for user_id in &self.tracked_users {
            let _ = sqlx::query("DELETE FROM user_sessions WHERE user_id = $1")
                .bind(user_id)
                .execute(&self.pool)
                .await;
            let _ = sqlx::query("DELETE FROM users WHERE id = $1")
                .bind(user_id)
                .execute(&self.pool)
                .await;
        }
        self.tracked_users.clear();
    }

    pub fn track_user(&mut self, user_id: Uuid) {
        self.tracked_users.push(user_id);
    }
}

impl Drop for TestDatabase {
    fn drop(&mut self) {
        let pool = self.pool.clone();
        let users = self.tracked_users.clone();
        tokio::spawn(async move {
            for user_id in users {
                let _ = sqlx::query("DELETE FROM user_sessions WHERE user_id = $1")
                    .bind(user_id)
                    .execute(&pool)
                    .await;
                let _ = sqlx::query("DELETE FROM users WHERE id = $1")
                    .bind(user_id)
                    .execute(&pool)
                    .await;
            }
        });
    }
}

/// Factory for creating test users with realistic data
pub struct UserFactory;

impl UserFactory {
    pub fn create_valid_register_request() -> auth_service::handlers::auth::RegisterRequest {
        auth_service::handlers::auth::RegisterRequest {
            first_name: "Test".to_string(),
            last_name: "User".to_string(),
            email: format!("test{}@example.com", Uuid::new_v4()),
            password: "SecurePassword123!".to_string(),
            department: Some("Engineering".to_string()),
            position: Some("Developer".to_string()),
            lab_affiliation: Some("Lab A".to_string()),
        }
    }

    pub fn create_invalid_register_request() -> auth_service::handlers::auth::RegisterRequest {
        auth_service::handlers::auth::RegisterRequest {
            first_name: "".to_string(), // Invalid
            last_name: "User".to_string(),
            email: "invalid-email".to_string(), // Invalid
            password: "weak".to_string(),       // Invalid
            department: None,
            position: None,
            lab_affiliation: None,
        }
    }

    pub fn create_valid_login_request(email: String) -> LoginRequest {
        LoginRequest {
            email,
            password: "SecurePassword123!".to_string(),
            remember_me: Some(false),
        }
    }

    pub fn create_invalid_login_request() -> LoginRequest {
        LoginRequest {
            email: "nonexistent@example.com".to_string(),
            password: "wrongpassword".to_string(),
            remember_me: Some(false),
        }
    }

    pub async fn create_test_user(auth_service: &Arc<AuthServiceImpl>) -> User {
        let email = format!("test{}@example.com", Uuid::new_v4());
        auth_service
            .create_user(
                "Test".to_string(),
                "User".to_string(),
                email,
                "SecurePassword123!".to_string(),
                UserRole::Guest,
            )
            .await
            .expect("Failed to create test user")
    }

    pub async fn create_admin_user(auth_service: &AuthServiceImpl) -> User {
        let email = format!("admin-{}@example.com", Uuid::new_v4());
        auth_service
            .create_user(
                "Admin".to_string(),
                "User".to_string(),
                email,
                "SecurePassword123!".to_string(),
                UserRole::LabAdministrator,
            )
            .await
            .expect("Failed to create admin user")
    }
}

/// JWT token utilities for testing
pub struct JwtTestUtils;

impl JwtTestUtils {
    pub fn create_test_token(user: &User) -> String {
        format!("test_token_for_user_{}", user.id)
    }

    pub fn create_expired_token() -> String {
        "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9.eyJleHAiOjEwMDAwMDAwMDB9.expired".to_string()
    }

    pub fn create_invalid_token() -> String {
        "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9.invalid.signature".to_string()
    }
}

/// HTTP test client wrapper with authentication helpers
pub struct AuthTestClient {
    pub server: TestServer,
    pub auth_token: Option<String>,
}

impl AuthTestClient {
    pub fn new(app: Router) -> Self {
        let server = TestServer::new(app).unwrap();
        Self {
            server,
            auth_token: None,
        }
    }

    pub fn with_auth_token(mut self, token: String) -> Self {
        self.auth_token = Some(token);
        self
    }

    pub async fn post_json<T: serde::Serialize>(
        &self,
        path: &str,
        body: &T,
    ) -> axum_test::TestResponse {
        let mut request = self.server.post(path).json(body);
        if let Some(token) = &self.auth_token {
            request = request.add_header("Authorization", &format!("Bearer {}", token));
        }
        request.await
    }

    pub async fn get(&self, path: &str) -> axum_test::TestResponse {
        let mut request = self.server.get(path);
        if let Some(token) = &self.auth_token {
            request = request.add_header("Authorization", &format!("Bearer {}", token));
        }
        request.await
    }

    pub async fn delete(&self, path: &str) -> axum_test::TestResponse {
        let mut request = self.server.delete(path);
        if let Some(token) = &self.auth_token {
            request = request.add_header("Authorization", &format!("Bearer {}", token));
        }
        request.await
    }

    pub async fn put_json<T: serde::Serialize>(
        &self,
        path: &str,
        body: &T,
    ) -> axum_test::TestResponse {
        let mut request = self.server.put(path).json(body);
        if let Some(token) = &self.auth_token {
            request = request.add_header("Authorization", &format!("Bearer {}", token));
        }
        request.await
    }
}

/// Common assertions for auth testing
pub struct AuthAssertions;

impl AuthAssertions {
    pub fn assert_successful_login(response: &serde_json::Value) {
        assert_eq!(response["success"], true);
        assert!(response["data"]["access_token"].is_string());
        assert!(response["data"]["refresh_token"].is_string());
        assert!(response["data"]["expires_at"].is_string());
        assert!(response["data"]["user"]["id"].is_string());
        assert!(response["data"]["user"]["email"].is_string());
    }

    pub fn assert_user_data(response: &serde_json::Value, expected_email: &str) {
        assert_eq!(response["success"], true);
        assert_eq!(response["data"]["email"], expected_email);
        assert!(response["data"]["id"].is_string());
        assert!(response["data"]["first_name"].is_string());
        assert!(response["data"]["last_name"].is_string());
        assert!(response["data"]["role"].is_string());
    }

    pub fn assert_validation_error(response: &serde_json::Value) {
        assert!(response["error"].is_object());
        assert_eq!(response["error"]["code"], "VALIDATION_ERROR");
    }

    pub fn assert_unauthorized(status: StatusCode) {
        assert_eq!(status, StatusCode::UNAUTHORIZED);
    }

    pub fn assert_forbidden(status: StatusCode) {
        assert_eq!(status, StatusCode::FORBIDDEN);
    }

    pub fn assert_not_found(status: StatusCode) {
        assert_eq!(status, StatusCode::NOT_FOUND);
    }
}

/// Test data generators
pub struct TestDataGenerator;

impl TestDataGenerator {
    pub fn random_email() -> String {
        format!("test{}@example.com", Uuid::new_v4())
    }

    pub fn random_password() -> String {
        format!(
            "SecurePass-{}",
            Uuid::new_v4().to_string()[..8].to_uppercase()
        )
    }

    pub fn random_name() -> String {
        Faker.fake::<String>()
    }

    pub fn weak_passwords() -> Vec<String> {
        vec![
            "123456".to_string(),
            "password".to_string(),
            "qwerty".to_string(),
            "abc123".to_string(),
            "letmein".to_string(),
            "".to_string(),
            "a".to_string(),
            "1234567".to_string(), // Just below minimum length
        ]
    }

    pub fn invalid_emails() -> Vec<String> {
        vec![
            "notanemail".to_string(),
            "@example.com".to_string(),
            "user@".to_string(),
            "user@.com".to_string(),
            "user@@example.com".to_string(),
            "user example@com".to_string(),
            "".to_string(),
            "user@example".to_string(),
        ]
    }
}

/// Security test utilities
pub struct SecurityTestUtils;

impl SecurityTestUtils {
    pub fn generate_sql_injection_attempts() -> Vec<String> {
        vec![
            "' OR '1'='1".to_string(),
            "admin'--".to_string(),
            "'; DROP TABLE users; --".to_string(),
            "1' UNION SELECT * FROM users--".to_string(),
            "' OR 1=1--".to_string(),
        ]
    }

    pub fn generate_xss_attempts() -> Vec<String> {
        vec![
            "<script>alert('XSS')</script>".to_string(),
            "javascript:alert('XSS')".to_string(),
            "<img src=x onerror=alert('XSS')>".to_string(),
            "<iframe src='javascript:alert(\"XSS\")'></iframe>".to_string(),
            "<svg onload=alert('XSS')>".to_string(),
        ]
    }

    pub async fn attempt_brute_force_login(
        client: &AuthTestClient,
        email: &str,
        attempts: usize,
    ) -> Vec<StatusCode> {
        let mut results = Vec::new();

        for i in 0..attempts {
            let login_req = LoginRequest {
                email: email.to_string(),
                password: format!("wrong_password_{}", i),
                remember_me: Some(false),
            };

            let response = client.post_json("/auth/login", &login_req).await;
            results.push(response.status_code());
        }

        results
    }
}

/// Performance test utilities
pub struct PerformanceTestUtils;

impl PerformanceTestUtils {
    pub async fn measure_login_performance(
        client: &AuthTestClient,
        user_email: &str,
        iterations: u32,
    ) -> std::time::Duration {
        let start = std::time::Instant::now();

        for _ in 0..iterations {
            let login_req = LoginRequest {
                email: user_email.to_string(),
                password: "SecurePassword123!".to_string(),
                remember_me: Some(false),
            };

            let _ = client.post_json("/auth/login", &login_req).await;
        }

        start.elapsed()
    }
}

/// Helper function to create test app state
pub async fn create_test_app_state() -> AppState {
    auth_service::test_utils::create_test_app_state().await
}
