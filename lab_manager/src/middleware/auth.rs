use axum::{
    extract::{Request, State},
    http::{header::AUTHORIZATION, StatusCode},
    middleware::Next,
    response::{IntoResponse, Response},
};
use serde_json::json;

use crate::{
    assembly::AppComponents,
    models::user::{User, UserRole},
};

/// Authentication middleware that verifies JWT tokens and injects user info
pub async fn auth_middleware(
    State(components): State<AppComponents>,
    mut request: Request,
    next: Next,
) -> Result<Response, impl IntoResponse> {
    let headers = request.headers();

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
                axum::Json(json!({
                    "error": {
                        "code": "MISSING_TOKEN",
                        "message": "Authorization header with Bearer token is required"
                    }
                })),
            ));
        }
    };

    // Verify token and get user
    let (user, session) = match components.auth_service.verify_token(token).await {
        Ok((user, session)) => (user, session),
        Err(e) => {
            return Err((
                StatusCode::UNAUTHORIZED,
                axum::Json(json!({
                    "error": {
                        "code": "INVALID_TOKEN",
                        "message": format!("Invalid or expired token: {}", e)
                    }
                })),
            ));
        }
    };

    // Add user and session to request extensions
    request.extensions_mut().insert(user);
    request.extensions_mut().insert(session.id);

    Ok(next.run(request).await)
}

/// Optional authentication middleware that injects user if token is present but doesn't require it
pub async fn optional_auth_middleware(
    State(components): State<AppComponents>,
    mut request: Request,
    next: Next,
) -> Response {
    let headers = request.headers();

    // Extract authorization header
    if let Some(auth_header) = headers
        .get(AUTHORIZATION)
        .and_then(|header| header.to_str().ok())
        .and_then(|header| {
            if header.starts_with("Bearer ") {
                Some(&header[7..])
            } else {
                None
            }
        })
    {
        // Try to verify token and inject user if valid
        if let Ok((user, session)) = components.auth_service.verify_token(auth_header).await {
            request.extensions_mut().insert(user);
            request.extensions_mut().insert(session.id);
        }
    }

    next.run(request).await
}

/// Check if a user role has equal or higher permissions than required role
pub fn has_role_or_higher(user_role: &UserRole, required_role: &UserRole) -> bool {
    let role_hierarchy = [
        UserRole::Guest,
        UserRole::DataAnalyst,
        UserRole::ResearchScientist,
        UserRole::LabTechnician,
        UserRole::PrincipalInvestigator,
        UserRole::LabAdministrator,
    ];

    let user_level = role_hierarchy
        .iter()
        .position(|r| r == user_role)
        .unwrap_or(0);
    let required_level = role_hierarchy
        .iter()
        .position(|r| r == required_role)
        .unwrap_or(usize::MAX);

    user_level >= required_level
}

/// Check if current user has admin privileges
pub fn is_admin(user: &User) -> bool {
    matches!(user.role, UserRole::LabAdministrator)
}
