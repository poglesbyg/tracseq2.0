//! Comprehensive Auth Service Tests
//! 
//! This test suite provides end-to-end testing of the authentication service
//! covering user lifecycle, authentication flows, and security features.

use auth_service::{AppState, Config, DatabasePool, AuthServiceImpl, create_router};
use axum_test::TestServer;
use serde_json::json;
use std::sync::Arc;
use uuid::Uuid;
use once_cell::sync::Lazy;
use sqlx::PgPool;

// Shared test database connection
static TEST_DB: Lazy<tokio::sync::Mutex<Option<PgPool>>> = Lazy::new(|| {
    tokio::sync::Mutex::new(None)
});

/// Test context for managing database state
struct TestContext {
    db_pool: PgPool,
    tracked_users: Vec<Uuid>,
}

impl TestContext {
    async fn new() -> anyhow::Result<Self> {
        let mut db_guard = TEST_DB.lock().await;
        
        let pool = if let Some(pool) = db_guard.as_ref() {
            pool.clone()
        } else {
            let database_url = std::env::var("TEST_DATABASE_URL")
                .unwrap_or_else(|_| "postgres://postgres:postgres@localhost:5432/auth_service_test".to_string());
            
            let pool = sqlx::postgres::PgPoolOptions::new()
                .max_connections(5)
                .connect(&database_url)
                .await?;
            
            // Run migrations
            sqlx::migrate!("./migrations")
                .run(&pool)
                .await?;
            
            *db_guard = Some(pool.clone());
            pool
        };
        
        Ok(Self {
            db_pool: pool,
            tracked_users: Vec::new(),
        })
    }
    
    fn track_user(&mut self, user_id: Uuid) {
        self.tracked_users.push(user_id);
    }
    
    async fn cleanup(&mut self) {
        for user_id in &self.tracked_users {
            // Clean up sessions first
            let _ = sqlx::query("DELETE FROM user_sessions WHERE user_id = $1")
                .bind(user_id)
                .execute(&self.db_pool)
                .await;
                
            // Then clean up users
            let _ = sqlx::query("DELETE FROM users WHERE id = $1")
                .bind(user_id)
                .execute(&self.db_pool)
                .await;
        }
        self.tracked_users.clear();
    }
}

impl Drop for TestContext {
    fn drop(&mut self) {
        let pool = self.db_pool.clone();
        let users = self.tracked_users.clone();
        
        // Spawn a task to clean up in the background
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

/// Create test app state with database
async fn create_test_app_state() -> anyhow::Result<(AppState, TestContext)> {
    let mut ctx = TestContext::new().await?;
    let db_pool = DatabasePool { pool: ctx.db_pool.clone() };
    
    let config = Config::test_config();
    let auth_service = AuthServiceImpl::new(db_pool.clone(), config.clone())?;
    
    let app_state = AppState {
        db_pool,
        config: Arc::new(config),
        auth_service: Arc::new(auth_service),
    };
    
    Ok((app_state, ctx))
}

/// Generate a unique test email
fn test_email() -> String {
    format!("test-{}@example.com", Uuid::new_v4())
}

#[tokio::test]
async fn test_complete_user_lifecycle() {
    // Setup
    let (app_state, mut ctx) = create_test_app_state().await
        .expect("Failed to create test app state");
    
    let app = create_router(app_state);
    let server = TestServer::new(app)
        .expect("Failed to create test server");
    
    // Test data
    let email = test_email();
    let password = "SecurePassword123!";
    
    // 1. Register user
    let register_request = json!({
        "first_name": "Test",
        "last_name": "User",
        "email": email,
        "password": password,
        "department": "Engineering",
        "lab_affiliation": "Lab A"
    });
    
    let response = server
        .post("/auth/register")
        .json(&register_request)
        .await;
    
    assert_eq!(response.status_code(), 201);
    
    let body: serde_json::Value = response.json();
    assert_eq!(body["success"], true);
    let user_id = body["data"]["user_id"].as_str().unwrap();
    ctx.track_user(Uuid::parse_str(user_id).unwrap());
    
    // 2. Login
    let login_request = json!({
        "email": email,
        "password": password,
        "remember_me": false
    });
    
    let response = server
        .post("/auth/login")
        .json(&login_request)
        .await;
    
    assert_eq!(response.status_code(), 200);
    
    let body: serde_json::Value = response.json();
    assert_eq!(body["success"], true);
    assert!(body["data"]["access_token"].is_string());
    let access_token = body["data"]["access_token"].as_str().unwrap();
    
    // 3. Get current user
    let response = server
        .get("/auth/me")
        .add_header("Authorization", &format!("Bearer {}", access_token))
        .await;
    
    assert_eq!(response.status_code(), 200);
    
    let body: serde_json::Value = response.json();
    assert_eq!(body["data"]["email"], email);
    assert_eq!(body["data"]["id"], user_id);
    
    // 4. Change password
    let new_password = "NewSecurePassword456!";
    let change_password_request = json!({
        "current_password": password,
        "new_password": new_password
    });
    
    let response = server
        .put("/auth/change-password")
        .add_header("Authorization", &format!("Bearer {}", access_token))
        .json(&change_password_request)
        .await;
    
    assert_eq!(response.status_code(), 200);
    
    // 5. Verify old password no longer works
    let response = server
        .post("/auth/login")
        .json(&login_request)
        .await;
    
    assert_eq!(response.status_code(), 401);
    
    // 6. Verify new password works
    let new_login_request = json!({
        "email": email,
        "password": new_password,
        "remember_me": false
    });
    
    let response = server
        .post("/auth/login")
        .json(&new_login_request)
        .await;
    
    assert_eq!(response.status_code(), 200);
    
    // Cleanup
    ctx.cleanup().await;
}

#[tokio::test]
async fn test_invalid_registration() {
    let (app_state, mut ctx) = create_test_app_state().await
        .expect("Failed to create test app state");
    
    let app = create_router(app_state);
    let server = TestServer::new(app)
        .expect("Failed to create test server");
    
    // Test invalid email
    let register_request = json!({
        "first_name": "Test",
        "last_name": "User",
        "email": "invalid-email",
        "password": "SecurePassword123!"
    });
    
    let response = server
        .post("/auth/register")
        .json(&register_request)
        .await;
    
    assert_eq!(response.status_code(), 400);
    
    // Test weak password
    let register_request = json!({
        "first_name": "Test",
        "last_name": "User",
        "email": test_email(),
        "password": "weak"
    });
    
    let response = server
        .post("/auth/register")
        .json(&register_request)
        .await;
    
    assert_eq!(response.status_code(), 400);
    
    // Test missing fields
    let register_request = json!({
        "email": test_email(),
        "password": "SecurePassword123!"
    });
    
    let response = server
        .post("/auth/register")
        .json(&register_request)
        .await;
    
    assert_eq!(response.status_code(), 400);
    
    ctx.cleanup().await;
}

#[tokio::test]
async fn test_duplicate_registration() {
    let (app_state, mut ctx) = create_test_app_state().await
        .expect("Failed to create test app state");
    
    let app = create_router(app_state);
    let server = TestServer::new(app)
        .expect("Failed to create test server");
    
    let email = test_email();
    let register_request = json!({
        "first_name": "Test",
        "last_name": "User",
        "email": email,
        "password": "SecurePassword123!"
    });
    
    // First registration should succeed
    let response = server
        .post("/auth/register")
        .json(&register_request)
        .await;
    
    assert_eq!(response.status_code(), 201);
    let body: serde_json::Value = response.json();
    let user_id = body["data"]["user_id"].as_str().unwrap();
    ctx.track_user(Uuid::parse_str(user_id).unwrap());
    
    // Second registration with same email should fail
    let response = server
        .post("/auth/register")
        .json(&register_request)
        .await;
    
    assert_eq!(response.status_code(), 409);
    
    ctx.cleanup().await;
}

#[tokio::test]
async fn test_login_with_invalid_credentials() {
    let (app_state, ctx) = create_test_app_state().await
        .expect("Failed to create test app state");
    
    let app = create_router(app_state);
    let server = TestServer::new(app)
        .expect("Failed to create test server");
    
    // Test login with non-existent user
    let login_request = json!({
        "email": test_email(),
        "password": "SomePassword123!",
        "remember_me": false
    });
    
    let response = server
        .post("/auth/login")
        .json(&login_request)
        .await;
    
    assert_eq!(response.status_code(), 401);
}

#[tokio::test]
async fn test_session_management() {
    let (app_state, mut ctx) = create_test_app_state().await
        .expect("Failed to create test app state");
    
    let app = create_router(app_state);
    let server = TestServer::new(app)
        .expect("Failed to create test server");
    
    // Register and login
    let email = test_email();
    let password = "SecurePassword123!";
    
    let register_request = json!({
        "first_name": "Test",
        "last_name": "User",
        "email": email,
        "password": password
    });
    
    let response = server
        .post("/auth/register")
        .json(&register_request)
        .await;
    
    let body: serde_json::Value = response.json();
    let user_id = body["data"]["user_id"].as_str().unwrap();
    ctx.track_user(Uuid::parse_str(user_id).unwrap());
    
    // Login to create a session
    let login_request = json!({
        "email": email,
        "password": password,
        "remember_me": false
    });
    
    let response = server
        .post("/auth/login")
        .json(&login_request)
        .await;
    
    let body: serde_json::Value = response.json();
    let access_token = body["data"]["access_token"].as_str().unwrap();
    let session_id = body["data"]["session_id"].as_str().unwrap();
    
    // Get sessions
    let response = server
        .get("/auth/sessions")
        .add_header("Authorization", &format!("Bearer {}", access_token))
        .await;
    
    assert_eq!(response.status_code(), 200);
    let body: serde_json::Value = response.json();
    assert!(body["data"].as_array().unwrap().len() > 0);
    
    // Revoke session
    let response = server
        .delete(&format!("/auth/sessions/{}", session_id))
        .add_header("Authorization", &format!("Bearer {}", access_token))
        .await;
    
    assert_eq!(response.status_code(), 200);
    
    // Verify token no longer works
    let response = server
        .get("/auth/me")
        .add_header("Authorization", &format!("Bearer {}", access_token))
        .await;
    
    assert_eq!(response.status_code(), 401);
    
    ctx.cleanup().await;
}

#[tokio::test] 
async fn test_concurrent_registrations() {
    let (app_state, mut ctx) = create_test_app_state().await
        .expect("Failed to create test app state");
    
    let app = create_router(app_state);
    let server = TestServer::new(app).expect("Failed to create test server");
    
    // Create multiple registration tasks
    let mut handles = vec![];
    
    for i in 0..5 {
        let register_request = json!({
            "first_name": format!("Test{}", i),
            "last_name": "User",
            "email": test_email(),
            "password": "SecurePassword123!"
        });
        
        // Execute the request directly without spawning
        let response = server
            .post("/auth/register")
            .json(&register_request)
            .await;
        
        handles.push((response.status_code(), response.json::<serde_json::Value>()));
    }
    
    // Process all registration results
    let mut user_ids = vec![];
    for (status, body) in handles {
        assert_eq!(status, 201);
        assert_eq!(body["success"], true);
        
        if let Some(user_id_str) = body["data"]["user_id"].as_str() {
            user_ids.push(Uuid::parse_str(user_id_str).unwrap());
        }
    }
    
    // Track all users for cleanup
    for user_id in user_ids {
        ctx.track_user(user_id);
    }
    
    ctx.cleanup().await;
}