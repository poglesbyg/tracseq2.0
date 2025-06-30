// Authentication and Authorization Middleware
use axum::{extract::State, http::Request, middleware::Next, response::Response, body::Body};
use crate::AppState;

pub async fn auth_middleware(
    State(_state): State<AppState>,
    request: Request<Body>,
    next: Next,
) -> Response {
    // Stub implementation - in real app would validate JWT tokens
    next.run(request).await
}

pub async fn admin_middleware(
    State(_state): State<AppState>,
    request: Request<Body>,
    next: Next,
) -> Response {
    // Stub implementation - in real app would check admin privileges  
    next.run(request).await
}
