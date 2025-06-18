use anyhow::Result;
use argon2::{
    password_hash::{rand_core::OsRng, SaltString},
    Argon2, PasswordHasher, PasswordVerifier,
};
use chrono::{DateTime, Duration, Utc};
use jsonwebtoken::{decode, encode, Algorithm, DecodingKey, EncodingKey, Header, Validation};
use sha2::{Digest, Sha256};
use sqlx::PgPool;
use std::collections::HashMap;
use tracing::{error, info, warn};
use uuid::Uuid;

use crate::{
    config::Config,
    database::DatabasePool,
    error::{AuthError, AuthResult},
    models::*,
};

/// Core authentication service implementation
#[derive(Debug, Clone)]
pub struct AuthServiceImpl {
    db: DatabasePool,
    config: Config,
    encoding_key: EncodingKey,
    decoding_key: DecodingKey,
}

impl AuthServiceImpl {
    /// Create a new authentication service
    pub fn new(db: DatabasePool, config: Config) -> Result<Self> {
        let encoding_key = EncodingKey::from_secret(config.jwt.secret.as_bytes());
        let decoding_key = DecodingKey::from_secret(config.jwt.secret.as_bytes());

        Ok(Self {
            db,
            config,
            encoding_key,
            decoding_key,
        })
    }

    /// Authenticate user with email and password
    pub async fn login(&self, request: LoginRequest) -> AuthResult<LoginResponse> {
        // Get user by email
        let user = self.get_user_by_email(&request.email).await?;

        // Check if user can login
        if !user.can_login() {
            if user.is_locked() {
                return Err(AuthError::account_locked("Account is temporarily locked due to too many failed login attempts"));
            }
            if !user.email_verified {
                return Err(AuthError::AccountNotVerified);
            }
            if user.status != UserStatus::Active {
                return Err(AuthError::AccountDisabled);
            }
        }

        // Verify password
        if !self.verify_password(&request.password, &user.password_hash)? {
            // Increment failed login attempts
            self.increment_failed_login_attempts(user.id).await?;
            return Err(AuthError::InvalidCredentials);
        }

        // Reset failed login attempts on successful login
        self.reset_failed_login_attempts(user.id).await?;

        // Update last login timestamp
        self.update_last_login(user.id).await?;

        // Create session and tokens
        let session_id = Uuid::new_v4();
        let expires_at = Utc::now() + Duration::hours(self.config.jwt.access_token_expiry_hours);

        // Create JWT claims
        let claims = AuthClaims {
            sub: user.id,
            email: user.email.clone(),
            role: user.role.clone(),
            exp: expires_at.timestamp(),
            iat: Utc::now().timestamp(),
            iss: self.config.jwt.issuer.clone(),
            aud: self.config.jwt.audience.clone(),
            jti: session_id,
        };

        // Generate tokens
        let access_token = self.generate_jwt_token(&claims)?;
        let refresh_token = if request.remember_me.unwrap_or(false) {
            Some(self.generate_refresh_token())
        } else {
            None
        };

        // Store session in database
        self.create_session(session_id, user.id, &access_token, refresh_token.as_deref(), expires_at).await?;

        // Log successful authentication
        self.log_security_event("LOGIN_SUCCESS", Some(user.id), None, None).await?;

        Ok(LoginResponse {
            user_id: user.id,
            email: user.email,
            role: user.role,
            access_token,
            refresh_token,
            expires_at,
            session_id,
        })
    }

    /// Validate a JWT token
    pub async fn validate_token(&self, token: &str) -> AuthResult<ValidateTokenResponse> {
        // Decode and validate JWT
        let claims = self.decode_jwt_token(token)?;

        // Check if session exists and is valid
        let session = self.get_session(claims.jti).await?;
        if session.user_id != claims.sub {
            return Err(AuthError::TokenInvalid);
        }

        // Check if token hash matches (for additional security)
        let token_hash = self.hash_token(token);
        if session.token_hash != token_hash {
            return Err(AuthError::TokenInvalid);
        }

        // Update session last used timestamp
        self.update_session_last_used(claims.jti).await?;

        Ok(ValidateTokenResponse {
            valid: true,
            user_id: Some(claims.sub),
            email: Some(claims.email),
            role: Some(claims.role),
            session_id: Some(claims.jti),
            expires_at: Some(DateTime::from_timestamp(claims.exp, 0).unwrap_or_default()),
        })
    }

    /// Generate JWT token
    fn generate_jwt_token(&self, claims: &AuthClaims) -> AuthResult<String> {
        let header = Header::new(Algorithm::HS256);
        encode(&header, claims, &self.encoding_key)
            .map_err(|e| AuthError::Jwt(e))
    }

    /// Decode JWT token
    fn decode_jwt_token(&self, token: &str) -> AuthResult<AuthClaims> {
        let mut validation = Validation::new(Algorithm::HS256);
        validation.set_issuer(&[&self.config.jwt.issuer]);
        validation.set_audience(&[&self.config.jwt.audience]);

        let token_data = decode::<AuthClaims>(token, &self.decoding_key, &validation)
            .map_err(|e| AuthError::Jwt(e))?;

        Ok(token_data.claims)
    }

    /// Hash password using Argon2
    fn hash_password(&self, password: &str) -> AuthResult<String> {
        let salt = SaltString::generate(&mut OsRng);
        let argon2 = Argon2::default();
        
        argon2
            .hash_password(password.as_bytes(), &salt)
            .map(|hash| hash.to_string())
            .map_err(|e| AuthError::PasswordHash(e))
    }

    /// Verify password using Argon2
    fn verify_password(&self, password: &str, hash: &str) -> AuthResult<bool> {
        let parsed_hash = argon2::PasswordHash::new(hash)
            .map_err(|e| AuthError::PasswordHash(e))?;
        
        let argon2 = Argon2::default();
        Ok(argon2.verify_password(password.as_bytes(), &parsed_hash).is_ok())
    }

    /// Hash token for storage
    fn hash_token(&self, token: &str) -> String {
        let mut hasher = Sha256::new();
        hasher.update(token);
        format!("{:x}", hasher.finalize())
    }

    /// Generate refresh token
    fn generate_refresh_token(&self) -> String {
        Uuid::new_v4().to_string()
    }

    /// Get user by email
    pub async fn get_user_by_email(&self, email: &str) -> AuthResult<User> {
        let user = sqlx::query_as::<_, User>("SELECT * FROM users WHERE email = $1")
            .bind(email)
            .fetch_optional(&self.db.pool)
            .await?
            .ok_or(AuthError::UserNotFound)?;

        Ok(user)
    }

    /// Get session by ID
    async fn get_session(&self, session_id: Uuid) -> AuthResult<UserSession> {
        let session = sqlx::query_as::<_, UserSession>(
            "SELECT * FROM user_sessions WHERE id = $1 AND expires_at > NOW() AND revoked = FALSE"
        )
        .bind(session_id)
        .fetch_optional(&self.db.pool)
        .await?
        .ok_or(AuthError::SessionNotFound)?;

        Ok(session)
    }

    /// Create user session
    async fn create_session(
        &self,
        session_id: Uuid,
        user_id: Uuid,
        token: &str,
        refresh_token: Option<&str>,
        expires_at: DateTime<Utc>,
    ) -> AuthResult<()> {
        let token_hash = self.hash_token(token);
        let refresh_token_hash = refresh_token.map(|t| self.hash_token(t));

        sqlx::query(
            r#"
            INSERT INTO user_sessions (
                id, user_id, token_hash, refresh_token_hash, expires_at, created_at, last_used_at
            ) VALUES ($1, $2, $3, $4, $5, NOW(), NOW())
            "#,
        )
        .bind(session_id)
        .bind(user_id)
        .bind(token_hash)
        .bind(refresh_token_hash)
        .bind(expires_at)
        .execute(&self.db.pool)
        .await?;

        Ok(())
    }

    async fn update_session_last_used(&self, session_id: Uuid) -> AuthResult<()> {
        sqlx::query("UPDATE user_sessions SET last_used_at = NOW() WHERE id = $1")
            .bind(session_id)
            .execute(&self.db.pool)
            .await?;
        Ok(())
    }

    async fn increment_failed_login_attempts(&self, user_id: Uuid) -> AuthResult<()> {
        sqlx::query(
            "UPDATE users SET failed_login_attempts = failed_login_attempts + 1 WHERE id = $1"
        )
        .bind(user_id)
        .execute(&self.db.pool)
        .await?;
        Ok(())
    }

    async fn reset_failed_login_attempts(&self, user_id: Uuid) -> AuthResult<()> {
        sqlx::query(
            "UPDATE users SET failed_login_attempts = 0, locked_until = NULL WHERE id = $1"
        )
        .bind(user_id)
        .execute(&self.db.pool)
        .await?;
        Ok(())
    }

    async fn update_last_login(&self, user_id: Uuid) -> AuthResult<()> {
        sqlx::query("UPDATE users SET last_login_at = NOW() WHERE id = $1")
            .bind(user_id)
            .execute(&self.db.pool)
            .await?;
        Ok(())
    }

    async fn log_security_event(
        &self,
        event_type: &str,
        user_id: Option<Uuid>,
        ip_address: Option<&str>,
        user_agent: Option<&str>,
    ) -> AuthResult<()> {
        let event_id = Uuid::new_v4();

        sqlx::query(
            r#"
            INSERT INTO security_audit_log (
                event_id, event_type, user_id, ip_address, user_agent, severity, timestamp
            ) VALUES ($1, $2, $3, $4, $5, 'MEDIUM', NOW())
            "#,
        )
        .bind(event_id)
        .bind(event_type)
        .bind(user_id)
        .bind(ip_address)
        .bind(user_agent)
        .execute(&self.db.pool)
        .await?;

        Ok(())
    }

    /// Health check
    pub async fn health_check(&self) -> AuthResult<()> {
        self.db.health_check().await?;
        Ok(())
    }
}
