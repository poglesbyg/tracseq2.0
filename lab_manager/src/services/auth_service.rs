use anyhow::{anyhow, Result};
use argon2::password_hash::{rand_core::OsRng, SaltString};
use argon2::{Argon2, PasswordHash, PasswordHasher, PasswordVerifier};
use chrono::{DateTime, Duration, Utc};
use jsonwebtoken::{decode, encode, Algorithm, DecodingKey, EncodingKey, Header, Validation};
use sqlx::{PgPool, Row};
use std::collections::HashMap;
use std::net::IpAddr;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{error, info, warn};
use uuid::Uuid;

use crate::errors::{api::ApiError, ComponentResult};
use crate::models::user::{
    AuthClaims, ChangePasswordRequest, ConfirmResetPasswordRequest, CreateUserRequest,
    LoginRequest, LoginResponse, ResetPasswordRequest, User, UserManager, UserSession, UserStatus,
};

#[derive(Clone)]
pub struct AuthService {
    pool: PgPool,
    user_manager: UserManager,
    jwt_secret: String,
    rate_limiter: Arc<RwLock<RateLimiter>>,
    security_config: SecurityConfig,
    audit_logger: AuditLogger,
}

#[derive(Debug, Clone)]
pub struct SecurityConfig {
    pub jwt_expiration_hours: i64,
    pub refresh_token_expiration_days: i64,
    pub max_login_attempts: u32,
    pub lockout_duration_minutes: u32,
    pub password_min_length: usize,
    pub password_require_uppercase: bool,
    pub password_require_lowercase: bool,
    pub password_require_numbers: bool,
    pub password_require_symbols: bool,
    pub session_timeout_minutes: u32,
}

impl Default for SecurityConfig {
    fn default() -> Self {
        Self {
            jwt_expiration_hours: 8,
            refresh_token_expiration_days: 30,
            max_login_attempts: 5,
            lockout_duration_minutes: 15,
            password_min_length: 8,
            password_require_uppercase: true,
            password_require_lowercase: true,
            password_require_numbers: true,
            password_require_symbols: false,
            session_timeout_minutes: 480, // 8 hours
        }
    }
}

/// Rate limiting implementation
pub struct RateLimiter {
    attempts: HashMap<String, Vec<DateTime<Utc>>>,
    lockouts: HashMap<String, DateTime<Utc>>,
}

impl RateLimiter {
    pub fn new() -> Self {
        Self {
            attempts: HashMap::new(),
            lockouts: HashMap::new(),
        }
    }

    pub fn check_rate_limit(
        &mut self,
        identifier: &str,
        max_attempts: u32,
        window_minutes: u32,
    ) -> bool {
        let now = Utc::now();
        let window_start = now - Duration::minutes(window_minutes as i64);

        // Check if currently locked out
        if let Some(lockout_until) = self.lockouts.get(identifier) {
            if now < *lockout_until {
                return false;
            } else {
                self.lockouts.remove(identifier);
            }
        }

        // Clean old attempts and count recent ones
        let attempts = self
            .attempts
            .entry(identifier.to_string())
            .or_insert_with(Vec::new);
        attempts.retain(|&attempt_time| attempt_time > window_start);

        attempts.len() < max_attempts as usize
    }

    pub fn record_attempt(
        &mut self,
        identifier: &str,
        max_attempts: u32,
        lockout_duration_minutes: u32,
    ) {
        let now = Utc::now();
        let attempts = self
            .attempts
            .entry(identifier.to_string())
            .or_insert_with(Vec::new);
        attempts.push(now);

        if attempts.len() >= max_attempts as usize {
            let lockout_until = now + Duration::minutes(lockout_duration_minutes as i64);
            self.lockouts.insert(identifier.to_string(), lockout_until);
            warn!(
                "Account locked due to too many failed attempts: {}",
                identifier
            );
        }
    }

    pub fn reset_attempts(&mut self, identifier: &str) {
        self.attempts.remove(identifier);
        self.lockouts.remove(identifier);
    }
}

/// Audit logging for security events
#[derive(Clone)]
pub struct AuditLogger {
    pool: PgPool,
}

impl AuditLogger {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn log_security_event(&self, event: SecurityEvent) -> Result<()> {
        sqlx::query(
            r#"
            INSERT INTO security_audit_log (
                event_id, event_type, user_id, user_email, ip_address,
                user_agent, details, severity, timestamp
            ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)
            "#,
        )
        .bind(event.event_id)
        .bind(event.event_type.to_string())
        .bind(event.user_id)
        .bind(event.user_email)
        .bind(event.ip_address)
        .bind(event.user_agent)
        .bind(serde_json::to_value(&event.details)?)
        .bind(event.severity.to_string())
        .bind(event.timestamp)
        .execute(&self.pool)
        .await?;

        Ok(())
    }
}

#[derive(Debug, Clone)]
pub struct SecurityEvent {
    pub event_id: Uuid,
    pub event_type: SecurityEventType,
    pub user_id: Option<Uuid>,
    pub user_email: Option<String>,
    pub ip_address: Option<String>,
    pub user_agent: Option<String>,
    pub details: HashMap<String, serde_json::Value>,
    pub severity: SecuritySeverity,
    pub timestamp: DateTime<Utc>,
}

#[derive(Debug, Clone)]
pub enum SecurityEventType {
    LoginSuccess,
    LoginFailure,
    LoginLockout,
    PasswordChanged,
    AccountCreated,
    AccountDisabled,
    PermissionEscalation,
    SensitiveDataAccess,
    InvalidTokenUsage,
    SessionExpired,
}

impl std::fmt::Display for SecurityEventType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SecurityEventType::LoginSuccess => write!(f, "LOGIN_SUCCESS"),
            SecurityEventType::LoginFailure => write!(f, "LOGIN_FAILURE"),
            SecurityEventType::LoginLockout => write!(f, "LOGIN_LOCKOUT"),
            SecurityEventType::PasswordChanged => write!(f, "PASSWORD_CHANGED"),
            SecurityEventType::AccountCreated => write!(f, "ACCOUNT_CREATED"),
            SecurityEventType::AccountDisabled => write!(f, "ACCOUNT_DISABLED"),
            SecurityEventType::PermissionEscalation => write!(f, "PERMISSION_ESCALATION"),
            SecurityEventType::SensitiveDataAccess => write!(f, "SENSITIVE_DATA_ACCESS"),
            SecurityEventType::InvalidTokenUsage => write!(f, "INVALID_TOKEN_USAGE"),
            SecurityEventType::SessionExpired => write!(f, "SESSION_EXPIRED"),
        }
    }
}

#[derive(Debug, Clone)]
pub enum SecuritySeverity {
    Low,
    Medium,
    High,
    Critical,
}

impl std::fmt::Display for SecuritySeverity {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SecuritySeverity::Low => write!(f, "LOW"),
            SecuritySeverity::Medium => write!(f, "MEDIUM"),
            SecuritySeverity::High => write!(f, "HIGH"),
            SecuritySeverity::Critical => write!(f, "CRITICAL"),
        }
    }
}

/// Password validation and strength checking
pub struct PasswordValidator {
    config: SecurityConfig,
}

impl PasswordValidator {
    pub fn new(config: SecurityConfig) -> Self {
        Self { config }
    }

    pub fn validate_password(&self, password: &str) -> Result<(), Vec<String>> {
        let mut errors = Vec::new();

        if password.len() < self.config.password_min_length {
            errors.push(format!(
                "Password must be at least {} characters long",
                self.config.password_min_length
            ));
        }

        if self.config.password_require_uppercase && !password.chars().any(|c| c.is_uppercase()) {
            errors.push("Password must contain at least one uppercase letter".to_string());
        }

        if self.config.password_require_lowercase && !password.chars().any(|c| c.is_lowercase()) {
            errors.push("Password must contain at least one lowercase letter".to_string());
        }

        if self.config.password_require_numbers && !password.chars().any(|c| c.is_numeric()) {
            errors.push("Password must contain at least one number".to_string());
        }

        if self.config.password_require_symbols
            && !password.chars().any(|c| c.is_ascii_punctuation())
        {
            errors.push("Password must contain at least one symbol".to_string());
        }

        // Check against common passwords
        if self.is_common_password(password) {
            errors.push("Password is too common, please choose a more secure password".to_string());
        }

        if errors.is_empty() {
            Ok(())
        } else {
            Err(errors)
        }
    }

    fn is_common_password(&self, password: &str) -> bool {
        // Common passwords to reject
        let common_passwords = [
            "password",
            "123456",
            "123456789",
            "12345678",
            "12345",
            "1234567",
            "admin",
            "password123",
            "letmein",
            "welcome",
            "monkey",
            "1234567890",
            "qwerty",
            "abc123",
        ];

        let lower_password = password.to_lowercase();
        common_passwords
            .iter()
            .any(|&common| lower_password.contains(common))
    }
}

impl AuthService {
    pub fn new(pool: PgPool, jwt_secret: String) -> Self {
        let user_manager = UserManager::new(pool.clone());
        let security_config = SecurityConfig::default();
        let audit_logger = AuditLogger::new(pool.clone());

        Self {
            pool,
            user_manager,
            jwt_secret,
            rate_limiter: Arc::new(RwLock::new(RateLimiter::new())),
            security_config,
            audit_logger,
        }
    }

    /// Authenticate user with email and password
    pub async fn login(
        &self,
        request: LoginRequest,
        ip_address: Option<IpAddr>,
        user_agent: Option<String>,
    ) -> Result<LoginResponse> {
        // Get user by email
        let user = self
            .user_manager
            .get_user_by_email(&request.email)
            .await
            .map_err(|_| anyhow!("Invalid email or password"))?;

        // Check if user can login
        if !user.can_login() {
            if user.is_locked() {
                return Err(anyhow!("Account is locked. Please try again later."));
            }
            if !user.email_verified {
                return Err(anyhow!(
                    "Please verify your email address before logging in."
                ));
            }
            if !matches!(user.status, UserStatus::Active) {
                return Err(anyhow!(
                    "Account is inactive. Please contact administrator."
                ));
            }
        }

        // Verify password
        if !self
            .user_manager
            .verify_password(&request.password, &user.password_hash)
            .await
        {
            // Increment failed login attempts
            let _ = self.user_manager.increment_failed_login(user.id).await;
            return Err(anyhow!("Invalid email or password"));
        }

        // Reset failed login attempts on successful login
        self.user_manager.reset_failed_login(user.id).await?;

        // Create JWT token and session
        let session = self
            .create_session(user.id, ip_address, user_agent.clone())
            .await?;

        let claims = AuthClaims {
            sub: user.id,
            email: user.email.clone(),
            role: user.role.clone(),
            exp: session.expires_at.timestamp(),
            iat: Utc::now().timestamp(),
            jti: session.id,
        };

        let token = self.generate_jwt_token(&claims)?;

        // Log successful login
        self.log_user_activity(
            Some(user.id),
            "login",
            None,
            None,
            ip_address,
            user_agent,
            None,
        )
        .await?;

        Ok(LoginResponse {
            user: user.into(),
            token,
            expires_at: session.expires_at,
        })
    }

    /// Logout user by invalidating session
    pub async fn logout(&self, session_id: Uuid, user_id: Option<Uuid>) -> Result<()> {
        // Delete session
        sqlx::query("DELETE FROM user_sessions WHERE id = $1")
            .bind(session_id)
            .execute(&self.pool)
            .await?;

        // Log logout
        if let Some(user_id) = user_id {
            self.log_user_activity(Some(user_id), "logout", None, None, None, None, None)
                .await?;
        }

        Ok(())
    }

    /// Verify JWT token and return user info
    pub async fn verify_token(&self, token: &str) -> Result<(User, UserSession)> {
        let claims = self.decode_jwt_token(token)?;

        // Get session from database
        let session = sqlx::query_as::<_, UserSession>(
            r#"
            SELECT id, user_id, token_hash, device_info, ip_address::text,
                   expires_at, created_at, last_used_at
            FROM user_sessions
            WHERE id = $1 AND expires_at > NOW()
            "#,
        )
        .bind(claims.jti)
        .fetch_one(&self.pool)
        .await
        .map_err(|_| anyhow!("Invalid or expired session"))?;

        // Verify token hash matches
        let token_hash = self.hash_token(token);
        if session.token_hash != token_hash {
            return Err(anyhow!("Invalid token"));
        }

        // Get user
        let user = self.user_manager.get_user_by_id(session.user_id).await?;

        // Check if user is still active
        if !user.can_login() {
            return Err(anyhow!("User account is inactive"));
        }

        // Update last used timestamp
        let _ = sqlx::query("UPDATE user_sessions SET last_used_at = NOW() WHERE id = $1")
            .bind(session.id)
            .execute(&self.pool)
            .await;

        Ok((user, session))
    }

    /// Change user password
    pub async fn change_password(
        &self,
        user_id: Uuid,
        request: ChangePasswordRequest,
    ) -> Result<()> {
        let user = self.user_manager.get_user_by_id(user_id).await?;

        // Verify current password
        if !self
            .user_manager
            .verify_password(&request.current_password, &user.password_hash)
            .await
        {
            return Err(anyhow!("Current password is incorrect"));
        }

        // Hash new password
        let salt = SaltString::generate(&mut OsRng);
        let argon2 = Argon2::default();
        let new_password_hash = argon2
            .hash_password(request.new_password.as_bytes(), &salt)
            .map_err(|_| anyhow!("Password hashing failed"))?
            .to_string();

        // Update password in database
        sqlx::query(
            r#"
            UPDATE users 
            SET password_hash = $1, password_changed_at = NOW()
            WHERE id = $2
            "#,
        )
        .bind(&new_password_hash)
        .bind(user_id)
        .execute(&self.pool)
        .await?;

        // Invalidate all existing sessions except current one (force re-login)
        sqlx::query("DELETE FROM user_sessions WHERE user_id = $1")
            .bind(user_id)
            .execute(&self.pool)
            .await?;

        // Log password change
        self.log_user_activity(
            Some(user_id),
            "password_change",
            None,
            None,
            None,
            None,
            None,
        )
        .await?;

        Ok(())
    }

    /// Initiate password reset
    pub async fn request_password_reset(&self, request: ResetPasswordRequest) -> Result<String> {
        let user = self
            .user_manager
            .get_user_by_email(&request.email)
            .await
            .map_err(|_| anyhow!("Email address not found"))?;

        // Generate reset token
        let reset_token = Uuid::new_v4().to_string();
        let token_hash = self.hash_token(&reset_token);
        let expires_at = Utc::now() + Duration::hours(24);

        // Store reset token
        sqlx::query(
            r#"
            INSERT INTO password_reset_tokens (user_id, token_hash, expires_at)
            VALUES ($1, $2, $3)
            "#,
        )
        .bind(user.id)
        .bind(&token_hash)
        .bind(expires_at)
        .execute(&self.pool)
        .await?;

        // Log password reset request
        self.log_user_activity(
            Some(user.id),
            "password_reset_requested",
            None,
            None,
            None,
            None,
            None,
        )
        .await?;

        Ok(reset_token)
    }

    /// Confirm password reset with token
    pub async fn confirm_password_reset(&self, request: ConfirmResetPasswordRequest) -> Result<()> {
        let token_hash = self.hash_token(&request.token);

        // Find and verify reset token
        let reset_token = sqlx::query(
            r#"
            SELECT user_id, expires_at
            FROM password_reset_tokens
            WHERE token_hash = $1 AND used_at IS NULL AND expires_at > NOW()
            "#,
        )
        .bind(&token_hash)
        .fetch_one(&self.pool)
        .await
        .map_err(|_| anyhow!("Invalid or expired reset token"))?;

        let user_id: Uuid = reset_token.get("user_id");
        let _expires_at: chrono::DateTime<chrono::Utc> = reset_token.get("expires_at");

        // Hash new password
        let salt = SaltString::generate(&mut OsRng);
        let argon2 = Argon2::default();
        let new_password_hash = argon2
            .hash_password(request.new_password.as_bytes(), &salt)
            .map_err(|_| anyhow!("Password hashing failed"))?
            .to_string();

        // Update password and mark token as used
        let mut tx = self.pool.begin().await?;

        sqlx::query(
            r#"
            UPDATE users 
            SET password_hash = $1, password_changed_at = NOW(), failed_login_attempts = 0, locked_until = NULL
            WHERE id = $2
            "#,
        )
        .bind(&new_password_hash)
        .bind(user_id)
        .execute(&mut *tx)
        .await?;

        sqlx::query("UPDATE password_reset_tokens SET used_at = NOW() WHERE token_hash = $1")
            .bind(&token_hash)
            .execute(&mut *tx)
            .await?;

        // Invalidate all existing sessions
        sqlx::query("DELETE FROM user_sessions WHERE user_id = $1")
            .bind(user_id)
            .execute(&mut *tx)
            .await?;

        tx.commit().await?;

        // Log password reset
        self.log_user_activity(
            Some(user_id),
            "password_reset_completed",
            None,
            None,
            None,
            None,
            None,
        )
        .await?;

        Ok(())
    }

    /// Get user's active sessions
    pub async fn get_user_sessions(&self, user_id: Uuid) -> Result<Vec<UserSession>> {
        let sessions = sqlx::query_as::<_, UserSession>(
            r#"
            SELECT id, user_id, token_hash, device_info, ip_address::text,
                   expires_at, created_at, last_used_at
            FROM user_sessions
            WHERE user_id = $1 AND expires_at > NOW()
            ORDER BY last_used_at DESC
            "#,
        )
        .bind(user_id)
        .fetch_all(&self.pool)
        .await?;

        Ok(sessions)
    }

    /// Revoke a specific session
    pub async fn revoke_session(&self, session_id: Uuid, user_id: Uuid) -> Result<()> {
        let result = sqlx::query("DELETE FROM user_sessions WHERE id = $1 AND user_id = $2")
            .bind(session_id)
            .bind(user_id)
            .execute(&self.pool)
            .await?;

        if result.rows_affected() == 0 {
            return Err(anyhow!("Session not found"));
        }

        // Log session revocation
        self.log_user_activity(
            Some(user_id),
            "session_revoked",
            None,
            None,
            None,
            None,
            Some(serde_json::json!({ "session_id": session_id })),
        )
        .await?;

        Ok(())
    }

    /// Revoke all sessions for a user (except optionally one)
    pub async fn revoke_all_sessions(
        &self,
        user_id: Uuid,
        except_session_id: Option<Uuid>,
    ) -> Result<u64> {
        let result = if let Some(except_id) = except_session_id {
            sqlx::query("DELETE FROM user_sessions WHERE user_id = $1 AND id != $2")
                .bind(user_id)
                .bind(except_id)
                .execute(&self.pool)
                .await?
        } else {
            sqlx::query("DELETE FROM user_sessions WHERE user_id = $1")
                .bind(user_id)
                .execute(&self.pool)
                .await?
        };

        // Log session revocation
        self.log_user_activity(
            Some(user_id),
            "all_sessions_revoked",
            None,
            None,
            None,
            None,
            Some(serde_json::json!({ "revoked_count": result.rows_affected() })),
        )
        .await?;

        Ok(result.rows_affected())
    }

    /// Clean up expired sessions and tokens
    pub async fn cleanup_expired(&self) -> Result<()> {
        // Remove expired sessions
        sqlx::query("DELETE FROM user_sessions WHERE expires_at <= NOW()")
            .execute(&self.pool)
            .await?;

        // Remove expired password reset tokens
        sqlx::query("DELETE FROM password_reset_tokens WHERE expires_at <= NOW()")
            .execute(&self.pool)
            .await?;

        // Remove expired email verification tokens
        sqlx::query("DELETE FROM email_verification_tokens WHERE expires_at <= NOW()")
            .execute(&self.pool)
            .await?;

        Ok(())
    }

    // Private helper methods

    pub async fn create_session(
        &self,
        user_id: Uuid,
        ip_address: Option<IpAddr>,
        user_agent: Option<String>,
    ) -> Result<UserSession> {
        let session_id = Uuid::new_v4();
        let expires_at = Utc::now() + Duration::days(7); // 7-day session

        // Create temporary token to hash
        let temp_token = format!("{}:{}", session_id, expires_at.timestamp());
        let token_hash = self.hash_token(&temp_token);

        let session = sqlx::query_as::<_, UserSession>(
            r#"
            INSERT INTO user_sessions (id, user_id, token_hash, device_info, ip_address, expires_at)
            VALUES ($1, $2, $3, $4, $5::inet, $6)
            RETURNING id, user_id, token_hash, device_info, ip_address::text, expires_at, created_at, last_used_at
            "#,
        )
        .bind(session_id)
        .bind(user_id)
        .bind(&token_hash)
        .bind(user_agent)
        .bind(ip_address.map(|ip| ip.to_string()))
        .bind(expires_at)
        .fetch_one(&self.pool)
        .await?;

        Ok(session)
    }

    fn generate_jwt_token(&self, claims: &AuthClaims) -> Result<String> {
        let header = Header::new(Algorithm::HS256);
        let token = encode(
            &header,
            claims,
            &EncodingKey::from_secret(self.jwt_secret.as_ref()),
        )?;
        Ok(token)
    }

    fn decode_jwt_token(&self, token: &str) -> Result<AuthClaims> {
        let mut validation = Validation::new(Algorithm::HS256);
        validation.leeway = 60; // 60 seconds leeway for time differences

        let token_data = decode::<AuthClaims>(
            token,
            &DecodingKey::from_secret(self.jwt_secret.as_ref()),
            &validation,
        )?;

        Ok(token_data.claims)
    }

    fn hash_token(&self, token: &str) -> String {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};

        let mut hasher = DefaultHasher::new();
        token.hash(&mut hasher);
        format!("{:x}", hasher.finish())
    }

    async fn log_user_activity(
        &self,
        user_id: Option<Uuid>,
        action: &str,
        resource_type: Option<&str>,
        resource_id: Option<Uuid>,
        ip_address: Option<IpAddr>,
        user_agent: Option<String>,
        details: Option<serde_json::Value>,
    ) -> Result<()> {
        sqlx::query(
            r#"
            INSERT INTO user_activity_log 
            (user_id, action, resource_type, resource_id, ip_address, user_agent, details)
            VALUES ($1, $2, $3, $4, $5::inet, $6, $7)
            "#,
        )
        .bind(user_id)
        .bind(action)
        .bind(resource_type)
        .bind(resource_id)
        .bind(ip_address.map(|ip| ip.to_string()))
        .bind(user_agent)
        .bind(details)
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    pub async fn login_with_security(
        &self,
        email: &str,
        password: &str,
        ip_address: Option<String>,
        user_agent: Option<String>,
    ) -> ComponentResult<LoginResponse, ApiError> {
        let identifier = format!("login:{}", email);

        // Check rate limit
        {
            let mut rate_limiter = self.rate_limiter.write().await;
            if !rate_limiter.check_rate_limit(
                &identifier,
                self.security_config.max_login_attempts,
                15,
            ) {
                self.audit_logger
                    .log_security_event(SecurityEvent {
                        event_id: Uuid::new_v4(),
                        event_type: SecurityEventType::LoginLockout,
                        user_id: None,
                        user_email: Some(email.to_string()),
                        ip_address: ip_address.clone(),
                        user_agent: user_agent.clone(),
                        details: HashMap::new(),
                        severity: SecuritySeverity::High,
                        timestamp: Utc::now(),
                    })
                    .await
                    .ok();

                return Err(ApiError::TooManyRequests(
                    "Account temporarily locked due to too many failed login attempts".to_string(),
                ));
            }
        }

        // Attempt login
        let login_result = self.authenticate_user(email, password).await;

        match login_result {
            Ok(user) => {
                // Reset rate limit on successful login
                {
                    let mut rate_limiter = self.rate_limiter.write().await;
                    rate_limiter.reset_attempts(&identifier);
                }

                // Log successful login
                self.audit_logger
                    .log_security_event(SecurityEvent {
                        event_id: Uuid::new_v4(),
                        event_type: SecurityEventType::LoginSuccess,
                        user_id: Some(user.id),
                        user_email: Some(email.to_string()),
                        ip_address,
                        user_agent,
                        details: HashMap::new(),
                        severity: SecuritySeverity::Low,
                        timestamp: Utc::now(),
                    })
                    .await
                    .ok();

                // Generate tokens
                let tokens = self.generate_tokens(&user)?;

                Ok(LoginResponse {
                    user: user.into(),
                    token: tokens.access_token,
                    expires_at: tokens.expires_at,
                })
            }
            Err(e) => {
                // Record failed attempt
                {
                    let mut rate_limiter = self.rate_limiter.write().await;
                    rate_limiter.record_attempt(
                        &identifier,
                        self.security_config.max_login_attempts,
                        self.security_config.lockout_duration_minutes,
                    );
                }

                // Log failed login
                self.audit_logger
                    .log_security_event(SecurityEvent {
                        event_id: Uuid::new_v4(),
                        event_type: SecurityEventType::LoginFailure,
                        user_id: None,
                        user_email: Some(email.to_string()),
                        ip_address,
                        user_agent,
                        details: {
                            let mut details = HashMap::new();
                            details.insert(
                                "error".to_string(),
                                serde_json::Value::String(e.to_string()),
                            );
                            details
                        },
                        severity: SecuritySeverity::Medium,
                        timestamp: Utc::now(),
                    })
                    .await
                    .ok();

                Err(e)
            }
        }
    }

    pub async fn create_user_with_validation(
        &self,
        request: CreateUserRequest,
    ) -> ComponentResult<User, ApiError> {
        // Validate password strength
        let validator = PasswordValidator::new(self.security_config.clone());
        if let Err(password_errors) = validator.validate_password(&request.password) {
            return Err(ApiError::ValidationError(format!(
                "Password validation failed: {}",
                password_errors.join(", ")
            )));
        }

        // Check if user already exists
        let existing_user = sqlx::query("SELECT id FROM users WHERE email = $1")
            .bind(&request.email)
            .fetch_optional(&self.pool)
            .await
            .map_err(|e| ApiError::DatabaseError(e.to_string()))?;

        if existing_user.is_some() {
            return Err(ApiError::Conflict(
                "User with this email already exists".to_string(),
            ));
        }

        // Hash password with Argon2
        let salt = SaltString::generate(&mut OsRng);
        let argon2 = Argon2::default();
        let password_hash = argon2
            .hash_password(request.password.as_bytes(), &salt)
            .map_err(|e| ApiError::InternalServerError(format!("Failed to hash password: {}", e)))?
            .to_string();

        // Create user
        let user_id = Uuid::new_v4();
        let now = Utc::now();

        sqlx::query(
            r#"
            INSERT INTO users (id, email, password_hash, first_name, last_name, role, status, created_at, updated_at)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)
            "#,
        )
        .bind(user_id)
        .bind(&request.email)
        .bind(&password_hash)
        .bind(&request.first_name)
        .bind(&request.last_name)
        .bind(request.role.to_string())
        .bind("active")
        .bind(now)
        .bind(now)
        .execute(&self.pool)
        .await
        .map_err(|e| ApiError::DatabaseError(e.to_string()))?;

        // Log account creation
        self.audit_logger
            .log_security_event(SecurityEvent {
                event_id: Uuid::new_v4(),
                event_type: SecurityEventType::AccountCreated,
                user_id: Some(user_id),
                user_email: Some(request.email.clone()),
                ip_address: None,
                user_agent: None,
                details: {
                    let mut details = HashMap::new();
                    details.insert(
                        "role".to_string(),
                        serde_json::Value::String(request.role.to_string()),
                    );
                    details
                },
                severity: SecuritySeverity::Medium,
                timestamp: Utc::now(),
            })
            .await
            .ok();

        // Fetch the created user from database to get all fields
        let user = self
            .user_manager
            .get_user_by_id(user_id)
            .await
            .map_err(|e| ApiError::DatabaseError(format!("Failed to fetch created user: {}", e)))?;

        Ok(user)
    }

    async fn authenticate_user(
        &self,
        email: &str,
        password: &str,
    ) -> ComponentResult<User, ApiError> {
        // Implementation with secure password verification
        todo!("Implement secure authentication")
    }

    fn generate_tokens(&self, user: &User) -> ComponentResult<TokenPair, ApiError> {
        // Implementation with secure token generation
        todo!("Implement secure token generation")
    }
}

#[derive(Debug, serde::Serialize)]
pub struct TokenPair {
    pub access_token: String,
    pub refresh_token: String,
    pub expires_at: DateTime<Utc>,
}
