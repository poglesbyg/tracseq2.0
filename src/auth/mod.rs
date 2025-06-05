mod saml;

use crate::auth::saml::{SamlConfig, SamlService, SamlUserInfo};
use axum::{
    extract::State,
    http::{Request, Response},
    middleware::Next,
    response::IntoResponse,
    Json,
};
use chrono::{DateTime, Utc};
use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, Validation};
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use std::sync::Arc;
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize)]
pub struct User {
    pub id: Uuid,
    pub unc_pid: String,
    pub email: String,
    pub display_name: String,
    pub role: String,
    pub status: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    pub sub: String,  // User ID
    pub exp: usize,   // Expiration time
    pub iat: usize,   // Issued at
    pub role: String, // User role
}

#[derive(Debug, Serialize, Deserialize)]
pub struct LoginResponse {
    pub token: String,
    pub user: User,
}

pub struct AuthService {
    db: PgPool,
    jwt_secret: String,
    saml_service: Arc<SamlService>,
}

impl AuthService {
    pub async fn new(
        db: PgPool,
        jwt_secret: String,
        saml_config: SamlConfig,
    ) -> Result<Self, AuthError> {
        let saml_service = SamlService::new(saml_config)
            .await
            .map_err(|e| AuthError::SamlError(e.to_string()))?;

        Ok(Self {
            db,
            jwt_secret,
            saml_service: Arc::new(saml_service),
        })
    }

    pub async fn initiate_sso(&self) -> Result<String, AuthError> {
        self.saml_service
            .create_auth_request()
            .await
            .map_err(|e| AuthError::SamlError(e.to_string()))
    }

    pub async fn handle_sso_callback(
        &self,
        saml_response: &str,
    ) -> Result<LoginResponse, AuthError> {
        // Parse SAML response and extract user information
        let user_info = self
            .saml_service
            .process_response(saml_response)
            .await
            .map_err(|e| AuthError::SamlError(e.to_string()))?;

        // Get or create user in database
        let user = self.get_or_create_user(&user_info).await?;

        // Generate JWT token
        let token = self.generate_token(&user)?;

        Ok(LoginResponse { token, user })
    }

    async fn get_or_create_user(&self, user_info: &SamlUserInfo) -> Result<User, AuthError> {
        let user = sqlx::query_as!(
            User,
            r#"
            INSERT INTO users (
                unc_pid, email, eppn, given_name, family_name, 
                display_name, affiliation, department, title,
                role, status
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11)
            ON CONFLICT (unc_pid) DO UPDATE SET
                email = EXCLUDED.email,
                eppn = EXCLUDED.eppn,
                given_name = EXCLUDED.given_name,
                family_name = EXCLUDED.family_name,
                display_name = EXCLUDED.display_name,
                affiliation = EXCLUDED.affiliation,
                department = EXCLUDED.department,
                title = EXCLUDED.title,
                updated_at = NOW()
            RETURNING id, unc_pid, email, display_name, role, status
            "#,
            user_info.unc_pid,
            user_info.email,
            user_info.eppn,
            user_info.given_name,
            user_info.family_name,
            user_info.display_name,
            user_info.affiliation,
            user_info.department,
            user_info.title,
            "viewer", // Default role
            "active"  // Default status
        )
        .fetch_one(&self.db)
        .await?;

        Ok(user)
    }

    fn generate_token(&self, user: &User) -> Result<String, AuthError> {
        let now = chrono::Utc::now();
        let exp = (now + chrono::Duration::hours(24)).timestamp() as usize;
        let iat = now.timestamp() as usize;

        let claims = Claims {
            sub: user.id.to_string(),
            exp,
            iat,
            role: user.role.clone(),
        };

        encode(
            &Header::default(),
            &claims,
            &EncodingKey::from_secret(self.jwt_secret.as_bytes()),
        )
        .map_err(AuthError::JwtError)
    }
}

#[derive(Debug, thiserror::Error)]
pub enum AuthError {
    #[error("Database error: {0}")]
    Database(#[from] sqlx::Error),

    #[error("JWT error: {0}")]
    JwtError(#[from] jsonwebtoken::errors::Error),

    #[error("SAML error: {0}")]
    SamlError(String),

    #[error("Invalid credentials")]
    InvalidCredentials,
}

// Middleware to verify JWT token
pub async fn auth_middleware<B>(
    State(auth_service): State<Arc<AuthService>>,
    request: Request<B>,
    next: Next<B>,
) -> Result<Response, AuthError> {
    let token = request
        .headers()
        .get("Authorization")
        .and_then(|h| h.to_str().ok())
        .and_then(|s| s.strip_prefix("Bearer "))
        .ok_or(AuthError::InvalidCredentials)?;

    let claims = decode::<Claims>(
        token,
        &DecodingKey::from_secret(auth_service.jwt_secret.as_bytes()),
        &Validation::default(),
    )
    .map_err(|_| AuthError::InvalidCredentials)?
    .claims;

    // Add user info to request extensions
    let mut request = request;
    request.extensions_mut().insert(claims);

    Ok(next.run(request).await)
}
