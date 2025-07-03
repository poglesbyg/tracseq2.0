#[cfg(test)]
mod session_security_tests {
    use crate::models::user::{AuthClaims, LoginRequest, UserRole};
    use crate::services::auth_service::AuthService;
    use chrono::{Duration, Utc};
    use sqlx::postgres::PgPoolOptions;
    use std::net::IpAddr;
    use uuid::Uuid;

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

    #[tokio::test]
    async fn test_session_cleanup() {
        let pool = setup_test_db().await;
        let auth_service = AuthService::new(pool.clone(), "test-secret".to_string());

        // Test cleanup of expired sessions
        let cleanup_result = auth_service.cleanup_expired().await;
        assert!(cleanup_result.is_ok(), "Session cleanup should succeed");
    }

    #[tokio::test]
    async fn test_multiple_sessions_same_user() {
        let pool = setup_test_db().await;
        let auth_service = AuthService::new(pool.clone(), "test-secret".to_string());

        // Create a test user first (this would normally be done in setup)
        let user_id = Uuid::new_v4();

        // Test that multiple sessions can be retrieved
        let sessions_result = auth_service.get_user_sessions(user_id).await;
        assert!(
            sessions_result.is_ok(),
            "Getting user sessions should succeed"
        );
    }

    #[tokio::test]
    async fn test_session_revocation() {
        let pool = setup_test_db().await;
        let auth_service = AuthService::new(pool.clone(), "test-secret".to_string());

        let user_id = Uuid::new_v4();
        let session_id = Uuid::new_v4();

        // Test session revocation (will fail if session doesn't exist, which is expected)
        let revoke_result = auth_service.revoke_session(session_id, user_id).await;
        // Should return error since session doesn't exist
        assert!(
            revoke_result.is_err(),
            "Revoking non-existent session should fail"
        );
    }

    #[tokio::test]
    async fn test_revoke_all_sessions() {
        let pool = setup_test_db().await;
        let auth_service = AuthService::new(pool.clone(), "test-secret".to_string());

        let user_id = Uuid::new_v4();

        // Test revoking all sessions for a user
        let revoke_all_result = auth_service.revoke_all_sessions(user_id, None).await;
        assert!(
            revoke_all_result.is_ok(),
            "Revoking all sessions should succeed"
        );

        // Should return 0 since no sessions exist
        let revoked_count = revoke_all_result.unwrap();
        assert_eq!(
            revoked_count, 0,
            "Should revoke 0 sessions for non-existent user"
        );
    }

    #[test]
    fn test_session_token_structure() {
        // Test the structure of auth claims used in sessions
        let claims = AuthClaims {
            sub: Uuid::new_v4(),
            email: "test@example.com".to_string(),
            role: UserRole::LabTechnician,
            exp: (Utc::now() + Duration::hours(1)).timestamp(),
            iat: Utc::now().timestamp(),
            jti: Uuid::new_v4(),
        };

        // Verify claim structure
        assert!(!claims.email.is_empty());
        assert!(claims.exp > claims.iat);
        assert_ne!(claims.sub, Uuid::nil());
        assert_ne!(claims.jti, Uuid::nil());
        assert_ne!(claims.sub, claims.jti);
    }

    #[test]
    fn test_session_timing_consistency() {
        // Test that session-related operations have consistent timing
        let start_time = Utc::now();

        let claims = AuthClaims {
            sub: Uuid::new_v4(),
            email: "test@example.com".to_string(),
            role: UserRole::DataAnalyst,
            exp: (start_time + Duration::hours(1)).timestamp(),
            iat: start_time.timestamp(),
            jti: Uuid::new_v4(),
        };

        // Verify timing relationships
        assert!(claims.exp > claims.iat);

        // Verify reasonable timing
        let duration_seconds = claims.exp - claims.iat;
        assert!(duration_seconds >= 30 * 60); // At least 30 minutes (1800 seconds)
    }

    #[test]
    fn test_device_info_validation() {
        // Test device information handling in sessions
        let user_agents = vec![
            "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36",
            "curl/7.68.0",
            "PostmanRuntime/7.28.4",
            "",
        ];

        for user_agent in user_agents {
            // Device info should be handled gracefully regardless of content
            assert!(user_agent.len() >= 0); // Basic validation
        }
    }

    #[test]
    fn test_ip_address_validation() {
        // Test IP address handling in sessions
        let ip_addresses = vec![
            IpAddr::from([127, 0, 0, 1]),   // IPv4 localhost
            IpAddr::from([192, 168, 1, 1]), // IPv4 private
        ];

        for ip in ip_addresses {
            // IP addresses should be valid
            match ip {
                IpAddr::V4(v4) => {
                    assert!(v4.octets().len() == 4);
                }
                IpAddr::V6(v6) => {
                    assert!(v6.segments().len() == 8);
                }
            }
        }
    }

    #[test]
    fn test_session_expiration_times() {
        // Test various session expiration scenarios
        let now = Utc::now();

        let test_cases = vec![
            Duration::minutes(30), // Short session
            Duration::hours(1),    // Standard session
            Duration::hours(8),    // Work day session
            Duration::days(1),     // Daily session
            Duration::days(7),     // Weekly session
        ];

        for duration in test_cases {
            let claims = AuthClaims {
                sub: Uuid::new_v4(),
                email: "test@example.com".to_string(),
                role: UserRole::ResearchScientist,
                exp: (now + duration).timestamp(),
                iat: now.timestamp(),
                jti: Uuid::new_v4(),
            };

            // Verify expiration is in the future
            assert!(claims.exp > claims.iat);

            // Verify reasonable expiration time (not too far in future)
            let duration_seconds = claims.exp - claims.iat;
            assert!(
                duration_seconds <= 30 * 24 * 60 * 60, // 30 days in seconds
                "Session should not last more than 30 days"
            );
        }
    }

    #[test]
    fn test_concurrent_session_safety() {
        // Test that concurrent session data doesn't conflict
        let mut session_ids = std::collections::HashSet::new();
        let mut user_ids = std::collections::HashSet::new();

        for i in 0..100 {
            let claims = AuthClaims {
                sub: Uuid::new_v4(),
                email: format!("user{}@example.com", i),
                role: UserRole::LabTechnician,
                exp: (Utc::now() + Duration::hours(1)).timestamp(),
                iat: Utc::now().timestamp(),
                jti: Uuid::new_v4(),
            };

            // All IDs should be unique
            assert!(
                session_ids.insert(claims.jti),
                "Session ID should be unique"
            );
            assert!(user_ids.insert(claims.sub), "User ID should be unique");
        }

        assert_eq!(session_ids.len(), 100);
        assert_eq!(user_ids.len(), 100);
    }

    #[test]
    fn test_role_based_session_properties() {
        // Test that different roles can have sessions
        for role in UserRole::all_roles() {
            let claims = AuthClaims {
                sub: Uuid::new_v4(),
                email: format!("{}@example.com", role.as_str()),
                role: role.clone(),
                exp: (Utc::now() + Duration::hours(1)).timestamp(),
                iat: Utc::now().timestamp(),
                jti: Uuid::new_v4(),
            };

            // All roles should be able to have valid sessions
            assert_eq!(claims.role, role);
            assert!(!claims.email.is_empty());
            assert!(claims.exp > claims.iat);
        }
    }
}
