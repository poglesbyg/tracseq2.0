use axum::{
    extract::{Request, State},
    http::StatusCode,
    middleware::Next,
    response::Response,
};

use crate::AppState;

/// Basic authentication middleware
pub async fn auth_middleware(
    State(_state): State<AppState>,
    request: Request,
    next: Next,
) -> Result<Response, StatusCode> {
    // TODO: Implement JWT validation
    // For now, just pass through
    Ok(next.run(request).await)
}

/// Admin authentication middleware
pub async fn admin_middleware(
    State(_state): State<AppState>,
    request: Request,
    next: Next,
) -> Result<Response, StatusCode> {
    // TODO: Implement admin role validation
    // For now, just pass through
    Ok(next.run(request).await)
} 
