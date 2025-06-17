#[cfg(test)]
mod tests {
    use crate::{
        config::ShibbolethConfig,
        middleware::shibboleth_auth::{
            extract_shibboleth_attributes, map_shibboleth_role_to_lab_role,
        },
        models::user::UserRole,
    };
    use axum::http::{HeaderMap, HeaderValue};
    use std::collections::HashMap;

    /// Test Shibboleth attribute extraction from HTTP headers
    #[test]
    fn test_shibboleth_attribute_extraction() {
        let mut headers = HeaderMap::new();

        headers.insert("HTTP_EPPN", HeaderValue::from_static("user@example.edu"));
        headers.insert("HTTP_MAIL", HeaderValue::from_static("user@example.edu"));
        headers.insert("HTTP_DISPLAYNAME", HeaderValue::from_static("John Doe"));
        headers.insert("HTTP_GIVENNAME", HeaderValue::from_static("John"));
        headers.insert("HTTP_SN", HeaderValue::from_static("Doe"));
        headers.insert(
            "HTTP_LAB_ROLE",
            HeaderValue::from_static("lab_administrator"),
        );
        headers.insert("HTTP_DEPARTMENT", HeaderValue::from_static("Biology"));

        let attributes = extract_shibboleth_attributes(&headers);

        assert_eq!(
            attributes.get("eppn"),
            Some(&"user@example.edu".to_string())
        );
        assert_eq!(
            attributes.get("mail"),
            Some(&"user@example.edu".to_string())
        );
        assert_eq!(attributes.get("displayName"), Some(&"John Doe".to_string()));
        assert_eq!(
            attributes.get("labRole"),
            Some(&"lab_administrator".to_string())
        );
        assert_eq!(attributes.get("department"), Some(&"Biology".to_string()));
    }

    /// Test role mapping from direct lab role attribute
    #[test]
    fn test_direct_role_mapping() {
        let mut attributes = HashMap::new();

        attributes.insert("labRole".to_string(), "lab_administrator".to_string());
        assert_eq!(
            map_shibboleth_role_to_lab_role(&attributes),
            UserRole::LabAdministrator
        );

        attributes.insert("labRole".to_string(), "principal_investigator".to_string());
        assert_eq!(
            map_shibboleth_role_to_lab_role(&attributes),
            UserRole::PrincipalInvestigator
        );

        attributes.insert("labRole".to_string(), "pi".to_string());
        assert_eq!(
            map_shibboleth_role_to_lab_role(&attributes),
            UserRole::PrincipalInvestigator
        );

        attributes.insert("labRole".to_string(), "lab_technician".to_string());
        assert_eq!(
            map_shibboleth_role_to_lab_role(&attributes),
            UserRole::LabTechnician
        );

        attributes.insert("labRole".to_string(), "research_scientist".to_string());
        assert_eq!(
            map_shibboleth_role_to_lab_role(&attributes),
            UserRole::ResearchScientist
        );

        attributes.insert("labRole".to_string(), "data_analyst".to_string());
        assert_eq!(
            map_shibboleth_role_to_lab_role(&attributes),
            UserRole::DataAnalyst
        );
    }

    /// Test role mapping from entitlements
    #[test]
    fn test_entitlement_role_mapping() {
        let mut attributes = HashMap::new();

        attributes.insert("entitlement".to_string(), "lab:admin".to_string());
        assert_eq!(
            map_shibboleth_role_to_lab_role(&attributes),
            UserRole::LabAdministrator
        );

        attributes.clear();
        attributes.insert(
            "entitlement".to_string(),
            "lab:pi;other:permission".to_string(),
        );
        assert_eq!(
            map_shibboleth_role_to_lab_role(&attributes),
            UserRole::PrincipalInvestigator
        );

        attributes.clear();
        attributes.insert("entitlement".to_string(), "lab:technician".to_string());
        assert_eq!(
            map_shibboleth_role_to_lab_role(&attributes),
            UserRole::LabTechnician
        );
    }

    /// Test role mapping from group memberships
    #[test]
    fn test_group_membership_role_mapping() {
        let mut attributes = HashMap::new();

        attributes.insert(
            "isMemberOf".to_string(),
            "cn=lab-administrators,ou=groups,dc=example,dc=edu".to_string(),
        );
        assert_eq!(
            map_shibboleth_role_to_lab_role(&attributes),
            UserRole::LabAdministrator
        );

        attributes.clear();
        attributes.insert(
            "isMemberOf".to_string(),
            "cn=principal-investigators,ou=groups,dc=example,dc=edu".to_string(),
        );
        assert_eq!(
            map_shibboleth_role_to_lab_role(&attributes),
            UserRole::PrincipalInvestigator
        );

        attributes.clear();
        attributes.insert(
            "isMemberOf".to_string(),
            "cn=lab-technicians,ou=groups,dc=example,dc=edu".to_string(),
        );
        assert_eq!(
            map_shibboleth_role_to_lab_role(&attributes),
            UserRole::LabTechnician
        );
    }

    /// Test role mapping precedence
    #[test]
    fn test_role_mapping_precedence() {
        let mut attributes = HashMap::new();

        // Add conflicting role information
        attributes.insert("labRole".to_string(), "lab_administrator".to_string());
        attributes.insert("entitlement".to_string(), "lab:technician".to_string());
        attributes.insert(
            "isMemberOf".to_string(),
            "cn=data-analysts,ou=groups,dc=example,dc=edu".to_string(),
        );

        // Should prefer direct labRole attribute
        assert_eq!(
            map_shibboleth_role_to_lab_role(&attributes),
            UserRole::LabAdministrator
        );

        // Remove labRole, should fall back to entitlement
        attributes.remove("labRole");
        assert_eq!(
            map_shibboleth_role_to_lab_role(&attributes),
            UserRole::LabTechnician
        );

        // Remove entitlement, should fall back to group
        attributes.remove("entitlement");
        assert_eq!(
            map_shibboleth_role_to_lab_role(&attributes),
            UserRole::DataAnalyst
        );

        // Remove all, should default to Guest
        attributes.remove("isMemberOf");
        assert_eq!(
            map_shibboleth_role_to_lab_role(&attributes),
            UserRole::Guest
        );
    }

    /// Test Shibboleth configuration defaults
    #[test]
    fn test_shibboleth_config_defaults() {
        let config = ShibbolethConfig::default();

        assert!(!config.enabled);
        assert!(config.hybrid_mode);
        assert_eq!(config.login_path, "/shibboleth-login");
        assert_eq!(config.success_redirect, "/dashboard");
        assert_eq!(config.logout_redirect, "/");
        assert!(config.auto_create_users);
        assert!(config.auto_update_attributes);
        assert_eq!(config.default_role, "Guest");
        assert_eq!(config.required_attributes, vec!["mail", "eppn"]);

        // Check some default attribute mappings
        assert_eq!(
            config.attribute_mappings.get("eppn"),
            Some(&"HTTP_EPPN".to_string())
        );
        assert_eq!(
            config.attribute_mappings.get("mail"),
            Some(&"HTTP_MAIL".to_string())
        );
    }

    /// Test case sensitivity in role mapping
    #[test]
    fn test_case_insensitive_role_mapping() {
        let mut attributes = HashMap::new();

        // Test uppercase
        attributes.insert("labRole".to_string(), "LAB_ADMINISTRATOR".to_string());
        assert_eq!(
            map_shibboleth_role_to_lab_role(&attributes),
            UserRole::LabAdministrator
        );

        // Test mixed case
        attributes.insert("labRole".to_string(), "Principal_Investigator".to_string());
        assert_eq!(
            map_shibboleth_role_to_lab_role(&attributes),
            UserRole::PrincipalInvestigator
        );

        // Test entitlement case insensitivity
        attributes.clear();
        attributes.insert("entitlement".to_string(), "LAB:ADMIN".to_string());
        assert_eq!(
            map_shibboleth_role_to_lab_role(&attributes),
            UserRole::LabAdministrator
        );
    }

    /// Test missing attributes handling
    #[test]
    fn test_missing_attributes_handling() {
        let mut headers = HeaderMap::new();

        // Only add minimal attributes
        headers.insert("HTTP_EPPN", HeaderValue::from_static("user@example.edu"));
        headers.insert("HTTP_MAIL", HeaderValue::from_static("user@example.edu"));

        let attributes = extract_shibboleth_attributes(&headers);

        // Should have the basic attributes
        assert!(attributes.contains_key("eppn"));
        assert!(attributes.contains_key("mail"));

        // Should not have optional attributes
        assert!(!attributes.contains_key("displayName"));
        assert!(!attributes.contains_key("department"));
        assert!(!attributes.contains_key("labRole"));

        // Role mapping should default to Guest
        assert_eq!(
            map_shibboleth_role_to_lab_role(&attributes),
            UserRole::Guest
        );
    }

    /// Test empty attributes scenario
    #[test]
    fn test_empty_attributes() {
        let attributes = HashMap::new();

        // Should default to Guest role
        assert_eq!(
            map_shibboleth_role_to_lab_role(&attributes),
            UserRole::Guest
        );
    }

    /// Test multiple entitlements handling
    #[test]
    fn test_multiple_entitlements() {
        let mut attributes = HashMap::new();

        // Test multiple entitlements with lab:admin first
        attributes.insert(
            "entitlement".to_string(),
            "lab:admin;urn:mace:dir:entitlement:common-lib-terms;lab:user".to_string(),
        );
        assert_eq!(
            map_shibboleth_role_to_lab_role(&attributes),
            UserRole::LabAdministrator
        );

        // Test with lab:technician in the middle
        attributes.insert(
            "entitlement".to_string(),
            "urn:mace:example;lab:technician;other:permission".to_string(),
        );
        assert_eq!(
            map_shibboleth_role_to_lab_role(&attributes),
            UserRole::LabTechnician
        );
    }

    /// Test user creation from Shibboleth attributes simulation
    #[test]
    fn test_user_creation_attributes() {
        let mut attributes = HashMap::new();
        attributes.insert("mail".to_string(), "john.doe@example.edu".to_string());
        attributes.insert("givenName".to_string(), "John".to_string());
        attributes.insert("surname".to_string(), "Doe".to_string());
        attributes.insert("displayName".to_string(), "John Doe".to_string());
        attributes.insert("department".to_string(), "Biology".to_string());
        attributes.insert("affiliation".to_string(), "faculty@example.edu".to_string());
        attributes.insert("labRole".to_string(), "research_scientist".to_string());

        let email = attributes.get("mail").unwrap();
        let given_name = attributes
            .get("givenName")
            .map_or("Unknown", |s| s.as_str());
        let surname = attributes.get("surname").map_or("User", |s| s.as_str());
        let role = map_shibboleth_role_to_lab_role(&attributes);

        assert_eq!(email, "john.doe@example.edu");
        assert_eq!(given_name, "John");
        assert_eq!(surname, "Doe");
        assert_eq!(role, UserRole::ResearchScientist);

        // Verify we have the necessary attributes for user creation
        assert!(attributes.contains_key("mail"));
        assert!(attributes.contains_key("givenName"));
        assert!(attributes.contains_key("surname"));
        assert!(attributes.contains_key("department"));
        assert!(attributes.contains_key("affiliation"));
    }
}
