use axum::{
    extract::Request,
    http::{HeaderValue, StatusCode},
    middleware::Next,
    response::Response,
};

pub async fn auth_middleware(request: Request, next: Next) -> Result<Response, StatusCode> {
    // TODO: Implement proper authentication logic
    // For now, just pass through all requests
    let response = next.run(request).await;
    Ok(response)
}