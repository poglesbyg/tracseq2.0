use auth_service::{models::*, test_utils::*, Config, AuthService};
use axum::{body::Body, http::{Request, StatusCode}, Router};
use axum_test::TestServer;
use fake::{Fake, Faker};
use serde_json::Value;
use sqlx::PgPool;
use std::sync::Arc;
use uuid::Uuid;

/// Test database manager for isolated test environments
pub struct TestDatabase {
    pub pool: PgPool,
    pub cleanup_users: Vec<Uuid>,
}

impl TestDatabase {
    pub async fn new() -> Self {
        let pool = get_test_db().await.clone();
        Self {
            pool,
            cleanup_users: Vec::new(),
        }
    }

    pub async fn cleanup(&mut self) {
        for user_id in &self.cleanup_users {
            let _ = sqlx::query("DELETE FROM user_sessions WHERE user_id = $1")
                .bind(user_id)
                .execute(&self.pool)
                .await;
            let _ = sqlx::query("DELETE FROM users WHERE id = $1")
                .bind(user_id)
                .execute(&self.pool)
                .await;
        }
        self.cleanup_users.clear();
    }

    pub fn track_user(&mut self, user_id: Uuid) {
        self.cleanup_users.push(user_id);
    }
}

impl Drop for TestDatabase {
    fn drop(&mut self) {
        let pool = self.pool.clone();
        let users = self.cleanup_users.clone();
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
    pub fn create_valid_register_request() -> RegisterRequest {
        RegisterRequest {
            first_name: Faker.fake::<String>(),
            last_name: Faker.fake::<String>(),
            email: format!("test-{}@example.com", Uuid::new_v4()),
            password: "SecurePassword123!".to_string(),
            department: Some("Laboratory".to_string()),
            position: Some("Researcher".to_string()),
            lab_affiliation: Some("Main Lab".to_string()),
        }
    }

    pub fn create_invalid_register_request() -> RegisterRequest {
        RegisterRequest {
            first_name: "".to_string(), // Invalid: empty
            last_name: "".to_string(),  // Invalid: empty
            email: "invalid-email".to_string(), // Invalid: bad format
            password: "123".to_string(), // Invalid: too short
            department: None,
            position: None,
            lab_affiliation: None,
        }
    }

    pub fn create_valid_login_request(email: String) -> LoginRequest {
        LoginRequest {
            email,
            password: "SecurePassword123!".to_string(),
        }
    }

    pub fn create_invalid_login_request() -> LoginRequest {
        LoginRequest {
            email: "nonexistent@example.com".to_string(),
            password: "wrongpassword".to_string(),
        }
    }

    pub async fn create_test_user(auth_service: &AuthService) -> User {
        let register_req = Self::create_valid_register_request();
        auth_service.create_user(
            register_req.first_name,
            register_req.last_name,
            register_req.email,
            register_req.password,
            UserRole::User,
        ).await.expect("Failed to create test user")
    }

    pub async fn create_admin_user(auth_service: &AuthService) -> User {
        let register_req = Self::create_valid_register_request();
        auth_service.create_user(
            register_req.first_name,
            register_req.last_name,
            register_req.email,
            register_req.password,
            UserRole::Admin,
        ).await.expect("Failed to create admin user")
    }
}

/// JWT token utilities for testing
pub struct JwtTestUtils;

impl JwtTestUtils {
    pub fn create_test_token(user: &User) -> String {
        let config = Config::test_config();
        // This would use the actual JWT creation logic from AuthService
        format!("test_token_for_user_{}", user.id)
    }

    pub fn create_expired_token() -> String {
        "expired.jwt.token".to_string()
    }

    pub fn create_invalid_token() -> String {
        "invalid.jwt.token".to_string()
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

    pub async fn post_json<T: serde::Serialize>(&self, path: &str, body: &T) -> axum_test::TestResponse {
        let mut request = self.server.post(path).json(body);
        if let Some(token) = &self.auth_token {
            request = request.add_header("Authorization", format!("Bearer {}", token).parse().unwrap());
        }
        request.await
    }

    pub async fn get(&self, path: &str) -> axum_test::TestResponse {
        let mut request = self.server.get(path);
        if let Some(token) = &self.auth_token {
            request = request.add_header("Authorization", format!("Bearer {}", token).parse().unwrap());
        }
        request.await
    }

    pub async fn delete(&self, path: &str) -> axum_test::TestResponse {
        let mut request = self.server.delete(path);
        if let Some(token) = &self.auth_token {
            request = request.add_header("Authorization", format!("Bearer {}", token).parse().unwrap());
        }
        request.await
    }

    pub async fn put_json<T: serde::Serialize>(&self, path: &str, body: &T) -> axum_test::TestResponse {
        let mut request = self.server.put(path).json(body);
        if let Some(token) = &self.auth_token {
            request = request.add_header("Authorization", format!("Bearer {}", token).parse().unwrap());
        }
        request.await
    }
}

/// Common assertions for auth testing
pub struct AuthAssertions;

impl AuthAssertions {
    pub fn assert_successful_login(response: &Value) {
        assert_eq!(response["success"], true);
        assert!(response["data"]["access_token"].is_string());
        assert!(response["data"]["refresh_token"].is_string());
        assert!(response["data"]["expires_in"].is_number());
    }

    pub fn assert_user_data(response: &Value, expected_email: &str) {
        assert_eq!(response["success"], true);
        assert_eq!(response["data"]["email"], expected_email);
        assert!(response["data"]["id"].is_string());
        assert!(response["data"]["created_at"].is_string());
    }

    pub fn assert_validation_error(response: &Value) {
        assert_eq!(response["success"], false);
        assert!(response["error"].is_string());
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
        format!("test-{}@example.com", Uuid::new_v4())
    }

    pub fn random_password() -> String {
        format!("SecurePass-{}", Uuid::new_v4().to_string()[..8].to_uppercase())
    }

    pub fn random_name() -> String {
        Faker.fake::<String>()
    }

    pub fn weak_passwords() -> Vec<String> {
        vec![
            "123".to_string(),
            "password".to_string(),
            "".to_string(),
            "abc".to_string(),
            "12345678".to_string(), // Only numbers
        ]
    }

    pub fn invalid_emails() -> Vec<String> {
        vec![
            "invalid-email".to_string(),
            "@example.com".to_string(),
            "test@".to_string(),
            "".to_string(),
            "spaces in@email.com".to_string(),
        ]
    }
}

/// Macro for creating database tests with automatic cleanup
#[macro_export]
macro_rules! test_with_auth_db {
    ($test_name:ident, $test_body:expr) => {
        #[tokio::test]
        async fn $test_name() {
            let mut test_db = TestDatabase::new().await;
            let result = std::panic::AssertUnwindSafe($test_body(&mut test_db))
                .catch_unwind()
                .await;
            test_db.cleanup().await;
            if let Err(panic) = result {
                std::panic::resume_unwind(panic);
            }
        }
    };
}

/// Security test utilities
pub struct SecurityTestUtils;

impl SecurityTestUtils {
    pub async fn attempt_brute_force_login(client: &AuthTestClient, email: &str, attempts: u32) -> Vec<StatusCode> {
        let mut results = Vec::new();
        
        for i in 0..attempts {
            let login_req = LoginRequest {
                email: email.to_string(),
                password: format!("wrong_password_{}", i),
            };
            
            let response = client.post_json("/auth/login", &login_req).await;
            results.push(response.status_code());
            
            // Small delay to avoid overwhelming the test
            tokio::time::sleep(tokio::time::Duration::from_millis(10)).await;
        }
        
        results
    }

    pub fn generate_sql_injection_attempts() -> Vec<String> {
        vec![
            "'; DROP TABLE users; --".to_string(),
            "' OR '1'='1".to_string(),
            "admin'--".to_string(),
            "' UNION SELECT * FROM users --".to_string(),
        ]
    }

    pub fn generate_xss_attempts() -> Vec<String> {
        vec![
            "<script>alert('xss')</script>".to_string(),
            "javascript:alert('xss')".to_string(),
            "<img src=x onerror=alert('xss')>".to_string(),
        ]
    }
}

/// Performance test utilities
pub struct PerformanceTestUtils;

impl PerformanceTestUtils {
    pub async fn measure_login_performance(client: &AuthTestClient, user_email: &str, iterations: u32) -> std::time::Duration {
        let start = std::time::Instant::now();
        
        for _ in 0..iterations {
            let login_req = LoginRequest {
                email: user_email.to_string(),
                password: "SecurePassword123!".to_string(),
            };
            
            let _ = client.post_json("/auth/login", &login_req).await;
        }
        
        start.elapsed()
    }

    pub async fn concurrent_login_test(client: &AuthTestClient, user_email: &str, concurrent_requests: u32) -> Vec<StatusCode> {
        let tasks: Vec<_> = (0..concurrent_requests)
            .map(|_| {
                let email = user_email.to_string();
                let client_clone = &client; // Note: This is a simplified approach
                async move {
                    let login_req = LoginRequest {
                        email,
                        password: "SecurePassword123!".to_string(),
                    };
                    client_clone.post_json("/auth/login", &login_req).await.status_code()
                }
            })
            .collect();

        futures::future::join_all(tasks).await
    }
} 
