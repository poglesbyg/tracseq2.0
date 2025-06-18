use axum::{extract::State, Json};
use serde_json::json;
use validator::Validate;

use crate::{
    AppState,
    error::AuthError,
    models::*,
};

/// Validate JWT token endpoint (for other services)
pub async fn validate_token(
    State(state): State<AppState>,
    Json(request): Json<ValidateTokenRequest>,
) -> Result<Json<serde_json::Value>, AuthError> {
    // Validate request
    request.validate()?;

    // Validate the token
    match state.auth_service.validate_token(&request.token).await {
        Ok(response) => {
            Ok(Json(json!({
                "success": true,
                "data": response
            })))
        }
        Err(AuthError::TokenInvalid | AuthError::TokenExpired | AuthError::SessionNotFound) => {
            // Return invalid response instead of error for validation endpoints
            Ok(Json(json!({
                "success": true,
                "data": {
                    "valid": false,
                    "user_id": null,
                    "email": null,
                    "role": null,
                    "session_id": null,
                    "expires_at": null
                }
            })))
        }
        Err(e) => Err(e),
    }
}

/// Validate permissions endpoint (for other services)
pub async fn validate_permissions(
    State(state): State<AppState>,
    Json(request): Json<ValidatePermissionsRequest>,
) -> Result<Json<serde_json::Value>, AuthError> {
    // Validate request
    request.validate()?;

    // First validate the token
    let token_response = match state.auth_service.validate_token(&request.token).await {
        Ok(response) => response,
        Err(AuthError::TokenInvalid | AuthError::TokenExpired | AuthError::SessionNotFound) => {
            // Return unauthorized response
            return Ok(Json(json!({
                "success": true,
                "data": {
                    "authorized": false,
                    "user": null,
                    "reason": "Invalid or expired token"
                }
            })));
        }
        Err(e) => return Err(e),
    };

    // Check if token is valid
    if !token_response.valid {
        return Ok(Json(json!({
            "success": true,
            "data": {
                "authorized": false,
                "user": null,
                "reason": "Invalid token"
            }
        })));
    }

    // Get user information
    let user_role = token_response.role.as_ref().ok_or(AuthError::TokenInvalid)?;

    // Check role permissions
    let authorized = has_role_or_higher(user_role, &request.required_role);

    let user_info = if authorized {
        Some(json!({
            "id": token_response.user_id,
            "email": token_response.email,
            "role": token_response.role
        }))
    } else {
        None
    };

    let reason = if !authorized {
        Some(format!("Insufficient permissions. Required: {:?}, User has: {:?}", request.required_role, user_role))
    } else {
        None
    };

    Ok(Json(json!({
        "success": true,
        "data": {
            "authorized": authorized,
            "user": user_info,
            "reason": reason
        }
    })))
}

/// Extract user from token endpoint (for middleware)
pub async fn extract_user(
    State(state): State<AppState>,
    Json(request): Json<ValidateTokenRequest>,
) -> Result<Json<serde_json::Value>, AuthError> {
    // Validate request
    request.validate()?;

    // Validate the token and get full user info
    let token_response = state.auth_service.validate_token(&request.token).await?;

    if !token_response.valid {
        return Err(AuthError::TokenInvalid);
    }

    // Get full user information
    let user_id = token_response.user_id.ok_or(AuthError::TokenInvalid)?;
    let user = state.auth_service.get_user_by_id(user_id).await?;

    Ok(Json(json!({
        "success": true,
        "data": {
            "user": {
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
            },
            "session_id": token_response.session_id
        }
    })))
}

// Helper function to check role hierarchy
fn has_role_or_higher(user_role: &UserRole, required_role: &UserRole) -> bool {
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

// Request models

#[derive(Debug, serde::Deserialize, validator::Validate)]
pub struct ValidateTokenRequest {
    #[validate(length(min = 1, message = "Token is required"))]
    pub token: String,
}

#[derive(Debug, serde::Deserialize, validator::Validate)]
pub struct ValidatePermissionsRequest {
    #[validate(length(min = 1, message = "Token is required"))]
    pub token: String,
    pub required_role: UserRole,
    pub resource: Option<String>,
} 
