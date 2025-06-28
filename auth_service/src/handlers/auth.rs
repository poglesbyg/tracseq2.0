use axum::{extract::State, Json};
use serde_json::json;
use validator::Validate;

use crate::{
    AppState,
    error::AuthError,
    models::*,
};

/// Login endpoint
#[allow(dead_code)]
pub async fn login(
    State(state): State<AppState>,
    Json(request): Json<LoginRequest>,
) -> Result<Json<serde_json::Value>, AuthError> {
    // Validate request
    request.validate()?;

    // Perform login
    let response = state.auth_service.login(request).await?;

    Ok(Json(json!({
        "success": true,
        "data": response
    })))
}

/// Register endpoint (if registration is enabled)
#[allow(dead_code)]
pub async fn register(
    State(state): State<AppState>,
    Json(request): Json<RegisterRequest>,
) -> Result<Json<serde_json::Value>, AuthError> {
    // Check if registration is enabled
    if !state.config.features.registration_enabled {
        return Err(AuthError::feature_disabled("registration"));
    }

    // Validate request
    request.validate()?;

    // Create user account
    let user = state.auth_service.create_user(
        request.first_name,
        request.last_name,
        request.email,
        request.password,
        UserRole::Guest, // Default role for self-registration
    ).await?;

    Ok(Json(json!({
        "success": true,
        "message": "User registered successfully",
        "user_id": user.id
    })))
}

/// Logout endpoint
#[allow(dead_code)]
pub async fn logout(
    State(state): State<AppState>,
    Json(request): Json<LogoutRequest>,
) -> Result<Json<serde_json::Value>, AuthError> {
    // Revoke session
    sqlx::query("UPDATE user_sessions SET revoked = TRUE, revoked_at = NOW() WHERE id = $1")
        .bind(request.session_id)
        .execute(&state.db_pool.pool)
        .await?;

    Ok(Json(json!({
        "success": true,
        "message": "Logged out successfully"
    })))
}

/// Get current user endpoint (requires authentication)
#[allow(dead_code)]
pub async fn get_current_user(
    State(_state): State<AppState>,
    user: User, // Injected by middleware
) -> Result<Json<serde_json::Value>, AuthError> {
    Ok(Json(json!({
        "success": true,
        "data": {
            "id": user.id,
            "email": user.email,
            "first_name": user.first_name,
            "last_name": user.last_name,
            "role": user.role,
            "status": user.status,
            "email_verified": user.email_verified,
            "last_login_at": user.last_login_at,
            "created_at": user.created_at,
            "department": user.department,
            "position": user.position,
            "lab_affiliation": user.lab_affiliation
        }
    })))
}

/// Get user sessions endpoint
#[allow(dead_code)]
pub async fn get_sessions(
    State(state): State<AppState>,
    user: User, // Injected by middleware
) -> Result<Json<serde_json::Value>, AuthError> {
    let sessions = sqlx::query_as::<_, UserSession>(
        "SELECT * FROM user_sessions WHERE user_id = $1 AND expires_at > NOW() AND revoked = FALSE ORDER BY last_used_at DESC"
    )
    .bind(user.id)
    .fetch_all(&state.db_pool.pool)
    .await?;

    let session_data: Vec<serde_json::Value> = sessions
        .into_iter()
        .map(|session| json!({
            "id": session.id,
            "device_info": session.device_info,
            "ip_address": session.ip_address,
            "created_at": session.created_at,
            "last_used_at": session.last_used_at,
            "expires_at": session.expires_at
        }))
        .collect();

    Ok(Json(json!({
        "success": true,
        "data": session_data
    })))
}

/// Revoke session endpoint
#[allow(dead_code)]
pub async fn revoke_session(
    State(state): State<AppState>,
    axum::extract::Path(session_id): axum::extract::Path<uuid::Uuid>,
    user: User, // Injected by middleware
) -> Result<Json<serde_json::Value>, AuthError> {
    // Revoke the specific session for this user
    let result = sqlx::query(
        "UPDATE user_sessions SET revoked = TRUE, revoked_at = NOW() WHERE id = $1 AND user_id = $2"
    )
    .bind(session_id)
    .bind(user.id)
    .execute(&state.db_pool.pool)
    .await?;

    if result.rows_affected() == 0 {
        return Err(AuthError::SessionNotFound);
    }

    Ok(Json(json!({
        "success": true,
        "message": "Session revoked successfully"
    })))
}

/// Change password endpoint
#[allow(dead_code)]
pub async fn change_password(
    State(state): State<AppState>,
    Json(request): Json<ChangePasswordRequest>,
    user: User, // Injected by middleware
) -> Result<Json<serde_json::Value>, AuthError> {
    // Validate request
    request.validate()?;

    // Verify current password
    if !state.auth_service.verify_password(&request.current_password, &user.password_hash)? {
        return Err(AuthError::InvalidCredentials);
    }

    // Hash new password
    let new_password_hash = state.auth_service.hash_password(&request.new_password)?;

    // Update password
    sqlx::query(
        "UPDATE users SET password_hash = $1, password_changed_at = NOW() WHERE id = $2"
    )
    .bind(&new_password_hash)
    .bind(user.id)
    .execute(&state.db_pool.pool)
    .await?;

    // Revoke all existing sessions for security
    sqlx::query("UPDATE user_sessions SET revoked = TRUE, revoked_at = NOW() WHERE user_id = $1")
        .bind(user.id)
        .execute(&state.db_pool.pool)
        .await?;

    Ok(Json(json!({
        "success": true,
        "message": "Password changed successfully. Please log in again."
    })))
}

/// Refresh token endpoint
#[allow(dead_code)]
pub async fn refresh_token(
    State(state): State<AppState>,
    Json(request): Json<RefreshTokenRequest>,
) -> Result<Json<serde_json::Value>, AuthError> {
    // Validate request
    request.validate()?;

    // Refresh the token using the service
    let response = state.auth_service.refresh_token(&request.refresh_token).await?;

    Ok(Json(json!({
        "success": true,
        "data": response
    })))
}

/// Forgot password endpoint
#[allow(dead_code)]
pub async fn forgot_password(
    State(state): State<AppState>,
    Json(request): Json<ForgotPasswordRequest>,
) -> Result<Json<serde_json::Value>, AuthError> {
    // Check if password reset is enabled
    if !state.config.features.password_reset_enabled {
        return Err(AuthError::feature_disabled("password reset"));
    }

    // Validate request
    request.validate()?;

    // Initiate password reset process
    state.auth_service.forgot_password(&request.email).await?;

    // Always return success for security (don't reveal if user exists)
    Ok(Json(json!({
        "success": true,
        "message": "If an account with that email exists, password reset instructions have been sent."
    })))
}

/// Reset password endpoint
#[allow(dead_code)]
pub async fn reset_password(
    State(state): State<AppState>,
    Json(request): Json<ResetPasswordRequest>,
) -> Result<Json<serde_json::Value>, AuthError> {
    // Check if password reset is enabled
    if !state.config.features.password_reset_enabled {
        return Err(AuthError::feature_disabled("password reset"));
    }

    // Validate request
    request.validate()?;

    // Reset the password using the service
    state.auth_service.reset_password(&request.token, &request.new_password).await?;

    Ok(Json(json!({
        "success": true,
        "message": "Password has been reset successfully. Please log in with your new password."
    })))
}

/// Verify email endpoint
#[allow(dead_code)]
pub async fn verify_email(
    State(state): State<AppState>,
    Json(request): Json<VerifyEmailRequest>,
) -> Result<Json<serde_json::Value>, AuthError> {
    // Check if email verification is required
    if !state.config.features.email_verification_required {
        return Err(AuthError::feature_disabled("email verification"));
    }

    // Validate request
    request.validate()?;

    // Verify the email using the service
    state.auth_service.verify_email(&request.token).await?;

    Ok(Json(json!({
        "success": true,
        "message": "Email address has been verified successfully."
    })))
}

// Helper request models that weren't in the main models

#[derive(Debug, serde::Serialize, serde::Deserialize, validator::Validate)]
pub struct RegisterRequest {
    #[validate(email(message = "Invalid email format"))]
    pub email: String,
    #[validate(length(min = 8, message = "Password must be at least 8 characters"))]
    pub password: String,
    #[validate(length(min = 1, message = "First name is required"))]
    pub first_name: String,
    #[validate(length(min = 1, message = "Last name is required"))]
    pub last_name: String,
    #[allow(dead_code)]
    pub department: Option<String>,
    #[allow(dead_code)]
    pub position: Option<String>,
    #[allow(dead_code)]
    pub lab_affiliation: Option<String>,
}

#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct LogoutRequest {
    #[allow(dead_code)]
    pub session_id: uuid::Uuid,
}

#[derive(Debug, serde::Serialize, serde::Deserialize, validator::Validate)]
pub struct ChangePasswordRequest {
    #[validate(length(min = 1, message = "Current password is required"))]
    pub current_password: String,
    #[validate(length(min = 8, message = "New password must be at least 8 characters"))]
    pub new_password: String,
}

#[derive(Debug, serde::Serialize, serde::Deserialize, validator::Validate)]
pub struct RefreshTokenRequest {
    #[validate(length(min = 1, message = "Refresh token is required"))]
    pub refresh_token: String,
}

#[derive(Debug, serde::Serialize, serde::Deserialize, validator::Validate)]
pub struct ForgotPasswordRequest {
    #[validate(email(message = "Invalid email format"))]
    pub email: String,
}

#[derive(Debug, serde::Serialize, serde::Deserialize, validator::Validate)]
pub struct ResetPasswordRequest {
    #[validate(length(min = 1, message = "Token is required"))]
    pub token: String,
    #[validate(length(min = 8, message = "Password must be at least 8 characters"))]
    pub new_password: String,
}

#[derive(Debug, serde::Serialize, serde::Deserialize, validator::Validate)]
pub struct VerifyEmailRequest {
    #[validate(length(min = 1, message = "Token is required"))]
    pub token: String,
} 
