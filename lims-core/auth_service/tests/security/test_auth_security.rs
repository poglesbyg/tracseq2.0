//! Authentication Security Tests
//! 
//! This module tests security aspects of the authentication service:
//! - SQL injection prevention
//! - XSS prevention
//! - Password security
//! - Rate limiting and brute force protection
//! - Token security and tampering
//! - Authorization and access control

use auth_service::{AppState, Config, DatabasePool, AuthServiceImpl, create_router};
use axum_test::TestServer;
use serde_json::json;
use std::sync::Arc;
use uuid::Uuid;
use regex::Regex;

/// Create a test server with security-focused configuration
async fn create_secure_test_server() -> anyhow::Result<TestServer> {
    let database_url = std::env::var("TEST_DATABASE_URL")
        .unwrap_or_else(|_| "postgres://postgres:postgres@localhost:5432/auth_service_test".to_string());
    
    let pool = sqlx::postgres::PgPoolOptions::new()
        .max_connections(5)
        .connect(&database_url)
        .await?;
    
    // Run migrations
    sqlx::migrate!("./migrations")
        .run(&pool)
        .await?;
    
    let db_pool = DatabasePool { pool };
    
    // Create config with strict security settings
    let mut config = Config::test_config();
    config.security.max_login_attempts = 3;
    config.security.lockout_duration_minutes = 30;
    config.security.password_min_length = 8;
    config.security.password_require_symbols = true;
    config.security.password_require_uppercase = true;
    config.security.password_require_numbers = true;
    
    let auth_service = AuthServiceImpl::new(db_pool.clone(), config.clone())?;
    
    let app_state = AppState {
        db_pool,
        config: Arc::new(config),
        auth_service: Arc::new(auth_service),
    };
    
    let app = create_router(app_state);
    Ok(TestServer::new(app)?)
}

#[tokio::test]
async fn test_sql_injection_prevention() {
    let server = create_secure_test_server().await
        .expect("Failed to create test server");
    
    // Common SQL injection attempts
    let sql_injection_attempts = vec![
        "admin'--",
        "' OR '1'='1",
        "'; DROP TABLE users; --",
        "1' UNION SELECT * FROM users--",
        "' OR 1=1--",
        "admin' /*",
        "' OR 'x'='x",
        "1' AND '1'='1",
    ];
    
    for injection in sql_injection_attempts {
        // Try injection in login
        let login_request = json!({
            "email": injection,
            "password": "password123",
            "remember_me": false
        });
        
        let response = server
            .post("/auth/login")
            .json(&login_request)
            .await;
        
        // Should get validation error or auth failure, not SQL error
        assert!(response.status_code() == 400 || response.status_code() == 401);
        
        // Try injection in registration
        let register_request = json!({
            "first_name": injection,
            "last_name": "Test",
            "email": format!("test{}@example.com", Uuid::new_v4()),
            "password": "SecurePassword123!"
        });
        
        let response = server
            .post("/auth/register")
            .json(&register_request)
            .await;
        
        // Should either succeed (data is properly escaped) or validation fail
        // But should never cause SQL error
        let status = response.status_code();
        assert!(status == 201 || status == 400);
    }
}

#[tokio::test]
async fn test_xss_prevention() {
    let server = create_secure_test_server().await
        .expect("Failed to create test server");
    
    let xss_attempts = vec![
        "<script>alert('XSS')</script>",
        "javascript:alert('XSS')",
        "<img src=x onerror=alert('XSS')>",
        "<iframe src='javascript:alert(\"XSS\")'></iframe>",
        "<svg onload=alert('XSS')>",
        "';alert(String.fromCharCode(88,83,83))//",
        "<input type=\"text\" onfocus=\"alert('XSS')\">",
        "<body onload=alert('XSS')>",
    ];
    
    for xss in xss_attempts {
        // Register user with XSS attempt in name
        let register_request = json!({
            "first_name": xss,
            "last_name": "Safe",
            "email": format!("xss{}@example.com", Uuid::new_v4()),
            "password": "SecurePassword123!"
        });
        
        let response = server
            .post("/auth/register")
            .json(&register_request)
            .await;
        
        if response.status_code() == 201 {
            let body: serde_json::Value = response.json();
            
            // Login to get token
            let login_request = json!({
                "email": register_request["email"],
                "password": "SecurePassword123!",
                "remember_me": false
            });
            
            let login_response = server
                .post("/auth/login")
                .json(&login_request)
                .await;
            
            let login_body: serde_json::Value = login_response.json();
            let token = login_body["data"]["access_token"].as_str().unwrap();
            
            // Get user profile
            let profile_response = server
                .get("/auth/me")
                .add_header("Authorization", &format!("Bearer {}", token))
                .await;
            
            let profile_body: serde_json::Value = profile_response.json();
            let first_name = profile_body["data"]["first_name"].as_str().unwrap();
            
            // Verify XSS is escaped/sanitized
            assert!(!first_name.contains("<script>"));
            assert!(!first_name.contains("javascript:"));
            
            // Check if it's properly encoded
            if first_name.contains("&lt;") || first_name.contains("&gt;") {
                // Good - HTML entities are encoded
                continue;
            }
        }
    }
}

#[tokio::test]
async fn test_password_security_requirements() {
    let server = create_secure_test_server().await
        .expect("Failed to create test server");
    
    let weak_passwords = vec![
        ("", "Password cannot be empty"),
        ("123456", "too short"),
        ("password", "uppercase"),
        ("PASSWORD", "lowercase"),
        ("Password", "number"),
        ("Password1", "special"),
        ("Pass1!", "too short"),
        ("        ", "empty"),
        ("12345678", "letter"),
    ];
    
    for (password, _expected_issue) in weak_passwords {
        let register_request = json!({
            "first_name": "Test",
            "last_name": "User",
            "email": format!("weak{}@example.com", Uuid::new_v4()),
            "password": password
        });
        
        let response = server
            .post("/auth/register")
            .json(&register_request)
            .await;
        
        // Weak passwords should be rejected
        assert_eq!(response.status_code(), 400);
        
        let body: serde_json::Value = response.json();
        // Should have validation error
        assert!(body["error"].is_object() || body["error"].is_string());
    }
    
    // Test strong password acceptance
    let strong_passwords = vec![
        "SecureP@ss123",
        "C0mpl3x!Pass",
        "Str0ng#Password",
        "P@ssw0rd123!",
        "MyS3cur3P@ss!",
    ];
    
    for password in strong_passwords {
        let register_request = json!({
            "first_name": "Test",
            "last_name": "User",
            "email": format!("strong{}@example.com", Uuid::new_v4()),
            "password": password
        });
        
        let response = server
            .post("/auth/register")
            .json(&register_request)
            .await;
        
        // Strong passwords should be accepted
        assert_eq!(response.status_code(), 201);
    }
}

#[tokio::test]
async fn test_brute_force_protection() {
    let server = create_secure_test_server().await
        .expect("Failed to create test server");
    
    let email = format!("bruteforce{}@example.com", Uuid::new_v4());
    
    // Register user
    let register_request = json!({
        "first_name": "Brute",
        "last_name": "Force",
        "email": email,
        "password": "CorrectP@ss123!"
    });
    
    server
        .post("/auth/register")
        .json(&register_request)
        .await;
    
    // Make multiple failed login attempts
    let mut locked = false;
    for i in 0..5 {
        let login_request = json!({
            "email": email,
            "password": format!("WrongPass{}!", i),
            "remember_me": false
        });
        
        let response = server
            .post("/auth/login")
            .json(&login_request)
            .await;
        
        let body: serde_json::Value = response.json();
        
        // Check if account is locked
        if let Some(error_msg) = body.get("error")
            .and_then(|e| e.as_object())
            .and_then(|e| e.get("message"))
            .and_then(|m| m.as_str()) 
        {
            if error_msg.to_lowercase().contains("locked") ||
               error_msg.to_lowercase().contains("too many") {
                locked = true;
                break;
            }
        }
    }
    
    assert!(locked, "Account should be locked after multiple failed attempts");
    
    // Even correct password should fail when locked
    let correct_login = json!({
        "email": email,
        "password": "CorrectP@ss123!",
        "remember_me": false
    });
    
    let response = server
        .post("/auth/login")
        .json(&correct_login)
        .await;
    
    assert!(response.status_code() == 401 || response.status_code() == 423);
}

#[tokio::test]
async fn test_jwt_tampering_detection() {
    let server = create_secure_test_server().await
        .expect("Failed to create test server");
    
    let email = format!("jwt{}@example.com", Uuid::new_v4());
    
    // Register and login
    let register_request = json!({
        "first_name": "JWT",
        "last_name": "Test",
        "email": email,
        "password": "SecureP@ss123!"
    });
    
    server
        .post("/auth/register")
        .json(&register_request)
        .await;
    
    let login_request = json!({
        "email": email,
        "password": "SecureP@ss123!",
        "remember_me": false
    });
    
    let response = server
        .post("/auth/login")
        .json(&login_request)
        .await;
    
    let body: serde_json::Value = response.json();
    let valid_token = body["data"]["access_token"].as_str().unwrap();
    
    // Test various tampered tokens
    let modified_token = format!("{}XXX", valid_token);
    let tampered_tokens = vec![
        // Modified payload
        &valid_token[..valid_token.len()-10],
        // Invalid signature
        modified_token.as_str(),
        // Completely invalid
        "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9.invalid.signature",
        // Empty
        "",
        // Just random string
        "not-a-jwt-token",
        // Modified header
        "eyJhbGciOiJub25lIiwidHlwIjoiSldUIn0.eyJzdWIiOiIxMjM0NTY3ODkwIn0.",
    ];
    
    for tampered in tampered_tokens {
        let response = server
            .get("/auth/me")
            .add_header("Authorization", &format!("Bearer {}", tampered))
            .await;
        
        // Should reject tampered tokens
        assert_eq!(response.status_code(), 401);
    }
}

#[tokio::test]
async fn test_session_hijacking_prevention() {
    let server = create_secure_test_server().await
        .expect("Failed to create test server");
    
    let email = format!("hijack{}@example.com", Uuid::new_v4());
    
    // Register user
    let register_request = json!({
        "first_name": "Session",
        "last_name": "Test",
        "email": email,
        "password": "SecureP@ss123!"
    });
    
    server
        .post("/auth/register")
        .json(&register_request)
        .await;
    
    // Login from "device 1"
    let login_request = json!({
        "email": email,
        "password": "SecureP@ss123!",
        "remember_me": false
    });
    
    let response1 = server
        .post("/auth/login")
        .add_header("User-Agent", "Device1/1.0")
        .json(&login_request)
        .await;
    
    let body1: serde_json::Value = response1.json();
    let token1 = body1["data"]["access_token"].as_str().unwrap();
    let session_id1 = body1["data"]["session_id"].as_str().unwrap();
    
    // Login from "device 2"
    let response2 = server
        .post("/auth/login")
        .add_header("User-Agent", "Device2/1.0")
        .json(&login_request)
        .await;
    
    let body2: serde_json::Value = response2.json();
    let token2 = body2["data"]["access_token"].as_str().unwrap();
    
    // Both tokens should work
    assert_eq!(
        server.get("/auth/me")
            .add_header("Authorization", &format!("Bearer {}", token1))
            .await
            .status_code(),
        200
    );
    
    assert_eq!(
        server.get("/auth/me")
            .add_header("Authorization", &format!("Bearer {}", token2))
            .await
            .status_code(),
        200
    );
    
    // User can see all their sessions
    let sessions_response = server
        .get("/auth/sessions")
        .add_header("Authorization", &format!("Bearer {}", token1))
        .await;
    
    assert_eq!(sessions_response.status_code(), 200);
    let sessions_body: serde_json::Value = sessions_response.json();
    let sessions = sessions_body["data"].as_array().unwrap();
    assert!(sessions.len() >= 2);
    
    // User can revoke suspicious session
    let revoke_response = server
        .delete(&format!("/auth/sessions/{}", session_id1))
        .add_header("Authorization", &format!("Bearer {}", token2))
        .await;
    
    assert_eq!(revoke_response.status_code(), 200);
    
    // Revoked token should no longer work
    assert_eq!(
        server.get("/auth/me")
            .add_header("Authorization", &format!("Bearer {}", token1))
            .await
            .status_code(),
        401
    );
}

#[tokio::test]
async fn test_authorization_bypass_prevention() {
    let server = create_secure_test_server().await
        .expect("Failed to create test server");
    
    // Create regular user
    let user_email = format!("user{}@example.com", Uuid::new_v4());
    let register_request = json!({
        "first_name": "Regular",
        "last_name": "User",
        "email": user_email,
        "password": "SecureP@ss123!",
        "role": "guest"
    });
    
    server
        .post("/auth/register")
        .json(&register_request)
        .await;
    
    let login_request = json!({
        "email": user_email,
        "password": "SecureP@ss123!",
        "remember_me": false
    });
    
    let response = server
        .post("/auth/login")
        .json(&login_request)
        .await;
    
    let body: serde_json::Value = response.json();
    let user_token = body["data"]["access_token"].as_str().unwrap();
    
    // Try to access admin endpoints
    let admin_endpoints = vec![
        ("/admin/users", "GET"),
        ("/admin/users/123", "GET"),
        ("/admin/users/123", "DELETE"),
        ("/admin/users/123/disable", "POST"),
        ("/admin/sessions", "GET"),
        ("/admin/audit-log", "GET"),
    ];
    
    for (endpoint, method) in admin_endpoints {
        let response = match method {
            "GET" => server.get(endpoint)
                .add_header("Authorization", &format!("Bearer {}", user_token))
                .await,
            "POST" => server.post(endpoint)
                .add_header("Authorization", &format!("Bearer {}", user_token))
                .await,
            "DELETE" => server.delete(endpoint)
                .add_header("Authorization", &format!("Bearer {}", user_token))
                .await,
            _ => panic!("Unsupported method"),
        };
        
        // Should be forbidden for non-admin users
        assert_eq!(response.status_code(), 403);
    }
}

#[tokio::test]
async fn test_timing_attack_prevention() {
    let server = create_secure_test_server().await
        .expect("Failed to create test server");
    
    // Register a user
    let existing_email = format!("timing{}@example.com", Uuid::new_v4());
    let register_request = json!({
        "first_name": "Timing",
        "last_name": "Test",
        "email": existing_email,
        "password": "SecureP@ss123!"
    });
    
    server
        .post("/auth/register")
        .json(&register_request)
        .await;
    
    // Measure login times for existing vs non-existing users
    let mut existing_times = vec![];
    let mut non_existing_times = vec![];
    
    for _ in 0..5 {
        // Time for existing user (wrong password)
        let start = std::time::Instant::now();
        server
            .post("/auth/login")
            .json(&json!({
                "email": existing_email,
                "password": "WrongPassword123!",
                "remember_me": false
            }))
            .await;
        existing_times.push(start.elapsed());
        
        // Time for non-existing user
        let start = std::time::Instant::now();
        server
            .post("/auth/login")
            .json(&json!({
                "email": format!("nonexist{}@example.com", Uuid::new_v4()),
                "password": "WrongPassword123!",
                "remember_me": false
            }))
            .await;
        non_existing_times.push(start.elapsed());
    }
    
    // Calculate average times
    let avg_existing: f64 = existing_times.iter()
        .map(|d| d.as_millis() as f64)
        .sum::<f64>() / existing_times.len() as f64;
    
    let avg_non_existing: f64 = non_existing_times.iter()
        .map(|d| d.as_millis() as f64)
        .sum::<f64>() / non_existing_times.len() as f64;
    
    // Times should be similar (within 50ms) to prevent timing attacks
    let time_diff = (avg_existing - avg_non_existing).abs();
    assert!(time_diff < 50.0, 
        "Timing difference too large: {}ms (existing: {}ms, non-existing: {}ms)", 
        time_diff, avg_existing, avg_non_existing);
} 
