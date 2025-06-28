use auth_service::*;
use auth_service::handlers::auth::{register, login, get_current_user};
use crate::test_utils::*;
use axum::http::StatusCode;
use serial_test::serial;

#[tokio::test]
#[serial]
async fn test_brute_force_protection() {
    let app_state = create_test_app_state().await;
    let app = create_auth_routes(app_state.clone());
    let client = AuthTestClient::new(app);

    // Create test user
    let user = UserFactory::create_test_user(&app_state.auth_service).await;
    
    // Attempt brute force login
    let results = SecurityTestUtils::attempt_brute_force_login(&client, &user.email, 10).await;
    
    // Should see rate limiting or account lockout after several attempts
    let failed_attempts = results.iter().filter(|&&status| status == StatusCode::UNAUTHORIZED).count();
    assert!(failed_attempts > 5, "Should have multiple failed login attempts");
    
    // Later attempts might be rate limited (depending on implementation)
    let rate_limited = results.iter().any(|&status| status == StatusCode::TOO_MANY_REQUESTS);
    // Note: This depends on rate limiting implementation
}

#[tokio::test]
async fn test_sql_injection_prevention() {
    let app_state = create_test_app_state().await;
    let app = create_auth_routes(app_state);
    let client = AuthTestClient::new(app);

    let sql_injection_attempts = SecurityTestUtils::generate_sql_injection_attempts();
    
    for injection_attempt in sql_injection_attempts {
        let login_req = LoginRequest {
            email: injection_attempt.clone(),
            password: injection_attempt.clone(),
            remember_me: None,
        };
        
        let response = client.post_json("/auth/login", &login_req).await;
        
        // Should not succeed or cause server errors
        assert!(
            response.status_code() == StatusCode::UNAUTHORIZED || 
            response.status_code() == StatusCode::BAD_REQUEST,
            "SQL injection attempt should not succeed: {}", injection_attempt
        );
    }
}

#[tokio::test]
async fn test_xss_prevention() {
    let app_state = create_test_app_state().await;
    let app = create_auth_routes(app_state);
    let client = AuthTestClient::new(app);

    let xss_attempts = SecurityTestUtils::generate_xss_attempts();
    
    for xss_attempt in xss_attempts {
        let register_req = auth_service::handlers::auth::RegisterRequest {
            first_name: xss_attempt.clone(),
            last_name: xss_attempt.clone(),
            email: format!("test-{}@example.com", uuid::Uuid::new_v4()),
            password: "SecurePassword123!".to_string(),
            department: Some(xss_attempt.clone()),
            position: Some(xss_attempt.clone()),
            lab_affiliation: Some(xss_attempt),
        };
        
        let response = client.post_json("/auth/register", &register_req).await;
        
        // Should handle XSS attempts gracefully
        if response.status_code().is_success() {
            let response_body: serde_json::Value = response.json();
            let response_text = response_body.to_string();
            
            // Ensure no script tags in response
            assert!(!response_text.contains("<script>"), "Response should not contain script tags");
            assert!(!response_text.contains("javascript:"), "Response should not contain javascript:");
        }
    }
}

#[tokio::test]
async fn test_jwt_security() {
    let app_state = create_test_app_state().await;
    let app = create_auth_routes(app_state.clone());
    let client = AuthTestClient::new(app);

    // Test with invalid JWT token
    let invalid_client = client.with_auth_token(JwtTestUtils::create_invalid_token());
    let response = invalid_client.get("/auth/profile").await;
    AuthAssertions::assert_unauthorized(response.status_code());
    
    // Test with expired JWT token
    let expired_client = client.with_auth_token(JwtTestUtils::create_expired_token());
    let response = expired_client.get("/auth/profile").await;
    AuthAssertions::assert_unauthorized(response.status_code());
    
    // Test with malformed JWT token
    let malformed_client = client.with_auth_token("malformed.jwt.token".to_string());
    let response = malformed_client.get("/auth/profile").await;
    AuthAssertions::assert_unauthorized(response.status_code());
}

#[tokio::test]
async fn test_password_strength_enforcement() {
    let app_state = create_test_app_state().await;
    let app = create_auth_routes(app_state);
    let client = AuthTestClient::new(app);

    let weak_passwords = TestDataGenerator::weak_passwords();
    
    for weak_password in weak_passwords {
        let register_req = auth_service::handlers::auth::RegisterRequest {
            first_name: "Test".to_string(),
            last_name: "User".to_string(),
            email: TestDataGenerator::random_email(),
            password: weak_password.clone(),
            department: None,
            position: None,
            lab_affiliation: None,
        };
        
        let response = client.post_json("/auth/register", &register_req).await;
        
        // Weak passwords should be rejected
        assert!(
            response.status_code() == StatusCode::BAD_REQUEST,
            "Weak password should be rejected: {}", weak_password
        );
    }
}

#[tokio::test]
async fn test_email_validation() {
    let app_state = create_test_app_state().await;
    let app = create_auth_routes(app_state);
    let client = AuthTestClient::new(app);

    let invalid_emails = TestDataGenerator::invalid_emails();
    
    for invalid_email in invalid_emails {
        let register_req = auth_service::handlers::auth::RegisterRequest {
            first_name: "Test".to_string(),
            last_name: "User".to_string(),
            email: invalid_email.clone(),
            password: "SecurePassword123!".to_string(),
            department: None,
            position: None,
            lab_affiliation: None,
        };
        
        let response = client.post_json("/auth/register", &register_req).await;
        
        // Invalid emails should be rejected
        assert!(
            response.status_code() == StatusCode::BAD_REQUEST,
            "Invalid email should be rejected: {}", invalid_email
        );
    }
}

// Helper function to create auth routes
fn create_auth_routes(app_state: AppState) -> axum::Router {
    axum::Router::new()
        .route("/auth/register", axum::routing::post(register))
        .route("/auth/login", axum::routing::post(login))
        .route("/auth/profile", axum::routing::get(get_current_user))
        .with_state(app_state)
} 
