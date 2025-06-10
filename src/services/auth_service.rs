use std::collections::HashMap;
use std::net::IpAddr;

use anyhow::{anyhow, Result};
use argon2::password_hash::{rand_core::OsRng, SaltString};
use argon2::{Argon2, PasswordHash, PasswordHasher, PasswordVerifier};
use chrono::{Duration, Utc};
use jsonwebtoken::{decode, encode, Algorithm, DecodingKey, EncodingKey, Header, Validation};
use sqlx::PgPool;
use uuid::Uuid;

use crate::models::user::{
    AuthClaims, ChangePasswordRequest, ConfirmResetPasswordRequest, LoginRequest, LoginResponse,
    ResetPasswordRequest, User, UserManager, UserSafeProfile, UserSession, UserStatus,
};

#[derive(Clone)]
pub struct AuthService {
    pool: PgPool,
    user_manager: UserManager,
    jwt_secret: String,
}

impl AuthService {
    pub fn new(pool: PgPool, jwt_secret: String) -> Self {
        let user_manager = UserManager::new(pool.clone());
        Self {
            pool,
            user_manager,
            jwt_secret,
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
            SELECT id, user_id, token_hash, device_info, ip_address,
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
        let reset_token = sqlx::query!(
            r#"
            SELECT user_id, expires_at
            FROM password_reset_tokens
            WHERE token_hash = $1 AND used_at IS NULL AND expires_at > NOW()
            "#,
            token_hash
        )
        .fetch_one(&self.pool)
        .await
        .map_err(|_| anyhow!("Invalid or expired reset token"))?;

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
        .bind(reset_token.user_id)
        .execute(&mut *tx)
        .await?;

        sqlx::query("UPDATE password_reset_tokens SET used_at = NOW() WHERE token_hash = $1")
            .bind(&token_hash)
            .execute(&mut *tx)
            .await?;

        // Invalidate all existing sessions
        sqlx::query("DELETE FROM user_sessions WHERE user_id = $1")
            .bind(reset_token.user_id)
            .execute(&mut *tx)
            .await?;

        tx.commit().await?;

        // Log password reset
        self.log_user_activity(
            Some(reset_token.user_id),
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
            SELECT id, user_id, token_hash, device_info, ip_address,
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

    async fn create_session(
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

        // Convert IpAddr to String for database storage
        let ip_str = ip_address.map(|ip| ip.to_string());

        let session = sqlx::query_as::<_, UserSession>(
            r#"
            INSERT INTO user_sessions (id, user_id, token_hash, device_info, ip_address, expires_at)
            VALUES ($1, $2, $3, $4, $5, $6)
            RETURNING id, user_id, token_hash, device_info, ip_address, expires_at, created_at, last_used_at
            "#,
        )
        .bind(session_id)
        .bind(user_id)
        .bind(&token_hash)
        .bind(user_agent)
        .bind(ip_str)
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
        // Convert IpAddr to String for database storage
        let ip_str = ip_address.map(|ip| ip.to_string());

        sqlx::query(
            r#"
            INSERT INTO user_activity_log 
            (user_id, action, resource_type, resource_id, ip_address, user_agent, details)
            VALUES ($1, $2, $3, $4, $5, $6, $7)
            "#,
        )
        .bind(user_id)
        .bind(action)
        .bind(resource_type)
        .bind(resource_id)
        .bind(ip_str)
        .bind(user_agent)
        .bind(details)
        .execute(&self.pool)
        .await?;

        Ok(())
    }
}
