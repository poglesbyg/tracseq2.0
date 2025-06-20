use axum::{
    extract::{Request, State},
    http::StatusCode,
    middleware::Next,
    response::Response,
};
use tracing::{warn, info};

use crate::{AppState, error::SequencingError};

/// Authentication middleware
pub async fn auth_middleware(
    State(_state): State<AppState>,
    request: Request,
    next: Next,
) -> Result<Response, SequencingError> {
    // TODO: Implement actual authentication logic
    // For now, just allow all requests
    info!("Processing authenticated request to: {}", request.uri());
    
    let response = next.run(request).await;
    Ok(response)
}

/// Admin-only middleware
pub async fn admin_middleware(
    State(_state): State<AppState>,
    request: Request,
    next: Next,
) -> Result<Response, SequencingError> {
    // TODO: Implement actual admin authorization logic
    // For now, just allow all requests
    info!("Processing admin request to: {}", request.uri());
    
    let response = next.run(request).await;
    Ok(response)
}
