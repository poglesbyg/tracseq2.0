use crate::test_utils::*;
use auth_service::*;
use auth_service::handlers::auth::{RegisterRequest};
use axum::{Router, extract::State, http::{StatusCode, header}};
use axum_test::TestServer;
use chrono::{DateTime, Utc};
use serde_json::json;
use uuid::Uuid;

#[tokio::test]
async fn test_complete_registration_login_flow() {
    let app_state = create_test_app_state().await;
    let app = create_test_router(app_state.clone()).await;
    let server = TestServer::new(app).unwrap();

    // Step 1: Register new user
    let email = format!("test{}@example.com", chrono::Utc::now().timestamp());
    let register_req = RegisterRequest {
        first_name: "Test".to_string(),
        last_name: "User".to_string(),
        email: email.clone(),
        password: "SecurePassword123!".to_string(),
        department: Some("Engineering".to_string()),
        position: Some("Developer".to_string()),
        lab_affiliation: Some("Lab A".to_string()),
    };

    let register_response = server.post("/auth/register").json(&register_req).await;

    assert_eq!(register_response.status_code(), 201);
    let register_data: serde_json::Value = register_response.json();
    assert_eq!(register_data["success"], true);
    assert!(register_data["data"]["user_id"].is_string());

    // Step 2: Login with new user
    let login_req = LoginRequest {
        email: email.clone(),
        password: register_req.password.clone(),
        remember_me: Some(false),
    };

    let login_response = server.post("/auth/login").json(&login_req).await;

    assert_eq!(login_response.status_code(), 200);
    let login_data: serde_json::Value = login_response.json();

    // Verify login response structure
    assert_eq!(login_data["success"], true);
    assert!(login_data["data"]["access_token"].is_string());
    assert!(login_data["data"]["user"]["email"].as_str().unwrap() == email);
    assert!(login_data["data"]["expires_at"].is_string());

    // Step 3: Access protected endpoint with token
    let auth_token = login_data["data"]["access_token"]
        .as_str()
        .unwrap()
        .to_string();

    let profile_response = server
        .get("/auth/me")
        .add_header(
            header::AUTHORIZATION,
            format!("Bearer {}", auth_token).parse().unwrap(),
        )
        .await;

    assert_eq!(profile_response.status_code(), 200);
    let profile_data: serde_json::Value = profile_response.json();

    // Verify profile data
    assert_eq!(profile_data["success"], true);
    assert_eq!(profile_data["data"]["email"], email);
    assert_eq!(profile_data["data"]["first_name"], "Test");
    assert_eq!(profile_data["data"]["last_name"], "User");
    assert_eq!(profile_data["data"]["role"], "guest");

    // Cleanup: Delete test user
    cleanup_test_user_by_email(&app_state.db_pool.pool, &email).await;
}

#[tokio::test]
async fn test_invalid_registration_validation() {
    let app_state = create_test_app_state().await;
    let app = create_test_router(app_state).await;
    let server = TestServer::new(app).unwrap();

    // Test invalid email
    let invalid_email_req = RegisterRequest {
        first_name: "Test".to_string(),
        last_name: "User".to_string(),
        email: "invalid-email".to_string(),
        password: "SecurePassword123!".to_string(),
        department: None,
        position: None,
        lab_affiliation: None,
    };

    let response = server.post("/auth/register").json(&invalid_email_req).await;

    assert_eq!(response.status_code(), 400);
    let error_data: serde_json::Value = response.json();
    assert_eq!(error_data["success"], false);
    assert!(error_data["error"].as_str().unwrap().contains("email"));

    // Test weak password
    let weak_password_req = RegisterRequest {
        first_name: "Test".to_string(),
        last_name: "User".to_string(),
        email: "test@example.com".to_string(),
        password: "123".to_string(), // Too short
        department: None,
        position: None,
        lab_affiliation: None,
    };

    let response = server.post("/auth/register").json(&weak_password_req).await;

    assert_eq!(response.status_code(), 400);
    let error_data: serde_json::Value = response.json();
    assert_eq!(error_data["success"], false);
    assert!(error_data["error"].as_str().unwrap().contains("Password"));
}

#[tokio::test]
async fn test_duplicate_user_registration() {
    let app_state = create_test_app_state().await;
    let app = create_test_router(app_state.clone()).await;
    let server = TestServer::new(app).unwrap();

    let email = format!("duplicate{}@example.com", chrono::Utc::now().timestamp());
    let register_req = RegisterRequest {
        first_name: "Test".to_string(),
        last_name: "User".to_string(),
        email: email.clone(),
        password: "SecurePassword123!".to_string(),
        department: None,
        position: None,
        lab_affiliation: None,
    };

    // First registration should succeed
    let first_response = server.post("/auth/register").json(&register_req).await;
    assert_eq!(first_response.status_code(), 201);

    // Second registration with same email should fail
    let second_response = server.post("/auth/register").json(&register_req).await;
    assert_eq!(second_response.status_code(), 409);

    let error_data: serde_json::Value = second_response.json();
    assert_eq!(error_data["success"], false);
    assert!(
        error_data["error"]
            .as_str()
            .unwrap()
            .contains("already exists")
    );

    // Cleanup
    cleanup_test_user_by_email(&app_state.db_pool.pool, &email).await;
}

#[tokio::test]
async fn test_login_with_invalid_credentials() {
    let app_state = create_test_app_state().await;
    let app = create_test_router(app_state).await;
    let server = TestServer::new(app).unwrap();

    // Test with non-existent user
    let login_req = LoginRequest {
        email: "nonexistent@example.com".to_string(),
        password: "AnyPassword123!".to_string(),
        remember_me: Some(false),
    };

    let response = server.post("/auth/login").json(&login_req).await;

    assert_eq!(response.status_code(), 401);
    let error_data: serde_json::Value = response.json();
    assert_eq!(error_data["success"], false);
    assert!(error_data["error"].as_str().unwrap().contains("Invalid"));
}

#[tokio::test]
async fn test_session_management_flow() {
    let app_state = create_test_app_state().await;
    let app = create_test_router(app_state.clone()).await;
    let server = TestServer::new(app).unwrap();

    // Create and login user
    let email = format!("session{}@example.com", chrono::Utc::now().timestamp());
    let register_req = RegisterRequest {
        first_name: "Session".to_string(),
        last_name: "User".to_string(),
        email: email.clone(),
        password: "SecurePassword123!".to_string(),
        department: None,
        position: None,
        lab_affiliation: None,
    };

    // Register user
    let register_response = server.post("/auth/register").json(&register_req).await;
    assert_eq!(register_response.status_code(), 201);

    // Login user
    let login_req = LoginRequest {
        email: email.clone(),
        password: register_req.password,
        remember_me: Some(false),
    };

    let login_response = server.post("/auth/login").json(&login_req).await;
    assert_eq!(login_response.status_code(), 200);

    let login_data: serde_json::Value = login_response.json();
    let auth_token = login_data["data"]["access_token"]
        .as_str()
        .unwrap()
        .to_string();

    // Get user sessions
    let sessions_response = server
        .get("/auth/sessions")
        .add_header(
            "Authorization".parse().unwrap(),
            format!("Bearer {}", auth_token).parse().unwrap(),
        )
        .await;

    assert_eq!(sessions_response.status_code(), 200);
    let sessions_data: serde_json::Value = sessions_response.json();
    assert_eq!(sessions_data["success"], true);
    assert!(sessions_data["data"].is_array());
    assert!(sessions_data["data"].as_array().unwrap().len() >= 1);

    // Verify session structure
    let session = &sessions_data["data"][0];
    assert!(session["id"].is_string());
    assert!(session["created_at"].is_string());
    assert!(session["last_used_at"].is_string());
    assert_eq!(session["revoked"], false);

    // Cleanup
    cleanup_test_user_by_email(&app_state.db_pool.pool, &email).await;
}

#[tokio::test]
async fn test_token_validation() {
    let app_state = create_test_app_state().await;
    let app = create_test_router(app_state.clone()).await;
    let server = TestServer::new(app).unwrap();

    // Create and login user
    let email = format!("token{}@example.com", chrono::Utc::now().timestamp());
    let (auth_token, _) = create_test_user_and_login(&server, &email).await;

    // Test valid token
    let validate_req = json!({
        "token": auth_token
    });

    let validate_response = server.post("/validate/token").json(&validate_req).await;

    assert_eq!(validate_response.status_code(), 200);
    let validate_data: serde_json::Value = validate_response.json();
    assert_eq!(validate_data["success"], true);
    assert_eq!(validate_data["data"]["valid"], true);
    assert!(validate_data["data"]["user_id"].is_string());
    assert_eq!(validate_data["data"]["email"], email);

    // Test invalid token
    let invalid_validate_req = json!({
        "token": "invalid-token-12345"
    });

    let invalid_response = server
        .post("/validate/token")
        .json(&invalid_validate_req)
        .await;

    assert_eq!(invalid_response.status_code(), 200);
    let invalid_data: serde_json::Value = invalid_response.json();
    assert_eq!(invalid_data["success"], true);
    assert_eq!(invalid_data["data"]["valid"], false);

    // Cleanup
    cleanup_test_user_by_email(&app_state.db_pool.pool, &email).await;
}

#[tokio::test]
async fn test_password_change_flow() {
    let app_state = create_test_app_state().await;
    let app = create_test_router(app_state.clone()).await;
    let server = TestServer::new(app).unwrap();

    // Create and login user
    let email = format!("pwchange{}@example.com", chrono::Utc::now().timestamp());
    let (auth_token, original_password) = create_test_user_and_login(&server, &email).await;

    // Change password
    let new_password = "NewSecurePassword456!";
    let change_password_req = json!({
        "current_password": original_password,
        "new_password": new_password
    });

    let change_response = server
        .put("/auth/change-password")
        .add_header(
            "Authorization".parse().unwrap(),
            format!("Bearer {}", auth_token).parse().unwrap(),
        )
        .json(&change_password_req)
        .await;

    assert_eq!(change_response.status_code(), 200);
    let change_data: serde_json::Value = change_response.json();
    assert_eq!(change_data["success"], true);

    // Verify old password no longer works
    let old_login_req = LoginRequest {
        email: email.clone(),
        password: original_password.to_string(),
        remember_me: Some(false),
    };

    let old_login_response = server.post("/auth/login").json(&old_login_req).await;
    assert_eq!(old_login_response.status_code(), 401);

    // Verify new password works
    let new_login_req = LoginRequest {
        email: email.clone(),
        password: new_password.to_string(),
        remember_me: Some(false),
    };

    let new_login_response = server.post("/auth/login").json(&new_login_req).await;
    assert_eq!(new_login_response.status_code(), 200);

    // Cleanup
    cleanup_test_user_by_email(&app_state.db_pool.pool, &email).await;
}

#[tokio::test]
async fn test_logout_flow() {
    let app_state = create_test_app_state().await;
    let app = create_test_router(app_state.clone()).await;
    let server = TestServer::new(app).unwrap();

    // Create and login user
    let email = format!("logout{}@example.com", chrono::Utc::now().timestamp());
    let (auth_token, _) = create_test_user_and_login(&server, &email).await;

    // Verify token works initially
    let profile_response = server
        .get("/auth/me")
        .add_header(
            "Authorization".parse().unwrap(),
            format!("Bearer {}", auth_token).parse().unwrap(),
        )
        .await;
    assert_eq!(profile_response.status_code(), 200);

    // Logout
    let logout_response = server
        .post("/auth/logout")
        .add_header(
            "Authorization".parse().unwrap(),
            format!("Bearer {}", auth_token).parse().unwrap(),
        )
        .await;
    assert_eq!(logout_response.status_code(), 200);

    // Verify token no longer works after logout
    let profile_response_after = server
        .get("/auth/me")
        .add_header(
            "Authorization".parse().unwrap(),
            format!("Bearer {}", auth_token).parse().unwrap(),
        )
        .await;
    assert_eq!(profile_response_after.status_code(), 401);

    // Cleanup
    cleanup_test_user_by_email(&app_state.db_pool.pool, &email).await;
}

// Helper function to create test router with real handlers
async fn create_test_router(app_state: AppState) -> Router {
    use auth_service::handlers::{auth, validation};
    
    // For tests, we'll create simplified routes without middleware complexity
    Router::new()
        .route("/auth/register", axum::routing::post(auth::register))
        .route("/auth/login", axum::routing::post(auth::login))
        .route("/auth/forgot-password", axum::routing::post(auth::forgot_password))
        .route("/auth/logout", axum::routing::post(auth::logout))
        .route("/validate/token", axum::routing::post(validation::validate_token))
        // These routes would normally require auth middleware, but for testing
        // we'll handle authentication differently
        .route("/auth/me", axum::routing::get(test_get_current_user))
        .route("/auth/sessions", axum::routing::get(test_get_sessions))
        .route("/auth/change-password", axum::routing::put(test_change_password))
        .with_state(app_state)
}

// Test wrapper for get_current_user that extracts user from token
async fn test_get_current_user(
    State(state): State<AppState>,
    headers: axum::http::HeaderMap,
) -> Result<axum::Json<serde_json::Value>, AuthError> {
    let token = extract_token_from_headers(&headers)?;
    let token_response = state.auth_service.validate_token(&token).await?;
    
    if !token_response.valid {
        return Err(AuthError::TokenInvalid);
    }
    
    let user_id = token_response.user_id.ok_or(AuthError::TokenInvalid)?;
    let user = state.auth_service.get_user_by_id(user_id).await?;
    
    auth::get_current_user(State(state), user).await
}

// Test wrapper for get_sessions
async fn test_get_sessions(
    State(state): State<AppState>,
    headers: axum::http::HeaderMap,
) -> Result<axum::Json<serde_json::Value>, AuthError> {
    let token = extract_token_from_headers(&headers)?;
    let token_response = state.auth_service.validate_token(&token).await?;
    
    if !token_response.valid {
        return Err(AuthError::TokenInvalid);
    }
    
    let user_id = token_response.user_id.ok_or(AuthError::TokenInvalid)?;
    let user = state.auth_service.get_user_by_id(user_id).await?;
    
    auth::get_sessions(State(state), user).await
}

// Test wrapper for change_password
async fn test_change_password(
    State(state): State<AppState>,
    headers: axum::http::HeaderMap,
    axum::Json(request): axum::Json<auth::ChangePasswordRequest>,
) -> Result<axum::Json<serde_json::Value>, AuthError> {
    let token = extract_token_from_headers(&headers)?;
    let token_response = state.auth_service.validate_token(&token).await?;
    
    if !token_response.valid {
        return Err(AuthError::TokenInvalid);
    }
    
    let user_id = token_response.user_id.ok_or(AuthError::TokenInvalid)?;
    let user = state.auth_service.get_user_by_id(user_id).await?;
    
    auth::change_password(State(state), axum::Json(request), user).await
}

// Helper to extract token from headers
fn extract_token_from_headers(headers: &axum::http::HeaderMap) -> Result<String, AuthError> {
    headers
        .get(header::AUTHORIZATION)
        .and_then(|h| h.to_str().ok())
        .and_then(|h| h.strip_prefix("Bearer "))
        .map(|s| s.to_string())
        .ok_or_else(|| AuthError::authentication("Missing or invalid authorization header"))
}

// Helper function to create user and login
async fn create_test_user_and_login(server: &TestServer, email: &str) -> (String, String) {
    let password = "SecurePassword123!";

    // Register user
    let register_req = RegisterRequest {
        first_name: "Test".to_string(),
        last_name: "User".to_string(),
        email: email.to_string(),
        password: password.to_string(),
        department: None,
        position: None,
        lab_affiliation: None,
    };

    let register_response = server.post("/auth/register").json(&register_req).await;
    assert_eq!(register_response.status_code(), 201);

    // Login user
    let login_req = LoginRequest {
        email: email.to_string(),
        password: password.to_string(),
        remember_me: Some(false),
    };

    let login_response = server.post("/auth/login").json(&login_req).await;
    assert_eq!(login_response.status_code(), 200);

    let login_data: serde_json::Value = login_response.json();
    let auth_token = login_data["data"]["access_token"]
        .as_str()
        .unwrap()
        .to_string();

    (auth_token, password.to_string())
}

// Helper function to cleanup test user by email
async fn cleanup_test_user_by_email(pool: &sqlx::PgPool, email: &str) {
    let _ = sqlx::query(
        "DELETE FROM user_sessions WHERE user_id IN (SELECT id FROM users WHERE email = $1)",
    )
    .bind(email)
    .execute(pool)
    .await;

    let _ = sqlx::query("DELETE FROM users WHERE email = $1")
        .bind(email)
        .execute(pool)
        .await;
}
