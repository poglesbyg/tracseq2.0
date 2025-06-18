use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;
use validator::Validate;

/// User role enumeration
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, sqlx::Type)]
#[sqlx(type_name = "text")]
pub enum UserRole {
    #[serde(rename = "guest")]
    Guest,
    #[serde(rename = "data_analyst")]
    DataAnalyst,
    #[serde(rename = "research_scientist")]
    ResearchScientist,
    #[serde(rename = "lab_technician")]
    LabTechnician,
    #[serde(rename = "principal_investigator")]
    PrincipalInvestigator,
    #[serde(rename = "lab_administrator")]
    LabAdministrator,
}

impl Default for UserRole {
    fn default() -> Self {
        Self::Guest
    }
}

/// User status enumeration
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, sqlx::Type)]
#[sqlx(type_name = "text")]
pub enum UserStatus {
    #[serde(rename = "active")]
    Active,
    #[serde(rename = "inactive")]
    Inactive,
    #[serde(rename = "suspended")]
    Suspended,
    #[serde(rename = "pending")]
    Pending,
}

impl Default for UserStatus {
    fn default() -> Self {
        Self::Pending
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
    pub email_verified: bool,
    #[serde(skip_serializing)]
    pub verification_token: Option<String>,
    pub failed_login_attempts: i32,
    pub locked_until: Option<DateTime<Utc>>,
    pub last_login_at: Option<DateTime<Utc>>,
    pub password_changed_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,

    // Lab-specific fields
    pub department: Option<String>,
    pub position: Option<String>,
    pub phone: Option<String>,
    pub office_location: Option<String>,
    pub lab_affiliation: Option<String>,

    // External integration fields
    pub shibboleth_id: Option<String>,
    pub external_id: Option<String>,
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
