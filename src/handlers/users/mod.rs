pub mod auth_helpers;

use std::net::SocketAddr;

use axum::{
    extract::{ConnectInfo, Path, Query, State},
    http::{HeaderMap, StatusCode},
    response::Json,
    Extension,
};
use serde_json::{json, Value};
use uuid::Uuid;
use validator::Validate;

use crate::{
    models::user::{
        ChangePasswordRequest, ConfirmResetPasswordRequest, CreateUserRequest, LoginRequest,
        ResetPasswordRequest, UpdateUserRequest, User, UserListQuery, UserSafeProfile,
    },
    services::auth_service::AuthService,
    AppComponents,
};

use auth_helpers::{require_admin, require_auth, verify_auth_token};

/// Login endpoint
pub async fn login(
    State(components): State<AppComponents>,
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
    headers: HeaderMap,
    Json(request): Json<LoginRequest>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    // Validate request
    if let Err(validation_errors) = request.validate() {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(json!({
                "error": {
                    "code": "VALIDATION_FAILED",
                    "message": "Request validation failed",
                    "details": validation_errors
                }
            })),
        ));
    }

    // Extract IP address and user agent
    let ip_address = Some(addr.ip());
    let user_agent = headers
        .get("user-agent")
        .and_then(|v| v.to_str().ok())
        .map(|s| s.to_string());

    match components
        .auth_service
        .login(request, ip_address, user_agent)
        .await
    {
        Ok(response) => Ok(Json(json!({
            "success": true,
            "data": response
        }))),
        Err(e) => Err((
            StatusCode::UNAUTHORIZED,
            Json(json!({
                "error": {
                    "code": "AUTHENTICATION_FAILED",
                    "message": e.to_string()
                }
            })),
        )),
    }
}

/// Logout endpoint
pub async fn logout(
    State(components): State<AppComponents>,
    headers: HeaderMap,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    let (current_user, session) = verify_auth_token(&components, &headers).await?;

    match components
        .auth_service
        .logout(session.id, Some(current_user.id))
        .await
    {
        Ok(()) => Ok(Json(json!({
            "success": true,
            "message": "Logged out successfully"
        }))),
        Err(e) => Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({
                "error": {
                    "code": "LOGOUT_FAILED",
                    "message": e.to_string()
                }
            })),
        )),
    }
}

/// Get current user profile
pub async fn get_current_user(
    State(components): State<AppComponents>,
    headers: HeaderMap,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    let current_user = require_auth(&components, &headers).await?;
    let user_profile: UserSafeProfile = current_user.into();
    Ok(Json(json!({
        "success": true,
        "data": user_profile
    })))
}

/// Update current user profile
pub async fn update_current_user(
    State(components): State<AppComponents>,
    Extension(current_user): Extension<User>,
    Json(request): Json<UpdateUserRequest>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    // Validate request
    if let Err(validation_errors) = request.validate() {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(json!({
                "error": {
                    "code": "VALIDATION_FAILED",
                    "message": "Request validation failed",
                    "details": validation_errors
                }
            })),
        ));
    }

    // Users can't change their own role or status
    let mut safe_request = request;
    safe_request.role = None;
    safe_request.status = None;

    match components
        .user_manager
        .update_user(current_user.id, safe_request)
        .await
    {
        Ok(updated_user) => {
            let user_profile: UserSafeProfile = updated_user.into();
            Ok(Json(json!({
                "success": true,
                "data": user_profile
            })))
        }
        Err(e) => Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({
                "error": {
                    "code": "UPDATE_FAILED",
                    "message": format!("Failed to update user: {}", e)
                }
            })),
        )),
    }
}

/// Change password
pub async fn change_password(
    State(components): State<AppComponents>,
    Extension(current_user): Extension<User>,
    Json(request): Json<ChangePasswordRequest>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    // Validate request
    if let Err(validation_errors) = request.validate() {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(json!({
                "error": {
                    "code": "VALIDATION_FAILED",
                    "message": "Request validation failed",
                    "details": validation_errors
                }
            })),
        ));
    }

    match components
        .auth_service
        .change_password(current_user.id, request)
        .await
    {
        Ok(()) => Ok(Json(json!({
            "success": true,
            "message": "Password changed successfully"
        }))),
        Err(e) => Err((
            StatusCode::BAD_REQUEST,
            Json(json!({
                "error": {
                    "code": "PASSWORD_CHANGE_FAILED",
                    "message": e.to_string()
                }
            })),
        )),
    }
}

/// Request password reset
pub async fn request_password_reset(
    State(components): State<AppComponents>,
    Json(request): Json<ResetPasswordRequest>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    // Validate request
    if let Err(validation_errors) = request.validate() {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(json!({
                "error": {
                    "code": "VALIDATION_FAILED",
                    "message": "Request validation failed",
                    "details": validation_errors
                }
            })),
        ));
    }

    // Always return success for security (don't reveal if email exists)
    let _ = components
        .auth_service
        .request_password_reset(request)
        .await;

    Ok(Json(json!({
        "success": true,
        "message": "If the email exists, a password reset link has been sent"
    })))
}

/// Confirm password reset
pub async fn confirm_password_reset(
    State(components): State<AppComponents>,
    Json(request): Json<ConfirmResetPasswordRequest>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    // Validate request
    if let Err(validation_errors) = request.validate() {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(json!({
                "error": {
                    "code": "VALIDATION_FAILED",
                    "message": "Request validation failed",
                    "details": validation_errors
                }
            })),
        ));
    }

    match components
        .auth_service
        .confirm_password_reset(request)
        .await
    {
        Ok(()) => Ok(Json(json!({
            "success": true,
            "message": "Password reset successfully"
        }))),
        Err(e) => Err((
            StatusCode::BAD_REQUEST,
            Json(json!({
                "error": {
                    "code": "PASSWORD_RESET_FAILED",
                    "message": e.to_string()
                }
            })),
        )),
    }
}

/// Get user sessions
pub async fn get_user_sessions(
    State(components): State<AppComponents>,
    Extension(current_user): Extension<User>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    match components
        .auth_service
        .get_user_sessions(current_user.id)
        .await
    {
        Ok(sessions) => Ok(Json(json!({
            "success": true,
            "data": sessions
        }))),
        Err(e) => Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({
                "error": {
                    "code": "SESSIONS_FETCH_FAILED",
                    "message": e.to_string()
                }
            })),
        )),
    }
}

/// Revoke a session
pub async fn revoke_session(
    State(components): State<AppComponents>,
    Extension(current_user): Extension<User>,
    Path(session_id): Path<Uuid>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    match components
        .auth_service
        .revoke_session(session_id, current_user.id)
        .await
    {
        Ok(()) => Ok(Json(json!({
            "success": true,
            "message": "Session revoked successfully"
        }))),
        Err(e) => Err((
            StatusCode::BAD_REQUEST,
            Json(json!({
                "error": {
                    "code": "SESSION_REVOKE_FAILED",
                    "message": e.to_string()
                }
            })),
        )),
    }
}

/// Revoke all sessions except current
pub async fn revoke_all_sessions(
    State(components): State<AppComponents>,
    Extension(current_user): Extension<User>,
    Extension(session_id): Extension<Uuid>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    match components
        .auth_service
        .revoke_all_sessions(current_user.id, Some(session_id))
        .await
    {
        Ok(count) => Ok(Json(json!({
            "success": true,
            "message": format!("Revoked {} sessions", count)
        }))),
        Err(e) => Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({
                "error": {
                    "code": "SESSIONS_REVOKE_FAILED",
                    "message": e.to_string()
                }
            })),
        )),
    }
}

// Admin-only endpoints

/// Create new user (admin only)
pub async fn create_user(
    State(components): State<AppComponents>,
    headers: HeaderMap,
    Json(request): Json<CreateUserRequest>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    // Require admin privileges
    let current_user = require_admin(&components, &headers).await?;

    // Validate request
    if let Err(validation_errors) = request.validate() {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(json!({
                "error": {
                    "code": "VALIDATION_FAILED",
                    "message": "Request validation failed",
                    "details": validation_errors
                }
            })),
        ));
    }

    match components
        .user_manager
        .create_user(request, Some(current_user.id))
        .await
    {
        Ok(user) => {
            let user_profile: UserSafeProfile = user.into();
            Ok(Json(json!({
                "success": true,
                "data": user_profile
            })))
        }
        Err(e) => Err((
            StatusCode::BAD_REQUEST,
            Json(json!({
                "error": {
                    "code": "USER_CREATION_FAILED",
                    "message": format!("Failed to create user: {}", e)
                }
            })),
        )),
    }
}

/// List users (admin only)
pub async fn list_users(
    State(components): State<AppComponents>,
    Extension(current_user): Extension<User>,
    Query(query): Query<UserListQuery>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    // Check read permission
    if !components
        .user_manager
        .check_permission(&current_user.role, "users", "read")
        .await
        .unwrap_or(false)
    {
        return Err((
            StatusCode::FORBIDDEN,
            Json(json!({
                "error": {
                    "code": "INSUFFICIENT_PERMISSIONS",
                    "message": "You don't have permission to list users"
                }
            })),
        ));
    }

    match components.user_manager.list_users(query).await {
        Ok(response) => Ok(Json(json!({
            "success": true,
            "data": response
        }))),
        Err(e) => Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({
                "error": {
                    "code": "USER_LIST_FAILED",
                    "message": format!("Failed to list users: {}", e)
                }
            })),
        )),
    }
}

/// Get user by ID (admin only)
pub async fn get_user(
    State(components): State<AppComponents>,
    Extension(current_user): Extension<User>,
    Path(user_id): Path<Uuid>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    // Check read permission
    if !components
        .user_manager
        .check_permission(&current_user.role, "users", "read")
        .await
        .unwrap_or(false)
    {
        return Err((
            StatusCode::FORBIDDEN,
            Json(json!({
                "error": {
                    "code": "INSUFFICIENT_PERMISSIONS",
                    "message": "You don't have permission to view users"
                }
            })),
        ));
    }

    match components.user_manager.get_user_by_id(user_id).await {
        Ok(user) => {
            let user_profile: UserSafeProfile = user.into();
            Ok(Json(json!({
                "success": true,
                "data": user_profile
            })))
        }
        Err(e) => Err((
            StatusCode::NOT_FOUND,
            Json(json!({
                "error": {
                    "code": "USER_NOT_FOUND",
                    "message": format!("User not found: {}", e)
                }
            })),
        )),
    }
}

/// Update user (admin only)
pub async fn update_user(
    State(components): State<AppComponents>,
    Extension(current_user): Extension<User>,
    Path(user_id): Path<Uuid>,
    Json(request): Json<UpdateUserRequest>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    // Check update permission
    if !components
        .user_manager
        .check_permission(&current_user.role, "users", "update")
        .await
        .unwrap_or(false)
    {
        return Err((
            StatusCode::FORBIDDEN,
            Json(json!({
                "error": {
                    "code": "INSUFFICIENT_PERMISSIONS",
                    "message": "You don't have permission to update users"
                }
            })),
        ));
    }

    // Validate request
    if let Err(validation_errors) = request.validate() {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(json!({
                "error": {
                    "code": "VALIDATION_FAILED",
                    "message": "Request validation failed",
                    "details": validation_errors
                }
            })),
        ));
    }

    match components.user_manager.update_user(user_id, request).await {
        Ok(user) => {
            let user_profile: UserSafeProfile = user.into();
            Ok(Json(json!({
                "success": true,
                "data": user_profile
            })))
        }
        Err(e) => Err((
            StatusCode::BAD_REQUEST,
            Json(json!({
                "error": {
                    "code": "USER_UPDATE_FAILED",
                    "message": format!("Failed to update user: {}", e)
                }
            })),
        )),
    }
}

/// Delete user (admin only)
pub async fn delete_user(
    State(components): State<AppComponents>,
    Extension(current_user): Extension<User>,
    Path(user_id): Path<Uuid>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    // Check delete permission
    if !components
        .user_manager
        .check_permission(&current_user.role, "users", "delete")
        .await
        .unwrap_or(false)
    {
        return Err((
            StatusCode::FORBIDDEN,
            Json(json!({
                "error": {
                    "code": "INSUFFICIENT_PERMISSIONS",
                    "message": "You don't have permission to delete users"
                }
            })),
        ));
    }

    // Prevent self-deletion
    if user_id == current_user.id {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(json!({
                "error": {
                    "code": "SELF_DELETE_FORBIDDEN",
                    "message": "You cannot delete your own account"
                }
            })),
        ));
    }

    match components.user_manager.delete_user(user_id).await {
        Ok(rows_affected) => {
            if rows_affected > 0 {
                Ok(Json(json!({
                    "success": true,
                    "message": "User deleted successfully"
                })))
            } else {
                Err((
                    StatusCode::NOT_FOUND,
                    Json(json!({
                        "error": {
                            "code": "USER_NOT_FOUND",
                            "message": "User not found"
                        }
                    })),
                ))
            }
        }
        Err(e) => Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({
                "error": {
                    "code": "USER_DELETE_FAILED",
                    "message": format!("Failed to delete user: {}", e)
                }
            })),
        )),
    }
}
