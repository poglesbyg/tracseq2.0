//! Basic Unit Tests for Authentication Service
//! 
//! Simple tests focusing on basic functionality without complex mocking

#[cfg(test)]
mod tests {
    use auth_service::{
        Config, 
        services::AuthServiceImpl,
        models::{UserRole, UserStatus},
    };
    use validator::Validate;

    #[test]
    fn test_config_validation() {
        let mut config = Config::test_config();
        
        // Test valid config
        assert!(config.validate().is_ok());
        
        // Test invalid JWT secret
        config.jwt.secret = "short".to_string();
        assert!(config.validate().is_err());
        
        // Reset and test invalid password length
        config = Config::test_config();
        config.security.password_min_length = 3;
        assert!(config.validate().is_err());
    }

    #[test]
    fn test_user_role_serialization() {
        let role = UserRole::LabAdministrator;
        let serialized = serde_json::to_string(&role).unwrap();
        assert_eq!(serialized, "\"lab_administrator\"");
        
        let deserialized: UserRole = serde_json::from_str(&serialized).unwrap();
        assert_eq!(deserialized, UserRole::LabAdministrator);
    }

    #[test]
    fn test_user_status_serialization() {
        let status = UserStatus::Active;
        let serialized = serde_json::to_string(&status).unwrap();
        assert_eq!(serialized, "\"active\"");
        
        let deserialized: UserStatus = serde_json::from_str(&serialized).unwrap();
        assert_eq!(deserialized, UserStatus::Active);
    }

    #[tokio::test]
    async fn test_password_hashing() {
        let config = Config::test_config();
        let db_pool = auth_service::DatabasePool {
            pool: sqlx::PgPool::connect_lazy("postgres://localhost/test").unwrap(),
        };
        
        let auth_service = AuthServiceImpl::new(db_pool, config).unwrap();
        
        // Test password hashing
        let password = "TestPassword123!";
        let hash_result = auth_service.hash_password(password);
        assert!(hash_result.is_ok());
        
        let hash = hash_result.unwrap();
        assert!(hash.starts_with("$argon2"));
        
        // Test password verification
        let verify_result = auth_service.verify_password(password, &hash);
        assert!(verify_result.is_ok());
        assert!(verify_result.unwrap());
        
        // Test wrong password
        let wrong_verify = auth_service.verify_password("WrongPassword", &hash);
        assert!(wrong_verify.is_ok());
        assert!(!wrong_verify.unwrap());
    }

    #[test]
    fn test_register_request_validation() {
        use auth_service::models::RegisterRequest;
        
        // Valid request
        let valid_request = RegisterRequest {
            first_name: "John".to_string(),
            last_name: "Doe".to_string(),
            email: "john@example.com".to_string(),
            password: "SecurePass123!".to_string(),
            department: None,
            position: None,
            lab_affiliation: None,
        };
        assert!(valid_request.validate().is_ok());
        
        // Invalid email
        let invalid_email = RegisterRequest {
            email: "not-an-email".to_string(),
            ..valid_request.clone()
        };
        assert!(invalid_email.validate().is_err());
        
        // Short password
        let short_password = RegisterRequest {
            password: "short".to_string(),
            ..valid_request.clone()
        };
        assert!(short_password.validate().is_err());
    }

    #[test]
    fn test_login_request_validation() {
        use auth_service::models::LoginRequest;
        
        // Valid request
        let valid_request = LoginRequest {
            email: "test@example.com".to_string(),
            password: "password123".to_string(),
            remember_me: Some(true),
        };
        assert!(valid_request.validate().is_ok());
        
        // Invalid email
        let invalid_email = LoginRequest {
            email: "".to_string(),
            ..valid_request.clone()
        };
        assert!(invalid_email.validate().is_err());
    }

    #[test]
    fn test_forgot_password_request_validation() {
        use auth_service::handlers::auth::ForgotPasswordRequest;
        
        // Valid request
        let valid_request = ForgotPasswordRequest {
            email: "test@example.com".to_string(),
        };
        assert!(valid_request.validate().is_ok());
        
        // Invalid email
        let invalid_request = ForgotPasswordRequest {
            email: "not-an-email".to_string(),
        };
        assert!(invalid_request.validate().is_err());
    }
}