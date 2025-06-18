use axum::{
    extract::{Request, State},
    http::{header::AUTHORIZATION, StatusCode},
    middleware::Next,
    response::Response,
};

use crate::{
    AppState,
    error::AuthError,
    models::*,
};

/// Authentication middleware that verifies JWT tokens and injects user info
pub async fn auth_middleware(
    State(state): State<AppState>,
    mut request: Request,
    next: Next,
) -> Result<Response, AuthError> {
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
            return Err(AuthError::authentication("Authorization header with Bearer token is required"));
        }
    };

    // Validate token
    let token_response = state.auth_service.validate_token(token).await?;

    if !token_response.valid {
        return Err(AuthError::TokenInvalid);
    }

    // Get full user information
    let user_id = token_response.user_id.ok_or(AuthError::TokenInvalid)?;
    let user = state.auth_service.get_user_by_id(user_id).await?;

    // Check if user is still active
    if !user.can_login() {
        return Err(AuthError::AccountDisabled);
    }

    // Inject user into request extensions
    request.extensions_mut().insert(user);
    
    // Also inject session ID if needed
    if let Some(session_id) = token_response.session_id {
        request.extensions_mut().insert(session_id);
    }

    Ok(next.run(request).await)
}

/// Admin middleware that requires admin privileges
pub async fn admin_middleware(
    State(state): State<AppState>,
    mut request: Request,
    next: Next,
) -> Result<Response, AuthError> {
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
            return Err(AuthError::authentication("Authorization header with Bearer token is required"));
        }
    };

    // Validate token
    let token_response = state.auth_service.validate_token(token).await?;

    if !token_response.valid {
        return Err(AuthError::TokenInvalid);
    }

    // Get full user information
    let user_id = token_response.user_id.ok_or(AuthError::TokenInvalid)?;
    let user = state.auth_service.get_user_by_id(user_id).await?;

    // Check if user is still active
    if !user.can_login() {
        return Err(AuthError::AccountDisabled);
    }

    // Check if user is admin
    if !user.is_admin() {
        return Err(AuthError::authorization("Administrator privileges required"));
    }

    // Inject user into request extensions
    request.extensions_mut().insert(user);
    
    // Also inject session ID if needed
    if let Some(session_id) = token_response.session_id {
        request.extensions_mut().insert(session_id);
    }

    Ok(next.run(request).await)
}

/// Service authentication middleware for inter-service communication
pub async fn service_auth_middleware(
    State(state): State<AppState>,
    mut request: Request,
    next: Next,
) -> Result<Response, AuthError> {
    let headers = request.headers();

    // Check for service authentication
    // This could be API key based or mutual TLS
    let auth_header = headers
        .get(AUTHORIZATION)
        .and_then(|header| header.to_str().ok());

    match auth_header {
        Some(header) if header.starts_with("Bearer ") => {
            // JWT token authentication
            let token = &header[7..];
            let token_response = state.auth_service.validate_token(token).await?;
            
            if !token_response.valid {
                return Err(AuthError::TokenInvalid);
            }

            // For service auth, we don't need to inject user, just validate the token
            Ok(next.run(request).await)
        }
        Some(header) if header.starts_with("Service ") => {
            // Service key authentication (simplified)
            let service_key = &header[8..];
            
            // In a real implementation, you'd validate the service key against a database
            // For now, we'll accept any service key that starts with "service_"
            if service_key.starts_with("service_") {
                Ok(next.run(request).await)
            } else {
                Err(AuthError::authentication("Invalid service key"))
            }
        }
        _ => {
            Err(AuthError::authentication("Service authentication required"))
        }
    }
}

/// Optional authentication middleware that doesn't require authentication but injects user if present
pub async fn optional_auth_middleware(
    State(state): State<AppState>,
    mut request: Request,
    next: Next,
) -> Response {
    let headers = request.headers();

    // Try to extract authorization header
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
        // Try to validate token and inject user if valid
        if let Ok(token_response) = state.auth_service.validate_token(auth_header).await {
            if token_response.valid {
                if let Some(user_id) = token_response.user_id {
                    if let Ok(user) = state.auth_service.get_user_by_id(user_id).await {
                        if user.can_login() {
                            request.extensions_mut().insert(user);
                            
                            if let Some(session_id) = token_response.session_id {
                                request.extensions_mut().insert(session_id);
                            }
                        }
                    }
                }
            }
        }
    }

    next.run(request).await
} 
