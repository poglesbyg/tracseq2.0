use axum::{
    extract::{Request, State},
    http::{header::AUTHORIZATION, StatusCode},
    response::Json,
};
use serde_json::{json, Value};

use crate::{
    models::user::{User, UserSession},
    AppComponents,
};

/// Extract and verify JWT token from request headers
pub async fn verify_auth_token(
    components: &AppComponents,
    headers: &axum::http::HeaderMap,
) -> Result<(User, UserSession), (StatusCode, Json<Value>)> {
    // Extract authorization header
    let auth_header = headers
        .get(AUTHORIZATION)
        .and_then(|header| header.to_str().ok())
        .and_then(|header| {
            if header.starts_with("Bearer ") {
                Some(&header[7..])
            } else {
                None
            }
        });

    let token = match auth_header {
        Some(token) => token,
        None => {
            return Err((
                StatusCode::UNAUTHORIZED,
                Json(json!({
                    "error": {
                        "code": "MISSING_TOKEN",
                        "message": "Authorization header with Bearer token is required"
                    }
                })),
            ));
        }
    };

    // Verify token and get user
    match components.auth_service.verify_token(token).await {
        Ok((user, session)) => Ok((user, session)),
        Err(e) => Err((
            StatusCode::UNAUTHORIZED,
            Json(json!({
                "error": {
                    "code": "INVALID_TOKEN",
                    "message": format!("Invalid or expired token: {}", e)
                }
            })),
        )),
    }
}

/// Helper function to require authentication in handlers
pub async fn require_auth(
    components: &AppComponents,
    headers: &axum::http::HeaderMap,
) -> Result<User, (StatusCode, Json<Value>)> {
    let (user, _session) = verify_auth_token(components, headers).await?;

    if !user.can_login() {
        return Err((
            StatusCode::UNAUTHORIZED,
            Json(json!({
                "error": {
                    "code": "ACCOUNT_INACTIVE",
                    "message": "User account is inactive or locked"
                }
            })),
        ));
    }

    Ok(user)
}

/// Helper function to require admin privileges
pub async fn require_admin(
    components: &AppComponents,
    headers: &axum::http::HeaderMap,
) -> Result<User, (StatusCode, Json<Value>)> {
    let user = require_auth(components, headers).await?;

    if !components
        .user_manager
        .check_permission(&user.role, "users", "manage")
        .await
        .unwrap_or(false)
    {
        return Err((
            StatusCode::FORBIDDEN,
            Json(json!({
                "error": {
                    "code": "INSUFFICIENT_PERMISSIONS",
                    "message": "Administrator privileges required"
                }
            })),
        ));
    }

    Ok(user)
}
