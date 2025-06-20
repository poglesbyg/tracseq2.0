use axum::{
    extract::{Request, State},
    http::{header::AUTHORIZATION, StatusCode},
    middleware::Next,
    response::Response,
    Json,
};
use serde_json::json;

use crate::{
    error::{SampleResult, SampleServiceError},
    AppState,
};

/// Authentication middleware that validates JWT tokens and injects user context
pub async fn auth_middleware(
    State(state): State<AppState>,
    mut request: Request,
    next: Next,
) -> Result<Response, SampleServiceError> {
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
            return Err(SampleServiceError::Authentication(
                "Authorization header with Bearer token is required".to_string(),
            ));
        }
    };

    // Validate token with auth service
    match state.auth_client.validate_token(token).await {
        Ok(true) => {
            // Get user information and inject into request
            if let Ok(Some(user_data)) = state.auth_client.get_user_from_token(token).await {
                // Store user context in request extensions for handlers to use
                request.extensions_mut().insert(UserContext {
                    user_id: user_data["user_id"].as_str().unwrap_or("").to_string(),
                    email: user_data["email"].as_str().unwrap_or("").to_string(),
                    role: user_data["role"].as_str().unwrap_or("guest").to_string(),
                    token: token.to_string(),
                });
            }

            Ok(next.run(request).await)
        }
        Ok(false) => {
            Err(SampleServiceError::Authentication(
                "Invalid or expired token".to_string(),
            ))
        }
        Err(e) => {
            Err(SampleServiceError::Authentication(format!(
                "Token validation failed: {}",
                e
            )))
        }
    }
}

/// Optional authentication middleware that doesn't require authentication but injects user if present
pub async fn optional_auth_middleware(
    State(state): State<AppState>,
    mut request: Request,
    next: Next,
) -> Result<Response, SampleServiceError> {
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
        // Try to validate token, but don't fail if it's invalid
        if let Ok(true) = state.auth_client.validate_token(auth_header).await {
            if let Ok(Some(user_data)) = state.auth_client.get_user_from_token(auth_header).await {
                request.extensions_mut().insert(UserContext {
                    user_id: user_data["user_id"].as_str().unwrap_or("").to_string(),
                    email: user_data["email"].as_str().unwrap_or("").to_string(),
                    role: user_data["role"].as_str().unwrap_or("guest").to_string(),
                    token: auth_header.to_string(),
                });
            }
        }
    }

    Ok(next.run(request).await)
}

/// Role-based authorization middleware
pub async fn require_role_middleware(
    required_role: &'static str,
) -> impl Fn(State<AppState>, Request, Next) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<Response, SampleServiceError>> + Send>> {
    move |State(state): State<AppState>, mut request: Request, next: Next| {
        Box::pin(async move {
            let headers = request.headers();

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
                    return Err(SampleServiceError::Authentication(
                        "Authorization header with Bearer token is required".to_string(),
                    ));
                }
            };

            // Validate token and check permissions
            match state.auth_client.validate_permissions(token, required_role).await {
                Ok(true) => {
                    // Inject user context
                    if let Ok(Some(user_data)) = state.auth_client.get_user_from_token(token).await {
                        request.extensions_mut().insert(UserContext {
                            user_id: user_data["user_id"].as_str().unwrap_or("").to_string(),
                            email: user_data["email"].as_str().unwrap_or("").to_string(),
                            role: user_data["role"].as_str().unwrap_or("guest").to_string(),
                            token: token.to_string(),
                        });
                    }

                    Ok(next.run(request).await)
                }
                Ok(false) => {
                    Err(SampleServiceError::Authorization(format!(
                        "Insufficient permissions. Required role: {}",
                        required_role
                    )))
                }
                Err(e) => {
                    Err(SampleServiceError::Authentication(format!(
                        "Permission validation failed: {}",
                        e
                    )))
                }
            }
        })
    }
}

/// Extract user context from request
pub fn extract_user_context(request: &Request) -> Option<&UserContext> {
    request.extensions().get::<UserContext>()
}

/// Helper function to create role-based middleware
pub fn require_role(role: &'static str) -> axum::middleware::FromFn<AppState, impl Fn(State<AppState>, Request, Next) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<Response, SampleServiceError>> + Send>>> {
    axum::middleware::from_fn_with_state(AppState::default(), require_role_middleware(role))
}

/// User context structure for request extensions
#[derive(Debug, Clone)]
pub struct UserContext {
    pub user_id: String,
    pub email: String,
    pub role: String,
    pub token: String,
}

impl UserContext {
    /// Check if user has a specific role or higher
    pub fn has_role(&self, required_role: &str) -> bool {
        let role_hierarchy = [
            "guest",
            "data_analyst",
            "research_scientist",
            "lab_technician",
            "principal_investigator",
            "lab_administrator",
        ];

        let user_level = role_hierarchy
            .iter()
            .position(|r| r == &self.role)
            .unwrap_or(0);
        let required_level = role_hierarchy
            .iter()
            .position(|r| r == &required_role)
            .unwrap_or(usize::MAX);

        user_level >= required_level
    }

    /// Check if user is admin
    pub fn is_admin(&self) -> bool {
        self.role == "lab_administrator"
    }

    /// Check if user can modify samples
    pub fn can_modify_samples(&self) -> bool {
        self.has_role("lab_technician")
    }

    /// Check if user can delete samples
    pub fn can_delete_samples(&self) -> bool {
        self.has_role("principal_investigator")
    }

    /// Check if user can manage sample workflows
    pub fn can_manage_workflows(&self) -> bool {
        self.has_role("lab_technician")
    }
} 
