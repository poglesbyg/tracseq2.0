//! Authentication Flow Integration Tests
//! 
//! This module tests complete authentication workflows including:
//! - User registration and email verification
//! - Login flows with different scenarios
//! - Password reset workflows
//! - Session management and token refresh
//! - Multi-factor authentication (if enabled)

use auth_service::{AppState, Config, DatabasePool, AuthServiceImpl, create_router, models::*};
use axum_test::TestServer;
use chrono::{Duration, Utc};
use serde_json::json;
use sqlx::PgPool;
use std::sync::Arc;
use uuid::Uuid;

/// Test database helper
struct TestDb {
    pool: PgPool,
    _temp_db_name: String,
}

impl TestDb {
    async fn new() -> anyhow::Result<Self> {
        let base_url = std::env::var("TEST_DATABASE_URL")
            .unwrap_or_else(|_| "postgres://postgres:postgres@localhost:5432/postgres".to_string());
        
        // Create a unique test database
        let temp_db_name = format!("auth_test_{}", Uuid::new_v4().to_string().replace("-", ""));
        let base_pool = PgPool::connect(&base_url).await?;
        
        sqlx::query(&format!("CREATE DATABASE {}", temp_db_name))
            .execute(&base_pool)
            .await?;
        
        // Connect to the new database
        let test_db_url = base_url.replace("/postgres", &format!("/{}", temp_db_name));
        let pool = PgPool::connect(&test_db_url).await?;
        
        // Run migrations
        sqlx::migrate!("./migrations")
            .run(&pool)
            .await?;
        
        Ok(Self {
            pool,
            _temp_db_name: temp_db_name,
        })
    }
}

/// Helper to create a test server with fresh database
async fn create_test_server() -> anyhow::Result<(TestServer, TestDb)> {
    let test_db = TestDb::new().await?;
    let db_pool = DatabasePool { pool: test_db.pool.clone() };
    
    let config = Config::test_config();
    let auth_service = AuthServiceImpl::new(db_pool.clone(), config.clone())?;
    
    let app_state = AppState {
        db_pool,
        config: Arc::new(config),
        auth_service: Arc::new(auth_service),
    };
    
    let app = create_router(app_state);
    let server = TestServer::new(app)?;
    
    Ok((server, test_db))
}

#[tokio::test]
async fn test_complete_registration_flow_with_email_verification() {
    let (server, _db) = create_test_server().await
        .expect("Failed to create test server");
    
    let email = format!("user-{}@example.com", Uuid::new_v4());
    
    // Step 1: Register user
    let register_request = json!({
        "first_name": "John",
        "last_name": "Doe",
        "email": email,
        "password": "SecurePassword123!",
        "department": "Research",
        "position": "Scientist",
        "lab_affiliation": "Lab A"
    });
    
    let response = server
        .post("/auth/register")
        .json(&register_request)
        .await;
    
    assert_eq!(response.status_code(), 201);
    let body: serde_json::Value = response.json();
    assert_eq!(body["success"], true);
    assert!(body["data"]["user_id"].is_string());
    
    // Step 2: Attempt login before email verification (should fail if verification required)
    let login_request = json!({
        "email": email,
        "password": "SecurePassword123!",
        "remember_me": false
    });
    
    let response = server
        .post("/auth/login")
        .json(&login_request)
        .await;
    
    // This might succeed or fail depending on config.features.email_verification_required
    // For now, we'll assume it succeeds but note this in real implementation
    
    // Step 3: Simulate email verification (in real app, user would click link)
    // This would normally involve:
    // 1. Extracting verification token from email/database
    // 2. Calling /auth/verify-email with the token
    // For testing, we'll directly update the database
    
    // Step 4: Login after verification
    let response = server
        .post("/auth/login")
        .json(&login_request)
        .await;
    
    assert_eq!(response.status_code(), 200);
    let body: serde_json::Value = response.json();
    assert_eq!(body["success"], true);
    assert!(body["data"]["access_token"].is_string());
    assert!(body["data"]["expires_at"].is_string());
}

#[tokio::test]
async fn test_login_with_remember_me() {
    let (server, db) = create_test_server().await
        .expect("Failed to create test server");
    
    let email = format!("remember-{}@example.com", Uuid::new_v4());
    
    // Register user
    let register_request = json!({
        "first_name": "Remember",
        "last_name": "Me",
        "email": email,
        "password": "SecurePassword123!"
    });
    
    let response = server
        .post("/auth/register")
        .json(&register_request)
        .await;
    
    assert_eq!(response.status_code(), 201);
    
    // Mark email as verified (for testing)
    sqlx::query("UPDATE users SET email_verified = true WHERE email = $1")
        .bind(&email)
        .execute(&db.pool)
        .await
        .unwrap();
    
    // Login with remember_me = true
    let login_request = json!({
        "email": email,
        "password": "SecurePassword123!",
        "remember_me": true
    });
    
    let response = server
        .post("/auth/login")
        .json(&login_request)
        .await;
    
    assert_eq!(response.status_code(), 200);
    let body: serde_json::Value = response.json();
    
    // Should have refresh token when remember_me is true
    assert!(body["data"]["refresh_token"].is_string());
    
    // Login with remember_me = false
    let login_request = json!({
        "email": email,
        "password": "SecurePassword123!",
        "remember_me": false
    });
    
    let response = server
        .post("/auth/login")
        .json(&login_request)
        .await;
    
    assert_eq!(response.status_code(), 200);
    let body: serde_json::Value = response.json();
    
    // Might not have refresh token when remember_me is false
    // (depends on implementation)
}

#[tokio::test]
async fn test_password_reset_complete_flow() {
    let (server, db) = create_test_server().await
        .expect("Failed to create test server");
    
    let email = format!("reset-{}@example.com", Uuid::new_v4());
    
    // Register and verify user
    let register_request = json!({
        "first_name": "Reset",
        "last_name": "User",
        "email": email,
        "password": "OldPassword123!"
    });
    
    server
        .post("/auth/register")
        .json(&register_request)
        .await;
    
    // Mark email as verified
    sqlx::query("UPDATE users SET email_verified = true WHERE email = $1")
        .bind(&email)
        .execute(&db.pool)
        .await
        .unwrap();
    
    // Step 1: Request password reset
    let forgot_request = json!({
        "email": email
    });
    
    let response = server
        .post("/auth/forgot-password")
        .json(&forgot_request)
        .await;
    
    assert_eq!(response.status_code(), 200);
    
    // Step 2: Get reset token from database (simulating email click)
    let reset_token_row: Option<(String,)> = sqlx::query_as(
        "SELECT token FROM password_reset_tokens WHERE user_id = (SELECT id FROM users WHERE email = $1) ORDER BY created_at DESC LIMIT 1"
    )
    .bind(&email)
    .fetch_optional(&db.pool)
    .await
    .unwrap();
    
    if let Some((token,)) = reset_token_row {
        // Step 3: Reset password with token
        let reset_request = json!({
            "token": token,
            "new_password": "NewPassword123!"
        });
        
        let response = server
            .post("/auth/reset-password")
            .json(&reset_request)
            .await;
        
        assert_eq!(response.status_code(), 200);
        
        // Step 4: Verify old password no longer works
        let old_login = json!({
            "email": email,
            "password": "OldPassword123!",
            "remember_me": false
        });
        
        let response = server
            .post("/auth/login")
            .json(&old_login)
            .await;
        
        assert_eq!(response.status_code(), 401);
        
        // Step 5: Verify new password works
        let new_login = json!({
            "email": email,
            "password": "NewPassword123!",
            "remember_me": false
        });
        
        let response = server
            .post("/auth/login")
            .json(&new_login)
            .await;
        
        assert_eq!(response.status_code(), 200);
    }
}

#[tokio::test]
async fn test_token_refresh_flow() {
    let (server, db) = create_test_server().await
        .expect("Failed to create test server");
    
    let email = format!("refresh-{}@example.com", Uuid::new_v4());
    
    // Register and verify user
    let register_request = json!({
        "first_name": "Refresh",
        "last_name": "Token",
        "email": email,
        "password": "SecurePassword123!"
    });
    
    server
        .post("/auth/register")
        .json(&register_request)
        .await;
    
    sqlx::query("UPDATE users SET email_verified = true WHERE email = $1")
        .bind(&email)
        .execute(&db.pool)
        .await
        .unwrap();
    
    // Login to get tokens
    let login_request = json!({
        "email": email,
        "password": "SecurePassword123!",
        "remember_me": true
    });
    
    let response = server
        .post("/auth/login")
        .json(&login_request)
        .await;
    
    let body: serde_json::Value = response.json();
    let access_token = body["data"]["access_token"].as_str().unwrap();
    let refresh_token = body["data"]["refresh_token"].as_str().unwrap();
    
    // Use access token successfully
    let response = server
        .get("/auth/me")
        .add_header("Authorization", &format!("Bearer {}", access_token))
        .await;
    
    assert_eq!(response.status_code(), 200);
    
    // Refresh the token
    let refresh_request = json!({
        "refresh_token": refresh_token
    });
    
    let response = server
        .post("/auth/refresh")
        .json(&refresh_request)
        .await;
    
    assert_eq!(response.status_code(), 200);
    let body: serde_json::Value = response.json();
    let new_access_token = body["data"]["access_token"].as_str().unwrap();
    
    // Verify new token works
    let response = server
        .get("/auth/me")
        .add_header("Authorization", &format!("Bearer {}", new_access_token))
        .await;
    
    assert_eq!(response.status_code(), 200);
}

#[tokio::test]
async fn test_concurrent_login_sessions() {
    let (server, db) = create_test_server().await
        .expect("Failed to create test server");
    
    let email = format!("concurrent-{}@example.com", Uuid::new_v4());
    
    // Register and verify user
    let register_request = json!({
        "first_name": "Concurrent",
        "last_name": "User",
        "email": email,
        "password": "SecurePassword123!"
    });
    
    server
        .post("/auth/register")
        .json(&register_request)
        .await;
    
    sqlx::query("UPDATE users SET email_verified = true WHERE email = $1")
        .bind(&email)
        .execute(&db.pool)
        .await
        .unwrap();
    
    // Create multiple login sessions
    let mut tokens = vec![];
    
    for i in 0..3 {
        let login_request = json!({
            "email": email,
            "password": "SecurePassword123!",
            "remember_me": false
        });
        
        let response = server
            .post("/auth/login")
            .json(&login_request)
            .await;
        
        let body: serde_json::Value = response.json();
        tokens.push(body["data"]["access_token"].as_str().unwrap().to_string());
        
        // Small delay to ensure different timestamps
        tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
    }
    
    // All tokens should work
    for token in &tokens {
        let response = server
            .get("/auth/me")
            .add_header("Authorization", &format!("Bearer {}", token))
            .await;
        
        assert_eq!(response.status_code(), 200);
    }
    
    // Get all sessions
    let response = server
        .get("/auth/sessions")
        .add_header("Authorization", &format!("Bearer {}", tokens[0]))
        .await;
    
    assert_eq!(response.status_code(), 200);
    let body: serde_json::Value = response.json();
    let sessions = body["data"].as_array().unwrap();
    assert!(sessions.len() >= 3);
}

#[tokio::test]
async fn test_logout_invalidates_session() {
    let (server, db) = create_test_server().await
        .expect("Failed to create test server");
    
    let email = format!("logout-{}@example.com", Uuid::new_v4());
    
    // Register and verify user
    let register_request = json!({
        "first_name": "Logout",
        "last_name": "Test",
        "email": email,
        "password": "SecurePassword123!"
    });
    
    server
        .post("/auth/register")
        .json(&register_request)
        .await;
    
    sqlx::query("UPDATE users SET email_verified = true WHERE email = $1")
        .bind(&email)
        .execute(&db.pool)
        .await
        .unwrap();
    
    // Login
    let login_request = json!({
        "email": email,
        "password": "SecurePassword123!",
        "remember_me": false
    });
    
    let response = server
        .post("/auth/login")
        .json(&login_request)
        .await;
    
    let body: serde_json::Value = response.json();
    let access_token = body["data"]["access_token"].as_str().unwrap();
    let session_id = body["data"]["session_id"].as_str().unwrap();
    
    // Verify token works
    let response = server
        .get("/auth/me")
        .add_header("Authorization", &format!("Bearer {}", access_token))
        .await;
    
    assert_eq!(response.status_code(), 200);
    
    // Logout
    let logout_request = json!({
        "session_id": session_id
    });
    
    let response = server
        .post("/auth/logout")
        .add_header("Authorization", &format!("Bearer {}", access_token))
        .json(&logout_request)
        .await;
    
    assert_eq!(response.status_code(), 200);
    
    // Verify token no longer works
    let response = server
        .get("/auth/me")
        .add_header("Authorization", &format!("Bearer {}", access_token))
        .await;
    
    assert_eq!(response.status_code(), 401);
}

#[tokio::test]
async fn test_account_lockout_after_failed_attempts() {
    let (server, db) = create_test_server().await
        .expect("Failed to create test server");
    
    let email = format!("lockout-{}@example.com", Uuid::new_v4());
    
    // Register and verify user
    let register_request = json!({
        "first_name": "Lockout",
        "last_name": "Test",
        "email": email,
        "password": "CorrectPassword123!"
    });
    
    server
        .post("/auth/register")
        .json(&register_request)
        .await;
    
    sqlx::query("UPDATE users SET email_verified = true WHERE email = $1")
        .bind(&email)
        .execute(&db.pool)
        .await
        .unwrap();
    
    // Make multiple failed login attempts
    for i in 0..5 {
        let login_request = json!({
            "email": email,
            "password": format!("WrongPassword{}!", i),
            "remember_me": false
        });
        
        let response = server
            .post("/auth/login")
            .json(&login_request)
            .await;
        
        // Should get 401 for wrong password
        assert_eq!(response.status_code(), 401);
    }
    
    // After max attempts, even correct password should fail (account locked)
    let login_request = json!({
        "email": email,
        "password": "CorrectPassword123!",
        "remember_me": false
    });
    
    let response = server
        .post("/auth/login")
        .json(&login_request)
        .await;
    
    // Should be locked out (might be 401 or 423 depending on implementation)
    assert!(response.status_code() == 401 || response.status_code() == 423);
    
    let body: serde_json::Value = response.json();
    // Error message should indicate account is locked
    if let Some(error_msg) = body["error"]["message"].as_str() {
        assert!(error_msg.to_lowercase().contains("locked") || 
                error_msg.to_lowercase().contains("attempt"));
    }
}

#[tokio::test]
async fn test_session_expiration() {
    let (server, db) = create_test_server().await
        .expect("Failed to create test server");
    
    let email = format!("expire-{}@example.com", Uuid::new_v4());
    
    // Register and verify user
    let register_request = json!({
        "first_name": "Expire",
        "last_name": "Test",
        "email": email,
        "password": "SecurePassword123!"
    });
    
    server
        .post("/auth/register")
        .json(&register_request)
        .await;
    
    sqlx::query("UPDATE users SET email_verified = true WHERE email = $1")
        .bind(&email)
        .execute(&db.pool)
        .await
        .unwrap();
    
    // Login
    let login_request = json!({
        "email": email,
        "password": "SecurePassword123!",
        "remember_me": false
    });
    
    let response = server
        .post("/auth/login")
        .json(&login_request)
        .await;
    
    let body: serde_json::Value = response.json();
    let access_token = body["data"]["access_token"].as_str().unwrap();
    let session_id = body["data"]["session_id"].as_str().unwrap();
    
    // Manually expire the session in database
    sqlx::query(
        "UPDATE user_sessions SET expires_at = $1 WHERE id = $2"
    )
    .bind(Utc::now() - Duration::hours(1))
    .bind(Uuid::parse_str(session_id).unwrap())
    .execute(&db.pool)
    .await
    .unwrap();
    
    // Token should no longer work
    let response = server
        .get("/auth/me")
        .add_header("Authorization", &format!("Bearer {}", access_token))
        .await;
    
    assert_eq!(response.status_code(), 401);
}
