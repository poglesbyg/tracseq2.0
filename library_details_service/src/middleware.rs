use axum::{
    http::{Request, StatusCode},
    middleware::Next,
    response::Response,
};
use jsonwebtoken::{decode, Algorithm, DecodingKey, Validation};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    pub sub: String,
    pub exp: usize,
    pub iat: usize,
    pub roles: Vec<String>,
}

pub async fn auth_middleware<B>(
    request: Request<B>,
    next: Next<B>,
) -> Result<Response, StatusCode> {
    let headers = request.headers();
    // Extract the Authorization header
    let auth_header = headers
        .get("Authorization")
        .and_then(|header| header.to_str().ok());

    if let Some(auth_header) = auth_header {
        if auth_header.starts_with("Bearer ") {
            let token = &auth_header[7..];
            
            // For now, use a simple validation - in production, this should use proper JWT validation
            if validate_token(token).is_ok() {
                return Ok(next.run(request).await);
            }
        }
    }

    // Check for API key authentication
    if let Some(api_key) = headers.get("X-API-Key").and_then(|h| h.to_str().ok()) {
        if validate_api_key(api_key) {
            return Ok(next.run(request).await);
        }
    }

    // For development, allow requests without authentication
    if std::env::var("SKIP_AUTH").unwrap_or_default() == "true" {
        return Ok(next.run(request).await);
    }

    Err(StatusCode::UNAUTHORIZED)
}

fn validate_token(token: &str) -> Result<Claims, jsonwebtoken::errors::Error> {
    let secret = std::env::var("AUTH_SECRET").unwrap_or_else(|_| "default-secret".to_string());
    let key = DecodingKey::from_secret(secret.as_ref());
    
    let validation = Validation::new(Algorithm::HS256);
    
    let token_data = decode::<Claims>(token, &key, &validation)?;
    Ok(token_data.claims)
}

fn validate_api_key(api_key: &str) -> bool {
    // Simple API key validation - in production, this should check against a database
    let valid_keys = vec![
        "dev-key-123",
        "test-key-456",
    ];
    
    valid_keys.contains(&api_key)
}