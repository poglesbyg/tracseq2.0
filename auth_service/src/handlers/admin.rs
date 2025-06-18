use axum::{extract::State, Json};
use serde_json::json;

use crate::{
    AppState,
    error::AuthError,
    models::*,
};

/// List all users (admin only)
pub async fn list_users(
    State(state): State<AppState>,
    _admin_user: User, // Injected by admin middleware
) -> Result<Json<serde_json::Value>, AuthError> {
    let users = sqlx::query_as::<_, User>(
        "SELECT * FROM users ORDER BY created_at DESC LIMIT 100"
    )
    .fetch_all(&state.db_pool.pool)
    .await?;

    let user_data: Vec<serde_json::Value> = users
        .into_iter()
        .map(|user| json!({
            "id": user.id,
            "email": user.email,
            "first_name": user.first_name,
            "last_name": user.last_name,
            "role": user.role,
            "status": user.status,
            "email_verified": user.email_verified,
            "failed_login_attempts": user.failed_login_attempts,
            "locked_until": user.locked_until,
            "last_login_at": user.last_login_at,
            "created_at": user.created_at,
            "department": user.department,
            "position": user.position,
            "lab_affiliation": user.lab_affiliation
        }))
        .collect();

    Ok(Json(json!({
        "success": true,
        "data": user_data
    })))
}

/// Get specific user (admin only)
pub async fn get_user(
    State(state): State<AppState>,
    axum::extract::Path(user_id): axum::extract::Path<uuid::Uuid>,
    _admin_user: User, // Injected by admin middleware
) -> Result<Json<serde_json::Value>, AuthError> {
    let user = sqlx::query_as::<_, User>("SELECT * FROM users WHERE id = $1")
        .bind(user_id)
        .fetch_optional(&state.db_pool.pool)
        .await?
        .ok_or(AuthError::UserNotFound)?;

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
            "failed_login_attempts": user.failed_login_attempts,
            "locked_until": user.locked_until,
            "last_login_at": user.last_login_at,
            "created_at": user.created_at,
            "updated_at": user.updated_at,
            "department": user.department,
            "position": user.position,
            "lab_affiliation": user.lab_affiliation
        }
    })))
}

/// Delete user (admin only)
pub async fn delete_user(
    State(state): State<AppState>,
    axum::extract::Path(user_id): axum::extract::Path<uuid::Uuid>,
    _admin_user: User, // Injected by admin middleware
) -> Result<Json<serde_json::Value>, AuthError> {
    let result = sqlx::query("DELETE FROM users WHERE id = $1")
        .bind(user_id)
        .execute(&state.db_pool.pool)
        .await?;

    if result.rows_affected() == 0 {
        return Err(AuthError::UserNotFound);
    }

    Ok(Json(json!({
        "success": true,
        "message": "User deleted successfully"
    })))
}

/// Disable user (admin only)
pub async fn disable_user(
    State(state): State<AppState>,
    axum::extract::Path(user_id): axum::extract::Path<uuid::Uuid>,
    _admin_user: User, // Injected by admin middleware
) -> Result<Json<serde_json::Value>, AuthError> {
    let result = sqlx::query(
        "UPDATE users SET status = 'inactive', updated_at = NOW() WHERE id = $1"
    )
    .bind(user_id)
    .execute(&state.db_pool.pool)
    .await?;

    if result.rows_affected() == 0 {
        return Err(AuthError::UserNotFound);
    }

    // Revoke all sessions for the disabled user
    sqlx::query("UPDATE user_sessions SET revoked = TRUE, revoked_at = NOW() WHERE user_id = $1")
        .bind(user_id)
        .execute(&state.db_pool.pool)
        .await?;

    Ok(Json(json!({
        "success": true,
        "message": "User disabled successfully"
    })))
}

/// Enable user (admin only)
pub async fn enable_user(
    State(state): State<AppState>,
    axum::extract::Path(user_id): axum::extract::Path<uuid::Uuid>,
    _admin_user: User, // Injected by admin middleware
) -> Result<Json<serde_json::Value>, AuthError> {
    let result = sqlx::query(
        "UPDATE users SET status = 'active', updated_at = NOW() WHERE id = $1"
    )
    .bind(user_id)
    .execute(&state.db_pool.pool)
    .await?;

    if result.rows_affected() == 0 {
        return Err(AuthError::UserNotFound);
    }

    Ok(Json(json!({
        "success": true,
        "message": "User enabled successfully"
    })))
}

/// List all sessions (admin only)
pub async fn list_sessions(
    State(state): State<AppState>,
    _admin_user: User, // Injected by admin middleware
) -> Result<Json<serde_json::Value>, AuthError> {
    let sessions = sqlx::query!(
        r#"
        SELECT 
            s.id, s.user_id, s.device_info, s.ip_address, s.user_agent,
            s.expires_at, s.created_at, s.last_used_at, s.revoked,
            u.email, u.first_name, u.last_name
        FROM user_sessions s
        JOIN users u ON s.user_id = u.id
        WHERE s.expires_at > NOW()
        ORDER BY s.last_used_at DESC
        LIMIT 100
        "#
    )
    .fetch_all(&state.db_pool.pool)
    .await?;

    let session_data: Vec<serde_json::Value> = sessions
        .into_iter()
        .map(|session| json!({
            "id": session.id,
            "user_id": session.user_id,
            "user_email": session.email,
            "user_name": format!("{} {}", session.first_name, session.last_name),
            "device_info": session.device_info,
            "ip_address": session.ip_address,
            "created_at": session.created_at,
            "last_used_at": session.last_used_at,
            "expires_at": session.expires_at,
            "revoked": session.revoked
        }))
        .collect();

    Ok(Json(json!({
        "success": true,
        "data": session_data
    })))
}

/// Get audit log (admin only)
pub async fn get_audit_log(
    State(state): State<AppState>,
    _admin_user: User, // Injected by admin middleware
) -> Result<Json<serde_json::Value>, AuthError> {
    let logs = sqlx::query!(
        r#"
        SELECT 
            id, event_id, event_type, user_id, user_email, ip_address,
            user_agent, details, severity, timestamp
        FROM security_audit_log
        ORDER BY timestamp DESC
        LIMIT 100
        "#
    )
    .fetch_all(&state.db_pool.pool)
    .await?;

    let log_data: Vec<serde_json::Value> = logs
        .into_iter()
        .map(|log| json!({
            "id": log.id,
            "event_id": log.event_id,
            "event_type": log.event_type,
            "user_id": log.user_id,
            "user_email": log.user_email,
            "ip_address": log.ip_address,
            "user_agent": log.user_agent,
            "details": log.details,
            "severity": log.severity,
            "timestamp": log.timestamp
        }))
        .collect();

    Ok(Json(json!({
        "success": true,
        "data": log_data
    })))
} 
