use anyhow::Result;
use argon2::{
    password_hash::{rand_core::OsRng, SaltString},
    Argon2, PasswordHasher, PasswordVerifier,
};
use chrono::{DateTime, Duration, Utc};
use jsonwebtoken::{decode, encode, Algorithm, DecodingKey, EncodingKey, Header, Validation};
use sha2::{Digest, Sha256};
use sqlx::Row;
use tracing::info;
use uuid::Uuid;

use crate::{
    config::Config,
    database::DatabasePool,
    error::{AuthError, AuthResult},
    models::*,
};

/// Core authentication service implementation
#[derive(Clone)]
pub struct AuthServiceImpl {
    db: DatabasePool,
    config: Config,
    encoding_key: EncodingKey,
    decoding_key: DecodingKey,
}

impl std::fmt::Debug for AuthServiceImpl {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("AuthServiceImpl")
            .field("db", &self.db)
            .field("config", &self.config)
            .field("encoding_key", &"[REDACTED]")
            .field("decoding_key", &"[REDACTED]")
            .finish()
    }
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

    /// Create a new user account
    pub async fn create_user(
        &self,
        first_name: String,
        last_name: String,
        email: String,
        password: String,
        role: UserRole,
    ) -> AuthResult<User> {
        // Check if user already exists
        if (self.get_user_by_email(&email).await).is_ok() {
            return Err(AuthError::UserAlreadyExists);
        }

        // Validate password strength
        self.validate_password_strength(&password)?;

        // Hash password
        let password_hash = self.hash_password(&password)?;

        // Create user
        let user_id = Uuid::new_v4();
        let email_verified = !self.config.features.email_verification_required;

        let user = sqlx::query_as::<_, User>(
            r#"
            INSERT INTO users (
                id, email, password_hash, first_name, last_name, role, status, 
                email_verified, failed_login_attempts, created_at, updated_at
            ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, 0, NOW(), NOW())
            RETURNING *
            "#,
        )
        .bind(user_id)
        .bind(&email)
        .bind(&password_hash)
        .bind(&first_name)
        .bind(&last_name)
        .bind(&role)
        .bind(UserStatus::Active)
        .bind(email_verified)
        .fetch_one(&self.db.pool)
        .await?;

        // Send email verification if required
        if self.config.features.email_verification_required {
            self.send_email_verification(&user).await?;
        }

        // Log user creation
        self.log_security_event("USER_CREATED", Some(user.id), None, None).await?;

        Ok(user)
    }

    /// Refresh access token using refresh token
    pub async fn refresh_token(&self, refresh_token: &str) -> AuthResult<LoginResponse> {
        // Find session by refresh token hash
        let refresh_token_hash = self.hash_token(refresh_token);
        
        let session = sqlx::query_as::<_, UserSession>(
            r#"
            SELECT * FROM user_sessions 
            WHERE refresh_token_hash = $1 
            AND expires_at > NOW() 
            AND revoked = FALSE
            "#,
        )
        .bind(&refresh_token_hash)
        .fetch_optional(&self.db.pool)
        .await?
        .ok_or(AuthError::TokenInvalid)?;

        // Get user information
        let user = self.get_user_by_id(session.user_id).await?;

        // Check if user can still login
        if !user.can_login() {
            return Err(AuthError::AccountDisabled);
        }

        // Create new session and tokens
        let new_session_id = Uuid::new_v4();
        let expires_at = Utc::now() + Duration::hours(self.config.jwt.access_token_expiry_hours);

        // Create new JWT claims
        let claims = AuthClaims {
            sub: user.id,
            email: user.email.clone(),
            role: user.role.clone(),
            exp: expires_at.timestamp(),
            iat: Utc::now().timestamp(),
            iss: self.config.jwt.issuer.clone(),
            aud: self.config.jwt.audience.clone(),
            jti: new_session_id,
        };

        // Generate new tokens
        let access_token = self.generate_jwt_token(&claims)?;
        let new_refresh_token = self.generate_refresh_token();

        // Revoke old session
        self.revoke_session(session.id).await?;

        // Create new session
        self.create_session(
            new_session_id,
            user.id,
            &access_token,
            Some(&new_refresh_token),
            expires_at,
        ).await?;

        // Log token refresh
        self.log_security_event("TOKEN_REFRESHED", Some(user.id), None, None).await?;

        Ok(LoginResponse {
            user_id: user.id,
            email: user.email,
            role: user.role,
            access_token,
            refresh_token: Some(new_refresh_token),
            expires_at,
            session_id: new_session_id,
        })
    }

    /// Initiate password reset process
    pub async fn forgot_password(&self, email: &str) -> AuthResult<()> {
        // Try to get user (but don't reveal if user exists)
        if let Ok(user) = self.get_user_by_email(email).await {
            // Generate reset token
            let reset_token = self.generate_reset_token();
            let token_hash = self.hash_token(&reset_token);
            let expires_at = Utc::now() + Duration::hours(1); // 1 hour expiry

            // Store reset token
            sqlx::query(
                r#"
                INSERT INTO password_reset_tokens (
                    id, user_id, token_hash, expires_at, created_at, used
                ) VALUES ($1, $2, $3, $4, NOW(), FALSE)
                "#,
            )
            .bind(Uuid::new_v4())
            .bind(user.id)
            .bind(&token_hash)
            .bind(expires_at)
            .execute(&self.db.pool)
            .await?;

            // Send password reset email
            self.send_password_reset_email(&user, &reset_token).await?;

            // Log password reset request
            self.log_security_event("PASSWORD_RESET_REQUESTED", Some(user.id), None, None).await?;
        }

        Ok(())
    }

    /// Reset password using reset token
    pub async fn reset_password(&self, token: &str, new_password: &str) -> AuthResult<()> {
        // Validate password strength
        self.validate_password_strength(new_password)?;

        // Find and validate reset token
        let token_hash = self.hash_token(token);
        
        let reset_token_row = sqlx::query(
            r#"
            SELECT id, user_id FROM password_reset_tokens 
            WHERE token_hash = $1 
            AND expires_at > NOW() 
            AND used = FALSE
            "#,
        )
        .bind(&token_hash)
        .fetch_optional(&self.db.pool)
        .await?
        .ok_or(AuthError::TokenInvalid)?;
        
        let reset_token_id: Uuid = reset_token_row.get("id");
        let reset_token_user_id: Uuid = reset_token_row.get("user_id");

        // Hash new password
        let password_hash = self.hash_password(new_password)?;

        // Update user password
        sqlx::query(
            "UPDATE users SET password_hash = $1, password_changed_at = NOW() WHERE id = $2"
        )
        .bind(&password_hash)
        .bind(reset_token_user_id)
        .execute(&self.db.pool)
        .await?;

        // Mark reset token as used
        sqlx::query("UPDATE password_reset_tokens SET used = TRUE WHERE id = $1")
            .bind(reset_token_id)
            .execute(&self.db.pool)
            .await?;

        // Revoke all existing sessions for security
        sqlx::query("UPDATE user_sessions SET revoked = TRUE, revoked_at = NOW() WHERE user_id = $1")
            .bind(reset_token_user_id)
            .execute(&self.db.pool)
            .await?;

        // Log password reset
        self.log_security_event("PASSWORD_RESET_COMPLETED", Some(reset_token_user_id), None, None).await?;

        Ok(())
    }

    /// Verify email address using verification token
    pub async fn verify_email(&self, token: &str) -> AuthResult<()> {
        // Find and validate verification token
        let token_hash = self.hash_token(token);
        
        let verification_token_row = sqlx::query(
            r#"
            SELECT id, user_id FROM email_verification_tokens 
            WHERE token_hash = $1 
            AND expires_at > NOW() 
            AND used = FALSE
            "#,
        )
        .bind(&token_hash)
        .fetch_optional(&self.db.pool)
        .await?
        .ok_or(AuthError::TokenInvalid)?;
        
        let verification_token_id: Uuid = verification_token_row.get("id");
        let verification_token_user_id: Uuid = verification_token_row.get("user_id");

        // Update user email_verified status
        sqlx::query("UPDATE users SET email_verified = TRUE WHERE id = $1")
            .bind(verification_token_user_id)
            .execute(&self.db.pool)
            .await?;

        // Mark verification token as used
        sqlx::query("UPDATE email_verification_tokens SET used = TRUE WHERE id = $1")
            .bind(verification_token_id)
            .execute(&self.db.pool)
            .await?;

        // Log email verification
        self.log_security_event("EMAIL_VERIFIED", Some(verification_token_user_id), None, None).await?;

        Ok(())
    }

    /// Send email verification
    async fn send_email_verification(&self, user: &User) -> AuthResult<()> {
        if !self.config.email.enabled {
            return Ok(());
        }

        // Generate verification token
        let verification_token = self.generate_verification_token();
        let token_hash = self.hash_token(&verification_token);
        let expires_at = Utc::now() + Duration::hours(24); // 24 hour expiry

        // Store verification token
        sqlx::query(
            r#"
            INSERT INTO email_verification_tokens (
                id, user_id, token_hash, expires_at, created_at, used
            ) VALUES ($1, $2, $3, $4, NOW(), FALSE)
            "#,
        )
        .bind(Uuid::new_v4())
        .bind(user.id)
        .bind(&token_hash)
        .bind(expires_at)
        .execute(&self.db.pool)
        .await?;

        // Send verification email (implementation would depend on email service)
        info!("Email verification token for {}: {}", user.email, verification_token);

        Ok(())
    }

    /// Send password reset email
    async fn send_password_reset_email(&self, user: &User, token: &str) -> AuthResult<()> {
        if !self.config.email.enabled {
            return Ok(());
        }

        // Send password reset email (implementation would depend on email service)
        info!("Password reset token for {}: {}", user.email, token);

        Ok(())
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

    /// Get user by ID
    pub async fn get_user_by_id(&self, user_id: Uuid) -> AuthResult<User> {
        let user = sqlx::query_as::<_, User>("SELECT * FROM users WHERE id = $1")
            .bind(user_id)
            .fetch_optional(&self.db.pool)
            .await?
            .ok_or(AuthError::UserNotFound)?;

        Ok(user)
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

    /// Hash password using Argon2 (public method)
    pub fn hash_password(&self, password: &str) -> AuthResult<String> {
        let salt = SaltString::generate(&mut OsRng);
        let argon2 = Argon2::default();
        
        argon2
            .hash_password(password.as_bytes(), &salt)
            .map(|hash| hash.to_string())
            .map_err(|e| AuthError::PasswordHash(e.to_string()))
    }

    /// Verify password using Argon2 (public method)
    pub fn verify_password(&self, password: &str, hash: &str) -> AuthResult<bool> {
        let parsed_hash = argon2::PasswordHash::new(hash)
            .map_err(|e| AuthError::PasswordHash(e.to_string()))?;
        
        let argon2 = Argon2::default();
        Ok(argon2.verify_password(password.as_bytes(), &parsed_hash).is_ok())
    }

    /// Validate password strength
    fn validate_password_strength(&self, password: &str) -> AuthResult<()> {
        if password.len() < self.config.security.password_min_length {
            return Err(AuthError::validation(format!(
                "Password must be at least {} characters long",
                self.config.security.password_min_length
            )));
        }

        if self.config.security.password_require_uppercase && !password.chars().any(|c| c.is_uppercase()) {
            return Err(AuthError::validation("Password must contain at least one uppercase letter"));
        }

        if self.config.security.password_require_lowercase && !password.chars().any(|c| c.is_lowercase()) {
            return Err(AuthError::validation("Password must contain at least one lowercase letter"));
        }

        if self.config.security.password_require_numbers && !password.chars().any(|c| c.is_numeric()) {
            return Err(AuthError::validation("Password must contain at least one number"));
        }

        if self.config.security.password_require_symbols && !password.chars().any(|c| !c.is_alphanumeric()) {
            return Err(AuthError::validation("Password must contain at least one symbol"));
        }

        Ok(())
    }

    /// Revoke session
    async fn revoke_session(&self, session_id: Uuid) -> AuthResult<()> {
        sqlx::query("UPDATE user_sessions SET revoked = TRUE, revoked_at = NOW() WHERE id = $1")
            .bind(session_id)
            .execute(&self.db.pool)
            .await?;
        Ok(())
    }

    /// Generate JWT token
    fn generate_jwt_token(&self, claims: &AuthClaims) -> AuthResult<String> {
        let header = Header::new(Algorithm::HS256);
        encode(&header, claims, &self.encoding_key)
            .map_err(AuthError::Jwt)
    }

    /// Decode JWT token
    fn decode_jwt_token(&self, token: &str) -> AuthResult<AuthClaims> {
        let mut validation = Validation::new(Algorithm::HS256);
        validation.set_issuer(&[&self.config.jwt.issuer]);
        validation.set_audience(&[&self.config.jwt.audience]);

        let token_data = decode::<AuthClaims>(token, &self.decoding_key, &validation)
            .map_err(AuthError::Jwt)?;

        Ok(token_data.claims)
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

    /// Generate password reset token
    fn generate_reset_token(&self) -> String {
        Uuid::new_v4().to_string()
    }

    /// Generate email verification token
    fn generate_verification_token(&self) -> String {
        Uuid::new_v4().to_string()
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
