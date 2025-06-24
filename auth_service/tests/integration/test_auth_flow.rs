use auth_service::*;
use crate::test_utils::*;
use axum::Router;
use axum_test::TestServer;
use serde_json::json;

#[tokio::test]
async fn test_complete_registration_login_flow() {
    let app_state = create_test_app_state().await;
    let app = create_auth_routes(app_state);
    let client = AuthTestClient::new(app);

    // Step 1: Register new user
    let register_req = UserFactory::create_valid_register_request();
    let email = register_req.email.clone();
    
    let register_response = client.post_json("/auth/register", &register_req).await;
    assert_eq!(register_response.status_code(), 201);

    // Step 2: Login with new user
    let login_req = LoginRequest {
        email: email.clone(),
        password: register_req.password,
    };
    
    let login_response = client.post_json("/auth/login", &login_req).await;
    assert_eq!(login_response.status_code(), 200);
    
    let login_data: serde_json::Value = login_response.json();
    AuthAssertions::assert_successful_login(&login_data);
    
    // Step 3: Access protected endpoint
    let auth_token = login_data["data"]["access_token"].as_str().unwrap().to_string();
    let auth_client = client.with_auth_token(auth_token);
    
    let profile_response = auth_client.get("/auth/profile").await;
    assert_eq!(profile_response.status_code(), 200);
    
    let profile_data: serde_json::Value = profile_response.json();
    AuthAssertions::assert_user_data(&profile_data, &email);
}

#[tokio::test]
async fn test_session_management_flow() {
    let app_state = create_test_app_state().await;
    let app = create_auth_routes(app_state.clone());
    let client = AuthTestClient::new(app);

    // Create and login user
    let user = UserFactory::create_test_user(&app_state.auth_service).await;
    let login_req = UserFactory::create_valid_login_request(user.email.clone());
    
    let login_response = client.post_json("/auth/login", &login_req).await;
    let login_data: serde_json::Value = login_response.json();
    let auth_token = login_data["data"]["access_token"].as_str().unwrap().to_string();
    
    let auth_client = client.with_auth_token(auth_token);
    
    // Get sessions
    let sessions_response = auth_client.get("/auth/sessions").await;
    assert_eq!(sessions_response.status_code(), 200);
    
    let sessions_data: serde_json::Value = sessions_response.json();
    assert_eq!(sessions_data["success"], true);
    assert!(sessions_data["data"].is_array());
}

#[tokio::test]
async fn test_password_change_flow() {
    let app_state = create_test_app_state().await;
    let app = create_auth_routes(app_state.clone());
    let client = AuthTestClient::new(app);

    // Create and login user
    let user = UserFactory::create_test_user(&app_state.auth_service).await;
    let login_req = UserFactory::create_valid_login_request(user.email.clone());
    
    let login_response = client.post_json("/auth/login", &login_req).await;
    let login_data: serde_json::Value = login_response.json();
    let auth_token = login_data["data"]["access_token"].as_str().unwrap().to_string();
    
    let auth_client = client.with_auth_token(auth_token);
    
    // Change password
    let change_password_req = json!({
        "current_password": "SecurePassword123!",
        "new_password": "NewSecurePassword456!"
    });
    
    let change_response = auth_client.put_json("/auth/change-password", &change_password_req).await;
    assert_eq!(change_response.status_code(), 200);
    
    // Verify old token is invalidated by trying to access protected endpoint
    let profile_response = auth_client.get("/auth/profile").await;
    assert_eq!(profile_response.status_code(), 401); // Should be unauthorized
}

// Helper function to create auth routes (simplified)
fn create_auth_routes(app_state: AppState) -> Router {
    Router::new()
        .route("/auth/register", axum::routing::post(register))
        .route("/auth/login", axum::routing::post(login))
        .route("/auth/profile", axum::routing::get(get_current_user))
        .route("/auth/sessions", axum::routing::get(get_sessions))
        .route("/auth/change-password", axum::routing::put(change_password))
        .with_state(app_state)
} 
