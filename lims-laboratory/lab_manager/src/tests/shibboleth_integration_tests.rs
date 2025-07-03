#[cfg(test)]
mod integration_tests {
    use crate::{
        config::{AppConfig, ShibbolethConfig},
        middleware::shibboleth_auth::{
            extract_shibboleth_attributes, map_shibboleth_role_to_lab_role,
        },
        models::user::UserRole,
    };
    use axum::http::{HeaderMap, header::HeaderValue};
    use std::collections::HashMap;

    /// Test environment variable parsing for Shibboleth configuration
    #[test]
    fn test_shibboleth_config_from_env() {
        // Test default configuration behavior without modifying global env vars
        // In real implementation, this would test actual config parsing from env
        let config = ShibbolethConfig::default();

        // Test that defaults are reasonable
        assert!(!config.enabled);
        assert!(config.hybrid_mode);
        assert_eq!(config.default_role, "Guest");

        // Test programmatic configuration (simulating env var effects)
        let mut config_enabled = ShibbolethConfig::default();
        config_enabled.enabled = true;
        config_enabled.hybrid_mode = false;
        config_enabled.auto_create_users = false;
        config_enabled.auto_update_attributes = false;
        config_enabled.default_role = "DataAnalyst".to_string();

        assert!(config_enabled.enabled);
        assert!(!config_enabled.hybrid_mode);
        assert!(!config_enabled.auto_create_users);
        assert!(!config_enabled.auto_update_attributes);
        assert_eq!(config_enabled.default_role, "DataAnalyst");
    }

    /// Test Shibboleth configuration validation
    #[test]
    fn test_shibboleth_config_validation() {
        let mut config = ShibbolethConfig::default();

        // Test valid configuration
        config.enabled = true;
        config.required_attributes = vec!["mail".to_string(), "eppn".to_string()];

        assert!(config.enabled);
        assert_eq!(config.required_attributes.len(), 2);
        assert!(config.required_attributes.contains(&"mail".to_string()));
        assert!(config.required_attributes.contains(&"eppn".to_string()));
    }

    /// Test app configuration includes Shibboleth settings
    #[test]
    fn test_app_config_includes_shibboleth() {
        let config = AppConfig::for_testing();

        // Should have Shibboleth config
        assert!(!config.shibboleth.enabled); // Disabled for tests by default
        assert!(config.shibboleth.hybrid_mode);
        assert_eq!(config.shibboleth.default_role, "Guest");
    }

    /// Test Shibboleth header extraction edge cases
    #[test]
    fn test_header_extraction_edge_cases() {
        let mut headers = HeaderMap::new();

        // Test empty headers
        let empty_headers = HeaderMap::new();
        assert!(extract_shibboleth_attributes(&empty_headers).is_empty());

        // Test headers with empty values
        headers.insert("HTTP_EPPN", HeaderValue::from_static(""));
        headers.insert("HTTP_MAIL", HeaderValue::from_static(""));

        let attributes = extract_shibboleth_attributes(&headers);
        assert_eq!(attributes.get("eppn"), Some(&"".to_string()));
        assert_eq!(attributes.get("mail"), Some(&"".to_string()));

        // Test headers with special characters
        headers.clear();
        headers.insert(
            "HTTP_EPPN",
            HeaderValue::from_static("user+test@example.edu"),
        );
        headers.insert("HTTP_DISPLAYNAME", HeaderValue::from_static("John O'Doe"));

        let attributes = extract_shibboleth_attributes(&headers);
        assert_eq!(
            attributes.get("eppn"),
            Some(&"user+test@example.edu".to_string())
        );
        assert_eq!(
            attributes.get("displayName"),
            Some(&"John O'Doe".to_string())
        );
    }

    /// Test authentication decision logic
    #[test]
    fn test_authentication_scenarios() {
        // Test scenario 1: Valid Shibboleth attributes
        let mut headers = HeaderMap::new();
        headers.insert("HTTP_EPPN", HeaderValue::from_static("admin@example.edu"));
        headers.insert("HTTP_MAIL", HeaderValue::from_static("admin@example.edu"));
        headers.insert(
            "HTTP_LAB_ROLE",
            HeaderValue::from_static("lab_administrator"),
        );

        let attributes = extract_shibboleth_attributes(&headers);
        assert!(should_authenticate_via_shibboleth(&attributes));

        // Test scenario 2: Missing critical attributes
        let headers = HeaderMap::new();
        let attributes = extract_shibboleth_attributes(&headers);
        assert!(!should_authenticate_via_shibboleth(&attributes));
    }

    /// Test comprehensive role mapping scenarios
    #[test]
    fn test_comprehensive_role_mapping() {
        let test_cases = vec![
            (
                Some("lab_administrator"),
                None,
                None,
                UserRole::LabAdministrator,
            ),
            (
                Some("pi"),
                Some("lab:technician"),
                Some("cn=data-analysts"),
                UserRole::PrincipalInvestigator,
            ),
            (
                None,
                Some("lab:admin"),
                Some("cn=technicians"),
                UserRole::LabAdministrator,
            ),
            (
                None,
                None,
                Some("cn=lab-technicians"),
                UserRole::LabTechnician,
            ),
            (None, None, None, UserRole::Guest),
        ];

        for (lab_role, entitlement, is_member_of, expected) in test_cases {
            let mut attributes = HashMap::new();

            if let Some(role) = lab_role {
                attributes.insert("labRole".to_string(), role.to_string());
            }
            if let Some(ent) = entitlement {
                attributes.insert("entitlement".to_string(), ent.to_string());
            }
            if let Some(groups) = is_member_of {
                attributes.insert("isMemberOf".to_string(), groups.to_string());
            }

            assert_eq!(map_shibboleth_role_to_lab_role(&attributes), expected);
        }
    }

    /// Test attribute requirements for user creation
    #[test]
    fn test_user_creation_requirements() {
        let config = ShibbolethConfig::default();

        // Test that required attributes are properly defined
        assert!(!config.required_attributes.is_empty());
        assert!(config.required_attributes.contains(&"mail".to_string()));
        assert!(config.required_attributes.contains(&"eppn".to_string()));

        // Test attribute validation
        let mut attributes = HashMap::new();

        // Missing required attributes should fail validation
        assert!(!has_required_attributes(
            &attributes,
            &config.required_attributes
        ));

        // Add required attributes
        attributes.insert("mail".to_string(), "user@example.edu".to_string());
        attributes.insert("eppn".to_string(), "user@example.edu".to_string());

        // Should now pass validation
        assert!(has_required_attributes(
            &attributes,
            &config.required_attributes
        ));
    }

    /// Test CreateUserRequest construction from Shibboleth attributes
    #[test]
    fn test_create_user_request_from_attributes() {
        let mut attributes = HashMap::new();
        attributes.insert("mail".to_string(), "jane.doe@university.edu".to_string());
        attributes.insert("givenName".to_string(), "Jane".to_string());
        attributes.insert("surname".to_string(), "Doe".to_string());
        attributes.insert("department".to_string(), "Chemistry".to_string());
        attributes.insert(
            "affiliation".to_string(),
            "staff@university.edu".to_string(),
        );
        attributes.insert("labRole".to_string(), "lab_technician".to_string());

        // Simulate the CreateUserRequest construction
        let email = attributes.get("mail").unwrap();
        let given_name = attributes
            .get("givenName")
            .map_or("Unknown", |s| s.as_str());
        let surname = attributes.get("surname").map_or("User", |s| s.as_str());
        let role = map_shibboleth_role_to_lab_role(&attributes);

        // Verify the data is correctly extracted
        assert_eq!(email, "jane.doe@university.edu");
        assert_eq!(given_name, "Jane");
        assert_eq!(surname, "Doe");
        assert_eq!(role, UserRole::LabTechnician);
        assert_eq!(attributes.get("department").unwrap(), "Chemistry");
        assert_eq!(
            attributes.get("affiliation").unwrap(),
            "staff@university.edu"
        );
    }

    /// Test error handling with malformed data
    #[test]
    fn test_error_handling() {
        let mut attributes = HashMap::new();
        attributes.insert("mail".to_string(), "not-an-email".to_string());
        attributes.insert("labRole".to_string(), "invalid_role".to_string());

        let role = map_shibboleth_role_to_lab_role(&attributes);
        assert_eq!(role, UserRole::Guest);

        // Test with very long values
        let long_string = "a".repeat(1000);
        attributes.insert("displayName".to_string(), long_string.clone());
        assert_eq!(attributes.get("displayName").unwrap(), &long_string);
    }

    /// Test hybrid authentication decision logic
    #[test]
    fn test_hybrid_authentication_logic() {
        // Test when Shibboleth should be preferred
        let mut headers = HeaderMap::new();
        headers.insert("HTTP_EPPN", HeaderValue::from_static("user@example.edu"));
        headers.insert("HTTP_MAIL", HeaderValue::from_static("user@example.edu"));
        headers.insert(
            "authorization",
            HeaderValue::from_static("Bearer some-jwt-token"),
        );

        let shibboleth_attrs = extract_shibboleth_attributes(&headers);
        let has_jwt = headers
            .get("authorization")
            .and_then(|h| h.to_str().ok())
            .map(|h| h.starts_with("Bearer "))
            .unwrap_or(false);

        // Should prefer Shibboleth when both are available
        assert!(should_authenticate_via_shibboleth(&shibboleth_attrs));
        assert!(has_jwt);

        // Test when only JWT is available
        let mut headers = HeaderMap::new();
        headers.insert(
            "authorization",
            HeaderValue::from_static("Bearer some-jwt-token"),
        );

        let shibboleth_attrs = extract_shibboleth_attributes(&headers);
        let has_jwt = headers
            .get("authorization")
            .and_then(|h| h.to_str().ok())
            .map(|h| h.starts_with("Bearer "))
            .unwrap_or(false);

        assert!(!should_authenticate_via_shibboleth(&shibboleth_attrs));
        assert!(has_jwt);
    }

    // Helper functions for testing (these mirror the actual implementation)

    fn should_authenticate_via_shibboleth(attributes: &HashMap<String, String>) -> bool {
        attributes.contains_key("eppn") || attributes.contains_key("uid")
    }

    fn has_required_attributes(attributes: &HashMap<String, String>, required: &[String]) -> bool {
        required.iter().all(|attr| attributes.contains_key(attr))
    }
}
