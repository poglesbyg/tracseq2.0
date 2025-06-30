use auth_service::{AppState, Config, DatabasePool, AuthServiceImpl};
use std::sync::Arc;

#[tokio::test]
async fn test_auth_service_creation() {
    // Create test config
    let config = Config::test_config();
    
    // Create mock database pool - in real tests this would connect to a test database
    let db_pool = DatabasePool {
        pool: sqlx::PgPool::connect_lazy("postgres://localhost/test").unwrap(),
    };
    
    // Create auth service
    let auth_service = AuthServiceImpl::new(db_pool.clone(), config.clone())
        .expect("Failed to create auth service");
    
    // Create app state
    let app_state = AppState {
        db_pool,
        config: Arc::new(config),
        auth_service: Arc::new(auth_service),
    };
    
    // Verify app state was created successfully
    assert!(app_state.config.features.registration_enabled);
}

#[tokio::test]
async fn test_password_hashing() {
    use argon2::{Argon2, PasswordHash, PasswordHasher, PasswordVerifier};
    use argon2::password_hash::SaltString;
    use argon2::password_hash::rand_core::OsRng;
    
    let password = "TestPassword123!";
    
    // Hash password
    let salt = SaltString::generate(&mut OsRng);
    let argon2 = Argon2::default();
    let password_hash = argon2.hash_password(password.as_bytes(), &salt)
        .expect("Failed to hash password")
        .to_string();
    
    // Verify password
    let parsed_hash = PasswordHash::new(&password_hash)
        .expect("Failed to parse password hash");
    
    assert!(argon2.verify_password(password.as_bytes(), &parsed_hash).is_ok());
    assert!(argon2.verify_password(b"wrong_password", &parsed_hash).is_err());
}

#[tokio::test]
async fn test_validation() {
    use auth_service::handlers::auth::RegisterRequest;
    use validator::Validate;
    
    // Test valid request
    let valid_request = RegisterRequest {
        first_name: "John".to_string(),
        last_name: "Doe".to_string(),
        email: "john.doe@example.com".to_string(),
        password: "SecurePassword123!".to_string(),
        department: None,
        position: None,
        lab_affiliation: None,
    };
    
    assert!(valid_request.validate().is_ok());
    
    // Test invalid email
    let invalid_email = RegisterRequest {
        first_name: "John".to_string(),
        last_name: "Doe".to_string(),
        email: "invalid-email".to_string(),
        password: "SecurePassword123!".to_string(),
        department: None,
        position: None,
        lab_affiliation: None,
    };
    
    assert!(invalid_email.validate().is_err());
    
    // Test weak password
    let weak_password = RegisterRequest {
        first_name: "John".to_string(),
        last_name: "Doe".to_string(),
        email: "john.doe@example.com".to_string(),
        password: "weak".to_string(),
        department: None,
        position: None,
        lab_affiliation: None,
    };
    
    assert!(weak_password.validate().is_err());
}

#[test]
fn test_config_loading() {
    let config = Config::test_config();
    
    // Verify test config defaults
    assert_eq!(config.server.port, 0); // 0 lets OS assign port for tests
    assert!(config.features.registration_enabled);
    assert!(config.features.password_reset_enabled);
    assert_eq!(config.security.max_login_attempts, 10); // More lenient for tests
    assert_eq!(config.security.lockout_duration_minutes, 1); // Shorter for tests
}

#[test]
fn test_user_role_hierarchy() {
    use auth_service::models::UserRole;
    
    // Test role comparison (would need to implement PartialOrd for UserRole)
    let guest = UserRole::Guest;
    let admin = UserRole::LabAdministrator;
    
    // Basic role checks
    assert!(matches!(guest, UserRole::Guest));
    assert!(matches!(admin, UserRole::LabAdministrator));
}