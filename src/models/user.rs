use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use std::collections::HashMap;
use uuid::Uuid;
use validator::Validate;

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::Type)]
#[sqlx(type_name = "user_role", rename_all = "snake_case")]
pub enum UserRole {
    LabAdministrator,
    PrincipalInvestigator,
    LabTechnician,
    ResearchScientist,
    DataAnalyst,
    Guest,
}

impl UserRole {
    pub fn as_str(&self) -> &'static str {
        match self {
            UserRole::LabAdministrator => "lab_administrator",
            UserRole::PrincipalInvestigator => "principal_investigator",
            UserRole::LabTechnician => "lab_technician",
            UserRole::ResearchScientist => "research_scientist",
            UserRole::DataAnalyst => "data_analyst",
            UserRole::Guest => "guest",
        }
    }

    pub fn display_name(&self) -> &'static str {
        match self {
            UserRole::LabAdministrator => "Lab Administrator",
            UserRole::PrincipalInvestigator => "Principal Investigator",
            UserRole::LabTechnician => "Lab Technician",
            UserRole::ResearchScientist => "Research Scientist",
            UserRole::DataAnalyst => "Data Analyst",
            UserRole::Guest => "Guest",
        }
    }

    pub fn description(&self) -> &'static str {
        match self {
            UserRole::LabAdministrator => "Full system access and user management capabilities",
            UserRole::PrincipalInvestigator => "Lab oversight and research coordination",
            UserRole::LabTechnician => "Sample processing and laboratory operations",
            UserRole::ResearchScientist => "Research activities and data analysis",
            UserRole::DataAnalyst => "Data analysis and reporting capabilities",
            UserRole::Guest => "Limited read-only access to laboratory data",
        }
    }

    pub fn all_roles() -> Vec<UserRole> {
        vec![
            UserRole::LabAdministrator,
            UserRole::PrincipalInvestigator,
            UserRole::LabTechnician,
            UserRole::ResearchScientist,
            UserRole::DataAnalyst,
            UserRole::Guest,
        ]
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::Type)]
#[sqlx(type_name = "user_status", rename_all = "snake_case")]
pub enum UserStatus {
    Active,
    Inactive,
    Locked,
    PendingVerification,
}

impl UserStatus {
    pub fn display_name(&self) -> &'static str {
        match self {
            UserStatus::Active => "Active",
            UserStatus::Inactive => "Inactive",
            UserStatus::Locked => "Locked",
            UserStatus::PendingVerification => "Pending Verification",
        }
    }
}

#[derive(Debug, Serialize, Deserialize, sqlx::FromRow)]
pub struct User {
    pub id: Uuid,
    pub email: String,
    #[serde(skip_serializing)]
    pub password_hash: String,
    pub first_name: String,
    pub last_name: String,
    pub role: UserRole,
    pub status: UserStatus,

    // Laboratory affiliation
    pub lab_affiliation: Option<String>,
    pub department: Option<String>,
    pub position: Option<String>,

    // Contact information
    pub phone: Option<String>,
    pub office_location: Option<String>,

    // Security fields
    pub email_verified: bool,
    pub failed_login_attempts: i32,
    pub locked_until: Option<DateTime<Utc>>,
    pub last_login: Option<DateTime<Utc>>,
    pub password_changed_at: DateTime<Utc>,

    // Audit fields
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub created_by: Option<Uuid>,
    pub metadata: serde_json::Value,
}

impl User {
    pub fn full_name(&self) -> String {
        format!("{} {}", self.first_name, self.last_name)
    }

    pub fn is_active(&self) -> bool {
        matches!(self.status, UserStatus::Active) && self.locked_until.is_none()
    }

    pub fn is_locked(&self) -> bool {
        matches!(self.status, UserStatus::Locked)
            || self
                .locked_until
                .map_or(false, |locked_until| locked_until > Utc::now())
    }

    pub fn can_login(&self) -> bool {
        self.is_active() && !self.is_locked() && self.email_verified
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UserSafeProfile {
    pub id: Uuid,
    pub email: String,
    pub first_name: String,
    pub last_name: String,
    pub role: UserRole,
    pub status: UserStatus,
    pub lab_affiliation: Option<String>,
    pub department: Option<String>,
    pub position: Option<String>,
    pub phone: Option<String>,
    pub office_location: Option<String>,
    pub email_verified: bool,
    pub last_login: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
}

impl From<User> for UserSafeProfile {
    fn from(user: User) -> Self {
        Self {
            id: user.id,
            email: user.email,
            first_name: user.first_name,
            last_name: user.last_name,
            role: user.role,
            status: user.status,
            lab_affiliation: user.lab_affiliation,
            department: user.department,
            position: user.position,
            phone: user.phone,
            office_location: user.office_location,
            email_verified: user.email_verified,
            last_login: user.last_login,
            created_at: user.created_at,
        }
    }
}

#[derive(Debug, Deserialize, Validate)]
pub struct CreateUserRequest {
    #[validate(email(message = "Invalid email format"))]
    pub email: String,

    #[validate(length(min = 8, message = "Password must be at least 8 characters"))]
    pub password: String,

    #[validate(length(min = 1, max = 100, message = "First name is required"))]
    pub first_name: String,

    #[validate(length(min = 1, max = 100, message = "Last name is required"))]
    pub last_name: String,

    pub role: UserRole,
    pub lab_affiliation: Option<String>,
    pub department: Option<String>,
    pub position: Option<String>,
    pub phone: Option<String>,
    pub office_location: Option<String>,
}

#[derive(Debug, Deserialize, Validate)]
pub struct UpdateUserRequest {
    #[validate(email(message = "Invalid email format"))]
    pub email: Option<String>,

    #[validate(length(min = 1, max = 100))]
    pub first_name: Option<String>,

    #[validate(length(min = 1, max = 100))]
    pub last_name: Option<String>,

    pub role: Option<UserRole>,
    pub status: Option<UserStatus>,
    pub lab_affiliation: Option<String>,
    pub department: Option<String>,
    pub position: Option<String>,
    pub phone: Option<String>,
    pub office_location: Option<String>,
}

#[derive(Debug, Deserialize, Validate)]
pub struct LoginRequest {
    #[validate(email(message = "Invalid email format"))]
    pub email: String,

    #[validate(length(min = 1, message = "Password is required"))]
    pub password: String,
}

#[derive(Debug, Deserialize, Validate)]
pub struct ChangePasswordRequest {
    #[validate(length(min = 1, message = "Current password is required"))]
    pub current_password: String,

    #[validate(length(min = 8, message = "New password must be at least 8 characters"))]
    pub new_password: String,
}

#[derive(Debug, Deserialize, Validate)]
pub struct ResetPasswordRequest {
    #[validate(email(message = "Invalid email format"))]
    pub email: String,
}

#[derive(Debug, Deserialize, Validate)]
pub struct ConfirmResetPasswordRequest {
    pub token: String,

    #[validate(length(min = 8, message = "Password must be at least 8 characters"))]
    pub new_password: String,
}

#[derive(Debug, Serialize, Deserialize, sqlx::FromRow)]
pub struct UserSession {
    pub id: Uuid,
    pub user_id: Uuid,
    pub token_hash: String,
    pub device_info: Option<String>,
    pub ip_address: Option<std::net::IpAddr>,
    pub expires_at: DateTime<Utc>,
    pub created_at: DateTime<Utc>,
    pub last_used_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct LoginResponse {
    pub user: UserSafeProfile,
    pub token: String,
    pub expires_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AuthClaims {
    pub sub: Uuid, // user_id
    pub email: String,
    pub role: UserRole,
    pub exp: i64,  // expiration timestamp
    pub iat: i64,  // issued at timestamp
    pub jti: Uuid, // JWT ID (session_id)
}

#[derive(Debug, Serialize, Deserialize, sqlx::FromRow)]
pub struct RolePermission {
    pub id: Uuid,
    pub role: UserRole,
    pub resource: String,
    pub action: String,
    pub granted: bool,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize, sqlx::FromRow)]
pub struct UserActivityLog {
    pub id: Uuid,
    pub user_id: Option<Uuid>,
    pub action: String,
    pub resource_type: Option<String>,
    pub resource_id: Option<Uuid>,
    pub ip_address: Option<std::net::IpAddr>,
    pub user_agent: Option<String>,
    pub details: Option<serde_json::Value>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UserListQuery {
    pub page: Option<u32>,
    pub per_page: Option<u32>,
    pub role: Option<UserRole>,
    pub status: Option<UserStatus>,
    pub lab_affiliation: Option<String>,
    pub search: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UserListResponse {
    pub users: Vec<UserSafeProfile>,
    pub total: u64,
    pub page: u32,
    pub per_page: u32,
    pub total_pages: u32,
}

pub struct UserManager {
    pool: PgPool,
}

impl UserManager {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn create_user(
        &self,
        request: CreateUserRequest,
        created_by: Option<Uuid>,
    ) -> Result<User, sqlx::Error> {
        // Hash password using Argon2
        let password_hash = argon2::hash_encoded(
            request.password.as_bytes(),
            b"lab_manager_salt", // In production, use a proper salt
            &argon2::Config::default(),
        )
        .map_err(|_| sqlx::Error::Protocol("Password hashing failed".into()))?;

        sqlx::query_as::<_, User>(
            r#"
            INSERT INTO users (
                email, password_hash, first_name, last_name, role,
                lab_affiliation, department, position, phone, office_location,
                created_by
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11)
            RETURNING id, email, password_hash, first_name, last_name, role, status,
                     lab_affiliation, department, position, phone, office_location,
                     email_verified, failed_login_attempts, locked_until, last_login,
                     password_changed_at, created_at, updated_at, created_by, metadata
            "#,
        )
        .bind(&request.email)
        .bind(&password_hash)
        .bind(&request.first_name)
        .bind(&request.last_name)
        .bind(&request.role)
        .bind(&request.lab_affiliation)
        .bind(&request.department)
        .bind(&request.position)
        .bind(&request.phone)
        .bind(&request.office_location)
        .bind(created_by)
        .fetch_one(&self.pool)
        .await
    }

    pub async fn get_user_by_id(&self, user_id: Uuid) -> Result<User, sqlx::Error> {
        sqlx::query_as::<_, User>(
            r#"
            SELECT id, email, password_hash, first_name, last_name, role, status,
                   lab_affiliation, department, position, phone, office_location,
                   email_verified, failed_login_attempts, locked_until, last_login,
                   password_changed_at, created_at, updated_at, created_by, metadata
            FROM users
            WHERE id = $1
            "#,
        )
        .bind(user_id)
        .fetch_one(&self.pool)
        .await
    }

    pub async fn get_user_by_email(&self, email: &str) -> Result<User, sqlx::Error> {
        sqlx::query_as::<_, User>(
            r#"
            SELECT id, email, password_hash, first_name, last_name, role, status,
                   lab_affiliation, department, position, phone, office_location,
                   email_verified, failed_login_attempts, locked_until, last_login,
                   password_changed_at, created_at, updated_at, created_by, metadata
            FROM users
            WHERE email = $1
            "#,
        )
        .bind(email)
        .fetch_one(&self.pool)
        .await
    }

    pub async fn update_user(
        &self,
        user_id: Uuid,
        request: UpdateUserRequest,
    ) -> Result<User, sqlx::Error> {
        let mut query_parts = Vec::new();
        let mut param_count = 1;

        if request.email.is_some() {
            query_parts.push(format!("email = ${}", param_count));
            param_count += 1;
        }
        if request.first_name.is_some() {
            query_parts.push(format!("first_name = ${}", param_count));
            param_count += 1;
        }
        if request.last_name.is_some() {
            query_parts.push(format!("last_name = ${}", param_count));
            param_count += 1;
        }
        if request.role.is_some() {
            query_parts.push(format!("role = ${}", param_count));
            param_count += 1;
        }
        if request.status.is_some() {
            query_parts.push(format!("status = ${}", param_count));
            param_count += 1;
        }
        if request.lab_affiliation.is_some() {
            query_parts.push(format!("lab_affiliation = ${}", param_count));
            param_count += 1;
        }
        if request.department.is_some() {
            query_parts.push(format!("department = ${}", param_count));
            param_count += 1;
        }
        if request.position.is_some() {
            query_parts.push(format!("position = ${}", param_count));
            param_count += 1;
        }
        if request.phone.is_some() {
            query_parts.push(format!("phone = ${}", param_count));
            param_count += 1;
        }
        if request.office_location.is_some() {
            query_parts.push(format!("office_location = ${}", param_count));
            param_count += 1;
        }

        if query_parts.is_empty() {
            return self.get_user_by_id(user_id).await;
        }

        let query = format!(
            r#"
            UPDATE users 
            SET {}, updated_at = NOW()
            WHERE id = ${}
            RETURNING id, email, password_hash, first_name, last_name, role, status,
                     lab_affiliation, department, position, phone, office_location,
                     email_verified, failed_login_attempts, locked_until, last_login,
                     password_changed_at, created_at, updated_at, created_by, metadata
            "#,
            query_parts.join(", "),
            param_count
        );

        let mut query_builder = sqlx::query_as::<_, User>(&query);

        if let Some(email) = request.email {
            query_builder = query_builder.bind(email);
        }
        if let Some(first_name) = request.first_name {
            query_builder = query_builder.bind(first_name);
        }
        if let Some(last_name) = request.last_name {
            query_builder = query_builder.bind(last_name);
        }
        if let Some(role) = request.role {
            query_builder = query_builder.bind(role);
        }
        if let Some(status) = request.status {
            query_builder = query_builder.bind(status);
        }
        if let Some(lab_affiliation) = request.lab_affiliation {
            query_builder = query_builder.bind(lab_affiliation);
        }
        if let Some(department) = request.department {
            query_builder = query_builder.bind(department);
        }
        if let Some(position) = request.position {
            query_builder = query_builder.bind(position);
        }
        if let Some(phone) = request.phone {
            query_builder = query_builder.bind(phone);
        }
        if let Some(office_location) = request.office_location {
            query_builder = query_builder.bind(office_location);
        }

        query_builder = query_builder.bind(user_id);

        query_builder.fetch_one(&self.pool).await
    }

    pub async fn list_users(&self, query: UserListQuery) -> Result<UserListResponse, sqlx::Error> {
        let page = query.page.unwrap_or(1).max(1);
        let per_page = query.per_page.unwrap_or(20).min(100);
        let offset = (page - 1) * per_page;

        let mut where_conditions = Vec::new();
        let mut param_count = 1;

        if query.role.is_some() {
            where_conditions.push(format!("role = ${}", param_count));
            param_count += 1;
        }
        if query.status.is_some() {
            where_conditions.push(format!("status = ${}", param_count));
            param_count += 1;
        }
        if query.lab_affiliation.is_some() {
            where_conditions.push(format!("lab_affiliation ILIKE ${}", param_count));
            param_count += 1;
        }
        if query.search.is_some() {
            where_conditions.push(format!(
                "(first_name ILIKE ${} OR last_name ILIKE ${} OR email ILIKE ${})",
                param_count, param_count, param_count
            ));
            param_count += 1;
        }

        let where_clause = if where_conditions.is_empty() {
            String::new()
        } else {
            format!("WHERE {}", where_conditions.join(" AND "))
        };

        // Count total records
        let count_query = format!("SELECT COUNT(*) as total FROM users {}", where_clause);

        let mut count_query_builder = sqlx::query_scalar::<_, i64>(&count_query);
        param_count = 1;

        if let Some(role) = &query.role {
            count_query_builder = count_query_builder.bind(role);
            param_count += 1;
        }
        if let Some(status) = &query.status {
            count_query_builder = count_query_builder.bind(status);
            param_count += 1;
        }
        if let Some(lab_affiliation) = &query.lab_affiliation {
            count_query_builder = count_query_builder.bind(format!("%{}%", lab_affiliation));
            param_count += 1;
        }
        if let Some(search) = &query.search {
            count_query_builder = count_query_builder.bind(format!("%{}%", search));
        }

        let total = count_query_builder.fetch_one(&self.pool).await? as u64;

        // Fetch users
        let users_query = format!(
            r#"
            SELECT id, email, password_hash, first_name, last_name, role, status,
                   lab_affiliation, department, position, phone, office_location,
                   email_verified, failed_login_attempts, locked_until, last_login,
                   password_changed_at, created_at, updated_at, created_by, metadata
            FROM users
            {}
            ORDER BY created_at DESC
            LIMIT ${} OFFSET ${}
            "#,
            where_clause,
            param_count,
            param_count + 1
        );

        let mut users_query_builder = sqlx::query_as::<_, User>(&users_query);
        param_count = 1;

        if let Some(role) = &query.role {
            users_query_builder = users_query_builder.bind(role);
            param_count += 1;
        }
        if let Some(status) = &query.status {
            users_query_builder = users_query_builder.bind(status);
            param_count += 1;
        }
        if let Some(lab_affiliation) = &query.lab_affiliation {
            users_query_builder = users_query_builder.bind(format!("%{}%", lab_affiliation));
            param_count += 1;
        }
        if let Some(search) = &query.search {
            users_query_builder = users_query_builder.bind(format!("%{}%", search));
            param_count += 1;
        }

        users_query_builder = users_query_builder
            .bind(per_page as i64)
            .bind(offset as i64);

        let users: Vec<User> = users_query_builder.fetch_all(&self.pool).await?;
        let users: Vec<UserSafeProfile> = users.into_iter().map(|u| u.into()).collect();

        let total_pages = (total as f64 / per_page as f64).ceil() as u32;

        Ok(UserListResponse {
            users,
            total,
            page,
            per_page,
            total_pages,
        })
    }

    pub async fn delete_user(&self, user_id: Uuid) -> Result<u64, sqlx::Error> {
        let result = sqlx::query("DELETE FROM users WHERE id = $1")
            .bind(user_id)
            .execute(&self.pool)
            .await?;

        Ok(result.rows_affected())
    }

    pub async fn verify_password(&self, password: &str, hash: &str) -> bool {
        argon2::verify_encoded(hash, password.as_bytes()).unwrap_or(false)
    }

    pub async fn increment_failed_login(&self, user_id: Uuid) -> Result<(), sqlx::Error> {
        sqlx::query(
            r#"
            UPDATE users 
            SET failed_login_attempts = failed_login_attempts + 1,
                locked_until = CASE 
                    WHEN failed_login_attempts >= 4 THEN NOW() + INTERVAL '30 minutes'
                    ELSE locked_until 
                END
            WHERE id = $1
            "#,
        )
        .bind(user_id)
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    pub async fn reset_failed_login(&self, user_id: Uuid) -> Result<(), sqlx::Error> {
        sqlx::query(
            r#"
            UPDATE users 
            SET failed_login_attempts = 0, locked_until = NULL, last_login = NOW()
            WHERE id = $1
            "#,
        )
        .bind(user_id)
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    pub async fn check_permission(
        &self,
        role: &UserRole,
        resource: &str,
        action: &str,
    ) -> Result<bool, sqlx::Error> {
        let result = sqlx::query_scalar::<_, bool>(
            "SELECT granted FROM role_permissions WHERE role = $1 AND resource = $2 AND action = $3"
        )
        .bind(role)
        .bind(resource)
        .bind(action)
        .fetch_optional(&self.pool)
        .await?;

        Ok(result.unwrap_or(false))
    }
}
