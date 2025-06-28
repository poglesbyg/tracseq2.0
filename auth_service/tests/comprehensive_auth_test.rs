//! Comprehensive Auth Service Tests using Test Helpers

use auth_service::{AppState, Config, DatabasePool, AuthServiceImpl};
use test_helpers::{
    TestContext, 
    fixtures::{UserFixture, TestDataBuilder},
    http::TestServer,
    database::DatabaseTestBuilder,
};
use serde_json::json;
use std::sync::Arc;
use uuid::Uuid;

/// Create test app state with database
async fn create_test_app_state() -> anyhow::Result<(AppState, TestContext)> {
    let ctx = TestContext::with_database().await?;
    let db_pool = DatabasePool { pool: ctx.db().clone() };
    
    // Run migrations
    db_pool.migrate().await?;
    
    let config = Config::test_config();
    let auth_service = AuthServiceImpl::new(db_pool.clone(), config.clone())?;
    
    let app_state = AppState {
        db_pool,
        config: Arc::new(config),
        auth_service: Arc::new(auth_service),
    };
    
    Ok((app_state, ctx))
}

#[tokio::test]
async fn test_complete_user_lifecycle() {
    // Setup
    let (app_state, ctx) = create_test_app_state().await
        .expect("Failed to create test app state");
    
    let app = auth_service::create_router(app_state);
    let server = TestServer::new(app).await
        .expect("Failed to create test server");
    
    // Test data
    let user_fixture = UserFixture::new();
    
    // 1. Register user
    let register_request = json!({
        "first_name": user_fixture.first_name,
        "last_name": user_fixture.last_name,
        "email": user_fixture.email,
        "password": user_fixture.password,
        "department": user_fixture.department,
        "lab_affiliation": user_fixture.lab_affiliation
    });
    
    let register_response = server
        .post("/auth/register")
        .json(&register_request)
        .send()
        .await;
    
    register_response.assert_status(hyper::StatusCode::CREATED);
    
    let register_body: serde_json::Value = register_response.json().await;
    assert_eq!(register_body["success"], true);
    let user_id = register_body["data"]["user_id"].as_str().unwrap();
    
    // 2. Login
    let login_request = json!({
        "email": user_fixture.email,
        "password": user_fixture.password,
        "remember_me": false
    });
    
    let login_response = server
        .post("/auth/login")
        .json(&login_request)
        .send()
        .await;
    
    login_response.assert_success();
    
    let login_body: serde_json::Value = login_response.json().await;
    assert_eq!(login_body["success"], true);
    let access_token = login_body["data"]["access_token"].as_str().unwrap();
    
    // 3. Get current user
    let me_response = server
        .get("/auth/me")
        .auth(access_token)
        .send()
        .await;
    
    me_response.assert_success();
    
    let me_body: serde_json::Value = me_response.json().await;
    assert_eq!(me_body["data"]["email"], user_fixture.email);
    assert_eq!(me_body["data"]["id"], user_id);
    
    // 4. Change password
    let new_password = "NewSecurePassword456!";
    let change_password_request = json!({
        "current_password": user_fixture.password,
        "new_password": new_password
    });
    
    let change_password_response = server
        .put("/auth/change-password")
        .auth(access_token)
        .json(&change_password_request)
        .send()
        .await;
    
    change_password_response.assert_success();
    
    // 5. Verify old password no longer works
    let old_login_response = server
        .post("/auth/login")
        .json(&login_request)
        .send()
        .await;
    
    old_login_response.assert_status(hyper::StatusCode::UNAUTHORIZED);
    
    // 6. Verify new password works
    let new_login_request = json!({
        "email": user_fixture.email,
        "password": new_password,
        "remember_me": false
    });
    
    let new_login_response = server
        .post("/auth/login")
        .json(&new_login_request)
        .send()
        .await;
    
    new_login_response.assert_success();
    
    // Cleanup is handled automatically by TestContext
}

#[tokio::test]
async fn test_role_based_access_control() {
    let (app_state, ctx) = create_test_app_state().await
        .expect("Failed to create test app state");
    
    let app = auth_service::create_router(app_state);
    let server = TestServer::new(app).await
        .expect("Failed to create test server");
    
    // Create users with different roles
    let admin = UserFixture::admin();
    let technician = UserFixture::with_role("technician");
    let guest = UserFixture::with_role("guest");
    
    // Register all users
    for user in [&admin, &technician, &guest] {
        let register_request = json!({
            "first_name": user.first_name,
            "last_name": user.last_name,
            "email": user.email,
            "password": user.password,
            "role": user.role
        });
        
        let response = server
            .post("/auth/register")
            .json(&register_request)
            .send()
            .await;
        
        response.assert_status(hyper::StatusCode::CREATED);
    }
    
    // Test role-specific endpoints (when implemented)
    // This is a placeholder for when role-based endpoints are added
}

#[tokio::test]
async fn test_security_features() {
    let (app_state, ctx) = create_test_app_state().await
        .expect("Failed to create test app state");
    
    let app = auth_service::create_router(app_state.clone());
    let server = TestServer::new(app).await
        .expect("Failed to create test server");
    
    let user = UserFixture::new();
    
    // Register user
    let register_request = json!({
        "first_name": user.first_name,
        "last_name": user.last_name,
        "email": user.email,
        "password": user.password
    });
    
    server
        .post("/auth/register")
        .json(&register_request)
        .send()
        .await
        .assert_status(hyper::StatusCode::CREATED);
    
    // Test account lockout after failed attempts
    let max_attempts = app_state.config.security.max_login_attempts;
    
    for i in 0..max_attempts + 1 {
        let wrong_login = json!({
            "email": user.email,
            "password": "WrongPassword123!",
            "remember_me": false
        });
        
        let response = server
            .post("/auth/login")
            .json(&wrong_login)
            .send()
            .await;
        
        if i < max_attempts {
            response.assert_status(hyper::StatusCode::UNAUTHORIZED);
        } else {
            // Account should be locked
            let body: serde_json::Value = response.json().await;
            assert!(body["error"].as_str().unwrap().contains("locked"));
        }
    }
}

#[tokio::test]
async fn test_password_reset_flow() {
    let (app_state, ctx) = create_test_app_state().await
        .expect("Failed to create test app state");
    
    let app = auth_service::create_router(app_state);
    let server = TestServer::new(app).await
        .expect("Failed to create test server");
    
    let user = UserFixture::new();
    
    // Register user
    let register_request = json!({
        "first_name": user.first_name,
        "last_name": user.last_name,
        "email": user.email,
        "password": user.password
    });
    
    server
        .post("/auth/register")
        .json(&register_request)
        .send()
        .await
        .assert_status(hyper::StatusCode::CREATED);
    
    // Request password reset
    let reset_request = json!({
        "email": user.email
    });
    
    let reset_response = server
        .post("/auth/forgot-password")
        .json(&reset_request)
        .send()
        .await;
    
    reset_response.assert_success();
    
    // In a real test, we would:
    // 1. Retrieve the reset token from the database or email
    // 2. Use the token to reset the password
    // 3. Verify the new password works
}

#[tokio::test]
async fn test_concurrent_requests() {
    let (app_state, ctx) = create_test_app_state().await
        .expect("Failed to create test app state");
    
    let app = auth_service::create_router(app_state);
    let server = TestServer::new(app).await
        .expect("Failed to create test server");
    
    // Create multiple users concurrently
    let users: Vec<UserFixture> = (0..10)
        .map(|_| UserFixture::new())
        .collect();
    
    let registration_futures: Vec<_> = users
        .iter()
        .map(|user| {
            let request = json!({
                "first_name": user.first_name,
                "last_name": user.last_name,
                "email": user.email,
                "password": user.password
            });
            
            server
                .post("/auth/register")
                .json(&request)
                .send()
        })
        .collect();
    
    // Execute all registrations concurrently
    let results = futures::future::join_all(registration_futures).await;
    
    // All should succeed
    for (i, result) in results.iter().enumerate() {
        result.assert_status(hyper::StatusCode::CREATED);
        
        let body: serde_json::Value = result.json().await;
        assert_eq!(body["success"], true);
    }
}