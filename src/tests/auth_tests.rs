#[cfg(test)]
mod auth_tests {
    use crate::models::user::{
        AuthClaims, ChangePasswordRequest, ConfirmResetPasswordRequest, CreateUserRequest,
        LoginRequest, ResetPasswordRequest, UpdateUserRequest, User, UserListQuery, UserManager,
        UserRole, UserSafeProfile, UserStatus,
    };
    use crate::services::auth_service::AuthService;
    use argon2::password_hash::{rand_core::OsRng, PasswordHasher, SaltString};
    use argon2::Argon2;
    use chrono::{Duration, Utc};
    use jsonwebtoken::{decode, Algorithm, DecodingKey, Validation};
    use serde_json::json;
    use sqlx::postgres::PgPoolOptions;
    use std::net::IpAddr;
    use uuid::Uuid;
    use validator::Validate;

    // Test database setup helper
    async fn setup_test_db() -> sqlx::PgPool {
        let database_url = std::env::var("TEST_DATABASE_URL").unwrap_or_else(|_| {
            "postgres://lab_manager:lab_manager@localhost:5432/lab_manager_test".to_string()
        });

        PgPoolOptions::new()
            .max_connections(5)
            .connect(&database_url)
            .await
            .expect("Failed to connect to test database")
    }

    // Test helper to create a test user
    async fn create_test_user(pool: &sqlx::PgPool) -> User {
        let user_manager = UserManager::new(pool.clone());
        let salt = SaltString::generate(&mut OsRng);
        let argon2 = Argon2::default();
        let password_hash = argon2
            .hash_password(b"test123", &salt)
            .expect("Failed to hash password")
            .to_string();

        let user_id = Uuid::new_v4();
        sqlx::query(
            r#"
            INSERT INTO users (
                id, email, password_hash, first_name, last_name, role, status,
                email_verified, created_at, updated_at, password_changed_at
            ) VALUES (
                $1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11
            )
            "#,
        )
        .bind(user_id)
        .bind("test@lab.local")
        .bind(&password_hash)
        .bind("Test")
        .bind("User")
        .bind(&UserRole::LabTechnician)
        .bind(&UserStatus::Active)
        .bind(true)
        .bind(Utc::now())
        .bind(Utc::now())
        .bind(Utc::now())
        .execute(pool)
        .await
        .expect("Failed to create test user");

        user_manager
            .get_user_by_id(user_id)
            .await
            .expect("Failed to get test user")
    }

    #[tokio::test]
    async fn test_user_creation() {
        let pool = setup_test_db().await;
        let user_manager = UserManager::new(pool.clone());

        let create_request = CreateUserRequest {
            email: "newuser@lab.local".to_string(),
            password: "password123".to_string(),
            first_name: "New".to_string(),
            last_name: "User".to_string(),
            role: UserRole::LabTechnician,
            lab_affiliation: Some("Test Lab".to_string()),
            department: Some("Testing".to_string()),
            position: Some("Tester".to_string()),
            phone: None,
            office_location: None,
        };

        let result = user_manager.create_user(create_request, None).await;
        assert!(result.is_ok(), "User creation should succeed");

        let user = result.unwrap();
        assert_eq!(user.email, "newuser@lab.local");
        assert_eq!(user.first_name, "New");
        assert_eq!(user.last_name, "User");
        assert_eq!(user.role, UserRole::LabTechnician);
        assert!(user.email_verified);

        // Cleanup
        sqlx::query("DELETE FROM users WHERE email = 'newuser@lab.local'")
            .execute(&pool)
            .await
            .expect("Failed to cleanup test user");
    }

    #[tokio::test]
    async fn test_password_hashing() {
        let pool = setup_test_db().await;
        let user_manager = UserManager::new(pool.clone());

        let password = "test123";
        let salt = SaltString::generate(&mut OsRng);
        let argon2 = Argon2::default();
        let password_hash = argon2
            .hash_password(password.as_bytes(), &salt)
            .expect("Failed to hash password")
            .to_string();

        let is_valid = user_manager.verify_password(password, &password_hash).await;
        assert!(is_valid, "Password verification should succeed");

        let is_invalid = user_manager
            .verify_password("wrongpassword", &password_hash)
            .await;
        assert!(!is_invalid, "Wrong password should fail verification");
    }

    #[test]
    fn test_user_roles() {
        let admin_role = UserRole::LabAdministrator;
        let tech_role = UserRole::LabTechnician;
        let guest_role = UserRole::Guest;

        assert_eq!(admin_role.display_name(), "Lab Administrator");
        assert_eq!(tech_role.display_name(), "Lab Technician");
        assert_eq!(guest_role.display_name(), "Guest");

        let all_roles = UserRole::all_roles();
        assert_eq!(all_roles.len(), 6);
        assert!(all_roles.contains(&admin_role));
    }

    #[test]
    fn test_auth_claims() {
        let claims = AuthClaims {
            sub: Uuid::new_v4(),
            email: "test@lab.local".to_string(),
            role: UserRole::LabAdministrator,
            exp: (Utc::now() + Duration::hours(1)).timestamp(),
            iat: Utc::now().timestamp(),
            jti: Uuid::new_v4(),
        };

        assert_eq!(claims.email, "test@lab.local");
        assert_eq!(claims.role, UserRole::LabAdministrator);
        assert!(claims.exp > claims.iat);
    }

    #[tokio::test]
    async fn test_user_roles_and_permissions() {
        // Test all user roles
        let admin_role = UserRole::LabAdministrator;
        let pi_role = UserRole::PrincipalInvestigator;
        let tech_role = UserRole::LabTechnician;
        let scientist_role = UserRole::ResearchScientist;
        let analyst_role = UserRole::DataAnalyst;
        let guest_role = UserRole::Guest;

        // Test role display names
        assert_eq!(admin_role.display_name(), "Lab Administrator");
        assert_eq!(pi_role.display_name(), "Principal Investigator");
        assert_eq!(tech_role.display_name(), "Lab Technician");
        assert_eq!(scientist_role.display_name(), "Research Scientist");
        assert_eq!(analyst_role.display_name(), "Data Analyst");
        assert_eq!(guest_role.display_name(), "Guest");

        // Test all roles are available
        let all_roles = UserRole::all_roles();
        assert_eq!(all_roles.len(), 6);
        assert!(all_roles.contains(&admin_role));
        assert!(all_roles.contains(&guest_role));
    }

    #[tokio::test]
    async fn test_user_status_validation() {
        let pool = setup_test_db().await;
        let user = create_test_user(&pool).await;

        // Test active user can login
        assert!(user.can_login(), "Active user should be able to login");
        assert!(user.is_active(), "User status should be active");
        assert!(!user.is_locked(), "User should not be locked");

        // Test status display names
        assert_eq!(UserStatus::Active.display_name(), "Active");
        assert_eq!(UserStatus::Inactive.display_name(), "Inactive");
        assert_eq!(UserStatus::Locked.display_name(), "Locked");
        assert_eq!(
            UserStatus::PendingVerification.display_name(),
            "Pending Verification"
        );

        // Cleanup
        sqlx::query("DELETE FROM users WHERE email = 'test@lab.local'")
            .execute(&pool)
            .await
            .expect("Failed to cleanup test user");
    }

    #[tokio::test]
    async fn test_auth_service_login() {
        let pool = setup_test_db().await;
        let user = create_test_user(&pool).await;
        let auth_service = AuthService::new(pool.clone(), "test-secret".to_string());

        let login_request = LoginRequest {
            email: "test@lab.local".to_string(),
            password: "test123".to_string(),
        };

        let result = auth_service
            .login(
                login_request,
                Some(IpAddr::from([127, 0, 0, 1])),
                Some("test-agent".to_string()),
            )
            .await;

        assert!(
            result.is_ok(),
            "Login should succeed with correct credentials"
        );

        let login_response = result.unwrap();
        assert_eq!(login_response.user.email, "test@lab.local");
        assert!(
            !login_response.token.is_empty(),
            "JWT token should be generated"
        );
        assert!(
            login_response.expires_at > Utc::now(),
            "Token should have future expiration"
        );

        // Cleanup
        sqlx::query("DELETE FROM users WHERE email = 'test@lab.local'")
            .execute(&pool)
            .await
            .expect("Failed to cleanup test user");
    }

    #[tokio::test]
    async fn test_auth_service_login_failure() {
        let pool = setup_test_db().await;
        let user = create_test_user(&pool).await;
        let auth_service = AuthService::new(pool.clone(), "test-secret".to_string());

        // Test with wrong password
        let login_request = LoginRequest {
            email: "test@lab.local".to_string(),
            password: "wrongpassword".to_string(),
        };

        let result = auth_service
            .login(
                login_request,
                Some(IpAddr::from([127, 0, 0, 1])),
                Some("test-agent".to_string()),
            )
            .await;

        assert!(result.is_err(), "Login should fail with incorrect password");

        // Test with non-existent user
        let login_request = LoginRequest {
            email: "nonexistent@lab.local".to_string(),
            password: "test123".to_string(),
        };

        let result = auth_service
            .login(
                login_request,
                Some(IpAddr::from([127, 0, 0, 1])),
                Some("test-agent".to_string()),
            )
            .await;

        assert!(result.is_err(), "Login should fail with non-existent user");

        // Cleanup
        sqlx::query("DELETE FROM users WHERE email = 'test@lab.local'")
            .execute(&pool)
            .await
            .expect("Failed to cleanup test user");
    }

    #[tokio::test]
    async fn test_jwt_token_validation() {
        let pool = setup_test_db().await;
        let user = create_test_user(&pool).await;
        let auth_service = AuthService::new(pool.clone(), "test-secret".to_string());

        // Login to get a token
        let login_request = LoginRequest {
            email: "test@lab.local".to_string(),
            password: "test123".to_string(),
        };

        let login_response = auth_service
            .login(
                login_request,
                Some(IpAddr::from([127, 0, 0, 1])),
                Some("test-agent".to_string()),
            )
            .await
            .expect("Login should succeed");

        // Verify the token
        let verification_result = auth_service.verify_token(&login_response.token).await;
        assert!(
            verification_result.is_ok(),
            "Token verification should succeed"
        );

        let (verified_user, session) = verification_result.unwrap();
        assert_eq!(verified_user.email, "test@lab.local");
        assert_eq!(verified_user.id, user.id);
        assert!(
            session.expires_at > Utc::now(),
            "Session should not be expired"
        );

        // Test with invalid token
        let invalid_result = auth_service.verify_token("invalid.token.here").await;
        assert!(
            invalid_result.is_err(),
            "Invalid token should fail verification"
        );

        // Cleanup
        sqlx::query("DELETE FROM users WHERE email = 'test@lab.local'")
            .execute(&pool)
            .await
            .expect("Failed to cleanup test user");
    }

    #[tokio::test]
    async fn test_password_change() {
        let pool = setup_test_db().await;
        let user = create_test_user(&pool).await;
        let auth_service = AuthService::new(pool.clone(), "test-secret".to_string());

        let change_request = ChangePasswordRequest {
            current_password: "test123".to_string(),
            new_password: "newpassword123".to_string(),
        };

        let result = auth_service.change_password(user.id, change_request).await;
        assert!(result.is_ok(), "Password change should succeed");

        // Test login with new password
        let login_request = LoginRequest {
            email: "test@lab.local".to_string(),
            password: "newpassword123".to_string(),
        };

        let login_result = auth_service
            .login(
                login_request,
                Some(IpAddr::from([127, 0, 0, 1])),
                Some("test-agent".to_string()),
            )
            .await;

        assert!(
            login_result.is_ok(),
            "Login should succeed with new password"
        );

        // Test login with old password fails
        let old_login_request = LoginRequest {
            email: "test@lab.local".to_string(),
            password: "test123".to_string(),
        };

        let old_login_result = auth_service
            .login(
                old_login_request,
                Some(IpAddr::from([127, 0, 0, 1])),
                Some("test-agent".to_string()),
            )
            .await;

        assert!(
            old_login_result.is_err(),
            "Login should fail with old password"
        );

        // Cleanup
        sqlx::query("DELETE FROM users WHERE email = 'test@lab.local'")
            .execute(&pool)
            .await
            .expect("Failed to cleanup test user");
    }

    #[tokio::test]
    async fn test_session_management() {
        let pool = setup_test_db().await;
        let user = create_test_user(&pool).await;
        let auth_service = AuthService::new(pool.clone(), "test-secret".to_string());

        // Login to create a session
        let login_request = LoginRequest {
            email: "test@lab.local".to_string(),
            password: "test123".to_string(),
        };

        let login_response = auth_service
            .login(
                login_request,
                Some(IpAddr::from([127, 0, 0, 1])),
                Some("test-agent".to_string()),
            )
            .await
            .expect("Login should succeed");

        // Get user sessions
        let sessions = auth_service.get_user_sessions(user.id).await;
        assert!(sessions.is_ok(), "Getting sessions should succeed");
        assert!(
            !sessions.unwrap().is_empty(),
            "Should have at least one session"
        );

        // Verify token to get session ID
        let (_, session) = auth_service
            .verify_token(&login_response.token)
            .await
            .expect("Token verification should succeed");

        // Logout/revoke session
        let logout_result = auth_service.logout(session.id, Some(user.id)).await;
        assert!(logout_result.is_ok(), "Logout should succeed");

        // Verify token should now fail
        let verify_result = auth_service.verify_token(&login_response.token).await;
        assert!(
            verify_result.is_err(),
            "Token should be invalid after logout"
        );

        // Cleanup
        sqlx::query("DELETE FROM users WHERE email = 'test@lab.local'")
            .execute(&pool)
            .await
            .expect("Failed to cleanup test user");
    }

    #[tokio::test]
    async fn test_failed_login_attempts() {
        let pool = setup_test_db().await;
        let user = create_test_user(&pool).await;
        let user_manager = UserManager::new(pool.clone());

        // Test increment failed login
        let result = user_manager.increment_failed_login(user.id).await;
        assert!(result.is_ok(), "Incrementing failed login should succeed");

        // Test reset failed login
        let reset_result = user_manager.reset_failed_login(user.id).await;
        assert!(
            reset_result.is_ok(),
            "Resetting failed login should succeed"
        );

        // Cleanup
        sqlx::query("DELETE FROM users WHERE email = 'test@lab.local'")
            .execute(&pool)
            .await
            .expect("Failed to cleanup test user");
    }

    #[tokio::test]
    async fn test_user_list_and_filtering() {
        let pool = setup_test_db().await;
        let user_manager = UserManager::new(pool.clone());

        // Create multiple test users
        let users_to_create = vec![
            (
                "user1@lab.local",
                UserRole::LabAdministrator,
                UserStatus::Active,
            ),
            (
                "user2@lab.local",
                UserRole::LabTechnician,
                UserStatus::Active,
            ),
            ("user3@lab.local", UserRole::Guest, UserStatus::Inactive),
        ];

        let mut created_user_ids = Vec::new();

        for (email, role, status) in users_to_create {
            let create_request = CreateUserRequest {
                email: email.to_string(),
                password: "password123".to_string(),
                first_name: "Test".to_string(),
                last_name: "User".to_string(),
                role,
                lab_affiliation: Some("Test Lab".to_string()),
                department: None,
                position: None,
                phone: None,
                office_location: None,
            };

            let created_user = user_manager
                .create_user(create_request, None)
                .await
                .expect("User creation should succeed");

            created_user_ids.push(created_user.id);

            // Update status if needed
            if status != UserStatus::Active {
                sqlx::query("UPDATE users SET status = $1 WHERE id = $2")
                    .bind(&status)
                    .bind(created_user.id)
                    .execute(&pool)
                    .await
                    .expect("Status update should succeed");
            }
        }

        // Test user listing
        let query = crate::models::user::UserListQuery {
            page: Some(1),
            per_page: Some(10),
            role: None,
            status: None,
            lab_affiliation: None,
            search: None,
        };

        let list_result = user_manager.list_users(query).await;
        assert!(list_result.is_ok(), "User listing should succeed");

        let user_list = list_result.unwrap();
        assert!(user_list.users.len() >= 3, "Should have at least 3 users");

        // Test filtering by role
        let role_query = crate::models::user::UserListQuery {
            page: Some(1),
            per_page: Some(10),
            role: Some(UserRole::LabTechnician),
            status: None,
            lab_affiliation: None,
            search: None,
        };

        let role_result = user_manager.list_users(role_query).await;
        assert!(role_result.is_ok(), "Role filtering should succeed");

        // Cleanup
        for user_id in created_user_ids {
            sqlx::query("DELETE FROM users WHERE id = $1")
                .bind(user_id)
                .execute(&pool)
                .await
                .expect("User cleanup should succeed");
        }
    }

    // New comprehensive tests start here

    #[test]
    fn test_user_role_serialization() {
        let role = UserRole::PrincipalInvestigator;
        let serialized = serde_json::to_string(&role).expect("Should serialize");
        let deserialized: UserRole = serde_json::from_str(&serialized).expect("Should deserialize");
        assert_eq!(role, deserialized);
    }

    #[test]
    fn test_user_role_descriptions() {
        assert!(UserRole::LabAdministrator.description().contains("Manage"));
        assert!(UserRole::LabTechnician.description().contains("Perform"));
        assert!(UserRole::Guest.description().contains("Limited"));
    }

    #[test]
    fn test_user_status_display() {
        assert_eq!(UserStatus::Active.display_name(), "Active");
        assert_eq!(UserStatus::Inactive.display_name(), "Inactive");
        assert_eq!(UserStatus::Locked.display_name(), "Locked");
        assert_eq!(
            UserStatus::PendingVerification.display_name(),
            "Pending Verification"
        );
    }

    #[test]
    fn test_create_user_request_validation() {
        // Valid request
        let valid_request = CreateUserRequest {
            email: "valid@test.com".to_string(),
            password: "validpassword123".to_string(),
            first_name: "Valid".to_string(),
            last_name: "User".to_string(),
            role: UserRole::LabTechnician,
            lab_affiliation: None,
            department: None,
            position: None,
            phone: None,
            office_location: None,
        };
        assert!(valid_request.validate().is_ok());

        // Invalid email
        let invalid_email = CreateUserRequest {
            email: "invalid-email".to_string(),
            password: "validpassword123".to_string(),
            first_name: "Test".to_string(),
            last_name: "User".to_string(),
            role: UserRole::LabTechnician,
            lab_affiliation: None,
            department: None,
            position: None,
            phone: None,
            office_location: None,
        };
        assert!(invalid_email.validate().is_err());

        // Password too short
        let short_password = CreateUserRequest {
            email: "valid@test.com".to_string(),
            password: "short".to_string(),
            first_name: "Test".to_string(),
            last_name: "User".to_string(),
            role: UserRole::LabTechnician,
            lab_affiliation: None,
            department: None,
            position: None,
            phone: None,
            office_location: None,
        };
        assert!(short_password.validate().is_err());

        // Empty first name
        let empty_first_name = CreateUserRequest {
            email: "valid@test.com".to_string(),
            password: "validpassword123".to_string(),
            first_name: "".to_string(),
            last_name: "User".to_string(),
            role: UserRole::LabTechnician,
            lab_affiliation: None,
            department: None,
            position: None,
            phone: None,
            office_location: None,
        };
        assert!(empty_first_name.validate().is_err());
    }

    #[test]
    fn test_login_request_validation() {
        // Valid login
        let valid_login = LoginRequest {
            email: "test@example.com".to_string(),
            password: "password123".to_string(),
        };
        assert!(valid_login.validate().is_ok());

        // Invalid email
        let invalid_email = LoginRequest {
            email: "not-an-email".to_string(),
            password: "password123".to_string(),
        };
        assert!(invalid_email.validate().is_err());

        // Empty password
        let empty_password = LoginRequest {
            email: "test@example.com".to_string(),
            password: "".to_string(),
        };
        assert!(empty_password.validate().is_err());
    }

    #[test]
    fn test_change_password_request_validation() {
        // Valid request
        let valid_request = ChangePasswordRequest {
            current_password: "oldpassword".to_string(),
            new_password: "newpassword123".to_string(),
        };
        assert!(valid_request.validate().is_ok());

        // New password too short
        let short_password = ChangePasswordRequest {
            current_password: "oldpassword".to_string(),
            new_password: "short".to_string(),
        };
        assert!(short_password.validate().is_err());

        // Empty current password
        let empty_current = ChangePasswordRequest {
            current_password: "".to_string(),
            new_password: "newpassword123".to_string(),
        };
        assert!(empty_current.validate().is_err());
    }

    #[test]
    fn test_user_safe_profile_conversion() {
        let user = User {
            id: Uuid::new_v4(),
            email: "test@example.com".to_string(),
            password_hash: "hashed_password".to_string(),
            first_name: "Test".to_string(),
            last_name: "User".to_string(),
            role: UserRole::LabTechnician,
            status: UserStatus::Active,
            lab_affiliation: Some("Test Lab".to_string()),
            department: Some("Testing".to_string()),
            position: Some("Tester".to_string()),
            phone: None,
            office_location: None,
            email_verified: true,
            failed_login_attempts: 0,
            locked_until: None,
            last_login: None,
            password_changed_at: Utc::now(),
            created_at: Utc::now(),
            updated_at: Utc::now(),
            created_by: None,
            metadata: serde_json::json!({}),
        };

        let safe_profile: UserSafeProfile = user.into();
        assert_eq!(safe_profile.email, "test@example.com");
        assert_eq!(safe_profile.first_name, "Test");
        assert_eq!(safe_profile.last_name, "User");
        assert_eq!(safe_profile.role, UserRole::LabTechnician);
        // Password hash should not be in safe profile
    }

    #[test]
    fn test_user_methods() {
        let user = User {
            id: Uuid::new_v4(),
            email: "test@example.com".to_string(),
            password_hash: "hashed_password".to_string(),
            first_name: "John".to_string(),
            last_name: "Doe".to_string(),
            role: UserRole::LabTechnician,
            status: UserStatus::Active,
            lab_affiliation: None,
            department: None,
            position: None,
            phone: None,
            office_location: None,
            email_verified: true,
            failed_login_attempts: 0,
            locked_until: None,
            last_login: None,
            password_changed_at: Utc::now(),
            created_at: Utc::now(),
            updated_at: Utc::now(),
            created_by: None,
            metadata: serde_json::json!({}),
        };

        assert_eq!(user.full_name(), "John Doe");
        assert!(user.is_active());
        assert!(!user.is_locked());
        assert!(user.can_login());
    }

    #[test]
    fn test_locked_user() {
        let locked_user = User {
            id: Uuid::new_v4(),
            email: "locked@example.com".to_string(),
            password_hash: "hashed_password".to_string(),
            first_name: "Locked".to_string(),
            last_name: "User".to_string(),
            role: UserRole::LabTechnician,
            status: UserStatus::Active,
            lab_affiliation: None,
            department: None,
            position: None,
            phone: None,
            office_location: None,
            email_verified: true,
            failed_login_attempts: 5,
            locked_until: Some(Utc::now() + Duration::minutes(30)),
            last_login: None,
            password_changed_at: Utc::now(),
            created_at: Utc::now(),
            updated_at: Utc::now(),
            created_by: None,
            metadata: serde_json::json!({}),
        };

        assert!(locked_user.is_locked());
        assert!(!locked_user.can_login());
    }

    #[test]
    fn test_inactive_user() {
        let inactive_user = User {
            id: Uuid::new_v4(),
            email: "inactive@example.com".to_string(),
            password_hash: "hashed_password".to_string(),
            first_name: "Inactive".to_string(),
            last_name: "User".to_string(),
            role: UserRole::LabTechnician,
            status: UserStatus::Inactive,
            lab_affiliation: None,
            department: None,
            position: None,
            phone: None,
            office_location: None,
            email_verified: true,
            failed_login_attempts: 0,
            locked_until: None,
            last_login: None,
            password_changed_at: Utc::now(),
            created_at: Utc::now(),
            updated_at: Utc::now(),
            created_by: None,
            metadata: serde_json::json!({}),
        };

        assert!(!inactive_user.is_active());
        assert!(!inactive_user.can_login());
    }

    #[test]
    fn test_unverified_user() {
        let unverified_user = User {
            id: Uuid::new_v4(),
            email: "unverified@example.com".to_string(),
            password_hash: "hashed_password".to_string(),
            first_name: "Unverified".to_string(),
            last_name: "User".to_string(),
            role: UserRole::LabTechnician,
            status: UserStatus::Active,
            lab_affiliation: None,
            department: None,
            position: None,
            phone: None,
            office_location: None,
            email_verified: false,
            failed_login_attempts: 0,
            locked_until: None,
            last_login: None,
            password_changed_at: Utc::now(),
            created_at: Utc::now(),
            updated_at: Utc::now(),
            created_by: None,
            metadata: serde_json::json!({}),
        };

        assert!(!unverified_user.can_login());
    }

    #[test]
    fn test_auth_claims_serialization() {
        let claims = AuthClaims {
            sub: Uuid::new_v4(),
            email: "test@example.com".to_string(),
            role: UserRole::DataAnalyst,
            exp: (Utc::now() + Duration::hours(2)).timestamp(),
            iat: Utc::now().timestamp(),
            jti: Uuid::new_v4(),
        };

        let serialized = serde_json::to_string(&claims).expect("Should serialize");
        let deserialized: AuthClaims =
            serde_json::from_str(&serialized).expect("Should deserialize");

        assert_eq!(claims.sub, deserialized.sub);
        assert_eq!(claims.email, deserialized.email);
        assert_eq!(claims.role, deserialized.role);
        assert_eq!(claims.exp, deserialized.exp);
        assert_eq!(claims.iat, deserialized.iat);
        assert_eq!(claims.jti, deserialized.jti);
    }

    #[test]
    fn test_user_list_query_defaults() {
        let query = UserListQuery {
            page: None,
            per_page: None,
            role: None,
            status: None,
            lab_affiliation: None,
            search: None,
        };

        // Test that all optional fields can be None
        assert!(query.page.is_none());
        assert!(query.per_page.is_none());
        assert!(query.role.is_none());
        assert!(query.status.is_none());
        assert!(query.lab_affiliation.is_none());
        assert!(query.search.is_none());
    }

    #[test]
    fn test_password_security_requirements() {
        let test_passwords = vec![
            ("", false),                     // Empty
            ("123", false),                  // Too short
            ("1234567", false),              // Still too short
            ("12345678", true),              // Minimum length
            ("password123", true),           // Valid
            ("MySecurePassword2024!", true), // Strong
        ];

        for (password, should_be_valid) in test_passwords {
            let request = CreateUserRequest {
                email: "test@example.com".to_string(),
                password: password.to_string(),
                first_name: "Test".to_string(),
                last_name: "User".to_string(),
                role: UserRole::LabTechnician,
                lab_affiliation: None,
                department: None,
                position: None,
                phone: None,
                office_location: None,
            };

            let validation_result = request.validate();
            if should_be_valid {
                assert!(
                    validation_result.is_ok(),
                    "Password '{}' should be valid",
                    password
                );
            } else {
                assert!(
                    validation_result.is_err(),
                    "Password '{}' should be invalid",
                    password
                );
            }
        }
    }

    #[test]
    fn test_email_validation() {
        let test_emails = vec![
            ("test@example.com", true),
            ("user.name@domain.co.uk", true),
            ("test+tag@example.org", true),
            ("", false),
            ("not-an-email", false),
            ("@domain.com", false),
            ("user@", false),
            ("user name@domain.com", false),
        ];

        for (email, should_be_valid) in test_emails {
            let request = LoginRequest {
                email: email.to_string(),
                password: "password123".to_string(),
            };

            let validation_result = request.validate();
            if should_be_valid {
                assert!(
                    validation_result.is_ok(),
                    "Email '{}' should be valid",
                    email
                );
            } else {
                assert!(
                    validation_result.is_err(),
                    "Email '{}' should be invalid",
                    email
                );
            }
        }
    }

    #[test]
    fn test_role_permission_levels() {
        // Test role hierarchy concepts
        let admin = UserRole::LabAdministrator;
        let pi = UserRole::PrincipalInvestigator;
        let scientist = UserRole::ResearchScientist;
        let technician = UserRole::LabTechnician;
        let analyst = UserRole::DataAnalyst;
        let guest = UserRole::Guest;

        // All roles should have unique display names
        let display_names: std::collections::HashSet<_> = UserRole::all_roles()
            .iter()
            .map(|r| r.display_name())
            .collect();
        assert_eq!(
            display_names.len(),
            6,
            "All roles should have unique display names"
        );

        // All roles should have descriptions
        for role in UserRole::all_roles() {
            assert!(
                !role.description().is_empty(),
                "Role {:?} should have a description",
                role
            );
        }
    }

    #[tokio::test]
    async fn test_password_hash_consistency() {
        let pool = setup_test_db().await;
        let user_manager = UserManager::new(pool.clone());

        let password = "consistent_password_test";

        // Hash the same password multiple times
        let salt1 = SaltString::generate(&mut OsRng);
        let salt2 = SaltString::generate(&mut OsRng);
        let argon2 = Argon2::default();

        let hash1 = argon2
            .hash_password(password.as_bytes(), &salt1)
            .unwrap()
            .to_string();
        let hash2 = argon2
            .hash_password(password.as_bytes(), &salt2)
            .unwrap()
            .to_string();

        // Hashes should be different due to different salts
        assert_ne!(
            hash1, hash2,
            "Different salts should produce different hashes"
        );

        // But both should verify against the same password
        assert!(user_manager.verify_password(password, &hash1).await);
        assert!(user_manager.verify_password(password, &hash2).await);

        // And both should fail with wrong password
        assert!(!user_manager.verify_password("wrong_password", &hash1).await);
        assert!(!user_manager.verify_password("wrong_password", &hash2).await);
    }

    #[test]
    fn test_request_validation_edge_cases() {
        // Test update request with all None values
        let empty_update = UpdateUserRequest {
            email: None,
            first_name: None,
            last_name: None,
            role: None,
            status: None,
            lab_affiliation: None,
            department: None,
            position: None,
            phone: None,
            office_location: None,
        };
        assert!(empty_update.validate().is_ok());

        // Test reset password request
        let reset_request = ResetPasswordRequest {
            email: "valid@example.com".to_string(),
        };
        assert!(reset_request.validate().is_ok());

        let invalid_reset = ResetPasswordRequest {
            email: "invalid-email".to_string(),
        };
        assert!(invalid_reset.validate().is_err());

        // Test confirm reset password request
        let confirm_reset = ConfirmResetPasswordRequest {
            token: "some-token".to_string(),
            new_password: "newpassword123".to_string(),
        };
        assert!(confirm_reset.validate().is_ok());

        let weak_password_reset = ConfirmResetPasswordRequest {
            token: "some-token".to_string(),
            new_password: "weak".to_string(),
        };
        assert!(weak_password_reset.validate().is_err());
    }
}
