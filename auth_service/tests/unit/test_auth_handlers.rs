use crate::test_utils::*;
use auth_service::{AuthError, models::*};
use axum::{Json, http::StatusCode};
use serde_json::json;
use serial_test::serial;
use validator::Validate;

#[test_with_auth_db]
async fn test_register_success(test_db: &mut TestDatabase) {
    let app_state = create_test_app_state().await;
    let request = UserFactory::create_valid_register_request();
    let email = request.email.clone();

    let result = register(axum::extract::State(app_state), Json(request)).await;

    assert!(result.is_ok());
    let response = result.unwrap();
    assert_eq!(response.0["success"], true);
    assert!(response.0["user_id"].is_string());

    // Verify user exists in database
    let user = sqlx::query_as::<_, User>("SELECT * FROM users WHERE email = $1")
        .bind(&email)
        .fetch_one(&test_db.pool)
        .await
        .expect("User should exist");

    test_db.track_user(user.id);
    assert_eq!(user.email, email);
    assert_eq!(user.role, UserRole::Guest);
}

#[test_with_auth_db]
async fn test_register_validation_failure(test_db: &mut TestDatabase) {
    let app_state = create_test_app_state().await;
    let request = UserFactory::create_invalid_register_request();

    let result = register(axum::extract::State(app_state), Json(request)).await;

    assert!(result.is_err());
    match result.unwrap_err() {
        AuthError::Validation(_) => {} // Expected
        other => panic!("Expected validation error, got: {:?}", other),
    }
}

#[test_with_auth_db]
async fn test_register_duplicate_email(test_db: &mut TestDatabase) {
    let app_state = create_test_app_state().await;
    let auth_service = &app_state.auth_service;

    // Create first user
    let user = UserFactory::create_test_user(auth_service).await;
    test_db.track_user(user.id);

    // Try to register with same email
    let request = RegisterRequest {
        email: user.email.clone(),
        ..UserFactory::create_valid_register_request()
    };

    let result = register(axum::extract::State(app_state), Json(request)).await;

    assert!(result.is_err());
    match result.unwrap_err() {
        AuthError::UserAlreadyExists => {} // Expected
        other => panic!("Expected UserAlreadyExists error, got: {:?}", other),
    }
}

#[test_with_auth_db]
async fn test_login_success(test_db: &mut TestDatabase) {
    let app_state = create_test_app_state().await;

    // Create test user
    let user = UserFactory::create_test_user(&app_state.auth_service).await;
    test_db.track_user(user.id);

    let request = UserFactory::create_valid_login_request(user.email.clone());
    let result = login(axum::extract::State(app_state), Json(request)).await;

    assert!(result.is_ok());
    let response = result.unwrap();
    AuthAssertions::assert_successful_login(&response.0);
}

#[test_with_auth_db]
async fn test_login_invalid_credentials(test_db: &mut TestDatabase) {
    let app_state = create_test_app_state().await;
    let request = UserFactory::create_invalid_login_request();

    let result = login(axum::extract::State(app_state), Json(request)).await;

    assert!(result.is_err());
}

#[test_with_auth_db]
async fn test_login_validation_failure(test_db: &mut TestDatabase) {
    let app_state = create_test_app_state().await;
    let request = LoginRequest {
        email: "".to_string(),    // Invalid
        password: "".to_string(), // Invalid
    };

    let result = login(axum::extract::State(app_state), Json(request)).await;

    assert!(result.is_err());
    match result.unwrap_err() {
        AuthError::Validation(_) => {} // Expected
        other => panic!("Expected validation error, got: {:?}", other),
    }
}

#[test_with_auth_db]
async fn test_get_current_user_success(test_db: &mut TestDatabase) {
    let app_state = create_test_app_state().await;
    let auth_service = &app_state.auth_service;

    // Create test user
    let user = UserFactory::create_test_user(auth_service).await;
    test_db.track_user(user.id);

    let result = get_current_user(axum::extract::State(app_state), user.clone()).await;

    assert!(result.is_ok());
    let response = result.unwrap();
    AuthAssertions::assert_user_data(&response.0, &user.email);
}

#[test_with_auth_db]
async fn test_change_password_success(test_db: &mut TestDatabase) {
    let app_state = create_test_app_state().await;

    let user = UserFactory::create_test_user(&app_state.auth_service).await;
    test_db.track_user(user.id);

    let request = ChangePasswordRequest {
        current_password: "SecurePassword123!".to_string(),
        new_password: "NewSecurePassword456!".to_string(),
    };

    let result = change_password(axum::extract::State(app_state), Json(request), user).await;

    assert!(result.is_ok());
    let response = result.unwrap();
    assert_eq!(response.0["success"], true);
    assert_eq!(
        response.0["message"],
        "Password changed successfully. Please log in again."
    );
}

#[test_with_auth_db]
async fn test_change_password_wrong_current(test_db: &mut TestDatabase) {
    let app_state = create_test_app_state().await;
    let auth_service = &app_state.auth_service;

    // Create test user
    let user = UserFactory::create_test_user(auth_service).await;
    test_db.track_user(user.id);

    let request = ChangePasswordRequest {
        current_password: "WrongPassword".to_string(),
        new_password: "NewSecurePassword456!".to_string(),
    };

    let result = change_password(axum::extract::State(app_state), Json(request), user).await;

    assert!(result.is_err());
    match result.unwrap_err() {
        AuthError::InvalidCredentials => {} // Expected
        other => panic!("Expected InvalidCredentials error, got: {:?}", other),
    }
}

#[test_with_auth_db]
async fn test_forgot_password_success(test_db: &mut TestDatabase) {
    let app_state = create_test_app_state().await;
    let auth_service = &app_state.auth_service;

    // Create test user
    let user = UserFactory::create_test_user(auth_service).await;
    test_db.track_user(user.id);

    let request = ForgotPasswordRequest {
        email: user.email.clone(),
    };

    let result = forgot_password(axum::extract::State(app_state), Json(request)).await;

    assert!(result.is_ok());
    let response = result.unwrap();
    assert_eq!(response.0["success"], true);
    assert!(
        response.0["message"]
            .as_str()
            .unwrap()
            .contains("password reset instructions")
    );
}

#[test_with_auth_db]
async fn test_forgot_password_nonexistent_user(test_db: &mut TestDatabase) {
    let app_state = create_test_app_state().await;

    let request = ForgotPasswordRequest {
        email: "nonexistent@example.com".to_string(),
    };

    let result = forgot_password(axum::extract::State(app_state), Json(request)).await;

    // Should still return success for security (don't reveal if user exists)
    assert!(result.is_ok());
    let response = result.unwrap();
    assert_eq!(response.0["success"], true);
}

#[test_with_auth_db]
async fn test_get_sessions_success(test_db: &mut TestDatabase) {
    let app_state = create_test_app_state().await;
    let auth_service = &app_service.auth_service;

    // Create test user
    let user = UserFactory::create_test_user(auth_service).await;
    test_db.track_user(user.id);

    // Create a session for the user
    let session_id = uuid::Uuid::new_v4();
    sqlx::query(
        "INSERT INTO user_sessions (id, user_id, device_info, ip_address, expires_at) VALUES ($1, $2, $3, $4, NOW() + INTERVAL '1 day')"
    )
    .bind(session_id)
    .bind(user.id)
    .bind("Test Device")
    .bind("127.0.0.1")
    .execute(&test_db.pool)
    .await
    .expect("Failed to create test session");

    let result = get_sessions(axum::extract::State(app_state), user).await;

    assert!(result.is_ok());
    let response = result.unwrap();
    assert_eq!(response.0["success"], true);
    assert!(response.0["data"].is_array());
    let sessions = response.0["data"].as_array().unwrap();
    assert!(sessions.len() > 0);
}

#[test_with_auth_db]
async fn test_revoke_session_success(test_db: &mut TestDatabase) {
    let app_state = create_test_app_state().await;
    let auth_service = &app_state.auth_service;

    // Create test user
    let user = UserFactory::create_test_user(auth_service).await;
    test_db.track_user(user.id);

    // Create a session for the user
    let session_id = uuid::Uuid::new_v4();
    sqlx::query(
        "INSERT INTO user_sessions (id, user_id, device_info, ip_address, expires_at) VALUES ($1, $2, $3, $4, NOW() + INTERVAL '1 day')"
    )
    .bind(session_id)
    .bind(user.id)
    .bind("Test Device")
    .bind("127.0.0.1")
    .execute(&test_db.pool)
    .await
    .expect("Failed to create test session");

    let result = revoke_session(
        axum::extract::State(app_state),
        axum::extract::Path(session_id),
        user,
    )
    .await;

    assert!(result.is_ok());
    let response = result.unwrap();
    assert_eq!(response.0["success"], true);
    assert_eq!(response.0["message"], "Session revoked successfully");

    // Verify session was revoked
    let revoked: bool = sqlx::query_scalar("SELECT revoked FROM user_sessions WHERE id = $1")
        .bind(session_id)
        .fetch_one(&test_db.pool)
        .await
        .expect("Session should still exist");

    assert!(revoked);
}

#[test_with_auth_db]
async fn test_revoke_nonexistent_session(test_db: &mut TestDatabase) {
    let app_state = create_test_app_state().await;
    let auth_service = &app_state.auth_service;

    // Create test user
    let user = UserFactory::create_test_user(auth_service).await;
    test_db.track_user(user.id);

    let nonexistent_session_id = uuid::Uuid::new_v4();

    let result = revoke_session(
        axum::extract::State(app_state),
        axum::extract::Path(nonexistent_session_id),
        user,
    )
    .await;

    assert!(result.is_err());
    match result.unwrap_err() {
        AuthError::SessionNotFound => {} // Expected
        other => panic!("Expected SessionNotFound error, got: {:?}", other),
    }
}

#[tokio::test]
async fn test_logout_success() {
    let app_state = create_test_app_state().await;
    let session_id = uuid::Uuid::new_v4();

    // Create a session first
    sqlx::query(
        "INSERT INTO user_sessions (id, user_id, device_info, ip_address, expires_at) VALUES ($1, $2, $3, $4, NOW() + INTERVAL '1 day')"
    )
    .bind(session_id)
    .bind(uuid::Uuid::new_v4()) // Random user id for this test
    .bind("Test Device")
    .bind("127.0.0.1")
    .execute(&app_state.db_pool.pool)
    .await
    .expect("Failed to create test session");

    let request = LogoutRequest { session_id };

    let result = logout(axum::extract::State(app_state.clone()), Json(request)).await;

    assert!(result.is_ok());
    let response = result.unwrap();
    assert_eq!(response.0["success"], true);
    assert_eq!(response.0["message"], "Logged out successfully");

    // Verify session was revoked
    let revoked: bool = sqlx::query_scalar("SELECT revoked FROM user_sessions WHERE id = $1")
        .bind(session_id)
        .fetch_one(&app_state.db_pool.pool)
        .await
        .expect("Session should still exist");

    assert!(revoked);

    // Cleanup
    let _ = sqlx::query("DELETE FROM user_sessions WHERE id = $1")
        .bind(session_id)
        .execute(&app_state.db_pool.pool)
        .await;
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
                // Strong passwords should pass validation (assuming other fields are valid)
                prop_assert!(validation_result.is_ok() ||
                           validation_result.as_ref().err().unwrap().to_string().contains("email"),
                           "Strong password should pass validation: {}", password);
            } else {
                // Weak passwords should fail validation
                prop_assert!(validation_result.is_err(), "Weak password should fail validation: {}", password);
            }
        }
    }
}
