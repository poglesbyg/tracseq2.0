use axum::{
    extract::{Request, State},
    http::StatusCode,
    middleware::Next,
    response::Response,
};
use crate::AppState;

pub async fn auth_middleware(
    State(_state): State<AppState>,
    request: Request,
    next: Next,
) -> Result<Response, StatusCode> {
    // TODO: Implement proper authentication logic
    // For now, just pass through all requests
    let response = next.run(request).await;
    Ok(response)
}

pub async fn admin_middleware(
    State(_state): State<AppState>,
    request: Request,
    next: Next,
) -> Result<Response, StatusCode> {
    // TODO: Implement proper admin authorization logic
    // For now, just pass through all requests
    let response = next.run(request).await;
    Ok(response)
}