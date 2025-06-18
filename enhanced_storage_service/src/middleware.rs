// Authentication and Authorization Middleware
use axum::{extract::State, http::Request, middleware::Next, response::Response};
use crate::AppState;

pub async fn auth_middleware<B>(
    State(_state): State<AppState>,
    request: Request<B>,
    next: Next<B>,
) -> Response {
    // Stub implementation - in real app would validate JWT tokens
    next.run(request).await
}

pub async fn admin_middleware<B>(
    State(_state): State<AppState>,
    request: Request<B>,
    next: Next<B>,
) -> Response {
    // Stub implementation - in real app would check admin privileges  
    next.run(request).await
}
