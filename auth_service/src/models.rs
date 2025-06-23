use crate::error::AuthError;
use axum::{
    async_trait,
    extract::FromRequestParts,
    http::request::Parts,
};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;
use validator::Validate;

/// User role enumeration
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, sqlx::Type)]
#[sqlx(type_name = "user_role")]
pub enum UserRole {
    #[serde(rename = "guest")]
    #[sqlx(rename = "guest")]
    Guest,
    #[serde(rename = "data_analyst")]
    #[sqlx(rename = "data_analyst")]
    DataAnalyst,
    #[serde(rename = "research_scientist")]
    #[sqlx(rename = "research_scientist")]
    ResearchScientist,
    #[serde(rename = "lab_technician")]
    #[sqlx(rename = "lab_technician")]
    LabTechnician,
    #[serde(rename = "principal_investigator")]
    #[sqlx(rename = "principal_investigator")]
    PrincipalInvestigator,
    #[serde(rename = "lab_administrator")]
    #[sqlx(rename = "lab_administrator")]
    LabAdministrator,
}

impl Default for UserRole {
    fn default() -> Self {
        Self::Guest
    }
}

/// User status enumeration
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, sqlx::Type)]
#[sqlx(type_name = "user_status")]
pub enum UserStatus {
    #[serde(rename = "active")]
    #[sqlx(rename = "active")]
    Active,
    #[serde(rename = "inactive")]
    #[sqlx(rename = "inactive")]
    Inactive,
    #[serde(rename = "suspended")]
    #[sqlx(rename = "suspended")]
    Suspended,
    #[serde(rename = "deleted")]
    #[sqlx(rename = "deleted")]
    Deleted,
}

impl Default for UserStatus {
    fn default() -> Self {
        Self::Active
    }
}

/// User model
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct User {
    pub id: Uuid,
    pub email: String,
    #[serde(skip_serializing)]
    pub password_hash: String,
    pub first_name: String,
    pub last_name: String,
    pub role: UserRole,
    pub status: UserStatus,

    // Profile information
    pub department: Option<String>,
    pub position: Option<String>,
    pub lab_affiliation: Option<String>,
    pub phone: Option<String>,

    // Authentication fields
    pub email_verified: bool,
    pub failed_login_attempts: i32,
    pub locked_until: Option<DateTime<Utc>>,
    pub last_login_at: Option<DateTime<Utc>>,
    pub password_changed_at: Option<DateTime<Utc>>,

    // Timestamps
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl User {
    /// Check if the user can login
    pub fn can_login(&self) -> bool {
        self.status == UserStatus::Active && self.email_verified && !self.is_locked()
    }

    /// Check if the user account is locked
    pub fn is_locked(&self) -> bool {
        if let Some(locked_until) = self.locked_until {
            Utc::now() < locked_until
        } else {
            false
        }
    }

    /// Get the user's display name
    pub fn display_name(&self) -> String {
        format!("{} {}", self.first_name, self.last_name)
    }

    /// Check if user has a specific role or higher
    pub fn has_role_or_higher(&self, required_role: &UserRole) -> bool {
        let role_hierarchy = [
            UserRole::Guest,
            UserRole::DataAnalyst,
            UserRole::ResearchScientist,
            UserRole::LabTechnician,
            UserRole::PrincipalInvestigator,
            UserRole::LabAdministrator,
        ];

        let user_level = role_hierarchy
            .iter()
            .position(|r| r == &self.role)
            .unwrap_or(0);
        let required_level = role_hierarchy
            .iter()
            .position(|r| r == required_role)
            .unwrap_or(usize::MAX);

        user_level >= required_level
    }

    /// Check if user is admin
    pub fn is_admin(&self) -> bool {
        self.role == UserRole::LabAdministrator
    }
}

/// User session model
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct UserSession {
    pub id: Uuid,
    pub user_id: Uuid,

    // Token information (hashed)
    #[serde(skip_serializing)]
    pub token_hash: String,
    #[serde(skip_serializing)]
    #[allow(dead_code)]
    pub refresh_token_hash: Option<String>,

    // Session metadata
    pub device_info: Option<serde_json::Value>,
    pub ip_address: Option<String>,
    pub user_agent: Option<String>,

    // Session lifecycle
    pub created_at: DateTime<Utc>,
    pub last_used_at: DateTime<Utc>,
    pub expires_at: DateTime<Utc>,
    pub revoked: bool,
    pub revoked_at: Option<DateTime<Utc>>,
}

/// JWT Claims
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthClaims {
    pub sub: Uuid,      // Subject (user ID)
    pub email: String,  // User email
    pub role: UserRole, // User role
    pub exp: i64,       // Expiration time
    pub iat: i64,       // Issued at
    pub iss: String,    // Issuer
    pub aud: String,    // Audience
    pub jti: Uuid,      // JWT ID (session ID)
}

/// Login request
#[derive(Debug, Clone, Deserialize, Validate)]
pub struct LoginRequest {
    #[validate(email(message = "Invalid email format"))]
    pub email: String,
    #[validate(length(min = 1, message = "Password is required"))]
    pub password: String,
    pub remember_me: Option<bool>,
}

/// Login response
#[derive(Debug, Clone, Serialize)]
pub struct LoginResponse {
    pub user_id: Uuid,
    pub email: String,
    pub role: UserRole,
    pub access_token: String,
    pub refresh_token: Option<String>,
    pub expires_at: DateTime<Utc>,
    pub session_id: Uuid,
}

/// Token validation request
#[derive(Debug, Clone, Deserialize, Validate)]
pub struct ValidateTokenRequest {
    #[validate(length(min = 1, message = "Token is required"))]
    pub token: String,
}

/// Token validation response
#[derive(Debug, Clone, Serialize)]
pub struct ValidateTokenResponse {
    pub valid: bool,
    pub user_id: Option<Uuid>,
    pub email: Option<String>,
    pub role: Option<UserRole>,
    pub session_id: Option<Uuid>,
    pub expires_at: Option<DateTime<Utc>>,
}

/// Permission validation request
#[derive(Debug, Clone, Deserialize, Validate)]
pub struct ValidatePermissionsRequest {
    #[validate(length(min = 1, message = "Token is required"))]
    pub token: String,
    #[allow(dead_code)]
    pub required_role: UserRole,
}

/// Permission validation response
#[derive(Debug, Clone, Serialize)]
#[allow(dead_code)]
pub struct ValidatePermissionsResponse {
    pub authorized: bool,
    pub user: Option<serde_json::Value>,
    pub reason: Option<String>,
}

/// User creation request (admin only)
#[derive(Debug, Clone, Deserialize, Validate)]
pub struct CreateUserRequest {
    #[validate(email(message = "Invalid email format"))]
    pub email: String,
    #[validate(length(min = 8, message = "Password must be at least 8 characters"))]
    pub password: String,
    #[validate(length(min = 1, message = "First name is required"))]
    pub first_name: String,
    #[validate(length(min = 1, message = "Last name is required"))]
    pub last_name: String,
    #[allow(dead_code)]
    pub role: UserRole,
    #[allow(dead_code)]
    pub department: Option<String>,
    #[allow(dead_code)]
    pub position: Option<String>,
    #[allow(dead_code)]
    pub lab_affiliation: Option<String>,
    #[allow(dead_code)]
    pub phone: Option<String>,
}

/// User update request
#[derive(Debug, Clone, Deserialize, Validate)]
pub struct UpdateUserRequest {
    #[allow(dead_code)]
    pub first_name: Option<String>,
    #[allow(dead_code)]
    pub last_name: Option<String>,
    #[allow(dead_code)]
    pub department: Option<String>,
    #[allow(dead_code)]
    pub position: Option<String>,
    #[allow(dead_code)]
    pub lab_affiliation: Option<String>,
    #[allow(dead_code)]
    pub phone: Option<String>,
    #[allow(dead_code)]
    pub role: Option<UserRole>,
    #[allow(dead_code)]
    pub status: Option<UserStatus>,
}

/// Security audit log entry
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct SecurityAuditLog {
    pub event_id: Uuid,
    pub event_type: String,
    pub user_id: Option<Uuid>,
    pub ip_address: Option<String>,
    pub user_agent: Option<String>,
    pub resource: Option<String>,
    pub action: Option<String>,
    pub outcome: Option<String>,
    pub severity: String,
    pub details: Option<serde_json::Value>,
    pub timestamp: DateTime<Utc>,
}

/// Helper function to check role hierarchy
#[allow(dead_code)]
pub fn has_role_or_higher(user_role: &UserRole, required_role: &UserRole) -> bool {
    let role_hierarchy = [
        UserRole::Guest,
        UserRole::DataAnalyst,
        UserRole::ResearchScientist,
        UserRole::LabTechnician,
        UserRole::PrincipalInvestigator,
        UserRole::LabAdministrator,
    ];

    let user_level = role_hierarchy
        .iter()
        .position(|r| r == user_role)
        .unwrap_or(0);
    let required_level = role_hierarchy
        .iter()
        .position(|r| r == required_role)
        .unwrap_or(usize::MAX);

    user_level >= required_level
}

/// Result type for authentication operations
#[allow(dead_code)]
pub type AuthResult<T> = Result<T, crate::error::AuthError>;

/// Axum extractor for authenticated users
#[async_trait]
impl<S> FromRequestParts<S> for User
where
    S: Send + Sync,
{
    type Rejection = AuthError;

    async fn from_request_parts(parts: &mut Parts, _state: &S) -> Result<Self, Self::Rejection> {
        parts
            .extensions
            .get::<User>()
            .cloned()
            .ok_or_else(|| AuthError::authentication("User not authenticated"))
    }
}
