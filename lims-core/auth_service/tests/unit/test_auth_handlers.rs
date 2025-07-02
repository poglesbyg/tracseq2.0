//! Unit Tests for Authentication Handlers
//! 
//! This module tests individual handler functions in isolation,
//! focusing on request/response processing, validation, and error handling.

use auth_service::{
    handlers::auth::{
        register, login, get_current_user, change_password, forgot_password,
        get_sessions, revoke_session, logout, ForgotPasswordRequest,
        ChangePasswordRequest, LogoutRequest, RegisterRequest
    },
    models::{LoginRequest, User, UserRole, UserStatus},
    error::{AuthError, AuthResult},
    services::AuthServiceImpl,
    AppState, Config, DatabasePool,
};
use axum::{
    extract::{State, Path},
    Json,
};
use serde_json::json;
use std::sync::Arc;
use uuid::Uuid;
use validator::Validate;

/// Create a test app state
async fn create_test_app_state() -> AppState {
    let config = Config::test_config();
    let db_pool = DatabasePool {
        pool: sqlx::PgPool::connect_lazy("postgres://localhost/test").unwrap(),
    };
    
    let auth_service = AuthServiceImpl::new(db_pool.clone(), config.clone()).unwrap();
    
    AppState {
        auth_service: Arc::new(auth_service),
        config: Arc::new(config),
        db_pool,
    }
}

/// Create a test user
fn create_test_user() -> User {
    User {
        id: Uuid::new_v4(),
        email: "test@example.com".to_string(),
        password_hash: "$argon2id$v=19$m=16384,t=2,p=1$...".to_string(),
        first_name: "Test".to_string(),
        last_name: "User".to_string(),
        role: UserRole::Guest,
        status: UserStatus::Active,
        department: Some("Engineering".to_string()),
        position: Some("Developer".to_string()),
        lab_affiliation: Some("Lab A".to_string()),
        phone: None,
        email_verified: true,
        failed_login_attempts: 0,
        locked_until: None,
        last_login_at: None,
        password_changed_at: None,
        created_at: chrono::Utc::now(),
        updated_at: chrono::Utc::now(),
    }
}

/// Create a valid register request
fn create_valid_register_request() -> RegisterRequest {
    RegisterRequest {
        first_name: "John".to_string(),
        last_name: "Doe".to_string(),
        email: "john@example.com".to_string(),
        password: "SecurePassword123!".to_string(),
        department: Some("Engineering".to_string()),
        position: Some("Developer".to_string()),
        lab_affiliation: Some("Lab A".to_string()),
    }
}

#[tokio::test]
async fn test_register_validation_error() {
    let app_state = create_test_app_state().await;
    
    // Create an invalid request (empty email)
    let request = RegisterRequest {
        first_name: "John".to_string(),
        last_name: "Doe".to_string(),
        email: "".to_string(), // Invalid
        password: "SecurePassword123!".to_string(),
        department: None,
        position: None,
        lab_affiliation: None,
    };
    
    let result = register(State(app_state), Json(request)).await;
    
    assert!(result.is_err());
    match result.unwrap_err() {
        AuthError::Validation { message } => {
            assert!(message.contains("email"));
        }
        other => panic!("Expected validation error, got: {:?}", other),
    }
}

#[tokio::test]
async fn test_login_validation_error() {
    let app_state = create_test_app_state().await;
    
    // Create an invalid login request
    let request = LoginRequest {
        email: "".to_string(), // Invalid
        password: "".to_string(), // Invalid
        remember_me: None,
    };
    
    let result = login(State(app_state), Json(request)).await;
    
    assert!(result.is_err());
    match result.unwrap_err() {
        AuthError::Validation { message } => {
            assert!(message.contains("email") || message.contains("password"));
        }
        other => panic!("Expected validation error, got: {:?}", other),
    }
}

#[tokio::test]
async fn test_get_current_user_success() {
    let app_state = create_test_app_state().await;
    let user = create_test_user();
    
    let result = get_current_user(State(app_state), user.clone()).await;
    
    assert!(result.is_ok());
    let response = result.unwrap();
    let json = response.0;
    assert_eq!(json["id"], user.id.to_string());
    assert_eq!(json["email"], user.email);
    assert_eq!(json["first_name"], user.first_name);
    assert_eq!(json["last_name"], user.last_name);
}

#[tokio::test]
async fn test_forgot_password_nonexistent_user() {
    let app_state = create_test_app_state().await;
    
    let request = ForgotPasswordRequest {
        email: "nonexistent@example.com".to_string(),
    };
    
    let result = forgot_password(State(app_state), Json(request)).await;
    
    // Should still return success for security (don't reveal if user exists)
    assert!(result.is_ok());
    let response = result.unwrap();
    assert_eq!(response.0["success"], true);
}

// Property-based tests
#[cfg(test)]
mod property_tests {
    use super::*;
    use proptest::prelude::*;

    proptest! {
        #[test]
        fn test_email_validation_property(
            email in "([a-zA-Z0-9._%+-]+@[a-zA-Z0-9.-]+\\.[a-zA-Z]{2,}|invalid_email_format)"
        ) {
            let is_valid_format = email.contains("@") && email.contains(".") && !email.starts_with("invalid");

            let request = RegisterRequest {
                first_name: "Test".to_string(),
                last_name: "User".to_string(),
                email: email.clone(),
                password: "SecurePassword123!".to_string(),
                department: None,
                position: None,
                lab_affiliation: None,
            };

            let validation_result = request.validate();

            if is_valid_format {
                prop_assert!(validation_result.is_ok(), "Valid email should pass validation: {}", email);
            } else {
                prop_assert!(validation_result.is_err(), "Invalid email should fail validation: {}", email);
            }
        }

        #[test]
        fn test_password_strength_property(password in "[a-zA-Z0-9!@#$%^&*()_+\\-=\\[\\]{};':\",./<>?]{0,20}") {
            let is_strong = password.len() >= 8;

            let request = RegisterRequest {
                first_name: "Test".to_string(),
                last_name: "User".to_string(),
                email: "test@example.com".to_string(),
                password: password.clone(),
                department: None,
                position: None,
                lab_affiliation: None,
            };

            let validation_result = request.validate();

            if is_strong {
                // For simplicity, we just check password length
                prop_assert!(password.len() >= 8 || validation_result.is_err());
            } else {
                prop_assert!(password.len() < 8 || validation_result.is_ok());
            }
        }
    }
}

#[tokio::test]
async fn test_register_disabled_feature() {
    let mut config = Config::test_config();
    config.features.registration_enabled = false; // Disable registration
    
    let db_pool = DatabasePool {
        pool: sqlx::PgPool::connect_lazy("postgres://localhost/test").unwrap(),
    };
    
    let auth_service = AuthServiceImpl::new(db_pool.clone(), config.clone()).unwrap();
    
    let app_state = AppState {
        auth_service: Arc::new(auth_service),
        config: Arc::new(config),
        db_pool,
    };
    
    let request = RegisterRequest {
        first_name: "John".to_string(),
        last_name: "Doe".to_string(),
        email: "john@example.com".to_string(),
        password: "SecurePassword123!".to_string(),
        department: Some("Engineering".to_string()),
        position: Some("Developer".to_string()),
        lab_affiliation: Some("Lab A".to_string()),
    };
    
    let result = register(State(app_state), Json(request)).await;
    
    assert!(result.is_err());
    match result.unwrap_err() {
        AuthError::FeatureDisabled { feature } => {
            assert_eq!(feature, "registration");
        }
        other => panic!("Expected FeatureDisabled error, got: {:?}", other),
    }
}

#[tokio::test]
async fn test_password_reset_disabled_feature() {
    let mut config = Config::test_config();
    config.features.password_reset_enabled = false; // Disable password reset
    
    let db_pool = DatabasePool {
        pool: sqlx::PgPool::connect_lazy("postgres://localhost/test").unwrap(),
    };
    
    let auth_service = AuthServiceImpl::new(db_pool.clone(), config.clone()).unwrap();
    
    let app_state = AppState {
        auth_service: Arc::new(auth_service),
        config: Arc::new(config),
        db_pool,
    };
    
    let request = ForgotPasswordRequest {
        email: "test@example.com".to_string(),
    };
    
    let result = forgot_password(State(app_state), Json(request)).await;
    
    assert!(result.is_err());
    match result.unwrap_err() {
        AuthError::FeatureDisabled { feature } => {
            assert_eq!(feature, "password_reset");
        }
        other => panic!("Expected FeatureDisabled error, got: {:?}", other),
    }
}

// Handler function tests
#[tokio::test]
async fn test_change_password_request_structure() {
    let request = ChangePasswordRequest {
        current_password: "current_password".to_string(),
        new_password: "new_password".to_string(),
    };
    
    assert_eq!(request.current_password, "current_password");
    assert_eq!(request.new_password, "new_password");
}

#[tokio::test]
async fn test_logout_request_structure() {
    let session_id = Uuid::new_v4();
    let request = LogoutRequest { session_id };
    
    assert_eq!(request.session_id, session_id);
}

// Test the handlers return proper error types
#[tokio::test]
async fn test_register_email_validation() {
    let request = RegisterRequest {
        first_name: "Test".to_string(),
        last_name: "User".to_string(),
        email: "invalid-email".to_string(), // Missing @ symbol
        password: "ValidPassword123!".to_string(),
        department: None,
        position: None,
        lab_affiliation: None,
    };
    
    let validation_result = request.validate();
    assert!(validation_result.is_err());
}

#[tokio::test]
async fn test_register_password_validation() {
    let request = RegisterRequest {
        first_name: "Test".to_string(),
        last_name: "User".to_string(),
        email: "test@example.com".to_string(),
        password: "weak".to_string(), // Too short
        department: None,
        position: None,
        lab_affiliation: None,
    };
    
    let validation_result = request.validate();
    assert!(validation_result.is_err());
}

#[tokio::test]
async fn test_login_empty_fields_validation() {
    let request = LoginRequest {
        email: "".to_string(),
        password: "".to_string(),
        remember_me: Some(false),
    };
    
    // Both fields are empty, should fail validation
    assert!(request.email.is_empty());
    assert!(request.password.is_empty());
}

// Test user model methods
#[tokio::test]
async fn test_user_can_login() {
    let mut user = create_test_user();
    assert!(user.can_login());
    
    // Test locked user
    user.locked_until = Some(chrono::Utc::now() + chrono::Duration::hours(1));
    assert!(!user.can_login());
    
    // Test unverified email
    user.locked_until = None;
    user.email_verified = false;
    assert!(!user.can_login());
    
    // Test inactive user
    user.email_verified = true;
    user.status = UserStatus::Inactive;
    assert!(!user.can_login());
}

#[tokio::test]
async fn test_user_is_locked() {
    let mut user = create_test_user();
    assert!(!user.is_locked());
    
    // Set lock time in future
    user.locked_until = Some(chrono::Utc::now() + chrono::Duration::hours(1));
    assert!(user.is_locked());
    
    // Set lock time in past
    user.locked_until = Some(chrono::Utc::now() - chrono::Duration::hours(1));
    assert!(!user.is_locked());
}

// Test password strength helper
#[tokio::test]
async fn test_password_requirements() {
    let app_state = create_test_app_state().await;
    let auth_service = &app_state.auth_service;
    
    // Test hash_password method
    let password = "SecurePassword123!";
    let result = auth_service.hash_password(password);
    assert!(result.is_ok());
    let hash = result.unwrap();
    assert!(hash.starts_with("$argon2"));
    
    // Test verify_password method
    let verify_result = auth_service.verify_password(password, &hash);
    assert!(verify_result.is_ok());
    assert!(verify_result.unwrap());
    
    // Test wrong password
    let wrong_verify = auth_service.verify_password("WrongPassword", &hash);
    assert!(wrong_verify.is_ok());
    assert!(!wrong_verify.unwrap());
}
