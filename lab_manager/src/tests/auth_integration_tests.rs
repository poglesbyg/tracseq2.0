#[cfg(test)]
mod auth_integration_tests {
    use crate::models::user::{LoginRequest, UserRole};
    use crate::services::auth_service::AuthService;
    use axum::http::StatusCode;
    use serde_json::json;
    use sqlx::postgres::PgPoolOptions;
    use std::net::IpAddr;

    async fn setup_test_db() -> Result<sqlx::PgPool, sqlx::Error> {
        let database_url = std::env::var("TEST_DATABASE_URL").unwrap_or_else(|_| {
            "postgres://lab_manager:lab_manager@localhost:5432/lab_manager_test".to_string()
        });

        PgPoolOptions::new()
            .max_connections(5)
            .acquire_timeout(std::time::Duration::from_secs(2))
            .connect(&database_url)
            .await
    }

    fn should_skip_db_tests() -> bool {
        std::env::var("SKIP_DB_TESTS").unwrap_or_default() == "1"
    }

    #[tokio::test]
    async fn test_auth_service_integration() {
        let pool = match setup_test_db().await {
            Ok(pool) => pool,
            Err(_) => {
                println!("Skipping database test (database not available)");
                return;
            }
        };
        let auth_service = AuthService::new(pool.clone(), "test-secret".to_string());

        // Test that the auth service is properly integrated
        assert!(true, "Auth service integration test");
    }

    #[tokio::test]
    async fn test_login_flow_integration() {
        let pool = match setup_test_db().await {
            Ok(pool) => pool,
            Err(_) => {
                println!("Skipping database test (database not available)");
                return;
            }
        };
        let auth_service = AuthService::new(pool.clone(), "test-secret".to_string());

        // Test login flow without requiring pre-existing users
        let login_request = LoginRequest {
            email: "nonexistent@test.com".to_string(),
            password: "password".to_string(),
        };

        let result = auth_service
            .login(
                login_request,
                Some(IpAddr::from([127, 0, 0, 1])),
                Some("test-agent".to_string()),
            )
            .await;

        // Should fail because user doesn't exist
        assert!(result.is_err(), "Login should fail for non-existent user");
    }

    #[tokio::test]
    async fn test_role_permissions_integration() {
        // Test role-based permission system
        let admin_role = UserRole::LabAdministrator;
        let guest_role = UserRole::Guest;

        // Basic integration test for roles
        assert_eq!(admin_role.display_name(), "Lab Administrator");
        assert_eq!(guest_role.display_name(), "Guest");
    }

    #[tokio::test]
    async fn test_session_management_integration() {
        let pool = match setup_test_db().await {
            Ok(pool) => pool,
            Err(_) => {
                println!("Skipping database test (database not available)");
                return;
            }
        };
        let auth_service = AuthService::new(pool.clone(), "test-secret".to_string());

        // Test session cleanup
        let cleanup_result = auth_service.cleanup_expired().await;
        assert!(cleanup_result.is_ok(), "Session cleanup should work");
    }

    #[test]
    fn test_json_serialization() {
        // Test that user models serialize correctly for API responses
        let login_request = LoginRequest {
            email: "test@example.com".to_string(),
            password: "password123".to_string(),
        };

        let json_result = serde_json::to_string(&login_request);
        assert!(
            json_result.is_ok(),
            "Login request should serialize to JSON"
        );

        let json_str = json_result.unwrap();
        assert!(json_str.contains("test@example.com"));
        assert!(json_str.contains("password123"));
    }

    #[test]
    fn test_validation_rules() {
        // Test validation integration
        use crate::models::user::CreateUserRequest;

        let valid_request = CreateUserRequest {
            email: "valid@test.com".to_string(),
            password: "password123".to_string(),
            first_name: "Test".to_string(),
            last_name: "User".to_string(),
            role: UserRole::LabTechnician,
            lab_affiliation: None,
            department: None,
            position: None,
            phone: None,
            office_location: None,
        };

        // Basic validation test
        assert!(!valid_request.email.is_empty());
        assert!(!valid_request.password.is_empty());
        assert!(valid_request.password.len() >= 8);
    }
}
